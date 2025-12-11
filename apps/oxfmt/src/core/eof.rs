//! EOF newline adjustment logic
//!
//! This module provides zero-allocation helpers for adjusting end-of-file newlines
//! based on the `insert_final_newline` option, applied at the point of write or check.

use std::{io::Write, path::Path};

use oxc_formatter::{InsertFinalNewline, LineEnding};

/// Calculate and apply EOF newline adjustment, returning content slice and optional newline.
///
/// This function uses string slicing to avoid allocations. It returns:
/// - A slice of the formatted code (possibly trimmed)
/// - An optional line ending to append
///
/// The caller can then write these two parts separately with zero allocations.
pub fn apply_eof_adjustment<'a>(
    formatted_code: &'a str,
    original_source: &str,
    insert_final_newline: InsertFinalNewline,
    line_ending: LineEnding,
) -> (&'a str, Option<&'static [u8]>) {
    let original_has_newline = has_trailing_newline(original_source);
    let line_ending_bytes = line_ending.as_bytes();

    let needs_newline = match insert_final_newline {
        InsertFinalNewline::Auto => original_has_newline,
        InsertFinalNewline::Always => true,
        InsertFinalNewline::Never => false,
    };

    let trimmed = formatted_code.trim_end_matches(['\r', '\n']);

    if needs_newline {
        // Check if already has exactly the right single line ending
        if has_correct_single_trailing_newline(formatted_code, line_ending) {
            // Perfect - return as-is (zero allocation)
            (formatted_code, None)
        } else {
            // Needs adjustment - return trimmed + newline
            (trimmed, Some(line_ending_bytes))
        }
    } else {
        // Should have no newline
        if trimmed.len() == formatted_code.len() {
            // Already has no trailing newline (zero allocation)
            (formatted_code, None)
        } else {
            // Has trailing newline(s), return trimmed
            (trimmed, None)
        }
    }
}

/// Check if two contents are equal considering EOF adjustment.
///
/// Used for change detection in check mode. This performs the comparison
/// without allocating a new string.
pub fn equals_with_eof_adjustment(
    original: &str,
    formatted: &str,
    insert_final_newline: InsertFinalNewline,
    line_ending: LineEnding,
) -> bool {
    let (content, newline) =
        apply_eof_adjustment(formatted, original, insert_final_newline, line_ending);

    match newline {
        Some(newline_bytes) => {
            // Would write: content + newline
            // SAFETY: line_ending_bytes comes from LineEnding::as_bytes() which is valid UTF-8
            let newline_str = unsafe { std::str::from_utf8_unchecked(newline_bytes) };
            original.len() == content.len() + newline_str.len()
                && original.starts_with(content)
                && original.ends_with(newline_str)
        }
        None => {
            // Would write: content (as-is or trimmed)
            original == content
        }
    }
}

/// Write formatted code to file with EOF adjustment.
///
/// This function applies the EOF adjustment during the write operation,
/// using two separate writes when a newline needs to be added. This avoids
/// allocating a new string for the adjusted content.
pub fn write_with_eof_adjustment(
    path: &Path,
    formatted_code: &str,
    original_source: &str,
    insert_final_newline: InsertFinalNewline,
    line_ending: LineEnding,
) -> std::io::Result<()> {
    let (content, newline) =
        apply_eof_adjustment(formatted_code, original_source, insert_final_newline, line_ending);

    if let Some(newline) = newline {
        // Write content + newline in two operations
        let file = std::fs::File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);
        writer.write_all(content.as_bytes())?;
        writer.write_all(newline)?;
        writer.flush()?;
    } else {
        // Write content as-is (might be trimmed or untouched)
        std::fs::write(path, content)?;
    }

    Ok(())
}

/// Check if string ends with any newline character(s).
#[inline]
fn has_trailing_newline(s: &str) -> bool {
    s.ends_with('\n') || s.ends_with('\r')
}

