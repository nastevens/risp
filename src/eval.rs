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

fn def(form: Form, env: &mut Env) -> Result<Form> {
    let (_, symbol, value): (Form, Ident, Form) = form.try_into()?;
    let evaluated = eval(value, env)?;
    env.set(symbol.name, evaluated.clone());
    Ok(evaluated)
}

fn let_(form: Form, env: &Env) -> Result<Form> {
    let (_, bindings, to_evaluate): (Form, Vec<Form>, Form) = form.try_into()?;
    let mut iter = bindings.into_iter().fuse();
    let mut env = Env::new_with(env);
    loop {
        let symbol: Option<Ident> = iter.next().map(TryInto::try_into).transpose()?;
        let value: Option<Form> = iter.next().map(TryInto::try_into).transpose()?;
        match (symbol, value) {
            (Some(symbol), Some(value)) => {
                let evaluated = eval(value, &mut env)?;
                env.set(symbol.name, evaluated);
                env = Env::new_with(&env);
            }
            (None, None) => break,
            _ => return Err(Error::InvalidArgument),
        }
    }
    eval(to_evaluate, &mut env)
}

fn fn_(form: Form, env: &Env) -> Result<Form> {
    let (_, binds, body): (Form, Vec<Ident>, Form) = form.try_into()?;
    let env = Env::new_with(env);
    Ok(Form::user_fn(binds, body, env))
}

impl Form {
    fn apply(self, apply_env: &mut Env) -> Result<Form> {
        match self.kind {
            FormKind::List(mut list) => {
                if list.is_empty() {
                    return Err(Error::InvalidApply);
                }
                let params = list.drain(1..).collect::<Vec<Form>>();
                match list.remove(0).kind {
                    FormKind::NativeFn(f) => f(Form::list(params)),
                    FormKind::UserFn {
                        binds,
                        body,
                        env: ref closure_env,
                        ..
                    } => {
                        let mut env = Env::new_with(closure_env);
                        for (bind, value) in binds.into_iter().zip(params) {
                            let evaluated = eval_ast(value, apply_env)?;
                            env.set(bind.name, evaluated);
                        }
                        eval(*body, &mut env)
                    }
                    _ => Err(Error::InvalidApply),
                }
            }
            _ => Err(Error::InvalidApply),
        }
    }

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

    fn is_callable(&self) -> bool {
        if let FormKind::List(ref inner) = self.kind {
            inner
                .first()
                .map(|first| matches!(first.kind, FormKind::NativeFn(_) | FormKind::UserFn { .. }))
                .unwrap_or(false)
        } else {
            false
        }
    }
}

pub fn eval(form: Form, env: &mut Env) -> Result<Form> {
    match form.calling() {
        Some("def!") => def(form, env),
        Some("let*") => let_(form, env),
        Some("fn*") => fn_(form, env)?.apply(env),
        _ if form.is_empty_list() => Ok(form),
        _ if form.is_callable() => eval_ast(form, env)?.apply(env),
        _ if form.is_list() => Err(Error::InvalidApply),
        _ => eval_ast(form, env),
    }
}
