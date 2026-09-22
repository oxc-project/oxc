use std::sync::OnceLock;

use lazy_regex::{Regex, regex::Error};
use oxc_str::JSStr;
use regex_automata::meta;
use regex_syntax::hir::{Capture, Class, ClassBytes, ClassBytesRange, Hir, Look};
use serde::{Deserialize, Deserializer, de::Error as _};

/// A Rust configuration regex matched against the original JavaScript string.
///
/// Keep the string regex's syntax validation and UTF-8 fast path. For values containing lone
/// surrogates, the byte regex searches the original WTF-8 bytes. Unicode classes (including `.`)
/// still match only Unicode scalar values, not surrogates. This is Rust regex semantics, not
/// JavaScript `RegExp` semantics; no replacement characters or escape spellings are introduced.
#[derive(Debug, Clone)]
pub struct ConfigRegex {
    utf8: Regex,
    bytes: OnceLock<meta::Regex>,
}

impl ConfigRegex {
    pub fn new(pattern: &str) -> Result<Self, Error> {
        Ok(Self { utf8: Regex::new(pattern)?, bytes: OnceLock::new() })
    }

    pub fn as_str(&self) -> &str {
        self.utf8.as_str()
    }

    pub fn is_match(&self, value: JSStr<'_>) -> bool {
        if let Some(text) = value.as_str() {
            self.utf8.is_match(text)
        } else {
            self.byte_regex().is_match(value.as_bytes())
        }
    }

    pub fn find<'a>(&self, value: JSStr<'a>) -> Option<&'a str> {
        if let Some(text) = value.as_str() {
            return self.utf8.find(text).map(|matched| matched.as_str());
        }

        let regex = self.byte_regex();
        let mut captures = regex.create_captures();
        regex.captures(value.as_bytes(), &mut captures);
        let matched = captures.get_group(self.utf8.captures_len())?;
        // The original string regex can only consume valid UTF-8. The extra boundary byte is
        // outside this capture, so it is never exposed as part of the matched string.
        Some(
            str::from_utf8(&value.as_bytes()[matched.range()]).expect("string regex matches UTF-8"),
        )
    }

    fn byte_regex(&self) -> &meta::Regex {
        self.bytes.get_or_init(|| {
            let hir = regex_syntax::Parser::new()
                .parse(self.as_str())
                .expect("pattern was validated by the string regex");
            let matched = Hir::capture(Capture {
                index: u32::try_from(self.utf8.captures_len()).unwrap(),
                name: None,
                sub: Box::new(hir),
            });
            // Consume a non-continuation byte, or reach the end of the input, after the original
            // match. This excludes empty matches inside a WTF-8 code point during the search,
            // without repeatedly scanning suffixes via find_iter. Non-empty matches already end
            // at a code point boundary because the original regex only consumes valid UTF-8.
            let boundary = Hir::alternation(vec![
                Hir::look(Look::End),
                Hir::class(Class::Bytes(ClassBytes::new([
                    ClassBytesRange::new(0, 0x7F),
                    ClassBytesRange::new(0xC0, 0xFF),
                ]))),
            ]);
            meta::Regex::builder()
                // The original regex already passed the size limit. Allow the fixed overhead
                // of the capture and boundary even when that pattern was just below the limit.
                .configure(meta::Regex::config().utf8_empty(false).nfa_size_limit(None))
                .build_from_hir(&Hir::concat(vec![matched, boundary]))
                .expect("a validated string regex with a byte boundary is a valid byte regex")
        })
    }
}

