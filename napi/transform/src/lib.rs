mod context;

pub use oxc::napi::{isolated_declarations, transform};

mod isolated_declaration;
pub use isolated_declaration::*;

mod transformer;
pub use transformer::*;
