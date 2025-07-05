mod options;
mod styled_components;

pub use options::PluginsOptions;
use oxc_ast::ast::*;
use oxc_traverse::Traverse;
pub use styled_components::StyledComponentsOptions;

use crate::{
    context::{TransformCtx, TraverseCtx},
    plugins::styled_components::StyledComponents,
    state::TransformState,
};

pub struct Plugins<'a, 'ctx> {
    styled_components: StyledComponents<'a, 'ctx>,
    options: PluginsOptions,
}

impl<'a, 'ctx> Plugins<'a, 'ctx> {
    pub fn new(options: PluginsOptions, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self {
            styled_components: StyledComponents::new(
                options.styled_components.clone().unwrap_or_default(),
                ctx,
            ),
            options,
        }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for Plugins<'a, '_> {
    fn enter_program(&mut self, node: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.styled_components.is_some() {
            self.styled_components.enter_program(node, ctx);
        }
    }

    fn enter_tagged_template_expression(
        &mut self,
        node: &mut TaggedTemplateExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.styled_components.is_some() {
            self.styled_components.enter_tagged_template_expression(node, ctx);
        }
    }

    fn enter_call_expression(&mut self, node: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.styled_components.is_some() {
            self.styled_components.enter_call_expression(node, ctx);
        }
    }
}
