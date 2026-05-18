use std::ops::Deref;

use lazy_regex::Regex;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use oxc_ast::{
    AstKind,
    ast::{
        AssignmentTarget, AssignmentTargetMaybeDefault, AssignmentTargetPropertyProperty,
        BindingIdentifier, BindingPattern, BindingProperty, ExportSpecifier, FunctionType,
        IdentifierName, IdentifierReference, LabelIdentifier, ModuleExportName, ObjectProperty,
        PrivateIdentifier, PropertyKey,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    config::GlobalValue,
    context::{ContextHost, LintContext},
    rule::{Rule, TupleRuleConfig},
    utils::deserialize_required_regex_option,
};

fn id_match_diagnostic(span: Span, name: &str, pattern: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Identifier '{name}' does not match the pattern '{pattern}'."))
        .with_label(span)
}

fn id_match_private_diagnostic(span: Span, name: &str, pattern: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Identifier '#{name}' does not match the pattern '{pattern}'."))
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
pub struct IdMatch(Box<IdMatchConfig>);

impl Deref for IdMatch {
    type Target = IdMatchConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(default)]
pub struct IdMatchConfig(
    #[serde(default, deserialize_with = "deserialize_required_regex_option")] Option<Regex>,
    IdMatchOptions,
);

impl IdMatchConfig {
    fn regex(&self) -> Option<&Regex> {
        self.0.as_ref()
    }

    fn options(&self) -> &IdMatchOptions {
        &self.1
    }

    fn pattern(&self) -> &str {
        self.regex().map_or("", Regex::as_str)
    }
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct IdMatchOptions {
    /// Whether class field names are checked, including public fields,
    /// accessor properties, and private field names.
    class_fields: bool,
    /// Whether object literal property names, class method names, and assigned
    /// member names such as `obj.prop = value` are checked.
    properties: bool,
    /// Whether to ignore shorthand and aliased bindings introduced by object
    /// destructuring, such as `foo` in `const { foo } = obj` and `alias` in
    /// `const { foo: alias } = obj`. This does not suppress computed key
    /// references such as `const { [key]: value } = obj`.
    ignore_destructuring: bool,
    /// Whether to check only declaration names. References, member names,
    /// labels, and function or arrow parameters are skipped.
    only_declarations: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a naming convention for identifiers by requiring each checked
    /// name to match a configured regular expression.
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent identifier names make code harder to read and maintain.
    ///
    /// This rule is most commonly used to enforce a project-wide convention
    /// such as `camelCase`, `snake_case`, or names without underscores.
    ///
    /// The configured pattern is compiled with Rust regex syntax. Most common
    /// naming patterns work the same way as JavaScript regular expressions, but
    /// JavaScript-specific features such as lookaround assertions and
    /// backreferences are not supported. Unicode escapes also use Rust syntax,
    /// so `\uXXXX` should be written as `\u{XXXX}`.
    ///
    /// ### Known differences from ESLint
    ///
    /// - Computed destructuring keys are checked in both binding and assignment
    ///   patterns, for example `const { [bad_name]: x } = obj` and
    ///   `({ [bad_name]: x } = obj)`. This still applies when
    ///   `ignoreDestructuring` is enabled because the computed key is a normal
    ///   reference expression, not a binding introduced by destructuring.
    /// - With `properties` enabled, ordinary top-level keys in dynamic import
    ///   options are checked, for example `import("x", { bad_option: true })`.
    ///   Import attributes inside `with { ... }` are still ignored.
    ///
    /// These cases are intentionally stricter than ESLint because they still
    /// contain user-controlled identifier names that participate in normal code.
    ///
    /// TypeScript syntax is supported on a best-effort basis for identifiers
    /// that flow through the same visited AST node kinds and transparent
    /// wrappers. This rule intentionally does not try to cover the full
    /// TypeScript naming surface.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// ```js
    /// /* id-match: ["error", "^[^_]+$"] */
    /// var first_name = "John";
    ///
    /// /* id-match: ["error", "^[^_]+$", { "properties": true }] */
    /// obj.first_name = "John";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```js
    /// /* id-match: ["error", "^[^_]+$"] */
    /// var firstName = "John";
    ///
    /// /* id-match: ["error", "^[^_]+$", { "ignoreDestructuring": true }] */
    /// const { first_name } = user;
    /// ```
    IdMatch,
    eslint,
    style,
    none,
    config = IdMatchConfig,
    version = "next",
);

impl Rule for IdMatch {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<TupleRuleConfig<IdMatchConfig>>(value)
            .map(|cfg| Self(Box::new(cfg.into_inner())))
    }

    fn should_run(&self, _ctx: &ContextHost) -> bool {
        self.regex().is_some()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::BindingIdentifier(ident) => self.check_binding_identifier(ident, node, ctx),
            AstKind::IdentifierReference(ident) => {
                self.check_identifier_reference(ident, node, ctx);
            }
            AstKind::IdentifierName(ident) => self.check_identifier_name(ident, node, ctx),
            AstKind::PrivateIdentifier(ident) => self.check_private_identifier(ident, node, ctx),
            AstKind::LabelIdentifier(ident) => self.check_label_identifier(ident, node, ctx),
            _ => {}
        }
    }
}

impl IdMatch {
    fn check_binding_identifier<'a>(
        &self,
        ident: &BindingIdentifier<'a>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        let regex = self.regex().expect("regex should be present when should_run returns true");
        let name = ident.name.as_str();
        if regex.is_match(name) {
            return;
        }

        let parent = ctx.nodes().parent_node(node.id());

        if binding_is_import_local(ident, parent) {
            self.report(ctx, node.span(), name);
            return;
        }

        match parent.kind() {
            AstKind::BindingProperty(property) => {
                if binding_property_value_is_handled_by_key(property, ident.span) {
                    return;
                }
                if self.options().ignore_destructuring
                    && binding_property_is_exact_key_value(property)
                {
                    return;
                }
            }
            AstKind::AssignmentPattern(_) => {
                if let AstKind::BindingProperty(property) = ctx.nodes().parent_kind(parent.id()) {
                    if binding_property_value_is_handled_by_key(property, ident.span) {
                        return;
                    }
                    if self.options().ignore_destructuring
                        && binding_property_is_exact_key_value(property)
                    {
                        return;
                    }
                }
                // ESLint does not check defaulted bindings unless property
                // checks are enabled. OXC represents array defaults with the
                // same assignment-pattern shape, for example
                // `const [bad_name = 1] = arr`.
                if !self.options().properties {
                    return;
                }
            }
            // Function parameter defaults are stored directly on
            // `FormalParameter`, but they follow the same policy as
            // assignment-pattern defaults above.
            AstKind::FormalParameter(param)
                if param.initializer.is_some() && !self.options().properties =>
            {
                return;
            }
            AstKind::BindingRestElement(_) => {}
            _ if self.options().ignore_destructuring && is_inside_object_pattern(node, ctx) => {
                return;
            }
            _ => {}
        }

        let effective_parent = eslint_effective_parent_for_binding(parent, ctx);
        self.report_in_generic_context(ctx, node.span(), name, effective_parent);
    }

    fn check_identifier_reference<'a>(
        &self,
        ident: &IdentifierReference<'a>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        let regex = self.regex().expect("regex should be present when should_run returns true");
        let name = ident.name.as_str();
        if regex.is_match(name) {
            return;
        }

        if is_known_external_global(ident, ctx) {
            return;
        }

        let (parent, subject_span) = transparent_reference_parent(node, ctx);

        if self.handle_member_reference(ident, node, subject_span, parent, ctx) {
            return;
        }

        match parent.kind() {
            AstKind::ObjectProperty(property) => {
                if property.shorthand {
                    return;
                }
                if !self.options().properties && !property.computed {
                    return;
                }
            }
            AstKind::BindingProperty(property) => {
                // A computed destructuring key is a reference expression, not
                // a binding introduced by destructuring.
                if property.computed && property.key.span().contains_inclusive(ident.span) {
                    self.report_in_generic_context(ctx, node.span(), name, parent);
                }
                return;
            }
            AstKind::AssignmentTargetPropertyIdentifier(_) => {
                if self.options().ignore_destructuring {
                    return;
                }
                self.report_in_generic_context(ctx, node.span(), name, parent);
                return;
            }
            AstKind::AssignmentTargetPropertyProperty(property) => {
                if property.name.span().contains_inclusive(ident.span) {
                    // Keep assignment destructuring consistent with binding
                    // destructuring for computed keys.
                    if property.computed {
                        self.report_in_generic_context(ctx, node.span(), name, parent);
                    }
                    return;
                }
                if self.options().ignore_destructuring
                    && assignment_property_is_exact_key_value(property)
                {
                    return;
                }
            }
            AstKind::PropertyDefinition(_) | AstKind::AccessorProperty(_) => {
                if self.options().class_fields {
                    self.report(ctx, node.span(), name);
                }
                return;
            }
            AstKind::AssignmentTargetRest(_) => {}
            // The right-hand side of a default value, such as `bad_name` in
            // `p = bad_name`, is skipped to match ESLint's default-value guard.
            AstKind::AssignmentPattern(_) | AstKind::FormalParameter(_) => return,
            _ if self.options().ignore_destructuring
                && is_inside_object_assignment_target(node, ctx) =>
            {
                return;
            }
            _ => {}
        }

        self.report_in_generic_context(ctx, node.span(), name, parent);
    }

    fn check_identifier_name<'a>(
        &self,
        ident: &IdentifierName<'a>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        let regex = self.regex().expect("regex should be present when should_run returns true");
        let name = ident.name.as_str();
        if regex.is_match(name) {
            return;
        }

        let parent = ctx.nodes().parent_node(node.id());

        match parent.kind() {
            AstKind::ImportAttribute(_) | AstKind::WithClause(_) => return,
            AstKind::ImportSpecifier(specifier)
                if specifier.imported.identifier_name() == Some(ident.name) =>
            {
                return;
            }
            AstKind::ExportSpecifier(specifier) => {
                if !export_specifier_is_duplicate_clone(specifier, node) {
                    self.report_in_generic_context(ctx, node.span(), name, parent);
                }
                return;
            }
            AstKind::ObjectProperty(property)
                if property.key.span().contains_inclusive(ident.span) && !property.computed =>
            {
                if !self.options().properties {
                    return;
                }
                if is_dynamic_import_attribute_object_property(property, ctx) {
                    return;
                }
                self.report(ctx, node.span(), name);
                return;
            }
            AstKind::BindingProperty(property)
                if property.key.span().contains_inclusive(ident.span) && !property.computed =>
            {
                if self.options().ignore_destructuring
                    && binding_property_is_exact_key_value(property)
                {
                    return;
                }
                if binding_property_key_should_report(property) {
                    self.report(ctx, node.span(), name);
                }
                return;
            }
            AstKind::StaticMemberExpression(member) if member.property.span == ident.span => {
                if self.options().properties && member_is_assignment_left(parent, ctx) {
                    self.report(ctx, node.span(), name);
                }
                return;
            }
            AstKind::MethodDefinition(_) => {
                self.report_in_generic_context(ctx, node.span(), name, parent);
                return;
            }
            AstKind::PropertyDefinition(_) | AstKind::AccessorProperty(_) => {
                if self.options().class_fields {
                    self.report(ctx, node.span(), name);
                }
                return;
            }
            AstKind::AssignmentTargetPropertyProperty(property)
                if property.name.span().contains_inclusive(ident.span) =>
            {
                if assignment_property_is_exact_key_value(property)
                    && !self.options().ignore_destructuring
                {
                    self.report(ctx, node.span(), name);
                }
                return;
            }
            _ => {}
        }

        self.report_in_generic_context(ctx, node.span(), name, parent);
    }

    fn check_private_identifier<'a>(
        &self,
        ident: &PrivateIdentifier<'a>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        let regex = self.regex().expect("regex should be present when should_run returns true");
        let name = ident.name.as_str();
        if regex.is_match(name) {
            return;
        }

        if matches!(
            ctx.nodes().parent_kind(node.id()),
            AstKind::PropertyDefinition(_) | AstKind::AccessorProperty(_)
        ) && !self.options().class_fields
        {
            return;
        }

        self.report_private(ctx, node.span(), name);
    }

    fn check_label_identifier<'a>(
        &self,
        ident: &LabelIdentifier<'a>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        let regex = self.regex().expect("regex should be present when should_run returns true");
        let name = ident.name.as_str();
        if regex.is_match(name) {
            return;
        }
        // Labels are not declaration names in ESLint, so `onlyDeclarations`
        // suppresses both label declarations and label references.
        if self.options().only_declarations {
            return;
        }
        self.report(ctx, node.span(), name);
    }

    fn report(&self, ctx: &LintContext, span: Span, name: &str) {
        ctx.diagnostic(id_match_diagnostic(span, name, self.pattern()));
    }

    fn report_private(&self, ctx: &LintContext, span: Span, name: &str) {
        ctx.diagnostic(id_match_private_diagnostic(span, name, self.pattern()));
    }

    fn report_in_generic_context<'a>(
        &self,
        ctx: &LintContext<'a>,
        span: Span,
        name: &str,
        effective_parent: &AstNode<'a>,
    ) {
        if self.options().only_declarations
            && !matches!(
                effective_parent.kind(),
                AstKind::Function(function)
                    if function.r#type == FunctionType::FunctionDeclaration
            )
            && !matches!(effective_parent.kind(), AstKind::VariableDeclarator(_))
        {
            return;
        }

        if matches!(effective_parent.kind(), AstKind::CallExpression(_) | AstKind::NewExpression(_))
        {
            return;
        }

        self.report(ctx, span, name);
    }

    fn handle_member_reference<'a>(
        &self,
        ident: &IdentifierReference<'a>,
        node: &AstNode<'a>,
        subject_span: Span,
        parent: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) -> bool {
        match parent.kind() {
            AstKind::StaticMemberExpression(member) => {
                if member.object.span() == subject_span && self.options().properties {
                    self.report(ctx, node.span(), ident.name.as_str());
                }
                true
            }
            AstKind::ComputedMemberExpression(member) => {
                if member.object.span() == subject_span {
                    if self.options().properties {
                        self.report(ctx, node.span(), ident.name.as_str());
                    }
                } else if member.expression.span() == subject_span
                    && self.options().properties
                    && member_is_assignment_left(parent, ctx)
                {
                    self.report(ctx, node.span(), ident.name.as_str());
                }
                true
            }
            AstKind::PrivateFieldExpression(member) => {
                if member.object.span() == subject_span && self.options().properties {
                    self.report(ctx, node.span(), ident.name.as_str());
                }
                true
            }
            _ => false,
        }
    }
}

