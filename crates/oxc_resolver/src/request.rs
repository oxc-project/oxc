#[derive(Debug, Eq, PartialEq)]
pub enum RequestError {
    Empty,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Request<'a> {
    /// `/path`
    Absolute(&'a str),
    /// `./path`, `../path`
    Relative(&'a str),
    /// `path`
    Module(&'a str),
}

impl<'a> TryFrom<&'a str> for Request<'a> {
    type Error = RequestError;

    fn try_from(request: &'a str) -> Result<Self, Self::Error> {
        Request::parse(request)
    }
}

impl<'a> Request<'a> {
    fn parse(request: &'a str) -> Result<Self, RequestError> {
        let mut chars = request.chars();

        match chars.next() {
            Some('/') => Ok(Self::Absolute(request)),
            Some('.') => Ok(Self::Relative(request)),
            Some(_) => Ok(Self::Module(request)),
            _ => Err(RequestError::Empty),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Request, RequestError};

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn size_asserts() {
        static_assertions::assert_eq_size!(Request, [u64; 3]);
    }

    #[test]
    fn test_empty() {
        let request = "";
        assert_eq!(Request::parse(request), Err(RequestError::Empty));
    }

    #[test]
    fn test_absolute() {
        let request = "/test";
        assert_eq!(Request::parse(request), Ok(Request::Absolute(request)));
    }

    #[test]
    fn test_parse_relative() {
        let requests = ["./test", "../test", "../../test"];
        for request in requests {
            assert_eq!(Request::parse(request), Ok(Request::Relative(request)));
        }
    }

    #[test]
    fn test_module() {
        let requests = ["module"];
        for request in requests {
            assert_eq!(Request::parse(request), Ok(Request::Module(request)));
        }
    }
}
