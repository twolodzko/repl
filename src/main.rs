use rustyline::{Config, DefaultEditor, error::ReadlineError};
use std::{
    env,
    process::{Command, ExitCode},
};

use crate::Arg::Literal;

fn main() -> ExitCode {
    // todo: help
    let prompt = env::var("REPL_PROMPT").unwrap_or("> ".to_string());
    let placeholder = env::var("REPL_PLACEHOLDER").unwrap_or("@".to_string());

    let mut args = env::args();
    let cmd: String = args.nth(1).unwrap();
    let args: Vec<Arg> = args.map(|s| Arg::parse(s, &placeholder)).collect();

    let config = Config::builder().auto_add_history(true).build();
    let mut reader = match DefaultEditor::with_config(config) {
        Ok(editor) => editor,
        Err(msg) => {
            println!("error: {}", msg);
            return ExitCode::FAILURE;
        }
    };
    loop {
        let line = match reader.readline(&prompt) {
            Ok(line) => line,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("error: {}", err);
                return ExitCode::FAILURE;
            }
        };
        // todo: multiline with \ \n

        match Command::new(&cmd)
            .args(
                args.iter()
                    .map(|a| a.interpolate(line.to_string()))
                    .collect::<Vec<String>>(),
            )
            .output()
        {
            Ok(out) => {
                if out.status.success() {
                    print!("{}", String::from_utf8_lossy(&out.stdout));
                } else {
                    print!("{}", String::from_utf8_lossy(&out.stderr));
                }
            }
            Err(err) => println!("{}", err),
        };
    }
    ExitCode::SUCCESS
}

enum Arg {
    Literal(String),
    Placeholder,
    Template(Vec<Arg>),
}

impl Arg {
    fn parse(value: String, placeholder: &str) -> Arg {
        let mut acc = Vec::new();
        let mut literals = value.split(placeholder);
        acc.push(
            literals
                .nth(0)
                .map_or(Arg::Placeholder, |s| Literal(s.to_string())),
        );
        for s in literals {
            acc.push(Arg::Placeholder);
            acc.push(Arg::Literal(s.to_string()));
        }
        if acc.len() == 1 {
            acc.pop().unwrap()
        } else {
            Arg::Template(acc)
        }
    }

    fn interpolate(&self, value: String) -> String {
        use Arg::*;
        match self {
            Literal(s) => s.to_string(),
            Placeholder => value,
            Template(v) => v
                .iter()
                .map(|a| a.interpolate(value.to_string()))
                .collect::<Vec<_>>()
                .join(""),
        }
    }
}
