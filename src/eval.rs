use crate::{ast::Ident, Env, Error, Form, FormKind, Result};

fn def(form: Form, env: Env) -> Result<(Form, Env)> {
    let (_, symbol, value): ((), Ident, Form) = form.try_into()?;
    let (evaluated, mut env) = eval(value, env)?;
    env.set(&symbol.name, evaluated);
    Ok((env.get(&symbol.name)?, env))
}

fn let_(form: Form, env: Env) -> Result<(Form, Env)> {
    let (_, bindings, to_evaluate): ((), Vec<Form>, Form) = form.try_into()?;
    let mut iter = bindings.into_iter().fuse();
    let mut env = Env::new_with(env);
    let mut evaluated;
    loop {
        let symbol: Option<Ident> = iter.next().map(TryInto::try_into).transpose()?;
        let value: Option<Form> = iter.next().map(TryInto::try_into).transpose()?;
        match (symbol, value) {
            (Some(symbol), Some(value)) => {
                (evaluated, env) = eval(value, env)?;
                env.set(symbol.name, evaluated);
            }
            (None, None) => break,
            _ => return Err(Error::InvalidArgument),
        }
    }
    Ok((to_evaluate, env))
}

fn fn_(form: Form, env: Env) -> Result<(Form, Env)> {
    let (_, binds, body): ((), Vec<Ident>, Form) = form.try_into()?;
    let env = Env::new_with(env);
    Ok((Form::user_fn(binds, body, env.clone()), env))
}

fn if_(form: Form, env: Env) -> Result<(Form, Env)> {
    let (_, predicate, on_true, on_false): ((), Form, Form, Form) = form.try_into()?;
    let (eval_predicate, env) = eval(predicate, env)?;
    if TryInto::<bool>::try_into(eval_predicate)? {
        Ok((on_true, env))
    } else {
        Ok((on_false, env))
    }
}

fn do_(form: Form, mut env: Env) -> Result<(Form, Env)> {
    let mut params = TryInto::<Vec<Form>>::try_into(form)?
        .drain(1..)
        .collect::<Vec<Form>>();
    let last = params.pop().unwrap_or_else(Form::nil);
    for form in params {
        (_, env) = eval(form, env)?;
    }
    Ok((last, env))
}

impl Form {
    fn apply(self, env: Env) -> Result<(Form, Env)> {
        match self.kind {
            FormKind::List(mut list) => {
                if list.is_empty() {
                    return Err(Error::InvalidApply);
                }
                let params = list.drain(1..).collect::<Vec<Form>>();
                match list.remove(0).kind {
                    FormKind::NativeFn(f) => Ok((f(Form::list(params))?, env)),
                    FormKind::UserFn {
                        binds,
                        body,
                        env: _closure_env,
                        ..
                    } => {
                        let mut env = Env::new_with(env);
                        for (bind, value) in binds.into_iter().zip(params) {
                            // let result = eval(value, &mut env)?;
                            env.set(bind.name, value);
                        }
                        Ok((*body, env))
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

    fn is_user_fn(&self) -> bool {
        if let FormKind::List(ref inner) = self.kind {
            inner
                .first()
                .map(|first| matches!(first.kind, FormKind::UserFn { .. }))
                .unwrap_or(false)
        } else {
            false
        }
    }
}

pub fn eval_ast(form: Form, mut env: Env) -> Result<(Form, Env)> {
    match form {
        Form {
            kind: FormKind::Symbol(Ident { name }),
        } => Ok((env.get(&name)?, env)),
        Form {
            kind: FormKind::List(inner),
        } => {
            let mut v = vec![];
            for mut form in inner {
                (form, env) = eval(form, env)?;
                v.push(form);
            }
            Ok((Form::list(v), env))
        }
        Form {
            kind: FormKind::Vector(inner),
        } => {
            let mut v = vec![];
            for form in inner {
                let (form, _) = eval(form, env.clone())?;
                v.push(form);
            }
            Ok((Form::vector(v), env))
        }
        Form {
            kind: FormKind::HashMap(inner),
        } => {
            let mut v = vec![];
            for form in inner {
                let (form, _) = eval(form, env.clone())?;
                v.push(form);
            }
            Ok((Form::hash_map(v), env))
        }
        other => Ok((other, env)),
    }
}

pub fn eval(mut form: Form, mut env: Env) -> Result<(Form, Env)> {
    loop {
        if !form.is_list() {
            return eval_ast(form, env);
        }
        if form.is_empty_list() {
            return Ok((form, env));
        }
        match form.calling() {
            Some("def!") => return def(form, env),
            Some("let*") => (form, env) = let_(form, env)?,
            Some("do") => (form, env) = do_(form, env)?,
            Some("if") => (form, env) = if_(form, env)?,
            Some("fn*") => return fn_(form, env),
            _ => {
                (form, env) = eval_ast(form, env)?;
                if form.is_user_fn() {
                    (form, env) = form.apply(env)?;
                } else {
                    return form.apply(env);
                }
            }
        }
    }
}
