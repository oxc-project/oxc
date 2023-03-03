use oxc_ast::Span;

mod fixer;

pub use fixer::Fixer;

#[derive(Debug)]
pub struct Fix {
    pub content: String,
    pub span: Span,
}

impl<'a> Fix {
    pub const fn delete(span: Span) -> Self {
        Self { content: String::new(), span }
    }

    pub fn apply(&self, source_text: &'a str) -> String {
        let mut output = String::new();

        let slice = &source_text[..self.span.start as usize];
        let remainder = &source_text[self.span.end as usize..];

        output.push_str(slice);
        output.push_str(&self.content);
        output.push_str(remainder);

        output
    }
}
