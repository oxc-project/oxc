//! Public option types of the `sort.*` features.
//!
//! `JsFormatOptions.sort` is the umbrella; each target is `Option<…>` and `None` means disabled.
//! Every printer hook is gated on its own target so that disabled targets cost nothing.

pub use super::sort_imports::options::*;

/// Umbrella for all sorting targets.
#[derive(Debug, Clone, Default)]
pub struct SortOptions {
    /// Sort import declarations (`sort.imports`). Disabled by default.
    pub imports: Option<SortImportsOptions>,
}

impl SortOptions {
    /// Whether any target is enabled.
    pub fn any_enabled(&self) -> bool {
        self.imports.is_some()
    }

    /// Validate every enabled target's option combination.
    ///
    /// # Errors
    /// Returns the first target's error message.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(imports) = &self.imports {
            imports.validate()?;
        }
        Ok(())
    }
}
