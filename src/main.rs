use risp::{form::Form, eval, Env, Error};
use rustyline::{error::ReadlineError, DefaultEditor};

const HISTORY_FILE: &str = ".risp-history";

fn read(input: &str) -> Result<Form, Error> {
    risp::read_str(input)
}

fn print(input: Form) -> String {
    risp::pr_str(&input)
}

fn read_eval_print(input: &str, env: &mut Env) -> Result<String, Error> {
    let form = read(input)?;
    let evaluated = eval::eval(form, env)?;
    Ok(print(evaluated))
}

fn main() {
    let mut rl = DefaultEditor::new().expect("initializing rustyline");
    if rl.load_history(HISTORY_FILE).is_err() {
        eprintln!("No previous history.");
    }

    let mut env = Env::new();
    risp::core::populate(&mut env);
    loop {
        match rl.readline("user> ") {
            Ok(line) if line.is_empty() => continue,
            Ok(line) => {
                if line.len() > 0 {
                    let _ = rl.add_history_entry(&line);
                    rl.save_history(HISTORY_FILE).expect("saving history");
                    match read_eval_print(&line, &mut env) {
                        Ok(result) => println!("{}", result),
                        Err(e) => eprintln!("{}", e),
                    }
                }
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
