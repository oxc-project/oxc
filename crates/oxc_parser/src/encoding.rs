use oxc_allocator::Allocator;

/// Detect encoding from BOM and normalize bytes to UTF-8.
///
/// - `0xFF 0xFE` → UTF-16 LE: transcode to UTF-8, allocate result in arena
/// - `0xFE 0xFF` → UTF-16 BE: transcode to UTF-8, allocate result in arena
/// - `0xEF 0xBB 0xBF` → UTF-8 BOM: strip BOM (return `&bytes[3..]`)
/// - Otherwise → return bytes as-is
pub fn detect_encoding_and_normalize<'a>(allocator: &'a Allocator, bytes: &'a [u8]) -> &'a [u8] {
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        // UTF-8 BOM — strip it
        return &bytes[3..];
    }

    if bytes.len() >= 2 {
        let (decoder, bom_len) = if bytes[0] == 0xFF && bytes[1] == 0xFE {
            // UTF-16 LE BOM
            (encoding_rs::UTF_16LE, 2)
        } else if bytes[0] == 0xFE && bytes[1] == 0xFF {
            // UTF-16 BE BOM
            (encoding_rs::UTF_16BE, 2)
        } else {
            return bytes;
        };

        let source_after_bom = &bytes[bom_len..];
        let (decoded, _, _) = decoder.decode(source_after_bom);
        match decoded {
            std::borrow::Cow::Borrowed(s) => {
                // Already valid UTF-8 (shouldn't happen for UTF-16, but handle gracefully)
                s.as_bytes()
            }
            std::borrow::Cow::Owned(s) => {
                // Allocate transcoded bytes in arena
                allocator.alloc_slice_copy(s.as_bytes())
            }
        }
    } else {
        bytes
    }
}
