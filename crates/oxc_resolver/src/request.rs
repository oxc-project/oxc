#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RequestError {
    Empty,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Request<'a> {
    pub path: RequestPath<'a>,
    pub query: Option<&'a str>,
    pub fragment: Option<&'a str>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RequestPath<'a> {
    /// `/path`
    Absolute(&'a str),
    /// `./path`, `../path`
    Relative(&'a str),
    /// `#path`
    Hash(&'a str),
    /// `path`, `@scope/path`
    Module(&'a str),
}

impl<'a> RequestPath<'a> {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Absolute(s) | Self::Relative(s) | Self::Hash(s) | Self::Module(s) => s,
        }
    }
}

impl<'a> Request<'a> {
    pub fn parse(request: &'a str) -> Result<Request<'a>, RequestError> {
        if request.is_empty() {
            return Err(RequestError::Empty);
        }

        let (path, query, fragment) = match request.as_bytes()[0] {
            b'/' => {
                let (path, query, fragment) = Self::parse_query_framgment(request, 1);
                (RequestPath::Absolute(path), query, fragment)
            }
            b'.' => {
                let (path, query, fragment) = Self::parse_query_framgment(request, 1);
                (RequestPath::Relative(path), query, fragment)
            }
            b'#' => {
                let (path, query, fragment) = Self::parse_query_framgment(request, 1);
                (RequestPath::Hash(path), query, fragment)
            }
            _ => {
                let (path, query, fragment) = Self::parse_query_framgment(request, 0);
                (RequestPath::Module(path), query, fragment)
            }
        };

        Ok(Self { path, query, fragment })
    }

    fn parse_query_framgment(request: &str, skip: usize) -> (&str, Option<&str>, Option<&str>) {
        let mut query_start: Option<usize> = None;
        let mut fragment_start: Option<usize> = None;

        for (i, c) in request.as_bytes().iter().enumerate().skip(skip) {
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
                (&request[..i], Some(&request[i..j]), Some(&request[j..]))
            }
            (Some(i), None) => (&request[..i], Some(&request[i..]), None),
            (None, Some(j)) => (&request[..j], None, Some(&request[j..])),
            _ => (request, None, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Request, RequestError, RequestPath};

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn size_asserts() {
        static_assertions::assert_eq_size!(Request, [u8; 56]);
    }

    #[test]
    fn empty() {
        let request = "";
        assert_eq!(Request::parse(request), Err(RequestError::Empty));
    }

    #[test]
    fn absolute() -> Result<(), RequestError> {
        let request = "/test?#";
        let parsed = Request::parse(request)?;
        assert_eq!(parsed.path, RequestPath::Absolute("/test"));
        assert_eq!(parsed.query, Some("?"));
        assert_eq!(parsed.fragment, Some("#"));
        Ok(())
    }

    #[test]
    fn relative() -> Result<(), RequestError> {
        let requests = ["./test", "../test", "../../test"];
        for request in requests {
            let mut r = request.to_string();
            r.push_str("?#");
            let parsed = Request::parse(&r)?;
            assert_eq!(parsed.path, RequestPath::Relative(request));
            assert_eq!(parsed.query, Some("?"));
            assert_eq!(parsed.fragment, Some("#"));
        }
        Ok(())
    }

    #[test]
    fn hash() -> Result<(), RequestError> {
        let requests = ["#", "#path"];
        for request in requests {
            let mut r = request.to_string();
            r.push_str("?#");
            let parsed = Request::parse(&r)?;
            assert_eq!(parsed.path, RequestPath::Hash(request));
            assert_eq!(parsed.query, Some("?"));
            assert_eq!(parsed.fragment, Some("#"));
        }
        Ok(())
    }

    #[test]
    fn module() -> Result<(), RequestError> {
        let requests = ["module"];
        for request in requests {
            let mut r = request.to_string();
            r.push_str("?#");
            let parsed = Request::parse(&r)?;
            assert_eq!(parsed.path, RequestPath::Module(request));
            assert_eq!(parsed.query, Some("?"));
            assert_eq!(parsed.fragment, Some("#"));
        }
        Ok(())
    }

    #[test]
    fn query_fragment() -> Result<(), RequestError> {
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

        for (request_str, query, fragment) in data {
            let request = Request::parse(request_str)?;
            assert_eq!(request.path.as_str(), "a", "{request_str}");
            assert_eq!(request.query, query, "{request_str}");
            assert_eq!(request.fragment, fragment, "{request_str}");
        }

        Ok(())
    }

    #[test]
    // https://github.com/webpack/enhanced-resolve/blob/main/test/identifier.test.js
    fn enhanced_resolve_edge_cases() -> Result<(), RequestError> {
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

        for (request_str, path, query, fragment) in data {
            let request = Request::parse(request_str)?;
            assert_eq!(request.path.as_str(), path, "{request_str}");
            assert_eq!(request.query.unwrap_or(""), query, "{request_str}");
            assert_eq!(request.fragment.unwrap_or(""), fragment, "{request_str}");
        }

        Ok(())
    }

    // https://github.com/webpack/enhanced-resolve/blob/main/test/identifier.test.js
    #[test]
    fn enhanced_resolve_windows_like() -> Result<(), RequestError> {
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

        for (request_str, path, query, fragment) in data {
            let request = Request::parse(request_str)?;
            assert_eq!(request.path.as_str(), path, "{request_str}");
            assert_eq!(request.query.unwrap_or(""), query, "{request_str}");
            assert_eq!(request.fragment.unwrap_or(""), fragment, "{request_str}");
        }

        Ok(())
    }
}
