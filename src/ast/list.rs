use super::{Ast, Form};

struct DisplayList<'a, 'b: 'a> {
    fmt: &'a mut std::fmt::Formatter<'b>,
    result: std::fmt::Result,
    has_fields: bool,
    end_symbol: &'static str,
    print_readably: bool,
}

impl<'a, 'b: 'a> DisplayList<'a, 'b> {
    fn new(
        start_symbol: &'static str,
        end_symbol: &'static str,
        fmt: &'a mut std::fmt::Formatter<'b>,
        print_readably: bool,
    ) -> DisplayList<'a, 'b> {
        let result = fmt.write_str(start_symbol);
        DisplayList {
            fmt,
            result,
            has_fields: false,
            end_symbol,
            print_readably,
        }
    }

    fn entry(&mut self, entry: &Ast) {
        self.result = self.result.and_then(|_| {
            if self.has_fields {
                self.fmt.write_str(" ")?
            }
            Form::fmt(entry, &mut self.fmt, self.print_readably)
        });

        self.has_fields = true;
    }

    fn finish(&mut self) -> std::fmt::Result {
        self.result
            .and_then(|_| self.fmt.write_str(self.end_symbol))
    }
}

pub struct List {
    pub(crate) values: Vec<Ast>,
}

impl List {
    pub fn with_values(values: impl IntoIterator<Item = Ast>) -> Ast {
        Ast::of(List {
            values: values.into_iter().collect::<Vec<_>>(),
        })
    }
}

impl Form for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, print_readably: bool) -> std::fmt::Result {
        let mut display = DisplayList::new("(", ")", f, print_readably);
        for value in self.values.iter() {
            display.entry(value);
        }
        display.finish()
    }

    fn get(&self, index: usize) -> Option<&Ast> {
        self.values.get(index)
    }
}


pub struct Vector {
    pub(crate) values: Vec<Ast>,
}

impl Vector {
    pub fn with_values(values: impl IntoIterator<Item = Ast>) -> Ast {
        Ast::of(Vector {
            values: values.into_iter().collect::<Vec<_>>(),
        })
    }
}

impl Form for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, print_readably: bool) -> std::fmt::Result {
        let mut display = DisplayList::new("[", "]", f, print_readably);
        for value in self.values.iter() {
            display.entry(value);
        }
        display.finish()
    }
}


pub struct HashMap {
    pub(crate) values: Vec<Ast>,
}

impl HashMap {
    pub fn with_values(values: impl IntoIterator<Item = Ast>) -> Ast {
        Ast::of(HashMap {
            values: values.into_iter().collect::<Vec<_>>(),
        })
    }
}

impl Form for HashMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, print_readably: bool) -> std::fmt::Result {
        let mut display = DisplayList::new("{", "}", f, print_readably);
        for value in self.values.iter() {
            display.entry(value);
        }
        display.finish()
    }
}
