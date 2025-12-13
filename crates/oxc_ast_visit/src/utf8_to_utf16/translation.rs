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

/// A translation from UTF-8 offset to line number.
#[derive(Clone, Copy)]
#[repr(align(8))]
pub struct LineTranslation {
    /// UTF-8 byte offset of the start of the line.
    pub utf8_offset: u32,
    /// UTF-16 offset difference at the start of this line.
    /// Note: Currently unused but kept for potential future use and struct layout compatibility
    #[expect(dead_code)]
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

    /// Check if chunk contains any line breaks (\r or \n).
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
#[expect(dead_code)]
pub fn build_translations(source_text: &str, translations: &mut Vec<Translation>) {
    build_translations_and_lines(source_text, translations, None);
}

/// Build tables of translations from UTF-8 offsets to UTF-16 offsets and line numbers.
///
/// Line breaks handled are \r, \n, \r\n (considered as 1 line break), LS, and PS.
/// LS and PS are Unicode chars, so handled in the cold path for non-ASCII.
pub fn build_translations_and_lines(
    source_text: &str,
    translations: &mut Vec<Translation>,
    lines: Option<&mut Vec<LineTranslation>>,
) {
    // Running counter of difference between UTF-8 and UTF-16 offset
    let mut utf16_difference = 0;

    // Line tracking
    let track_lines = lines.is_some();
    let mut lines = lines;
    if track_lines {
        // Add first line starting at offset 0
        lines.as_mut().unwrap().push(LineTranslation { utf8_offset: 0, utf16_difference });
    }

    // Closure that processes a slice of bytes for both unicode and line breaks
    let mut process_slice = |slice: &[u8], start_offset: usize| {
        let mut index = 0;
        while index < slice.len() {
            let byte = slice[index];

            // Handle ASCII line breaks first
            if track_lines {
                let line_break_len = match byte {
                    b'\n' => 1,
                    b'\r' => {
                        // Check for \r\n - always use full source text to handle chunk boundaries
                        if start_offset + index + 1 < source_text.len()
                            && source_text.as_bytes()[start_offset + index + 1] == b'\n'
                        {
                            2
                        } else {
                            1
                        }
                    }
                    _ => 0,
                };

                if line_break_len > 0 {
                    let line_end_offset = start_offset + index + line_break_len;
                    // Always record line breaks, even if at end of file
                    #[expect(clippy::cast_possible_truncation)]
                    let utf8_offset = line_end_offset as u32;
                    lines.as_mut().unwrap().push(LineTranslation { utf8_offset, utf16_difference });
                    index += line_break_len;
                    continue;
                }
            }

            // Handle Unicode line separators LS (\u2028) and PS (\u2029) if tracking lines
            if track_lines && byte == 0xE2 {
                let full_offset = start_offset + index;

                // Always use full source text for PS/LS detection to simplify logic.
                // source_text is guaranteed valid UTF-8, so if byte is 0xE2, there must be 2 more bytes
                // after it (it's a 3-byte Unicode character). Assert this to catch any bugs.
                assert!(full_offset + 3 <= source_text.len());

                let source_bytes = source_text.as_bytes();
                let has_ls_ps = source_bytes[full_offset + 1] == 0x80
                    && (source_bytes[full_offset + 2] == 0xA8
                        || source_bytes[full_offset + 2] == 0xA9);

                if has_ls_ps {
                    let line_end_offset = full_offset + 3;
                    #[expect(clippy::cast_possible_truncation)]
                    let utf8_offset = line_end_offset as u32;
                    lines.as_mut().unwrap().push(LineTranslation { utf8_offset, utf16_difference });
                    // Don't skip - continue processing for unicode translation as PS/LS are Unicode chars
                }
            }

            // Handle Unicode characters
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

    debug_assert!((ptr as usize).is_multiple_of(CHUNK_ALIGNMENT));

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
    debug_assert!((body_end_ptr as usize - ptr as usize).is_multiple_of(CHUNK_SIZE));

    while ptr < body_end_ptr {
        // SAFETY: `ptr` was aligned to `CHUNK_ALIGNMENT` after processing 1st chunk.
        // It is incremented in this loop by `CHUNK_SIZE`, which is a multiple of `CHUNK_ALIGNMENT`,
        // so `ptr` remains always aligned for `CHUNK_ALIGNMENT`.
        // `ptr < body_end_ptr` check ensures it's valid to read `CHUNK_SIZE` bytes starting at `ptr`.
        #[expect(clippy::cast_ptr_alignment)]
        let chunk = unsafe { ptr.cast::<AlignedChunk>().as_ref().unwrap_unchecked() };

        // Process chunk for Unicode characters
        let has_unicode = chunk.contains_unicode();
        let has_line_breaks = track_lines && chunk.contains_line_breaks();

        if has_unicode || has_line_breaks {
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
