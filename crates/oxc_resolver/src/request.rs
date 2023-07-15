#[derive(Debug, Eq, PartialEq)]
pub enum RequestError {
    Empty,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Request<'a> {
    pub path: RequestPath<'a>,
    pub query: Option<&'a str>,
    pub fragment: Option<&'a str>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum RequestPath<'a> {
    /// `/path`
    Absolute(&'a str),
    /// `./path`, `../path`
    Relative(&'a str),
    /// `path`
    Module(&'a str),
}

impl<'a> Request<'a> {
    pub fn parse(request: &'a str) -> Result<Request<'a>, RequestError> {
        let (request, query, fragment) = Self::parse_query_framgment(request);
        RequestPath::parse(request).map(|path| Self { path, query, fragment })
    }

    fn parse_query_framgment(request: &str) -> (&str, Option<&str>, Option<&str>) {
        let mut query_start: Option<usize> = None;
        let mut fragment_start: Option<usize> = None;

        for (i, c) in request.as_bytes().iter().enumerate() {
            match *c {
                b'?' => query_start = Some(i),
                b'#' => fragment_start = Some(i),
                _ => {}
            }
        }

        match (query_start, fragment_start) {
            (Some(i), Some(j)) if i < j => {
                (&request[..i], Some(&request[i..j]), Some(&request[j..]))
            }
            (Some(i), Some(j)) if i > j => {
                (&request[..j], Some(&request[i..]), Some(&request[j..i]))
            }
            (Some(i), None) => (&request[..i], Some(&request[i..]), None),
            (None, Some(j)) => (&request[..j], None, Some(&request[j..])),
            _ => (request, None, None),
        }
    }
}

impl<'a> RequestPath<'a> {
    fn parse(request: &'a str) -> Result<Self, RequestError> {
        match request.chars().next() {
            Some('/') => Ok(Self::Absolute(request)),
            Some('.') => Ok(Self::Relative(request)),
            Some(_) => Ok(Self::Module(request)),
            _ => Err(RequestError::Empty),
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
        let request = "/test";
        let parsed = Request::parse(request)?;
        assert_eq!(parsed.path, RequestPath::Absolute(request));
        Ok(())
    }

    #[test]
    fn parse_relative() -> Result<(), RequestError> {
        let requests = ["./test", "../test", "../../test"];
        for request in requests {
            let parsed = Request::parse(request)?;
            assert_eq!(parsed.path, RequestPath::Relative(request));
        }
        Ok(())
    }

    #[test]
    fn module() -> Result<(), RequestError> {
        let requests = ["module"];
        for request in requests {
            let parsed = Request::parse(request)?;
            assert_eq!(parsed.path, RequestPath::Module(request));
        }
        Ok(())
    }

    #[test]
    fn query_fragment() -> Result<(), RequestError> {
        assert_eq!(Request::parse("a?")?.query, Some("?"));
        assert_eq!(Request::parse("a?b")?.query, Some("?b"));

        assert_eq!(Request::parse("a#")?.fragment, Some("#"));
        assert_eq!(Request::parse("a#b")?.fragment, Some("#b"));

        let request = Request::parse("a?#")?;
        assert_eq!(request.query, Some("?"));
        assert_eq!(request.fragment, Some("#"));

        let request = Request::parse("a?b#c")?;
        assert_eq!(request.query, Some("?b"));
        assert_eq!(request.fragment, Some("#c"));

        let request = Request::parse("a#b?c")?;
        assert_eq!(request.query, Some("?c"));
        assert_eq!(request.fragment, Some("#b"));
        Ok(())
    }
}
