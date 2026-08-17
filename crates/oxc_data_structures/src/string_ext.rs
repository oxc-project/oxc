//! [`StringExt`] trait - extension methods for [`String`].

use crate::assert_unchecked;

/// Extension trait for [`String`].
pub trait StringExt {
    /// Append a [`char`] to the end of this `String`, without checking that the `String`
    /// has sufficient spare capacity to hold it.
    ///
    /// This is a faster version of [`String::push`] which skips the check for whether the
    /// `String`'s backing buffer needs to grow (and the cold grow/reallocation branch).
    ///
    /// # SAFETY
    /// `String` must have spare capacity for `ch`
    /// i.e. `self.len() + ch.len_utf8() <= self.capacity()`.
    unsafe fn push_unchecked(&mut self, ch: char);

    /// Append a `&str` to the end of this `String`, without checking that the `String`
    /// has sufficient spare capacity to hold it.
    ///
    /// This is a faster version of [`String::push_str`] which skips the check for whether the
    /// `String`'s backing buffer needs to grow (and the cold grow/reallocation branch).
    ///
    /// # SAFETY
    /// `String` must have spare capacity for `s`
    /// i.e. `self.len() + s.len() <= self.capacity()`.
    unsafe fn push_str_unchecked(&mut self, s: &str);
}

impl StringExt for String {
    #[inline]
    unsafe fn push_unchecked(&mut self, ch: char) {
        // SAFETY: Caller guarantees `self.len() + ch.len_utf8() <= self.capacity()`.
        //
        // The 2nd assertion is the promise to the compiler that no growth is needed. It is phrased as
        // `additional <= capacity - len` (rather than `len + additional <= capacity`) to match
        // the exact shape of the grow check inside `String::push` (`additional > capacity.wrapping_sub(len)`).
        // LLVM does not derive the difference form from the sum form, so only this phrasing removes
        // the capacity check and grow branch.
        // The 1st assertion guarantees `capacity - len` does not wrap.
        unsafe {
            assert_unchecked!(self.len() <= self.capacity());
            assert_unchecked!(ch.len_utf8() <= self.capacity() - self.len());
        }

        self.push(ch);
    }

    #[inline]
    unsafe fn push_str_unchecked(&mut self, s: &str) {
        // SAFETY: Caller guarantees `self.len() + s.len() <= self.capacity()`.
        // See `push_unchecked` above for why the assertions are phrased this way.
        unsafe {
            assert_unchecked!(self.len() <= self.capacity());
            assert_unchecked!(s.len() <= self.capacity() - self.len());
        }

        self.push_str(s);
    }
}

#[cfg(test)]
mod tests {
    use super::StringExt;

    #[test]
    fn push_unchecked_ascii() {
        let mut s = String::with_capacity(8);
        // SAFETY: capacity 8, pushing single bytes well within capacity
        unsafe {
            s.push_unchecked('a');
            s.push_unchecked('$');
        }
        assert_eq!(s, "a$");
    }

    #[test]
    fn push_unchecked_multibyte() {
        let mut s = String::with_capacity(8);
        // SAFETY: capacity 8; 'é' is 2 bytes, '가' is 3 bytes -> 5 bytes total <= 8
        unsafe {
            s.push_unchecked('é');
            s.push_unchecked('가');
        }
        assert_eq!(s, "é가");
    }

    #[test]
    fn push_str_unchecked_works() {
        let mut s = String::with_capacity(16);
        // SAFETY: capacity 16, "foo" + "bar" = 6 bytes <= 16
        unsafe {
            s.push_str_unchecked("foo");
            s.push_str_unchecked("bar");
        }
        assert_eq!(s, "foobar");
    }

    #[test]
    fn push_up_to_exact_capacity() {
        let mut s = String::with_capacity(3);
        // SAFETY: capacity 3; filling it exactly (2 + 1 bytes)
        unsafe {
            s.push_str_unchecked("ab");
            s.push_unchecked('c');
        }
        assert_eq!(s, "abc");
    }
}
