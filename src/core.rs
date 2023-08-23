use crate::{Env, Form, FormKind, Result};

pub fn populate(env: &mut Env) {
    env.set("+", Form::native_fn(&add));
    env.set("-", Form::native_fn(&sub));
    env.set("*", Form::native_fn(&mul));
    env.set("/", Form::native_fn(&div));
    env.set("list", Form::native_fn(&list));
    env.set("list?", Form::native_fn(&is_list));
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
    let parsed: (Form,) = params.try_into()?;
    Ok(Form::boolean(matches!(
        parsed,
        (Form {
            kind: FormKind::List(_)
        },)
    )))
}
