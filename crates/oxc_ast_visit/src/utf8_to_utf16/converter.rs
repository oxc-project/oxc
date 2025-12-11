use std::cmp::min;

use oxc_span::Span;
use oxc_syntax::module_record::VisitMutModuleRecord;

use super::{Translation, Utf8ToUtf16};

/// Offset converter, optimized for converting a sequence of offsets in ascending order.
///
/// ## Implementation details
///
/// At any time, one range of the source text is active.
/// This range starts at byte `range_start`, and is `range_len` bytes long.
/// The range describes a stretch of source text which contains only ASCII characters.
/// A UTF-8 offset within this range can be converted to UTF-16 offset with the formula
/// `utf16_offset = utf8_offset - range_start_utf8 + range_start_utf16`.
///
/// [`convert_offset`] has a very fast path for converting offsets in the current range.
///
/// If the offset is outside current range (either before it, or after it), the range containing that
/// offset is identified, and becomes the new current range.
///
/// Therefore, when converting a sequence of offsets in ascending order, the vast majority of
/// conversions will hit the fast path, as they'll be within the same range as the last offset.
/// When an offset is outside current range, there's a cost (`convert_offset_slow`),
/// but then the stretch of source text containing that offset becomes the current range,
/// and the next run of offsets which are before the end of that range will all hit the fast path again.
///
/// [`convert_offset`]: Self::convert_offset
pub struct Utf8ToUtf16Converter<'t> {
    /// Translation table
    translations: &'t [Translation],
    /// UTF-8 offset of start of current range
    range_start_utf8: u32,
    /// Length of current range in UTF-8 bytes
    range_len_utf8: u32,
    /// UTF-16 offset of start of range.
    /// To convert offset within this range:
    /// `utf16_offset = utf8_offset - range_start_utf8 + range_start_utf16`.
    /// We store UTF-16 range start, rather than `utf16_difference`, because it makes
    /// [`Self::convert_offset`] more efficient - 1 less instruction, and 1 less register.
    /// <https://godbolt.org/z/hz5xWGfYn>
    range_start_utf16: u32,
    /// Index of current `Translation`
    index: u32,
    /// Previous UTF-8 offset which [`Utf8ToUtf16Converter::convert_offset`] was called with.
    /// Only used in conformance tests.
    #[cfg(feature = "conformance")]
    previous_offset_utf8: u32,
    /// `true` if offsets will be converted in ascending order.
    /// Only used in conformance tests.
    #[cfg(feature = "conformance")]
    ascending_order: bool,
}

impl<'t> Utf8ToUtf16Converter<'t> {
    /// Create new [`Utf8ToUtf16Converter`].
    ///
    /// # SAFETY
    /// `translations` must contain at least 2 entries.
    #[cfg_attr(not(feature = "conformance"), expect(unused_variables))]
    pub(super) unsafe fn new(translations: &'t [Translation], ascending_order: bool) -> Self {
        debug_assert!(translations.len() >= 2);

        // SAFETY: Caller guarantees `translations` contains at least 2 entries
        let range_len_utf8 = unsafe { translations.get_unchecked(1) }.utf8_offset;

        Self {
            translations,
            range_start_utf8: 0,
            range_start_utf16: 0,
            range_len_utf8,
            index: 0,
            #[cfg(feature = "conformance")]
            previous_offset_utf8: 0,
            #[cfg(feature = "conformance")]
            ascending_order,
        }
    }

    /// Reset this [`Utf8ToUtf16Converter`] to starting position.
    ///
    /// After this call, it's ready to convert an ascending sequence of offsets in most efficient way.
    pub fn reset(&mut self) {
        self.range_start_utf8 = 0;
        self.range_start_utf16 = 0;
        // SAFETY: Caller guaranteed `translations` contains at least 2 entries in `new`
        self.range_len_utf8 = unsafe { self.translations.get_unchecked(1) }.utf8_offset;
        self.index = 0;

        #[cfg(feature = "conformance")]
        {
            self.previous_offset_utf8 = 0;
        }
    }

