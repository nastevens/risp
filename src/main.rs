use risp::{RispError, RispForm};
use rustyline::{DefaultEditor, error::ReadlineError};

fn read(input: &str) -> Result<RispForm, RispError> {
    risp::read_str(input)
}

fn eval(input: RispForm) -> Result<RispForm, RispError> {
    Ok(input)
}

fn print(input: RispForm) -> String {
    risp::pr_str(&input)
}

fn read_eval_print(input: &str) -> Result<String, RispError> {
    let ast = read(input)?;
    let evaluated = eval(ast)?;
    Ok(print(evaluated))
}

fn main() {
    let mut rl = DefaultEditor::new().expect("initializing rustyline");
    if rl.load_history(".mal-history").is_err() {
        eprintln!("No previous history.");
    }

    loop {
        match rl.readline("user> ") {
            Ok(line) => {
                if line.len() > 0 {
                    let _ = rl.add_history_entry(&line);
                    rl.save_history(".risp-history").expect("saving history");
                    match read_eval_print(&line) {
                        Ok(result) => println!("{}", result),
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
