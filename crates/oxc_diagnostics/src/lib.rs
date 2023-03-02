//! Diagnostics Wrapper
//! Exports `thiserror` and `miette`

use std::{cell::RefCell, ops::Deref, rc::Rc};

pub use miette;
pub use thiserror;

pub type Error = miette::Error;
pub type Severity = miette::Severity;
pub type Report = miette::Report;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Default, Clone)]
pub struct Diagnostics(Rc<RefCell<Vec<Error>>>);

impl Deref for Diagnostics {
    type Target = Rc<RefCell<Vec<Error>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Diagnostics {
    /// # Panics
    #[must_use]
    pub fn into_inner(self) -> Vec<Error> {
        Rc::try_unwrap(self.0).unwrap().into_inner()
    }
}
