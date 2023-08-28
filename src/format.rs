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
        FormKind::String(s) => format!("\"{}\"", s),
        FormKind::Keyword(k) => k.to_string(),
        FormKind::List(list) => list_to_string(list, "(", ")"),
        FormKind::Vector(list) => list_to_string(list, "[", "]"),
        FormKind::HashMap(list) => list_to_string(list, "{", "}"),
        FormKind::NativeFn { .. } => "#<function>".to_string(),
        FormKind::UserFn { .. } => "#<function>".to_string(),
    }
}

impl std::fmt::Debug for FormKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FormKind::")?;
        match self {
            FormKind::Nil => f.write_str("Nil"),
            FormKind::Boolean(val) => write!(f, "Boolean({:?})", val),
            FormKind::Symbol(val) => write!(f, "Symbol({:?})", val),
            FormKind::Integer(val) => write!(f, "Integer({:?})", val),
            FormKind::Float(val) => write!(f, "Float({:?})", val),
            FormKind::String(val) => write!(f, "String({:?})", val),
            FormKind::Keyword(val) => write!(f, "Keyword({:?})", val),
            FormKind::List(val) => write!(f, "List({:?})", val),
            FormKind::Vector(val) => write!(f, "Vector({:?})", val),
            FormKind::HashMap(val) => write!(f, "HashMap({:?})", val),
            FormKind::NativeFn(_) => write!(f, "NativeFn(#<function>)"),
            FormKind::UserFn { .. } => write!(f, "UserFn"),
        }
    }
}

//     pub fn expand(&self) -> Result<String, Error> {
//         use nom::{
//             branch::alt,
//             bytes::complete::{escaped_transform, is_not, tag},
//             combinator::value,
//             sequence::delimited,
//             IResult,
//         };
//         fn extract_and_expand(input: &str) -> IResult<&str, String> {
//             alt((
//                 value(String::new(), tag("\"\"")),
//                 delimited(
//                     tag("\""),
//                     escaped_transform(
//                         is_not("\\\""),
//                         '\\',
//                         alt((
//                             value("\\", tag("\\")),
//                             value("\"", tag("\"")),
//                             value("\n", tag("n")),
//                         )),
//                     ),
//                     tag("\""),
//                 ),
//             ))(input)
//         }
//         Ok(extract_and_expand(&self.value)
//             .map(|result| result.1)
//             .map_err(|_| Error::Eof)?)
//     }
// }
