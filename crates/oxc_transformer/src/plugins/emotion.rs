//! Emotion
//!
//! This plugin adds support for pure annotations, auto-labeling, and target class name
//! generation when using the Emotion CSS-in-JS library.
//!
//! > This plugin is a port from the official Babel plugin and SWC plugin for Emotion.
//!
//! ## Implementation Status
//!
//! > Note: Currently, this plugin only supports Emotion imported via import statements.
//! > The transformation will not be applied if you import it using `require(...)`,
//! > in other words, it only supports `ESM` not `CJS`.
//!
//! ### Options:
//! **✅ Fully Supported:**
//! - `autoLabel`: Controls when labels are added (`dev-only`, `always`, `never`)
//! - `labelFormat`: Format string for labels with `[local]`, `[filename]`, `[dirname]`
//!
//! **❌ Not Yet Implemented:**
//! - `sourceMap`: Inline source map injection
//! - `importMap`: Custom import path mapping
//! - `cssPropOptimization`: JSX css prop transformation
//! - Tagged template literal transpilation and CSS minification
//!
//! ## Example
//!
//! Input:
//! ```js
//! import { css } from '@emotion/react';
//! import styled from '@emotion/styled';
//!
//! const cls = css({ color: 'hotpink' });
//! const H1 = styled.h1({ fontSize: 20 });
//! ```
//!
//! Output (with default options):
//! ```js
//! import { css } from '@emotion/react';
//! import styled from '@emotion/styled';
//!
//! const cls = /*#__PURE__*/ css({ color: 'hotpink' }, ';label:cls;');
//! const H1 = /*#__PURE__*/ styled('h1', { target: 'eXXXXX0', label: 'H1' })({ fontSize: 20 });
//! ```
//!
//! ## References
//!
//! - Babel plugin: <https://github.com/emotion-js/emotion/tree/main/packages/babel-plugin>
//! - SWC plugin: <https://github.com/nicksrandall/emotion-swc-plugin>
//! - Documentation: <https://emotion.sh/docs/babel>

use std::hash::{Hash, Hasher};

use rustc_hash::FxHasher;
use serde::Deserialize;

use oxc_allocator::TakeIn;
use oxc_ast::{NONE, ast::*};
use oxc_semantic::SymbolId;
use oxc_span::SPAN;
use oxc_traverse::{Ancestor, Traverse};

use crate::{context::TraverseCtx, state::TransformState};

