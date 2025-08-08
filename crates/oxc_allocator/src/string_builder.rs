// Most methods are small and cheap. We always want them to be inlined.
#![expect(clippy::inline_always)]

use std::{
    alloc::Layout,
    cmp,
    fmt::{self, Debug, Display, Write},
    mem,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
    slice, str,
};

use oxc_data_structures::assert_unchecked;

use crate::{Allocator, alloc::Alloc};

/// Default minimum capacity
const DEFAULT_MIN_CAPACITY: usize = 8;
const _: () = {
    assert!(DEFAULT_MIN_CAPACITY > 0);
    assert!(DEFAULT_MIN_CAPACITY <= isize::MAX as usize);
};

/// String builder.
///
/// Use to construct a string in arena. Once constructed, convert into an immutable `&str` with lifetime
/// of the allocator.
///
/// `StringBuilder` is designed for temporary use while constructing a string. To make [`push`] etc
/// methods optimally efficient, it is larger than it would be otherwise. It's recommended to convert
/// to a `&str` with [`into_str`] before storing it elsewhere.
///
/// # Example
/// ```
/// use oxc_allocator::{Allocator, StringBuilder};
/// let allocator = Allocator::new();
///
/// let mut s = StringBuilder::new_in(&allocator);
/// s.push_str("foo");
/// s.push('b');
/// s.push_str("ar");
/// s.push_ascii_byte(b'!');
///
/// let s = s.into_str();
/// assert_eq!(s, "foobar!");
/// ```
///
/// [`push`]: Self::push
/// [`into_str`]: Self::into_str
pub struct StringBuilder<'a> {
    /// Start of allocation
    start_ptr: NonNull<u8>,
    /// End of string data
    end_ptr: NonNull<u8>,
    /// End of allocation
    end_capacity_ptr: NonNull<u8>,
    /// Allocator
    allocator: &'a Allocator,
}

impl<'a> StringBuilder<'a> {
    /// Create new [`StringBuilder`] which allocates in given `allocator`,
    /// without allocating any capacity for it.
    ///
    /// # Example
    /// ```
    /// use oxc_allocator::{Allocator, StringBuilder};
    /// let allocator = Allocator::new();
    ///
    /// let mut s = StringBuilder::new_in(&allocator);
    /// assert_eq!(s.len(), 0);
    /// assert_eq!(s.capacity(), 0);
    /// ```
    #[inline(always)]
    pub fn new_in(allocator: &'a Allocator) -> Self {
        let start_ptr = NonNull::dangling();
        Self { start_ptr, end_ptr: start_ptr, end_capacity_ptr: start_ptr, allocator }
    }

    /// Create new [`StringBuilder`], with capacity of `capacity` bytes,
    /// allocated in the given `allocator`.
    ///
    /// # Panics
    /// Panics if `capacity` exceeds `isize::MAX`.
    ///
    /// # Example
    /// ```
    /// use oxc_allocator::{Allocator, StringBuilder};
    /// let allocator = Allocator::new();
    ///
    /// let mut s = StringBuilder::with_capacity_in(10, &allocator);
    /// assert_eq!(s.len(), 0);
    /// assert_eq!(s.capacity(), 10);
    /// ```
    #[inline]
    pub fn with_capacity_in(capacity: usize, allocator: &'a Allocator) -> Self {
        if capacity == 0 {
            return Self::new_in(allocator);
        }

        let layout = Layout::from_size_align(capacity, 1).expect("`capacity` exceeds `isize::MAX");
        let start_ptr = allocator.bump().alloc_layout(layout);
        // SAFETY: We just allocated `capacity` bytes, starting at `start_ptr`
        let end_capacity_ptr = unsafe { start_ptr.add(capacity) };

        Self { start_ptr, end_ptr: start_ptr, end_capacity_ptr, allocator }
    }

