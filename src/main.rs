use rustyline::{Config, DefaultEditor, error::ReadlineError};
use std::{
    env,
    process::{Command, ExitCode},
    sync::LazyLock,
};

static PLACEHOLDER: LazyLock<String> =
    LazyLock::new(|| env::var("REPL_PLACEHOLDER").unwrap_or("@".to_string()));
static PROMPT: LazyLock<String> =
    LazyLock::new(|| env::var("REPL_PROMPT").unwrap_or("> ".to_string()));

fn main() -> ExitCode {
    let mut args = env::args();

    if args.len() == 1 {
        let bin = args.next().unwrap();
        println!(
            "Usage: {} command arg...\n\n\
            Open read-eval-print loop for the command with the arg... arguments. \
            In the arguments, replace {} with the text provided through the REPL prompt, \
            if {} is not among the arguments, push the text at the end of the evaluated command. \
            For example:\n\n\
            {} sh -c 'echo $(({}))'\n\n\
            would start a REPL that evaluates arithmetic expressions using shell's $(()) \
            and print them using echo.\n\n\
            The {} placeholder can be changed using the REPL_PLACEHOLDER environment variable. \
            The REPL prompt can be customized using the REPL_PROMPT environment variable.",
            bin, *PLACEHOLDER, *PLACEHOLDER, bin, *PLACEHOLDER, *PLACEHOLDER,
        );
        return ExitCode::SUCCESS;
    }

    let cmd: String = args.nth(1).unwrap();
    let args = Args::new(args.map(Arg::parse).collect());

    let config = Config::builder().auto_add_history(true).build();
    let mut reader = match DefaultEditor::with_config(config) {
        Ok(editor) => editor,
        Err(msg) => {
            eprintln!("{}", msg);
            return ExitCode::FAILURE;
        }
    };

    print!("Use ^C to cancel command and ^D to exit\n\n");

    'outer: loop {
        let mut line = String::new();
        let mut prompt = PROMPT.to_string();
        let mut done = false;

        while !done {
            done = true;
            match reader.readline(&prompt) {
                Ok(mut s) => {
                    if s.ends_with('\\') {
                        done = false;
                        s.pop();
                    }
                    line.push_str(&s);
                    if s.ends_with('\\') {
                        done = true
                    } else if !done {
                        line.push('\n');
                    }
                }
                Err(ReadlineError::Interrupted) => continue 'outer,
                Err(ReadlineError::Eof) => break 'outer,
                Err(err) => {
                    eprintln!("{}", err);
                    return ExitCode::FAILURE;
                }
            }
            prompt.clear();
        }

        let interpolated = args.interpolate(&line);
        match Command::new(&cmd).args(&interpolated).output() {
            Ok(out) => {
                if out.status.success() {
                    print!("{}", String::from_utf8_lossy(&out.stdout));
                } else if !out.stderr.is_empty() {
                    print!("{}", String::from_utf8_lossy(&out.stderr));
                }
            }
            Err(err) => println!("{}", err),
        };
    }
    ExitCode::SUCCESS
}

#[derive(Debug)]
struct Args {
    args: Vec<Arg>,
}

impl Args {
    fn new(mut args: Vec<Arg>) -> Self {
        if !args
            .iter()
            .any(|a| matches!(a, Arg::Variable | Arg::Template(_)))
        {
            args.push(Arg::Variable);
        }
        Args { args }
    }

    fn interpolate(&self, value: &str) -> Vec<String> {
        let mut acc = Vec::new();
        for a in &self.args {
            let s = a.interpolate(value);
            acc.push(s);
        }
        acc
    }
}

#[derive(Debug)]
enum Arg {
    Literal(String),
    Variable,
    Template(Vec<Arg>),
}

impl Arg {
    fn parse(value: String) -> Arg {
        let mut acc = Vec::new();
        let mut literals = value.split(&*PLACEHOLDER);

        if let Some(s) = literals.nth(0) {
            if !s.is_empty() {
                acc.push(Arg::Literal(s.to_string()));
            }
        } else {
            return Arg::Variable;
        }

        for s in literals {
            acc.push(Arg::Variable);
            if !s.is_empty() {
                acc.push(Arg::Literal(s.to_string()));
            }
        }
        if acc.len() == 1 {
            acc.pop().unwrap()
        } else {
            Arg::Template(acc)
        }
    }

    fn interpolate(&self, value: &str) -> String {
        use Arg::*;
        match self {
            Literal(s) => s.to_string(),
            Variable => value.to_string(),
            Template(v) => v
                .iter()
                .map(|a| a.interpolate(value))
                .collect::<Vec<_>>()
                .join(""),
        }
    }
}
