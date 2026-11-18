//! Combinators for parsing lists of syntax elements.

use std::ops::ControlFlow;

use oxc_allocator::{ArenaBox, ArenaVec};
use oxc_ast::ast::BindingRestElement;
use oxc_span::{GetSpan, Span};

use crate::{
    ParserConfig as Config, ParserImpl,
    diagnostics::{self, ParserDiagnostic},
    lexer::Kind,
};

impl<'a, C: Config> ParserImpl<'a, C> {
    /// Collect a parser list with one exact allocation for up to four elements.
    /// Small elements in longer lists use a separate buffer before spilling into the arena.
    #[inline]
    pub(crate) fn collect_list<T>(
        &mut self,
        mut next: impl FnMut(&mut Self) -> Option<T>,
    ) -> ArenaVec<'a, T> {
        let Some(first) = next(self) else { return ArenaVec::new_in(self) };
        let Some(second) = next(self) else { return ArenaVec::from_value_in(first, self) };
        let Some(third) = next(self) else {
            return ArenaVec::from_array_in([first, second], self);
        };
        let Some(fourth) = next(self) else {
            return ArenaVec::from_array_in([first, second, third], self);
        };
        let Some(fifth) = next(self) else {
            return ArenaVec::from_array_in([first, second, third, fourth], self);
        };
        let prefix = [first, second, third, fourth, fifth];
        if size_of::<T>() <= 16 {
            return self.collect_long_list(prefix, next);
        }
        let mut list = ArenaVec::with_capacity_in(8, self);
        list.extend(prefix);
        while let Some(element) = next(self) {
            list.push(element);
        }
        list
    }

    /// Buffer small list elements separately from the common short-list path so
    /// recursive parsing only pays for this stack space when a list is longer.
    #[inline(never)]
    fn collect_long_list<T>(
        &mut self,
        prefix: [T; 5],
        mut next: impl FnMut(&mut Self) -> Option<T>,
    ) -> ArenaVec<'a, T> {
        let mut tail: [Option<T>; 11] = std::array::from_fn(|_| None);
        for (index, slot) in tail.iter_mut().enumerate() {
            let Some(element) = next(self) else {
                let mut list = ArenaVec::with_capacity_in(5 + index, self);
                list.extend(prefix);
                list.extend(tail.into_iter().flatten());
                return list;
            };
            *slot = Some(element);
        }
        let extra = next(self);
        let mut list = ArenaVec::with_capacity_in(if extra.is_some() { 32 } else { 16 }, self);
        list.extend(prefix);
        list.extend(tail.into_iter().flatten());
        if let Some(extra) = extra {
            list.push(extra);
            while let Some(element) = next(self) {
                list.push(element);
            }
        }
        list
    }

    pub(crate) fn parse_normal_list<F, T>(
        &mut self,
        open: Kind,
        close: Kind,
        mut f: F,
    ) -> ArenaVec<'a, T>
    where
        F: FnMut(&mut Self) -> T,
    {
        self.parse_normal_list_breakable(open, close, |p| Some(f(p)))
    }

    pub(crate) fn parse_normal_list_breakable<F, T>(
        &mut self,
        open: Kind,
        close: Kind,
        mut parse_element: F,
    ) -> ArenaVec<'a, T>
    where
        F: FnMut(&mut Self) -> Option<T>,
    {
        let opening_span = self.cur_token().span();
        self.expect(open);
        let list = self.collect_list(|p| {
            if p.at(close) || p.has_fatal_error() { None } else { parse_element(p) }
        });
        self.expect_closing(close, opening_span);
        list
    }

    pub(crate) fn parse_delimited_list<F, T>(
        &mut self,
        close: Kind,
        separator: Kind,
        opening_span: Span,
        parse_element: F,
    ) -> (ArenaVec<'a, T>, Option<u32>)
    where
        F: FnMut(&mut Self) -> T,
    {
        self.parse_delimited_list_with_prefix(None, close, separator, opening_span, parse_element)
    }

    /// Include an already parsed prefix without consuming a separator for it.
    pub(crate) fn parse_delimited_list_with_prefix<F, T>(
        &mut self,
        mut first: Option<T>,
        close: Kind,
        separator: Kind,
        opening_span: Span,
        mut parse_element: F,
    ) -> (ArenaVec<'a, T>, Option<u32>)
    where
        F: FnMut(&mut Self) -> T,
    {
        let mut finished = self.at(close) || self.has_fatal_error();
        let mut trailing_separator = None;
        let list = self.collect_list(|p| {
            if let Some(first) = first.take() {
                return Some(first);
            }
            if finished {
                return None;
            }
            let element = parse_element(p);
            if let ControlFlow::Break(trailing) =
                p.parse_delimited_list_separator(close, separator, opening_span)
            {
                trailing_separator = trailing;
                finished = true;
            }
            Some(element)
        });
        (list, trailing_separator)
    }

    #[inline]
    fn parse_delimited_list_separator(
        &mut self,
        close: Kind,
        separator: Kind,
        opening_span: Span,
    ) -> ControlFlow<Option<u32>> {
        let kind = self.cur_kind();
        if kind == close || self.has_fatal_error() {
            return ControlFlow::Break(None);
        }
        if kind != separator {
            self.set_fatal_error(diagnostics::expect_closing_or_separator(
                close.to_str(),
                separator.to_str(),
                kind.to_str(),
                self.cur_token().span(),
                opening_span,
            ));
            return ControlFlow::Break(None);
        }
        self.advance(separator);
        let kind = self.cur_kind();
        if kind == close {
            let trailing_separator = self.prev_token_end - 1;
            return ControlFlow::Break(Some(trailing_separator));
        }
        if matches!(kind, Kind::Eof | Kind::Undetermined) {
            return ControlFlow::Break(None);
        }
        ControlFlow::Continue(())
    }

    pub(crate) fn parse_delimited_list_with_rest<E, A, R, D>(
        &mut self,
        close: Kind,
        opening_span: Span,
        parse_element: E,
        parse_rest: R,
        rest_last_diagnostic: D,
    ) -> (ArenaVec<'a, A>, Option<ArenaBox<'a, BindingRestElement<'a>>>)
    where
        E: Fn(&mut Self) -> A,
        R: Fn(&mut Self) -> ArenaBox<'a, BindingRestElement<'a>>,
        D: Fn(Span) -> ParserDiagnostic<'a>,
    {
        let mut rest: Option<ArenaBox<'a, BindingRestElement<'a>>> = None;
        let mut first = true;
        let list = self.collect_list(|p| {
            loop {
                let kind = p.cur_kind();
                if kind == close || p.has_fatal_error() {
                    return None;
                }

                if first {
                    first = false;
                } else {
                    let comma_span = p.cur_token().span();
                    if kind != Kind::Comma {
                        let error = diagnostics::expect_closing_or_separator(
                            close.to_str(),
                            Kind::Comma.to_str(),
                            kind.to_str(),
                            comma_span,
                            opening_span,
                        );
                        p.set_fatal_error(error);
                        return None;
                    }
                    p.bump_any();
                    let kind = p.cur_kind();
                    if kind == close {
                        if rest.is_some() && !p.ctx.has_ambient() {
                            p.error(diagnostics::rest_element_trailing_comma(comma_span));
                        }
                        return None;
                    }
                    if matches!(kind, Kind::Eof | Kind::Undetermined) {
                        return None;
                    }
                }

                if let Some(r) = &rest {
                    p.set_fatal_error(rest_last_diagnostic(r.span()));
                    return None;
                }

                // Re-capture kind to get the current token (may have changed after else branch)
                let kind = p.cur_kind();
                if kind == Kind::Dot3 {
                    rest.replace(parse_rest(p));
                } else {
                    return Some(parse_element(p));
                }
            }
        });

        (list, rest)
    }
}
