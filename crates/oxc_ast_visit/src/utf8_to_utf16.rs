//! Convert UTF-8 span offsets to UTF-16.

use std::{cmp::min, slice};

use oxc_ast::ast::{Comment, Program};
use oxc_data_structures::pointer_ext::PointerExt;
use oxc_span::Span;
use oxc_syntax::module_record::{ModuleRecord, VisitMutModuleRecord};

use crate::VisitMut;

/// Conversion table of UTF-8 span offsets to UTF-16.
pub struct Utf8ToUtf16 {
    translations: Vec<Translation>,
}

/// A translation from UTF-8 offset to UTF-16 offset.
#[derive(Clone, Copy)]
#[repr(align(8))]
struct Translation {
    /// UTF-8 byte offset.
    /// This is the UTF-8 offset of start of a Unicode character PLUS 1.
    /// So this offset sits in the middle of the Unicode character.
    /// Exception is the dummy first entry in table, where it's 0.
    utf8_offset: u32,
    /// Number to subtract from UTF-8 byte offset to get UTF-16 char offset
    /// for UTF-8 offsets after `utf8_offset`
    utf16_difference: u32,
}

impl Utf8ToUtf16 {
    /// Create new [`Utf8ToUtf16`] conversion table from source text.
    pub fn new(source_text: &str) -> Self {
        let mut translations = Vec::with_capacity(16);

        translations.push(Translation { utf8_offset: 0, utf16_difference: 0 });

        build_translations(source_text, &mut translations);

        // If no translations have been added after the first `0, 0` dummy, then source is entirely ASCII.
        // Remove the dummy entry.
        // Therefore, `translations` always has at least 2 entries, if it has any.
        if translations.len() == 1 {
            // In conformance tests, force offset conversion to happen on all inputs,
            // even if they are pure ASCII
            if cfg!(feature = "conformance") {
                translations.push(Translation { utf8_offset: u32::MAX, utf16_difference: 0 });
            } else {
                translations.clear();
            }
        }

        Self { translations }
    }

    /// Create a [`Utf8ToUtf16Converter`] converter, to convert offsets from UTF-8 to UTF-16.
    ///
    /// The converter is optimized for converting a sequence of offsets in ascending order.
    /// It will also correctly handle offsets in any order, but at a performance cost.
    ///
    /// Returns `None` if the source text is entirely ASCII, and so requires no conversion.
    pub fn converter(&self) -> Option<Utf8ToUtf16Converter<'_>> {
        if self.translations.is_empty() {
            None
        } else {
            // SAFETY: `translations` contains at least 2 entries if it's not empty.
            // We just checked it's not empty.
            Some(unsafe { Utf8ToUtf16Converter::new(&self.translations, false) })
        }
    }

    /// Convert all spans in AST to UTF-16.
    pub fn convert_program(&self, program: &mut Program<'_>) {
        if let Some(mut converter) = self.converter() {
            converter.visit_program(program);
        }
    }

    /// Convert all spans in AST to UTF-16.
    ///
    /// Additionally, checks that conversion of offsets during traversal via [`Utf8ToUtf16Converter`]
    /// happens in ascending order of offset. Panics if it doesn't.
    ///
    /// This results in the fastest conversion, and [`Utf8ToUtf16Converter`] is designed to ensure that
    /// [`Utf8ToUtf16Converter::convert_offset`] is called with offsets strictly in ascending order.
    /// This should always be the case when the AST has come direct from parser.
    /// It might well not be the case in an AST which has been modified, e.g. by transformer or minifier.
    ///
    /// This method is for use only in conformance tests, and requires `conformance` Cargo feature.
    ///
    /// # Panics
    ///
    /// Panics if offsets are converted out of order.
    #[cfg(feature = "conformance")]
    pub fn convert_program_with_ascending_order_checks(&self, program: &mut Program<'_>) {
        assert!(self.translations.len() >= 2);

        // SAFETY: We just checked `translations` contains at least 2 entries
        let mut converter = unsafe { Utf8ToUtf16Converter::new(&self.translations, true) };
        converter.visit_program(program);
    }

    /// Convert all spans in comments to UTF-16.
    pub fn convert_comments(&self, comments: &mut [Comment]) {
        if let Some(mut converter) = self.converter() {
            for comment in comments {
                converter.convert_span(&mut comment.span);
            }
        }
    }

    /// Convert all spans in `ModuleRecord` to UTF-16.
    pub fn convert_module_record(&self, module_record: &mut ModuleRecord<'_>) {
        if let Some(mut converter) = self.converter() {
            converter.visit_module_record(module_record);
        }
    }
}

