use std::rc::Rc;

use oxc_ast::{ast::*, AstBuilder};
use oxc_span::Atom;
use oxc_syntax::identifier::{LS, PS};

use crate::{context::TransformerCtx, options::TransformTarget};

/// ES2019: Json Strings
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-json-strings>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-json-strings>
pub struct JsonStrings<'a> {
    ast: Rc<AstBuilder<'a>>,
}

impl<'a> JsonStrings<'a> {
    pub fn new(ctx: TransformerCtx<'a>) -> Option<Self> {
        (ctx.options.target < TransformTarget::ES2019 || ctx.options.json_strings)
            .then_some(Self { ast: ctx.ast })
    }

    // Allow `U+2028` and `U+2029` in string literals
    // <https://tc39.es/proposal-json-superset>
    // <https://github.com/tc39/proposal-json-superset>
    fn normalize_str(&self, str: &str) -> Option<Atom<'a>> {
        if !str.contains(LS) && !str.contains(PS) {
            return None;
        }
        let mut buf = String::new();
        let mut is_escaped = false;
        for c in str.chars() {
            match (is_escaped, c) {
                (false, LS) => buf.push_str("\\u2028"),
                (false, PS) => buf.push_str("\\u2029"),
                _ => buf.push(c),
            }
            is_escaped = !is_escaped && matches!(c, '\\');
        }
        Some(self.ast.new_atom(&buf))
    }

    #[allow(clippy::unused_self)]
    // TODO oxc_codegen currently prints json strings correctly,
    // but we need a way to turn off this behaviour from codegen
    // and do the transformation here.
    pub fn transform_string_literal(&mut self, _literal: &mut StringLiteral) {
        // let str = &self.ctx.semantic().source_text()[literal.span.start as usize + 1..literal.span.end as usize - 1];
        // if let Some(value) = Self::normalize_str(str) {
        //     literal.value = value;
        // }
    }

    #[allow(clippy::unused_self)]
    pub fn transform_directive(&mut self, directive: &mut Directive<'a>) {
        if let Some(value) = self.normalize_str(directive.directive.as_str()) {
            directive.directive = value;
        }
    }
}
