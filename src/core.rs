use crate::{convert::Rest, form::Atom, Env, Form, Result, FormKind};
use std::fmt::Write;

pub fn populate(env: &mut Env) {
    env.extend(
        [
            ("+", Form::native_fn(&add)),
            ("-", Form::native_fn(&sub)),
            ("*", Form::native_fn(&mul)),
            ("/", Form::native_fn(&div)),
            ("list", Form::native_fn(&list)),
            ("list?", Form::native_fn(&is_list)),
            ("empty?", Form::native_fn(&is_empty)),
            ("count", Form::native_fn(&count)),
            ("=", Form::native_fn(&eq)),
            ("<", Form::native_fn(&lt)),
            ("<=", Form::native_fn(&lte)),
            (">", Form::native_fn(&gt)),
            (">=", Form::native_fn(&gte)),
            ("pr-str", Form::native_fn(&pr_str)),
            ("str", Form::native_fn(&str_)),
            ("prn", Form::native_fn(&prn)),
            ("println", Form::native_fn(&println_)),
            ("read-string", Form::native_fn(&read_string)),
            ("slurp", Form::native_fn(&slurp)),
            ("atom", Form::native_fn(&atom)),
            ("atom?", Form::native_fn(&is_atom)),
            ("deref", Form::native_fn(&deref)),
            ("reset!", Form::native_fn(&reset)),
            ("swap!", Form::native_fn(&swap)),
            ("cons", Form::native_fn(&cons)),
            ("concat", Form::native_fn(&concat)),
            ("vec", Form::native_fn(&vec_)),
            ("nth", Form::native_fn(&nth)),
            ("first", Form::native_fn(&first)),
            ("rest", Form::native_fn(&rest)),
            ("apply", Form::native_fn(&apply)),
            ("map", Form::native_fn(&map)),
            ("nil?", Form::native_fn(&is_nil)),
            ("true?", Form::native_fn(&is_true)),
            ("false?", Form::native_fn(&is_false)),
            ("symbol", Form::native_fn(&symbol)),
            ("symbol?", Form::native_fn(&is_symbol)),
            ("vector", Form::native_fn(&vector)),
            ("vector?", Form::native_fn(&is_vector)),
            ("sequential?", Form::native_fn(&is_sequential)),
            ("throw", Form::native_fn(&throw)),
        ]
        .into_iter()
        .map(|(symbol, func)| (symbol.to_string(), func)),
    );
    crate::eval_str(r#"(def! not (fn* (a) (if a false true)))"#, env);
    crate::eval_str(
        r#"(def! load-file (fn* (f) (eval (read-string (str "(do " (slurp f) "\nnil)")))))"#,
        env,
    );
    crate::eval_str(
        r#"(defmacro! cond (fn* (& xs) (if (> (count xs) 0) (list 'if (first xs) (if (> (count xs) 1) (nth xs 1) (throw "odd number of forms to cond")) (cons 'cond (rest (rest xs)))))))"#,
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

fn swap(params: Form) -> Result<Form> {
    let (atom, func, Rest { values: rest }): (Atom, Form, Rest) = params.try_into()?;
    let mut handle = atom.value.borrow_mut();
    let old_value = std::mem::replace(&mut *handle, Form::nil());
    let mut args = vec![old_value];
    args.extend(rest);
    *handle = func.call(Form::list(args))?;
    Ok(handle.clone())
}

fn cons(params: Form) -> Result<Form> {
    let (x, mut seq): (Form, Vec<Form>) = params.try_into()?;
    seq.insert(0, x);
    Ok(Form::list(seq))
}

fn concat(params: Form) -> Result<Form> {
    let lists: Vec<Vec<Form>> = params.try_into()?;
    Ok(Form::list(lists.into_iter().flatten()))
}

fn vec_(params: Form) -> Result<Form> {
    let (arg,): (Vec<Form>,) = params.try_into()?;
    Ok(Form::vector(arg))
}

fn nth(params: Form) -> Result<Form> {
    let (list, index): (Vec<Form>, i64) = params.try_into()?;
    let index: usize = index.try_into()?;
    list.get(index)
        .cloned()
        .ok_or(crate::Error::IndexOutOfRange(index))
}

fn first(params: Form) -> Result<Form> {
    let (list,): (Form,) = params.try_into()?;
    if list.is_nil() || list.is_empty_sequential() {
        Ok(Form::nil())
    } else if list.is_sequential() {
        let (first, _): (Form, Rest) = list.try_into()?;
        Ok(first)
    } else {
        Err(crate::Error::InvalidArgument)
    }
}

fn rest(params: Form) -> Result<Form> {
    let (list,): (Form,) = params.try_into()?;
    if list.is_nil() || list.is_empty_sequential() {
        Ok(Form::empty_list())
    } else if let Ok((_, rest)) = <Form as TryInto<(Form, Rest)>>::try_into(list) {
        Ok(rest.into())
    } else {
        Err(crate::Error::InvalidArgument)
    }
}

fn apply(params: Form) -> Result<Form> {
    let (f, mut rest): (Form, Rest) = params.try_into()?;
    let last = rest.values.pop();
    let mut args = rest.values.drain(..rest.values.len()).collect::<Vec<_>>();
    if let Some(list_args) = last {
        args.extend(list_args.try_into_iter()?);
        f.call(Form::list(args))
    } else {
        Err(crate::Error::InvalidArgument)
    }
}

fn map(params: Form) -> Result<Form> {
    let (f, list): (Form, Vec<Form>) = params.try_into()?;
    let mapped = list
        .into_iter()
        .map(|arg| f.clone().call(Form::list([arg])))
        .collect::<Result<Vec<Form>>>()?;
    Ok(Form::list(mapped))
}

fn is_nil(params: Form) -> Result<Form> {
    let (arg,): (Form,) = params.try_into()?;
    Ok(Form::boolean(arg.is_nil()))
}

fn is_true(params: Form) -> Result<Form> {
    let (arg,): (Form,) = params.try_into()?;
    Ok(Form::boolean(matches!(arg.kind, FormKind::Boolean(true))))
}

fn is_false(params: Form) -> Result<Form> {
    let (arg,): (Form,) = params.try_into()?;
    Ok(Form::boolean(matches!(arg.kind, FormKind::Boolean(false))))
}

fn is_symbol(params: Form) -> Result<Form> {
    let (arg,): (Form,) = params.try_into()?;
    Ok(Form::boolean(arg.is_symbol()))
}

fn symbol(params: Form) -> Result<Form> {
    let (arg,): (String,) = params.try_into()?;
    Ok(Form::symbol(&arg))
}

fn vector(params: Form) -> Result<Form> {
    let args: Vec<Form> = params.try_into()?;
    Ok(Form::vector(args))
}

fn is_vector(params: Form) -> Result<Form> {
    let (arg,): (Form,) = params.try_into()?;
    Ok(Form::boolean(arg.is_vector()))
}

fn is_sequential(params: Form) -> Result<Form> {
    let (arg,): (Form,) = params.try_into()?;
    Ok(Form::boolean(arg.is_sequential()))
}

fn throw(params: Form) -> Result<Form> {
    let (arg, ): (Form, ) = params.try_into()?;
    Err(crate::Error::UserError(arg))
}

fn _template(_params: Form) -> Result<Form> {
    todo!()
}