/// Offset converter, optimized for converting a sequence of offsets in ascending order.
///
/// ## Implementation details
///
/// At any time, one range of the source text is active.
/// This range starts at byte `range_start`, and is `range_len` bytes long.
/// The range describes a stretch of source text which contains only ASCII characters.
/// A UTF-8 offset within this range can be converted to UTF-16 offset with the formula
/// `utf16_offset = (utf8_offset - range_start_utf8).wrapping_add(range_start_utf16)`.
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
    /// `utf16_offset = (utf8_offset - range_start_utf8).wrapping_add(range_start_utf16)`.
    /// Note: `range_start_utf16` is calculated and used with wrapping addition/subtraction,
    /// because it can wrap around when a Unicode character very close to start of source.
    /// We store UTF-16 range start, rather than `utf16_difference`, because it makes
    /// [`Self::convert_offset`] more efficient - 1 less instruction, and 1 less register.
    /// <https://godbolt.org/z/1xnx1v17T>
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
    unsafe fn new(translations: &'t [Translation], ascending_order: bool) -> Self {
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
    // https://godbolt.org/z/1xnx1v17T
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
        if bytes_from_start_of_range <= self.range_len_utf8 {
            // Offset is within current range.
            // `wrapping_add` because `range_start_utf16` can be `u32::MAX`.
            *offset = self.range_start_utf16.wrapping_add(bytes_from_start_of_range);
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
        // Only need to search before current range, as we already current range starts after `utf8_offset`.
        // SAFETY: `index` is always in bounds of `translations`.
        let search_ranges = unsafe { self.translations.get_unchecked(..self.index as usize) };
        let next_index =
            search_ranges.partition_point(|translation| utf8_offset >= translation.utf8_offset);

        // SAFETY: We only searched up to `self.index`, which is less than `translations.len()`.
        // So `next_index` is guaranteed to be in bounds.
        let range_end_utf8 = unsafe { self.translations.get_unchecked(next_index) }.utf8_offset;

        // First entry in table is `0, 0`. `partition_point` finds the first entry where
        // `utf8_offset >= translation.utf8_offset` (or `translations.len()` if none exists).
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
        // We don't need to include next range in search because we know it starts before `uft8_offset`,
        // and we're looking for a range which starts *after* `uft8_offset`.
        let mut next_index = self.index as usize + 2;
        let linear_search_end_index =
            min(next_index + LINEAR_SEARCH_ITERATIONS, self.translations.len());
        while next_index < linear_search_end_index {
            // SAFETY: `linear_search_end_index` is capped at `translations.len()`,
            // so `next_index` is in bounds
            let translation = unsafe { self.translations.get_unchecked(next_index) };
            if utf8_offset < translation.utf8_offset {
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
}

impl VisitMutModuleRecord for Utf8ToUtf16Converter<'_> {
    fn visit_span(&mut self, span: &mut Span) {
        self.convert_span(span);
    }
}

const CHUNK_SIZE: usize = 32;
const CHUNK_ALIGNMENT: usize = align_of::<AlignedChunk>();
const _: () = {
    assert!(CHUNK_SIZE >= CHUNK_ALIGNMENT);
    assert!(CHUNK_SIZE % CHUNK_ALIGNMENT == 0);
    assert!(CHUNK_SIZE == size_of::<AlignedChunk>());
};

#[repr(C, align(16))]
struct AlignedChunk([u8; CHUNK_SIZE]);

impl AlignedChunk {
    /// Check if chunk contains any non-ASCII bytes.
    ///
    /// This boils down to 3 x SIMD ops to check 32 bytes in one go.
    /// <https://godbolt.org/z/E7jc51Mf5>
    #[inline]
    fn contains_unicode(&self) -> bool {
        for index in 0..CHUNK_SIZE {
            if !self.0[index].is_ascii() {
                return true;
            }
        }
        false
    }

    /// Get contents of chunk as a `&[u8]` slice.
    #[inline]
    fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

/// Build table of translations from UTF-8 offsets to UTF-16 offsets.
///
/// Process bulk of source text in chunks of 32 bytes, using SIMD instructions.
/// This should be much faster than byte-by-byte processing, assuming non-ASCII chars are rare in source code.
///
/// Translation as follows:
///
/// * 1-byte UTF-8 sequence
///   = 1st byte 0xxxxxxx (0 - 0x7F)
///   -> 1 x UTF-16 char
///   UTF-16 len = UTF-8 len
/// * 2-byte UTF-8 sequence
///   = 1st byte 110xxxxx (0xC0 - 0xDF), remaining bytes 10xxxxxx (0x80 - 0xBF)
///   -> 1 x UTF-16
///   UTF-16 len = UTF-8 len - 1
/// * 3-byte UTF-8 sequence
///   = 1st byte 1110xxxx (0xE0 - 0xEF), remaining bytes 10xxxxxx (0x80 - 0xBF)
///   -> 1 x UTF-16
///   UTF-16 len = UTF-8 len - 2
/// * 4-byte UTF-8 sequence
///   = 1st byte 1111xxxx (0xF0 - 0xFF), remaining bytes 10xxxxxx (0x80 - 0xBF)
///   -> 2 x UTF-16
///   UTF-16 len = UTF-8 len - 2
///
/// So UTF-16 offset = UTF-8 offset - count of bytes `>= 0xC0` - count of bytes `>= 0xE0`
fn build_translations(source_text: &str, translations: &mut Vec<Translation>) {
    // Running counter of difference between UTF-8 and UTF-16 offset
    let mut utf16_difference = 0;

    // Closure that processes a slice of bytes
    let mut process_slice = |slice: &[u8], start_offset: usize| {
        for (index, &byte) in slice.iter().enumerate() {
            #[expect(clippy::cast_possible_truncation)]
            if byte >= 0xC0 {
                let difference_for_this_byte = u32::from(byte >= 0xE0) + 1;
                utf16_difference += difference_for_this_byte;
                // Record `offset + 1` not `offset`, because it's only offsets *after* this
                // Unicode character that need to be shifted.
                // `offset + 1` cannot overflow, because source is limited to `u32::MAX` bytes,
                // so a multi-byte Unicode character can't start at offset `u32::MAX`, because there
                // isn't space to complete the multi-byte sequence, which would not be a valid `&str`.
                let offset = start_offset + index;
                let utf8_offset = (offset + 1) as u32;
                translations.push(Translation { utf8_offset, utf16_difference });
            }
        }
    };

    // If source text is short, just process byte-by-byte
    let bytes = source_text.as_bytes();
    if bytes.len() < CHUNK_SIZE {
        process_slice(bytes, 0);
        return;
    }

    // Process first few bytes of source
    let start_ptr = bytes.as_ptr();
    let mut remaining_len = bytes.len();

    let mut ptr = start_ptr;

    let first_chunk_len = ptr.align_offset(CHUNK_ALIGNMENT);
    if first_chunk_len > 0 {
        // SAFETY: `first_chunk_len` is less than `CHUNK_ALIGNMENT`, which in turn is no bigger than
        // `CHUNK_SIZE`. We already exited if source is shorter than `CHUNK_SIZE` bytes,
        // so there must be at least `first_chunk_len` bytes in source.
        let first_chunk = unsafe { slice::from_raw_parts(ptr, first_chunk_len) };
        process_slice(first_chunk, 0);
        // SAFETY: For reasons given above, `first_chunk_len` must be in bounds
        ptr = unsafe { ptr.add(first_chunk_len) };
        remaining_len -= first_chunk_len;
    }

    debug_assert!((ptr as usize) % CHUNK_ALIGNMENT == 0);

    // Process main body as aligned chunks of 32 bytes.
    //
    // We've aligned `ptr` to `CHUNK_ALIGNMENT`, so can now read the rest of source as `AlignedChunk`s
    // (apart from a few bytes on end which may not be enough to make a whole `AlignedChunk`).
    //
    // Do a fast check for any non-ASCII bytes in each chunk using SIMD.
    // Only if that finds non-ASCII bytes, process the chunk byte-by-byte.

    // Get length of body of `bytes` which we can process as `AlignedChunk`s.
    // Round down remaining length to a multiple of `CHUNK_SIZE`.
    let body_len = remaining_len & !(CHUNK_SIZE - 1);
    remaining_len -= body_len;
    // SAFETY: `body_len` is less than number of bytes remaining in `bytes`, so in bounds
    let body_end_ptr = unsafe { ptr.add(body_len) };

    debug_assert!(body_end_ptr as usize <= start_ptr as usize + bytes.len());
    debug_assert!((body_end_ptr as usize - ptr as usize) % CHUNK_SIZE == 0);

    while ptr < body_end_ptr {
        // SAFETY: `ptr` was aligned to `CHUNK_ALIGNMENT` after processing 1st chunk.
        // It is incremented in this loop by `CHUNK_SIZE`, which is a multiple of `CHUNK_ALIGNMENT`,
        // so `ptr` remains always aligned for `CHUNK_ALIGNMENT`.
        // `ptr < body_end_ptr` check ensures it's valid to read `CHUNK_SIZE` bytes starting at `ptr`.
        #[expect(clippy::cast_ptr_alignment)]
        let chunk = unsafe { ptr.cast::<AlignedChunk>().as_ref().unwrap_unchecked() };
        if chunk.contains_unicode() {
            // SAFETY: `ptr` is equal to or after `start_ptr`. Both are within bounds of `bytes`.
            // `ptr` is derived from `start_ptr`.
            let offset = unsafe { ptr.offset_from_usize(start_ptr) };
            process_slice(chunk.as_slice(), offset);
        }

        // SAFETY: `ptr` and `body_end_ptr` are within bounds at start of this loop.
        // Distance between `ptr` and `body_end_ptr` is always a multiple of `CHUNK_SIZE`.
        // So `ptr + CHUNK_SIZE` is either equal to `body_end_ptr` or before it. So is within bounds.
        ptr = unsafe { ptr.add(CHUNK_SIZE) };
    }

    debug_assert!(ptr == body_end_ptr);

    // Process last chunk
    if remaining_len > 0 {
        debug_assert!(ptr as usize + remaining_len == bytes.as_ptr() as usize + bytes.len());

        // SAFETY: `ptr` is within `bytes` and `ptr + remaining_len` is end of `bytes`.
        // `bytes` is a `&[u8]`, so guaranteed initialized and valid for reads.
        let last_chunk = unsafe { slice::from_raw_parts(ptr, remaining_len) };
        // SAFETY: `ptr` is after `start_ptr`. Both are within bounds of `bytes`.
        // `ptr` is derived from `start_ptr`.
        let offset = unsafe { ptr.offset_from_usize(start_ptr) };
        process_slice(last_chunk, offset);
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_ast::{
        AstBuilder, Comment, CommentKind,
        ast::{Expression, Statement},
    };
    use oxc_span::{GetSpan, SourceType, Span};

    use super::Utf8ToUtf16;

    #[test]
    fn translate_ast() {
        let allocator = Allocator::new();
        let ast = AstBuilder::new(&allocator);

        let mut program = ast.program(
            Span::new(0, 15),
            SourceType::default(),
            ";'🤨' // 🤨",
            ast.vec1(Comment::new(8, 15, CommentKind::Line)),
            None,
            ast.vec(),
            ast.vec_from_array([
                ast.statement_empty(Span::new(0, 1)),
                ast.statement_expression(
                    Span::new(1, 7),
                    ast.expression_string_literal(Span::new(1, 7), "🤨", None),
                ),
            ]),
        );

        let span_converter = Utf8ToUtf16::new(program.source_text);
        span_converter.convert_program(&mut program);
        span_converter.convert_comments(&mut program.comments);

        assert_eq!(program.span, Span::new(0, 11));
        assert_eq!(program.body[1].span(), Span::new(1, 5));
        let Statement::ExpressionStatement(expr_stmt) = &program.body[1] else { unreachable!() };
        let Expression::StringLiteral(s) = &expr_stmt.expression else { unreachable!() };
        assert_eq!(s.span, Span::new(1, 5));
        assert_eq!(program.comments[0].span, Span::new(6, 11));
    }

    #[test]
    fn translate_offsets() {
        assert_eq!('_'.len_utf8(), 1);
        assert_eq!('_'.len_utf16(), 1);
        assert_eq!('£'.len_utf8(), 2);
        assert_eq!('£'.len_utf16(), 1);
        assert_eq!('ऊ'.len_utf8(), 3);
        assert_eq!('ऊ'.len_utf16(), 1);
        assert_eq!('🤨'.len_utf8(), 4);
        assert_eq!('🤨'.len_utf16(), 2);

        let cases: &[(&str, &[(u32, u32)])] = &[
            // 1-byte
            ("_", &[(0, 0), (1, 1)]),
            // 2-byte
            ("£", &[(0, 0), (2, 1)]),
            ("£_", &[(0, 0), (2, 1), (3, 2)]),
            ("_£", &[(0, 0), (1, 1), (3, 2)]),
            ("_£_", &[(0, 0), (1, 1), (3, 2), (4, 3)]),
            ("_££_", &[(0, 0), (1, 1), (3, 2), (5, 3), (6, 4)]),
            ("_£_£_", &[(0, 0), (1, 1), (3, 2), (4, 3), (6, 4), (7, 5)]),
            // 3-byte
            ("ऊ", &[(0, 0), (3, 1)]),
            ("ऊ_", &[(0, 0), (3, 1), (4, 2)]),
            ("_ऊ", &[(0, 0), (1, 1), (4, 2)]),
            ("_ऊ_", &[(0, 0), (1, 1), (4, 2), (5, 3)]),
            ("_ऊऊ_", &[(0, 0), (1, 1), (4, 2), (7, 3), (8, 4)]),
            ("_ऊ_ऊ_", &[(0, 0), (1, 1), (4, 2), (5, 3), (8, 4), (9, 5)]),
            // 4-byte
            ("🤨", &[(0, 0), (4, 2)]),
            ("🤨_", &[(0, 0), (4, 2), (5, 3)]),
            ("_🤨", &[(0, 0), (1, 1), (5, 3)]),
            ("_🤨_", &[(0, 0), (1, 1), (5, 3), (6, 4)]),
            ("_🤨🤨_", &[(0, 0), (1, 1), (5, 3), (9, 5), (10, 6)]),
            ("_🤨_🤨_", &[(0, 0), (1, 1), (5, 3), (6, 4), (10, 6), (11, 7)]),
        ];

        // Convert cases to `Vec`
        let mut cases_vec = cases
            .iter()
            .map(|&(text, translations)| (text, translations.to_vec()))
            .collect::<Vec<_>>();

        // Create 1 long string containing 99 repeats of each test case, concatenated
        let repeats = 99u32;
        let mut texts = String::new();
        for (text, _) in cases {
            for _i in 0..repeats {
                texts.push_str(text);
            }
        }

        // Generate more test cases for each of the defined cases repeated 99 times.
        // Each case references a slice of the large `texts` string.
        // Reason we do that is so that these string slices have uneven alignments, to exercise all parts
        // of `build_translations`, which handles unaligned header/tail differently from the main body
        // of the source text.
        // The number of repeats is 99, for the same reason - to ensure each string slice begins at
        // a memory address which is not evenly aligned.
        let mut offset = 0;
        for &(text, translations) in cases {
            let end_offset = offset + text.len() * (repeats as usize);
            let repeated_text = &texts[offset..end_offset];

            let (len_utf8, len_utf16) = *translations.last().unwrap();
            assert_eq!(text.len(), len_utf8 as usize);

            let mut repeated_translations = vec![];
            for i in 0..repeats {
                for &(offset_utf8, offset_utf16) in translations {
                    repeated_translations
                        .push((offset_utf8 + len_utf8 * i, offset_utf16 + len_utf16 * i));
                }
            }

            cases_vec.push((repeated_text, repeated_translations));

            offset = end_offset;
        }

        for (text, translations) in cases_vec {
            let table = Utf8ToUtf16::new(text);
            let converter = table.converter();
            if let Some(mut converter) = converter {
                // Iterate in forwards order
                for &(utf8_offset, expected_utf16_offset) in &translations {
                    let mut utf16_offset = utf8_offset;
                    converter.convert_offset(&mut utf16_offset);
                    assert_eq!(utf16_offset, expected_utf16_offset);
                }

                // Iterate in backwards order
                for &(utf8_offset, expected_utf16_offset) in translations.iter().rev() {
                    let mut utf16_offset = utf8_offset;
                    converter.convert_offset(&mut utf16_offset);
                    assert_eq!(utf16_offset, expected_utf16_offset);
                }
            } else {
                // No Unicode chars. All offsets should be the same.
                for &(utf8_offset, expected_utf16_offset) in &translations {
                    assert_eq!(utf8_offset, expected_utf16_offset);
                }
            }
        }
    }

    // Check assumptions about how many UTF-16 chars result from different UTF-8 character sequences,
    // which are relied on by `build_table`
    #[test]
    fn char_lengths() {
        macro_rules! assert_utf8_bytes_eq {
            ($c:expr, $bytes:expr) => {{
                let mut buffer = [0; 4];
                let bytes = $c.encode_utf8(&mut buffer).as_bytes();
                assert!($bytes == bytes);
            }};
        }

        // All 1-byte UTF-8 character sequences = 1 x UTF-16 character.
        // First byte is 0x00 - 0x7F.
        let min_1_byte_char = char::from_u32(0).unwrap();
        assert_eq!(min_1_byte_char.len_utf8(), 1);
        assert_eq!(min_1_byte_char.len_utf16(), 1);
        assert_utf8_bytes_eq!(min_1_byte_char, [0x00]);
        let max_1_byte_char = char::from_u32(0x7F).unwrap();
        assert_eq!(max_1_byte_char.len_utf8(), 1);
        assert_eq!(max_1_byte_char.len_utf16(), 1);
        assert_utf8_bytes_eq!(max_1_byte_char, [0x7F]);

        // All 2-byte UTF-8 character sequences = 1 x UTF-16 character
        // First byte is 0xC2 - 0xDF.
        let min_2_byte_char = char::from_u32(0x80).unwrap();
        assert_eq!(min_2_byte_char.len_utf8(), 2);
        assert_eq!(min_2_byte_char.len_utf16(), 1);
        assert_utf8_bytes_eq!(min_2_byte_char, [0xC2, 0x80]);
        let max_2_byte_char = char::from_u32(0x7FF).unwrap();
        assert_eq!(max_2_byte_char.len_utf8(), 2);
        assert_eq!(max_2_byte_char.len_utf16(), 1);
        assert_utf8_bytes_eq!(max_2_byte_char, [0xDF, 0xBF]);

        // All 3-byte UTF-8 character sequences = 1 x UTF-16 character
        // First byte is 0xE0 - 0xEF.
        let min_3_byte_char = char::from_u32(0x800).unwrap();
        assert_eq!(min_3_byte_char.len_utf8(), 3);
        assert_eq!(min_3_byte_char.len_utf16(), 1);
        assert_utf8_bytes_eq!(min_3_byte_char, [0xE0, 0xA0, 0x80]);
        let max_3_byte_char = char::from_u32(0xFFFF).unwrap();
        assert_eq!(max_3_byte_char.len_utf8(), 3);
        assert_eq!(max_3_byte_char.len_utf16(), 1);
        assert_utf8_bytes_eq!(max_3_byte_char, [0xEF, 0xBF, 0xBF]);

        // All 4-byte UTF-8 character sequences = 2 x UTF-16 characters
        // First byte is 0xF0 - 0xF4.
        let min_4_byte_char = char::from_u32(0x10000).unwrap();
        assert_eq!(min_4_byte_char.len_utf8(), 4);
        assert_eq!(min_4_byte_char.len_utf16(), 2);
        assert_utf8_bytes_eq!(min_4_byte_char, [0xF0, 0x90, 0x80, 0x80]);
        let max_4_byte_char = char::MAX;
        assert_eq!(max_4_byte_char.len_utf8(), 4);
        assert_eq!(max_4_byte_char.len_utf16(), 2);
        assert_utf8_bytes_eq!(max_4_byte_char, [0xF4, 0x8F, 0xBF, 0xBF]);
    }
}