impl<'de> Deserialize<'de> for ConfigRegex {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let pattern = String::deserialize(deserializer)?;
        Self::new(&pattern).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod test {
    use lazy_regex::BytesRegex;
    use oxc_allocator::Allocator;
    use oxc_str::JSStrBuilder;

    use super::*;

    #[test]
    fn rust_regex_syntax_and_utf8_behavior() {
        for pattern in [r"(?-u:.)", r"(?-u:\xFF)", r"\uD800", r"(?=a)", r"(a)\1"] {
            assert!(ConfigRegex::new(pattern).is_err(), "{pattern}");
        }
        for pattern in ["", r"^.$", r"\b\w+\b", r"(?i)é", r"[a-z&&[^aeiou]]+", r"(?-u:\w+)"] {
            let regex = ConfigRegex::new(pattern).unwrap();
            let original = Regex::new(pattern).unwrap();
            for text in ["", "a", "abc", "é", "é日", "\u{FFFD}", "😀", " a\nb "] {
                assert_eq!(
                    regex.is_match(text.into()),
                    original.is_match(text),
                    "{pattern}: {text}"
                );
                assert_eq!(regex.find(text.into()), original.find(text).map(|m| m.as_str()));
            }
        }
    }

    #[test]
    fn lone_surrogates_are_not_replacement_characters() {
        let allocator = Allocator::default();
        for surrogate in [0xD800, 0xDBFF, 0xDC00, 0xDFFF] {
            let mut builder = JSStrBuilder::new_in(&allocator);
            builder.push_code_unit(surrogate);
            let value = builder.into_js_str();
            for pattern in ["�", r"[\uE000-\uFFFF]", r"^.$", r"[^a]", r"\\[uU][0-9a-fA-F]{4}"] {
                assert!(!ConfigRegex::new(pattern).unwrap().is_match(value), "{pattern}");
            }
            for pattern in ["", "^", "$", r"(?-u:\B)"] {
                assert_eq!(ConfigRegex::new(pattern).unwrap().find(value), Some(""), "{pattern}");
            }
        }
        assert!(ConfigRegex::new("^�$").unwrap().is_match("�".into()));
        assert!(ConfigRegex::new("^.$").unwrap().is_match("😀".into()));
    }

    #[test]
    fn searches_keep_surrounding_text_and_original_anchors() {
        let allocator = Allocator::default();
        let mut builder = JSStrBuilder::new_in(&allocator);
        builder.push_str("abc ");
        builder.push_code_unit(0xD800);
        builder.push_str(" É日 � xyz");
        let value = builder.into_js_str();
        for (pattern, expected) in [
            (r"^abc", Some("abc")),
            (r"xyz$", Some("xyz")),
            (r"(?i)é日", Some("É日")),
            (r"\bxyz\b", Some("xyz")),
            ("�", Some("�")),
            ("^xyz", None),
            ("abc$", None),
            ("^.*$", None),
            ("abc  É日", None),
        ] {
            assert_eq!(ConfigRegex::new(pattern).unwrap().find(value), expected, "{pattern}");
        }
    }

    #[test]
    #[expect(clippy::trivial_regex)] // Compare the byte engine's empty matches with character boundaries.
    fn empty_matches_do_not_split_code_points() {
        let allocator = Allocator::default();
        let mut builder = JSStrBuilder::new_in(&allocator);
        builder.push_str("aéb");
        builder.push_code_unit(0xD800);
        builder.push_str("c");
        let value = builder.into_js_str();

        // The ASCII non-word boundary matches only inside é and the surrogate's byte sequences.
        // There is no match between code points: every boundary has an ASCII word character
        // on exactly one side.
        assert!(BytesRegex::new(r"(?-u:\B)").unwrap().is_match(value.as_bytes()));
        assert!(!ConfigRegex::new(r"(?-u:\B)").unwrap().is_match(value));
    }

    #[test]
    fn boundary_guard_preserves_match_priority_and_captures() {
        let allocator = Allocator::default();
        for pattern in [
            "",
            "^",
            "$",
            "a|ab",
            "ab|a",
            "a*",
            "a*?",
            "a?",
            "a??",
            "a+",
            "a+?",
            r"\b",
            r"\B",
            r"(?-u:\b)",
            r"(?-u:\B)",
            r"(?-u:\b{start-half})",
            r"(?-u:\b{end-half})",
            r".*Z|(?-u:\B)",
            r"(a)(?<tail>b?)",
            r"(?i)é",
            r"(?m)^a|b$",
            "(?x)a # trailing comment",
            r"(?P<word>\w+)",
        ] {
            let regex = ConfigRegex::new(pattern).unwrap();
            let reference = BytesRegex::new(pattern).unwrap();
            for (prefix, suffix) in [
                ("", ""),
                ("a", "b"),
                ("aé", "b"),
                ("é", "日"),
                ("ab", "ab"),
                ("a\n", "\nb"),
                ("😀", "�"),
                ("aa", "Z"),
            ] {
                for surrogate in [0xD800, 0xDC00] {
                    let mut builder = JSStrBuilder::new_in(&allocator);
                    builder.push_str(prefix);
                    builder.push_code_unit(surrogate);
                    builder.push_str(suffix);
                    let value = builder.into_js_str();
                    let bytes = value.as_bytes();
                    let expected = reference.find_iter(bytes).find_map(|matched| {
                        if matched.is_empty()
                            && bytes.get(matched.start()).is_some_and(|b| b & 0xC0 == 0x80)
                        {
                            return None;
                        }
                        Some(str::from_utf8(matched.as_bytes()).unwrap())
                    });
                    assert_eq!(regex.find(value), expected, "{pattern}: {value:?}");
                    assert_eq!(regex.is_match(value), expected.is_some(), "{pattern}: {value:?}");
                }
            }
        }
    }

    #[test]
    fn repeated_internal_empty_matches_use_one_search() {
        let allocator = Allocator::default();
        let mut builder = JSStrBuilder::new_in(&allocator);
        builder.push_str(&"aé".repeat(8_000));
        builder.push_str("a");
        builder.push_code_unit(0xD800);
        builder.push_str("b");
        let value = builder.into_js_str();
        let regex = ConfigRegex::new(r".*Z|(?-u:\B)").unwrap();
        assert!(!regex.is_match(value));
        assert_eq!(regex.find(value), None);
    }
}
