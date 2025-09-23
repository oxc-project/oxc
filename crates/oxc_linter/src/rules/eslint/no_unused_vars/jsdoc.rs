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
        if !is_identifier_name(symbol) {
            return;
        }
        self.push(symbol);
    }

    fn try_push_owned(&mut self, symbol: &str) {
        if !is_identifier_name(symbol) {
            return;
        }
        self.symbols.push(Cow::Owned(symbol.to_owned()));
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
                self.try_push_owned(linked);
            }
        } else {
            if let Some(ty) = tag.r#type() {
                let ty = ty.parsed().trim();
                self.try_push(ty);
            }
            let comment = tag.comment().parsed();
            for linked in LinkIter::new(&comment) {
                self.try_push_owned(linked);
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
        const LINK: &str = "@link";

        // find the next @link occurrence
        let link_pos = self.comment[self.pos..].find(LINK)?;
        let mut pos = self.pos + link_pos + LINK.len();

        // skip past leading whitespace
        for c in self.comment.chars().skip(pos) {
            if !c.is_whitespace() {
                break;
            }
            pos += 1;
        }

        // NOTE: won't panic b/c we know slicing `@link` won't hit a utf-8 char boundary
        let comment = &self.comment[pos..];

        let mut end = 0;
        for c in comment.chars() {
            match c {
                // start of MemberExpression
                '.' | '['
                // end of tag
                | '}'
                => break,
                // end of identifier
                c if c.is_whitespace() => break,
                _ => end += 1,
            }
        }

        // `end` needs to be justified to existing comment space (instead of
        // w.r.t current pos)
        self.pos += end;
        if end == 0 {
            // likely end of search. If so, searching for the next "@link" will yield None
            return self.next();
        }

        // make sure end is just before terminators checked for in above iteration
        #[cfg(debug_assertions)]
        {
            let end_char = comment.chars().skip(end - 1).next();
            for terminal in &['.', '[', '}', ' '] {
                assert_ne!(end_char, Some(*terminal))
            }
        }

        let mut link_text = &comment[..end];
        if link_text.ends_with('}') {
            link_text = &link_text[..link_text.len() - 1];
        }

        let link_text = link_text.trim();
        if link_text.is_empty() {
            // move past this @link and try again
            self.next()
        } else {
            Some(link_text)
        }
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
        assert_eq!(links.len(), 0);
    }

    #[test]
    fn test_link_iter_empty() {
        let comment = "";
        let links: Vec<&str> = LinkIter::new(comment).collect();
        assert_eq!(links.len(), 0);
    }

    #[test]
    fn test_utf8_boundary() {
        let comment = "This is a {@link 日本語} reference";
        let links: Vec<&str> = LinkIter::new(comment).collect();
        assert_eq!(links, vec!["日本語"]);
    }

    #[test]
    fn test_broken_tags() {
        let comment = "This is a {@link reference";
        let links: Vec<&str> = LinkIter::new(comment).collect();
        assert_eq!(links, vec!["reference"]); // maybe this is too flexible?
    }
}
