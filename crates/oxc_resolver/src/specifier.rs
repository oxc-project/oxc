use crate::error::SpecifierError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Specifier<'a> {
    pub path: SpecifierPath<'a>,
    pub query: Option<&'a str>,
    pub fragment: Option<&'a str>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SpecifierPath<'a> {
    /// `/path`
    Absolute(&'a str),

    /// `./path`, `../path`
    Relative(&'a str),

    /// `#path`
    Hash(&'a str),

    /// Specifier without any leading syntax is called a bare specifier.
    Bare(&'a str),
}

impl<'a> SpecifierPath<'a> {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Absolute(s) | Self::Relative(s) | Self::Hash(s) | Self::Bare(s) => s,
        }
    }
}

impl<'a> Specifier<'a> {
    pub fn parse(specifier: &'a str) -> Result<Specifier<'a>, SpecifierError> {
        if specifier.is_empty() {
            return Err(SpecifierError::Empty);
        }

        let (path, query, fragment) = match specifier.as_bytes()[0] {
            b'/' => {
                let (path, query, fragment) = Self::parse_query_framgment(specifier, 1);
                (SpecifierPath::Absolute(path), query, fragment)
            }
            b'.' => {
                let (path, query, fragment) = Self::parse_query_framgment(specifier, 1);
                (SpecifierPath::Relative(path), query, fragment)
            }
            b'#' => {
                let (path, query, fragment) = Self::parse_query_framgment(specifier, 1);
                (SpecifierPath::Hash(path), query, fragment)
            }
            _ => {
                let (path, query, fragment) = Self::parse_query_framgment(specifier, 0);
                (SpecifierPath::Bare(path), query, fragment)
            }
        };

        Ok(Self { path, query, fragment })
    }

    fn parse_query_framgment(specifier: &str, skip: usize) -> (&str, Option<&str>, Option<&str>) {
        let mut query_start: Option<usize> = None;
        let mut fragment_start: Option<usize> = None;

        for (i, c) in specifier.as_bytes().iter().enumerate().skip(skip) {
            if *c == b'?' {
                query_start = Some(i);
            }
            if *c == b'#' {
                fragment_start = Some(i);
                break;
            }
        }

        match (query_start, fragment_start) {
            (Some(i), Some(j)) => {
                debug_assert!(i < j);
                (&specifier[..i], Some(&specifier[i..j]), Some(&specifier[j..]))
            }
            (Some(i), None) => (&specifier[..i], Some(&specifier[i..]), None),
            (None, Some(j)) => (&specifier[..j], None, Some(&specifier[j..])),
            _ => (specifier, None, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Specifier, SpecifierError, SpecifierPath};

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn size_asserts() {
        static_assertions::assert_eq_size!(Specifier, [u8; 56]);
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
        assert_eq!(parsed.path, SpecifierPath::Absolute("/test"));
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
            assert_eq!(parsed.path, SpecifierPath::Relative(specifier));
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
            assert_eq!(parsed.path, SpecifierPath::Hash(specifier));
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
            assert_eq!(parsed.path, SpecifierPath::Bare(specifier));
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
            assert_eq!(specifier.path.as_str(), "a", "{specifier_str}");
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
            assert_eq!(specifier.path.as_str(), path, "{specifier_str}");
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
            assert_eq!(specifier.path.as_str(), path, "{specifier_str}");
            assert_eq!(specifier.query.unwrap_or(""), query, "{specifier_str}");
            assert_eq!(specifier.fragment.unwrap_or(""), fragment, "{specifier_str}");
        }

        Ok(())
    }
}
