use crate::{ast::Ident, Env, Error, Form, FormKind, Result};

pub fn eval_ast(form: Form, env: &mut Env) -> Result<Form> {
    match form {
        Form {
            kind: FormKind::Symbol(Ident { name }),
        } => env.get(&name),
        Form {
            kind: FormKind::List(inner),
        } => {
            let evaluated = inner
                .into_iter()
                .map(|form| eval(form, env))
                .collect::<Result<Vec<_>>>()?;
            Ok(Form::list(evaluated))
        }
        Form {
            kind: FormKind::Vector(inner),
        } => {
            let evaluated = inner
                .into_iter()
                .map(|form| eval(form, env))
                .collect::<Result<Vec<_>>>()?;
            Ok(Form::vector(evaluated))
        }
        Form {
            kind: FormKind::HashMap(inner),
        } => {
            let evaluated = inner
                .into_iter()
                .map(|form| eval(form, env))
                .collect::<Result<Vec<_>>>()?;
            Ok(Form::hash_map(evaluated))
        }
        other => return Ok(other),
    }
}

fn callable_name<'a>(form: &'a Form) -> Option<&'a str> {
    if let FormKind::List(ref inner) = form.kind {
        inner.first().and_then(|first| match first.kind {
            FormKind::Symbol(ref ident) => Some(&*ident.name),
            FormKind::NativeFn { name, .. } => Some(name),
            _ => None,
        })
    } else {
        None
    }
}

fn def(form: Form, env: &mut Env) -> Result<Form> {
    dbg!(&form);
    let (_, symbol, value): (Form, Ident, Form) = form.try_into()?;
    let evaluated = eval(value, env)?;
    env.set(symbol.name, evaluated.clone());
    Ok(evaluated)
}

/// # Panics
///
/// Panics if the provided Form is not a List
fn apply(form: Form, called_as: Option<&str>) -> Result<Form> {
    let called_as = called_as.unwrap_or("#<anonymous>");
    match form.kind {
        FormKind::List(mut list) => {
            let args = list.drain(1..).collect::<Vec<Form>>();
            if let FormKind::NativeFn { f, .. } = list.remove(0).kind {
                f(&called_as, Form::list(args))
            } else {
                Err(Error::InvalidApply)
            }
        }
        other => panic!(
            "`apply` requires a FormKind::List, but kind was {:?}",
            other
        ),
    }
}

pub fn eval(form: Form, env: &mut Env) -> Result<Form> {
    match callable_name(&form) {
        Some("def!") => def(form, env),
        Some("let*") => todo!(),
        Some(name) => {
            // Guard that callable_name isn't refactored to return Some(..) on non-list types
            assert!(form.is_list());

            // Need to release borrow from fn_name call
            let called_as = name.to_string();
            let evaluated = eval_ast(form, env)?;
            apply(evaluated, Some(&called_as))
        }
        None if form.is_empty_list() => Ok(form),
        None if form.is_list() => {
            // Not valid if there's no callable_name for a list - need to use (list ...)
            Err(Error::InvalidApply)
        }
        None => eval_ast(form, env),
    }
}
