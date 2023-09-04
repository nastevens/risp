use crate::{form::Ident, Env, Error, Form, FormKind, Result};

fn def(form: Form, env: &mut Env) -> Result<Form> {
    let (_, symbol, value): ((), Ident, Form) = form.try_into()?;
    let evaluated = eval(value, env)?;
    env.set(&symbol.name, evaluated.clone());
    Ok(evaluated)
}

fn let_(form: Form, env: &Env) -> Result<(Form, Env)> {
    let (_, bindings, to_evaluate): ((), Vec<Form>, Form) = form.try_into()?;
    let mut iter = bindings.into_iter().fuse();
    let mut env = Env::new_with(env);
    let mut evaluated;
    loop {
        let symbol: Option<Ident> = iter.next().map(TryInto::try_into).transpose()?;
        let value: Option<Form> = iter.next().map(TryInto::try_into).transpose()?;
        match (symbol, value) {
            (Some(symbol), Some(value)) => {
                evaluated = eval(value, &mut env)?;
                env.set(symbol.name, evaluated);
            }
            (None, None) => break,
            _ => return Err(Error::InvalidArgument),
        }
    }
    Ok((to_evaluate, env))
}

fn fn_(form: Form, env: &Env) -> Result<Form> {
    let (_, binds, body): ((), Vec<Ident>, Form) = form.try_into()?;
    let closure_env = Env::new_with(env);
    Ok(Form::user_fn(binds, body, closure_env))
}

fn if_(form: Form, env: &mut Env) -> Result<Form> {
    let (_, predicate, on_true, on_false): ((), Form, Form, Form) = form.try_into()?;
    let eval_predicate = eval(predicate, env)?;
    if TryInto::<bool>::try_into(eval_predicate)? {
        Ok(on_true)
    } else {
        Ok(on_false)
    }
}

fn do_(form: Form, env: &mut Env) -> Result<Form> {
    let mut params = TryInto::<Vec<Form>>::try_into(form)?
        .drain(1..)
        .collect::<Vec<Form>>();
    let last = params.pop().unwrap_or_else(Form::nil);
    for form in params {
        let _ = eval(form, env)?;
    }
    Ok(last)
}

fn extract_fn(form: Form) -> Result<(Form, Form)> {
    match form.kind {
        FormKind::List(mut list) => {
            if list.is_empty() {
                return Err(Error::InvalidApply);
            }
            let params = list.drain(1..).collect::<Vec<Form>>();
            Ok((list.remove(0), Form::list(params)))
        }
        _ => Err(Error::InvalidApply),
    }
}

fn apply_native_fn(f: Form, params: Form) -> Result<Form> {
    assert!(params.is_list());
    if let FormKind::NativeFn(f) = f.kind {
        Ok(f(params)?)
    } else {
        panic!("apply_native_fn called with wrong Form type: {:?}", f)
    }
}

fn apply_user_fn(f: Form, params: Form) -> Result<(Form, Env)> {
    assert!(f.is_user_fn());
    assert!(params.is_list());
    match f.kind {
        FormKind::UserFn {
            binds,
            body,
            env: closure_env,
            ..
        } => {
            let mut env = Env::new_with(&closure_env);
            for (bind, value) in binds.into_iter().zip(params.into_iter()?) {
                env.set(bind.name, value);
            }
            Ok((*body, env))
        }
        _ => panic!("apply_user_fn called with wrong Form type: {:?}", f),
    }
}

impl Form {
    fn calling(&self) -> Option<&str> {
        if let FormKind::List(ref inner) = self.kind {
            inner.first().and_then(|first| match first.kind {
                FormKind::Symbol(ref ident) => Some(&*ident.name),
                _ => None,
            })
        } else {
            None
        }
    }
}

pub fn eval_ast(form: Form, env: &mut Env) -> Result<Form> {
    match form {
        Form {
            kind: FormKind::Symbol(Ident { name }),
        } => Ok(env.get(&name)?),
        Form {
            kind: FormKind::List(inner),
        } => {
            let evaluated = inner
                .into_iter()
                .map(|form| eval(form, env))
                .collect::<Result<Vec<Form>>>()?;
            Ok(Form::list(evaluated))
        }
        Form {
            kind: FormKind::Vector(inner),
        } => {
            let evaluated = inner
                .into_iter()
                .map(|form| eval(form, env))
                .collect::<Result<Vec<Form>>>()?;
            Ok(Form::vector(evaluated))
        }
        Form {
            kind: FormKind::HashMap(inner),
        } => {
            let evaluated = inner
                .into_iter()
                .map(|form| eval(form, env))
                .collect::<Result<Vec<Form>>>()?;
            Ok(Form::hash_map(evaluated))
        }
        other => Ok(other),
    }
}

pub fn eval(mut form: Form, outer_env: &mut Env) -> Result<Form> {
    let mut tco_env: Option<Env> = None;
    loop {
        tracing::trace!(?form);
        let env = if let Some(ref mut inner_env) = tco_env {
            inner_env
        } else {
            &mut *outer_env
        };
        if !form.is_list() {
            return eval_ast(form, env);
        }
        if form.is_empty_list() {
            return Ok(form);
        }
        match form.calling() {
            Some("def!") => return def(form, env),
            Some("let*") => {
                let new_env;
                (form, new_env) = let_(form, env)?;
                tco_env = Some(new_env);
            }
            Some("do") => form = do_(form, env)?,
            Some("if") => form = if_(form, env)?,
            Some("fn*") => return fn_(form, env),
            _ => {
                let (f, params) = extract_fn(eval_ast(form, env)?)?;
                if f.is_user_fn() {
                    let new_env;
                    (form, new_env) = apply_user_fn(f, params)?;
                    tco_env = Some(new_env);
                } else {
                    return apply_native_fn(f, params);
                }
            }
        }
    }
}
