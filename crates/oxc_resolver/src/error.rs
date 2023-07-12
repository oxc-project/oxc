use crate::request::RequestError;

#[derive(Debug, Eq, PartialEq)]
pub enum ResolveError {
    NotFound,
    RequestError(RequestError),
}
