//! Diagnostics Wrapper
//! Exports `thiserror` and `miette`

use std::{cell::RefCell, ops::Deref, rc::Rc};

pub use miette;
pub use thiserror;

pub type PError = miette::Error;

pub type Result<T> = std::result::Result<T, PError>;

#[derive(Debug, Default, Clone)]
pub struct Diagnostics(Rc<RefCell<Vec<PError>>>);

impl Deref for Diagnostics {
    type Target = Rc<RefCell<Vec<PError>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Diagnostics {
    /// # Panics
    #[must_use]
    pub fn into_inner(self) -> Vec<PError> {
        Rc::try_unwrap(self.0).unwrap().into_inner()
    }
}
