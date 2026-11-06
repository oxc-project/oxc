use std::ffi::OsStr;

use cow_utils::CowUtils;
use lazy_regex::Regex;
use oxc_ast::AstKind;
use oxc_ast::ast::{PropertyKey, TSTypePredicateName};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;
use oxc_syntax::identifier::is_identifier_name;
use oxc_syntax::keyword::is_reserved_keyword;
use oxc_syntax::symbol::SymbolId;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    fixer::RuleFixer,
    rule::{DefaultRuleConfig, Rule},
    utils::deserialize_regex_vec,
};

fn variable_diagnostic(name: &str, replacement: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The variable `{name}` should be named `{replacement}`. A more descriptive name will do too."
    ))
    .with_label(span)
}

fn variable_suggestion_diagnostic(
    name: &str,
    replacements: &[CompactStr],
    span: Span,
) -> OxcDiagnostic {
    let replacements_text =
        replacements.iter().map(|r| format!("`{r}`")).collect::<Vec<_>>().join(", ");
    OxcDiagnostic::warn(format!(
        "Please rename the variable `{name}`. Suggested names are: {replacements_text}. A more descriptive name will do too."
    ))
    .with_help("Choose the replacement that best fits the context.")
    .with_label(span)
}

fn filename_diagnostic(filename: &str, replacement: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The filename `{filename}` should be named `{replacement}`. A more descriptive name will do too."
    ))
}

fn filename_suggestion_diagnostic(filename: &str, replacements: &[String]) -> OxcDiagnostic {
    let replacements_text =
        replacements.iter().map(|r| format!("`{r}`")).collect::<Vec<_>>().join(", ");
    OxcDiagnostic::warn(format!(
        "Please rename the filename `{filename}`. Suggested names are: {replacements_text}. A more descriptive name will do too."
    ))
    .with_help("Choose the replacement that best fits the context.")
}

fn property_diagnostic(name: &str, replacement: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The property `{name}` should be named `{replacement}`. A more descriptive name will do too."
    ))
    .with_label(span)
}