    /// Create new [`StringBuilder`], and copy provided `&str` into it,
    /// allocated in the given `allocator`.
    ///
    /// # Example
    /// ```
    /// use oxc_allocator::{Allocator, StringBuilder};
    /// let allocator = Allocator::new();
    ///
    /// let mut s = StringBuilder::from_str_in("foo", &allocator);
    /// assert_eq!(s.len(), 3);
    /// assert_eq!(s.capacity(), 3);
    /// ```
    #[inline]
    pub fn from_str_in(s: &str, allocator: &'a Allocator) -> Self {
        let layout = Layout::for_value(s);
        let start_ptr = allocator.bump().alloc_layout(layout);

        // SAFETY: `s.as_ptr()` is the start of `s` string, so valid for reading `s.len()` bytes.
        // `start_ptr.as_ptr()` is valid for writing `bytes.len()` bytes as we just reserved capacity.
        // We're writing to unused part of allocation, so cannot overlap with `bytes`.
        unsafe { ptr::copy_nonoverlapping(s.as_ptr(), start_ptr.as_ptr(), s.len()) };

        // SAFETY: We just allocated `s.len()` bytes, starting at `start_ptr`
        let end_ptr = unsafe { start_ptr.add(s.len()) };

        Self { start_ptr, end_ptr, end_capacity_ptr: end_ptr, allocator }
    }

    /// Create new [`StringBuilder`] from a fixed-size array of `&str`s concatenated together,
    /// allocated in the given `allocator`.
    ///
    /// This is more efficient than creating a `StringBuilder`, and then making multiple [`push`] calls
    /// to fill it.
    ///
    /// If you're not altering the `StringBuilder` after this call, and just converting it to an `Atom`,
    /// `Atom::from_strs_array_in` may be slightly more efficient.
    ///
    /// # Panics
    ///
    /// Panics if the sum of length of all strings exceeds `usize::MAX`.
    ///
    /// # Example
    /// ```
    /// use oxc_allocator::{Allocator, StringBuilder};
    ///
    /// let allocator = Allocator::default();
    /// let s = StringBuilder::from_strs_array_in(["hello", " ", "world", "!"], &allocator);
    /// assert_eq!(s, "hello world!");
    /// ```
    ///
    /// [`push`]: Self::push
    #[inline(always)]
    pub fn from_strs_array_in<const N: usize>(
        strings: [&str; N],
        allocator: &'a Allocator,
    ) -> StringBuilder<'a> {
        // Calculate total length of all the strings concatenated.
        //
        // We have to use `checked_add` here to guard against additions wrapping around
        // if some of the input `&str`s are very long, or there's many of them.
        //
        // However, `&str`s have max length of `isize::MAX`.
        // https://users.rust-lang.org/t/does-str-reliably-have-length-isize-max/126777
        // Use `assert_unchecked!` to communicate this invariant to compiler, which allows it to
        // optimize out the overflow checks where some of `strings` are static, so their size is known.
        //
        // e.g. `StringBuilder::from_strs_array_in(["__vite_ssr_import_", str, "__"])`, for example,
        // requires no checks at all, because the static parts have total length of 20 bytes,
        // and `str` has max length of `isize::MAX`. `isize::MAX as usize + 20` cannot overflow `usize`.
        // Compiler can see that, and removes the overflow check.
        // https://godbolt.org/z/MGh44Yz5d
        #[expect(clippy::checked_conversions)]
        let total_len = strings.iter().fold(0usize, |total_len, s| {
            let len = s.len();
            // SAFETY: `&str`s have maximum length of `isize::MAX`
            unsafe { assert_unchecked!(len <= (isize::MAX as usize)) };
            total_len.checked_add(len).unwrap()
        });
        assert!(
            isize::try_from(total_len).is_ok(),
            "attempted to create a string longer than `isize::MAX` bytes"
        );

