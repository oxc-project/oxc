use oxc_ast::Trivias;
use oxc_span::Span;

#[derive(Debug, Default)]
pub struct TriviaBuilder {
    trivias: Trivias,
}

impl TriviaBuilder {
    pub fn build(self) -> Trivias {
        self.trivias
    }

    #[allow(unused)]
    pub fn add_single_line_comment(&mut self, start: u32, end: u32) {
        // skip leading `//`
        self.trivias.add_single_line_comment(Span::new(start + 2, end));
    }

    #[allow(unused)]
    pub fn add_multi_line_comment(&mut self, start: u32, end: u32) {
        // skip leading `/*` and trailing `*/`
        self.trivias.add_multi_line_comment(Span::new(start + 2, end - 2));
    }
}
