use super::LintContext;
use oxc_span::Span;
use std::ops::Deref;

pub(crate) struct Pattern<T>(pub(super) T);

impl<T> Deref for Pattern<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Clone for Pattern<T>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Copy for Pattern<T> where T: Copy {}

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

pub(crate) trait Searcher<P> {
    fn find_next(&self, start: u32, pattern: Pattern<P>) -> Option<u32>;
    fn find_prev(&self, end: u32, pattern: Pattern<P>) -> Option<u32>;
}

impl<'a> Searcher<&str> for LintContext<'a> {
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
        source
            .rmatch_indices(*pattern)
            .find(|(a, _)| !self.is_inside_comment(*a as u32))
            .map(|(a, _)| a as u32)
    }
}

impl<'a> Searcher<char> for LintContext<'a> {
    #[inline]
    fn find_next(&self, start: u32, pattern: Pattern<char>) -> Option<u32> {
        let source = self.source_after(start);
        let bytes = u32::from(pattern.0);
        match pattern.len_utf8() {
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

impl<'a> Searcher<u8> for LintContext<'a> {
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

    fn source_after(&self, start: u32) -> &'a str {
        self.source_range(Span::new(start, self.source_text().len() as u32))
    }

    fn search_next<I: Iterator<Item = usize>>(&self, start: u32, mut iter: I) -> Option<u32> {
        iter.find(|&index| !self.is_inside_comment(start + index as u32))
            .map(|index| start + index as u32)
    }

    fn search_prev<I: DoubleEndedIterator<Item = usize>>(
        &self,
        end: u32,
        mut iter: I,
    ) -> Option<u32> {
        iter.rfind(|&index| !self.is_inside_comment(end - index as u32))
            .map(|index| end - index as u32)
    }
}
