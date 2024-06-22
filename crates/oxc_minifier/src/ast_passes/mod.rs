#![allow(clippy::wildcard_imports)]

mod remove_dead_code;
mod remove_parens;
mod replace_global_defines;

pub use remove_dead_code::RemoveDeadCode;
pub use remove_parens::RemoveParens;
pub use replace_global_defines::{ReplaceGlobalDefines, ReplaceGlobalDefinesConfig};
