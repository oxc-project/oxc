use std::borrow::Cow;

use super::Fix;

pub struct Fixer<'a> {
    source_text: &'a str,
    fixes: Vec<Fix>,
}

impl<'a> Fixer<'a> {
    pub fn new(source_text: &'a str, fixes: Vec<Fix>) -> Self {
        Self { source_text, fixes }
    }

    pub fn fix(&self) -> Cow<'a, str> {
        if self.fixes.is_empty() {
            Cow::Borrowed(self.source_text)
        } else {
            let mut fixed_source = String::new();
            for fix in &self.fixes {
                fixed_source = fix.apply(self.source_text);
            }
            Cow::Owned(fixed_source)
        }
    }
}
