mod arrow_functions;
mod duplicate_keys;
mod function_name;
mod instanceof;
mod literals;
mod new_target;
mod shorthand_properties;
mod template_literals;

pub use arrow_functions::{ArrowFunctions, ArrowFunctionsOptions};
pub use duplicate_keys::DuplicateKeys;
pub use function_name::FunctionName;
pub use instanceof::Instanceof;
pub use literals::Literals;
pub use new_target::NewTarget;
pub use shorthand_properties::ShorthandProperties;
pub use template_literals::TemplateLiterals;