    /// Convert UTF-8 offset to UTF-16.
    ///
    /// Conversion is faster if `convert_offset` is called with offsets in ascending order.
    ///
    /// # Panics
    /// Panics if `offset` is less than previous offset `convert_offset` was called with,
    /// `conformance` Cargo features is enabled, and `Utf8ToUtf16Converter` was created with
    /// `ascending_order: true`.
    //
    // This method is optimized for the offset being within the current range.
    // This will be the case if `convert_offset` is called with offsets in ascending order,
    // and Unicode characters are fairly rare within the source.
    //
    // This method is written to reduce this common path to as few instructions as possible.
    // It's only 8 instructions on x86_64, with 2 branches, and using only 1 register.
    // https://godbolt.org/z/hz5xWGfYn
    //
    // `#[inline(always)]` because this function is small and on a very hot path.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn convert_offset(&mut self, offset: &mut u32) {
        let utf8_offset = *offset;

        #[cfg(feature = "conformance")]
        if self.ascending_order {
            assert!(utf8_offset >= self.previous_offset_utf8);
            self.previous_offset_utf8 = utf8_offset;
        }

        // When AST has been modified, it may contain unspanned AST nodes.
        // Offset 0 always translates to 0.
        // Don't allow this to fall into the slow path, and don't update the current range,
        // because nodes following this will likely be within same range as the last non-generated node.
        if utf8_offset == 0 {
            return;
        }