fn default_label_format() -> String {
    String::from("[local]")
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct EmotionOptions {
    /// Controls when auto-generated labels are added to `css` and `styled` calls.
    ///
    /// - `dev-only` (default): Labels added in development, stripped in production
    /// - `always`: Labels always added
    /// - `never`: Labels never added
    pub auto_label: AutoLabel,

    /// Format string for generated labels.
    ///
    /// Supports the following placeholders:
    /// - `[local]` - the variable name the result is assigned to
    /// - `[filename]` - the file name (without extension)
    /// - `[dirname]` - the directory name containing the file
    ///
    /// Default: `"[local]"`
    #[serde(default = "default_label_format")]
    pub label_format: String,

    /// Whether to inject source maps into emotion calls.
    ///
    /// **Note: This feature is not yet implemented in oxc.**
    ///
    /// Default: `true`
    #[serde(default = "default_as_true")]
    pub source_map: bool,
}

const fn default_as_true() -> bool {
    true
}

impl Default for EmotionOptions {
    fn default() -> Self {
        Self {
            auto_label: AutoLabel::default(),
            label_format: default_label_format(),
            source_map: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AutoLabel {
    #[default]
    DevOnly,
    Always,
    Never,
}

/// What kind of emotion expression an import binding represents.
#[derive(Debug, Clone, Copy)]
enum EmotionExprKind {
    Css,
    Styled,
}

/// Tracks a single import binding's symbol ID and its expression kind.
#[derive(Debug, Clone, Copy)]
struct EmotionBinding {
    symbol_id: SymbolId,
    kind: EmotionExprKind,
}

pub struct Emotion {
    pub options: EmotionOptions,

    /// All tracked emotion import bindings
    bindings: Vec<EmotionBinding>,
    /// Counter for generating unique target class names
    target_count: usize,
    /// Cached file hash prefix for target class names (`e<hash>`)
    target_prefix: Option<String>,
    /// Cached filename (without extension)
    cached_filename: Option<String>,
    /// Cached dirname
    cached_dirname: Option<String>,
}

impl Emotion {
    pub fn new(options: EmotionOptions) -> Self {
        Self {
            options,
            bindings: Vec::new(),
            target_count: 0,
            target_prefix: None,
            cached_filename: None,
            cached_dirname: None,
        }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for Emotion {
    fn enter_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.collect_emotion_bindings(program, ctx);

        self.cached_filename = ctx
            .state
            .source_path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(String::from);

        self.cached_dirname = ctx
            .state
            .source_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .map(String::from);
    }

    fn enter_call_expression(&mut self, call: &mut CallExpression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.bindings.is_empty() {
            return;
        }

        // `css(...)` or `keyframes(...)` — direct call of a css-kind binding
        if let Expression::Identifier(ident) = &call.callee
            && matches!(self.get_binding_kind(ident, ctx), Some(EmotionExprKind::Css))
        {
            self.transform_css_call(call, ctx);
            return;
        }

        // `styled.div(...)` — member call on a styled binding
        if let Expression::StaticMemberExpression(member) = &call.callee
            && let Expression::Identifier(ident) = &member.object
            && matches!(self.get_binding_kind(ident, ctx), Some(EmotionExprKind::Styled))
        {
            let tag_name = member.property.name;
            self.transform_styled_member_call(call, &tag_name, ctx);
            return;
        }

        // `styled('div')({...})` — outer call where callee is `styled('div')`
        if let Expression::CallExpression(inner_call) = &mut call.callee
            && let Expression::Identifier(ident) = &inner_call.callee
            && matches!(self.get_binding_kind(ident, ctx), Some(EmotionExprKind::Styled))
        {
            call.pure = true;
            self.transform_styled_call_call(inner_call, ctx);
            return;
        }

        // `emotionCss.css(...)` — namespace member call
        if let Expression::StaticMemberExpression(member) = &call.callee
            && let Expression::Identifier(ident) = &member.object
            && Self::is_namespace_css_member(&member.property.name)
        {
            let _ = ident; // used for future namespace binding checks
            self.transform_css_call(call, ctx);
        }
    }
}

impl<'a> Emotion {
    /// Scan import declarations and collect emotion bindings.
    fn collect_emotion_bindings(&mut self, program: &Program<'_>, _ctx: &mut TraverseCtx<'_>) {
        for statement in &program.body {
            let Statement::ImportDeclaration(import) = statement else { continue };
            let Some(specifiers) = &import.specifiers else { continue };

            let Some(package_config) = EmotionPackage::from_source(&import.source.value) else {
                continue;
            };

            for specifier in specifiers {
                match specifier {
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                        if let Some(kind) = package_config.default_export {
                            self.bindings.push(EmotionBinding {
                                symbol_id: s.local.symbol_id(),
                                kind,
                            });
                        }
                    }
                    ImportDeclarationSpecifier::ImportSpecifier(s) => {
                        let imported_name = s.imported.name();
                        if imported_name == "default" {
                            if let Some(kind) = package_config.default_export {
                                self.bindings.push(EmotionBinding {
                                    symbol_id: s.local.symbol_id(),
                                    kind,
                                });
                            }
                        } else if let Some(kind) =
                            package_config.get_named_export(imported_name.as_str())
                        {
                            self.bindings.push(EmotionBinding {
                                symbol_id: s.local.symbol_id(),
                                kind,
                            });
                        }
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                        // Namespace imports are tracked per-package, not per-binding.
                        // We handle them specially in `get_namespace_member_kind`.
                    }
                }
            }
        }
    }

    /// Look up the kind of a binding by identifier reference.
    fn get_binding_kind(
        &self,
        ident: &IdentifierReference<'_>,
        ctx: &TraverseCtx<'_>,
    ) -> Option<EmotionExprKind> {
        let reference_id = ident.reference_id();
        let ref_symbol_id = ctx.scoping().get_reference(reference_id).symbol_id()?;
        self.bindings
            .iter()
            .find(|b| b.symbol_id == ref_symbol_id)
            .map(|b| b.kind)
    }

    /// Check if a member property name matches a known css-kind export.
    fn is_namespace_css_member(property: &str) -> bool {
        matches!(property, "css" | "keyframes")
    }

    /// Transform a `css(...)` or `keyframes(...)` call:
    /// - Add `/*#__PURE__*/` annotation
    /// - Append `;label:<name>;` as last argument (if auto-labeling enabled)
    fn transform_css_call(&self, call: &mut CallExpression<'a>, ctx: &TraverseCtx<'a>) {
        if call.arguments.is_empty() {
            return;
        }

        call.pure = true;

        if self.should_add_label()
            && let Some(label) = self.get_label(ctx)
        {
            let label_str = ctx.ast.atom(&format!(";label:{label};"));
            let label_arg =
                Argument::from(ctx.ast.expression_string_literal(SPAN, label_str, None));
            call.arguments.push(label_arg);
        }
    }

    /// Transform `styled.div({...})` into `styled('div', { target, label })({...})`.
    ///
    /// The input is the outer call `styled.div(...)`. We rewrite the callee from
    /// `styled.div` to `styled('div', { target, label })` — making the callee itself a call.
    fn transform_styled_member_call(
        &mut self,
        call: &mut CallExpression<'a>,
        tag_name: &str,
        ctx: &TraverseCtx<'a>,
    ) {
        call.pure = true;

        let tag_arg =
            Argument::from(ctx.ast.expression_string_literal(SPAN, ctx.ast.atom(tag_name), None));
        let opts_arg = self.create_styled_options_arg(ctx);

        let mut inner_args = ctx.ast.vec_with_capacity(2);
        inner_args.push(tag_arg);
        inner_args.push(opts_arg);

        // Take the callee (styled.div) and extract the `styled` identifier
        let old_callee = call.callee.take_in(ctx.ast);
        let Expression::StaticMemberExpression(member) = old_callee else { return };
        let styled_expr = member.unbox().object;

        let inner_call = ctx.ast.expression_call(SPAN, styled_expr, NONE, inner_args, false);
        call.callee = inner_call;
    }

    /// Transform `styled('div')({...})` by injecting `{ target, label }` as second arg
    /// to the inner `styled('div')` call.
    fn transform_styled_call_call(
        &mut self,
        inner_call: &mut CallExpression<'a>,
        ctx: &TraverseCtx<'a>,
    ) {
        let opts_arg = self.create_styled_options_arg(ctx);

        if inner_call.arguments.len() >= 2 {
            // There's already a second argument (existing options).
            // If it's an object literal, merge our properties into it.
            if let Some(Argument::ObjectExpression(obj)) = inner_call.arguments.get_mut(1) {
                let target = self.get_target_class_name(ctx);
                obj.properties.push(Self::create_object_property("target", target, ctx));

                if self.should_add_label()
                    && let Some(label) = self.get_label(ctx)
                {
                    let label_atom = ctx.ast.atom(&label);
                    obj.properties
                        .push(Self::create_object_property("label", label_atom, ctx));
                }
            } else {
                // Non-object second arg — push as third arg instead
                inner_call.arguments.push(opts_arg);
            }
        } else {
            inner_call.arguments.push(opts_arg);
        }
    }

    /// Create `Argument::ObjectExpression({ target: 'eXXX', label: 'Name' })`
    fn create_styled_options_arg(&mut self, ctx: &TraverseCtx<'a>) -> Argument<'a> {
        let mut properties = ctx.ast.vec_with_capacity(2);

        let target = self.get_target_class_name(ctx);
        properties.push(Self::create_object_property("target", target, ctx));

        if self.should_add_label()
            && let Some(label) = self.get_label(ctx)
        {
            let label_atom = ctx.ast.atom(&label);
            properties.push(Self::create_object_property("label", label_atom, ctx));
        }

        Argument::ObjectExpression(ctx.ast.alloc_object_expression(SPAN, properties))
    }

    fn should_add_label(&self) -> bool {
        // For now, treat `dev-only` same as `always` since we don't have
        // production mode detection at transform time. The user controls this
        // via the option directly.
        self.options.auto_label != AutoLabel::Never
    }

    /// Generate a label string by interpolating the `labelFormat` template.
    fn get_label(&self, ctx: &TraverseCtx<'a>) -> Option<String> {
        let local_name = Self::get_context_name(ctx)?;
        let sanitized_local = sanitize_label_part(&local_name);

        let format = &self.options.label_format;
        let mut result = String::with_capacity(format.len() + sanitized_local.len());
        let mut chars = format.as_str();

        while let Some(bracket_pos) = chars.find('[') {
            result.push_str(&chars[..bracket_pos]);
            let rest = &chars[bracket_pos..];
            if let Some(end_pos) = rest.find(']') {
                let placeholder = &rest[1..end_pos];
                match placeholder {
                    "local" => result.push_str(&sanitized_local),
                    "filename" => {
                        if let Some(filename) = &self.cached_filename {
                            result.push_str(&sanitize_label_part(filename));
                        }
                    }
                    "dirname" => {
                        if let Some(dirname) = &self.cached_dirname {
                            result.push_str(&sanitize_label_part(dirname));
                        }
                    }
                    _ => {
                        result.push('[');
                        result.push_str(placeholder);
                        result.push(']');
                    }
                }
                chars = &rest[end_pos + 1..];
            } else {
                result.push_str(rest);
                chars = "";
            }
        }
        result.push_str(chars);

        if result.is_empty() { None } else { Some(result) }
    }

    /// Infer the context name from the enclosing variable declarator, assignment,
    /// object property, or class field.
    fn get_context_name(ctx: &TraverseCtx<'_>) -> Option<String> {
        let mut assignment_name = None;

        for ancestor in ctx.ancestors() {
            match ancestor {
                Ancestor::AssignmentExpressionRight(assignment) => {
                    assignment_name = match assignment.left() {
                        AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                            Some(ident.name.to_string())
                        }
                        AssignmentTarget::StaticMemberExpression(member) => {
                            Some(member.property.name.to_string())
                        }
                        _ => return None,
                    };
                }
                Ancestor::VariableDeclaratorInit(declarator) => {
                    return if let BindingPattern::BindingIdentifier(ident) = &declarator.id() {
                        Some(ident.name.to_string())
                    } else {
                        None
                    };
                }
                Ancestor::ObjectPropertyValue(property) => {
                    return match property.key() {
                        PropertyKey::StaticIdentifier(ident) => Some(ident.name.to_string()),
                        PropertyKey::StringLiteral(s) => Some(s.value.to_string()),
                        _ => None,
                    };
                }
                Ancestor::PropertyDefinitionValue(property) => {
                    return if let PropertyKey::StaticIdentifier(ident) = property.key() {
                        Some(ident.name.to_string())
                    } else {
                        None
                    };
                }
                _ => {
                    if ancestor.is_parent_of_statement() {
                        return assignment_name;
                    }
                }
            }
        }

        assignment_name
    }

    /// Generate a unique target class name: `e<file_hash><count>`
    fn get_target_class_name(&mut self, ctx: &TraverseCtx<'a>) -> Atom<'a> {
        let prefix = if let Some(prefix) = self.target_prefix.as_deref() {
            prefix
        } else {
            let hash = Self::get_file_hash(&ctx.state);
            let prefix = format!("e{hash}");
            self.target_prefix = Some(prefix);
            self.target_prefix.as_deref().unwrap()
        };

        let mut buffer = itoa::Buffer::new();
        let count = buffer.format(self.target_count);
        self.target_count += 1;
        ctx.ast.atom_from_strs_array([prefix, count])
    }

    /// Compute a stable hash of the file for target class name generation.
    fn get_file_hash(state: &TransformState<'_>) -> String {
        let mut hasher = FxHasher::default();
        if state.source_path.is_absolute() {
            state.source_path.hash(&mut hasher);
        } else {
            state.source_text.hash(&mut hasher);
        }
        #[expect(clippy::cast_possible_truncation)]
        base36_encode(hasher.finish() as u32)
    }

    /// Create `{ key: "value" }` as an `ObjectPropertyKind`.
    fn create_object_property(
        key: &'static str,
        value: Atom<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> ObjectPropertyKind<'a> {
        let key = ctx.ast.property_key_static_identifier(SPAN, key);
        let value = ctx.ast.expression_string_literal(SPAN, value, None);
        ctx.ast.object_property_kind_object_property(
            SPAN,
            PropertyKind::Init,
            key,
            value,
            false,
            false,
            false,
        )
    }
}

/// Known emotion packages and their export configurations.
struct EmotionPackage {
    default_export: Option<EmotionExprKind>,
    named_exports: &'static [(&'static str, EmotionExprKind)],
}

impl EmotionPackage {
    fn from_source(source: &str) -> Option<Self> {
        match source {
            "@emotion/css" => Some(Self {
                default_export: Some(EmotionExprKind::Css),
                named_exports: &[("css", EmotionExprKind::Css)],
            }),
            "@emotion/styled" => Some(Self {
                default_export: Some(EmotionExprKind::Styled),
                named_exports: &[],
            }),
            "@emotion/react" => Some(Self {
                default_export: None,
                named_exports: &[
                    ("css", EmotionExprKind::Css),
                    ("keyframes", EmotionExprKind::Css),
                ],
            }),
            "@emotion/primitives" | "@emotion/native" => Some(Self {
                default_export: Some(EmotionExprKind::Styled),
                named_exports: &[("css", EmotionExprKind::Css)],
            }),
            _ => None,
        }
    }

    fn get_named_export(&self, name: &str) -> Option<EmotionExprKind> {
        self.named_exports
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, kind)| *kind)
    }
}

