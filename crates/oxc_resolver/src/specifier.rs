use crate::error::SpecifierError;
use std::borrow::Cow;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Specifier<'a> {
    path: Cow<'a, str>,
    pub kind: SpecifierKind,
    pub query: Option<&'a str>,
    pub fragment: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SpecifierKind {
    /// `/path`
    Absolute,

    /// `./path`, `../path`
    Relative,

    /// `#path`
    Hash,

    /// Specifier without any leading syntax is called a bare specifier.
    Bare,
}

impl<'a> Specifier<'a> {
    pub fn path(&'a self) -> &'a str {
        self.path.as_ref()
    }

    pub fn set_path(&mut self, path: &'a str) {
        self.path = Cow::Borrowed(path);
    }

    pub fn parse(specifier: &'a str) -> Result<Specifier<'a>, SpecifierError> {
        if specifier.is_empty() {
            return Err(SpecifierError::Empty);
        }
        let (kind, offset) = match specifier.as_bytes()[0] {
            b'/' => (SpecifierKind::Absolute, 1),
            b'.' => (SpecifierKind::Relative, 1),
            b'#' => (SpecifierKind::Hash, 1),
            _ => (SpecifierKind::Bare, 0),
        };
        let (path, query, fragment) = Self::parse_query_framgment(specifier, offset);
        Ok(Self { path, kind, query, fragment })
    }

    fn parse_query_framgment(
        specifier: &'a str,
        skip: usize,
    ) -> (Cow<'a, str>, Option<&str>, Option<&str>) {
        let mut query_start: Option<usize> = None;
        let mut fragment_start: Option<usize> = None;

        let mut prev = specifier.chars().next().unwrap();
        let mut escaped_indexes = vec![];
        for (i, c) in specifier.chars().enumerate().skip(skip) {
            if c == '?' {
                query_start = Some(i);
            }
            if c == '#' {
                if prev == '\0' {
                    escaped_indexes.push(i - 1);
                } else {
                    fragment_start = Some(i);
                    break;
                }
            }
            prev = c;
        }

        let (path, query, fragment) = match (query_start, fragment_start) {
            (Some(i), Some(j)) => {
                debug_assert!(i < j);
                (&specifier[..i], Some(&specifier[i..j]), Some(&specifier[j..]))
            }
            (Some(i), None) => (&specifier[..i], Some(&specifier[i..]), None),
            (None, Some(j)) => (&specifier[..j], None, Some(&specifier[j..])),
            _ => (specifier, None, None),
        };

        let path = if escaped_indexes.is_empty() {
            Cow::Borrowed(path)
        } else {
            // Remove the `\0` characters for a legal path.
            Cow::Owned(
                path.chars()
                    .enumerate()
                    .filter_map(|(i, c)| (!escaped_indexes.contains(&i)).then_some(c))
                    .collect::<String>(),
            )
        };

        (path, query, fragment)
    }
}

#[cfg(test)]
mod tests {
    use super::{Specifier, SpecifierError, SpecifierKind};

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn size_asserts() {
        static_assertions::assert_eq_size!(Specifier, [u8; 64]);
    }

    #[test]
    fn empty() {
        let specifier = "";
        assert_eq!(Specifier::parse(specifier), Err(SpecifierError::Empty));
    }

    #[test]
    fn absolute() -> Result<(), SpecifierError> {
        let specifier = "/test?#";
        let parsed = Specifier::parse(specifier)?;
        assert_eq!(parsed.path, "/test");
        assert_eq!(parsed.kind, SpecifierKind::Absolute);
        assert_eq!(parsed.query, Some("?"));
        assert_eq!(parsed.fragment, Some("#"));
        Ok(())
    }

    #[test]
    fn relative() -> Result<(), SpecifierError> {
        let specifiers = ["./test", "../test", "../../test"];
        for specifier in specifiers {
            let mut r = specifier.to_string();
            r.push_str("?#");
            let parsed = Specifier::parse(&r)?;
            assert_eq!(parsed.path, specifier);
            assert_eq!(parsed.kind, SpecifierKind::Relative);
            assert_eq!(parsed.query, Some("?"));
            assert_eq!(parsed.fragment, Some("#"));
        }
        Ok(())
    }

    #[test]
    fn hash() -> Result<(), SpecifierError> {
        let specifiers = ["#", "#path"];
        for specifier in specifiers {
            let mut r = specifier.to_string();
            r.push_str("?#");
            let parsed = Specifier::parse(&r)?;
            assert_eq!(parsed.path, specifier);
            assert_eq!(parsed.kind, SpecifierKind::Hash);
            assert_eq!(parsed.query, Some("?"));
            assert_eq!(parsed.fragment, Some("#"));
        }
        Ok(())
    }

    #[test]
    fn module() -> Result<(), SpecifierError> {
        let specifiers = ["module"];
        for specifier in specifiers {
            let mut r = specifier.to_string();
            r.push_str("?#");
            let parsed = Specifier::parse(&r)?;
            assert_eq!(parsed.path, specifier);
            assert_eq!(parsed.kind, SpecifierKind::Bare);
            assert_eq!(parsed.query, Some("?"));
            assert_eq!(parsed.fragment, Some("#"));
        }
        Ok(())
    }

    #[test]
    fn query_fragment() -> Result<(), SpecifierError> {
        let data = [
            ("a?", Some("?"), None),
            ("a?query", Some("?query"), None),
            ("a#", None, Some("#")),
            ("a#fragment", None, Some("#fragment")),
            ("a?#", Some("?"), Some("#")),
            ("a?#fragment", Some("?"), Some("#fragment")),
            ("a?query#", Some("?query"), Some("#")),
            ("a?query#fragment", Some("?query"), Some("#fragment")),
            ("a#fragment?", None, Some("#fragment?")),
            ("a#fragment?query", None, Some("#fragment?query")),
        ];

        for (specifier_str, query, fragment) in data {
            let specifier = Specifier::parse(specifier_str)?;
            assert_eq!(specifier.path, "a", "{specifier_str}");
            assert_eq!(specifier.query, query, "{specifier_str}");
            assert_eq!(specifier.fragment, fragment, "{specifier_str}");
        }

        Ok(())
    }

    #[test]
    // https://github.com/webpack/enhanced-resolve/blob/main/test/identifier.test.js
    fn enhanced_resolve_edge_cases() -> Result<(), SpecifierError> {
        let data = [
            ("path/#", "path/", "", "#"),
            ("path/as/?", "path/as/", "?", ""),
            ("path/#/?", "path/", "", "#/?"),
            ("path/#repo#hash", "path/", "", "#repo#hash"),
            ("path/#r#hash", "path/", "", "#r#hash"),
            ("path/#repo/#repo2#hash", "path/", "", "#repo/#repo2#hash"),
            ("path/#r/#r#hash", "path/", "", "#r/#r#hash"),
            ("path/#/not/a/hash?not-a-query", "path/", "", "#/not/a/hash?not-a-query"),
        ];

        for (specifier_str, path, query, fragment) in data {
            let specifier = Specifier::parse(specifier_str)?;
            assert_eq!(specifier.path, path, "{specifier_str}");
            assert_eq!(specifier.query.unwrap_or(""), query, "{specifier_str}");
            assert_eq!(specifier.fragment.unwrap_or(""), fragment, "{specifier_str}");
        }

        Ok(())
    }

    // https://github.com/webpack/enhanced-resolve/blob/main/test/identifier.test.js
    #[test]
    fn enhanced_resolve_windows_like() -> Result<(), SpecifierError> {
        let data = [
            ("path\\#", "path\\", "", "#"),
            ("path\\as\\?", "path\\as\\", "?", ""),
            ("path\\#\\?", "path\\", "", "#\\?"),
            ("path\\#repo#hash", "path\\", "", "#repo#hash"),
            ("path\\#r#hash", "path\\", "", "#r#hash"),
            ("path\\#repo\\#repo2#hash", "path\\", "", "#repo\\#repo2#hash"),
            ("path\\#r\\#r#hash", "path\\", "", "#r\\#r#hash"),
            ("path\\#/not/a/hash?not-a-query", "path\\", "", "#/not/a/hash?not-a-query"),
        ];

        for (specifier_str, path, query, fragment) in data {
            let specifier = Specifier::parse(specifier_str)?;
            assert_eq!(specifier.path, path, "{specifier_str}");
            assert_eq!(specifier.query.unwrap_or(""), query, "{specifier_str}");
            assert_eq!(specifier.fragment.unwrap_or(""), fragment, "{specifier_str}");
        }

        Ok(())
    }
}
