//! AST Definitions

mod js;
mod jsx;
mod literal;
mod macros;
mod ts;

pub use self::{js::*, jsx::*, literal::*, ts::*};
use macros::inherit_variants;