fn is_known_external_global(ident: &IdentifierReference, ctx: &LintContext) -> bool {
    ident.is_global_reference(ctx.scoping())
        && ctx
            .get_global_variable_value(ident.name.as_str())
            .is_some_and(|value| value != GlobalValue::Off)
}

fn binding_is_import_local<'a>(ident: &BindingIdentifier<'a>, parent: &AstNode<'a>) -> bool {
    match parent.kind() {
        AstKind::ImportDefaultSpecifier(specifier) => specifier.local.span == ident.span,
        AstKind::ImportNamespaceSpecifier(specifier) => specifier.local.span == ident.span,
        AstKind::ImportSpecifier(specifier) => specifier.local.span == ident.span,
        _ => false,
    }
}

fn transparent_reference_parent<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> (&'b AstNode<'a>, Span) {
    let mut subject_span = node.span();
    for ancestor in ctx.nodes().ancestors(node.id()) {
        if matches!(
            ancestor.kind(),
            AstKind::ParenthesizedExpression(_)
                | AstKind::TSAsExpression(_)
                | AstKind::TSSatisfiesExpression(_)
                | AstKind::TSTypeAssertion(_)
                | AstKind::TSNonNullExpression(_)
                | AstKind::TSInstantiationExpression(_)
        ) {
            subject_span = ancestor.span();
            continue;
        }
        return (ancestor, subject_span);
    }
    // The Program node is always present at the top of the AST, so the loop
    // returns before exhausting the ancestor iterator in practice.
    unreachable!("ancestors iterator must yield the Program node")
}

