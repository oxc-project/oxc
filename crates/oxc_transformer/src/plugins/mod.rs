mod options;
mod styled_components;
mod tagged_template_transform;

pub use options::PluginsOptions;
use oxc_ast::ast::*;
use oxc_traverse::Traverse;
pub use styled_components::StyledComponentsOptions;

use crate::{
    context::{TransformCtx, TraverseCtx},
    plugins::{
        styled_components::StyledComponents, tagged_template_transform::TaggedTemplateTransform,
    },
    state::TransformState,
};

pub struct Plugins<'a, 'ctx> {
    styled_components: Option<StyledComponents<'a, 'ctx>>,
    tagged_template_escape: Option<TaggedTemplateTransform<'a, 'ctx>>,
}

impl<'a, 'ctx> Plugins<'a, 'ctx> {
    pub fn new(options: PluginsOptions, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self {
            styled_components: options
                .styled_components
                .map(|options| StyledComponents::new(options, ctx)),
            tagged_template_escape: if options.tagged_template_transform {
                Some(TaggedTemplateTransform::new(ctx))
            } else {
                None
            },
        }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for Plugins<'a, '_> {
    fn enter_program(&mut self, node: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(styled_components) = &mut self.styled_components {
            styled_components.enter_program(node, ctx);
        }
    }

    fn enter_variable_declarator(
        &mut self,
        node: &mut VariableDeclarator<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if let Some(styled_components) = &mut self.styled_components {
            styled_components.enter_variable_declarator(node, ctx);
        }
    }

    fn enter_expression(
        &mut self,
        node: &mut Expression<'a>,
        ctx: &mut oxc_traverse::TraverseCtx<'a, TransformState<'a>>,
    ) {
        if let Some(styled_components) = &mut self.styled_components {
            styled_components.enter_expression(node, ctx);
        }
        if let Some(tagged_template_escape) = &mut self.tagged_template_escape {
            tagged_template_escape.enter_expression(node, ctx);
        }
    }

    fn enter_call_expression(&mut self, node: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(styled_components) = &mut self.styled_components {
            styled_components.enter_call_expression(node, ctx);
        }
    }
}
