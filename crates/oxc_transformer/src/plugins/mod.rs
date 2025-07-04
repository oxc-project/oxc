mod options;
mod styled_components;

pub use options::PluginsOptions;
pub use styled_components::StyledComponentsOptions;

use crate::{context::TransformCtx, plugins::styled_components::StyledComponents};

pub struct Plugins<'a, 'ctx> {
    styled_components: StyledComponents<'a, 'ctx>,
}

impl<'a, 'ctx> Plugins<'a, 'ctx> {
    pub fn new(options: PluginsOptions, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self {
            styled_components: StyledComponents::new(
                options.styled_components.unwrap_or_default(),
                ctx,
            ),
        }
    }
}