fn property_suggestion_diagnostic(
    name: &str,
    replacements: &[CompactStr],
    span: Span,
) -> OxcDiagnostic {
    let replacements_text =
        replacements.iter().map(|r| format!("`{r}`")).collect::<Vec<_>>().join(", ");
    OxcDiagnostic::warn(format!(
        "Please rename the property `{name}`. Suggested names are: {replacements_text}. A more descriptive name will do too."
    ))
    .with_help("Choose the replacement that best fits the context.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NameReplacements(Box<NameReplacementsConfig>);

impl std::ops::Deref for NameReplacements {
    type Target = NameReplacementsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Value for `checkDefaultAndNamespaceImports` and `checkShorthandImports` options.
/// Can be a boolean or the string "internal".
#[derive(Debug, Clone, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
pub enum ImportCheckMode {
    Bool(bool),
    Internal(InternalMode),
}

#[derive(Debug, Clone, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum InternalMode {
    #[serde(rename = "internal")]
    Internal,
}

impl Default for ImportCheckMode {
    fn default() -> Self {
        Self::Internal(InternalMode::Internal)
    }
}

impl ImportCheckMode {
    fn should_check(&self, is_internal: bool) -> bool {
        match self {
            Self::Bool(false) => false,
            Self::Bool(true) => true,
            Self::Internal(_) => is_internal,
        }
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NameReplacementsConfig {
    /// Whether to check variable names.
    #[serde(default = "default_true")]
    check_variables: bool,

    /// Whether to check property names.
    #[serde(default)]
    check_properties: bool,

    /// Whether to check default and namespace import names.
    /// Can be `true`, `false`, or `"internal"` (default).
    /// `"internal"` only checks imports from relative paths.
    #[serde(default)]
    check_default_and_namespace_imports: ImportCheckMode,

    /// Whether to check shorthand import names.
    /// Can be `true`, `false`, or `"internal"` (default).
    /// `"internal"` only checks imports from relative paths.
    #[serde(default)]
    check_shorthand_imports: ImportCheckMode,

    /// Whether to check shorthand property names in destructuring.
    #[serde(default)]
    check_shorthand_properties: bool,

    /// Whether to check filenames.
    #[serde(default = "default_true")]
    check_filenames: bool,

    /// Whether to extend the default replacements with custom ones.
    #[serde(default = "default_true")]
    extend_default_replacements: bool,

    /// Custom name replacements. Maps abbreviated names to their full replacements.
    /// Set a name to `false` to disable checking that abbreviation.
    #[serde(default)]
    replacements: FxHashMap<String, ReplacementValue>,

    /// Whether to extend the default allow list.
    #[serde(default = "default_true")]
    extend_default_allow_list: bool,

    /// Custom allow list of names that should not be checked.
    #[serde(default)]
    allow_list: FxHashMap<String, bool>,

    /// Patterns to ignore.
    #[serde(default, deserialize_with = "deserialize_regex_vec")]
    ignore: Vec<Regex>,
}

fn default_true() -> bool {
    true
}

/// Configuration value for a replacement entry.
/// Can be `false` to disable checking, or a map of replacement options.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ReplacementValue {
    /// Disable checking for this abbreviation
    Disabled(bool),
    /// Map of replacement names to whether they're enabled
    Map(FxHashMap<String, bool>),
}

impl Default for NameReplacementsConfig {
    fn default() -> Self {
        Self {
            check_variables: true,
            check_properties: false,
            check_default_and_namespace_imports: ImportCheckMode::default(),
            check_shorthand_imports: ImportCheckMode::default(),
            check_shorthand_properties: false,
            check_filenames: true,
            extend_default_replacements: true,
            replacements: FxHashMap::default(),
            extend_default_allow_list: true,
            allow_list: FxHashMap::default(),
            ignore: vec![],
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces descriptive names by preventing the use of common
    /// abbreviations and suggesting full words instead.
    ///
    /// ### Why is this bad?
    ///
    /// Abbreviated names like `err`, `cb`, `fn`, `str`, etc. reduce
    /// code readability. Using full, descriptive names like `error`, `callback`,
    /// `function`, `string` makes code self-documenting and easier to understand,
    /// especially for newcomers to a codebase.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const err = new Error('message');
    /// const cb = () => {};
    /// function processStr(str) { return str.trim(); }
    /// const btn = document.querySelector('button');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const error = new Error('message');
    /// const callback = () => {};
    /// function processString(string) { return string.trim(); }
    /// const button = document.querySelector('button');
    /// ```
    ///
    /// ### Options
    ///
    /// ```json
    /// {
    ///   "unicorn/name-replacements": ["error", {
    ///     "checkVariables": true,
    ///     "checkProperties": false,
    ///     "checkFilenames": true,
    ///     "checkDefaultAndNamespaceImports": "internal",
    ///     "checkShorthandImports": "internal",
    ///     "checkShorthandProperties": false,
    ///     "replacements": {
    ///       "e": { "error": true, "event": true },
    ///       "db": false
    ///     },
    ///     "allowList": {
    ///       "propTypes": true
    ///     },
    ///     "ignore": ["i18n", "^_"]
    ///   }]
    /// }
    /// ```
    ///
    /// ### Limitations
    ///
    /// Unlike ESLint's rule, auto-fix is not skipped for Vue template refs.
    /// Manual review is recommended when using Vue.
    NameReplacements,
    unicorn,
    style,
    fix_suggestion,
    config = NameReplacementsConfig,
    version = "0.0.0",
    short_description = "Enforces descriptive names by preventing common abbreviations.",
);

impl Rule for NameReplacements {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<NameReplacementsConfig>::from_value(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run_once(&self, ctx: &LintContext) {
        if !self.check_filenames {
            return;
        }

        let Some(file_name) = ctx.file_path().file_name().and_then(OsStr::to_str) else {
            return;
        };

        // Skip virtual filenames (stdin, etc.)
        if file_name.starts_with('<') && file_name.ends_with('>') {
            return;
        }

        // Get the basename without extension
        let extension = std::path::Path::new(file_name)
            .extension()
            .and_then(OsStr::to_str)
            .map(|e| format!(".{e}"))
            .unwrap_or_default();

        let basename = file_name.strip_suffix(&extension).unwrap_or(file_name);

        // Skip if allowed or ignored
        if self.should_skip(basename) {
            return;
        }

        // Get replacements
        let Some(replacements) = self.get_replacements(basename) else {
            return;
        };

        if replacements.len() == 1 {
            ctx.diagnostic(filename_diagnostic(
                file_name,
                &format!("{}{extension}", replacements[0]),
            ));
        } else {
            let replacements_with_ext: Vec<String> =
                replacements.iter().map(|r| format!("{r}{extension}")).collect();
            ctx.diagnostic(filename_suggestion_diagnostic(file_name, &replacements_with_ext));
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::BindingIdentifier(binding_ident) if self.check_variables => {
                // Skip if this binding is inside an import specifier or TS declaration - those are handled separately
                let parent = ctx.nodes().parent_node(node.id());
                if matches!(
                    parent.kind(),
                    AstKind::ImportSpecifier(_)
                        | AstKind::ImportDefaultSpecifier(_)
                        | AstKind::ImportNamespaceSpecifier(_)
                        | AstKind::TSTypeAliasDeclaration(_)
                        | AstKind::TSInterfaceDeclaration(_)
                        | AstKind::TSEnumDeclaration(_)
                ) {
                    return;
                }
                // Check for require() bindings - treat like default imports
                if let Some(source) = is_require_binding(ctx, node) {
                    let is_internal = is_internal_import(&source);
                    if !self.check_default_and_namespace_imports.should_check(is_internal) {
                        return;
                    }
                }
                // Check if this is a shorthand property in destructuring
                let is_shorthand_property =
                    matches!(parent.kind(), AstKind::BindingProperty(prop) if prop.shorthand);
                if !self.check_shorthand_properties && is_shorthand_property {
                    return;
                }
                self.check_binding(
                    binding_ident.name.as_str(),
                    binding_ident.span,
                    binding_ident.symbol_id(),
                    is_shorthand_property,
                    node,
                    ctx,
                );
            }
            AstKind::ImportSpecifier(specifier) if self.check_variables => {
                // Check if this is a shorthand import (import { foo } vs import { foo as bar })
                let is_shorthand = specifier.imported.name() == specifier.local.name.as_str();
                if is_shorthand {
                    let source = get_import_source(ctx, node);
                    let is_internal = is_internal_import(&source);
                    if !self.check_shorthand_imports.should_check(is_internal) {
                        return;
                    }
                }
                self.check_binding(
                    specifier.local.name.as_str(),
                    specifier.local.span,
                    specifier.local.symbol_id(),
                    false,
                    node,
                    ctx,
                );
            }
            AstKind::ImportDefaultSpecifier(specifier) if self.check_variables => {
                let source = get_import_source(ctx, node);
                let is_internal = is_internal_import(&source);
                if !self.check_default_and_namespace_imports.should_check(is_internal) {
                    return;
                }
                self.check_binding(
                    specifier.local.name.as_str(),
                    specifier.local.span,
                    specifier.local.symbol_id(),
                    false,
                    node,
                    ctx,
                );
            }
            AstKind::ImportNamespaceSpecifier(specifier) if self.check_variables => {
                let source = get_import_source(ctx, node);
                let is_internal = is_internal_import(&source);
                if !self.check_default_and_namespace_imports.should_check(is_internal) {
                    return;
                }
                self.check_binding(
                    specifier.local.name.as_str(),
                    specifier.local.span,
                    specifier.local.symbol_id(),
                    false,
                    node,
                    ctx,
                );
            }
            AstKind::StaticMemberExpression(member_expr) if self.check_properties => {
                // Check property writes: obj.prop = value or obj.prop++
                let parent = ctx.nodes().parent_node(node.id());
                match parent.kind() {
                    AstKind::AssignmentExpression(expr) => {
                        // Check if this member expression is the left side
                        let left_span = expr.left.span();
                        if member_expr.span.start >= left_span.start
                            && member_expr.span.end <= left_span.end
                        {
                            self.check_property(
                                member_expr.property.name.as_str(),
                                member_expr.property.span,
                                ctx,
                            );
                        }
                    }
                    AstKind::UpdateExpression(_) => {
                        // obj.err++ or ++obj.err
                        self.check_property(
                            member_expr.property.name.as_str(),
                            member_expr.property.span,
                            ctx,
                        );
                    }
                    _ => {}
                }
            }
            AstKind::ExportSpecifier(specifier) if self.check_properties => {
                // Check renamed exports: export { foo as err }
                // Skip if local === exported (shorthand, handled as variable)
                let exported_name = specifier.exported.name();
                if specifier.local.name() != exported_name {
                    self.check_property(exported_name.as_str(), specifier.exported.span(), ctx);
                }
            }
            AstKind::ObjectProperty(prop) if self.check_properties => {
                // Skip shorthand properties (handled as variables)
                if prop.shorthand {
                    return;
                }
                // Skip computed properties
                if prop.computed {
                    return;
                }
                if let PropertyKey::StaticIdentifier(ident) = &prop.key {
                    self.check_property(ident.name.as_str(), ident.span, ctx);
                }
            }
            AstKind::PropertyDefinition(prop) if self.check_properties => {
                if prop.computed {
                    return;
                }
                if let PropertyKey::StaticIdentifier(ident) = &prop.key {
                    self.check_property(ident.name.as_str(), ident.span, ctx);
                }
            }
            AstKind::MethodDefinition(method) if self.check_properties => {
                if method.computed {
                    return;
                }
                if let PropertyKey::StaticIdentifier(ident) = &method.key {
                    self.check_property(ident.name.as_str(), ident.span, ctx);
                }
            }
            // TypeScript declarations
            AstKind::TSTypeAliasDeclaration(decl) if self.check_variables => {
                self.check_identifier_name(decl.id.name.as_str(), decl.id.span, ctx);
            }
            AstKind::TSInterfaceDeclaration(decl) if self.check_variables => {
                self.check_identifier_name(decl.id.name.as_str(), decl.id.span, ctx);
            }
            AstKind::TSEnumDeclaration(decl) if self.check_variables => {
                self.check_identifier_name(decl.id.name.as_str(), decl.id.span, ctx);
            }
            AstKind::TSNamespaceDeclaration(decl) if self.check_variables => {
                self.check_identifier_name(decl.id.name.as_str(), decl.id.span, ctx);
            }
            _ => {}
        }
    }
}

/// Check if an import source is "internal" (relative path, not from node_modules).
fn is_internal_import(source: &str) -> bool {
    !source.contains("node_modules") && (source.starts_with('.') || source.starts_with('/'))
}

/// Check if identifier is a restricted identifier in strict mode.
/// `arguments` and `eval` are valid identifiers in non-strict mode but cannot be
/// used as parameter names in strict mode or arrow functions.
fn is_restricted_identifier(s: &str) -> bool {
    matches!(s, "arguments" | "eval")
}

/// Get the import source from an import specifier node.
fn get_import_source(ctx: &LintContext, node: &AstNode) -> String {
    for ancestor in ctx.nodes().ancestors(node.id()) {
        if let AstKind::ImportDeclaration(decl) = ancestor.kind() {
            return decl.source.value.as_str().unwrap_or("").to_string();
        }
    }
    String::new()
}

/// Check if a binding is part of an exported declaration.
/// ESLint skips auto-fix for exports to avoid breaking external consumers.
fn is_exported_binding(ctx: &LintContext, node: &AstNode) -> bool {
    for ancestor in ctx.nodes().ancestors(node.id()) {
        match ancestor.kind() {
            // oxc uses ExportDeclaration for `export const x = ...`
            // ExportNamedDeclaration for `export { x }`
            // ExportDefaultDeclaration for `export default ...`
            AstKind::ExportDeclaration(_)
            | AstKind::ExportNamedDeclaration(_)
            | AstKind::ExportDefaultDeclaration(_) => {
                return true;
            }
            // Stop at function/class boundaries
            AstKind::Function(_) | AstKind::Class(_) | AstKind::ArrowFunctionExpression(_) => {
                return false;
            }
            _ => {}
        }
    }
    false
}

/// Check if any reference to this symbol is a JSX identifier.
/// Renaming would cause mismatched opening/closing tags.
fn has_jsx_references(ctx: &LintContext, symbol_id: SymbolId) -> bool {
    for reference in ctx.semantic().symbol_references(symbol_id) {
        let node = ctx.nodes().get_node(reference.node_id());
        // Check if this reference is used as a JSX element name
        let parent = ctx.nodes().parent_node(node.id());
        if matches!(parent.kind(), AstKind::JSXOpeningElement(_) | AstKind::JSXClosingElement(_)) {
            return true;
        }
    }
    false
}

/// Check if a function parameter has JSDoc @param documentation.
/// ESLint skips auto-fix to avoid leaving stale documentation.
fn has_jsdoc_param(ctx: &LintContext, node: &AstNode) -> bool {
    // Find the enclosing function
    for ancestor in ctx.nodes().ancestors(node.id()) {
        match ancestor.kind() {
            AstKind::Function(func) => {
                // Check for leading block comment with @param
                let comments = ctx.semantic().comments();
                for comment in comments {
                    // Only check block comments that end before the function starts
                    if comment.span.end < func.span.start {
                        let text = comment.content_span().source_text(ctx.source_text());
                        if text.trim_start().starts_with('*') && text.contains("@param") {
                            return true;
                        }
                    }
                }
                return false;
            }
            AstKind::ArrowFunctionExpression(_) => return false,
            _ => {}
        }
    }
    false
}

/// Find the span of a TSTypePredicate parameter name that matches the given name.
/// Type predicates like `val is string` reference the parameter by name, not as a symbol reference.
/// Returns the span if found and name matches.
fn find_type_predicate_param_span(
    ctx: &LintContext,
    node: &AstNode,
    param_name: &str,
) -> Option<Span> {
    // Walk up to find the enclosing function
    for ancestor in ctx.nodes().ancestors(node.id()) {
        let func_return_type = match ancestor.kind() {
            AstKind::Function(func) => func.return_type.as_ref(),
            AstKind::ArrowFunctionExpression(arrow) => arrow.return_type.as_ref(),
            _ => continue,
        };

        // Check if return type is a type predicate
        if let Some(type_annotation) = func_return_type
            && let oxc_ast::ast::TSType::TSTypePredicate(predicate) =
                &type_annotation.type_annotation
            && let TSTypePredicateName::Identifier(ident) = &predicate.parameter_name
            && ident.name == param_name
        {
            return Some(ident.span);
        }
        break;
    }
    None
}

/// Check if a binding is a TypeScript parameter property.
/// `constructor(private err: Error)` should not be auto-fixed.
fn is_ts_parameter_property(ctx: &LintContext, node: &AstNode) -> bool {
    // Walk up to find the FormalParameter and check if it has accessibility modifiers
    for ancestor in ctx.nodes().ancestors(node.id()) {
        if let AstKind::FormalParameter(param) = ancestor.kind() {
            // A FormalParameter with accessibility, readonly, or override is a TS parameter property
            return param.accessibility.is_some() || param.readonly || param.r#override;
        }
        // Stop at function boundaries
        if matches!(ancestor.kind(), AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)) {
            break;
        }
    }
    false
}

/// Check if this is a static require() call that should be treated like a default import.
fn is_require_binding(ctx: &LintContext, node: &AstNode) -> Option<String> {
    // Look for: const err = require('./module')
    let parent = ctx.nodes().parent_node(node.id());
    let AstKind::VariableDeclarator(decl) = parent.kind() else {
        return None;
    };
    let Some(oxc_ast::ast::Expression::CallExpression(call)) = &decl.init else {
        return None;
    };
    let oxc_ast::ast::Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if callee.name == "require"
        && call.arguments.len() == 1
        && let oxc_ast::ast::Argument::StringLiteral(lit) = &call.arguments[0]
    {
        return lit.value.as_str().map(str::to_string);
    }
    None
}

impl NameReplacements {
    /// Check a binding identifier and report if it should be renamed.
    /// `is_shorthand_property` indicates if this is a shorthand property in destructuring,
    /// which requires special fix handling: `{ err }` -> `{ err: error }`.
    /// `node` is the AstNode for the binding, used to check context for fix safety.
    fn check_binding(
        &self,
        name: &str,
        span: Span,
        symbol_id: SymbolId,
        is_shorthand_property: bool,
        node: &AstNode,
        ctx: &LintContext,
    ) {
        if self.should_skip(name) {
            return;
        }

        let Some(replacements) = self.get_replacements(name) else {
            return;
        };

        // Determine if auto-fix is safe
        let can_fix = !is_exported_binding(ctx, node)
            && !has_jsx_references(ctx, symbol_id)
            && !has_jsdoc_param(ctx, node)
            && !is_ts_parameter_property(ctx, node);

        let mut references = ctx.semantic().symbol_references(symbol_id).peekable();

        if replacements.len() == 1 {
            let fixed_name = Self::get_fixed_name(ctx, replacements[0].as_str(), symbol_id);

            if can_fix {
                ctx.diagnostic_with_fix(
                    variable_diagnostic(name, fixed_name.as_str(), span),
                    |fixer| {
                        // For shorthand properties in destructuring, use `key: value` syntax
                        // to preserve the property access: `{ err }` -> `{ err: error }`
                        let binding_fix = if is_shorthand_property {
                            fixer.replace(span, format!("{name}: {fixed_name}"))
                        } else {
                            fixer.replace(span, fixed_name.clone())
                        };

                        if references.peek().is_none() {
                            return binding_fix;
                        }

                        let fixer = fixer.for_multifix();
                        let capacity =
                            ctx.scoping().get_resolved_reference_ids(symbol_id).len() + 1;
                        let mut fixes = fixer.new_fix_with_capacity(capacity);

                        fixes.push(binding_fix);
                        for reference in references {
                            let ref_node = ctx.nodes().get_node(reference.node_id()).kind();
                            let Some(id) = ref_node.as_identifier_reference() else {
                                continue;
                            };
                            fixes.push(fixer.replace(id.span, fixed_name.clone()));
                        }

                        // Also fix type predicate parameter name if present
                        if let Some(predicate_span) =
                            find_type_predicate_param_span(ctx, node, name)
                        {
                            fixes.push(fixer.replace(predicate_span, fixed_name.clone()));
                        }

                        fixes.with_message(format!("Rename `{name}` to `{fixed_name}`"))
                    },
                );
            } else {
                // Report without fix when auto-fix is unsafe
                ctx.diagnostic(variable_diagnostic(name, fixed_name.as_str(), span));
            }
        } else if can_fix {
            // Multiple replacement options - provide suggestions
            let references: Vec<_> = ctx.semantic().symbol_references(symbol_id).collect();
            let fixer = RuleFixer::new(FixKind::Suggestion, ctx).for_multifix();
            let capacity = references.len() + 1;

            let suggestions: Vec<_> = replacements
                .iter()
                .map(|replacement| {
                    let fixed_name = Self::get_fixed_name(ctx, replacement.as_str(), symbol_id);
                    let mut fixes = fixer.new_fix_with_capacity(capacity);

                    // Fix the binding
                    let binding_fix = if is_shorthand_property {
                        fixer.replace(span, format!("{name}: {fixed_name}"))
                    } else {
                        fixer.replace(span, fixed_name.clone())
                    };
                    fixes.push(binding_fix);

                    // Fix all references
                    for reference in &references {
                        let ref_node = ctx.nodes().get_node(reference.node_id()).kind();
                        if let Some(id) = ref_node.as_identifier_reference() {
                            fixes.push(fixer.replace(id.span, fixed_name.clone()));
                        }
                    }

                    // Also fix type predicate parameter name if present
                    if let Some(predicate_span) = find_type_predicate_param_span(ctx, node, name) {
                        fixes.push(fixer.replace(predicate_span, fixed_name.clone()));
                    }

                    fixes.with_message(format!("Rename to `{fixed_name}`"))
                })
                .collect();

            ctx.diagnostic_with_suggestions(
                variable_suggestion_diagnostic(name, &replacements, span),
                suggestions,
            );
        } else {
            // Multiple options but can't fix - just warn
            ctx.diagnostic(variable_suggestion_diagnostic(name, &replacements, span));
        }
    }

    /// Check a property identifier (no auto-fix for properties).
    fn check_property(&self, name: &str, span: Span, ctx: &LintContext) {
        if self.should_skip(name) {
            return;
        }

        let Some(replacements) = self.get_replacements(name) else {
            return;
        };

        if replacements.len() == 1 {
            ctx.diagnostic(property_diagnostic(name, replacements[0].as_str(), span));
        } else {
            ctx.diagnostic(property_suggestion_diagnostic(name, &replacements, span));
        }
    }

    /// Check a simple identifier name (TypeScript declarations, no auto-fix).
    fn check_identifier_name(&self, name: &str, span: Span, ctx: &LintContext) {
        if self.should_skip(name) {
            return;
        }

        let Some(replacements) = self.get_replacements(name) else {
            return;
        };

        if replacements.len() == 1 {
            ctx.diagnostic(variable_diagnostic(name, replacements[0].as_str(), span));
        } else {
            ctx.diagnostic(variable_suggestion_diagnostic(name, &replacements, span));
        }
    }

    /// Check if a name should be skipped (allowed or ignored).
    fn should_skip(&self, name: &str) -> bool {
        // Skip UPPER_CASE constants
        if name.chars().all(|c| c.is_uppercase() || c == '_')
            && name.chars().any(char::is_alphabetic)
        {
            return true;
        }

        // Check user allow list
        if self.allow_list.get(name).copied().unwrap_or(false) {
            return true;
        }

        // Check default allow list
        if self.extend_default_allow_list && DEFAULT_ALLOW_LIST.contains(&name) {
            return true;
        }

        // Check ignore patterns
        if self.ignore.iter().any(|pattern| pattern.is_match(name)) {
            return true;
        }

        // Check default ignore patterns
        for pattern in DEFAULT_IGNORE {
            if name.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Get replacement suggestions for a name.
    /// Returns `None` if no replacements apply, `Some(vec)` with suggestions otherwise.
    fn get_replacements(&self, name: &str) -> Option<Vec<CompactStr>> {
        let lower_name = name.cow_to_lowercase();

        // Check if user explicitly disabled this replacement
        if matches!(
            self.replacements.get(lower_name.as_ref()),
            Some(ReplacementValue::Disabled(false))
        ) {
            return None;
        }

        // Check user-defined replacements first (if they provide a map)
        if let Some(ReplacementValue::Map(map)) = self.replacements.get(lower_name.as_ref()) {
            let enabled: Vec<CompactStr> = map
                .iter()
                .filter(|(_, v)| **v)
                .filter(|(k, _)| !is_reserved_keyword(k.as_str()))
                .filter(|(k, _)| !is_restricted_identifier(k.as_str()))
                .filter(|(k, _)| is_identifier_name(k.as_str()))
                .map(|(k, _)| CompactStr::from(k.as_str()))
                .collect();
            if !enabled.is_empty() {
                return Some(enabled);
            }
        }

        // Check default replacements
        if self.extend_default_replacements {
            if let Some(&replacements) = DEFAULT_REPLACEMENTS.get(lower_name.as_ref()) {
                let filtered: Vec<CompactStr> = replacements
                    .iter()
                    .filter(|&s| !is_reserved_keyword(s))
                    .filter(|&s| !is_restricted_identifier(s))
                    .map(|&s| CompactStr::from(s))
                    .collect();
                if !filtered.is_empty() {
                    return Some(filtered);
                }
                return None;
            }

            // Try splitting compound names (e.g., "errMsg" -> ["err", "Msg"])
            return get_compound_replacements(name);
        }

        None
    }

    /// Get a fixed name that doesn't conflict with existing bindings.
    fn get_fixed_name(ctx: &LintContext, preferred_name: &str, symbol_id: SymbolId) -> CompactStr {
        let scoping = ctx.scoping();
        let scope_id = scoping.symbol_scope_id(symbol_id);
        let mut name = preferred_name.to_string();

        while scoping.find_binding(scope_id, name.as_str().into()).is_some()
            || scoping.get_resolved_references(symbol_id).any(|reference| {
                scoping.find_binding(reference.scope_id(), name.as_str().into()).is_some()
            })
            || scoping.root_unresolved_references().get(name.as_str()).is_some_and(|references| {
                references.iter().any(|&reference_id| {
                    let reference_scope_id = scoping.get_reference(reference_id).scope_id();
                    reference_scope_id == scope_id
                        || scoping.scope_is_descendant_of(reference_scope_id, scope_id)
                })
            })
        {
            name.push('_');
        }

        CompactStr::from(name)
    }
}

fn upper_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Handle compound names like "errMsg" -> "errorMessage" or "err-handler" -> "error-handler".
fn get_compound_replacements(name: &str) -> Option<Vec<CompactStr>> {
    let words = split_on_word_boundaries(name);
    if words.len() <= 1 {
        return None;
    }

    // Collect replacements for each segment.
    // Non-alphabetic segments (separators, digits) are kept as-is.
    // Alphabetic words are looked up in replacements.
    let mut word_replacements: Vec<(&str, &[&'static str], bool)> = Vec::with_capacity(words.len());
    let mut any_needs_replacement = false;

    for word in &words {
        if word.chars().all(char::is_alphabetic) {
            let lower_word = word.cow_to_lowercase();
            if let Some(&replacements) = DEFAULT_REPLACEMENTS.get(lower_word.as_ref()) {
                word_replacements.push((word.as_str(), replacements, true));
                any_needs_replacement = true;
            } else {
                // Alphabetic word with no replacement - keep as-is
                word_replacements.push((word.as_str(), &[], true));
            }
        } else {
            // Non-alphabetic segment (separator, digit, etc.) - keep as-is
            word_replacements.push((word.as_str(), &[], false));
        }
    }

    if !any_needs_replacement {
        return None;
    }

    // Build all combinations of replacements
    // Track if the name uses separators (kebab-case, snake_case) vs camelCase
    // Only apply upper_first for camelCase names (no separators between words)
    let has_separators = word_replacements.iter().any(|(_, _, is_alpha)| !is_alpha);

    let mut results: Vec<String> = vec![String::new()];
    let mut is_first_alphabetic_word = true;

    for (original_word, replacements, is_alphabetic) in &word_replacements {
        if !is_alphabetic {
            // Non-alphabetic segment - append as-is to all results
            for result in &mut results {
                result.push_str(original_word);
            }
            continue;
        }

        // Only apply camelCase casing if the name doesn't have separators
        let needs_upper = !has_separators && !is_first_alphabetic_word;
        is_first_alphabetic_word = false;

        if replacements.is_empty() {
            // No replacement for this word - append original (with proper casing)
            for result in &mut results {
                if needs_upper {
                    result.push_str(&upper_first(original_word));
                } else {
                    result.push_str(original_word);
                }
            }
        } else if replacements.len() == 1 {
            // Single replacement
            for result in &mut results {
                let replacement = if needs_upper {
                    upper_first(replacements[0])
                } else {
                    replacements[0].to_string()
                };
                result.push_str(&replacement);
            }
        } else {
            // Multiple replacements - multiply results
            let mut new_results = Vec::with_capacity(results.len() * replacements.len());
            for result in &results {
                for &replacement in *replacements {
                    let mut new_result = result.clone();
                    let cased = if needs_upper {
                        upper_first(replacement)
                    } else {
                        replacement.to_string()
                    };
                    new_result.push_str(&cased);
                    new_results.push(new_result);
                }
            }
            results = new_results;
        }
    }

    // Limit to reasonable number of suggestions
    results.truncate(4);

    Some(results.into_iter().map(CompactStr::from).collect())
}

/// Split a name on word boundaries matching ESLint's behavior.
/// Uses the equivalent of `/(?=\P{Lowercase_Letter})|(?<=\P{Letter})/u`:
/// - Split before any non-lowercase letter (uppercase, digit, symbol)
/// - Split after any non-letter (hyphen, underscore, digit, symbol)
fn split_on_word_boundaries(name: &str) -> Vec<String> {
    if name.is_empty() {
        return vec![];
    }

    let chars: Vec<char> = name.chars().collect();
    let mut words = Vec::new();
    let mut start = 0;

    for i in 1..chars.len() {
        let prev = chars[i - 1];
        let curr = chars[i];

        // Split before non-lowercase letter
        let split_before = !curr.is_lowercase();

        // Split after non-letter
        let split_after = !prev.is_alphabetic();

        if split_before || split_after {
            let word: String = chars[start..i].iter().collect();
            if !word.is_empty() {
                words.push(word);
            }
            start = i;
        }
    }

    // Don't forget the last segment
    let word: String = chars[start..].iter().collect();
    if !word.is_empty() {
        words.push(word);
    }

    words
}

// Default replacements: maps abbreviations to full names.
// The first entry is the preferred replacement for auto-fix.
static DEFAULT_REPLACEMENTS: phf::Map<&'static str, &'static [&'static str]> = phf::phf_map! {
    "acc" => &["accumulator"],
    "application" => &["app"],
    "applications" => &["apps"],
    "arg" => &["argument"],
    "args" => &["arguments"],
    "arr" => &["array"],
    "attr" => &["attribute"],
    "attrs" => &["attributes"],
    "btn" => &["button"],
    "buf" => &["buffer"],
    "cb" => &["callback"],
    "cfg" => &["config"],
    "cmd" => &["command"],
    "conf" => &["config"],
    "configuration" => &["config"],
    "ctx" => &["context"],
    "cur" => &["current"],
    "curr" => &["current"],
    "db" => &["database"],
    "decl" => &["declaration"],
    "decls" => &["declarations"],
    "def" => &["defer", "deferred", "define", "definition"],
    "dep" => &["dependency"],
    "deps" => &["dependencies"],
    "dest" => &["destination"],
    "dev" => &["development"],
    "dir" => &["direction", "directory"],
    "dirs" => &["directories"],
    "dist" => &["distribution"],
    "doc" => &["document"],
    "docs" => &["documentation", "documents"],
    "dst" => &["daylightSavingTime", "destination", "distribution"],
    "e" => &["error", "event"],
    "el" => &["element"],
    "elem" => &["element"],
    "elems" => &["elements"],
    "env" => &["environment"],
    "envs" => &["environments"],
    "err" => &["error"],
    "ev" => &["event"],
    "evt" => &["event"],
    "expr" => &["expression"],
    "exprs" => &["expressions"],
    "ext" => &["extension"],
    "exts" => &["extensions"],
    "fn" => &["function"],
    "func" => &["function"],
    "i" => &["index"],
    "ident" => &["identifier"],
    "idents" => &["identifiers"],
    "idx" => &["index"],
    "j" => &["index"],
    "len" => &["length"],
    "lib" => &["library"],
    "mod" => &["module"],
    "msg" => &["message"],
    "num" => &["number"],
    "obj" => &["object"],
    "opts" => &["options"],
    "param" => &["parameter"],
    "params" => &["parameters"],
    "perf" => &["performance"],
    "pkg" => &["package"],
    "prev" => &["previous"],
    "prod" => &["production"],
    "prop" => &["property"],
    "props" => &["properties"],
    "proto" => &["prototype"],
    "ref" => &["reference"],
    "refs" => &["references"],
    "rel" => &["related", "relationship", "relative"],
    "repo" => &["repository"],
    "req" => &["request"],
    "res" => &["resource", "response", "result"],
    "ret" => &["returnValue"],
    "retval" => &["returnValue"],
    "sep" => &["separator"],
    "src" => &["source"],
    "stdDev" => &["standardDeviation"],
    "stmt" => &["statement"],
    "stmts" => &["statements"],
    "str" => &["string"],
    "tbl" => &["table"],
    "temp" => &["temporary"],
    "tit" => &["title"],
    "tmp" => &["temporary"],
    "util" => &["utility"],
    "utils" => &["utilities"],
    "val" => &["value"],
    "var" => &["variable"],
    "vars" => &["variables"],
    "ver" => &["version"],
};

// Default allow list: well-known names that should not be checked.
static DEFAULT_ALLOW_LIST: &[&str] = &[
    "defaultProps",
    "devDependencies",
    "EmberENV",
    "getDerivedStateFromProps",
    "getInitialProps",
    "getServerSideProps",
    "getStaticProps",
    "ImportMetaEnv",
    "iOS",
    "ProcessEnv",
    "propTypes",
    "setupFilesAfterEnv",
];

// Default patterns to ignore.
static DEFAULT_IGNORE: &[&str] = &["i18n", "l10n", "a11y", "e2e", "jQuery", "vite-env"];

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Full names are allowed
        ("let x", None),
        ("let error", None),
        ("const {error} = {}", None),
        ("({error: 1})", None),
        ("({[err]: 1})", None),
        ("error => {}", None),
        ("({error}) => ({error})", None),
        ("httpError => {}", None),
        ("({httpError}) => ({httpError})", None),
        ("(function (error) {})", None),
        ("(function ({error}) {})", None),
        ("function f(x) {}", None),
        ("function f({x: y}) {}", None),
        ("(class {})", None),
        ("(class C {})", None),
        ("class C {}", None),
        // Internationalization and accessibility patterns
        ("const i18n = new I18n({ locales: [\"en\", \"fr\"] })", None),
        ("const i18nData = {}", None),
        ("const l10n = new L10n()", None),
        ("const e2e = 1", None),
        ("const e2eTests = []", None),
        ("const a11y = {}", None),
        // Allow list
        ("const iOS = true", None),
        ("const jQueryEvent = true", None),
        ("const propTypes = {}", None),
        ("const devDependencies = {}", None),
        // UPPER_CASE constants
        ("const ERR = 'error';", None),
        ("const CB_TIMEOUT = 1000;", None),
        ("let E", None),
        // Class methods are allowed (properties disabled by default)
        ("(class { error() {} })", None),
        ("(class { [err]() {} })", None),
        ("class C { x() {} }", None),
        // Property access (properties disabled by default)
        ("foo.error", None),
        ("foo.x", None),
        ("foo.err", None),
        // Disabled in config
        (
            "const err = new Error();",
            Some(serde_json::json!([{ "replacements": { "err": false } }])),
        ),
        // Custom allow list
        ("const err = new Error();", Some(serde_json::json!([{ "allowList": { "err": true } }]))),
        // Check disabled
        ("const err = new Error();", Some(serde_json::json!([{ "checkVariables": false }]))),
        // Ignore pattern
        ("const _err = new Error();", Some(serde_json::json!([{ "ignore": ["^_"] }]))),
        // External imports (default: checkDefaultAndNamespaceImports = "internal")
        ("import err from 'lodash'", None),
        ("import * as err from 'lodash'", None),
        ("import { err } from 'lodash'", None),
        // checkShorthandImports: false skips shorthand imports
        (
            "import { err } from './module'",
            Some(serde_json::json!([{ "checkShorthandImports": false }])),
        ),
        // checkDefaultAndNamespaceImports: false skips default/namespace imports
        (
            "import err from './module'",
            Some(serde_json::json!([{ "checkDefaultAndNamespaceImports": false }])),
        ),
        // Shorthand properties in destructuring (disabled by default)
        ("const { err } = obj", None),
        // extendDefaultAllowList: false with custom allowList
        (
            "const propTypes = {}",
            Some(
                serde_json::json!([{ "extendDefaultAllowList": false, "allowList": { "propTypes": true } }]),
            ),
        ),
        // External require() (default: checkDefaultAndNamespaceImports = "internal")
        ("const err = require('lodash')", None),
        // require() with checkDefaultAndNamespaceImports: false
        (
            "const err = require('./module')",
            Some(serde_json::json!([{ "checkDefaultAndNamespaceImports": false }])),
        ),
        // Export specifiers (properties disabled by default)
        ("export { foo as err }", None),
        // Reserved word replacements are skipped (fn -> function is invalid)
        ("const fn = () => {}", None),
        ("function fn() {}", None),
        // Catch clause with full name
        ("try {} catch (error) {}", None),
        // For-of/for-in with full names
        ("for (const element of array) {}", None),
        ("for (const key in object) {}", None),
        // Arrow functions with full names
        ("const handler = (error) => error", None),
        ("const handler = (callback) => callback()", None),
        ("array.map(element => element)", None),
        ("array.filter(item => item)", None),
        // Async/generator functions
        ("async function fetchData(error) {}", None),
        ("function* generateItems(index) {}", None),
        // Rest parameters with full names
        ("function f(...arguments) {}", None),
        // args skipped since arguments is a restricted identifier
        ("function f(...args) {}", None),
        ("const args = [];", None),
        // Default parameters with full names
        ("function f(error = null) {}", None),
        // Renamed imports (local name is full word)
        ("import { err as error } from './module'", None),
        // Object method shorthand
        ("({ error() {} })", None),
        // Class with full property names
        ("class C { error = 1 }", None),
        // Nested destructuring with full names
        ("const { outer: { error } } = obj", None),
        ("const [error] = arr", None),
        // Computed property access (not checked by default)
        ("obj[err] = 1", None),
        // Function declaration with full names
        ("function handleError(error) { return error; }", None),
        // Multiple declarations with full names
        ("let error, callback, message;", None),
        // Object pattern in parameters
        ("function f({ error, callback }) {}", None),
        // Array pattern in parameters
        ("function f([error, callback]) {}", None),
        // Re-export (properties disabled by default)
        ("export { err } from './module'", None),
        // Class static members
        ("class C { static error = 1 }", None),
        // Object spread (uses full name)
        ("const data = { ...error }", None),
        // Template literal (not a variable)
        ("`${error}`", None),
        // Property shorthand with full name
        ("const error = 1; ({ error })", None),
        // === More abbreviation variations (untested defaults) ===
        ("const standardDeviation = 1", None),
        ("const performanceBudget = {}", None),
        ("const repository = 'github.com'", None),
        ("const prototype = {}", None),
        ("const identifier = 'abc'", None),
        ("const identifiers = []", None),
        ("const statement = 'SELECT *'", None),
        ("const statements = []", None),
        ("const declaration = {}", None),
        ("const declarations = []", None),
        ("const expression = {}", None),
        ("const expressions = []", None),
        ("const command = 'npm'", None),
        ("const dependency = {}", None),
        ("const dependencies = []", None),
        // === Reverse direction (long → short) should pass ===
        ("const app = {}; const apps = []", None),
        ("const config = {}", None),
        // === Class declarations with full names ===
        ("class Callback {}", None),
        ("class Error {}", None),
        ("class ErrorHandler {}", None),
        ("abstract class Middleware { abstract handle(context: any): void; }", None),
        // === TypeScript type predicates with full names ===
        (
            "function isString(value: unknown): value is string { return typeof value === 'string'; }",
            None,
        ),
        ("function assertString(value: unknown): asserts value is string {}", None),
        // === Scoping - different scopes are independent ===
        ("{ let errorCallback = 1; } { let errorCallback = 2; }", None),
        ("function outer() { const error = 1; function inner() { const error = 2; } }", None),
        // === Unicode escapes - not identifier context ===
        ("const s = '\\u0065rr'", None),
        // === __proto__ is flagged (proto -> prototype) ===
        // ("const __proto__ = {}", None), // Actually flagged
        // === Single letter T for generics ===
        ("type Identity<T> = T", None),
        // === Reserved words as replacement targets should pass ===
        ("const fn = () => {}", None),
        ("function fn() {}", None),
        // === Invalid replacement targets NOT filtered (shown in suggestions) ===
        // Note: ESLint filters these but we currently don't
        // === Closure captures with full names ===
        ("function outer() { const error = 1; return () => error; }", None),
        // === Nested function shadows with full names ===
        ("const error = 1; function f(error: Error) { return error; }", None),
    ];

    let fail = vec![
        // Basic abbreviations with single replacement
        ("const err = new Error();", None),
        ("const cb = () => {};", None),
        ("const btn = document.querySelector('button');", None),
        ("const msg = 'hello';", None),
        ("const str = 'text';", None),
        ("const num = 42;", None),
        ("const obj = {};", None),
        ("const arr = [];", None),
        ("const idx = 0;", None),
        ("const ctx = {};", None),
        ("function process(buf) { return buf; }", None),
        ("const cfg = {};", None),
        ("const attr = '';", None),
        ("const len = 10;", None),
        ("const val = 5;", None),
        ("const ref = {};", None),
        ("const src = '';", None),
        // Multiple replacement options (no auto-fix)
        ("const e = new Error();", None),
        ("const res = fetch('/api');", None),
        ("const def = {};", None),
        ("const dir = './';", None),
        ("const rel = {};", None),
        // Internal imports should be checked
        ("import err from './module'", None),
        ("import * as err from './module'", None),
        ("import { err } from './module'", None),
        // TypeScript declarations
        ("type err = string", None),
        ("interface err {}", None),
        ("enum err { A }", None),
        // TypeScript namespace/module declarations
        ("namespace Err {}", None),
        ("module Msg { export const x = 1; }", None),
        // Properties when enabled
        ("const obj = { err: 1 }", Some(serde_json::json!([{ "checkProperties": true }]))),
        ("class C { err = 1 }", Some(serde_json::json!([{ "checkProperties": true }]))),
        // Compound names
        ("const errMsg = 'oops';", None),
        ("const btnClick = () => {};", None),
        // Compound names with multiple replacement options
        ("const eCbOpts = {};", None),
        // User-defined replacements
        (
            "const custom = 1;",
            Some(serde_json::json!([{ "replacements": { "custom": { "replacement": true } } }])),
        ),
        // checkDefaultAndNamespaceImports: true checks external imports
        (
            "import err from 'lodash'",
            Some(serde_json::json!([{ "checkDefaultAndNamespaceImports": true }])),
        ),
        // checkShorthandImports: true checks external shorthand imports
        (
            "import { err } from 'lodash'",
            Some(serde_json::json!([{ "checkShorthandImports": true }])),
        ),
        // checkShorthandProperties: true checks destructuring shorthand
        ("const { err } = obj", Some(serde_json::json!([{ "checkShorthandProperties": true }]))),
        // extendDefaultAllowList: false removes default allow list
        (
            "const propTypes = {}",
            Some(
                serde_json::json!([{ "extendDefaultAllowList": false, "replacements": { "propTypes": { "propertyTypes": true } } }]),
            ),
        ),
        // extendDefaultReplacements: false with custom replacements only
        (
            "const foo = 1",
            Some(
                serde_json::json!([{ "extendDefaultReplacements": false, "replacements": { "foo": { "foobar": true } } }]),
            ),
        ),
        // Internal require() should be checked
        ("const err = require('./module')", None),
        // Export specifiers with checkProperties
        ("export { foo as err }", Some(serde_json::json!([{ "checkProperties": true }]))),
        // UpdateExpression with checkProperties
        ("obj.err++", Some(serde_json::json!([{ "checkProperties": true }]))),
        ("++obj.err", Some(serde_json::json!([{ "checkProperties": true }]))),
        // Exported variable (should warn but not fix)
        ("export const err = new Error();", None),
        // TS parameter property (should warn but not fix)
        ("class C { constructor(private err: Error) {} }", None),
        // Catch clause with abbreviation
        ("try {} catch (err) {}", None),
        ("try {} catch (e) {}", None),
        // For loop variables
        ("for (let i = 0; i < 10; i++) {}", None),
        ("for (let idx = 0; idx < arr.length; idx++) {}", None),
        // For-of/for-in with abbreviations
        ("for (const el of elements) {}", None),
        ("for (const attr in obj) {}", None),
        // Arrow function parameters
        ("const handler = (err) => err", None),
        ("const handler = (cb) => cb()", None),
        ("array.map(el => el)", None),
        ("array.forEach(elem => console.log(elem))", None),
        ("array.filter(val => val > 0)", None),
        ("array.reduce((acc, cur) => acc + cur, 0)", None),
        // Async/generator functions with abbreviations
        ("async function fetch(err) {}", None),
        ("function* gen(idx) {}", None),
        // Rest parameters with abbreviations - args skipped since arguments is a restricted identifier
        // ("function f(...args) {}", None), // moved to pass
        // Default parameters with abbreviations
        ("function f(err = null) {}", None),
        ("const fn = (cb = () => {}) => cb()", None),
        // Function expression with abbreviation
        ("const handler = function(err) { return err; }", None),
        // Class methods when checkProperties enabled
        ("class C { err() {} }", Some(serde_json::json!([{ "checkProperties": true }]))),
        ("class C { static err() {} }", Some(serde_json::json!([{ "checkProperties": true }]))),
        // Object methods when checkProperties enabled
        ("({ err() {} })", Some(serde_json::json!([{ "checkProperties": true }]))),
        // Nested destructuring
        (
            "const { outer: { err } } = obj",
            Some(serde_json::json!([{ "checkShorthandProperties": true }])),
        ),
        ("const [err] = arr", None),
        ("const [, err] = arr", None),
        // Multiple declarations
        ("let err, cb, msg;", None),
        ("const err = 1, cb = 2;", None),
        // Object pattern in parameters
        (
            "function f({ err, cb }) {}",
            Some(serde_json::json!([{ "checkShorthandProperties": true }])),
        ),
        // Array pattern in parameters
        ("function f([err, cb]) {}", None),
        // Named function expression
        ("const x = function err() {}", None),
        // Property assignment when checkProperties enabled
        ("obj.err = 1", Some(serde_json::json!([{ "checkProperties": true }]))),
        ("this.err = value", Some(serde_json::json!([{ "checkProperties": true }]))),
        // Compound assignment
        ("obj.err += 1", Some(serde_json::json!([{ "checkProperties": true }]))),
        // More compound names
        ("const cbFn = () => {};", None),
        ("const errHandler = () => {};", None),
        ("const msgText = '';", None),
        ("const strValue = '';", None),
        ("const numCount = 0;", None),
        ("const objData = {};", None),
        ("const arrItems = [];", None),
        // Deeply nested compound names
        ("const errCbOpts = {};", None),
        ("const btnClickHandler = () => {};", None),
        // Class field with checkProperties
        ("class C { static err = 1 }", Some(serde_json::json!([{ "checkProperties": true }]))),
        // Getter/setter when checkProperties enabled
        ("class C { get err() {} }", Some(serde_json::json!([{ "checkProperties": true }]))),
        ("class C { set err(v) {} }", Some(serde_json::json!([{ "checkProperties": true }]))),
        // Object getter/setter when checkProperties enabled
        ("({ get err() {} })", Some(serde_json::json!([{ "checkProperties": true }]))),
        ("({ set err(v) {} })", Some(serde_json::json!([{ "checkProperties": true }]))),
        // Renamed export when checkProperties enabled
        ("export { error as err }", Some(serde_json::json!([{ "checkProperties": true }]))),
        // Import with renamed to abbreviation
        ("import { error as err } from './module'", None),
        // Function with both abbreviated param and body var
        ("function f(err) { const cb = () => {}; return cb(err); }", None),
        // More TypeScript
        ("type ErrHandler = (err: Error) => void", None),
        ("interface IErr { msg: string }", None),
        // Generic type parameter (common abbreviation)
        ("function identity<T>(arg: T): T { return arg; }", None),
        // Labels are not checked (but binding in for-of is)
        ("err: for (const el of arr) { break err; }", None),
        // === More abbreviation variations (untested defaults) ===
        ("const stdDev = 1", None),
        ("const perfBudget = {}", None),
        ("const repo = 'github.com'", None),
        ("const repoUrl = 'https://github.com'", None),
        ("const proto = {}", None),
        ("const ident = 'abc'", None),
        ("const idents = []", None),
        ("const stmt = 'SELECT *'", None),
        ("const stmts = []", None),
        ("const decl = {}", None),
        ("const decls = []", None),
        ("const expr = {}", None),
        ("const exprs = []", None),
        ("const cmd = 'npm'", None),
        ("const dep = {}", None),
        ("const deps = []", None),
        // === Reverse direction tests (long → short is also flagged) ===
        ("const application = {}", None),
        ("const applications = []", None),
        ("const configuration = {}", None),
        ("const webApplication = {}", None),
        ("const buildConfiguration = {}", None),
        // === Class declarations with abbreviations ===
        ("class cb {}", None),
        ("class Err {}", None),
        ("class Cb {}", None),
        ("class Res {}", None),
        ("abstract class Ctx { abstract handle(ctx: any): void; }", None),
        // === TypeScript type predicates with abbreviations ===
        (
            "function isString(val: unknown): val is string { return typeof val === 'string'; }",
            None,
        ),
        ("function check(e: unknown): e is Error { return e instanceof Error; }", None),
        // === Property capitalization variations ===
        ("({Err: 1})", Some(serde_json::json!([{ "checkProperties": true }]))),
        ("({Res: 1})", Some(serde_json::json!([{ "checkProperties": true }]))),
        // === Scoping conflict tests - abbreviations in different scopes ===
        ("{ let errCb = 1; } { let errorCb = 2; }", None),
        ("function outer(err: Error) { const cb = () => {}; }", None),
        // === Combinatorial/stress tests ===
        ("const errCbOptsObj = {}", None),
        ("const btnClickHandlerFn = () => {}", None),
        // === More config option combinations ===
        (
            "const propTypes = 1",
            Some(
                serde_json::json!([{ "extendDefaultAllowList": false, "replacements": { "prop": { "property": true } } }]),
            ),
        ),
        (
            "const bar = 1",
            Some(
                serde_json::json!([{ "extendDefaultReplacements": false, "replacements": { "bar": { "barFull": true } } }]),
            ),
        ),
        // Custom replacement test
        (
            "const xyz = 1",
            Some(
                serde_json::json!([{ "extendDefaultReplacements": false, "replacements": { "xyz": { "xyzFull": true } } }]),
            ),
        ),
        // === Remaining patterns ===
        ("const [prop = 1] = []", None),
        ("const {prop = 1} = {}", Some(serde_json::json!([{ "checkShorthandProperties": true }]))),
        ("(extraParams?: string) => {}", None),
        ("class M { handle = (ctx: any) => ctx.body; }", None),
        // === More TypeScript ===
        ("type Cb = () => void", None),
        ("interface ErrMsg { code: number }", None),
        ("enum Btn { Primary, Secondary }", None),
        // === Destructuring with rename ===
        ("const { foo: err } = obj", None),
        // === Object method shorthand with checkProperties ===
        ("const obj = { err() {} }", Some(serde_json::json!([{ "checkProperties": true }]))),
        // === Shorthand import from internal module ===
        ("import { cb } from './handlers'", None),
        // === Named export with abbreviation ===
        ("export const cb = () => {}", None),
        // Re-export specifiers checked with checkProperties
        (
            "export { foo as cb } from './module'",
            Some(serde_json::json!([{ "checkProperties": true }])),
        ),
        // === More scoping conflict tests ===
        // Closure captures with abbreviations
        ("function outer() { const err = 1; return () => err; }", None),
        // Nested function shadows with abbreviations
        ("const err = 1; function f(err: Error) { return err; }", None),
        // Same abbreviation in sibling scopes
        ("{ const cb = 1; } { const cb = 2; }", None),
        // === Unicode in dynamic require (edge case) ===
        ("const err = require(`./\\u0065rr`)", None),
    ];

    let fix = vec![
        // Single replacement fixes
        ("const err = new Error();", "const error = new Error();", None),
        ("const cb = () => {};", "const callback = () => {};", None),
        (
            "const btn = document.querySelector('button');",
            "const button = document.querySelector('button');",
            None,
        ),
        ("const msg = 'hello';", "const message = 'hello';", None),
        ("const str = 'text';", "const string = 'text';", None),
        ("const num = 42;", "const number = 42;", None),
        ("const obj = {};", "const object = {};", None),
        ("const arr = [];", "const array = [];", None),
        ("const idx = 0;", "const index = 0;", None),
        ("const ctx = {};", "const context = {};", None),
        (
            "function process(buf) { return buf; }",
            "function process(buffer) { return buffer; }",
            None,
        ),
        // Reference renaming
        (
            "const err = new Error(); console.log(err);",
            "const error = new Error(); console.log(error);",
            None,
        ),
        (
            "function f(cb) { cb(); return cb; }",
            "function f(callback) { callback(); return callback; }",
            None,
        ),
        // Conflict avoidance with underscore suffix
        (
            "const error = 'outer'; const err = new Error();",
            "const error = 'outer'; const error_ = new Error();",
            None,
        ),
        (
            "const error = 1; const error_ = 2; const err = 3;",
            "const error = 1; const error_ = 2; const error__ = 3;",
            None,
        ),
        // Compound name fixes
        ("const errMsg = 'oops';", "const errorMessage = 'oops';", None),
        ("const btnClick = () => {};", "const buttonClick = () => {};", None),
        // Shorthand property destructuring fix: { err } -> { err: error }
        (
            "const { err } = obj; console.log(err);",
            "const { err: error } = obj; console.log(error);",
            Some(serde_json::json!([{ "checkShorthandProperties": true }])),
        ),
        // Import fix
        (
            "import err from './module'; console.log(err);",
            "import error from './module'; console.log(error);",
            None,
        ),
        // User-defined replacement fix
        (
            "const foo = 1; return foo;",
            "const foobar = 1; return foobar;",
            Some(
                serde_json::json!([{ "extendDefaultReplacements": false, "replacements": { "foo": { "foobar": true } } }]),
            ),
        ),
        // Internal require() fix
        (
            "const err = require('./module'); console.log(err);",
            "const error = require('./module'); console.log(error);",
            None,
        ),
        // Exported variable - NO fix (would break external consumers)
        ("export const err = new Error();", "export const err = new Error();", None),
        // TS parameter property - NO fix (declares both parameter and class field)
        (
            "class C { constructor(private err: Error) {} }",
            "class C { constructor(private err: Error) {} }",
            None,
        ),
        // Catch clause fix
        (
            "try {} catch (err) { console.log(err); }",
            "try {} catch (error) { console.log(error); }",
            None,
        ),
        // For loop variable fix
        (
            "for (let i = 0; i < 10; i++) { console.log(i); }",
            "for (let index = 0; index < 10; index++) { console.log(index); }",
            None,
        ),
        (
            "for (let idx = 0; idx < arr.length; idx++) { console.log(idx); }",
            "for (let index = 0; index < arr.length; index++) { console.log(index); }",
            None,
        ),
        // For-of/for-in fixes
        (
            "for (const el of elements) { console.log(el); }",
            "for (const element of elements) { console.log(element); }",
            None,
        ),
        (
            "for (const attr in obj) { console.log(attr); }",
            "for (const attribute in obj) { console.log(attribute); }",
            None,
        ),
        // Arrow function parameter fixes
        ("const handler = (err) => err", "const handler = (error) => error", None),
        ("const handler = (cb) => cb()", "const handler = (callback) => callback()", None),
        ("array.map(el => el)", "array.map(element => element)", None),
        (
            "array.forEach(elem => console.log(elem))",
            "array.forEach(element => console.log(element))",
            None,
        ),
        // Reduce with accumulator fix (only fixes one at a time)
        (
            "array.reduce((acc, cur) => acc + cur, 0)",
            "array.reduce((accumulator, cur) => accumulator + cur, 0)",
            None,
        ),
        // Rest parameters fix - args skipped since arguments is a restricted identifier
        // (
        //     "function f(...args) { return args; }",
        //     "function f(...arguments) { return arguments; }",
        //     None,
        // ),
        // Default parameters fix
        (
            "function f(err = null) { return err; }",
            "function f(error = null) { return error; }",
            None,
        ),
        // Function expression fix
        (
            "const handler = function(err) { return err; }",
            "const handler = function(error) { return error; }",
            None,
        ),
        // Array destructuring fix
        ("const [err] = arr; console.log(err);", "const [error] = arr; console.log(error);", None),
        (
            "const [, err] = arr; console.log(err);",
            "const [, error] = arr; console.log(error);",
            None,
        ),
        // Multiple declarations fix (only fixes one at a time)
        ("let err, cb; console.log(err, cb);", "let error, cb; console.log(error, cb);", None),
        // Array pattern in parameters fix (only fixes one at a time)
        (
            "function f([err, cb]) { return err + cb; }",
            "function f([error, cb]) { return error + cb; }",
            None,
        ),
        // Compound name fixes - more cases
        ("const cbFn = () => {};", "const callbackFunction = () => {};", None),
        ("const errHandler = () => {};", "const errorHandler = () => {};", None),
        ("const msgText = '';", "const messageText = '';", None),
        // Async function fix
        (
            "async function fetch(err) { return err; }",
            "async function fetch(error) { return error; }",
            None,
        ),
        // Generator function fix
        ("function* gen(idx) { yield idx; }", "function* gen(index) { yield index; }", None),
        // Named function expression fix
        ("const x = function err() {}", "const x = function error() {}", None),
        // Import namespace fix
        (
            "import * as err from './module'; console.log(err);",
            "import * as error from './module'; console.log(error);",
            None,
        ),
        // Nested scope with conflict avoidance
        (
            "function outer() { const err = 1; function inner() { const err = 2; } }",
            "function outer() { const error = 1; function inner() { const error = 2; } }",
            None,
        ),
        // Import renamed to abbreviation - fix the local binding
        (
            "import { error as err } from './module'; console.log(err);",
            "import { error as error } from './module'; console.log(error);",
            None,
        ),
        // Kebab-case compound name (preserves separator)
        ("const err_msg = 'oops';", "const error_message = 'oops';", None),
        // === More abbreviation variations fixes ===
        ("const repo = 'git'; use(repo)", "const repository = 'git'; use(repository)", None),
        ("const proto = {}", "const prototype = {}", None),
        ("const ident = 'a'; log(ident)", "const identifier = 'a'; log(identifier)", None),
        ("const idents = []", "const identifiers = []", None),
        ("const stmt = 'x'", "const statement = 'x'", None),
        ("const stmts = []", "const statements = []", None),
        ("const decl = {}", "const declaration = {}", None),
        ("const decls = []", "const declarations = []", None),
        ("const expr = {}", "const expression = {}", None),
        ("const exprs = []", "const expressions = []", None),
        ("const cmd = 'x'", "const command = 'x'", None),
        ("const dep = {}", "const dependency = {}", None),
        ("const deps = []", "const dependencies = []", None),
        // === Reverse direction fixes ===
        ("const application = {}", "const app = {}", None),
        ("const applications = []", "const apps = []", None),
        ("const configuration = {}", "const config = {}", None),
        // === Class declaration fixes (lowercase class names stay lowercase) ===
        ("class cb {}", "class callback {}", None),
        ("class err {}", "class error {}", None),
        // === TypeScript type predicate fix (parameter name in predicate is also renamed) ===
        (
            "function check(val: unknown): val is string { return typeof val === 'string'; }",
            "function check(value: unknown): value is string { return typeof value === 'string'; }",
            None,
        ),
        // === Compound names with multiple abbreviations ===
        ("const errCbOptsObj = {}", "const errorCallbackOptionsObject = {}", None),
        ("const btnClickHandlerFn = () => {}", "const buttonClickHandlerFunction = () => {}", None),
        // === Destructuring rename pattern ===
        ("const { foo: err } = obj; use(err)", "const { foo: error } = obj; use(error)", None),
        // === Class field fixes ===
        (
            "class M { handle = (ctx: any) => ctx }",
            "class M { handle = (context: any) => context }",
            None,
        ),
        // === Array destructuring with default ===
        ("const [prop = 1] = []; use(prop)", "const [property = 1] = []; use(property)", None),
        // === Shorthand import fix (renames the binding, not the import specifier) ===
        (
            "import { cb } from './handlers'; cb()",
            "import { callback } from './handlers'; callback()",
            None,
        ),
        // === Config combinations ===
        (
            "const bar = 1",
            "const barFull = 1",
            Some(
                serde_json::json!([{ "extendDefaultReplacements": false, "replacements": { "bar": { "barFull": true } } }]),
            ),
        ),
        // === Closure capture fix ===
        (
            "function outer() { const err = 1; return () => err; }",
            "function outer() { const error = 1; return () => error; }",
            None,
        ),
        // === Sibling scope fix (fixes both since they're in different scopes) ===
        (
            "{ const cb = 1; } { const cb = 2; }",
            "{ const callback = 1; } { const callback = 2; }",
            None,
        ),
    ];

    Tester::new(NameReplacements::NAME, NameReplacements::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

#[test]
fn test_filenames() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        // Valid filenames
        ("const x = 1", None, None, Some(PathBuf::from("http-error.js"))),
        (
            "const x = 1",
            Some(serde_json::json!([{ "checkFilenames": false }])),
            None,
            Some(PathBuf::from("http-err.js")),
        ),
        ("const x = 1", None, None, Some(PathBuf::from("e2e.js"))),
        ("const x = 1", None, None, Some(PathBuf::from("vite-env.ts"))),
        ("const x = 1", None, None, Some(PathBuf::from("i18n.js"))),
    ];

    let fail = vec![
        // Invalid filenames
        ("const x = 1", None, None, Some(PathBuf::from("http-err.js"))),
        ("const x = 1", None, None, Some(PathBuf::from("err.js"))),
        ("const x = 1", None, None, Some(PathBuf::from("btn-click.js"))),
    ];

    Tester::new(NameReplacements::NAME, NameReplacements::PLUGIN, pass, fail)
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("filenames")
        .test_and_snapshot();
}
