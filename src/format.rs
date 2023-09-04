use crate::form::{Form, FormKind};

pub fn pr_str(input: &Form) -> String {
    format!("{:?}", input.kind)
}

fn fmt_list<'a>(
    list: &[Form],
    start: &'static str,
    end: &'static str,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    f.write_str(start)?;
    let mut has_fields = false;
    for form in list {
        if has_fields {
            f.write_str(" ")?;
        }
        std::fmt::Debug::fmt(form, f)?;
        has_fields = true;
    }
    f.write_str(end)
}

impl std::fmt::Debug for Form {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.kind, f)
    }
}

impl std::fmt::Debug for FormKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::{Debug, Display};
        match self {
            FormKind::Nil => f.write_str("nil"),
            FormKind::Boolean(b) => Debug::fmt(b, f),
            FormKind::Symbol(ident) => Display::fmt(&ident.name, f),
            FormKind::Integer(n) => Debug::fmt(n, f),
            FormKind::Float(n) => Debug::fmt(n, f),
            FormKind::String(s) => Display::fmt(s, f),
            FormKind::Keyword(k) => Display::fmt(k, f),
            FormKind::List(val) => fmt_list(val, "(", ")", f),
            FormKind::Vector(val) => fmt_list(val, "[", "]", f),
            FormKind::HashMap(val) => fmt_list(val, "{", "}", f),
            FormKind::NativeFn(_) => write!(f, "#<function>"),
            FormKind::UserFn { .. } => write!(f, "#<function>"),
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