        let bytes_from_start_of_range = utf8_offset.wrapping_sub(self.range_start_utf8);
        if bytes_from_start_of_range < self.range_len_utf8 {
            // Offset is within current range.
            // `wrapping_add` because `range_start_utf16` can be `u32::MAX`.
            let result = self.range_start_utf16.wrapping_add(bytes_from_start_of_range);
            *offset = result;
        } else {
            // Offset is outside current range - slow path
            self.convert_offset_slow(offset);
        }
    }

    /// Convert UTF-8 offset to UTF-16 where offset is outside of current range
    /// (either before it, or after it).
    ///
    /// We have 1 method for both cases, and branch here on before/after to keep `convert_offset`
    /// as streamlined as possible.
    #[cold]
    #[inline(never)]
    #[expect(clippy::cast_possible_truncation)]
    fn convert_offset_slow(&mut self, offset: &mut u32) {
        // Find the range containing this offset
        let utf8_offset = *offset;
        let (next_index, range_end_utf8) = if utf8_offset < self.range_start_utf8 {
            #[cfg(feature = "conformance")]
            {
                assert!(!self.ascending_order);
            }

            self.find_range_before(utf8_offset)
        } else {
            self.find_range_after(utf8_offset)
        };

        // `find_range_before` and `find_range_after` always return a `next_index` which is > 0,
        // so `next_index - 1` cannot wrap around
        let index = next_index - 1;

        // SAFETY: `find_range_before` and `find_range_after` always return a `next_index` which is
        // `<= translations.len()`. So `next_index - 1` is in bounds.
        let translation = unsafe { *self.translations.get_unchecked(index) };
        let range_start_utf8 = translation.utf8_offset;
        let utf16_difference = translation.utf16_difference;

        self.index = index as u32;
        self.range_start_utf8 = range_start_utf8;
        self.range_len_utf8 = range_end_utf8 - range_start_utf8;
        // `wrapping_sub` because `utf16_difference` can be `> range_start_utf8` where one of
        // first few characters of source is Unicode. e.g.:
        //
        // * 1st char is Unicode:
        //   * `range_start_utf8 = 1` (offsets in `Translation`s are the offset of the character + 1).
        //   * `utf16_difference` is the length of the Unicode char, which is `> 1`.
        //
        // * If 1st 2 chars are ASCII, but 3rd char is a 4-byte Unicode char:
        //   * `range_start_utf8 = 3`.
        //   * `utf16_difference = 4`.
        self.range_start_utf16 = range_start_utf8.wrapping_sub(utf16_difference);

        *offset = utf8_offset - utf16_difference;
    }

    /// Find range containing `utf8_offset` which is before current range.
    ///
    /// Returns index of range *after* the range containing the offset,
    /// and UTF-8 offset of start of that next range.
    /// i.e. the range containing the offset has index 1 less than the index that's returned by this method.
    ///
    /// The index returned is always `> 0` and `<= self.translations.len()`.
    fn find_range_before(
        &self,
        utf8_offset: u32,
    ) -> (
        usize, // index of next range
        u32,   // UTF-8 offset of start of next range
    ) {
        // TODO: Do linear search here before resorting to binary search.
        // I (@overlookmotel) have left that out for now, because when processing an AST straight
        // from the parser, it has offsets in ascending order, so this method won't be called anyway
        // for AST spans. It may still be called when processing module record, which may be out of order,
        // but module record has few entries, so is not critical for performance.

        // Find the first entry in table *after* the UTF-8 offset. This is the end of the new range.
        // Only need to search before current range, as we already know current range starts after `utf8_offset`.
        // SAFETY: `index` is always in bounds of `translations`.
        let search_ranges = unsafe { self.translations.get_unchecked(..self.index as usize) };
        let next_index =
            search_ranges.partition_point(|translation| utf8_offset >= translation.utf8_offset);

        // SAFETY: We only searched up to `self.index`, which is less than `translations.len()`.
        // So `next_index` is guaranteed to be in bounds.
        let range_end_utf8 = unsafe { self.translations.get_unchecked(next_index) }.utf8_offset;

        // First entry in table is `0, 0`. `partition_point` finds the first entry where
        // `utf8_offset < translation.utf8_offset` (or `translations.len()` if none exists).
        // So guaranteed `next_index > 0`, and `next_index <= translations.len()`.
        (next_index, range_end_utf8)
    }

    /// Find range containing `utf8_offset` which is after current range.
    ///
    /// Returns index of range *after* the range containing the offset,
    /// and UTF-8 offset of start of that next range.
    /// i.e. the range containing the offset has index 1 less than the index that's returned by this method.
    ///
    /// The index returned is always `> 0` and `<= self.translations.len()`.
    fn find_range_after(
        &self,
        utf8_offset: u32,
    ) -> (
        usize, // index of next range
        u32,   // UTF-8 offset of start of next range
    ) {
        // Find the first entry in table *after* the UTF-8 offset. This is the end of the new range.

        // Try linear search first.
        const LINEAR_SEARCH_ITERATIONS: usize = 8;

        // `utf8_offset` is after current range, so there must be another range after this one.
        // We don't need to include next range in search because we know it starts before `utf8_offset`,
        // and we're looking for a range which starts *after* `utf8_offset`.
        let mut next_index = self.index as usize + 1;
        let linear_search_end_index =
            min(next_index + LINEAR_SEARCH_ITERATIONS, self.translations.len());

        while next_index < linear_search_end_index {
            // SAFETY: `linear_search_end_index` is capped at `translations.len()`,
            // so `next_index` is in bounds
            let translation = unsafe { self.translations.get_unchecked(next_index) };

            // For the problematic case "_ऊ_ऊ_", we have:
            // - First ऊ (3-byte) at offsets 1-3, translation.utf8_offset=4
            // - Second ऊ (3-byte) at offsets 5-7, translation.utf8_offset=8
            // When looking for offset 5, we need to realize that the second ऊ starts at offset 5

            // The key insight: translation.utf8_offset is the position AFTER the Unicode character
            // So for the Unicode character, we need to find where it starts
            let prev_end = if next_index > 0 {
                // SAFETY: `next_index > 0` ensures `next_index - 1` is a valid index.
                // `next_index` comes from `partition_point` which returns a value in range `0..=len`,
                // so `next_index - 1` is in range `0..len`, which is valid for `get_unchecked`.
                let prev_translation = unsafe { self.translations.get_unchecked(next_index - 1) };
                prev_translation.utf8_offset
            } else {
                0
            };

            // The Unicode character spans from prev_end to translation.utf8_offset
            // So any offset < translation.utf8_offset and >= prev_end is within this Unicode char
            // But we want to find the ASCII range that comes BEFORE this Unicode char

            // Actually, let's think differently. The issue is that we need to find the correct range.
            // For offset 5 in "_ऊ_ऊ_":
            // - We're currently in range ending at offset 4 (after first ऊ)
            // - Offset 5 is the start of the second ऊ
            // - So offset 5 should be in a Unicode range, not ASCII range
            // - The correct behavior should be to find that offset 5 is within the Unicode char at offsets 5-7

            // Let's check if utf8_offset is within the span of this Unicode character
            if utf8_offset >= prev_end && utf8_offset < translation.utf8_offset {
                // The offset is within this Unicode character
                // We should return this as the range boundary
                return (next_index, translation.utf8_offset);
            }
            next_index += 1;
        }

        // If linear search exhausted all ranges, without finding a range which starts after `utf8_offset`,
        // then offset is in the last range. Return `u32::MAX` as the range end.
        if next_index == self.translations.len() {
            return (next_index, u32::MAX);
        }

        // Binary search over remaining translations.
        // SAFETY: `next_index < self.translations.len()`.
        let search_ranges = unsafe { self.translations.get_unchecked(next_index..) };
        next_index +=
            search_ranges.partition_point(|translation| utf8_offset >= translation.utf8_offset);

        let range_end_utf8 = if next_index < self.translations.len() {
            self.translations[next_index].utf8_offset
        } else {
            // `utf8_offset` is in last range. Return `u32::MAX` as the range end.
            u32::MAX
        };

        // We started search at a non-zero index, so `next_index` cannot be 0.
        // `next_index <= translations.len()`.
        (next_index, range_end_utf8)
    }

    /// Convert [`Span`] from UTF-8 offsets to UTF-16 offsets.
    pub fn convert_span(&mut self, span: &mut Span) {
        self.convert_offset(&mut span.start);
        self.convert_offset(&mut span.end);
    }

    /// Convert UTF-8 offset to line and column using the provided table.
    /// Returns (line, column) where both are 0-based.
    pub fn offset_to_line_column(table: &Utf8ToUtf16, utf8_offset: u32) -> Option<(u32, u32)> {
        table.offset_to_line_column(utf8_offset)
    }

    /// Convert UTF-16 offset to UTF-8 offset.
    pub fn convert_offset_back(&self, offset: &mut u32) {
        // Find first translation whose UTF-16 offset is after `utf16_offset`
        let utf16_offset = *offset;
        let next_index = self.translations.partition_point(|translation| {
            utf16_offset >= translation.utf8_offset - translation.utf16_difference
        });

        // First entry in table is `0, 0`. `partition_point` finds the first entry where
        // `utf16_offset < translation.utf8_offset - translation.utf16_difference`
        // (or `translations.len()` if none exists).
        // So guaranteed `next_index > 0`, and `next_index <= translations.len()`.
        let index = next_index - 1;

        // SAFETY: `next_index <= translations.len()`, so `next_index - 1` is in bounds
        let translation = unsafe { self.translations.get_unchecked(index) };

        *offset += translation.utf16_difference;
    }

    /// Convert [`Span`] from UTF-16 offsets to UTF-8 offsets.
    pub fn convert_span_back(&self, span: &mut Span) {
        self.convert_offset_back(&mut span.start);
        self.convert_offset_back(&mut span.end);
    }
}

impl VisitMutModuleRecord for Utf8ToUtf16Converter<'_> {
    fn visit_span(&mut self, span: &mut Span) {
        self.convert_span(span);
    }
}
