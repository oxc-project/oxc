mod collapse;
mod fold_constants;
mod remove_dead_code;
mod remove_syntax;
mod substitute_alternate_syntax;

pub use collapse::Collapse;
pub use fold_constants::FoldConstants;
pub use remove_dead_code::RemoveDeadCode;
pub use remove_syntax::RemoveSyntax;
pub use substitute_alternate_syntax::SubstituteAlternateSyntax;
