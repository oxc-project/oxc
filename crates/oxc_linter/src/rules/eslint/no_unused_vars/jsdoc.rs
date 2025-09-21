use std::{borrow::Cow, cell::OnceCell, ops};

use crate::LintContext;
use oxc_semantic::JSDocTag;
use oxc_syntax::identifier::is_identifier_name;

type JSDocReference<'a> = Cow<'a, str>;

pub(super) struct JSDocSymbols<'c, 'a> {
    symbols: OnceCell<Vec<JSDocReference<'a>>>,
    ctx: &'c LintContext<'a>,
}

impl<'c, 'a> JSDocSymbols<'c, 'a> {
    pub fn new(ctx: &'c LintContext<'a>) -> Self {
        Self { symbols: OnceCell::new(), ctx }
    }
    pub fn get(&self) -> &Vec<Cow<'a, str>> {
        self.symbols.get_or_init(|| JSDocCollector::new(self.ctx).get_symbols_referenced_in_jsdoc())
    }
    pub fn has(&self, symbol: &str) -> bool {
        self.get().contains(&Cow::Borrowed(symbol))
    }
}

impl<'a> ops::Deref for JSDocSymbols<'_, 'a> {
    type Target = Vec<Cow<'a, str>>;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

pub struct JSDocCollector<'c, 'a> {
    ctx: &'c LintContext<'a>,
    symbols: Vec<JSDocReference<'a>>,
}

impl<'c, 'a> JSDocCollector<'c, 'a> {
    pub fn new(ctx: &'c LintContext<'a>) -> Self {
        Self { ctx, symbols: vec![] }
    }

    #[inline]
    fn push(&mut self, symbol: &'a str) {
        self.symbols.push(Cow::Borrowed(symbol));
    }

    fn try_push(&mut self, symbol: &'a str) {
        if is_identifier_name(symbol) {
            self.push(symbol);
        }
    }

    pub(super) fn get_symbols_referenced_in_jsdoc(mut self) -> Vec<Cow<'a, str>> {
        let doc = self.ctx.jsdoc();
        let iter = doc.iter_all();

        let hint = iter.size_hint();
        self.symbols.reserve(hint.1.unwrap_or(hint.0));

        for jsdoc in iter {
            for tag in jsdoc.tags() {
                self.extract_from_tags(tag);
            }
        }
        self.symbols
    }

    fn extract_from_tags(&mut self, tag: &JSDocTag<'a>) {
        if tag.kind.parsed() == "see" {
            let (ty, name, comment) = tag.type_name_comment();

            // e.g. `@see {@link Foo} maybe trailing comment`
            if let Some(ty) = ty {
                let ty = ty.parsed().trim();
                for linked in LinkIter::new(ty) {
                    self.try_push(linked);
                }
                return;
            }

            // e.g. `@see Foo`
            if let Some(name) = name {
                self.try_push(name.parsed().trim());
            }

            let comment = comment.parsed();
            for linked in LinkIter::new(&comment) {
                if is_identifier_name(linked) {
                    self.symbols.push(Cow::Owned(linked.to_owned()));
                }
            }
        } else {
            if let Some(ty) = tag.r#type() {
                let ty = ty.parsed().trim();
                self.try_push(ty);
            }
            let comment = tag.comment().parsed();
            for linked in LinkIter::new(&comment) {
                if is_identifier_name(linked) {
                    self.symbols.push(Cow::Owned(linked.to_owned()));
                }
            }
        }
    }
}

/// Iterator that finds all `@link` JSDoc tags in a comment
pub struct LinkIter<'a> {
    comment: &'a str,
    pos: usize,
}

impl<'a> LinkIter<'a> {
    pub fn new(comment: &'a str) -> Self {
        Self { comment, pos: 0 }
    }
}

impl<'a> Iterator for LinkIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        // Find the next @link occurrence
        let link_pos = self.comment[self.pos..].find("@link")?;

        let actual_pos = self.pos + link_pos;
        let comment = &self.comment[(actual_pos + "@link".len())..].trim();

        // Find next whitespace
        let end = comment.find(' ').unwrap_or(comment.len());
        if end == 0 {
            // Move past this @link and try again
            self.pos = actual_pos + "@link".len();
            return self.next();
        }

        let mut link_text = &comment[..end];
        if link_text.ends_with('}') {
            link_text = &link_text[..link_text.len() - 1];
        }

        let link_text = link_text.trim();
        if link_text.is_empty() {
            // Move past this @link and try again
            self.pos = actual_pos + "@link".len();
            return self.next();
        }

        // Move past this @link for next iteration
        self.pos = actual_pos + "@link".len();
        Some(link_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_iter_single() {
        let comment = "This is a {@link Foo} reference";
        let links: Vec<&str> = LinkIter::new(comment).collect();
        assert_eq!(links, vec!["Foo"]);
    }

    #[test]
    fn test_link_iter_multiple() {
        let comment = "This has {@link Foo} and {@link Bar} references";
        let links: Vec<&str> = LinkIter::new(comment).collect();
        assert_eq!(links, vec!["Foo", "Bar"]);
    }

    #[test]
    fn test_link_iter_no_links() {
        let comment = "This has no links at all";
        let links: Vec<&str> = LinkIter::new(comment).collect();
        assert_eq!(links, Vec::<&str>::new());
    }

    #[test]
    fn test_link_iter_empty() {
        let comment = "";
        let links: Vec<&str> = LinkIter::new(comment).collect();
        assert_eq!(links, Vec::<&str>::new());
    }
}