// Collapse OXC's formal-parameter wrappers to the parent shape used by ESLint.
// Defaulted parameters intentionally stop at `FormalParameter`.
fn eslint_effective_parent_for_binding<'a, 'b>(
    parent: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> &'b AstNode<'a> {
    if let AstKind::FormalParameter(param) = parent.kind() {
        if param.initializer.is_some() {
            return parent;
        }
        let formal_params = ctx.nodes().parent_node(parent.id());
        if matches!(formal_params.kind(), AstKind::FormalParameters(_)) {
            return ctx.nodes().parent_node(formal_params.id());
        }
    }
    parent
}

// Shorthand `export { foo }` has the same source range for the local and
// exported names. Report it once, matching ESLint's range-based deduplication.
fn export_specifier_is_duplicate_clone<'a>(
    specifier: &ExportSpecifier<'a>,
    current_node: &AstNode<'a>,
) -> bool {
    if specifier.local.span() != specifier.exported.span() {
        return false;
    }
    match &specifier.exported {
        ModuleExportName::IdentifierName(inner) => inner.node_id.get() == current_node.id(),
        _ => false,
    }
}

fn member_is_assignment_left<'a>(member_node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    matches!(
        ctx.nodes().parent_kind(member_node.id()),
        AstKind::AssignmentExpression(assignment) if assignment.left.span() == member_node.span()
    )
}

