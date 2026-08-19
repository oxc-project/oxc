//! Front matter detection and byte-preserving blanking shared by
//! document-envelope hosts (CSS today, Markdown later).
//!
//! Core owns ONLY the mechanics:
//! what the header language MEANS (yaml? toml? an Astro component script?)
//! and how the block composes into output are the host formatter's policy, never encoded here.

/// A front matter block at byte 0 of a document.
///
/// The slicing rules are a faithful port of Prettier's `front-matter/parse.js` (verified against bundled 3.9);
/// when in doubt, match that file, not intuition.
#[derive(Debug)]
pub struct FrontMatter<'a> {
    /// The verbatim slice from byte 0 through the closing delimiter's last character.
    /// Trailing text on the closing line stays OUTSIDE (in the body).
    pub raw: &'a str,
    /// Between the opening line's newline and the newline before the closing delimiter;
    /// empty for `---\n---`.
    pub value: &'a str,
    /// `"---"` or `"+++"`.
    pub start_delimiter: &'a str,
    /// Same as the opening delimiter, or `"..."` via the yaml-only fallback.
    pub end_delimiter: &'a str,
    /// The trimmed text after the opening delimiter on its line,
    /// if nonempty (`---yaml` → `Some("yaml")`).
    pub explicit_language: Option<&'a str>,
}

impl<'a> FrontMatter<'a> {
    /// The resolved language: [`Self::explicit_language`],
    /// or the delimiter default (`---` → `"yaml"`, `+++` → `"toml"`).
    /// What the name MEANS stays the host's decision.
    pub fn language(&self) -> &'a str {
        self.explicit_language.unwrap_or(if self.start_delimiter == "---" {
            "yaml"
        } else {
            "toml"
        })
    }
}

/// Detects a [`FrontMatter`] block at byte 0 of `text`.
///
/// Rules (Prettier `front-matter/parse.js` parity):
/// - only `---` / `+++` at byte 0 open a candidate, and the opening line must end in a newline
/// - the closing delimiter is `\n` + the SAME delimiter (`---` never closes `+++`)
/// - `\n...` closes only as a fallback: opening `---`, resolved language `yaml`,
///   and no `\n---` found ANYWHERE (a later `\n---` beats an earlier `\n...`)
/// - no constraint on what follows the closing delimiter on its line
///   (parse.js's `/\s?/` check is an always-true no-op; stricter would diverge)
pub fn parse_front_matter(text: &str) -> Option<FrontMatter<'_>> {
    let (start_delimiter, close_pattern) = if text.starts_with("---") {
        ("---", "\n---")
    } else if text.starts_with("+++") {
        ("+++", "\n+++")
    } else {
        return None;
    };

    // The opening line must end in a newline
    // (`trim` also drops a CRLF's `\r` so it never leaks into the language tag).
    let opening_newline = text.find('\n')?;
    let explicit = text[start_delimiter.len()..opening_newline].trim();
    let explicit_language = (!explicit.is_empty()).then_some(explicit);
    let resolved_yaml = explicit_language.is_none_or(|language| language == "yaml");

    // Search from the opening line's own newline,
    // so `---\n---` closes immediately with an empty value.
    let search = &text[opening_newline..];
    let (close_offset, end_delimiter) = if let Some(offset) = search.find(close_pattern) {
        (offset, start_delimiter)
    } else if start_delimiter == "---" && resolved_yaml {
        (search.find("\n...")?, "...")
    } else {
        return None;
    };

    // Absolute index of the newline right before the closing delimiter
    let closing_newline = opening_newline + close_offset;
    let raw = &text[..closing_newline + 1 + end_delimiter.len()];
    // Clamped: for `---\n---` the closing newline IS the opening newline
    let value = &text[(opening_newline + 1).min(closing_newline)..closing_newline];

    Some(FrontMatter { raw, value, start_delimiter, end_delimiter, explicit_language })
}

/// Blanks the leading `raw_len` bytes of `text` (a [`FrontMatter::raw`] length).
///
/// The host's parser never sees the block, while every span,
/// line offset, and gap after it stays byte-identical:
/// `\n` / `\r` survive, every other byte becomes one ASCII space
/// (multi-byte characters become runs of spaces, preserving byte length).
pub fn blank_front_matter(text: &str, raw_len: usize) -> String {
    let mut out = String::with_capacity(text.len());
    for &byte in &text.as_bytes()[..raw_len] {
        out.push(match byte {
            b'\n' => '\n',
            b'\r' => '\r',
            _ => ' ',
        });
    }
    out.push_str(&text[raw_len..]);
    out
}

#[cfg(test)]
mod tests {
    use super::{blank_front_matter, parse_front_matter};

