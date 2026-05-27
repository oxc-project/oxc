use std::slice;

/// A translation from UTF-8 offset to UTF-16 offset.
#[derive(Clone, Copy)]
#[repr(align(8))]
pub struct Translation {
    /// UTF-8 byte offset.
    /// This is the UTF-8 offset of start of a Unicode character PLUS 1.
    /// So this offset sits in the middle of the Unicode character.
    /// Exception is the dummy first entry in table, where it's 0.
    pub utf8_offset: u32,
    /// Number to subtract from UTF-8 byte offset to get UTF-16 char offset
    /// for UTF-8 offsets after `utf8_offset`
    pub utf16_difference: u32,
}

/// A line break record used to convert UTF-8 / UTF-16 offsets to `(line, column)`.
///
/// Each entry marks the start of a new line.
#[derive(Clone, Copy)]
#[repr(align(8))]
pub struct LineTranslation {
    /// UTF-8 byte offset of the first byte of the line.
    pub utf8_offset: u32,
    /// Cumulative UTF-16 difference at the start of this line.
    ///
    /// Line start in UTF-16 = `utf8_offset - utf16_difference`.
    pub utf16_difference: u32,
}

const CHUNK_SIZE: usize = 32;
const CHUNK_ALIGNMENT: usize = align_of::<AlignedChunk>();
const _: () = {
    assert!(CHUNK_SIZE >= CHUNK_ALIGNMENT);
    assert!(CHUNK_SIZE.is_multiple_of(CHUNK_ALIGNMENT));
    assert!(CHUNK_SIZE == size_of::<AlignedChunk>());
};

