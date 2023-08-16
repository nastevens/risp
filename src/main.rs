use risp::{ast::Form, eval, Env, Error};
use rustyline::{error::ReadlineError, DefaultEditor};

fn read(input: &str) -> Result<Form, Error> {
    risp::read_str(input)
}

// fn eval(ast: Form, env: Env) -> Result<(Form, Env), Error> {
// let mut ast = ast;
// let mut env = env;
// loop {
//     let (new_ast, new_env) = Eval::eval(&ast, &env)?;
//     ast = new_ast;
//     env = new_env.unwrap_or(env);
//     if ast.is_scalar() {
//         break;
//     }
// }
// Ok((ast, env))
// }

fn print(input: Form) -> String {
    risp::pr_str(&input)
}

fn read_eval_print(input: &str) -> Result<String, Error> {
    let form = read(input)?;
    let mut env = Env::new();
    let evaluated = eval::eval(form, &mut env)?;
    Ok(print(evaluated))
}

fn main() {
    let mut rl = DefaultEditor::new().expect("initializing rustyline");
    if rl.load_history(".mal-history").is_err() {
        eprintln!("No previous history.");
    }

    loop {
        match rl.readline("user> ") {
            Ok(line) if line.is_empty() => continue,
            Ok(line) => {
                if line.len() > 0 {
                    let _ = rl.add_history_entry(&line);
                    rl.save_history(".risp-history").expect("saving history");
                    match read_eval_print(&line) {
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
