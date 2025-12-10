use super::StyledComponentsOptions;

#[derive(Default, Debug, Clone)]
pub struct PluginsOptions {
    pub styled_components: Option<StyledComponentsOptions>,
    pub tagged_template_transform: bool,
}
