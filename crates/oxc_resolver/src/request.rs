#[derive(Debug, Eq, PartialEq)]
pub enum RequestError {
    Empty,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Request<'a> {
    Absolute(&'a str),
    Relative(&'a str),
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
        let parsed = Request::parse(request);
        let expected = RequestError::Empty;
        assert_eq!(parsed, Err(expected));
    }

    #[test]
    fn test_absolute() {
        let request = "/test";
        let parsed = Request::parse(request);
        let expected = Request::Absolute(request);
        assert_eq!(parsed, Ok(expected));
    }

    #[test]
    fn test_parse_relative() {
        let request = "./test";
        let parsed = Request::parse(request);
        let expected = Request::Relative(request);
        assert_eq!(parsed, Ok(expected));
    }
}
