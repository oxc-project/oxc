use oxc_ast::ast::*;
use oxc_span::Atom;
use oxc_syntax::identifier::{LS, PS};

use crate::options::{TransformOptions, TransformTarget};

/// ES2019: Json Strings
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-json-strings>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-json-strings>
pub struct JsonStrings;

impl JsonStrings {
    pub fn new(options: &TransformOptions) -> Option<Self> {
        (options.target < TransformTarget::ES2019 || options.json_strings).then(|| Self {})
    }

    // Allow `U+2028` and `U+2029` in string literals
    // <https://tc39.es/proposal-json-superset>
    // <https://github.com/tc39/proposal-json-superset>
    fn normalize_str(str: &str) -> Option<Atom> {
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
        Some(buf.into())
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
    pub fn transform_directive(&mut self, directive: &mut Directive) {
        if let Some(value) = Self::normalize_str(directive.directive.as_str()) {
            directive.directive = value;
        }
    }
}