fn binding_property_key_should_report(property: &BindingProperty) -> bool {
    if property.computed {
        return false;
    }
    property.shorthand || binding_property_is_exact_key_value(property)
}

fn binding_property_value_is_handled_by_key(property: &BindingProperty, ident_span: Span) -> bool {
    property.shorthand
        || matches!(
            &property.value,
            BindingPattern::AssignmentPattern(pattern)
                if pattern.left.span() == ident_span && property.key.span() == ident_span
        )
}

fn binding_property_is_exact_key_value(property: &BindingProperty) -> bool {
    let Some(key_name) = property_static_identifier_name(&property.key) else {
        return false;
    };
    binding_pattern_identifier_name(&property.value)
        .is_some_and(|value_name| value_name == key_name)
}

fn assignment_property_is_exact_key_value(property: &AssignmentTargetPropertyProperty) -> bool {
    let Some(key_name) = property_static_identifier_name(&property.name) else {
        return false;
    };
    assignment_target_identifier_name(&property.binding)
        .is_some_and(|value_name| value_name == key_name)
}

fn is_inside_object_pattern<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    for ancestor in ctx.nodes().ancestors(node.id()) {
        match ancestor.kind() {
            AstKind::ObjectPattern(_) => return true,
            AstKind::ArrayPattern(_)
            | AstKind::Function(_)
            | AstKind::Program(_)
            | AstKind::BlockStatement(_) => return false,
            _ => {}
        }
    }
    false
}

fn is_inside_object_assignment_target<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    for ancestor in ctx.nodes().ancestors(node.id()) {
        match ancestor.kind() {
            AstKind::ObjectAssignmentTarget(_) => return true,
            AstKind::ArrayAssignmentTarget(_)
            | AstKind::Function(_)
            | AstKind::Program(_)
            | AstKind::BlockStatement(_) => return false,
            _ => {}
        }
    }
    false
}

fn is_dynamic_import_attribute_object_property<'a>(
    property: &ObjectProperty<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    if property.computed {
        return false;
    }

    if property.key.is_specific_static_name("with") {
        return object_property_is_import_options_with(property, ctx);
    }

    object_property_is_inside_dynamic_import_with(property, ctx)
}

fn object_property_is_import_options_with<'a>(
    property: &ObjectProperty<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    let property_node = ctx.nodes().get_node(property.node_id.get());
    let object_node = ctx.nodes().parent_node(property_node.id());
    matches!(
        ctx.nodes().parent_kind(object_node.id()),
        AstKind::ImportExpression(import_expr)
            if import_expr.options.as_ref().is_some_and(|options| options.span() == object_node.span())
    )
}

fn object_property_is_inside_dynamic_import_with<'a>(
    property: &ObjectProperty<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    let property_node = ctx.nodes().get_node(property.node_id.get());
    let object_node = ctx.nodes().parent_node(property_node.id());
    let outer_property_node = ctx.nodes().parent_node(object_node.id());
    let AstKind::ObjectProperty(outer_property) = outer_property_node.kind() else {
        return false;
    };
    if !outer_property.key.is_specific_static_name("with")
        || outer_property.value.span() != object_node.span()
    {
        return false;
    }
    object_property_is_import_options_with(outer_property, ctx)
}

fn property_static_identifier_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(ident) => Some(ident.name.as_str()),
        _ => None,
    }
}

fn binding_pattern_identifier_name<'a>(pattern: &'a BindingPattern<'a>) -> Option<&'a str> {
    match pattern {
        BindingPattern::BindingIdentifier(ident) => Some(ident.name.as_str()),
        BindingPattern::AssignmentPattern(pattern) => {
            binding_pattern_identifier_name(&pattern.left)
        }
        _ => None,
    }
}

fn assignment_target_identifier_name<'a>(
    target: &'a AssignmentTargetMaybeDefault<'a>,
) -> Option<&'a str> {
    match target {
        AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(ident) => {
            Some(ident.name.as_str())
        }
        AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(target) => {
            assignment_target_identifier_name_from_target(&target.binding)
        }
        _ => None,
    }
}

