mod emotion;
mod options;
mod styled_components;
mod tagged_template_transform;

pub use emotion::EmotionOptions;
pub use options::PluginsOptions;
use oxc_ast::ast::*;
use oxc_data_structures::inline_string::InlineString;
use oxc_traverse::Traverse;
pub use styled_components::StyledComponentsOptions;

const fn default_as_true() -> bool {
    true
}

/// Encode a `u64` as a base-36 string (a-z 0-9), capped to at most 6 characters.
///
/// Used by both the styled-components and emotion plugins for file-hash generation.
#[inline]
fn base36_encode(mut num: u64) -> InlineString<7, u8> {
    const BASE36_BYTES: &[u8; 36] = b"abcdefghijklmnopqrstuvwxyz0123456789";

    num %= 36_u64.pow(6); // 36^6, to ensure the result is <= 6 characters long.

    let mut str = InlineString::new();
    while num != 0 {
        // SAFETY: `num < 36.pow(6)` to start with, and is divided by 36 on each turn of loop,
        // so we cannot push more than 6 bytes. Capacity of `InlineString` is 7.
        // All bytes in `BASE36_BYTES` are ASCII.
        unsafe { str.push_unchecked(BASE36_BYTES[(num % 36) as usize]) };
        num /= 36;
    }
    str
}

use crate::{
    context::TraverseCtx,
    plugins::{
        emotion::Emotion, styled_components::StyledComponents,
        tagged_template_transform::TaggedTemplateTransform,
    },
    state::TransformState,
};

pub struct Plugins<'a> {
    styled_components: Option<StyledComponents<'a>>,
    tagged_template_escape: Option<TaggedTemplateTransform>,
    emotion: Option<Emotion<'a>>,
}

impl Plugins<'_> {
    pub fn new(options: PluginsOptions) -> Self {
        Self {
            styled_components: options.styled_components.map(StyledComponents::new),
            tagged_template_escape: if options.tagged_template_transform {
                Some(TaggedTemplateTransform::new())
            } else {
                None
            },
            emotion: options.emotion.map(Emotion::new),
        }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for Plugins<'a> {
    fn enter_program(&mut self, node: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Some(styled_components) = &mut self.styled_components {
            styled_components.enter_program(node, ctx);
        }
        if let Some(emotion) = &mut self.emotion {
            emotion.enter_program(node, ctx);
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
        if let Some(emotion) = &mut self.emotion {
            emotion.enter_call_expression(node, ctx);
        }
    }
}
