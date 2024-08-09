use std::{
    cell::RefCell,
    fmt::{self, Write},
};

#[derive(Debug)]
pub(crate) struct HtmlWriter {
    inner: RefCell<String>,
}

impl fmt::Write for HtmlWriter {
    #[inline]
    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        self.inner.get_mut().write_fmt(args)
    }

    #[inline]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.inner.get_mut().write_char(c)
    }

    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.inner.get_mut().write_str(s)
    }
}

impl From<HtmlWriter> for String {
    #[inline]
    fn from(html: HtmlWriter) -> Self {
        html.into_inner()
    }
}

impl HtmlWriter {
    pub fn new() -> Self {
        Self { inner: RefCell::new(String::new()) }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { inner: RefCell::new(String::with_capacity(capacity)) }
    }

    pub fn writeln<S: AsRef<str>>(&self, line: S) -> fmt::Result {
        writeln!(self.inner.borrow_mut(), "{}", line.as_ref())
    }

    pub fn into_inner(self) -> String {
        self.inner.into_inner()
    }

    pub fn html<F>(&self, tag: &'static str, attrs: &str, inner: F) -> fmt::Result
    where
        F: FnOnce(&Self) -> fmt::Result,
    {
        // Allocate space for the HTML being printed
        let write_amt_guess = {
            // opening tag. 2 extra for '<' and '>'
            2 + tag.len() + attrs.len() +
            // approximate inner content length
            256 +
            // closing tag. 3 extra for '</' and '>'
            3 + tag.len()
        };
        let mut s = self.inner.borrow_mut();
        s.reserve(write_amt_guess);

        // Write the opening tag
        write!(s, "<{tag}")?;
        if attrs.is_empty() {
            writeln!(s, ">")?;
        } else {
            writeln!(s, " {attrs}>")?;
        }

        // Callback produces the inner content
        drop(s);
        inner(self)?;

        // Write the closing tag
        writeln!(self.inner.borrow_mut(), "</{tag}>")?;

        Ok(())
    }
}

macro_rules! make_tag {
    ($name:ident) => {
        impl HtmlWriter {
            #[inline]
            pub fn $name<F>(&self, attrs: &str, inner: F) -> fmt::Result
            where
                F: FnOnce(&Self) -> fmt::Result,
            {
                self.html(stringify!($name), attrs, inner)
            }
        }
    };
}

make_tag!(div);
make_tag!(span);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_div() {
        let html = HtmlWriter::new();
        html.div("", |html| html.writeln("Hello, world!")).unwrap();

        assert_eq!(
            html.into_inner().as_str(),
            "<div>
Hello, world!
</div>
"
        );
    }
}
