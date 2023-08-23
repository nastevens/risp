use crate::form::{Form, FormKind};

fn list_to_string<'a>(
    list: impl IntoIterator<Item = &'a Form>,
    start: &'static str,
    end: &'static str,
) -> String {
    let mut output = String::from(start);
    let mut has_fields = false;
    for value in list.into_iter() {
        if has_fields {
            output.push(' ');
        }
        output.push_str(&pr_str(&value));
        has_fields = true;
    }
    output.push_str(end);
    output
}

pub fn pr_str(input: &Form) -> String {
    match &input.kind {
        FormKind::Nil => "nil".to_string(),
        FormKind::Boolean(b) => format!("{}", b),
        FormKind::Symbol(ident) => ident.name.to_string(),
        FormKind::Integer(n) => format!("{}", n),
        FormKind::Float(n) => format!("{}", n),
        FormKind::String(s) => s.to_string(),
        FormKind::Keyword(k) => k.to_string(),
        FormKind::List(list) => list_to_string(list, "(", ")"),
        FormKind::Vector(list) => list_to_string(list, "[", "]"),
        FormKind::HashMap(list) => list_to_string(list, "{", "}"),
        FormKind::NativeFn { .. } => "#<function>".to_string(),
        FormKind::UserFn { .. } => "#<function>".to_string(),
    }
}
