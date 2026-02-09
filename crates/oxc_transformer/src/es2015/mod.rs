use oxc_traverse::Traverse;

use crate::state::TransformState;

mod arrow_functions;
mod options;

pub use arrow_functions::{ArrowFunctions, ArrowFunctionsOptions};
pub use options::ES2015Options;

pub struct ES2015<'a> {
    #[expect(unused)]
    options: ES2015Options,

    // Plugins
    #[expect(unused)]
    arrow_functions: ArrowFunctions,

    _marker: std::marker::PhantomData<&'a ()>,
}

impl ES2015<'_> {
    pub fn new(options: ES2015Options) -> Self {
        Self {
            arrow_functions: ArrowFunctions::new(options.arrow_function.unwrap_or_default()),
            options,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for ES2015<'a> {}
