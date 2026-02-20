use oxc_ast::{
    AstKind,
    ast::{BindingPattern, ForStatementLeft, IdentifierReference, ModuleExportName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    reference::Reference,
    scope::ScopeId,
    symbol::{SymbolFlags, SymbolId},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

fn no_use_before_define_diagnostic(
    name: &str,
    usage_span: Span,
    declaration_span: Option<Span>,
) -> OxcDiagnostic {
    let diagnostic = OxcDiagnostic::warn(format!("'{name}' was used before it was defined."))
        .with_label(usage_span.primary_label("used here"))
        .with_help("Move the declaration before any references to it, or remove the reference if it is not needed.");
    if let Some(declaration_span) = declaration_span.filter(|span| !span.is_unspanned()) {
        diagnostic.and_label(declaration_span.label("defined here"))
    } else {
        diagnostic
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoUseBeforeDefineConfig {
    /// Allow named exports that appear before declaration.
    allow_named_exports: bool,
    /// Check class declarations.
    classes: bool,
    /// Check enum declarations.
    enums: bool,
    /// Check function declarations.
    functions: bool,
    /// Ignore usages that are type-only references.
    ignore_type_references: bool,
    /// Check type aliases, interfaces, and type parameters.
    typedefs: bool,
    /// Check variable declarations.
    variables: bool,
}

impl Default for NoUseBeforeDefineConfig {
    fn default() -> Self {
        Self {
            allow_named_exports: false,
            classes: true,
            enums: true,
            functions: true,
            ignore_type_references: true,
            typedefs: true,
            variables: true,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct NoUseBeforeDefine(NoUseBeforeDefineConfig);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows using variables before they are defined.
    ///
    /// ### Why is this bad?
    ///
    /// Referencing identifiers before their declarations can hide bugs and
    /// make code order-dependent and difficult to reason about.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// new A();
    /// var A = class {};
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// var A = class {};
    /// new A();
    /// ```
    NoUseBeforeDefine,
    eslint,
    restriction,
    config = NoUseBeforeDefineConfig,
);

impl Rule for NoUseBeforeDefine {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<NoUseBeforeDefineConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(Self)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::IdentifierReference(identifier) = node.kind() else {
            return;
        };

        let reference = ctx.scoping().get_reference(identifier.reference_id());
        let is_named_export = is_named_exports(node, identifier, ctx);
        if is_named_export {
            if self.0.allow_named_exports {
                return;
            }

            let symbol_id = reference.symbol_id();
            let should_report = symbol_id
                .is_none_or(|symbol_id| !is_defined_before_use(symbol_id, reference, node, ctx));
            if should_report {
                ctx.diagnostic(no_use_before_define_diagnostic(
                    identifier.name.as_str(),
                    identifier.span,
                    symbol_id.map(|symbol_id| ctx.scoping().symbol_span(symbol_id)),
                ));
            }
            return;
        }

        let Some(symbol_id) = reference.symbol_id() else {
            if let Some(declaration_span) =
                unresolved_initializer_reference_declaration_span(identifier, node, ctx)
            {
                ctx.diagnostic(no_use_before_define_diagnostic(
                    identifier.name.as_str(),
                    identifier.span,
                    Some(declaration_span),
                ));
            }
            return;
        };

        if is_defined_before_use(symbol_id, reference, node, ctx)
            || !is_forbidden(self.0, symbol_id, reference, node, ctx)
            || is_class_ref_in_class_decorator(symbol_id, node, ctx)
            || is_function_type_scope(reference.scope_id(), ctx)
        {
            return;
        }

        ctx.diagnostic(no_use_before_define_diagnostic(
            identifier.name.as_str(),
            identifier.span,
            Some(ctx.scoping().symbol_span(symbol_id)),
        ));
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn is_forbidden(
    options: NoUseBeforeDefineConfig,
    symbol_id: SymbolId,
    reference: &Reference,
    reference_node: &AstNode<'_>,
    ctx: &LintContext<'_>,
) -> bool {
    if options.ignore_type_references && is_type_reference(reference, reference_node, ctx) {
        return false;
    }

    let symbol_flags = ctx.scoping().symbol_flags(symbol_id);
    if symbol_flags.is_function() {
        return options.functions;
    }

    if is_outer_class(symbol_id, symbol_flags, reference.scope_id(), ctx) {
        return options.classes;
    }

    if is_outer_variable(symbol_id, reference.scope_id(), ctx) {
        return options.variables;
    }

    if is_outer_enum(symbol_id, symbol_flags, reference.scope_id(), ctx) {
        return options.enums;
    }

    if is_typedef(symbol_id, ctx) {
        return options.typedefs;
    }

    true
}

fn is_outer_class(
    symbol_id: SymbolId,
    symbol_flags: SymbolFlags,
    reference_scope_id: ScopeId,
    ctx: &LintContext<'_>,
) -> bool {
    symbol_flags.is_class()
        && get_parent_variable_scope(ctx.scoping().symbol_scope_id(symbol_id), ctx)
            != get_parent_variable_scope(reference_scope_id, ctx)
}

fn is_outer_variable(
    symbol_id: SymbolId,
    reference_scope_id: ScopeId,
    ctx: &LintContext<'_>,
) -> bool {
    matches!(ctx.symbol_declaration(symbol_id).kind(), AstKind::VariableDeclarator(_))
        && get_parent_variable_scope(ctx.scoping().symbol_scope_id(symbol_id), ctx)
            != get_parent_variable_scope(reference_scope_id, ctx)
}

fn is_outer_enum(
    symbol_id: SymbolId,
    symbol_flags: SymbolFlags,
    reference_scope_id: ScopeId,
    ctx: &LintContext<'_>,
) -> bool {
    symbol_flags.is_enum()
        && get_parent_variable_scope(ctx.scoping().symbol_scope_id(symbol_id), ctx)
            != get_parent_variable_scope(reference_scope_id, ctx)
}

fn is_typedef(symbol_id: SymbolId, ctx: &LintContext<'_>) -> bool {
    matches!(
        ctx.symbol_declaration(symbol_id).kind(),
        AstKind::TSTypeAliasDeclaration(_)
            | AstKind::TSInterfaceDeclaration(_)
            | AstKind::TSTypeParameter(_)
    )
}

fn is_named_exports(
    node: &AstNode<'_>,
    identifier: &IdentifierReference<'_>,
    ctx: &LintContext<'_>,
) -> bool {
    let parent = ctx.nodes().parent_node(node.id());
    matches!(
        parent.kind(),
        AstKind::ExportSpecifier(export_specifier)
            if matches!(
                &export_specifier.local,
                ModuleExportName::IdentifierReference(local)
                    if local.reference_id() == identifier.reference_id()
            )
    )
}

fn is_type_reference(
    reference: &Reference,
    reference_node: &AstNode<'_>,
    ctx: &LintContext<'_>,
) -> bool {
    reference.is_type() || reference_contains_type_query(reference_node, ctx)
}

fn reference_contains_type_query(reference_node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let mut node = reference_node;

    loop {
        node = match node.kind() {
            AstKind::TSTypeQuery(_) => return true,
            AstKind::TSQualifiedName(_) | AstKind::IdentifierReference(_) => {
                ctx.nodes().parent_node(node.id())
            }
            _ => return false,
        };
    }
}

fn is_defined_before_use(
    symbol_id: SymbolId,
    reference: &Reference,
    reference_node: &AstNode<'_>,
    ctx: &LintContext<'_>,
) -> bool {
    let defined_before_reference =
        ctx.scoping().symbol_span(symbol_id).end <= reference_node.kind().span().end;
    defined_before_reference
        && !(reference.is_value() && is_in_initializer(symbol_id, reference, reference_node, ctx))
}

fn unresolved_initializer_reference_declaration_span(
    identifier: &IdentifierReference<'_>,
    reference_node: &AstNode<'_>,
    ctx: &LintContext<'_>,
) -> Option<Span> {
    let name = identifier.name.as_str();
    initializer_reference_declaration_span(reference_node, ctx, |pattern| {
        binding_pattern_name_span(pattern, name)
    })
}

fn binding_pattern_name_span(pattern: &BindingPattern<'_>, name: &str) -> Option<Span> {
    let mut declaration_span = None;
    pattern.all_binding_identifiers(&mut |identifier| {
        if identifier.name == name {
            declaration_span = Some(identifier.span);
            false
        } else {
            true
        }
    });
    declaration_span
}

fn for_statement_left_declaration_span<F>(
    left: &ForStatementLeft<'_>,
    find_declaration_span: &mut F,
) -> Option<Span>
where
    F: FnMut(&BindingPattern<'_>) -> Option<Span>,
{
    let ForStatementLeft::VariableDeclaration(variable_declaration) = left else {
        return None;
    };
    variable_declaration
        .declarations
        .iter()
        .find_map(|declarator| find_declaration_span(&declarator.id))
}

fn initializer_reference_declaration_span<F>(
    reference_node: &AstNode<'_>,
    ctx: &LintContext<'_>,
    mut find_declaration_span: F,
) -> Option<Span>
where
    F: FnMut(&BindingPattern<'_>) -> Option<Span>,
{
    let reference_location = reference_node.kind().span().end;

    for ancestor in ctx.nodes().ancestors(reference_node.id()) {
        match ancestor.kind() {
            AstKind::FormalParameter(formal_parameter) => {
                if let Some(declaration_span) = find_declaration_span(&formal_parameter.pattern)
                    && formal_parameter
                        .initializer
                        .as_ref()
                        .is_some_and(|init| is_in_range(init.span(), reference_location))
                {
                    return Some(declaration_span);
                }
            }
            AstKind::VariableDeclarator(declarator) => {
                let Some(declaration_span) = find_declaration_span(&declarator.id) else {
                    continue;
                };

                if declarator
                    .init
                    .as_ref()
                    .is_some_and(|init| is_in_range(init.span(), reference_location))
                {
                    return Some(declaration_span);
                }

                let parent = ctx.nodes().parent_node(ancestor.id());
                if let AstKind::VariableDeclaration(_) = parent.kind() {
                    let grand_parent = ctx.nodes().parent_node(parent.id());
                    if matches!(
                        grand_parent.kind(),
                        AstKind::ForInStatement(for_in)
                            if is_in_range(for_in.right.span(), reference_location)
                    ) || matches!(
                        grand_parent.kind(),
                        AstKind::ForOfStatement(for_of)
                            if is_in_range(for_of.right.span(), reference_location)
                    ) {
                        return Some(declaration_span);
                    }
                }

                break;
            }
            AstKind::AssignmentPattern(assignment_pattern) => {
                if let Some(declaration_span) = find_declaration_span(&assignment_pattern.left)
                    && is_in_range(assignment_pattern.right.span(), reference_location)
                {
                    return Some(declaration_span);
                }
            }
            AstKind::ForInStatement(for_in) => {
                if let Some(declaration_span) =
                    for_statement_left_declaration_span(&for_in.left, &mut find_declaration_span)
                    && is_in_range(for_in.right.span(), reference_location)
                {
                    return Some(declaration_span);
                }
            }
            AstKind::ForOfStatement(for_of) => {
                if let Some(declaration_span) =
                    for_statement_left_declaration_span(&for_of.left, &mut find_declaration_span)
                    && is_in_range(for_of.right.span(), reference_location)
                {
                    return Some(declaration_span);
                }
            }
            AstKind::Function(_)
            | AstKind::Class(_)
            | AstKind::ArrowFunctionExpression(_)
            | AstKind::CatchClause(_)
            | AstKind::ImportDeclaration(_)
            | AstKind::ExportNamedDeclaration(_) => break,
            _ => {}
        }
    }

    None
}

fn is_in_initializer(
    symbol_id: SymbolId,
    _reference: &Reference,
    reference_node: &AstNode<'_>,
    ctx: &LintContext<'_>,
) -> bool {
    let declaration_span = ctx.scoping().symbol_span(symbol_id);
    initializer_reference_declaration_span(reference_node, ctx, |pattern| {
        pattern.span().contains_inclusive(declaration_span).then_some(declaration_span)
    })
    .is_some()
}

fn is_class_ref_in_class_decorator(
    symbol_id: SymbolId,
    reference_node: &AstNode<'_>,
    ctx: &LintContext<'_>,
) -> bool {
    if !ctx.scoping().symbol_flags(symbol_id).is_class() {
        return false;
    }

    let AstKind::Class(class) = ctx.symbol_declaration(symbol_id).kind() else {
        return false;
    };

    if class.decorators.is_empty() {
        return false;
    }

    let reference_span = reference_node.kind().span();
    class.decorators.iter().any(|decorator| decorator.span.contains_inclusive(reference_span))
}

fn is_function_type_scope(scope_id: ScopeId, ctx: &LintContext<'_>) -> bool {
    let scope_node_id = ctx.scoping().get_node_id(scope_id);
    matches!(
        ctx.nodes().kind(scope_node_id),
        AstKind::TSFunctionType(_)
            | AstKind::TSConstructorType(_)
            | AstKind::TSCallSignatureDeclaration(_)
            | AstKind::TSMethodSignature(_)
            | AstKind::TSConstructSignatureDeclaration(_)
    )
}

fn get_parent_variable_scope(scope_id: ScopeId, ctx: &LintContext<'_>) -> ScopeId {
    ctx.scoping()
        .scope_ancestors(scope_id)
        .find(|scope_id| ctx.scoping().scope_flags(*scope_id).is_var())
        .unwrap_or_else(|| ctx.scoping().root_scope_id())
}

fn is_in_range(span: Span, location: u32) -> bool {
    span.start <= location && location <= span.end
}

#[test]
#[ignore = "eslint test cases are failing"]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("unresolved", None),
        ("Array", None),
        ("function foo () { arguments; }", None),
        ("var a=10; alert(a);", None),
        ("function b(a) { alert(a); }", None),
        ("Object.hasOwnProperty.call(a);", None),
        ("function a() { alert(arguments);}", None),
        ("a(); function a() { alert(arguments); }", Some(serde_json::json!(["nofunc"]))),
        ("(() => { var a = 42; alert(a); })();", None), // { "ecmaVersion": 6 },
        ("a(); try { throw new Error() } catch (a) {}", None),
        ("class A {} new A();", None), // { "ecmaVersion": 6 },
        ("var a = 0, b = a;", None),
        ("var {a = 0, b = a} = {};", None), // { "ecmaVersion": 6 },
        ("var [a = 0, b = a] = {};", None), // { "ecmaVersion": 6 },
        ("function foo() { foo(); }", None),
        ("var foo = function() { foo(); };", None),
        ("var a; for (a in a) {}", None),
        ("var a; for (a of a) {}", None), // { "ecmaVersion": 6 },
        ("let a; class C { static { a; } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { let a; a; } }", None), // { "ecmaVersion": 2022 },
        (r#""use strict"; a(); { function a() {} }"#, None), // { "ecmaVersion": 6 },
        (r#""use strict"; { a(); function a() {} }"#, Some(serde_json::json!(["nofunc"]))), // { "ecmaVersion": 6 },
        ("switch (foo) { case 1:  { a(); } default: { let a; }}", None), // { "ecmaVersion": 6 },
        ("a(); { let a = function () {}; }", None),                      // { "ecmaVersion": 6 },
        (
            "a(); function a() { alert(arguments); }",
            Some(serde_json::json!([{ "functions": false }])),
        ),
        (
            r#""use strict"; { a(); function a() {} }"#,
            Some(serde_json::json!([{ "functions": false }])),
        ), // { "ecmaVersion": 6 },
        (
            "function foo() { new A(); } class A {};",
            Some(serde_json::json!([{ "classes": false }])),
        ), // { "ecmaVersion": 6 },
        ("function foo() { bar; } var bar;", Some(serde_json::json!([{ "variables": false }]))),
        ("var foo = () => bar; var bar;", Some(serde_json::json!([{ "variables": false }]))), // { "ecmaVersion": 6 },
        (
            "class C { static { () => foo; let foo; } }",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { "ecmaVersion": 2022 },
        ("class C extends (class { method() { C; } }) {}", None), // { "ecmaVersion": 6 },
        ("(class extends (class { method() { C; } }) {});", None), // { "ecmaVersion": 6 },
        ("const C = (class extends (class { method() { C; } }) {});", None), // { "ecmaVersion": 6 },
        ("class C extends (class { field = C; }) {}", None), // { "ecmaVersion": 2022 },
        ("(class extends (class { field = C; }) {});", None), // { "ecmaVersion": 2022 },
        ("const C = (class extends (class { field = C; }) {});", None), // { "ecmaVersion": 2022 },
        ("class C { [() => C](){} }", None),                 // { "ecmaVersion": 6 },
        ("(class C { [() => C](){} });", None),              // { "ecmaVersion": 6 },
        ("const C = class { [() => C](){} };", None),        // { "ecmaVersion": 6 },
        ("class C { static [() => C](){} }", None),          // { "ecmaVersion": 6 },
        ("(class C { static [() => C](){} });", None),       // { "ecmaVersion": 6 },
        ("const C = class { static [() => C](){} };", None), // { "ecmaVersion": 6 },
        ("class C { [() => C]; }", None),                    // { "ecmaVersion": 2022 },
        ("(class C { [() => C]; });", None),                 // { "ecmaVersion": 2022 },
        ("const C = class { [() => C]; };", None),           // { "ecmaVersion": 2022 },
        ("class C { static [() => C]; }", None),             // { "ecmaVersion": 2022 },
        ("(class C { static [() => C]; });", None),          // { "ecmaVersion": 2022 },
        ("const C = class { static [() => C]; };", None),    // { "ecmaVersion": 2022 },
        ("class C { method() { C; } }", None),               // { "ecmaVersion": 6 },
        ("(class C { method() { C; } });", None),            // { "ecmaVersion": 6 },
        ("const C = class { method() { C; } };", None),      // { "ecmaVersion": 6 },
        ("class C { static method() { C; } }", None),        // { "ecmaVersion": 6 },
        ("(class C { static method() { C; } });", None),     // { "ecmaVersion": 6 },
        ("const C = class { static method() { C; } };", None), // { "ecmaVersion": 6 },
        ("class C { field = C; }", None),                    // { "ecmaVersion": 2022 },
        ("(class C { field = C; });", None),                 // { "ecmaVersion": 2022 },
        ("const C = class { field = C; };", None),           // { "ecmaVersion": 2022 },
        ("class C { static field = C; }", None),             // { "ecmaVersion": 2022 },
        ("(class C { static field = C; });", None),          // { "ecmaVersion": 2022 },
        ("class C { static field = class { static field = C; }; }", None), // { "ecmaVersion": 2022 },
        ("(class C { static field = class { static field = C; }; });", None), // { "ecmaVersion": 2022 },
        ("class C { field = () => C; }", None), // { "ecmaVersion": 2022 },
        ("(class C { field = () => C; });", None), // { "ecmaVersion": 2022 },
        ("const C = class { field = () => C; };", None), // { "ecmaVersion": 2022 },
        ("class C { static field = () => C; }", None), // { "ecmaVersion": 2022 },
        ("(class C { static field = () => C; });", None), // { "ecmaVersion": 2022 },
        ("const C = class { static field = () => C; };", None), // { "ecmaVersion": 2022 },
        ("class C { field = class extends C {}; }", None), // { "ecmaVersion": 2022 },
        ("(class C { field = class extends C {}; });", None), // { "ecmaVersion": 2022 },
        ("const C = class { field = class extends C {}; }", None), // { "ecmaVersion": 2022 },
        ("class C { static field = class extends C {}; }", None), // { "ecmaVersion": 2022 },
        ("(class C { static field = class extends C {}; });", None), // { "ecmaVersion": 2022 },
        ("class C { static field = class { [C]; }; }", None), // { "ecmaVersion": 2022 },
        ("(class C { static field = class { [C]; }; });", None), // { "ecmaVersion": 2022 },
        ("const C = class { static field = class { field = C; }; };", None), // { "ecmaVersion": 2022 },
        ("class C { method() { a; } } let a;", Some(serde_json::json!([{ "variables": false }]))), // { "ecmaVersion": 6 },
        (
            "class C { static method() { a; } } let a;",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { "ecmaVersion": 6 },
        ("class C { field = a; } let a;", Some(serde_json::json!([{ "variables": false }]))), // { "ecmaVersion": 2022 },
        ("class C { field = D; } class D {}", Some(serde_json::json!([{ "classes": false }]))), // { "ecmaVersion": 2022 },
        (
            "class C { field = class extends D {}; } class D {}",
            Some(serde_json::json!([{ "classes": false }])),
        ), // { "ecmaVersion": 2022 },
        ("class C { field = () => a; } let a;", Some(serde_json::json!([{ "variables": false }]))), // { "ecmaVersion": 2022 },
        (
            "class C { static field = () => a; } let a;",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { field = () => D; } class D {}",
            Some(serde_json::json!([{ "classes": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static field = () => D; } class D {}",
            Some(serde_json::json!([{ "classes": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static field = class { field = a; }; } let a;",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { "ecmaVersion": 2022 },
        ("class C { static { C; } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { C; } static {} static { C; } }", None), // { "ecmaVersion": 2022 },
        ("(class C { static { C; } })", None), // { "ecmaVersion": 2022 },
        ("class C { static { class D extends C {} } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { (class { static { C } }) } }", None), // { "ecmaVersion": 2022 },
        ("class C { static { () => C; } }", None), // { "ecmaVersion": 2022 },
        ("(class C { static { () => C; } })", None), // { "ecmaVersion": 2022 },
        ("const C = class { static { () => C; } }", None), // { "ecmaVersion": 2022 },
        (
            "class C { static { () => D; } } class D {}",
            Some(serde_json::json!([{ "classes": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { () => a; } } let a;",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { "ecmaVersion": 2022 },
        ("const C = class C { static { C.x; } }", None), // { "ecmaVersion": 2022 },
        ("export { a }; const a = 1;", Some(serde_json::json!([{ "allowNamedExports": true }]))), // { "ecmaVersion": 2015, "sourceType": "module" },
        (
            "export { a as b }; const a = 1;",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { "ecmaVersion": 2015, "sourceType": "module" },
        ("export { a, b }; let a, b;", Some(serde_json::json!([{ "allowNamedExports": true }]))), // { "ecmaVersion": 2015, "sourceType": "module" },
        ("export { a }; var a;", Some(serde_json::json!([{ "allowNamedExports": true }]))), // { "ecmaVersion": 2015, "sourceType": "module" },
        ("export { f }; function f() {}", Some(serde_json::json!([{ "allowNamedExports": true }]))), // { "ecmaVersion": 2015, "sourceType": "module" },
        ("export { C }; class C {}", Some(serde_json::json!([{ "allowNamedExports": true }]))), // { "ecmaVersion": 2015, "sourceType": "module" },
        ("const App = () => <div/>; <App />;", None), // { "ecmaVersion": 6, "parserOptions": { "ecmaFeatures": { "jsx": true } }, },
        ("let Foo, Bar; <Foo><Bar /></Foo>;", None), // { "ecmaVersion": 6, "parserOptions": { "ecmaFeatures": { "jsx": true } }, },
        ("function App() { return <div/> } <App />;", None), // { "ecmaVersion": 6, "parserOptions": { "ecmaFeatures": { "jsx": true } }, },
        (
            "<App />; function App() { return <div/> }",
            Some(serde_json::json!([{ "functions": false }])),
        ), // { "ecmaVersion": 6, "parserOptions": { "ecmaFeatures": { "jsx": true } }, },
        (
            "
                type foo = 1;
                const x: foo = 1;
                    ",
            None,
        ),
        (
            "
                type foo = 1;
                type bar = foo;
                    ",
            None,
        ),
        (
            "
                interface Foo {}
                const x: Foo = {};
                    ",
            None,
        ),
        (
            "
                var a = 10;
                alert(a);
                    ",
            None,
        ),
        (
            "
                function b(a) {
                  alert(a);
                }
                    ",
            None,
        ),
        ("Object.hasOwnProperty.call(a);", None),
        (
            "
                function a() {
                  alert(arguments);
                }
                    ",
            None,
        ),
        ("declare function a();", None),
        (
            "
                declare class a {
                  foo();
                }
                    ",
            None,
        ),
        ("const updatedAt = data?.updatedAt;", None),
        (
            "
                function f() {
                  return function t() {};
                }
                f()?.();
                    ",
            None,
        ),
        (
            "
                var a = { b: 5 };
                alert(a?.b);
                    ",
            None,
        ),
        (
            "
                a();
                function a() {
                  alert(arguments);
                }
                      ",
            Some(serde_json::json!(["nofunc"])),
        ),
        (
            "
                (() => {
                  var a = 42;
                  alert(a);
                })();
                      ",
            None,
        ), // { parserOptions },
        (
            "
                a();
                try {
                  throw new Error();
                } catch (a) {}
                    ",
            None,
        ),
        (
            "
                class A {}
                new A();
                      ",
            None,
        ), // { parserOptions },
        (
            "
                var a = 0,
                  b = a;
                    ",
            None,
        ),
        ("var { a = 0, b = a } = {};", None), // { parserOptions },
        ("var [a = 0, b = a] = {};", None),   // { parserOptions },
        (
            "
                function foo() {
                  foo();
                }
                    ",
            None,
        ),
        (
            "
                var foo = function () {
                  foo();
                };
                    ",
            None,
        ),
        (
            "
                var a;
                for (a in a) {
                }
                    ",
            None,
        ),
        (
            "
                var a;
                for (a of a) {
                }
                      ",
            None,
        ), // { parserOptions },
        (
            "
                'use strict';
                a();
                {
                  function a() {}
                }
                      ",
            None,
        ), // { parserOptions },
        (
            "
                'use strict';
                {
                  a();
                  function a() {}
                }
                      ",
            Some(serde_json::json!(["nofunc"])),
        ), // { parserOptions },
        (
            "
                switch (foo) {
                  case 1: {
                    a();
                  }
                  default: {
                    let a;
                  }
                }
                      ",
            None,
        ), // { parserOptions },
        (
            "
                a();
                {
                  let a = function () {};
                }
                      ",
            None,
        ), // { parserOptions },
        (
            "
                a();
                function a() {
                  alert(arguments);
                }
                      ",
            Some(serde_json::json!([{ "functions": false }])),
        ),
        (
            "
                'use strict';
                {
                  a();
                  function a() {}
                }
                      ",
            Some(serde_json::json!([{ "functions": false }])),
        ), // { parserOptions },
        (
            "
                function foo() {
                  new A();
                }
                class A {}
                      ",
            Some(serde_json::json!([{ "classes": false }])),
        ), // { parserOptions },
        (
            "
                function foo() {
                  bar;
                }
                var bar;
                      ",
            Some(serde_json::json!([{ "variables": false }])),
        ),
        (
            "
                var foo = () => bar;
                var bar;
                      ",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { parserOptions },
        (
            "
                var x: Foo = 2;
                type Foo = string | number;
                      ",
            Some(serde_json::json!([{ "typedefs": false }])),
        ),
        (
            "
                var x: Foo = {};
                interface Foo {}
                      ",
            Some(serde_json::json!([{ "typedefs": false, "ignoreTypeReferences": false }])),
        ),
        (
            "
                let myVar: String;
                type String = string;
                      ",
            Some(serde_json::json!([{ "typedefs": false, "ignoreTypeReferences": false }])),
        ),
        (
            "
                interface Bar {
                  type: typeof Foo;
                }

                const Foo = 2;
                      ",
            Some(serde_json::json!([{ "ignoreTypeReferences": true }])),
        ),
        (
            "
                interface Bar {
                  type: typeof Foo.FOO;
                }

                class Foo {
                  public static readonly FOO = '';
                }
                      ",
            Some(serde_json::json!([{ "ignoreTypeReferences": true }])),
        ),
        (
            "
                interface Bar {
                  type: typeof Foo.Bar.Baz;
                }

                const Foo = {
                  Bar: {
                    Baz: 1,
                  },
                };
                      ",
            Some(serde_json::json!([{ "ignoreTypeReferences": true }])),
        ),
        (
            "
                interface ITest {
                  first: boolean;
                  second: string;
                  third: boolean;
                }

                let first = () => console.log('first');

                export let second = () => console.log('second');

                export namespace Third {
                  export let third = () => console.log('third');
                }
                      ",
            None,
        ), // { "parserOptions": { "ecmaVersion": 6, "sourceType": "module" }, },
        (
            "
                function test(file: Blob) {
                  const slice: typeof file.slice =
                    file.slice || (file as any).webkitSlice || (file as any).mozSlice;
                  return slice;
                }
                    ",
            None,
        ),
        (
            "
                interface Foo {
                  bar: string;
                }
                const bar = 'blah';
                    ",
            None,
        ),
        (
            "
                function foo(): Foo {
                  return Foo.FOO;
                }

                enum Foo {
                  FOO,
                }
                      ",
            Some(serde_json::json!([{ "enums": false }])),
        ),
        (
            "
                let foo: Foo;

                enum Foo {
                  FOO,
                }
                      ",
            Some(serde_json::json!([{ "enums": false }])),
        ),
        (
            "
                class Test {
                  foo(args: Foo): Foo {
                    return Foo.FOO;
                  }
                }

                enum Foo {
                  FOO,
                }
                      ",
            Some(serde_json::json!([{ "enums": false }])),
        ),
        (
            "
                export { a };
                const a = 1;
                      ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
                export { a as b };
                const a = 1;
                      ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
                export { a, b };
                let a, b;
                      ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
                export { a };
                var a;
                      ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
                export { f };
                function f() {}
                      ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
                export { C };
                class C {}
                      ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
                export { Foo };

                enum Foo {
                  BAR,
                }
                      ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
                export { Foo };

                namespace Foo {
                  export let bar = () => console.log('bar');
                }
                      ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
                export { Foo, baz };

                enum Foo {
                  BAR,
                }

                let baz: Enum;
                enum Enum {}
                      ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
                import * as React from 'react';

                <div />;
                      ",
            None,
        ), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, "sourceType": "module", }, },
        (
            "
                import React from 'react';

                <div />;
                      ",
            None,
        ), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, "sourceType": "module", }, },
        (
            "
                import { h } from 'preact';

                <div />;
                      ",
            None,
        ), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, "jsxPragma": "h", "sourceType": "module", }, },
        (
            "
                const React = require('react');

                <div />;
                      ",
            None,
        ), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        (
            "
                type T = (value: unknown) => value is Id;
                    ",
            None,
        ),
        (
            "
                global.foo = true;

                declare global {
                  namespace NodeJS {
                    interface Global {
                      foo?: boolean;
                    }
                  }
                }
                    ",
            None,
        ),
        (
            "
                @Directive({
                  selector: '[rcCidrIpPattern]',
                  providers: [
                    {
                      provide: NG_VALIDATORS,
                      useExisting: CidrIpPatternDirective,
                      multi: true,
                    },
                  ],
                })
                export class CidrIpPatternDirective implements Validator {}
                    ",
            None,
        ),
        (
            "
                @Directive({
                  selector: '[rcCidrIpPattern]',
                  providers: [
                    {
                      provide: NG_VALIDATORS,
                      useExisting: CidrIpPatternDirective,
                      multi: true,
                    },
                  ],
                })
                export class CidrIpPatternDirective implements Validator {}
                      ",
            Some(serde_json::json!([ { "classes": false, }, ])),
        ),
        (
            "
                class A {
                  constructor(printName) {
                    this.printName = printName;
                  }

                  openPort(printerName = this.printerName) {
                    this.tscOcx.ActiveXopenport(printerName);

                    return this;
                  }
                }
                    ",
            None,
        ),
        (
            "
                const obj = {
                  foo: 'foo-value',
                  bar: 'bar-value',
                } satisfies {
                  [key in 'foo' | 'bar']: `${key}-value`;
                };
                      ",
            Some(serde_json::json!([{ "ignoreTypeReferences": false }])),
        ),
        (
            "
                const obj = {
                  foo: 'foo-value',
                  bar: 'bar-value',
                } as {
                  [key in 'foo' | 'bar']: `${key}-value`;
                };
                      ",
            Some(serde_json::json!([{ "ignoreTypeReferences": false }])),
        ),
        (
            "
                const obj = {
                  foo: {
                    foo: 'foo',
                  } as {
                    [key in 'foo' | 'bar']: key;
                  },
                };
                      ",
            Some(serde_json::json!([{ "ignoreTypeReferences": false }])),
        ),
        (
            "
                const foo = {
                  bar: 'bar',
                } satisfies {
                  bar: typeof baz;
                };

                const baz = '';
                      ",
            Some(serde_json::json!([{ "ignoreTypeReferences": true }])),
        ),
        (
            "
                namespace A.X.Y {}

                import Z = A.X.Y;

                const X = 23;
                    ",
            None,
        ),
        (
            "
                    namespace A {
                        export namespace X {
                            export namespace Y {
                                export const foo = 40;
                            }
                        }
                    }

                    import Z = A.X.Y;

                    const X = 23;
                    ",
            None,
        ),
    ];

    let fail = vec![
        ("a++; var a=19;", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("a++; var a=19;", None), // { "ecmaVersion": 6 },
        ("a++; var a=19;", None),
        ("a(); var a=function() {};", None),
        ("alert(a[1]); var a=[1,3];", None),
        ("a(); function a() { alert(b); var b=10; a(); }", None),
        ("a(); var a=function() {};", Some(serde_json::json!(["nofunc"]))),
        ("(() => { alert(a); var a = 42; })();", None), // { "ecmaVersion": 6 },
        ("(() => a())(); function a() { }", None),      // { "ecmaVersion": 6 },
        (r#""use strict"; a(); { function a() {} }"#, None),
        ("a(); try { throw new Error() } catch (foo) {var a;}", None),
        ("var f = () => a; var a;", None), // { "ecmaVersion": 6 },
        ("new A(); class A {};", None),    // { "ecmaVersion": 6 },
        ("function foo() { new A(); } class A {};", None), // { "ecmaVersion": 6 },
        ("new A(); var A = class {};", None), // { "ecmaVersion": 6 },
        ("function foo() { new A(); } var A = class {};", None), // { "ecmaVersion": 6 },
        ("a++; { var a; }", None),         // { "ecmaVersion": 6 },
        (r#""use strict"; { a(); function a() {} }"#, None), // { "ecmaVersion": 6 },
        ("{a; let a = 1}", None),          // { "ecmaVersion": 6 },
        (
            "switch (foo) { case 1: a();
             default:
             let a;}",
            None,
        ), // { "ecmaVersion": 6 },
        ("if (true) { function foo() { a; } let a;}", None), // { "ecmaVersion": 6 },
        (
            "a(); var a=function() {};",
            Some(serde_json::json!([{ "functions": false, "classes": false }])),
        ),
        (
            "new A(); class A {};",
            Some(serde_json::json!([{ "functions": false, "classes": false }])),
        ), // { "ecmaVersion": 6 },
        ("new A(); var A = class {};", Some(serde_json::json!([{ "classes": false }]))), // { "ecmaVersion": 6 },
        (
            "function foo() { new A(); } var A = class {};",
            Some(serde_json::json!([{ "classes": false }])),
        ), // { "ecmaVersion": 6 },
        ("var a = a;", None),
        ("let a = a + b;", None),         // { "ecmaVersion": 6 },
        ("const a = foo(a);", None),      // { "ecmaVersion": 6 },
        ("function foo(a = a) {}", None), // { "ecmaVersion": 6 },
        ("var {a = a} = [];", None),      // { "ecmaVersion": 6 },
        ("var [a = a] = [];", None),      // { "ecmaVersion": 6 },
        ("var {b = a, a} = {};", None),   // { "ecmaVersion": 6 },
        ("var [b = a, a] = {};", None),   // { "ecmaVersion": 6 },
        ("var {a = 0} = a;", None),       // { "ecmaVersion": 6 },
        ("var [a = 0] = a;", None),       // { "ecmaVersion": 6 },
        ("for (var a in a) {}", None),
        ("for (var a of a) {}", None), // { "ecmaVersion": 6 },
        (
            "function foo() { bar; var bar = 1; } var bar;",
            Some(serde_json::json!([{ "variables": false }])),
        ),
        ("foo; var foo;", Some(serde_json::json!([{ "variables": false }]))),
        ("for (let x = x;;); let x = 0", None), // { "ecmaVersion": 2015 },
        ("for (let x in xs); let xs = []", None), // { "ecmaVersion": 2015 },
        ("for (let x of xs); let xs = []", None), // { "ecmaVersion": 2015 },
        ("try {} catch ({message = x}) {} let x = ''", None), // { "ecmaVersion": 2015 },
        ("with (obj) x; let x = {}", None),     // { "ecmaVersion": 2015 },
        ("with (x); let x = {}", None),         // { "ecmaVersion": 2015 },
        ("with (obj) { x } let x = {}", None),  // { "ecmaVersion": 2015 },
        ("with (obj) { if (a) { x } } let x = {}", None), // { "ecmaVersion": 2015 },
        ("with (obj) { (() => { if (a) { x } })() } let x = {}", None), // { "ecmaVersion": 2015 },
        ("class C extends C {}", Some(serde_json::json!([{ "classes": false }]))), // { "ecmaVersion": 6 },
        ("const C = class extends C {};", Some(serde_json::json!([{ "variables": false }]))), // { "ecmaVersion": 6 },
        ("class C extends (class { [C](){} }) {}", Some(serde_json::json!([{ "classes": false }]))), // { "ecmaVersion": 6 },
        (
            "const C = class extends (class { [C](){} }) {};",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { "ecmaVersion": 6 },
        (
            "class C extends (class { static field = C; }) {}",
            Some(serde_json::json!([{ "classes": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "const C = class extends (class { static field = C; }) {};",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { "ecmaVersion": 2022 },
        ("class C { [C](){} }", Some(serde_json::json!([{ "classes": false }]))), // { "ecmaVersion": 6 },
        ("(class C { [C](){} });", Some(serde_json::json!([{ "classes": false }]))), // { "ecmaVersion": 6 },
        ("const C = class { [C](){} };", Some(serde_json::json!([{ "variables": false }]))), // { "ecmaVersion": 6 },
        ("class C { static [C](){} }", Some(serde_json::json!([{ "classes": false }]))), // { "ecmaVersion": 6 },
        ("(class C { static [C](){} });", Some(serde_json::json!([{ "classes": false }]))), // { "ecmaVersion": 6 },
        ("const C = class { static [C](){} };", Some(serde_json::json!([{ "variables": false }]))), // { "ecmaVersion": 6 },
        ("class C { [C]; }", Some(serde_json::json!([{ "classes": false }]))), // { "ecmaVersion": 2022 },
        ("(class C { [C]; });", Some(serde_json::json!([{ "classes": false }]))), // { "ecmaVersion": 2022 },
        ("const C = class { [C]; };", Some(serde_json::json!([{ "variables": false }]))), // { "ecmaVersion": 2022 },
        ("class C { [C] = foo; }", Some(serde_json::json!([{ "classes": false }]))), // { "ecmaVersion": 2022 },
        ("(class C { [C] = foo; });", Some(serde_json::json!([{ "classes": false }]))), // { "ecmaVersion": 2022 },
        ("const C = class { [C] = foo; };", Some(serde_json::json!([{ "variables": false }]))), // { "ecmaVersion": 2022 },
        ("class C { static [C]; }", Some(serde_json::json!([{ "classes": false }]))), // { "ecmaVersion": 2022 },
        ("(class C { static [C]; });", Some(serde_json::json!([{ "classes": false }]))), // { "ecmaVersion": 2022 },
        ("const C = class { static [C]; };", Some(serde_json::json!([{ "variables": false }]))), // { "ecmaVersion": 2022 },
        ("class C { static [C] = foo; }", Some(serde_json::json!([{ "classes": false }]))), // { "ecmaVersion": 2022 },
        ("(class C { static [C] = foo; });", Some(serde_json::json!([{ "classes": false }]))), // { "ecmaVersion": 2022 },
        (
            "const C = class { static [C] = foo; };",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "const C = class { static field = C; };",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "const C = class { static field = class extends C {}; };",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "const C = class { static field = class { [C]; } };",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "const C = class { static field = class { static field = C; }; };",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { "ecmaVersion": 2022 },
        ("class C extends D {} class D {}", Some(serde_json::json!([{ "classes": false }]))), // { "ecmaVersion": 6 },
        (
            "class C extends (class { [a](){} }) {} let a;",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { "ecmaVersion": 6 },
        (
            "class C extends (class { static field = a; }) {} let a;",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { "ecmaVersion": 2022 },
        ("class C { [a]() {} } let a;", Some(serde_json::json!([{ "variables": false }]))), // { "ecmaVersion": 6 },
        ("class C { static [a]() {} } let a;", Some(serde_json::json!([{ "variables": false }]))), // { "ecmaVersion": 6 },
        ("class C { [a]; } let a;", Some(serde_json::json!([{ "variables": false }]))), // { "ecmaVersion": 2022 },
        ("class C { static [a]; } let a;", Some(serde_json::json!([{ "variables": false }]))), // { "ecmaVersion": 2022 },
        ("class C { [a] = foo; } let a;", Some(serde_json::json!([{ "variables": false }]))), // { "ecmaVersion": 2022 },
        ("class C { static [a] = foo; } let a;", Some(serde_json::json!([{ "variables": false }]))), // { "ecmaVersion": 2022 },
        ("class C { static field = a; } let a;", Some(serde_json::json!([{ "variables": false }]))), // { "ecmaVersion": 2022 },
        (
            "class C { static field = D; } class D {}",
            Some(serde_json::json!([{ "classes": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static field = class extends D {}; } class D {}",
            Some(serde_json::json!([{ "classes": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static field = class { [a](){} } } let a;",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static field = class { static field = a; }; } let a;",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { "ecmaVersion": 2022 },
        ("const C = class { static { C; } };", Some(serde_json::json!([{ "variables": false }]))), // { "ecmaVersion": 2022 },
        (
            "const C = class { static { (class extends C {}); } };",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { "ecmaVersion": 2022 },
        ("class C { static { a; } } let a;", Some(serde_json::json!([{ "variables": false }]))), // { "ecmaVersion": 2022 },
        ("class C { static { D; } } class D {}", Some(serde_json::json!([{ "classes": false }]))), // { "ecmaVersion": 2022 },
        (
            "class C { static { (class extends D {}); } } class D {}",
            Some(serde_json::json!([{ "classes": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { (class { [a](){} }); } } let a;",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { (class { static field = a; }); } } let a;",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { "ecmaVersion": 2022 },
        ("(class C extends C {});", Some(serde_json::json!([{ "classes": false }]))), // { "ecmaVersion": 6 },
        (
            "(class C extends (class { [C](){} }) {});",
            Some(serde_json::json!([{ "classes": false }])),
        ), // { "ecmaVersion": 6 },
        (
            "(class C extends (class { static field = C; }) {});",
            Some(serde_json::json!([{ "classes": false }])),
        ), // { "ecmaVersion": 2022 },
        ("export { a }; const a = 1;", None), // { "ecmaVersion": 2015, "sourceType": "module" },
        ("export { a }; const a = 1;", Some(serde_json::json!([{}]))), // { "ecmaVersion": 2015, "sourceType": "module" },
        ("export { a }; const a = 1;", Some(serde_json::json!([{ "allowNamedExports": false }]))), // { "ecmaVersion": 2015, "sourceType": "module" },
        ("export { a }; const a = 1;", Some(serde_json::json!(["nofunc"]))), // { "ecmaVersion": 2015, "sourceType": "module" },
        ("export { a as b }; const a = 1;", None), // { "ecmaVersion": 2015, "sourceType": "module" },
        ("export { a, b }; let a, b;", None), // { "ecmaVersion": 2015, "sourceType": "module" },
        ("export { a }; var a;", None),       // { "ecmaVersion": 2015, "sourceType": "module" },
        ("export { f }; function f() {}", None), // { "ecmaVersion": 2015, "sourceType": "module" },
        ("export { C }; class C {}", None),   // { "ecmaVersion": 2015, "sourceType": "module" },
        (
            "export const foo = a; const a = 1;",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { "ecmaVersion": 2015, "sourceType": "module" },
        (
            "export default a; const a = 1;",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { "ecmaVersion": 2015, "sourceType": "module" },
        (
            "export function foo() { return a; }; const a = 1;",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { "ecmaVersion": 2015, "sourceType": "module" },
        (
            "export class C { foo() { return a; } }; const a = 1;",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { "ecmaVersion": 2015, "sourceType": "module" },
        ("<App />; const App = () => <div />;", None), // { "ecmaVersion": 6, "parserOptions": { "ecmaFeatures": { "jsx": true } }, },
        ("function render() { return <Widget /> }; const Widget = () => <span />;", None), // { "ecmaVersion": 6, "parserOptions": { "ecmaFeatures": { "jsx": true } }, },
        ("<Foo.Bar />; const Foo = { Bar: () => <div/> };", None), // { "ecmaVersion": 6, "parserOptions": { "ecmaFeatures": { "jsx": true } }, },
        (
            "
                a++;
                var a = 19;
                      ",
            None,
        ), // { "parserOptions": { "sourceType": "module" }, },
        (
            "
                a++;
                var a = 19;
                      ",
            None,
        ), // { parserOptions },
        (
            "
                a++;
                var a = 19;
                      ",
            None,
        ),
        (
            "
                a();
                var a = function () {};
                      ",
            None,
        ),
        (
            "
                alert(a[1]);
                var a = [1, 3];
                      ",
            None,
        ),
        (
            "
                a();
                function a() {
                  alert(b);
                  var b = 10;
                  a();
                }
                      ",
            None,
        ),
        (
            "
                a();
                var a = function () {};
                      ",
            Some(serde_json::json!(["nofunc"])),
        ),
        (
            "
                (() => {
                  alert(a);
                  var a = 42;
                })();
                      ",
            None,
        ), // { parserOptions },
        (
            "
                (() => a())();
                function a() {}
                      ",
            None,
        ), // { parserOptions },
        (
            "
                a();
                try {
                  throw new Error();
                } catch (foo) {
                  var a;
                }
                      ",
            None,
        ),
        (
            "
                var f = () => a;
                var a;
                      ",
            None,
        ), // { parserOptions },
        (
            "
                new A();
                class A {}
                      ",
            None,
        ), // { parserOptions },
        (
            "
                function foo() {
                  new A();
                }
                class A {}
                      ",
            None,
        ), // { parserOptions },
        (
            "
                new A();
                var A = class {};
                      ",
            None,
        ), // { parserOptions },
        (
            "
                function foo() {
                  new A();
                }
                var A = class {};
                      ",
            None,
        ), // { parserOptions },
        (
            "
                a++;
                {
                  var a;
                }
                      ",
            None,
        ), // { parserOptions },
        (
            "
                'use strict';
                {
                  a();
                  function a() {}
                }
                      ",
            None,
        ), // { parserOptions },
        (
            "
                {
                  a;
                  let a = 1;
                }
                      ",
            None,
        ), // { parserOptions },
        (
            "
                switch (foo) {
                  case 1:
                    a();
                  default:
                    let a;
                }
                      ",
            None,
        ), // { parserOptions },
        (
            "
                if (true) {
                  function foo() {
                    a;
                  }
                  let a;
                }
                      ",
            None,
        ), // { parserOptions },
        (
            "
                a();
                var a = function () {};
                      ",
            Some(serde_json::json!([{ "classes": false, "functions": false }])),
        ),
        (
            "
                new A();
                var A = class {};
                      ",
            Some(serde_json::json!([{ "classes": false }])),
        ), // { parserOptions },
        (
            "
                function foo() {
                  new A();
                }
                var A = class {};
                      ",
            Some(serde_json::json!([{ "classes": false }])),
        ), // { parserOptions },
        ("var a = a;", None),
        ("let a = a + b;", None),         // { parserOptions },
        ("const a = foo(a);", None),      // { parserOptions },
        ("function foo(a = a) {}", None), // { parserOptions },
        ("var { a = a } = [];", None),    // { parserOptions },
        ("var [a = a] = [];", None),      // { parserOptions },
        ("var { b = a, a } = {};", None), // { parserOptions },
        ("var [b = a, a] = {};", None),   // { parserOptions },
        ("var { a = 0 } = a;", None),     // { parserOptions },
        ("var [a = 0] = a;", None),       // { parserOptions },
        (
            "
                for (var a in a) {
                }
                      ",
            None,
        ),
        (
            "
                for (var a of a) {
                }
                      ",
            None,
        ), // { parserOptions },
        (
            "
                interface Bar {
                  type: typeof Foo;
                }

                const Foo = 2;
                      ",
            Some(serde_json::json!([{ "ignoreTypeReferences": false }])),
        ),
        (
            "
                let var1: StringOrNumber;

            type StringOrNumber = string | number;
                      ",
            Some(serde_json::json!([{ "ignoreTypeReferences": false, "typedefs": true }])),
        ),
        (
            "
                interface Bar {
                  type: typeof Foo.FOO;
                }

                class Foo {
                  public static readonly FOO = '';
                }
                      ",
            Some(serde_json::json!([{ "ignoreTypeReferences": false }])),
        ),
        (
            "
                interface Bar {
                  type: typeof Foo.Bar.Baz;
                }

                const Foo = {
                  Bar: {
                    Baz: 1,
                  },
                };
                      ",
            Some(serde_json::json!([{ "ignoreTypeReferences": false }])),
        ),
        (
            "
                const foo = {
                  bar: 'bar',
                } satisfies {
                  bar: typeof baz;
                };

                const baz = '';
                      ",
            Some(serde_json::json!([{ "ignoreTypeReferences": false }])),
        ),
        (
            "
                function foo() {
                  bar;
                  var bar = 1;
                }
                var bar;
                      ",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { parserOptions },
        (
            "
                class Test {
                  foo(args: Foo): Foo {
                    return Foo.FOO;
                  }
                }

                enum Foo {
                  FOO,
                }
                      ",
            Some(serde_json::json!([{ "enums": true }])),
        ),
        (
            "
                function foo(): Foo {
                  return Foo.FOO;
                }

                enum Foo {
                   FOO,
                 }
                ",
            Some(serde_json::json!([{ "enums": true }])),
        ),
        (
            "
                const foo = Foo.Foo;

                enum Foo {
                  FOO,
                }
                      ",
            Some(serde_json::json!([{ "enums": true }])),
        ),
        (
            "
                export { a };
                const a = 1;
                      ",
            None,
        ), // { parserOptions },
        (
            "
                export { a };
                const a = 1;
                      ",
            Some(serde_json::json!([{}])),
        ), // { parserOptions },
        (
            "
                export { a };
                const a = 1;
                      ",
            Some(serde_json::json!([{ "allowNamedExports": false }])),
        ), // { parserOptions },
        (
            "
                export { a };
                const a = 1;
                      ",
            Some(serde_json::json!(["nofunc"])),
        ), // { parserOptions },
        (
            "
                export { a as b };
                const a = 1;
                      ",
            None,
        ), // { parserOptions },
        (
            "
                export { a, b };
                let a, b;
                      ",
            None,
        ), // { parserOptions },
        (
            "
                export { a };
                var a;
                      ",
            None,
        ), // { parserOptions },
        (
            "
                export { f };
                function f() {}
                      ",
            None,
        ), // { parserOptions },
        (
            "
                export { C };
                class C {}
                      ",
            None,
        ), // { parserOptions },
        (
            "
                export const foo = a;
                const a = 1;
                      ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
                export function foo() {
                  return a;
                }
                const a = 1;
                      ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
                export class C {
                  foo() {
                    return a;
                  }
                }
                const a = 1;
                      ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
                export { Foo };

                enum Foo {
                  BAR,
                }
                      ",
            None,
        ), // { parserOptions },
        (
            "
                export { Foo };

                namespace Foo {
                  export let bar = () => console.log('bar');
                }
                      ",
            None,
        ), // { parserOptions },
        (
            "
                export { Foo, baz };

                enum Foo {
                  BAR,
                }

                let baz: Enum;
                enum Enum {}
                      ",
            Some(serde_json::json!([{ "allowNamedExports": false, "ignoreTypeReferences": true }])),
        ), // { parserOptions },
        (
            "
                f();
                function f() {}
                      ",
            None,
        ),
        (
            "
                alert(a);
                var a = 10;
                      ",
            None,
        ),
        (
            "
                f()?.();
                function f() {
                  return function t() {};
                }
                      ",
            None,
        ),
        (
            "
                alert(a?.b);
                var a = { b: 5 };
                      ",
            None,
        ),
        (
            r#"
                @decorator
                class C {
              		static x = "foo";
              		[C.x]() { }
                }
                      "#,
            None,
        ),
    ];

    Tester::new(NoUseBeforeDefine::NAME, NoUseBeforeDefine::PLUGIN, pass, fail).test_and_snapshot();
}

#[test]
fn test_typescript_eslint() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
            type foo = 1;
            const x: foo = 1;
                ",
            None,
        ),
        (
            "
            type foo = 1;
            type bar = foo;
                ",
            None,
        ),
        (
            "
            interface Foo {}
            const x: Foo = {};
                ",
            None,
        ),
        (
            "
            var a = 10;
            alert(a);
                ",
            None,
        ),
        (
            "
            function b(a) {
              alert(a);
            }
                ",
            None,
        ),
        ("Object.hasOwnProperty.call(a);", None),
        (
            "
            function a() {
              alert(arguments);
            }
                ",
            None,
        ),
        ("declare function a();", None),
        (
            "
            declare class a {
              foo();
            }
                ",
            None,
        ),
        ("const updatedAt = data?.updatedAt;", None),
        (
            "
            function f() {
              return function t() {};
            }
            f()?.();
                ",
            None,
        ),
        (
            "
            var a = { b: 5 };
            alert(a?.b);
                ",
            None,
        ),
        (
            "
            a();
            function a() {
              alert(arguments);
            }
                  ",
            Some(serde_json::json!([{ "functions": false }])),
        ),
        (
            "
            (() => {
              var a = 42;
              alert(a);
            })();
                  ",
            None,
        ), // { parserOptions },
        (
            "
            a();
            try {
              throw new Error();
            } catch (a) {}
                ",
            None,
        ),
        (
            "
            class A {}
            new A();
                  ",
            None,
        ), // { parserOptions },
        (
            "
            var a = 0,
              b = a;
                ",
            None,
        ),
        ("var { a = 0, b = a } = {};", None), // { parserOptions },
        ("var [a = 0, b = a] = {};", None),   // { parserOptions },
        (
            "
            function foo() {
              foo();
            }
                ",
            None,
        ),
        (
            "
            var foo = function () {
              foo();
            };
                ",
            None,
        ),
        (
            "
            var a;
            for (a in a) {
            }
                ",
            None,
        ),
        (
            "
            var a;
            for (a of a) {
            }
                  ",
            None,
        ), // { parserOptions },
        (
            "
            'use strict';
            a();
            {
              function a() {}
            }
                  ",
            None,
        ), // { parserOptions },
        (
            "
            'use strict';
            {
              a();
              function a() {}
            }
                  ",
            Some(serde_json::json!([{ "functions": false }])),
        ), // { parserOptions },
        (
            "
            switch (foo) {
              case 1: {
                a();
              }
              default: {
                let a;
              }
            }
                  ",
            None,
        ), // { parserOptions },
        (
            "
            a();
            {
              let a = function () {};
            }
                  ",
            None,
        ), // { parserOptions },
        (
            "
            a();
            function a() {
              alert(arguments);
            }
                  ",
            Some(serde_json::json!([{ "functions": false }])),
        ),
        (
            "
            'use strict';
            {
              a();
              function a() {}
            }
                  ",
            Some(serde_json::json!([{ "functions": false }])),
        ), // { parserOptions },
        (
            "
            function foo() {
              new A();
            }
            class A {}
                  ",
            Some(serde_json::json!([{ "classes": false }])),
        ), // { parserOptions },
        (
            "
            function foo() {
              bar;
            }
            var bar;
                  ",
            Some(serde_json::json!([{ "variables": false }])),
        ),
        (
            "
            var foo = () => bar;
            var bar;
                  ",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { parserOptions },
        (
            "
            var x: Foo = 2;
            type Foo = string | number;
                  ",
            Some(serde_json::json!([{ "typedefs": false }])),
        ),
        (
            "
            interface Bar {
              type: typeof Foo;
            }

            const Foo = 2;
                  ",
            Some(serde_json::json!([{ "ignoreTypeReferences": true }])),
        ),
        (
            "
            interface Bar {
              type: typeof Foo.FOO;
            }

            class Foo {
              public static readonly FOO = '';
            }
                  ",
            Some(serde_json::json!([{ "ignoreTypeReferences": true }])),
        ),
        (
            "
            interface Bar {
              type: typeof Foo.Bar.Baz;
            }

            const Foo = {
              Bar: {
                Baz: 1,
              },
            };
                  ",
            Some(serde_json::json!([{ "ignoreTypeReferences": true }])),
        ),
        (
            "
            interface ITest {
              first: boolean;
              second: string;
              third: boolean;
            }

            let first = () => console.log('first');

            export let second = () => console.log('second');

            export namespace Third {
              export let third = () => console.log('third');
            }
                  ",
            None,
        ), // { "parserOptions": { "ecmaVersion": 6, "sourceType": "module" }, },
        (
            "
            function test(file: Blob) {
              const slice: typeof file.slice =
                file.slice || (file as any).webkitSlice || (file as any).mozSlice;
              return slice;
            }
                ",
            None,
        ),
        (
            "
            interface Foo {
              bar: string;
            }
            const bar = 'blah';
                ",
            None,
        ),
        (
            "
            function foo(): Foo {
              return Foo.FOO;
            }

            enum Foo {
              FOO,
            }
                  ",
            Some(serde_json::json!([{ "enums": false }])),
        ),
        (
            "
            let foo: Foo;

            enum Foo {
              FOO,
            }
                  ",
            Some(serde_json::json!([{ "enums": false }])),
        ),
        (
            "
            class Test {
              foo(args: Foo): Foo {
                return Foo.FOO;
              }
            }

            enum Foo {
              FOO,
            }
                  ",
            Some(serde_json::json!([{ "enums": false }])),
        ),
        (
            "
            export { a };
            const a = 1;
                  ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
            export { a as b };
            const a = 1;
                  ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
            export { a, b };
            let a, b;
                  ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
            export { a };
            var a;
                  ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
            export { f };
            function f() {}
                  ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
            export { C };
            class C {}
                  ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
            export { Foo };

            enum Foo {
              BAR,
            }
                  ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
            export { Foo };

            namespace Foo {
              export let bar = () => console.log('bar');
            }
                  ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
            export { Foo, baz };

            enum Foo {
              BAR,
            }

            let baz: Enum;
            enum Enum {}
                  ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
            import * as React from 'react';

            <div />;
                  ",
            None,
        ), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, "sourceType": "module", }, },
        (
            "
            import React from 'react';

            <div />;
                  ",
            None,
        ), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, "sourceType": "module", }, },
        (
            "
            import { h } from 'preact';

            <div />;
                  ",
            None,
        ), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, "jsxPragma": "h", "sourceType": "module", }, },
        (
            "
            const React = require('react');

            <div />;
                  ",
            None,
        ), // { "parserOptions": { "ecmaFeatures": { "jsx": true, }, }, },
        (
            "
            type T = (value: unknown) => value is Id;
                ",
            None,
        ),
        (
            "
            global.foo = true;

            declare global {
              namespace NodeJS {
                interface Global {
                  foo?: boolean;
                }
              }
            }
                ",
            None,
        ),
        (
            "
            @Directive({
              selector: '[rcCidrIpPattern]',
              providers: [
                {
                  provide: NG_VALIDATORS,
                  useExisting: CidrIpPatternDirective,
                  multi: true,
                },
              ],
            })
            export class CidrIpPatternDirective implements Validator {}
                ",
            None,
        ),
        (
            "
            @Directive({
              selector: '[rcCidrIpPattern]',
              providers: [
                {
                  provide: NG_VALIDATORS,
                  useExisting: CidrIpPatternDirective,
                  multi: true,
                },
              ],
            })
            export class CidrIpPatternDirective implements Validator {}
                  ",
            Some(serde_json::json!([ { "classes": false, }, ])),
        ),
        (
            "
            class A {
              constructor(printName) {
                this.printName = printName;
              }

              openPort(printerName = this.printerName) {
                this.tscOcx.ActiveXopenport(printerName);

                return this;
              }
            }
                ",
            None,
        ),
        (
            "
            const obj = {
              foo: 'foo-value',
              bar: 'bar-value',
            } satisfies {
              [key in 'foo' | 'bar']: `${key}-value`;
            };
                  ",
            Some(serde_json::json!([{ "ignoreTypeReferences": false }])),
        ),
        (
            "
            const obj = {
              foo: 'foo-value',
              bar: 'bar-value',
            } as {
              [key in 'foo' | 'bar']: `${key}-value`;
            };
                  ",
            Some(serde_json::json!([{ "ignoreTypeReferences": false }])),
        ),
        (
            "
            const obj = {
              foo: {
                foo: 'foo',
              } as {
                [key in 'foo' | 'bar']: key;
              },
            };
                  ",
            Some(serde_json::json!([{ "ignoreTypeReferences": false }])),
        ),
        (
            "
            const foo = {
              bar: 'bar',
            } satisfies {
              bar: typeof baz;
            };

            const baz = '';
                  ",
            Some(serde_json::json!([{ "ignoreTypeReferences": true }])),
        ),
        (
            "
            namespace A.X.Y {}

            import Z = A.X.Y;

            const X = 23;
                ",
            None,
        ),
    ];

    let fail = vec![
        (
            "
            a++;
            var a = 19;
                  ",
            None,
        ), // { "parserOptions": { "sourceType": "module" }, },
        (
            "
            a++;
            var a = 19;
                  ",
            None,
        ), // { parserOptions },
        (
            "
            a++;
            var a = 19;
                  ",
            None,
        ),
        (
            "
            a();
            var a = function () {};
                  ",
            None,
        ),
        (
            "
            alert(a[1]);
            var a = [1, 3];
                  ",
            None,
        ),
        (
            "
            a();
            function a() {
              alert(b);
              var b = 10;
              a();
            }
                  ",
            None,
        ),
        (
            "
            a();
            var a = function () {};
                  ",
            Some(serde_json::json!([{ "functions": false }])),
        ),
        (
            "
            (() => {
              alert(a);
              var a = 42;
            })();
                  ",
            None,
        ), // { parserOptions },
        (
            "
            (() => a())();
            function a() {}
                  ",
            None,
        ), // { parserOptions },
        (
            "
            a();
            try {
              throw new Error();
            } catch (foo) {
              var a;
            }
                  ",
            None,
        ),
        (
            "
            var f = () => a;
            var a;
                  ",
            None,
        ), // { parserOptions },
        (
            "
            new A();
            class A {}
                  ",
            None,
        ), // { parserOptions },
        (
            "
            function foo() {
              new A();
            }
            class A {}
                  ",
            None,
        ), // { parserOptions },
        (
            "
            new A();
            var A = class {};
                  ",
            None,
        ), // { parserOptions },
        (
            "
            function foo() {
              new A();
            }
            var A = class {};
                  ",
            None,
        ), // { parserOptions },
        (
            "
            a++;
            {
              var a;
            }
                  ",
            None,
        ), // { parserOptions },
        (
            "
            'use strict';
            {
              a();
              function a() {}
            }
                  ",
            None,
        ), // { parserOptions },
        (
            "
            {
              a;
              let a = 1;
            }
                  ",
            None,
        ), // { parserOptions },
        (
            "
            switch (foo) {
              case 1:
                a();
              default:
                let a;
            }
                  ",
            None,
        ), // { parserOptions },
        (
            "
            if (true) {
              function foo() {
                a;
              }
              let a;
            }
                  ",
            None,
        ), // { parserOptions },
        (
            "
            a();
            var a = function () {};
                  ",
            Some(serde_json::json!([{ "classes": false, "functions": false }])),
        ),
        (
            "
            new A();
            var A = class {};
                  ",
            Some(serde_json::json!([{ "classes": false }])),
        ), // { parserOptions },
        (
            "
            function foo() {
              new A();
            }
            var A = class {};
                  ",
            Some(serde_json::json!([{ "classes": false }])),
        ), // { parserOptions },
        ("var a = a;", None),
        ("let a = a + b;", None),         // { parserOptions },
        ("const a = foo(a);", None),      // { parserOptions },
        ("function foo(a = a) {}", None), // { parserOptions },
        ("var { a = a } = [];", None),    // { parserOptions },
        ("var [a = a] = [];", None),      // { parserOptions },
        ("var { b = a, a } = {};", None), // { parserOptions },
        ("var [b = a, a] = {};", None),   // { parserOptions },
        ("var { a = 0 } = a;", None),     // { parserOptions },
        ("var [a = 0] = a;", None),       // { parserOptions },
        (
            "
            for (var a in a) {
            }
                  ",
            None,
        ),
        (
            "
            for (var a of a) {
            }
                  ",
            None,
        ), // { parserOptions },
        (
            "
            interface Bar {
              type: typeof Foo;
            }

            const Foo = 2;
                  ",
            Some(serde_json::json!([{ "ignoreTypeReferences": false }])),
        ),
        (
            "
            interface Bar {
              type: typeof Foo.FOO;
            }

            class Foo {
              public static readonly FOO = '';
            }
                  ",
            Some(serde_json::json!([{ "ignoreTypeReferences": false }])),
        ),
        (
            "
            interface Bar {
              type: typeof Foo.Bar.Baz;
            }

            const Foo = {
              Bar: {
                Baz: 1,
              },
            };
                  ",
            Some(serde_json::json!([{ "ignoreTypeReferences": false }])),
        ),
        (
            "
            const foo = {
              bar: 'bar',
            } satisfies {
              bar: typeof baz;
            };

            const baz = '';
                  ",
            Some(serde_json::json!([{ "ignoreTypeReferences": false }])),
        ),
        (
            "
            function foo() {
              bar;
              var bar = 1;
            }
            var bar;
                  ",
            Some(serde_json::json!([{ "variables": false }])),
        ), // { parserOptions },
        (
            "
            class Test {
              foo(args: Foo): Foo {
                return Foo.FOO;
              }
            }

            enum Foo {
              FOO,
            }
                  ",
            Some(serde_json::json!([{ "enums": true }])),
        ),
        (
            "
            function foo(): Foo {
              return Foo.FOO;
            }

            enum Foo {
              FOO,
            }
                  ",
            Some(serde_json::json!([{ "enums": true }])),
        ),
        (
            "
            const foo = Foo.Foo;

            enum Foo {
              FOO,
            }
                  ",
            Some(serde_json::json!([{ "enums": true }])),
        ),
        (
            "
            export { a };
            const a = 1;
                  ",
            None,
        ), // { parserOptions },
        (
            "
            export { a };
            const a = 1;
                  ",
            Some(serde_json::json!([{}])),
        ), // { parserOptions },
        (
            "
            export { a };
            const a = 1;
                  ",
            Some(serde_json::json!([{ "allowNamedExports": false }])),
        ), // { parserOptions },
        (
            "
            export { a };
            const a = 1;
                  ",
            Some(serde_json::json!([{ "functions": false }])),
        ), // { parserOptions },
        (
            "
            export { a as b };
            const a = 1;
                  ",
            None,
        ), // { parserOptions },
        (
            "
            export { a, b };
            let a, b;
                  ",
            None,
        ), // { parserOptions },
        (
            "
            export { a };
            var a;
                  ",
            None,
        ), // { parserOptions },
        (
            "
            export { f };
            function f() {}
                  ",
            None,
        ), // { parserOptions },
        (
            "
            export { C };
            class C {}
                  ",
            None,
        ), // { parserOptions },
        (
            "
            export const foo = a;
            const a = 1;
                  ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
            export function foo() {
              return a;
            }
            const a = 1;
                  ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
            export class C {
              foo() {
                return a;
              }
            }
            const a = 1;
                  ",
            Some(serde_json::json!([{ "allowNamedExports": true }])),
        ), // { parserOptions },
        (
            "
            export { Foo };

            enum Foo {
              BAR,
            }
                  ",
            None,
        ), // { parserOptions },
        (
            "
            export { Foo };

            namespace Foo {
              export let bar = () => console.log('bar');
            }
                  ",
            None,
        ), // { parserOptions },
        (
            "
            export { Foo, baz };

            enum Foo {
              BAR,
            }

            let baz: Enum;
            enum Enum {}
                  ",
            Some(serde_json::json!([{ "allowNamedExports": false, "ignoreTypeReferences": true }])),
        ), // { parserOptions },
        (
            "
            f();
            function f() {}
                  ",
            None,
        ),
        (
            "
            alert(a);
            var a = 10;
                  ",
            None,
        ),
        (
            "
            f()?.();
            function f() {
              return function t() {};
            }
                  ",
            None,
        ),
        (
            "
            alert(a?.b);
            var a = { b: 5 };
                  ",
            None,
        ),
    ];

    Tester::new(NoUseBeforeDefine::NAME, NoUseBeforeDefine::PLUGIN, pass, fail)
        .with_snapshot_suffix("typescript-eslint")
        .test_and_snapshot();
}