/// Array of [`CHUNK_SIZE`] bytes, aligned on 16.
///
/// Alignment of 16 means it can be read with 2 x 16-byte aligned XMM reads (SIMD registers).
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

    /// Check if chunk contains any ASCII line break bytes (`\r` or `\n`).
    ///
    /// Compiler vectorizes this byte scan in the same way as [`Self::contains_unicode`].
    #[inline]
    fn contains_line_breaks(&self) -> bool {
        for index in 0..CHUNK_SIZE {
            if matches!(self.0[index], b'\r' | b'\n') {
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
/// `offset` is the starting offset. Usually 0, unless trimming BOM from start of file.
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
pub fn build_translations(source_text: &str, translations: &mut Vec<Translation>, offset: u32) {
    build_translations_impl::<false>(source_text, translations, None, offset);
}

/// Build tables of translations from UTF-8 offsets to UTF-16 offsets and line starts.
///
/// `lines` is populated with the UTF-8 offset of the start of each line *after* the first.
/// Recognized line terminators are `\n`, `\r`, `\r\n` (treated as a single break),
/// U+2028 (LS) and U+2029 (PS).
///
/// `lines` may be reused across calls but must be empty - the caller is expected to push
/// the line 0 sentinel (`utf8_offset: 0, utf16_difference: 0`) themselves so that this function
/// only needs to record breaks it observes.
pub fn build_translations_and_lines(
    source_text: &str,
    translations: &mut Vec<Translation>,
    lines: &mut Vec<LineTranslation>,
) {
    build_translations_impl::<true>(source_text, translations, Some(lines), 0);
}

/// Shared implementation for [`build_translations`] and [`build_translations_and_lines`].
///
/// The `TRACK_LINES` const generic switches line-break detection on at zero cost when off:
/// the closure's line-break branch is dead-code-eliminated when `TRACK_LINES == false`,
/// leaving the original Unicode-only scan.
fn build_translations_impl<const TRACK_LINES: bool>(
    source_text: &str,
    translations: &mut Vec<Translation>,
    mut lines: Option<&mut Vec<LineTranslation>>,
    offset: u32,
) {
    // Running counter of difference between UTF-8 and UTF-16 offset
    let mut utf16_difference = offset;

    let source_bytes = source_text.as_bytes();
    let source_len = source_bytes.len();

    // Closure that processes a slice of bytes for Unicode and (optionally) line breaks.
    //
    // `start_offset` is the offset of `slice` within `source_text`.
    let mut process_slice = |slice: &[u8], start_offset: usize| {
        let mut index = 0;
        while index < slice.len() {
            let byte = slice[index];

            if TRACK_LINES {
                // Handle ASCII line terminators \n, \r, \r\n.
                let line_break_len = match byte {
                    b'\n' => 1,
                    b'\r' => {
                        // Peek into the full source rather than `slice`, so \r\n straddling
                        // a chunk boundary is still detected as a single line break.
                        let next = start_offset + index + 1;
                        if next < source_len && source_bytes[next] == b'\n' { 2 } else { 1 }
                    }
                    _ => 0,
                };

                if line_break_len > 0 {
                    #[expect(clippy::cast_possible_truncation)]
                    let line_start = (start_offset + index + line_break_len) as u32;
                    // SAFETY: when `TRACK_LINES` is true, `lines` is always `Some` (set by callers).
                    let lines = unsafe { lines.as_deref_mut().unwrap_unchecked() };
                    lines.push(LineTranslation { utf8_offset: line_start, utf16_difference });
                    index += line_break_len;
                    continue;
                }

                // Handle U+2028 (LS) and U+2029 (PS). Both are 3-byte sequences starting `E2 80`.
                // Source is valid UTF-8, so a `0xE2` byte guarantees two more bytes follow.
                if byte == 0xE2 {
                    let full = start_offset + index;
                    debug_assert!(full + 3 <= source_len);
                    let b1 = source_bytes[full + 1];
                    let b2 = source_bytes[full + 2];
                    if b1 == 0x80 && (b2 == 0xA8 || b2 == 0xA9) {
                        // Note: don't `continue` - fall through so the Unicode block below
                        // also records the translation for this 3-byte character.
                        #[expect(clippy::cast_possible_truncation)]
                        let line_start = (full + 3) as u32;
                        // SAFETY: see above.
                        let lines = unsafe { lines.as_deref_mut().unwrap_unchecked() };
                        // `utf16_difference` is recorded *before* this character; the column
                        // formula handles the offset of the line start itself.
                        lines.push(LineTranslation { utf8_offset: line_start, utf16_difference });
                    }
                }
            }

            if byte >= 0xC0 {
                let difference_for_this_byte = u32::from(byte >= 0xE0) + 1;
                utf16_difference += difference_for_this_byte;

                // Record the index of the end of this Unicode character, because it's only offsets
                // *after* this Unicode character that need to be shifted.
                // Addition cannot overflow because length of source text is max `u32::MAX`.
                let bytes_in_char =
                    difference_for_this_byte as usize + usize::from(byte >= 0xF0) + 1;
                #[expect(clippy::cast_possible_truncation)]
                let utf8_offset = (start_offset + index + bytes_in_char) as u32;
                translations.push(Translation { utf8_offset, utf16_difference });
            }

            index += 1;
        }
    };

    // If source text is short, just process byte-by-byte
    let bytes = source_bytes;
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

    debug_assert!((ptr as usize).is_multiple_of(CHUNK_ALIGNMENT));

    // Process main body as aligned chunks of 32 bytes.
    //
    // We've aligned `ptr` to `CHUNK_ALIGNMENT`, so can now read the rest of source as `AlignedChunk`s
    // (apart from a few bytes on end which may not be enough to make a whole `AlignedChunk`).
    //
    // Do a fast check for any non-ASCII bytes (and, when tracking lines, line-break bytes) in each
    // chunk using SIMD-friendly byte scans.
    // Only if that finds an interesting byte, process the chunk byte-by-byte.

    // Get length of body of `bytes` which we can process as `AlignedChunk`s.
    // Round down remaining length to a multiple of `CHUNK_SIZE`.
    let body_len = remaining_len & !(CHUNK_SIZE - 1);
    remaining_len -= body_len;
    // SAFETY: `body_len` is less than number of bytes remaining in `bytes`, so in bounds
    let body_end_ptr = unsafe { ptr.add(body_len) };

    debug_assert!(body_end_ptr as usize <= start_ptr as usize + bytes.len());
    debug_assert!((body_end_ptr as usize - ptr as usize).is_multiple_of(CHUNK_SIZE));

    while ptr < body_end_ptr {
        // SAFETY: `ptr` was aligned to `CHUNK_ALIGNMENT` after processing 1st chunk.
        // It is incremented in this loop by `CHUNK_SIZE`, which is a multiple of `CHUNK_ALIGNMENT`,
        // so `ptr` remains always aligned for `CHUNK_ALIGNMENT`.
        // `ptr < body_end_ptr` check ensures it's valid to read `CHUNK_SIZE` bytes starting at `ptr`.
        #[expect(clippy::cast_ptr_alignment)]
        let chunk = unsafe { ptr.cast::<AlignedChunk>().as_ref().unwrap_unchecked() };
        let has_unicode = chunk.contains_unicode();
        let needs_scan = has_unicode || (TRACK_LINES && chunk.contains_line_breaks());
        if needs_scan {
            // SAFETY: `ptr` is equal to or after `start_ptr`. Both are within bounds of `bytes`.
            // `ptr` is derived from `start_ptr`.
            let offset = unsafe { ptr.offset_from_unsigned(start_ptr) };
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
        let offset = unsafe { ptr.offset_from_unsigned(start_ptr) };
        process_slice(last_chunk, offset);
    }
}
