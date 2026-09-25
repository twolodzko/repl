# The missing REPL for command line utilities

Provide a command with a list o arguments as parameters to run them
interactively as a read-eval-print loop. Use `@` as w wildcard parameter
to be replaced by the command provided in a REPL command. If `@` is not
given, the command is appended to the parameters.

For example, oo create a REPL for sed run

```shell
repl sed @ src/main.rs
```

or to run a calculator app run

```shell
repl sh -c 'echo $((@))'
```

You can change `@` to any placeholder by setting the `REPL_PLACEHOLDER` environment variable.
