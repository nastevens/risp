use crate::{form::Atom, Env, Form, Result};
use std::fmt::Write;

pub fn populate(env: &mut Env) {
    env.set("+", Form::native_fn(&add));
    env.set("-", Form::native_fn(&sub));
    env.set("*", Form::native_fn(&mul));
    env.set("/", Form::native_fn(&div));
    env.set("list", Form::native_fn(&list));
    env.set("list?", Form::native_fn(&is_list));
    env.set("empty?", Form::native_fn(&is_empty));
    env.set("count", Form::native_fn(&count));
    env.set("=", Form::native_fn(&eq));
    env.set("<", Form::native_fn(&lt));
    env.set("<=", Form::native_fn(&lte));
    env.set(">", Form::native_fn(&gt));
    env.set(">=", Form::native_fn(&gte));
    env.set("pr-str", Form::native_fn(&pr_str));
    env.set("str", Form::native_fn(&str_));
    env.set("prn", Form::native_fn(&prn));
    env.set("println", Form::native_fn(&println_));
    env.set("read-string", Form::native_fn(&read_string));
    env.set("slurp", Form::native_fn(&slurp));
    env.set("atom", Form::native_fn(&atom));
    env.set("atom?", Form::native_fn(&is_atom));
    env.set("deref", Form::native_fn(&deref));
    env.set("reset!", Form::native_fn(&reset));
    crate::eval_str(r#"(def! not (fn* (a) (if a false true)))"#, env);
    crate::eval_str(
        r#"(def! load-file (fn* (f) (eval (read-string (str "(do " (slurp f) "\nnil)")))))"#,
        env,
    );
}

fn add(params: Form) -> Result<Form> {
    let parsed: Vec<i64> = params.try_into()?;
    Ok(Form::int(parsed.iter().sum()))
}

fn sub(params: Form) -> Result<Form> {
    let parsed: Vec<i64> = params.try_into()?;
    let mut iter = parsed.into_iter();
    let first = iter.next().unwrap_or(0);
    Ok(Form::int(iter.fold(first, |accum, arg| accum - arg)))
}

fn mul(params: Form) -> Result<Form> {
    let parsed: Vec<i64> = params.try_into()?;
    Ok(Form::int(parsed.iter().product()))
}

fn div(params: Form) -> Result<Form> {
    let parsed: (i64, i64) = params.try_into()?;
    Ok(Form::int(parsed.0 / parsed.1))
}

fn list(params: Form) -> Result<Form> {
    Ok(params)
}

fn is_list(params: Form) -> Result<Form> {
    let (parsed,): (Form,) = params.try_into()?;
    Ok(Form::boolean(parsed.is_list()))
}

fn is_empty(params: Form) -> Result<Form> {
    let (parsed,): (Vec<Form>,) = params.try_into()?;
    Ok(Form::boolean(parsed.is_empty()))
}

fn count(params: Form) -> Result<Form> {
    let (parsed,): (Vec<Form>,) = params.try_into()?;
    Ok(Form::int(parsed.len().try_into()?))
}

fn eq(params: Form) -> Result<Form> {
    let (a, b): (Form, Form) = params.try_into()?;
    Ok(Form::boolean(a == b))
}

fn lt(params: Form) -> Result<Form> {
    let (a, b): (i64, i64) = params.try_into()?;
    Ok(Form::boolean(a < b))
}

fn lte(params: Form) -> Result<Form> {
    let (a, b): (i64, i64) = params.try_into()?;
    Ok(Form::boolean(a <= b))
}

fn gt(params: Form) -> Result<Form> {
    let (a, b): (i64, i64) = params.try_into()?;
    Ok(Form::boolean(a > b))
}

fn gte(params: Form) -> Result<Form> {
    let (a, b): (i64, i64) = params.try_into()?;
    Ok(Form::boolean(a >= b))
}

fn pr_str(params: Form) -> Result<Form> {
    let forms: Vec<Form> = params.try_into()?;
    let mut sep = "";
    let mut s = String::new();
    for form in forms {
        let _ = write!(&mut s, "{sep}{form:?}");
        sep = " ";
    }
    Ok(Form::string(s))
}

fn str_(params: Form) -> Result<Form> {
    let forms: Vec<Form> = params.try_into()?;
    let mut s = String::new();
    for form in forms {
        let _ = write!(&mut s, "{form}");
    }
    Ok(Form::string(s))
}

fn prn(params: Form) -> Result<Form> {
    let forms: Vec<Form> = params.try_into()?;
    let mut sep = "";
    for form in forms {
        print!("{sep}{form:?}");
        sep = " ";
    }
    print!("\n");
    Ok(Form::nil())
}

fn println_(params: Form) -> Result<Form> {
    let forms: Vec<Form> = params.try_into()?;
    let mut sep = "";
    for form in forms {
        print!("{sep}{form}");
        sep = " ";
    }
    print!("\n");
    Ok(Form::nil())
}

fn read_string(params: Form) -> Result<Form> {
    let (s,): (String,) = params.try_into()?;
    crate::read_str(&s)
}

fn slurp(params: Form) -> Result<Form> {
    let (file,): (String,) = params.try_into()?;
    Ok(Form::string(std::fs::read_to_string(file)?))
}

fn atom(params: Form) -> Result<Form> {
    let (form,): (Form,) = params.try_into()?;
    Ok(Form::atom(Atom::new(form)))
}

fn is_atom(params: Form) -> Result<Form> {
    let (form,): (Form,) = params.try_into()?;
    Ok(Form::boolean(form.is_atom()))
}

fn deref(params: Form) -> Result<Form> {
    let (atom,): (Atom,) = params.try_into()?;
    let form = atom.value.borrow().clone();
    Ok(form)
}

fn reset(params: Form) -> Result<Form> {
    let (atom, form): (Atom, Form) = params.try_into()?;
    *atom.value.borrow_mut() = form.clone();
    Ok(form)
}
