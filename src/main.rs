use risp::{form::Form, Env, Error};
use rustyline::{error::ReadlineError, DefaultEditor};
use tracing_subscriber;

const HISTORY_FILE: &str = ".risp-history";

fn read_eval(input: &str, env: &mut Env) -> Result<Form, Error> {
    let form = risp::read_str(input)?;
    Ok(risp::eval(form, env)?)
}

fn main() {
    tracing_subscriber::fmt::init();

    let mut rl = DefaultEditor::new().expect("initializing rustyline");
    if rl.load_history(HISTORY_FILE).is_err() {
        eprintln!("No previous history.");
    }

    let mut env = Env::new();
    let args = Form::list(std::env::args().skip(1).map(|s| Form::string(s)));
    env.set("*ARGV*", args.clone());
    risp::core::populate(&mut env);
    let _ = read_eval(r#"(println (str "Mal [" *host-language* "]"))"#, &mut env);
    loop {
        match rl.readline("user> ") {
            Ok(line) if line.is_empty() => continue,
            Ok(line) => {
                if !line.is_empty() {
                    let _ = rl.add_history_entry(&line);
                    rl.save_history(HISTORY_FILE).expect("saving history");
                    match read_eval(&line, &mut env) {
                        Ok(result) => println!("{:?}", result),
                        Err(e) => eprintln!("{:?}", e),
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