    #[test]
    fn bare_yaml_block() {
        let fm = parse_front_matter("---\ntitle: x\n---\nbody").unwrap();
        assert_eq!(fm.raw, "---\ntitle: x\n---");
        assert_eq!(fm.value, "title: x");
        assert_eq!(fm.start_delimiter, "---");
        assert_eq!(fm.end_delimiter, "---");
        assert_eq!(fm.explicit_language, None);
        assert_eq!(fm.language(), "yaml");
    }

    #[test]
    fn toml_delimiter_defaults_to_toml() {
        let fm = parse_front_matter("+++\na = 1\n+++\n").unwrap();
        assert_eq!(fm.language(), "toml");
        assert_eq!(fm.raw, "+++\na = 1\n+++");
    }

    #[test]
    fn explicit_language_is_captured() {
        let fm = parse_front_matter("---mycustomparser\ntitle: x\n---\n").unwrap();
        assert_eq!(fm.explicit_language, Some("mycustomparser"));
        assert_eq!(fm.language(), "mycustomparser");
    }

    #[test]
    fn crlf_keeps_the_cr_out_of_the_language_and_in_the_value() {
        let fm = parse_front_matter("---\r\ntitle: x\r\n---\r\nbody").unwrap();
        assert_eq!(fm.explicit_language, None);
        assert_eq!(fm.language(), "yaml");
        assert_eq!(fm.value, "title: x\r");
        assert_eq!(fm.raw, "---\r\ntitle: x\r\n---");
    }

    #[test]
    fn empty_block_has_an_empty_value() {
        let fm = parse_front_matter("---\n---\nbody").unwrap();
        assert_eq!(fm.raw, "---\n---");
        assert_eq!(fm.value, "");
    }

    #[test]
    fn cross_delimiters_never_close() {
        assert!(parse_front_matter("---\na: 1\n+++\n").is_none());
        assert!(parse_front_matter("+++\na = 1\n---\n").is_none());
    }

    #[test]
    fn dots_close_only_as_the_yaml_fallback() {
        let fm = parse_front_matter("---\ntitle: x\n...\nbody").unwrap();
        assert_eq!(fm.end_delimiter, "...");
        assert_eq!(fm.raw, "---\ntitle: x\n...");

        // A later `\n---` beats an earlier `\n...` (indexOf priority)
        let fm = parse_front_matter("---\na: 1\n...\nb: 2\n---\n").unwrap();
        assert_eq!(fm.end_delimiter, "---");
        assert_eq!(fm.value, "a: 1\n...\nb: 2");

        // Non-yaml language: no dots fallback.
        assert!(parse_front_matter("---css\na {}\n...\n").is_none());
        assert!(parse_front_matter("+++\na = 1\n...\n").is_none());
    }

    #[test]
    fn unclosed_block_is_none() {
        assert!(parse_front_matter("---\ntitle: x\n").is_none());
        assert!(parse_front_matter("---").is_none());
        assert!(parse_front_matter("---\n").is_none());
    }

    #[test]
    fn closing_line_trailing_text_stays_in_the_body() {
        let fm = parse_front_matter("---\ntitle: x\n---trailing\nbody").unwrap();
        assert_eq!(fm.raw, "---\ntitle: x\n---");
        assert_eq!(fm.value, "title: x");
    }

    #[test]
    fn a_fence_past_byte_zero_is_not_front_matter() {
        assert!(parse_front_matter("\n---\ntitle: x\n---\n").is_none());
        assert!(parse_front_matter("a {}\n---\nx\n---\n").is_none());
    }

    #[test]
    fn blanking_preserves_byte_length_and_line_offsets() {
        let text = "---\ntitle: x\nemoji: ✨\n---\nbody { color: red }";
        let raw_len = parse_front_matter(text).unwrap().raw.len();
        let blanked = blank_front_matter(text, raw_len);

        assert_eq!(blanked.len(), text.len());
        let offsets = |s: &str| -> Vec<usize> {
            s.bytes().enumerate().filter(|(_, b)| *b == b'\n').map(|(i, _)| i).collect::<Vec<_>>()
        };
        assert_eq!(offsets(&blanked), offsets(text));
        // The body is untouched, the block is whitespace-only
        assert_eq!(&blanked[raw_len..], "\nbody { color: red }");
        assert!(blanked[..raw_len].chars().all(|c| c == ' ' || c == '\n'));
    }

    #[test]
    fn blanking_keeps_crlf_line_structure() {
        let text = "---\r\ntitle: x\r\n---\r\nbody";
        let raw_len = parse_front_matter(text).unwrap().raw.len();
        let blanked = blank_front_matter(text, raw_len);
        assert_eq!(blanked.len(), text.len());
        assert_eq!(&blanked[..raw_len], "   \r\n        \r\n   ");
    }
}
