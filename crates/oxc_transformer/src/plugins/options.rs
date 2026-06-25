use super::StyledComponentsOptions;

#[derive(Default, Debug, Clone)]
/// Plugin-specific transform options.
pub struct PluginsOptions {
    /// Options for `styled-components` transform.
    pub styled_components: Option<StyledComponentsOptions>,
    /// Enable tagged template transform plugin.
    pub tagged_template_transform: bool,
}