fn assignment_target_identifier_name_from_target<'a>(
    target: &'a AssignmentTarget<'a>,
) -> Option<&'a str> {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(ident) => Some(ident.name.as_str()),
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            r#"__foo = "Matthieu""#,
            Some(serde_json::json!([ "^[a-z]+$", { "onlyDeclarations": true, }, ])),
        ),
        (r#"firstname = "Matthieu""#, Some(serde_json::json!(["^[a-z]+$"]))),
        (r#"first_name = "Matthieu""#, Some(serde_json::json!(["[a-z]+"]))),
        (r#"firstname = "Matthieu""#, Some(serde_json::json!(["^f"]))),
        (r#"last_Name = "Larcher""#, Some(serde_json::json!(["^[a-z]+(_[A-Z][a-z]+)*$"]))),
        (r#"param = "none""#, Some(serde_json::json!(["^[a-z]+(_[A-Z][a-z])*$"]))),
        ("function noUnder(){}", Some(serde_json::json!(["^[^_]+$"]))),
        ("no_under()", Some(serde_json::json!(["^[^_]+$"]))),
        ("foo.no_under2()", Some(serde_json::json!(["^[^_]+$"]))),
        ("var foo = bar.no_under3;", Some(serde_json::json!(["^[^_]+$"]))),
        ("var foo = bar.no_under4.something;", Some(serde_json::json!(["^[^_]+$"]))),
        ("foo.no_under5.qux = bar.no_under6.something;", Some(serde_json::json!(["^[^_]+$"]))),
        ("if (bar.no_under7) {}", Some(serde_json::json!(["^[^_]+$"]))),
        ("var obj = { key: foo.no_under8 };", Some(serde_json::json!(["^[^_]+$"]))),
        ("var arr = [foo.no_under9];", Some(serde_json::json!(["^[^_]+$"]))),
        ("[foo.no_under10]", Some(serde_json::json!(["^[^_]+$"]))),
        ("var arr = [foo.no_under11.qux];", Some(serde_json::json!(["^[^_]+$"]))),
        ("[foo.no_under12.nesting]", Some(serde_json::json!(["^[^_]+$"]))),
        (
            "if (foo.no_under13 === boom.no_under14) { [foo.no_under15] }",
            Some(serde_json::json!(["^[^_]+$"])),
        ),
        (
            "var myArray = new Array(); var myDate = new Date();",
            Some(serde_json::json!(["^[a-z$]+([A-Z][a-z]+)*$"])),
        ),
        ("var x = obj._foo;", Some(serde_json::json!(["^[^_]+$"]))),
        (
            "var obj = {key: no_under}",
            Some(
                serde_json::json!([ "^[^_]+$", { "properties": true, "onlyDeclarations": true, }, ]),
            ),
        ),
        (
            "var {key_no_under: key} = {}",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            "var { category_id } = query;",
            Some(
                serde_json::json!([ "^[^_]+$", { "properties": true, "ignoreDestructuring": true, }, ]),
            ),
        ),
        (
            "var { category_id: category_id } = query;",
            Some(
                serde_json::json!([ "^[^_]+$", { "properties": true, "ignoreDestructuring": true, }, ]),
            ),
        ),
        (
            "var { category_id = 1 } = query;",
            Some(
                serde_json::json!([ "^[^_]+$", { "properties": true, "ignoreDestructuring": true, }, ]),
            ),
        ),
        ("var o = {key: 1}", Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ]))),
        (
            "var o = {no_under16: 1}",
            Some(serde_json::json!([ "^[^_]+$", { "properties": false, }, ])),
        ),
        ("obj.no_under17 = 2;", Some(serde_json::json!([ "^[^_]+$", { "properties": false, }, ]))),
        (
            "var obj = { no_under18: 1 }; obj.no_under19 = 2;",
            Some(serde_json::json!([ "^[^_]+$", { "properties": false, }, ])),
        ),
        (
            "obj.no_under20 = function(){};",
            Some(serde_json::json!([ "^[^_]+$", { "properties": false, }, ])),
        ),
        ("var x = obj._foo2;", Some(serde_json::json!([ "^[^_]+$", { "properties": false, }, ]))),
        (
            "const foo = Object.keys(bar); const a = Array.from(b); const bar = () => Array;",
            Some(
                serde_json::json!([ "^\\$?[a-z]+([A-Z0-9][a-z0-9]+)*$", { "properties": true, }, ]),
            ),
        ),
        (
            "const foo = { foo_one: 1, bar_one: 2, fooBar: 3 };",
            Some(serde_json::json!([ "^[^_]+$", { "properties": false, }, ])),
        ),
        (
            "const foo = { foo_one: 1, bar_one: 2, fooBar: 3 };",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        (
            "const foo = { foo_one: 1, bar_one: 2, fooBar: 3 };",
            Some(
                serde_json::json!([ "^[^_]+$", { "properties": false, "onlyDeclarations": false, }, ]),
            ),
        ),
        (
            "const foo = { [a]: 1 };",
            Some(
                serde_json::json!([ "^[^a]", { "properties": true, "onlyDeclarations": true, }, ]),
            ),
        ),
        ("class x { foo() {} }", Some(serde_json::json!(["^[^_]+$"]))),
        ("class x { #foo() {} }", Some(serde_json::json!(["^[^_]+$"]))),
        ("class x { _foo = 1; }", Some(serde_json::json!(["^[^_]+$"]))),
        (
            "class x { _foo = 1; }",
            Some(serde_json::json!([ "^[^_]+$", { "classFields": false, }, ])),
        ),
        (
            "class x { #_foo = 1; }",
            Some(serde_json::json!([ "^[^_]+$", { "classFields": false, }, ])),
        ),
        ("class x { [_foo] = 1; }", Some(serde_json::json!(["^[^_]+$"]))),
        ("class x { foo = _foo; }", Some(serde_json::json!(["^[^_]+$"]))),
        ("class x { #_foo = 1; }", Some(serde_json::json!(["^[^_]+$"]))),
        (
            "import foo from 'foo.json' with { type: 'json' }",
            Some(serde_json::json!([ "^foo", { "properties": true, }, ])),
        ),
        (
            "export * from 'foo.json' with { type: 'json' }",
            Some(serde_json::json!([ "^foo", { "properties": true, }, ])),
        ),
        (
            "export { default } from 'foo.json' with { type: 'json' }",
            Some(serde_json::json!([ "^def", { "properties": true, }, ])),
        ),
        (
            "import('foo.json', { with: { type: 'json' } })",
            Some(serde_json::json!([ "^foo", { "properties": true, }, ])),
        ),
        (
            "import('foo.json', { 'with': { type: 'json' } })",
            Some(serde_json::json!([ "^foo", { "properties": true, }, ])),
        ),
        (
            "import('foo.json', { with: { type } })",
            Some(serde_json::json!([ "^foo", { "properties": true, }, ])),
        ),
        (
            "class Bad_Name {}",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        (
            "const f = function bad_name() {};",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        (
            "class X { bad_name() {} }",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        (
            r#"export * as bad_name from "external-module";"#,
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        ("foo[bad_name];", Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ]))),
        ("foo[bad_name]();", Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ]))),
        ("(bad_name)();", Some(serde_json::json!(["^[^_]+$"]))),
        ("new (bad_name)();", Some(serde_json::json!(["^[^_]+$"]))),
        ("foo[(bad_name)];", Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ]))),
        ("foo[bad_name] = 1;", Some(serde_json::json!([ "^[^_]+$", { "properties": false, }, ]))),
        ("bad_name.foo;", Some(serde_json::json!(["^[^_]+$"]))),
        (
            "const o = { [good]: bad_name };",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        (
            "({ bad_name } = obj);",
            Some(serde_json::json!([ "^[^_]+$", { "ignoreDestructuring": true, }, ])),
        ),
        (
            "({ bad_name: bad_name } = obj);",
            Some(serde_json::json!([ "^[^_]+$", { "ignoreDestructuring": true, }, ])),
        ),
        (
            "const [good, ...bad_rest] = arr;",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        (
            "const f = function(bad_name) {};",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        (
            "const f = (bad_name) => {};",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        (
            "function f(p = bad_name) {}",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        (
            "class X { m(bad_name) {} }",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        (
            "const goodName = 1; export { goodName as bad_name };",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        // `onlyDeclarations:true` suppresses this via the generic helper.
        (
            "const { [bad_name]: goodName } = obj;",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        // ESLint suppresses default-value bindings and RHS default refs unless
        // they become ordinary property/value checks under `properties:true`.
        ("function f(bad_name = 1) {}", Some(serde_json::json!(["^[^_]+$"]))),
        (
            "function f(bad_name = 1) {}",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        ("const [bad_name = 1] = arr;", Some(serde_json::json!(["^[^_]+$"]))),
        ("function f(p = bad_name) {}", Some(serde_json::json!(["^[^_]+$"]))),
        (
            "function f(p = bad_name) {}",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        ("const { p = bad_name } = obj;", Some(serde_json::json!(["^[^_]+$"]))),
        (
            "const { p = bad_name } = obj;",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        ("good_label: while (foo) { break good_label; }", Some(serde_json::json!(["^[a-z_]+$"]))),
        (
            "bad_label: while (foo) { break bad_label; }",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        (
            "class A { #foo; m(bad_name) { return bad_name.#foo; } }",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        (
            "class C { accessor bad_name = 1 }",
            Some(serde_json::json!([ "^[^_]+$", { "classFields": false, }, ])),
        ),
        (
            "class C { accessor #bad_name = 1 }",
            Some(serde_json::json!([ "^[^_]+$", { "classFields": false, }, ])),
        ),
        (
            "const { user_id: userId, first_name: firstName } = response;",
            Some(serde_json::json!(["^[a-z]+([A-Z][a-z]+)*$"])),
        ),
        (
            "const { user: { user_id: userId } } = response;",
            Some(serde_json::json!(["^[a-z]+([A-Z][a-z]+)*$"])),
        ),
        (
            "for (const { user_id: userId } of users) {}",
            Some(serde_json::json!(["^[a-z]+([A-Z][a-z]+)*$"])),
        ),
        (
            "try {} catch ({ error_code: errorCode }) {}",
            Some(serde_json::json!(["^[a-z]+([A-Z][a-z]+)*$"])),
        ),
        (
            "try {} catch (bad_name) {}",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        ("class C { static #bad_private = 1 }", Some(serde_json::json!(["^[^_]+$"]))),
        ("foo?.bad_name", Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ]))),
        ("class A { #foo; m() { return bad_name.#foo; } }", Some(serde_json::json!(["^[^_]+$"]))),
        (
            "class C { accessor [bad_name] = 1 }",
            Some(serde_json::json!([ "^[^_]+$", { "classFields": false, }, ])),
        ),
    ];

    let fail = vec![
        (
            r#"var __foo = "Matthieu""#,
            Some(serde_json::json!([ "^[a-z]+$", { "onlyDeclarations": true, }, ])),
        ),
        (r#"first_name = "Matthieu""#, Some(serde_json::json!(["^[a-z]+$"]))),
        (r#"first_name = "Matthieu""#, Some(serde_json::json!(["^z"]))),
        (r#"Last_Name = "Larcher""#, Some(serde_json::json!(["^[a-z]+(_[A-Z][a-z])*$"]))),
        (
            "var obj = {key: no_under}",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        ("function no_under21(){}", Some(serde_json::json!(["^[^_]+$"]))),
        (
            "obj.no_under22 = function(){};",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            "no_under23.foo = function(){};",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        ("[no_under24.baz]", Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ]))),
        (
            "if (foo.bar_baz === boom.bam_pow) { [no_under25.baz] }",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            "foo.no_under26 = boom.bam_pow",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            "var foo = { no_under27: boom.bam_pow }",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            "foo.qux.no_under28 = { bar: boom.bam_pow }",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            "var o = {no_under29: 1}",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        ("obj.no_under30 = 2;", Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ]))),
        (
            "var { category_id: category_alias } = query;",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            "var { category_id: category_alias } = query;",
            Some(
                serde_json::json!([ "^[^_]+$", { "properties": true, "ignoreDestructuring": true, }, ]),
            ),
        ),
        (
            "var { category_id: categoryId, ...other_props } = query;",
            Some(
                serde_json::json!([ "^[^_]+$", { "properties": true, "ignoreDestructuring": true, }, ]),
            ),
        ),
        (
            "var { category_id } = query;",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            "var { category_id = 1 } = query;",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            r#"import no_camelcased from "external-module";"#,
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            r#"import * as no_camelcased from "external-module";"#,
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            r#"export * as no_camelcased from "external-module";"#,
            Some(serde_json::json!(["^[^_]+$"])),
        ),
        (
            r#"import { no_camelcased } from "external-module";"#,
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            r#"import { no_camelcased as no_camel_cased } from "external module";"#,
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            r#"import { camelCased as no_camel_cased } from "external module";"#,
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            r#"import { camelCased, no_camelcased } from "external-module";"#,
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            r#"import { no_camelcased as camelCased, another_no_camelcased } from "external-module";"#,
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            r#"import camelCased, { no_camelcased } from "external-module";"#,
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            r#"import no_camelcased, { another_no_camelcased as camelCased } from "external-module";"#,
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            "function foo({ no_camelcased }) {};",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            "function foo({ no_camelcased = 'default value' }) {};",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            "const no_camelcased = 0; function foo({ camelcased_value = no_camelcased }) {}",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            "const { bar: no_camelcased } = foo;",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            "function foo({ value_1: my_default }) {}",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            "function foo({ isCamelcased: no_camelcased }) {};",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            "var { foo: bar_baz = 1 } = quz;",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            "const { no_camelcased = false } = bar;",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            "const foo_variable = 1; class MyClass {} let a = new MyClass(); let b = { id: 1 }; let c = Object.keys(b); let d = Array.from(b); let e = (Object) => Object.keys(obj, prop); let f = (Array) => Array.from(obj, prop); foo.Array = 5;",
            Some(
                serde_json::json!([ "^\\$?[a-z]+([A-Z0-9][a-z0-9]+)*$", { "properties": true, }, ]),
            ),
        ),
        ("class x { _foo() {} }", Some(serde_json::json!(["^[^_]+$"]))),
        ("class x { #_foo() {} }", Some(serde_json::json!(["^[^_]+$"]))),
        (
            "class x { _foo = 1; }",
            Some(serde_json::json!([ "^[^_]+$", { "classFields": true, }, ])),
        ),
        (
            "class x { #_foo = 1; }",
            Some(serde_json::json!([ "^[^_]+$", { "classFields": true, }, ])),
        ),
        (
            "class x { [_foo] = 1; }",
            Some(serde_json::json!([ "^[^_]+$", { "classFields": true, }, ])),
        ),
        (
            "class x { foo = _foo; }",
            Some(serde_json::json!([ "^[^_]+$", { "classFields": true, }, ])),
        ),
        (
            "const foo = { foo_one: 1, bar_one: 2, fooBar: 3 };",
            Some(
                serde_json::json!([ "^[^_]+$", { "properties": true, "onlyDeclarations": true, }, ]),
            ),
        ),
        (
            "const foo = { foo_one: 1, bar_one: 2, fooBar: 3 };",
            Some(
                serde_json::json!([ "^[^_]+$", { "properties": true, "onlyDeclarations": false, }, ]),
            ),
        ),
        (
            "const foo = { [a]: 1 };",
            Some(
                serde_json::json!([ "^[^a]", { "properties": true, "onlyDeclarations": false, }, ]),
            ),
        ),
        (
            "const foo = { [a]: 1 };",
            Some(
                serde_json::json!([ "^[^a]", { "properties": false, "onlyDeclarations": false, }, ]),
            ),
        ),
        (
            "import('foo.json', { with: { [type]: 'json' } })",
            Some(serde_json::json!([ "^foo", { "properties": true, }, ])),
        ),
        (
            "import('foo.json', { with: { type: json } })",
            Some(serde_json::json!([ "^foo", { "properties": true, }, ])),
        ),
        (
            r#"import bad_name from "external-module";"#,
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        (
            "class X { #bad_name() {} }",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        (
            "class X { bad_name = 1 }",
            Some(
                serde_json::json!([ "^[^_]+$", { "classFields": true, "onlyDeclarations": true, }, ]),
            ),
        ),
        ("foo[bad_name] = 1;", Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ]))),
        ("bad_name[foo];", Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ]))),
        ("const o = { [good]: bad_name };", Some(serde_json::json!(["^[^_]+$"]))),
        ("({ bad_name } = obj);", Some(serde_json::json!(["^[^_]+$"]))),
        (
            "({ goodName: bad_name } = obj);",
            Some(serde_json::json!([ "^[^_]+$", { "ignoreDestructuring": true, }, ])),
        ),
        ("const [good, ...bad_rest] = arr;", Some(serde_json::json!(["^[^_]+$"]))),
        (
            "function f(bad_name) {}",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        (
            "const goodName = 1; export { goodName as bad_name };",
            Some(serde_json::json!(["^[^_]+$"])),
        ),
        (r#"export { bad_name } from "mod";"#, Some(serde_json::json!(["^[^_]+$"]))),
        (r#"export { goodName as bad_name } from "mod";"#, Some(serde_json::json!(["^[^_]+$"]))),
        // Intentionally stricter than ESLint: computed destructuring keys use a
        // local reference, so binding and assignment forms both report.
        ("const { [bad_name]: goodName } = obj;", Some(serde_json::json!(["^[^_]+$"]))),
        (
            "function f(bad_name = 1) {}",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        (
            "const [bad_name = 1] = arr;",
            Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ])),
        ),
        // Shorthand export reports once; explicit `as bad_name` still reports
        // both ranges and is locked in by the snapshot.
        ("const bad_name = 1; export { bad_name };", Some(serde_json::json!(["^[^_]+$"]))),
        (
            "const bad_name = 1; export { bad_name as bad_name };",
            Some(serde_json::json!(["^[^_]+$"])),
        ),
        (
            "const bad_name = 1; export { bad_name as goodName };",
            Some(serde_json::json!(["^[^_]+$"])),
        ),
        ("bad_label: while (foo) { break bad_label; }", Some(serde_json::json!(["^[^_]+$"]))),
        // Intentionally stricter than ESLint/origin: keys outside the inner
        // `with` attribute object are still ordinary user-controlled names.
        (
            r#"import("x.json", { bad_option: 1 })"#,
            Some(serde_json::json!([ "^foo", { "properties": true, }, ])),
        ),
        (
            "class A { #foo; m(bad_name) { return bad_name.#foo; } }",
            Some(
                serde_json::json!([ "^[^_]+$", { "properties": true, "onlyDeclarations": true, }, ]),
            ),
        ),
        (
            "class C { accessor bad_name = 1 }",
            Some(serde_json::json!([ "^[^_]+$", { "classFields": true, }, ])),
        ),
        (
            "class C { accessor #bad_name = 1 }",
            Some(serde_json::json!([ "^[^_]+$", { "classFields": true, }, ])),
        ),
        ("({ [bad_name]: value } = obj);", Some(serde_json::json!(["^[^_]+$"]))),
        (
            "const { [bad_name]: goodName } = obj;",
            Some(serde_json::json!([ "^[^_]+$", { "ignoreDestructuring": true, }, ])),
        ),
        ("const { user: { bad_name } } = response;", Some(serde_json::json!(["^[^_]+$"]))),
        ("for (const { bad_name } of users) {}", Some(serde_json::json!(["^[^_]+$"]))),
        ("try {} catch (bad_name) {}", Some(serde_json::json!(["^[^_]+$"]))),
        ("function* bad_gen() {}", Some(serde_json::json!(["^[^_]+$"]))),
        ("async function bad_async() {}", Some(serde_json::json!(["^[^_]+$"]))),
        (
            "class C { static #bad_private = 1 }",
            Some(serde_json::json!([ "^[^_]+$", { "classFields": true, }, ])),
        ),
        ("const o = { [bad_name + suffix]: 1 }", Some(serde_json::json!(["^[^_]+$"]))),
        (
            "class C { accessor [bad_name] = 1 }",
            Some(serde_json::json!([ "^[^_]+$", { "classFields": true, }, ])),
        ),
        (
            "({ [bad_name]: value } = obj);",
            Some(serde_json::json!([ "^[^_]+$", { "ignoreDestructuring": true, }, ])),
        ),
    ];

    Tester::new(IdMatch::NAME, IdMatch::PLUGIN, pass, fail).test_and_snapshot();
}

#[test]
fn test_typescript() {
    use crate::tester::Tester;

    // Minimal TS policy: TS-only declarations fall through to the generic
    // helper and are suppressed by `onlyDeclarations:true`.
    let pass = vec![
        (
            "interface FooBar {}",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        (
            "type FooBar = string;",
            Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ])),
        ),
        ("enum FooBar {}", Some(serde_json::json!([ "^[^_]+$", { "onlyDeclarations": true, }, ]))),
        ("(bad_name as Foo).foo", Some(serde_json::json!(["^[^_]+$"]))),
        ("bad_name!.foo", Some(serde_json::json!(["^[^_]+$"]))),
    ];

    let fail = vec![
        ("interface Bad_Name {}", Some(serde_json::json!(["^[^_]+$"]))),
        ("type Bad_Name = string;", Some(serde_json::json!(["^[^_]+$"]))),
        ("enum Bad_Name {}", Some(serde_json::json!(["^[^_]+$"]))),
        ("(bad_name as Foo).foo", Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ]))),
        ("bad_name!.foo", Some(serde_json::json!([ "^[^_]+$", { "properties": true, }, ]))),
    ];

    Tester::new(IdMatch::NAME, IdMatch::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .with_snapshot_suffix("ts")
        .test_and_snapshot();
}

#[test]
fn invalid_configs_error_in_from_configuration() {
    // ESLint's id-match JSON schema requires a string pattern, a closed options
    // object, and at most two tuple elements. Rust regex additionally rejects
    // JavaScript-only features such as lookaround.
    for config in [
        serde_json::json!(["(?<=x)y"]),
        serde_json::json!([123]),
        serde_json::json!([null]),
        serde_json::json!([null, {}]),
        serde_json::json!(["^foo", null]),
        serde_json::json!(["^foo", { "unknown": true }]),
        serde_json::json!(["^foo", {}, "extra"]),
    ] {
        assert!(
            IdMatch::from_configuration(config.clone()).is_err(),
            "expected error for config: {config}"
        );
    }
}

#[test]
fn empty_array_uses_default_no_pattern() {
    let cfg = IdMatch::from_configuration(serde_json::json!([]))
        .expect("[] should deserialize as the default config");
    assert!(cfg.regex().is_none());
}
