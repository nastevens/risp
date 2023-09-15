use std::sync::OnceLock;

use crate::form::{Atom, Form, FormKind};

pub fn pr_str(input: &Form) -> String {
    format!("{:?}", input.kind)
}

struct ListWriter<'a> {
    values: &'a [Form],
    start: &'static str,
    end: &'static str,
}

impl<'a> ListWriter<'a> {
    pub fn new(values: &'a [Form], start: &'static str, end: &'static str) -> ListWriter<'a> {
        ListWriter { values, start, end }
    }

    pub fn write<F>(&self, fmt: F, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where
        F: Fn(&Form, &mut std::fmt::Formatter) -> std::fmt::Result,
    {
        f.write_str(self.start)?;
        let mut has_fields = false;
        for form in self.values {
            if has_fields {
                f.write_str(" ")?;
            }
            fmt(form, f)?;
            has_fields = true;
        }
        f.write_str(self.end)
    }
}

fn escape_unprintable(s: &str) -> String {
    use aho_corasick::AhoCorasick;
    static AC: OnceLock<AhoCorasick> = OnceLock::new();
    const PATTERNS: &[&str] = &["\\", "\n", "\""];
    const REPLACEMENTS: &[&str] = &["\\\\", "\\n", "\\\""];
    let escape_replacment =
        AC.get_or_init(|| AhoCorasick::new(PATTERNS).expect("parsing static AhoCorasick patterns"));
    escape_replacment.replace_all(s, REPLACEMENTS)
}

impl std::fmt::Debug for Form {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.kind, f)
    }
}

impl std::fmt::Debug for FormKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormKind::Nil => f.write_str("nil"),
            FormKind::Boolean(b) => write!(f, "{b}"),
            FormKind::Symbol(ident) => write!(f, "{}", &ident.name),
            FormKind::Integer(n) => write!(f, "{n}"),
            FormKind::Float(n) => write!(f, "{n}"),
            FormKind::String(s) => write!(f, "\"{}\"", escape_unprintable(s)),
            FormKind::Keyword(k) => write!(f, "{k}"),
            FormKind::List(val) => ListWriter::new(val, "(", ")").write(std::fmt::Debug::fmt, f),
            FormKind::Vector(val) => ListWriter::new(val, "[", "]").write(std::fmt::Debug::fmt, f),
            FormKind::HashMap(val) => ListWriter::new(val, "{", "}").write(std::fmt::Debug::fmt, f),
            FormKind::NativeFn(_) => write!(f, "#<function>"),
            FormKind::UserFn { .. } => write!(f, "#<function>"),
            FormKind::Atom(atom) => write!(f, "(atom {:?})", *atom.value.borrow()),
        }
    }
}

impl std::fmt::Display for Form {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.kind, f)
    }
}

impl std::fmt::Display for FormKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormKind::String(s) => write!(f, "{s}"),
            FormKind::List(val) => ListWriter::new(val, "(", ")").write(std::fmt::Display::fmt, f),
            FormKind::Vector(val) => {
                ListWriter::new(val, "[", "]").write(std::fmt::Display::fmt, f)
            }
            FormKind::HashMap(val) => {
                ListWriter::new(val, "{", "}").write(std::fmt::Display::fmt, f)
            }
            FormKind::Atom(Atom { value }) => write!(f, "{}", *value.borrow()),
            other => std::fmt::Debug::fmt(other, f),
        }
    }
}