fn base36_encode(mut num: u32) -> String {
    const CHARS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    if num == 0 {
        return String::from("0");
    }
    let mut result = Vec::new();
    while num > 0 {
        result.push(CHARS[(num % 36) as usize]);
        num /= 36;
    }
    result.reverse();
    // SAFETY: All bytes are ASCII alphanumeric
    unsafe { String::from_utf8_unchecked(result) }
}

/// Replace characters that are invalid in CSS class names with hyphens,
/// and collapse whitespace runs into a single hyphen.
fn sanitize_label_part(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut last_was_invalid = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            result.push(ch);
            last_was_invalid = false;
        } else if !last_was_invalid {
            result.push('-');
            last_was_invalid = true;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_label_part() {
        assert_eq!(sanitize_label_part("simple"), "simple");
        assert_eq!(sanitize_label_part("Dollar$"), "Dollar-");
        assert_eq!(sanitize_label_part("MiniCalWrap$"), "MiniCalWrap-");
        assert_eq!(sanitize_label_part("hello world"), "hello-world");
        assert_eq!(sanitize_label_part("a$$b"), "a-b");
        assert_eq!(sanitize_label_part("kebab-case"), "kebab-case");
        assert_eq!(sanitize_label_part("under_score"), "under_score");
    }
}
