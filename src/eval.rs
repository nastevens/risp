use crate::{form::Ident, Env, Error, Form, FormKind, Result};

fn def(form: Form, env: &mut Env) -> Result<Form> {
    let (_, symbol, value): ((), Ident, Form) = form.try_into()?;
    let evaluated = eval(value, env)?;
    env.set(&symbol.name, evaluated.clone());
    Ok(evaluated)
}

// fn defmacro(form: Form, env: &mut Env) -> Result<Form> {
//     let (_, symbol, maybe_macro): ((), Ident, Form) = form.try_into()?;
//     let evaluated = eval(maybe_macro, env)?;
//     match evaluated.kind {
//         FormKind::UserFn { binds, bind_rest, body, env, is_macro }
//     }
//     env.set(&symbol.name, evaluated.clone());
//     Ok(evaluated)
// }

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
    let (_, bind_symbols, body): ((), Vec<Ident>, Form) = form.try_into()?;
    let mut iter = bind_symbols.into_iter();
    let binds = iter
        .by_ref()
        .take_while(|ident| ident.name != "&")
        .collect::<Vec<_>>();
    let bind_rest = iter.next();
    let closure_env = Env::new_with(env);
    Ok(Form::user_fn(binds, bind_rest, body, closure_env))
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

fn quote(form: Form) -> Result<Form> {
    let (_, quoted): (Form, Form) = form.try_into()?;
    Ok(quoted)
}

fn quasiquote_(form: Form) -> Result<Form> {
    if form.as_fn_name() == Some("unquote") {
        let (_, arg): (Form, Form) = form.try_into()?;
        Ok(arg)
    } else if form.is_empty_list() {
        // Note that this arm should not use `is_empty_collection` - it breaks the tests for
        // `(quasiquoteexpand [])`, which expects to get back `(vec ())`, not `[]`
        Ok(form)
    } else if form.is_collection() {
        let result = form
            .clone()
            .try_into_iter()
            .expect("previously confirmed as list")
            .rfold(Ok(Form::list([])), |accum: Result<Form>, elem| {
                if elem.as_fn_name() == Some("splice-unquote") {
                    let (_, arg): (Form, Form) = elem.try_into()?;
                    Ok(Form::list([Form::symbol("concat"), arg, accum?]))
                } else {
                    Ok(Form::list([
                        Form::symbol("cons"),
                        quasiquote_(elem)?,
                        accum?,
                    ]))
                }
            })?;
        if form.is_vector() {
            Ok(Form::list([Form::symbol("vec"), result]))
        } else {
            Ok(result)
        }
    } else if form.is_symbol() || form.is_hash_map() {
        Ok(Form::list([Form::symbol("quote"), form]))
    } else {
        Ok(form)
    }
}

fn quasiquoteexpand(form: Form) -> Result<Form> {
    let (_, arg): (Form, Form) = form.try_into()?;
    quasiquote_(arg)
}

fn quasiquote(form: Form) -> Result<Form> {
    let (_, arg): (Form, Form) = form.try_into()?;
    quasiquote_(arg)
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

fn as_macro_call(form: &Form, env: &Env) -> Option<Form> {
    let list = match form.kind {
        FormKind::List(ref list) => list,
        _ => return None,
    };
    let name = list.first()?.as_fn_name()?;
    env.get(name).ok().filter(Form::is_macro)
}

fn macro_expand(mut form: Form, env: &Env) -> Result<Form> {
    while let Some(macro_) = as_macro_call(&form, env) {
        let params = Form::list(form.try_into_iter()?.skip(1));
        let (new_form, _) = apply_user_fn(macro_, params)?;
        form = new_form;
    }
    Ok(form)
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
            bind_rest,
            body,
            env: closure_env,
            is_macro: _,
        } => {
            let mut env = Env::new_with(&closure_env);
            let mut param_iter = params.try_into_iter()?.fuse();
            let mut binds_iter = binds.into_iter().fuse();
            let mut rest = Vec::new();
            loop {
                match (binds_iter.next(), param_iter.next()) {
                    (Some(bind), Some(value)) => env.set(bind.name, value),
                    (None, Some(value)) if bind_rest.is_some() => rest.push(value),
                    (None, Some(_)) => {
                        // Parameter isn't used, no reason to save it
                    }
                    (Some(_), None) => return Err(Error::InvalidArgument), //TODO need a better error
                    (None, None) => break,
                }
            }
            if let Some(bind_rest_ident) = bind_rest {
                env.set(bind_rest_ident.name, Form::list(rest));
            }
            Ok((*body, env))
        }
        _ => panic!("apply_user_fn called with wrong Form type: {:?}", f),
    }
}

impl Form {
    pub fn as_symbol_name(&self) -> Option<&str> {
        if let FormKind::Symbol(Ident { ref name }) = self.kind {
            Some(name)
        } else {
            None
        }
    }

    pub fn as_fn_name(&self) -> Option<&str> {
        if let FormKind::List(ref inner) = self.kind {
            inner.first().and_then(|first| match first.kind {
                FormKind::Symbol(ref ident) => Some(&*ident.name),
                _ => None,
            })
        } else {
            None
        }
    }

    pub fn as_slice(&self) -> Option<&[Form]> {
        if let FormKind::List(ref inner) | FormKind::Vector(ref inner) = self.kind {
            Some(inner)
        } else {
            None
        }
    }

    pub fn call(self, params: Form) -> Result<Form> {
        if self.is_native_fn() {
            apply_native_fn(self, params)
        } else if self.is_user_fn() {
            apply_user_fn(self, params).and_then(|(form, mut env)| eval(form, &mut env))
        } else {
            Err(Error::NotCallable)
        }
    }

    pub fn is_macro(&self) -> bool {
        matches!(self.kind, FormKind::UserFn { is_macro: true, .. })
    }
}

fn eval_(form: Form, env: &mut Env) -> Result<Form> {
    let (_, param): (Form, Form) = form.try_into()?;
    let evaluated = eval(param, env)?;
    eval(evaluated, &mut env.root())
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
        form = macro_expand(form, env)?;
        if !form.is_list() {
            return eval_ast(form, env);
        }
        if form.is_empty_list() {
            return Ok(form);
        }
        match form.as_fn_name() {
            Some("def!") => return def(form, env),
            Some("defmacro!") => return def(form, env),
            Some("let*") => {
                let new_env;
                (form, new_env) = let_(form, env)?;
                tco_env = Some(new_env);
            }
            Some("do") => form = do_(form, env)?,
            Some("if") => form = if_(form, env)?,
            Some("fn*") => return fn_(form, env),
            Some("eval") => return eval_(form, env),
            Some("quote") => return quote(form),
            Some("quasiquote") => form = quasiquote(form)?,
            Some("quasiquoteexpand") => return quasiquoteexpand(form),
            Some("macroexpand") => {
                let (_, arg): (Form, Form) = form.try_into()?;
                return macro_expand(arg, env);
            }
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
