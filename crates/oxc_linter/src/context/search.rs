use super::LintContext;
use oxc_span::Span;
use std::ops::Deref;

pub struct Pattern<T>(pub(super) T);

impl<T> Deref for Pattern<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

macro_rules! into_pattern {
    ($type:ty) => {
        impl From<$type> for Pattern<$type> {
            fn from(value: $type) -> Self {
                Self(value)
            }
        }
    };
}
impl<'a> From<&'a str> for Pattern<&'a str> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}
into_pattern!(char);
into_pattern!(u8);

pub trait Searcher<P> {
    fn find_next(&self, start: u32, pattern: Pattern<P>) -> Option<u32>;
    fn find_prev(&self, end: u32, pattern: Pattern<P>) -> Option<u32>;
}

impl Searcher<&str> for LintContext<'_> {
    fn find_next(&self, start: u32, pattern: Pattern<&str>) -> Option<u32> {
        let source = self.source_after(start);
        let bytes = pattern.as_bytes();
        if bytes.len() < 4 {
            return match bytes.len() {
                1 => self.search_next(start, memchr::memchr_iter(bytes[0], source.as_bytes())),
                2 => self.search_next(
                    start,
                    memchr::memchr2_iter(bytes[0], bytes[1], source.as_bytes()),
                ),
                3 => self.search_next(
                    start,
                    memchr::memchr3_iter(bytes[0], bytes[1], bytes[2], source.as_bytes()),
                ),
                _ => unreachable!(),
            };
        }
        self.search_next(start, source.match_indices(*pattern).map(|(a, _)| a))
    }

    fn find_prev(&self, end: u32, pattern: Pattern<&str>) -> Option<u32> {
        let source = self.source_before(end);
        let bytes = pattern.as_bytes();
        if bytes.len() < 4 {
            return match bytes.len() {
                1 => self.search_prev(end, memchr::memchr_iter(bytes[0], source.as_bytes())),
                2 => self
                    .search_prev(end, memchr::memchr2_iter(bytes[0], bytes[1], source.as_bytes())),
                3 => self.search_prev(
                    end,
                    memchr::memchr3_iter(bytes[0], bytes[1], bytes[2], source.as_bytes()),
                ),
                _ => unreachable!(),
            };
        }
        #[expect(clippy::cast_possible_truncation)]
        source.rmatch_indices(*pattern).map(|(a, _)| a as u32).find(|a| !self.is_inside_comment(*a))
    }
}

impl Searcher<char> for LintContext<'_> {
    #[inline]
    fn find_next(&self, start: u32, pattern: Pattern<char>) -> Option<u32> {
        let source = self.source_after(start);
        let bytes = u32::from(pattern.0);
        match pattern.len_utf8() {
            #[expect(clippy::cast_possible_truncation)]
            1 => self.search_next(start, memchr::memchr_iter(bytes as u8, source.as_bytes())),
            2 => {
                let [first, second, ..] = bytes.to_be_bytes();
                self.search_next(start, memchr::memchr2_iter(first, second, source.as_bytes()))
            }
            3 => {
                let [first, second, third, _] = bytes.to_be_bytes();
                self.search_next(
                    start,
                    memchr::memchr3_iter(first, second, third, source.as_bytes()),
                )
            }
            _ => self.search_next(start, source.match_indices(pattern.0).map(|(a, _)| a)),
        }
    }

    #[inline]
    fn find_prev(&self, end: u32, pattern: Pattern<char>) -> Option<u32> {
        let source = self.source_before(end);
        let bytes = u32::from(pattern.0);
        match pattern.len_utf8() {
            #[expect(clippy::cast_possible_truncation)]
            1 => self.search_prev(end, memchr::memchr_iter(bytes as u8, source.as_bytes())),
            2 => {
                let [first, second, ..] = bytes.to_be_bytes();
                self.search_prev(end, memchr::memchr2_iter(first, second, source.as_bytes()))
            }
            3 => {
                let [first, second, third, _] = bytes.to_be_bytes();
                self.search_prev(end, memchr::memchr3_iter(first, second, third, source.as_bytes()))
            }
            _ => self.search_prev(end, source.rmatch_indices(pattern.0).map(|(a, _)| a)),
        }
    }
}

impl Searcher<u8> for LintContext<'_> {
    fn find_next(&self, start: u32, pattern: Pattern<u8>) -> Option<u32> {
        self.search_next(start, memchr::memchr_iter(pattern.0, self.source_after(start).as_bytes()))
    }

    fn find_prev(&self, end: u32, pattern: Pattern<u8>) -> Option<u32> {
        self.search_prev(end, memchr::memchr_iter(pattern.0, self.source_before(end).as_bytes()))
    }
}

impl<'a> LintContext<'a> {
    fn source_before(&self, end: u32) -> &'a str {
        self.source_range(Span::from(0..end))
    }

    #[expect(clippy::cast_possible_truncation)]
    fn source_after(&self, start: u32) -> &'a str {
        self.source_range(Span::new(start, self.source_text().len() as u32))
    }

    #[expect(clippy::cast_possible_truncation)]
    fn search_next<I: Iterator<Item = usize>>(&self, start: u32, mut iter: I) -> Option<u32> {
        iter.find(|&index| !self.is_inside_comment(start + index as u32))
            .map(|index| start + index as u32)
    }

    #[expect(clippy::cast_possible_truncation)]
    fn search_prev<I: DoubleEndedIterator<Item = usize>>(
        &self,
        _end: u32,
        mut iter: I,
    ) -> Option<u32> {
        // Note: memchr_iter gives indices from the start of the slice (0-based),
        // so we use rfind to get the last occurrence and return it directly as the absolute position
        iter.rfind(|&index| !self.is_inside_comment(index as u32)).map(|index| index as u32)
    }
}