/// Check if string ends with exactly one instance of the specified line ending.
///
/// Returns false if:
/// - The string doesn't end with the specified line ending
/// - The string has multiple trailing newlines
fn has_correct_single_trailing_newline(s: &str, line_ending: LineEnding) -> bool {
    let line_ending_bytes = line_ending.as_bytes();
    // SAFETY: line_ending_bytes comes from LineEnding::as_bytes() which is valid UTF-8
    let line_ending_str = unsafe { std::str::from_utf8_unchecked(line_ending_bytes) };

    if !s.ends_with(line_ending_str) {
        return false;
    }

    // Check if there's only one line ending (no multiple trailing newlines)
    let before_newline = &s[..s.len() - line_ending_bytes.len()];
    !has_trailing_newline(before_newline)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_eof_adjustment_auto_preserve_no_newline() {
        let formatted = "const x = 1;";
        let original = "const x=1;"; // No newline

        let (content, newline) =
            apply_eof_adjustment(formatted, original, InsertFinalNewline::Auto, LineEnding::Lf);

        assert_eq!(content, "const x = 1;");
        assert_eq!(newline, None);
    }

    #[test]
    fn test_apply_eof_adjustment_auto_preserve_with_newline() {
        let formatted = "const x = 1;\n";
        let original = "const x=1;\n"; // Has newline

        let (content, newline) =
            apply_eof_adjustment(formatted, original, InsertFinalNewline::Auto, LineEnding::Lf);

        // Already has correct newline, return as-is
        assert_eq!(content, "const x = 1;\n");
        assert_eq!(newline, None);
    }

    #[test]
    fn test_apply_eof_adjustment_always_add_newline() {
        let formatted = "const x = 1;"; // No newline
        let original = "const x=1;"; // No newline

        let (content, newline) =
            apply_eof_adjustment(formatted, original, InsertFinalNewline::Always, LineEnding::Lf);

        assert_eq!(content, "const x = 1;");
        assert_eq!(newline, Some(b"\n".as_slice()));
    }

    #[test]
    fn test_apply_eof_adjustment_never_remove_newline() {
        let formatted = "const x = 1;\n"; // Has newline
        let original = "const x=1;\n"; // Has newline

        let (content, newline) =
            apply_eof_adjustment(formatted, original, InsertFinalNewline::Never, LineEnding::Lf);

        assert_eq!(content, "const x = 1;"); // Trimmed
        assert_eq!(newline, None);
    }

    #[test]
    fn test_apply_eof_adjustment_normalize_multiple_newlines() {
        let formatted = "const x = 1;\n\n\n"; // Multiple newlines
        let original = "const x=1;\n"; // Single newline

        let (content, newline) =
            apply_eof_adjustment(formatted, original, InsertFinalNewline::Auto, LineEnding::Lf);

        assert_eq!(content, "const x = 1;");
        assert_eq!(newline, Some(b"\n".as_slice())); // Normalized to one
    }

    #[test]
    fn test_apply_eof_adjustment_respects_line_ending() {
        let formatted = "const x = 1;";
        let original = "const x=1;";

        // Test CRLF
        let (content, newline) =
            apply_eof_adjustment(formatted, original, InsertFinalNewline::Always, LineEnding::Crlf);
        assert_eq!(content, "const x = 1;");
        assert_eq!(newline, Some(b"\r\n".as_slice()));

        // Test CR
        let (content, newline) =
            apply_eof_adjustment(formatted, original, InsertFinalNewline::Always, LineEnding::Cr);
        assert_eq!(content, "const x = 1;");
        assert_eq!(newline, Some(b"\r".as_slice()));
    }

    #[test]
    fn test_equals_with_eof_adjustment_auto_mode() {
        let original = "const x = 1;\n";
        let formatted = "const x = 1;\n\n"; // Extra newline

        // Should be equal after adjustment (normalize to single newline)
        assert!(equals_with_eof_adjustment(
            original,
            formatted,
            InsertFinalNewline::Auto,
            LineEnding::Lf,
        ));
    }

    #[test]
    fn test_equals_with_eof_adjustment_different_content() {
        let original = "const x = 1;\n";
        let formatted = "const y = 2;\n";

        // Should not be equal - different content
        assert!(!equals_with_eof_adjustment(
            original,
            formatted,
            InsertFinalNewline::Auto,
            LineEnding::Lf,
        ));
    }

    #[test]
    fn test_has_correct_single_trailing_newline() {
        // Single LF - correct
        assert!(has_correct_single_trailing_newline("test\n", LineEnding::Lf));

        // Multiple LF - incorrect
        assert!(!has_correct_single_trailing_newline("test\n\n", LineEnding::Lf));

        // No newline - incorrect
        assert!(!has_correct_single_trailing_newline("test", LineEnding::Lf));

        // Single CRLF - correct
        assert!(has_correct_single_trailing_newline("test\r\n", LineEnding::Crlf));

        // LF when expecting CRLF - incorrect
        assert!(!has_correct_single_trailing_newline("test\n", LineEnding::Crlf));

        // CRLF when expecting LF - incorrect (ends with LF but has extra CR)
        assert!(!has_correct_single_trailing_newline("test\r\n", LineEnding::Lf));
    }

    #[test]
    fn test_has_trailing_newline() {
        assert!(has_trailing_newline("test\n"));
        assert!(has_trailing_newline("test\r"));
        assert!(has_trailing_newline("test\r\n"));
        assert!(!has_trailing_newline("test"));
        assert!(!has_trailing_newline(""));
    }

    #[test]
    fn test_empty_file_auto_mode() {
        let formatted = "";
        let original = "";

        let (content, newline) =
            apply_eof_adjustment(formatted, original, InsertFinalNewline::Auto, LineEnding::Lf);

        assert_eq!(content, "");
        assert_eq!(newline, None);
    }

    #[test]
    fn test_empty_file_always_mode() {
        let formatted = "";
        let original = "";

        let (content, newline) =
            apply_eof_adjustment(formatted, original, InsertFinalNewline::Always, LineEnding::Lf);

        assert_eq!(content, "");
        assert_eq!(newline, Some(b"\n".as_slice()));
    }

    #[test]
    fn test_empty_file_never_mode() {
        let formatted = "";
        let original = "";

        let (content, newline) =
            apply_eof_adjustment(formatted, original, InsertFinalNewline::Never, LineEnding::Lf);

        assert_eq!(content, "");
        assert_eq!(newline, None);
    }
}
