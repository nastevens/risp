use crate::{Env, Form, Result};

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
    env.set("prn", Form::native_fn(&prn));
    env.set("pr-str", Form::native_fn(&pr_str));
    crate::eval_str("(def! not (fn* (a) (if a false true)))", env);
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
    let values = TryInto::<Vec<Form>>::try_into(params)?;
    let strings = values.iter().map(crate::pr_str).collect::<Vec<String>>();
    Ok(Form::string(strings.join(" ")))
}

fn prn(params: Form) -> Result<Form> {
    let string: String = pr_str(params)?.try_into()?;
    println!("{}", string);
    Ok(Form::nil())
}
