use crate::{Env, Form, FormKind, Result};

pub fn populate(env: &mut Env) {
    env.set("+", Form::native("+", &add));
    env.set("-", Form::native("-", &sub));
    env.set("*", Form::native("*", &mul));
    env.set("/", Form::native("/", &div));
    env.set("list", Form::native("list", &list));
    env.set("list?", Form::native("list?", &is_list));
}

fn add(_called_as: &str, args: Form) -> Result<Form> {
    let parsed: Vec<i64> = args.try_into()?;
    Ok(Form::int(parsed.iter().sum()))
}

fn sub(_called_as: &str, args: Form) -> Result<Form> {
    let parsed: Vec<i64> = args.try_into()?;
    let mut iter = parsed.into_iter();
    let first = iter.next().unwrap_or(0);
    Ok(Form::int(iter.fold(first, |accum, arg| accum - arg)))
}

fn mul(_called_as: &str, args: Form) -> Result<Form> {
    let parsed: Vec<i64> = args.try_into()?;
    Ok(Form::int(parsed.iter().product()))
}

fn div(_called_as: &str, args: Form) -> Result<Form> {
    let parsed: (i64, i64) = args.try_into()?;
    Ok(Form::int(parsed.0 / parsed.1))
}

fn list(_called_as: &str, args: Form) -> Result<Form> {
    Ok(args)
}

fn is_list(_called_as: &str, args: Form) -> Result<Form> {
    let parsed: (Form,) = args.try_into()?;
    Ok(Form::boolean(matches!(
        parsed,
        (Form {
            kind: FormKind::List(_)
        },)
    )))
}
