use super::{EmotionOptions, StyledComponentsOptions};

/// Plugin-specific transform options.
#[derive(Default, Debug, Clone)]
pub struct PluginsOptions {
    /// Options for `styled-components` transform.
    pub styled_components: Option<StyledComponentsOptions>,
    /// Enable tagged template transform plugin.
    pub tagged_template_transform: bool,
    /// Options for `emotion` transform.
    pub emotion: Option<EmotionOptions>,
}
