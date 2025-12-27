use lazy_regex::Regex;
use oxc_ast::AstKind;
use oxc_ast::ast::{
    AssignmentTarget, AssignmentTargetMaybeDefault, BindingPatternKind, ImportAttributeKey,
    ImportDeclarationSpecifier, PropertyKey,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::node::NodeId;
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn camelcase_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Identifier '{name}' is not in camel case."))
        .with_help("Rename this identifier to use camelCase.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum PropertiesOption {
    #[default]
    Always,
    Never,
}

/// Pre-compiled allow pattern
#[derive(Debug, Clone)]
struct AllowPattern {
    literal: String,
    regex: Option<Regex>,
}

impl AllowPattern {
    fn new(pattern: String) -> Self {
        let regex = Regex::new(&pattern).ok();
        Self { literal: pattern, regex }
    }

    fn matches(&self, name: &str) -> bool {
        if name == self.literal {
            return true;
        }
        if let Some(ref re) = self.regex
            && re.is_match(name)
        {
            return true;
        }
        false
    }
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct CamelcaseConfig {
    properties: PropertiesOption,
    ignore_destructuring: bool,
    ignore_imports: bool,
    ignore_globals: bool,
    #[serde(default)]
    allow: Vec<String>,
}

#[derive(Debug, Default, Clone)]
struct CamelcaseRuntime {
    properties: PropertiesOption,
    ignore_destructuring: bool,
    ignore_imports: bool,
    ignore_globals: bool,
    allow_patterns: Vec<AllowPattern>,
}

impl From<CamelcaseConfig> for CamelcaseRuntime {
    fn from(config: CamelcaseConfig) -> Self {
        let allow_patterns = config.allow.into_iter().map(AllowPattern::new).collect();
        Self {
            properties: config.properties,
            ignore_destructuring: config.ignore_destructuring,
            ignore_imports: config.ignore_imports,
            ignore_globals: config.ignore_globals,
            allow_patterns,
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(from = "CamelcaseConfig")]
pub struct Camelcase(Box<CamelcaseRuntime>);

impl From<CamelcaseConfig> for Camelcase {
    fn from(config: CamelcaseConfig) -> Self {
        Self(Box::new(config.into()))
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces camelCase naming convention.
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent naming conventions make code harder to read and maintain.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var my_variable = 1;
    /// function do_something() {}
    /// obj.my_prop = 2;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var myVariable = 1;
    /// function doSomething() {}
    /// obj.myProp = 2;
    /// ```
    Camelcase,
    eslint,
    style,
    config = CamelcaseConfig,
);

impl Rule for Camelcase {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).unwrap_or_default().into_inner()
    }

    fn run_once(&self, ctx: &LintContext) {
        // Pre-compute positions to skip (equalsToOriginalName)
        let skip_positions = self.compute_skip_positions(ctx);
        let mut reported: FxHashSet<Span> = FxHashSet::default();

        // 1. Check all declared symbols
        for symbol_id in ctx.scoping().symbol_ids() {
            let name = ctx.scoping().symbol_name(symbol_id);
            let span = ctx.scoping().symbol_span(symbol_id);

            // Check the symbol declaration (skip if equalsToOriginalName)
            if !skip_positions.contains(&span.start) {
                self.report_if_bad(name, span, &mut reported, ctx);
            }

            // Check references to this symbol
            for reference in ctx.scoping().get_resolved_references(symbol_id) {
                let ref_node = ctx.nodes().get_node(reference.node_id());
                let ref_span = ref_node.span();

                // Skip if equalsToOriginalName (shorthand property, etc.)
                if skip_positions.contains(&ref_span.start) {
                    continue;
                }

                // Skip certain reference contexts (call, new, assignment pattern default)
                if Self::should_skip_reference(reference.node_id(), ctx) {
                    continue;
                }

                self.report_if_bad(name, ref_span, &mut reported, ctx);
            }
        }

        // 2. Check unresolved (through) references
        for (name, reference_ids) in ctx.scoping().root_unresolved_references() {
            // ignoreGlobals: skip references to variables that are globals
            // This includes both explicit globals config and env-provided globals (like browser/node builtins)
            if self.0.ignore_globals
                && (ctx.globals().is_enabled(*name) || ctx.env_contains_var(name))
            {
                continue;
            }

            for &reference_id in reference_ids {
                let reference = ctx.scoping().get_reference(reference_id);
                let ref_node = ctx.nodes().get_node(reference.node_id());
                let ref_span = ref_node.span();

                // Skip if equalsToOriginalName
                if skip_positions.contains(&ref_span.start) {
                    continue;
                }

                // Skip certain reference contexts
                if Self::should_skip_reference(reference.node_id(), ctx) {
                    continue;
                }

                self.report_if_bad(name, ref_span, &mut reported, ctx);
            }
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            // Object property definitions: { foo_bar: 1 }
            AstKind::ObjectProperty(prop) => {
                if self.0.properties == PropertiesOption::Never {
                    return;
                }

                // Skip import attribute keys: import ... with { my_type: 'json' }
                if Self::is_import_attribute_key(node, ctx) {
                    return;
                }

                // Check static property key
                if let PropertyKey::StaticIdentifier(ident) = &prop.key
                    && !self.is_good_name(&ident.name)
                {
                    ctx.diagnostic(camelcase_diagnostic(&ident.name, ident.span));
                }
            }

            // Class property definitions
            AstKind::PropertyDefinition(prop) => {
                if self.0.properties == PropertiesOption::Never {
                    return;
                }
                if let PropertyKey::StaticIdentifier(ident) = &prop.key
                    && !self.is_good_name(&ident.name)
                {
                    ctx.diagnostic(camelcase_diagnostic(&ident.name, ident.span));
                }
            }

            // Private identifiers in classes
            AstKind::PrivateIdentifier(ident) => {
                if self.0.properties == PropertiesOption::Never {
                    return;
                }
                if !self.is_good_name(&ident.name) {
                    ctx.diagnostic(camelcase_diagnostic(&ident.name, ident.span));
                }
            }

            // Method definitions in classes and objects
            AstKind::MethodDefinition(method) => {
                if self.0.properties == PropertiesOption::Never {
                    return;
                }
                if let PropertyKey::StaticIdentifier(ident) = &method.key
                    && !self.is_good_name(&ident.name)
                {
                    ctx.diagnostic(camelcase_diagnostic(&ident.name, ident.span));
                }
            }

            // Export specifiers: export { foo_bar } or export { camelCase as bar_baz }
            AstKind::ExportSpecifier(specifier) => {
                // Check the exported name (what it's being exported as)
                // ESLint checks the exported name, not the local name
                // Skip string literals: export { a as 'snake_cased' }
                match &specifier.exported {
                    oxc_ast::ast::ModuleExportName::IdentifierName(ident) => {
                        if !self.is_good_name(&ident.name) {
                            ctx.diagnostic(camelcase_diagnostic(&ident.name, ident.span));
                        }
                    }
                    oxc_ast::ast::ModuleExportName::IdentifierReference(ident) => {
                        if !self.is_good_name(&ident.name) {
                            ctx.diagnostic(camelcase_diagnostic(&ident.name, ident.span));
                        }
                    }
                    oxc_ast::ast::ModuleExportName::StringLiteral(_) => {
                        // String literal exports are allowed
                    }
                }
            }

            // Export all declarations: export * as foo_bar from 'mod'
            AstKind::ExportAllDeclaration(decl) => {
                if let Some(exported) = &decl.exported {
                    // Skip string literals: export * as 'snake_cased' from 'mod'
                    match exported {
                        oxc_ast::ast::ModuleExportName::IdentifierName(ident) => {
                            if !self.is_good_name(&ident.name) {
                                ctx.diagnostic(camelcase_diagnostic(&ident.name, ident.span));
                            }
                        }
                        oxc_ast::ast::ModuleExportName::IdentifierReference(ident) => {
                            if !self.is_good_name(&ident.name) {
                                ctx.diagnostic(camelcase_diagnostic(&ident.name, ident.span));
                            }
                        }
                        oxc_ast::ast::ModuleExportName::StringLiteral(_) => {
                            // String literal exports are allowed
                        }
                    }
                }
            }

            // Labels
            AstKind::LabeledStatement(stmt) => {
                if !self.is_good_name(&stmt.label.name) {
                    ctx.diagnostic(camelcase_diagnostic(&stmt.label.name, stmt.label.span));
                }
            }
            AstKind::BreakStatement(stmt) => {
                if let Some(label) = &stmt.label
                    && !self.is_good_name(&label.name)
                {
                    ctx.diagnostic(camelcase_diagnostic(&label.name, label.span));
                }
            }
            AstKind::ContinueStatement(stmt) => {
                if let Some(label) = &stmt.label
                    && !self.is_good_name(&label.name)
                {
                    ctx.diagnostic(camelcase_diagnostic(&label.name, label.span));
                }
            }

            // Assignment expressions - check member expression property on LHS
            AstKind::AssignmentExpression(expr) => {
                if self.0.properties == PropertiesOption::Never {
                    return;
                }
                self.check_assignment_target(&expr.left, ctx);
            }

            // Update expressions: obj.foo_bar++
            AstKind::UpdateExpression(expr) => {
                if self.0.properties == PropertiesOption::Never {
                    return;
                }
                if let oxc_ast::ast::SimpleAssignmentTarget::StaticMemberExpression(member) =
                    &expr.argument
                    && !self.is_good_name(&member.property.name)
                {
                    ctx.diagnostic(camelcase_diagnostic(
                        &member.property.name,
                        member.property.span,
                    ));
                }
            }

            _ => {}
        }
    }
}

impl Camelcase {
    /// Compute positions (span.start) where identifiers should be skipped
    /// due to equalsToOriginalName semantics
    fn compute_skip_positions(&self, ctx: &LintContext) -> FxHashSet<u32> {
        let mut skip_positions = FxHashSet::default();

        for node in ctx.nodes().iter() {
            match node.kind() {
                // Destructuring: { foo_bar } or { foo_bar: foo_bar }
                AstKind::BindingProperty(prop) => {
                    if self.0.ignore_destructuring {
                        let key_name = match &prop.key {
                            PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str()),
                            _ => None,
                        };
                        let value_name = match &prop.value.kind {
                            BindingPatternKind::BindingIdentifier(ident) => {
                                Some((ident.name.as_str(), ident.span.start))
                            }
                            BindingPatternKind::AssignmentPattern(pattern) => {
                                if let BindingPatternKind::BindingIdentifier(ident) =
                                    &pattern.left.kind
                                {
                                    Some((ident.name.as_str(), ident.span.start))
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        };
                        if let (Some(key), Some((value, pos))) = (key_name, value_name)
                            && key == value
                        {
                            skip_positions.insert(pos);
                        }
                    }
                }

                // Object shorthand property: const o = { some_property }
                AstKind::ObjectProperty(prop) => {
                    if self.0.ignore_destructuring
                        && prop.shorthand
                        && let PropertyKey::StaticIdentifier(ident) = &prop.key
                    {
                        // The identifier reference in shorthand should be skipped
                        // (the property key is the same as the value reference)
                        skip_positions.insert(ident.span.start);
                    }
                }

                // Import: import { foo_bar } from 'mod'
                AstKind::ImportDeclaration(decl) => {
                    if self.0.ignore_imports
                        && let Some(specifiers) = &decl.specifiers
                    {
                        for spec in specifiers {
                            if let ImportDeclarationSpecifier::ImportSpecifier(import_spec) = spec {
                                let imported_name = import_spec.imported.name();
                                if imported_name == import_spec.local.name.as_str() {
                                    skip_positions.insert(import_spec.local.span.start);
                                }
                            }
                        }
                    }
                }

                // Import attributes: import ... with { my_type: 'json' }
                // These keys should always be skipped
                AstKind::ImportAttribute(attr) => match &attr.key {
                    ImportAttributeKey::Identifier(ident) => {
                        skip_positions.insert(ident.span.start);
                    }
                    ImportAttributeKey::StringLiteral(_) => {}
                },

                // Assignment target shorthand: ({ foo_bar } = obj)
                AstKind::AssignmentTargetPropertyIdentifier(prop) => {
                    if self.0.ignore_destructuring {
                        // Shorthand assignment target - skip the binding
                        skip_positions.insert(prop.binding.span.start);
                    }
                }

                // Assignment target non-shorthand: ({ foo: foo } = obj)
                AstKind::AssignmentTargetPropertyProperty(prop) => {
                    if self.0.ignore_destructuring {
                        let key_name = match &prop.name {
                            PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str()),
                            _ => None,
                        };
                        let binding_info = Self::get_assignment_target_name(&prop.binding);
                        if let (Some(key), Some((binding_name, pos))) = (key_name, binding_info)
                            && key == binding_name
                        {
                            skip_positions.insert(pos);
                        }
                    }
                }

                _ => {}
            }
        }

        skip_positions
    }

    /// Extract the identifier name and span from an AssignmentTargetMaybeDefault
    fn get_assignment_target_name<'a>(
        target: &'a AssignmentTargetMaybeDefault<'a>,
    ) -> Option<(&'a str, u32)> {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(ident) => {
                Some((ident.name.as_str(), ident.span.start))
            }
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(with_default) => {
                if let AssignmentTarget::AssignmentTargetIdentifier(ident) = &with_default.binding {
                    Some((ident.name.as_str(), ident.span.start))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Check if a reference should be skipped based on its context
    /// ESLint skips identifiers in Call/New expressions entirely (backward compatibility)
    fn should_skip_reference(node_id: NodeId, ctx: &LintContext) -> bool {
        let node = ctx.nodes().get_node(node_id);
        let parent = ctx.nodes().parent_node(node_id);

        match parent.kind() {
            // Skip ALL identifiers in call expressions (callee, arguments, etc.)
            // This is ESLint's backward-compat behavior
            AstKind::CallExpression(_) | AstKind::NewExpression(_) => true,
            // Skip assignment pattern right side only: function foo(a = foo_bar) {}
            // ESLint skips only when the identifier is the `right` of AssignmentPattern
            AstKind::AssignmentPattern(pattern) => pattern.right.span() == node.span(),
            // Skip shorthand properties in import options
            AstKind::ObjectProperty(prop) if prop.shorthand => {
                Self::is_inside_import_options(node_id, ctx)
            }
            _ => false,
        }
    }

    /// Check if a node is inside import options (second argument to import())
    fn is_inside_import_options(node_id: NodeId, ctx: &LintContext) -> bool {
        let mut in_object_chain = false;

        for ancestor in ctx.nodes().ancestors(node_id) {
            match ancestor.kind() {
                AstKind::ObjectExpression(_) | AstKind::ObjectProperty(_) => {
                    in_object_chain = true;
                }
                AstKind::ImportExpression(_) => {
                    return in_object_chain;
                }
                AstKind::ImportDeclaration(_)
                | AstKind::ExportNamedDeclaration(_)
                | AstKind::ExportAllDeclaration(_)
                | AstKind::ExpressionStatement(_)
                | AstKind::Program(_)
                | AstKind::Function(_)
                | AstKind::ArrowFunctionExpression(_) => return false,
                _ => {}
            }
        }
        false
    }

    /// Check if this node is an import attribute key or dynamic import options key
    fn is_import_attribute_key(node: &AstNode, ctx: &LintContext) -> bool {
        let mut in_object_chain = false;

        // Note: ancestors() already starts from the parent node, not the node itself
        for ancestor in ctx.nodes().ancestors(node.id()) {
            match ancestor.kind() {
                // Static import attributes: import ... with { my_type: 'json' }
                AstKind::ImportAttribute(_) | AstKind::WithClause(_) => return true,

                // Track that we're inside an object literal chain
                AstKind::ObjectExpression(_) | AstKind::ObjectProperty(_) => {
                    in_object_chain = true;
                }

                // Dynamic import: import('foo.json', { my_with: { my_type: 'json' } })
                // If we're in an object chain and hit ImportExpression, we're in options
                AstKind::ImportExpression(_) => return in_object_chain,

                AstKind::ImportDeclaration(_)
                | AstKind::ExportNamedDeclaration(_)
                | AstKind::ExportAllDeclaration(_)
                | AstKind::ExpressionStatement(_)
                | AstKind::Program(_)
                | AstKind::Function(_)
                | AstKind::ArrowFunctionExpression(_) => return false,

                _ => {}
            }
        }
        false
    }

    /// Check assignment target for member expression properties
    fn check_assignment_target(&self, target: &AssignmentTarget, ctx: &LintContext) {
        match target {
            AssignmentTarget::StaticMemberExpression(member) => {
                if !self.is_good_name(&member.property.name) {
                    ctx.diagnostic(camelcase_diagnostic(
                        &member.property.name,
                        member.property.span,
                    ));
                }
            }
            AssignmentTarget::ArrayAssignmentTarget(arr) => {
                for element in arr.elements.iter().flatten() {
                    self.check_assignment_target_maybe_default(element, ctx);
                }
                // Check rest: [...obj.fo_o] = bar
                if let Some(rest) = &arr.rest {
                    self.check_assignment_target(&rest.target, ctx);
                }
            }
            AssignmentTarget::ObjectAssignmentTarget(obj) => {
                for prop in &obj.properties {
                    if let oxc_ast::ast::AssignmentTargetProperty::AssignmentTargetPropertyProperty(
                        p,
                    ) = prop
                    {
                        self.check_assignment_target_maybe_default(&p.binding, ctx);
                    }
                }
                // Check rest: {...obj.fo_o} = bar
                if let Some(rest) = &obj.rest {
                    self.check_assignment_target(&rest.target, ctx);
                }
            }
            _ => {}
        }
    }

    /// Check assignment target maybe default for member expression properties
    fn check_assignment_target_maybe_default(
        &self,
        target: &AssignmentTargetMaybeDefault,
        ctx: &LintContext,
    ) {
        match target {
            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(with_default) => {
                self.check_assignment_target(&with_default.binding, ctx);
            }
            AssignmentTargetMaybeDefault::StaticMemberExpression(member) => {
                if !self.is_good_name(&member.property.name) {
                    ctx.diagnostic(camelcase_diagnostic(
                        &member.property.name,
                        member.property.span,
                    ));
                }
            }
            AssignmentTargetMaybeDefault::ArrayAssignmentTarget(arr) => {
                for element in arr.elements.iter().flatten() {
                    self.check_assignment_target_maybe_default(element, ctx);
                }
                if let Some(rest) = &arr.rest {
                    self.check_assignment_target(&rest.target, ctx);
                }
            }
            AssignmentTargetMaybeDefault::ObjectAssignmentTarget(obj) => {
                for prop in &obj.properties {
                    if let oxc_ast::ast::AssignmentTargetProperty::AssignmentTargetPropertyProperty(
                        p,
                    ) = prop
                    {
                        self.check_assignment_target_maybe_default(&p.binding, ctx);
                    }
                }
                if let Some(rest) = &obj.rest {
                    self.check_assignment_target(&rest.target, ctx);
                }
            }
            // Other variants don't have properties we care about
            _ => {}
        }
    }

    /// Report if name is bad, using deduplication
    fn report_if_bad(
        &self,
        name: &str,
        span: Span,
        reported: &mut FxHashSet<Span>,
        ctx: &LintContext,
    ) {
        if reported.contains(&span) {
            return;
        }
        if !self.is_good_name(name) {
            reported.insert(span);
            ctx.diagnostic(camelcase_diagnostic(name, span));
        }
    }

    /// Check if a name is acceptable
    fn is_good_name(&self, name: &str) -> bool {
        if self.0.allow_patterns.iter().any(|p| p.matches(name)) {
            return true;
        }
        !is_underscored(name)
    }
}

/// Check if a name contains underscores in the middle (not camelCase)
fn is_underscored(name: &str) -> bool {
    let name = name.trim_start_matches('_');
    let name = name.trim_end_matches('_');

    if name.is_empty() {
        return false;
    }

    if is_all_caps(name) {
        return false;
    }

    name.contains('_')
}

/// Check if a name is in ALL_CAPS style
fn is_all_caps(name: &str) -> bool {
    let has_letter = name.chars().any(char::is_alphabetic);
    if !has_letter {
        return false;
    }
    name.chars().all(|c| c.is_uppercase() || c.is_ascii_digit() || c == '_')
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"firstName = "Nicholas""#, None),
        (r#"FIRST_NAME = "Nicholas""#, None),
        (r#"__myPrivateVariable = "Patrick""#, None),
        (r#"myPrivateVariable_ = "Patrick""#, None),
        ("function doSomething(){}", None),
        ("do_something()", None),
        ("new do_something", None),
        ("new do_something()", None),
        ("foo.do_something()", None),
        ("var foo = bar.baz_boom;", None),
        ("var foo = bar.baz_boom.something;", None),
        ("foo.boom_pow.qux = bar.baz_boom.something;", None),
        ("if (bar.baz_boom) {}", None),
        ("var obj = { key: foo.bar_baz };", None),
        ("var arr = [foo.bar_baz];", None),
        ("[foo.bar_baz]", None),
        ("var arr = [foo.bar_baz.qux];", None),
        ("[foo.bar_baz.nesting]", None),
        ("if (foo.bar_baz === boom.bam_pow) { [foo.baz_boom] }", None),
        ("var o = {key: 1}", Some(serde_json::json!([{ "properties": "always" }]))),
        ("var o = {_leading: 1}", Some(serde_json::json!([{ "properties": "always" }]))),
        ("var o = {trailing_: 1}", Some(serde_json::json!([{ "properties": "always" }]))),
        ("var o = {bar_baz: 1}", Some(serde_json::json!([{ "properties": "never" }]))),
        ("var o = {_leading: 1}", Some(serde_json::json!([{ "properties": "never" }]))),
        ("var o = {trailing_: 1}", Some(serde_json::json!([{ "properties": "never" }]))),
        ("obj.a_b = 2;", Some(serde_json::json!([{ "properties": "never" }]))),
        ("obj._a = 2;", Some(serde_json::json!([{ "properties": "always" }]))),
        ("obj.a_ = 2;", Some(serde_json::json!([{ "properties": "always" }]))),
        ("obj._a = 2;", Some(serde_json::json!([{ "properties": "never" }]))),
        ("obj.a_ = 2;", Some(serde_json::json!([{ "properties": "never" }]))),
        (
            "var obj = {
			 a_a: 1
			};
			 obj.a_b = 2;",
            Some(serde_json::json!([{ "properties": "never" }])),
        ),
        ("obj.foo_bar = function(){};", Some(serde_json::json!([{ "properties": "never" }]))),
        ("const { ['foo']: _foo } = obj;", None),
        ("const { [_foo_]: foo } = obj;", None),
        (
            "var { category_id } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        (
            "var { category_id: category_id } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        (
            "var { category_id = 1 } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        (
            "var { [{category_id} = query]: categoryId } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        ("var { category_id: category } = query;", None),
        ("var { _leading } = query;", None),
        ("var { trailing_ } = query;", None),
        (r#"import { camelCased } from "external module";"#, None),
        (r#"import { _leading } from "external module";"#, None),
        (r#"import { trailing_ } from "external module";"#, None),
        (r#"import { no_camelcased as camelCased } from "external-module";"#, None),
        (r#"import { no_camelcased as _leading } from "external-module";"#, None),
        (r#"import { no_camelcased as trailing_ } from "external-module";"#, None),
        (
            r#"import { no_camelcased as camelCased, anotherCamelCased } from "external-module";"#,
            None,
        ),
        ("import { snake_cased } from 'mod'", Some(serde_json::json!([{ "ignoreImports": true }]))),
        (
            "import { snake_cased as snake_cased } from 'mod'",
            Some(serde_json::json!([{ "ignoreImports": true }])),
        ),
        (
            "import { 'snake_cased' as snake_cased } from 'mod'",
            Some(serde_json::json!([{ "ignoreImports": true }])),
        ),
        ("import { camelCased } from 'mod'", Some(serde_json::json!([{ "ignoreImports": false }]))),
        ("export { a as 'snake_cased' } from 'mod'", None),
        ("export * as 'snake_cased' from 'mod'", None),
        (
            "var _camelCased = aGlobalVariable",
            Some(serde_json::json!([{ "ignoreGlobals": false }])),
        ),
        (
            "var camelCased = _aGlobalVariable",
            Some(serde_json::json!([{ "ignoreGlobals": false }])),
        ),
        // Note: ignoreGlobals tests with explicit globals config are in a separate Tester run below
        ("function foo({ no_camelcased: camelCased }) {};", None),
        ("function foo({ no_camelcased: _leading }) {};", None),
        ("function foo({ no_camelcased: trailing_ }) {};", None),
        ("function foo({ camelCased = 'default value' }) {};", None),
        ("function foo({ _leading = 'default value' }) {};", None),
        ("function foo({ trailing_ = 'default value' }) {};", None),
        ("function foo({ camelCased }) {};", None),
        ("function foo({ _leading }) {}", None),
        ("function foo({ trailing_ }) {}", None),
        ("ignored_foo = 0;", Some(serde_json::json!([{ "allow": ["ignored_foo"] }]))),
        (
            "ignored_foo = 0; ignored_bar = 1;",
            Some(serde_json::json!([{ "allow": ["ignored_foo", "ignored_bar"] }])),
        ),
        ("user_id = 0;", Some(serde_json::json!([{ "allow": ["_id$"] }]))),
        ("__option_foo__ = 0;", Some(serde_json::json!([{ "allow": ["__option_foo__"] }]))),
        (
            "__option_foo__ = 0; user_id = 0; foo = 1",
            Some(serde_json::json!([{ "allow": ["__option_foo__", "_id$"] }])),
        ),
        ("fo_o = 0;", Some(serde_json::json!([{ "allow": ["__option_foo__", "fo_o"] }]))),
        ("user = 0;", Some(serde_json::json!([{ "allow": [] }]))),
        ("foo = { [computedBar]: 0 };", Some(serde_json::json!([{ "ignoreDestructuring": true }]))),
        ("({ a: obj.fo_o } = bar);", Some(serde_json::json!([{ "allow": ["fo_o"] }]))),
        ("({ a: obj.foo } = bar);", Some(serde_json::json!([{ "allow": ["fo_o"] }]))),
        ("({ a: obj.fo_o } = bar);", Some(serde_json::json!([{ "properties": "never" }]))),
        ("({ a: obj.fo_o.b_ar } = bar);", Some(serde_json::json!([{ "properties": "never" }]))),
        ("({ a: { b: obj.fo_o } } = bar);", Some(serde_json::json!([{ "properties": "never" }]))),
        ("([obj.fo_o] = bar);", Some(serde_json::json!([{ "properties": "never" }]))),
        ("({ c: [ob.fo_o]} = bar);", Some(serde_json::json!([{ "properties": "never" }]))),
        ("([obj.fo_o.b_ar] = bar);", Some(serde_json::json!([{ "properties": "never" }]))),
        ("({obj} = baz.fo_o);", None),
        ("([obj] = baz.fo_o);", None),
        ("([obj.foo = obj.fo_o] = bar);", Some(serde_json::json!([{ "properties": "always" }]))),
        (
            "class C { camelCase; #camelCase; #camelCase2() {} }",
            Some(serde_json::json!([{ "properties": "always" }])),
        ),
        (
            "class C { snake_case; #snake_case; #snake_case2() {} }",
            Some(serde_json::json!([{ "properties": "never" }])),
        ),
        (
            "
			            const { some_property } = obj;

			            const bar = { some_property };

			            obj.some_property = 10;

			            const xyz = { some_property: obj.some_property };

			            const foo = ({ some_property }) => {
			                console.log(some_property)
			            };
			            ",
            Some(serde_json::json!([{ "properties": "never", "ignoreDestructuring": true }])),
        ),
        (
            "
			            const { some_property } = obj;
			            doSomething({ some_property });
			            ",
            Some(serde_json::json!([{ "properties": "never", "ignoreDestructuring": true }])),
        ),
        (
            "import foo from 'foo.json' with { my_type: 'json' }",
            Some(serde_json::json!([{ "properties": "always", "ignoreImports": false }])),
        ),
        (
            "export * from 'foo.json' with { my_type: 'json' }",
            Some(serde_json::json!([{ "properties": "always", "ignoreImports": false }])),
        ),
        (
            "export { default } from 'foo.json' with { my_type: 'json' }",
            Some(serde_json::json!([{ "properties": "always", "ignoreImports": false }])),
        ),
        (
            "import('foo.json', { my_with: { my_type: 'json' } })",
            Some(serde_json::json!([{ "properties": "always", "ignoreImports": false }])),
        ),
        (
            "import('foo.json', { 'with': { my_type: 'json' } })",
            Some(serde_json::json!([{ "properties": "always", "ignoreImports": false }])),
        ),
        (
            "import('foo.json', { my_with: { my_type } })",
            Some(serde_json::json!([{ "properties": "always", "ignoreImports": false }])),
        ),
        (
            "import('foo.json', { my_with: { my_type } })",
            Some(serde_json::json!([{ "properties": "always", "ignoreImports": false }])),
        ),
    ];

    let fail = vec![
        (r#"first_name = "Nicholas""#, None),
        (r#"__private_first_name = "Patrick""#, None),
        ("function foo_bar(){}", None),
        ("obj.foo_bar = function(){};", None),
        ("bar_baz.foo = function(){};", None),
        ("[foo_bar.baz]", None),
        ("if (foo.bar_baz === boom.bam_pow) { [foo_bar.baz] }", None),
        ("foo.bar_baz = boom.bam_pow", None),
        ("var foo = { bar_baz: boom.bam_pow }", None),
        (
            "var foo = { bar_baz: boom.bam_pow }",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        ("foo.qux.boom_pow = { bar: boom.bam_pow }", None),
        ("var o = {bar_baz: 1}", Some(serde_json::json!([{ "properties": "always" }]))),
        ("obj.a_b = 2;", Some(serde_json::json!([{ "properties": "always" }]))),
        ("var { category_id: category_alias } = query;", None),
        (
            "var { category_id: category_alias } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        (
            "var { [category_id]: categoryId } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        ("var { [category_id]: categoryId } = query;", None),
        (
            "var { category_id: categoryId, ...other_props } = query;",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        ("var { category_id } = query;", None),
        ("var { category_id: category_id } = query;", None),
        ("var { category_id = 1 } = query;", None),
        (r#"import no_camelcased from "external-module";"#, None),
        (r#"import * as no_camelcased from "external-module";"#, None),
        (r#"import { no_camelcased } from "external-module";"#, None),
        (r#"import { no_camelcased as no_camel_cased } from "external module";"#, None),
        (r#"import { camelCased as no_camel_cased } from "external module";"#, None),
        ("import { 'snake_cased' as snake_cased } from 'mod'", None),
        (
            "import { 'snake_cased' as another_snake_cased } from 'mod'",
            Some(serde_json::json!([{ "ignoreImports": true }])),
        ),
        (r#"import { camelCased, no_camelcased } from "external-module";"#, None),
        (
            r#"import { no_camelcased as camelCased, another_no_camelcased } from "external-module";"#,
            None,
        ),
        (r#"import camelCased, { no_camelcased } from "external-module";"#, None),
        (
            r#"import no_camelcased, { another_no_camelcased as camelCased } from "external-module";"#,
            None,
        ),
        ("import snake_cased from 'mod'", Some(serde_json::json!([{ "ignoreImports": true }]))),
        (
            "import * as snake_cased from 'mod'",
            Some(serde_json::json!([{ "ignoreImports": true }])),
        ),
        ("import snake_cased from 'mod'", Some(serde_json::json!([{ "ignoreImports": false }]))),
        (
            "import * as snake_cased from 'mod'",
            Some(serde_json::json!([{ "ignoreImports": false }])),
        ),
        ("var camelCased = snake_cased", Some(serde_json::json!([{ "ignoreGlobals": false }]))),
        ("a_global_variable.foo()", Some(serde_json::json!([{ "ignoreGlobals": false }]))),
        ("a_global_variable[undefined]", Some(serde_json::json!([{ "ignoreGlobals": false }]))),
        ("var camelCased = snake_cased", None),
        ("var camelCased = snake_cased", Some(serde_json::json!([{}]))),
        ("foo.a_global_variable = bar", Some(serde_json::json!([{ "ignoreGlobals": true }]))),
        (
            "var foo = { a_global_variable: bar }",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ),
        (
            "var foo = { a_global_variable: a_global_variable }",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ),
        (
            "var foo = { a_global_variable() {} }",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ),
        (
            "class Foo { a_global_variable() {} }",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ),
        ("a_global_variable: for (;;);", Some(serde_json::json!([{ "ignoreGlobals": true }]))),
        (
            "if (foo) { let a_global_variable; a_global_variable = bar; }",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ),
        (
            "function foo(a_global_variable) { foo = a_global_variable; }",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ),
        ("var a_global_variable", Some(serde_json::json!([{ "ignoreGlobals": true }]))),
        ("function a_global_variable () {}", Some(serde_json::json!([{ "ignoreGlobals": true }]))),
        (
            "const a_global_variable = foo; bar = a_global_variable",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ),
        (
            "bar = a_global_variable; var a_global_variable;",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
        ),
        ("var foo = { a_global_variable }", Some(serde_json::json!([{ "ignoreGlobals": true }]))),
        // ESLint: ignoreGlobals only skips configured globals, undefined variables still fail
        ("undefined_variable", Some(serde_json::json!([{ "ignoreGlobals": true }]))),
        ("implicit_global = 1", Some(serde_json::json!([{ "ignoreGlobals": true }]))),
        ("export * as snake_cased from 'mod'", None),
        ("function foo({ no_camelcased }) {};", None),
        ("function foo({ no_camelcased = 'default value' }) {};", None),
        ("const no_camelcased = 0; function foo({ camelcased_value = no_camelcased}) {}", None),
        ("const { bar: no_camelcased } = foo;", None),
        ("function foo({ value_1: my_default }) {}", None),
        ("function foo({ isCamelcased: no_camelcased }) {};", None),
        ("var { foo: bar_baz = 1 } = quz;", None),
        ("const { no_camelcased = false } = bar;", None),
        ("const { no_camelcased = foo_bar } = bar;", None),
        ("not_ignored_foo = 0;", Some(serde_json::json!([{ "allow": ["ignored_bar"] }]))),
        ("not_ignored_foo = 0;", Some(serde_json::json!([{ "allow": ["_id$"] }]))),
        (
            "foo = { [computed_bar]: 0 };",
            Some(serde_json::json!([{ "ignoreDestructuring": true }])),
        ),
        ("({ a: obj.fo_o } = bar);", None),
        ("({ a: obj.fo_o } = bar);", Some(serde_json::json!([{ "ignoreDestructuring": true }]))),
        ("({ a: obj.fo_o.b_ar } = baz);", None),
        ("({ a: { b: { c: obj.fo_o } } } = bar);", None),
        ("({ a: { b: { c: obj.fo_o.b_ar } } } = baz);", None),
        ("([obj.fo_o] = bar);", None),
        ("([obj.fo_o] = bar);", Some(serde_json::json!([{ "ignoreDestructuring": true }]))),
        ("([obj.fo_o = 1] = bar);", Some(serde_json::json!([{ "properties": "always" }]))),
        ("({ a: [obj.fo_o] } = bar);", None),
        ("({ a: { b: [obj.fo_o] } } = bar);", None),
        ("([obj.fo_o.ba_r] = baz);", None),
        ("({...obj.fo_o} = baz);", None),
        ("({...obj.fo_o.ba_r} = baz);", None),
        ("({c: {...obj.fo_o }} = baz);", None),
        ("obj.o_k.non_camelcase = 0", Some(serde_json::json!([{ "properties": "always" }]))),
        ("(obj?.o_k).non_camelcase = 0", Some(serde_json::json!([{ "properties": "always" }]))),
        ("class C { snake_case; }", Some(serde_json::json!([{ "properties": "always" }]))),
        (
            "class C { #snake_case; foo() { this.#snake_case; } }",
            Some(serde_json::json!([{ "properties": "always" }])),
        ),
        ("class C { #snake_case() {} }", Some(serde_json::json!([{ "properties": "always" }]))),
        (
            "
			            const { some_property } = obj;
			            doSomething({ some_property });
			            ",
            Some(serde_json::json!([{ "properties": "always", "ignoreDestructuring": true }])),
        ),
        (
            r#"
			            const { some_property } = obj;
			            doSomething({ some_property });
			            doSomething({ [some_property]: "bar" });
			            "#,
            Some(serde_json::json!([{ "properties": "never", "ignoreDestructuring": true }])),
        ),
        (
            "
			            const { some_property } = obj;

			            const bar = { some_property };

			            obj.some_property = 10;

			            const xyz = { some_property: obj.some_property };

			            const foo = ({ some_property }) => {
			                console.log(some_property)
			            };
			            ",
            Some(serde_json::json!([{ "properties": "always", "ignoreDestructuring": true }])),
        ),
        (
            "import('foo.json', { my_with: { [my_type]: 'json' } })",
            Some(serde_json::json!([{ "properties": "always", "ignoreImports": false }])),
        ),
        (
            "import('foo.json', { my_with: { my_type: my_json } })",
            Some(serde_json::json!([{ "properties": "always", "ignoreImports": false }])),
        ),
    ];

    Tester::new(Camelcase::NAME, Camelcase::PLUGIN, pass, fail).test_and_snapshot();

    // Separate test for ignoreGlobals with explicit globals config
    // ESLint only skips globals that are explicitly configured
    let pass_with_globals = vec![
        (
            "var camelCased = a_global_variable",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
            Some(serde_json::json!({ "globals": { "a_global_variable": "readonly" } })),
        ),
        (
            "a_global_variable.foo()",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
            Some(serde_json::json!({ "globals": { "a_global_variable": "readonly" } })),
        ),
        (
            "a_global_variable[undefined]",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
            Some(
                serde_json::json!({ "globals": { "a_global_variable": "readonly", "undefined": "readonly" } }),
            ),
        ),
        (
            "var foo = a_global_variable.bar",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
            Some(serde_json::json!({ "globals": { "a_global_variable": "readonly" } })),
        ),
        (
            "a_global_variable.foo = bar",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
            Some(
                serde_json::json!({ "globals": { "a_global_variable": "writable", "bar": "readonly" } }),
            ),
        ),
        (
            "( { foo: a_global_variable.bar } = baz )",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
            Some(
                serde_json::json!({ "globals": { "a_global_variable": "readonly", "baz": "readonly" } }),
            ),
        ),
        (
            "a_global_variable = foo",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
            Some(
                serde_json::json!({ "globals": { "a_global_variable": "writable", "foo": "readonly" } }),
            ),
        ),
        (
            "({ a_global_variable } = foo)",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
            Some(
                serde_json::json!({ "globals": { "a_global_variable": "writable", "foo": "readonly" } }),
            ),
        ),
        (
            "({ snake_cased: a_global_variable } = foo)",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
            Some(
                serde_json::json!({ "globals": { "a_global_variable": "writable", "foo": "readonly" } }),
            ),
        ),
        (
            "({ snake_cased: a_global_variable = foo } = bar)",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
            Some(
                serde_json::json!({ "globals": { "a_global_variable": "writable", "foo": "readonly", "bar": "readonly" } }),
            ),
        ),
        (
            "[a_global_variable] = bar",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
            Some(
                serde_json::json!({ "globals": { "a_global_variable": "writable", "bar": "readonly" } }),
            ),
        ),
        (
            "[a_global_variable = foo] = bar",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
            Some(
                serde_json::json!({ "globals": { "a_global_variable": "writable", "foo": "readonly", "bar": "readonly" } }),
            ),
        ),
        (
            "foo[a_global_variable] = bar",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
            Some(
                serde_json::json!({ "globals": { "a_global_variable": "readonly", "foo": "writable", "bar": "readonly" } }),
            ),
        ),
        (
            "var foo = { [a_global_variable]: bar }",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
            Some(
                serde_json::json!({ "globals": { "a_global_variable": "readonly", "bar": "readonly" } }),
            ),
        ),
        (
            "var { [a_global_variable]: foo } = bar",
            Some(serde_json::json!([{ "ignoreGlobals": true }])),
            Some(
                serde_json::json!({ "globals": { "a_global_variable": "readonly", "bar": "readonly" } }),
            ),
        ),
    ];
    let fail_with_globals: Vec<(&str, Option<serde_json::Value>, Option<serde_json::Value>)> =
        vec![];

    Tester::new(Camelcase::NAME, Camelcase::PLUGIN, pass_with_globals, fail_with_globals).test();
}
