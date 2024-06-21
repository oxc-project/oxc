#![allow(clippy::wildcard_imports)]

mod remove_dead_code;
mod remove_parens;

pub use remove_dead_code::RemoveDeadCode;
pub use remove_parens::RemoveParens;
