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
//! - `autoLabel`: Controls whether labels are added (`true` by default)
//! - `labelFormat`: Format string for labels with `[local]`, `[filename]`, `[dirname]`
//!
//! **❌ Not Yet Implemented:**
//! - `sourceMap`: Inline source map injection
//! - `importMap`: Custom import path mapping
//! - `cssPropOptimization`: JSX css prop transformation
//! - Tagged template literal transpilation and CSS minification
//! - Namespace import support (`import * as emotion from '@emotion/react'`)
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

use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
};

use rustc_hash::FxHasher;
use serde::Deserialize;

use oxc_allocator::TakeIn;
use oxc_ast::{NONE, ast::*};
use oxc_semantic::SymbolId;
use oxc_span::SPAN;
use oxc_traverse::{Ancestor, Traverse};

use crate::{context::TraverseCtx, state::TransformState};

use super::{base36_encode, default_as_true};

fn default_label_format() -> String {
    String::from("[local]")
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct EmotionOptions {
    /// Controls whether auto-generated labels are added to `css` and `styled` calls.
    ///
    /// Defaults to `true`.
    ///
    /// NOTE: For backwards compatibility, legacy string values are also accepted:
    /// - `"always"` / `"dev-only"` => `true`
    /// - `"never"` => `false`
    #[serde(default = "default_as_true", deserialize_with = "deserialize_auto_label")]
    pub auto_label: bool,

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

    /// NOT IMPLEMENTED YET.
    ///
    /// This option is accepted for config compatibility, but currently ignored.
    #[serde(default = "default_as_true")]
    pub source_map: bool,

    /// NOT IMPLEMENTED YET.
    ///
    /// This option is accepted for config compatibility, but currently ignored.
    #[serde(default)]
    pub import_map: Option<serde_json::Value>,

    /// NOT IMPLEMENTED YET.
    ///
    /// This option is accepted for config compatibility, but currently ignored.
    #[serde(default = "default_as_true")]
    pub css_prop_optimization: bool,
}

impl Default for EmotionOptions {
    fn default() -> Self {
        Self {
            auto_label: default_as_true(),
            label_format: default_label_format(),
            source_map: default_as_true(),
            import_map: None,
            css_prop_optimization: default_as_true(),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum AutoLabelValue {
    Bool(bool),
    Legacy(AutoLabelLegacy),
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum AutoLabelLegacy {
    DevOnly,
    Always,
    Never,
}

fn deserialize_auto_label<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = AutoLabelValue::deserialize(deserializer)?;
    Ok(match value {
        AutoLabelValue::Bool(value) => value,
        AutoLabelValue::Legacy(AutoLabelLegacy::Never) => false,
        AutoLabelValue::Legacy(AutoLabelLegacy::Always | AutoLabelLegacy::DevOnly) => true,
    })
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

pub struct Emotion<'a> {
    options: EmotionOptions,

    /// All tracked emotion import bindings
    bindings: Vec<EmotionBinding>,
    /// Counter for generating unique target class names
    target_count: usize,
    /// Cached file hash prefix for target class names (`e<hash>`)
    target_prefix: Option<String>,
    /// Cached filename (without extension)
    cached_filename: Option<Atom<'a>>,
    /// Cached dirname
    cached_dirname: Option<Atom<'a>>,
}

impl Emotion<'_> {
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

impl<'a> Traverse<'a, TransformState<'a>> for Emotion<'a> {
    fn enter_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.collect_emotion_bindings(program);

        self.cached_filename = ctx
            .state
            .source_path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| ctx.ast.atom(s));

        self.cached_dirname = ctx
            .state
            .source_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .map(|s| ctx.ast.atom(s));
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
        }
    }
}

impl<'a> Emotion<'a> {
    /// Scan import declarations and collect emotion bindings.
    fn collect_emotion_bindings(&mut self, program: &Program<'_>) {
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
                            self.bindings
                                .push(EmotionBinding { symbol_id: s.local.symbol_id(), kind });
                        }
                    }
                    ImportDeclarationSpecifier::ImportSpecifier(s) => {
                        let imported_name = s.imported.name();
                        if imported_name == "default" {
                            if let Some(kind) = package_config.default_export {
                                self.bindings
                                    .push(EmotionBinding { symbol_id: s.local.symbol_id(), kind });
                            }
                        } else if let Some(kind) =
                            package_config.get_named_export(imported_name.as_str())
                        {
                            self.bindings
                                .push(EmotionBinding { symbol_id: s.local.symbol_id(), kind });
                        }
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                        // TODO: Namespace imports are not yet supported.
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
        self.bindings.iter().find(|b| b.symbol_id == ref_symbol_id).map(|b| b.kind)
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
            && let Some(label) = Self::get_context_name(ctx)
        {
            let label_str = self.format_label(&label, ctx);
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
        if inner_call.arguments.len() >= 2 {
            if let Some(Argument::ObjectExpression(obj)) = inner_call.arguments.get_mut(1) {
                if !Self::has_static_property(&obj.properties, "target") {
                    let target = self.get_target_class_name(ctx);
                    obj.properties.push(Self::create_object_property("target", target, ctx));
                }

                if self.should_add_label()
                    && !Self::has_static_property(&obj.properties, "label")
                    && let Some(label) = Self::get_context_name(ctx)
                {
                    let label_atom = self.format_label_atom(&label, ctx);
                    obj.properties.push(Self::create_object_property("label", label_atom, ctx));
                }
            } else {
                let opts_arg = self.create_styled_options_arg(ctx);
                inner_call.arguments.push(opts_arg);
            }
        } else {
            let opts_arg = self.create_styled_options_arg(ctx);
            inner_call.arguments.push(opts_arg);
        }
    }

    /// Create `Argument::ObjectExpression({ target: 'eXXX', label: 'Name' })`
    fn create_styled_options_arg(&mut self, ctx: &TraverseCtx<'a>) -> Argument<'a> {
        let mut properties = ctx.ast.vec_with_capacity(2);

        let target = self.get_target_class_name(ctx);
        properties.push(Self::create_object_property("target", target, ctx));

        if self.should_add_label()
            && let Some(label) = Self::get_context_name(ctx)
        {
            let label_atom = self.format_label_atom(&label, ctx);
            properties.push(Self::create_object_property("label", label_atom, ctx));
        }

        Argument::ObjectExpression(ctx.ast.alloc_object_expression(SPAN, properties))
    }

    fn should_add_label(&self) -> bool {
        self.options.auto_label
    }

    /// Build the `;label:<formatted>;` string for css calls.
    fn format_label(&self, local_name: &Atom<'a>, ctx: &TraverseCtx<'a>) -> Atom<'a> {
        let sanitized = sanitize_label_part(local_name);

        if self.options.label_format == "[local]" {
            return ctx.ast.atom_from_strs_array([";label:", &sanitized, ";"]);
        }

        let formatted = self.interpolate_label_format(&sanitized);
        ctx.ast.atom_from_strs_array([";label:", &formatted, ";"])
    }

    /// Build just the label value (e.g. `"MyComp"`) for styled options objects.
    fn format_label_atom(&self, local_name: &Atom<'a>, ctx: &TraverseCtx<'a>) -> Atom<'a> {
        let sanitized = sanitize_label_part(local_name);

        if self.options.label_format == "[local]" {
            return ctx.ast.atom(&sanitized);
        }

        let formatted = self.interpolate_label_format(&sanitized);
        ctx.ast.atom(&formatted)
    }

    /// Interpolate the `labelFormat` template with `[local]`, `[filename]`, `[dirname]`.
    ///
    /// Not called for the default `"[local]"` format — that case is short-circuited
    /// in the callers above.
    fn interpolate_label_format(&self, sanitized_local: &str) -> String {
        let format = &self.options.label_format;
        let mut result = String::with_capacity(format.len() + sanitized_local.len());
        let mut remaining = format.as_str();

        while let Some(bracket_pos) = remaining.find('[') {
            result.push_str(&remaining[..bracket_pos]);
            let rest = &remaining[bracket_pos..];
            if let Some(end_pos) = rest.find(']') {
                let placeholder = &rest[1..end_pos];
                match placeholder {
                    "local" => result.push_str(sanitized_local),
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
                remaining = &rest[end_pos + 1..];
            } else {
                result.push_str(rest);
                remaining = "";
            }
        }
        result.push_str(remaining);

        result
    }

    /// Infer the context name from the enclosing variable declarator, assignment,
    /// object property, or class field.
    fn get_context_name(ctx: &TraverseCtx<'a>) -> Option<Atom<'a>> {
        let mut assignment_name = None;

        for ancestor in ctx.ancestors() {
            match ancestor {
                Ancestor::AssignmentExpressionRight(assignment) => {
                    assignment_name = match assignment.left() {
                        AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                            Some(ident.name.into())
                        }
                        AssignmentTarget::StaticMemberExpression(member) => {
                            Some(member.property.name.into())
                        }
                        _ => return None,
                    };
                }
                Ancestor::VariableDeclaratorInit(declarator) => {
                    return if let BindingPattern::BindingIdentifier(ident) = &declarator.id() {
                        Some(ident.name.into())
                    } else {
                        None
                    };
                }
                Ancestor::ObjectPropertyValue(property) => {
                    return match property.key() {
                        PropertyKey::StaticIdentifier(ident) => Some(ident.name.into()),
                        PropertyKey::StringLiteral(s) => Some(s.value),
                        _ => None,
                    };
                }
                Ancestor::PropertyDefinitionValue(property) => {
                    return if let PropertyKey::StaticIdentifier(ident) = property.key() {
                        Some(ident.name.into())
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

    fn get_file_hash(state: &TransformState<'_>) -> impl std::fmt::Display {
        let mut hasher = FxHasher::default();
        if state.source_path.is_absolute() {
            state.source_path.hash(&mut hasher);
        } else {
            state.source_text.hash(&mut hasher);
        }

        base36_encode(hasher.finish())
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

    fn has_static_property(properties: &[ObjectPropertyKind<'a>], name: &str) -> bool {
        properties.iter().any(|property| {
            matches!(property, ObjectPropertyKind::ObjectProperty(property)
                if matches!(&property.key, PropertyKey::StaticIdentifier(ident) if ident.name == name)
            )
        })
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
            "@emotion/styled" => {
                Some(Self { default_export: Some(EmotionExprKind::Styled), named_exports: &[] })
            }
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
        self.named_exports.iter().find(|(n, _)| *n == name).map(|(_, kind)| *kind)
    }
}

/// Replace characters that are invalid in CSS class names with hyphens,
/// and collapse consecutive invalid character runs into a single hyphen.
///
/// Returns `Cow::Borrowed` when no replacement is needed (common case for
/// JS identifiers), avoiding a heap allocation.
fn sanitize_label_part(input: &str) -> Cow<'_, str> {
    let needs_sanitization = input.bytes().any(|b| {
        !(b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    });

    if !needs_sanitization {
        return Cow::Borrowed(input);
    }

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

    Cow::Owned(result)
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