        // Create actual `StringBuilder` in a separate function, to ensure that `from_strs_array_in`
        // is inlined, so that compiler has knowledge to remove the overflow checks above.
        // When some of `strings` are static, this function is usually only a few instructions.
        // Compiler can choose whether or not to inline `from_strs_array_with_total_len`.
        // SAFETY: `total_len` has been calculated correctly above.
        // `total_len` is `<= isize::MAX`.
        unsafe { Self::from_strs_array_with_total_len_in(strings, total_len, allocator) }
    }

    /// Create a new [`StringBuilder`] from a fixed-size array of `&str`s concatenated together,
    /// allocated in the given `allocator`, with provided `total_len`.
    ///
    /// # SAFETY
    /// * `total_len` must be the total length of all `strings` concatenated.
    /// * `total_len` must be `<= isize::MAX`.
    unsafe fn from_strs_array_with_total_len_in<const N: usize>(
        strings: [&str; N],
        total_len: usize,
        allocator: &'a Allocator,
    ) -> StringBuilder<'a> {
        if total_len == 0 {
            return Self::new_in(allocator);
        }

        // Allocate `total_len` bytes.
        // SAFETY: Caller guarantees `total_len <= isize::MAX`.
        let layout = unsafe { Layout::from_size_align_unchecked(total_len, 1) };
        let start_ptr = allocator.bump().alloc_layout(layout);

        let mut end_ptr = start_ptr;
        for str in strings {
            let src_ptr = str.as_ptr();
            let len = str.len();

            // SAFETY:
            // `src` is obtained from a `&str` with length `len`, so is valid for reading `len` bytes.
            // `end_ptr` is within bounds of the allocation. So is `end_ptr + len`.
            // `u8` has no alignment requirements, so `src_ptr` and `end_ptr` are sufficiently aligned.
            // No overlapping, because we're copying from an existing `&str` to a newly allocated buffer.
            unsafe { ptr::copy_nonoverlapping(src_ptr, end_ptr.as_ptr(), len) };

            // SAFETY: We allocated sufficient capacity for all the strings concatenated.
            // So `end_ptr.add(len)` cannot go out of bounds.
            end_ptr = unsafe { end_ptr.add(len) };
        }

        debug_assert_eq!(end_ptr.as_ptr() as usize - start_ptr.as_ptr() as usize, total_len);

        Self { start_ptr, end_ptr, end_capacity_ptr: end_ptr, allocator }
    }

    /// Get number of bytes in the string.
    #[inline(always)]
    pub fn len(&self) -> usize {
        // SAFETY: `end_ptr` is always equal to or after `start_ptr`
        unsafe { self.end_ptr.offset_from_unsigned(self.start_ptr) }
    }

    /// Returns `true` if string is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.end_ptr == self.start_ptr
    }

    /// Get current capacity of [`StringBuilder`]'s buffer.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        // SAFETY: `end_capacity_ptr` is always equal to or after `start_ptr`
        unsafe { self.end_capacity_ptr.offset_from_unsigned(self.start_ptr) }
    }

    /// Consume [`StringBuilder`] and produce a `&'a str` with lifetime of the arena.
    #[inline(always)]
    pub fn into_str(self) -> &'a str {
        let s = self.as_str();
        // Extend lifetime to `'a`.
        // SAFETY: String data is stored in arena, so lives as long as the borrow of the arena does
        // (until `Allocator` is reset).
        unsafe { mem::transmute::<&str, &'a str>(s) }
    }

    /// Push ASCII byte onto end of the string.
    ///
    /// # Panics
    /// Panics if `byte` is not ASCII.
    #[inline(always)]
    pub fn push_ascii_byte(&mut self, byte: u8) {
        assert!(byte.is_ascii(), "`byte` is not ASCII");
        // SAFETY: Just checked byte is ASCII
        unsafe { self.push_byte_unchecked(byte) };
    }

    /// Push a byte onto end of the string, without check for UTF-8 validity.
    ///
    /// # SAFETY
    ///
    /// Caller must ensure either:
    ///
    /// 1. `byte` is ASCII, or
    /// 2. Further calls to `push_byte_unchecked` and/or other `push` methods complete the UTF-8
    ///    byte sequence, so the buffer ends up containing a valid UTF-8 string.
    #[inline(always)]
    pub unsafe fn push_byte_unchecked(&mut self, byte: u8) {
        if self.end_ptr == self.end_capacity_ptr {
            // Full to capacity. Grow the allocation.
            // SAFETY: We just checked allocation is full to capacity.
            unsafe { self.grow_one() };
        }

        // SAFETY: If there wasn't already 1 byte spare capacity, `grow` above makes sure there now is.
        // Therefore safe to write a byte to `end_ptr`, and `end_ptr + 1` cannot exceed `end_capacity_ptr`.
        unsafe {
            self.end_ptr.write(byte);
            self.end_ptr = self.end_ptr.add(1);
        }
    }

    /// Push ASCII byte onto start of the string.
    ///
    /// # Panics
    /// Panics if `byte` is not ASCII.
    #[inline(always)]
    pub fn push_ascii_byte_start(&mut self, byte: u8) {
        assert!(byte.is_ascii(), "`byte` is not ASCII");
        // SAFETY: Just checked byte is ASCII
        unsafe { self.push_byte_start_unchecked(byte) };
    }

    /// Push a byte onto start of the string, without check for UTF-8 validity.
    ///
    /// # SAFETY
    ///
    /// Caller must ensure either:
    ///
    /// 1. `byte` is ASCII, or
    /// 2. Further calls to `push_byte_start_unchecked` complete the UTF-8 byte sequence,
    ///    so the buffer ends up containing a valid UTF-8 string.
    #[inline(always)]
    pub unsafe fn push_byte_start_unchecked(&mut self, byte: u8) {
        if self.end_ptr == self.end_capacity_ptr {
            // Full to capacity. Grow the allocation.
            // SAFETY: We just checked allocation is full to capacity.
            // TODO: `grow_one` copies the data, and then we copy it again below.
            // Once we have a `grow_in_place` method on allocator, avoid this double-copy.
            unsafe { self.grow_one() };
        }

        // Shift up string data by 1 byte, write `byte` to the start, and increase length by 1.
        // SAFETY: If there wasn't already 1 byte spare capacity, `grow` above makes sure there now is.
        // Therefore writing `self.len()` bytes from `start_ptr + 1` is in bounds of the allocation,
        // and `end_ptr + 1` cannot exceed `end_capacity_ptr`.
        unsafe {
            ptr::copy(self.start_ptr.as_ptr(), self.start_ptr.as_ptr().add(1), self.len());
            self.start_ptr.write(byte);
            self.end_ptr = self.end_ptr.add(1);
        }
    }

    /// Push bytes to the string, without check for UTF-8 validity.
    ///
    /// # SAFETY
    ///
    /// Caller must ensure either:
    ///
    /// 1. `bytes` contains only ASCII bytes, or
    /// 2. Further calls to `push_byte_unchecked` and/or other `push` methods complete the UTF-8
    ///    byte sequence, so the buffer ends up containing a valid UTF-8 string.
    #[inline(always)]
    pub unsafe fn push_bytes_unchecked(&mut self, bytes: &[u8]) {
        let len = bytes.len();

        self.reserve(len);

        // SAFETY: `bytes.as_ptr()` is the start of `bytes` slice, so valid for reading `len` bytes.
        // `self.end.as_ptr()` is valid for writing `len` bytes as we just reserved capacity.
        // We're writing to unused part of allocation, so cannot overlap with `bytes`.
        unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), self.end_ptr.as_ptr(), len) };

        // SAFETY: We reserved extra capacity of `len` bytes, so `end_ptr + len` cannot exceed
        // `end_capacity_ptr`
        self.end_ptr = unsafe { self.end_ptr.add(len) };
    }

    /// Push a `char` onto end of the string.
    #[inline(always)]
    pub fn push(&mut self, c: char) {
        if c.is_ascii() {
            // SAFETY: Just checked `c` is ASCII
            unsafe { self.push_byte_unchecked(c as u8) };
        } else {
            let mut buff = [0; 4];
            let s = c.encode_utf8(&mut buff);
            self.push_str(s);
        }
    }

    /// Push a `&str` onto end of the string.
    #[inline(always)]
    pub fn push_str(&mut self, s: &str) {
        // SAFETY: Concatenating a `&str` onto end of a valid UTF-8 string results in a valid UTF-8 string
        unsafe { self.push_bytes_unchecked(s.as_bytes()) };
    }

    /// Push an ASCII byte onto end the string, repeated `n` times.
    ///
    /// # Panics
    /// Panics if `byte` is not ASCII.
    ///
    /// # Example
    /// ```
    /// use oxc_allocator::{Allocator, StringBuilder};
    /// let allocator = Allocator::new();
    ///
    /// let mut s = StringBuilder::new_in(&allocator);
    /// s.push_str("hello");
    /// s.push_ascii_byte_repeat(b'!', 3);
    ///
    /// let s = s.into_str();
    /// assert_eq!(s, "hello!!!");
    /// ```
    #[inline]
    pub fn push_ascii_byte_repeat(&mut self, byte: u8, n: usize) {
        assert!(byte.is_ascii(), "`byte` is not ASCII");

        self.reserve(n);

        // SAFETY: `reserve` ensures there are at least `n` bytes of spare capacity.
        // So writing `n` bytes from `end_ptr` cannot write past the end of the allocation.
        // Ditto adding `n` to `end_ptr`.
        // `reserve` would have panicked if `n > isize::MAX` or total length `> isize::MAX`.
        // `byte` is ASCII, so cannot produce an invalid UTF-8 string.
        unsafe {
            ptr::write_bytes(self.end_ptr.as_ptr(), byte, n);
            self.end_ptr = self.end_ptr.add(n);
        }
    }

    /// Reserve space for `additional` bytes in the buffer.
    ///
    /// After this call, there's guaranteed to be at least `additional` bytes of excess capacity
    /// in the buffer.
    ///
    /// # Panics
    /// Panics if unable to reserve sufficient capacity.
    #[inline(always)]
    pub fn reserve(&mut self, additional: usize) {
        // SAFETY: `end_capacity` is always equal to or after `end`
        let free_bytes = unsafe { self.end_capacity_ptr.offset_from_unsigned(self.end_ptr) };
        if free_bytes < additional {
            // Insufficient capacity for `additional` bytes. Grow the allocation.
            // SAFETY: We just checked allocation is full to capacity.
            // `additional` can't be 0.
            unsafe { self.grow(additional) };
        }
    }

    /// Grow buffer, so it contains space for at least `additional` bytes.
    ///
    /// # SAFETY
    /// * Allocation must be full to capacity.
    /// * `additional` must be greater than 0.
    #[cold]
    #[inline(never)]
    unsafe fn grow(&mut self, additional: usize) {
        let current_capacity = self.capacity();
        if current_capacity == 0 {
            // Ensure don't allocate less than 8 bytes
            let additional = cmp::max(additional, DEFAULT_MIN_CAPACITY);
            let layout = Layout::from_size_align(additional, 1)
                .expect("attempt to grow `StringBuilder` beyond `isize::MAX` bytes");
            let start_ptr = self.allocator.bump().alloc_layout(layout);
            self.start_ptr = start_ptr;
            self.end_ptr = start_ptr;
            // SAFETY: Just allocated `additional` bytes, starting at `start_ptr`,
            // so `end_capacity_ptr` is end of the allocation
            self.end_capacity_ptr = unsafe { start_ptr.add(additional) };
        } else {
            // At least double the capacity. Don't allocate less than 8 bytes.
            let additional = cmp::max(additional, current_capacity);
            // TODO: Once we bump forwards for strings, ensure capacity is never less than
            // `DEFAULT_MIN_CAPACITY` in `with_capacity_in` and `from_str_in`, then can remove this line
            let additional = cmp::max(additional, DEFAULT_MIN_CAPACITY);

            let new_capacity = current_capacity
                .checked_add(additional)
                .expect("attempt to grow `StringBuilder` beyond `isize::MAX` bytes");

            // SAFETY: We already allocated this, so must have a valid layout
            let old_layout = unsafe { Layout::from_size_align_unchecked(current_capacity, 1) };
            let new_layout = Layout::from_size_align(new_capacity, 1)
                .expect("attempt to grow `StringBuilder` beyond `isize::MAX` bytes");

            let len = self.len();

            // SAFETY: Previously allocated at `start_ptr` with `old_layout`.
            // `new_layout` is larger than `old_layout`.
            let new_start_ptr =
                unsafe { self.allocator.bump().grow(self.start_ptr, old_layout, new_layout) };

            self.start_ptr = new_start_ptr;
            // SAFETY: `len` is always less than or equal to capacity.
            // Capacity has now increased, so `len` is less than capacity.
            // So `new_start_ptr + len` is in bounds.
            self.end_ptr = unsafe { new_start_ptr.add(len) };
            // SAFETY: Just allocated `new_capacity` bytes, so `new_start_ptr + new_capacity`
            // is end of the allocation
            self.end_capacity_ptr = unsafe { new_start_ptr.add(new_capacity) };
        }
    }

    /// Grow buffer, so it contains space for at least 1 more byte.
    ///
    /// # SAFETY
    /// Allocation must be full to capacity.
    #[cold]
    #[inline(never)]
    unsafe fn grow_one(&mut self) {
        let current_capacity = self.capacity();
        if current_capacity == 0 {
            // Ensure don't allocate less than 8 bytes.
            // SAFETY: `DEFAULT_MIN_CAPACITY` is a valid size for `Layout` with align 1.
            let layout = unsafe { Layout::from_size_align_unchecked(DEFAULT_MIN_CAPACITY, 1) };
            let start_ptr = self.allocator.bump().alloc_layout(layout);
            self.start_ptr = start_ptr;
            self.end_ptr = start_ptr;
            // SAFETY: Just allocated `DEFAULT_MIN_CAPACITY` bytes, starting at `start_ptr`,
            // so `end_capacity` is end of the allocation
            self.end_capacity_ptr = unsafe { start_ptr.add(DEFAULT_MIN_CAPACITY) };
        } else {
            // At least double the capacity. Don't allocate less than 8 bytes.
            // TODO: Once we bump forwards for strings, ensure capacity is never less than
            // `DEFAULT_MIN_CAPACITY` in `with_capacity_in` and `from_str_in`, then can remove this line.
            let additional = cmp::max(current_capacity, DEFAULT_MIN_CAPACITY);

            let new_capacity = current_capacity
                .checked_add(additional)
                .expect("attempt to grow `StringBuilder` beyond `isize::MAX` bytes");

            // SAFETY: We already allocated this, so must have a valid layout
            let old_layout = unsafe { Layout::from_size_align_unchecked(current_capacity, 1) };
            let new_layout = Layout::from_size_align(new_capacity, 1)
                .expect("attempt to grow `StringBuilder` beyond `isize::MAX` bytes");

            let len = self.len();

            // SAFETY: Previously allocated at `start_ptr` with `old_layout`.
            // `new_layout` is larger than `old_layout`.
            let new_start_ptr =
                unsafe { self.allocator.bump().grow(self.start_ptr, old_layout, new_layout) };

            self.start_ptr = new_start_ptr;
            // SAFETY: `len` is always less than or equal to capacity.
            // Capacity has now increased, so `len` is less than capacity.
            // So `new_start_ptr + len` is in bounds.
            self.end_ptr = unsafe { new_start_ptr.add(len) };
            // SAFETY: Just allocated `new_capacity` bytes, so `new_start_ptr + new_capacity`
            // is end of the allocation
            self.end_capacity_ptr = unsafe { new_start_ptr.add(new_capacity) };
        }
    }

    /// Get string as a `&str`.
    #[inline(always)]
    pub fn as_str(&self) -> &str {
        // SAFETY: Slice between `start` and `end` is within the same allocation.
        // All bytes between `start` and `end` are initialized, and form a valid UTF-8 string.
        unsafe {
            let len = self.len();
            let slice = slice::from_raw_parts(self.start_ptr.as_ptr(), len);
            str::from_utf8_unchecked(slice)
        }
    }

    /// Get string as a `&mut str`.
    #[inline(always)]
    pub fn as_mut_str(&mut self) -> &mut str {
        // SAFETY: Slice between `start` and `end` is within the same allocation.
        // All bytes between `start` and `end` are initialized, and form a valid UTF-8 string.
        unsafe {
            let len = self.len();
            let slice = slice::from_raw_parts_mut(self.start_ptr.as_ptr(), len);
            str::from_utf8_unchecked_mut(slice)
        }
    }
}

impl Deref for StringBuilder<'_> {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl DerefMut for StringBuilder<'_> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

impl PartialEq for StringBuilder<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for StringBuilder<'_> {}

impl PartialEq<str> for StringBuilder<'_> {
    fn eq(&self, s: &str) -> bool {
        self.as_str() == s
    }
}

impl PartialEq<StringBuilder<'_>> for str {
    fn eq(&self, s: &StringBuilder<'_>) -> bool {
        self == s.as_str()
    }
}

impl PartialEq<&str> for StringBuilder<'_> {
    fn eq(&self, s: &&str) -> bool {
        self.as_str() == *s
    }
}

impl PartialEq<StringBuilder<'_>> for &str {
    fn eq(&self, s: &StringBuilder<'_>) -> bool {
        *self == s.as_str()
    }
}

impl Write for StringBuilder<'_> {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }

    #[inline]
    fn write_char(&mut self, c: char) -> fmt::Result {
        self.push(c);
        Ok(())
    }
}

impl Display for StringBuilder<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl Debug for StringBuilder<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}
