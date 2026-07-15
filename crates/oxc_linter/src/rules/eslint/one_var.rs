use oxc_ast::{
    AstKind, AstType,
    ast::{Expression, Statement, VariableDeclaration, VariableDeclarationKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{node::NodeId, scope::ScopeId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    fixer::{Fix, RuleFix},
    rule::{DefaultRuleConfig, Rule},
};

fn one_var_diagnostic(span: Span, message: String) -> OxcDiagnostic {
    OxcDiagnostic::warn(message).with_label(span)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum OneVarMode {
    #[default]
    Always,
    Never,
    Consecutive,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct OneVarOptions {
    r#const: Option<OneVarMode>,
    separate_requires: bool,
    var: Option<OneVarMode>,
    await_using: Option<OneVarMode>,
    r#let: Option<OneVarMode>,
    using: Option<OneVarMode>,
    initialized: Option<OneVarMode>,
    uninitialized: Option<OneVarMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
enum OneVarConfig {
    Mode(OneVarMode),
    Options(OneVarOptions),
}

impl Default for OneVarConfig {
    fn default() -> Self {
        Self::Mode(OneVarMode::Always)
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct OneVar(OneVarConfig);

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces variables to be declared either together or separately.
    ///
    /// ### Why is this bad?
    ///
    /// Consistent declaration grouping makes variable lifetimes and initialization patterns easier
    /// to scan. This rule can require one declaration per scope, one declarator per statement, or
    /// grouping only consecutive declarations.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var foo = 1;
    /// var bar = 2;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var foo = 1, bar = 2;
    /// ```
    OneVar,
    eslint,
    style,
    conditional_fix,
    config = OneVar,
    version = "next",
    short_description = "Enforce variables to be declared either together or separately in functions.",
);

impl Rule for OneVar {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        if !ctx.nodes().contains(AstType::VariableDeclaration) {
            return;
        }

        let mut scopes = vec![[ScopeState::default(); 5]; ctx.scoping().scopes_len()];
        let separate_requires = self.separate_requires();

        for node in ctx.nodes().iter() {
            let AstKind::VariableDeclaration(declaration) = node.kind() else {
                continue;
            };
            let modes = self.modes(declaration.kind);
            if modes.initialized.is_none() && modes.uninitialized.is_none() {
                continue;
            }

            let facts = DeclarationFacts::new(declaration);
            if separate_requires
                && modes.initialized == Some(OneVarMode::Always)
                && facts.requires > 0
                && facts.requires != declaration.declarations.len()
            {
                ctx.diagnostic(one_var_diagnostic(
                    declaration.span,
                    "Split requires to be separated into a single block.".to_string(),
                ));
            }

            if (modes.initialized == Some(OneVarMode::Consecutive)
                || modes.uninitialized == Some(OneVarMode::Consecutive))
                && let Some(previous) = previous_declaration(node, ctx)
                && previous.kind == declaration.kind
            {
                let previous_facts = DeclarationFacts::new(previous);
                if !facts.mixed_requires_with(&previous_facts) {
                    if modes.initialized == Some(OneVarMode::Consecutive)
                        && modes.uninitialized == Some(OneVarMode::Consecutive)
                    {
                        report_join(
                            ctx,
                            declaration,
                            previous,
                            format!(
                                "Combine this with the previous '{}' statement.",
                                declaration.kind.as_str()
                            ),
                        );
                    } else {
                        if modes.initialized == Some(OneVarMode::Consecutive)
                            && facts.initialized > 0
                            && previous_facts.initialized > 0
                        {
                            report_join(
                                ctx,
                                declaration,
                                previous,
                                format!(
                                    "Combine this with the previous '{}' statement with initialized variables.",
                                    declaration.kind.as_str()
                                ),
                            );
                        }
                        if modes.uninitialized == Some(OneVarMode::Consecutive)
                            && facts.uninitialized > 0
                            && previous_facts.uninitialized > 0
                        {
                            report_join(
                                ctx,
                                declaration,
                                previous,
                                format!(
                                    "Combine this with the previous '{}' statement with uninitialized variables.",
                                    declaration.kind.as_str()
                                ),
                            );
                        }
                    }
                }
            }

            let scope_id = declaration_scope(node, ctx);
            let state = &mut scopes[scope_id.index()][declaration.kind as usize];
            let has_requires = facts.requires > 0;
            let should_join_initialized = modes.initialized == Some(OneVarMode::Always)
                && facts.initialized > 0
                && state.initialized
                && !has_requires;
            let should_join_uninitialized = modes.uninitialized == Some(OneVarMode::Always)
                && facts.uninitialized > 0
                && state.uninitialized;
            let should_join_all = modes.initialized == Some(OneVarMode::Always)
                && modes.uninitialized == Some(OneVarMode::Always)
                && (state.initialized || state.uninitialized)
                && !has_requires;
            let should_join_require = separate_requires && has_requires && state.required;

            if should_join_all || should_join_require {
                report_join_with_optional_previous(
                    ctx,
                    node,
                    declaration,
                    format!(
                        "Combine this with the previous '{}' statement.",
                        declaration.kind.as_str()
                    ),
                );
            } else {
                if should_join_initialized {
                    report_join_with_optional_previous(
                        ctx,
                        node,
                        declaration,
                        format!(
                            "Combine this with the previous '{}' statement with initialized variables.",
                            declaration.kind.as_str()
                        ),
                    );
                }
                if should_join_uninitialized && !is_for_in_or_of_left(node, ctx) {
                    report_join_with_optional_previous(
                        ctx,
                        node,
                        declaration,
                        format!(
                            "Combine this with the previous '{}' statement with uninitialized variables.",
                            declaration.kind.as_str()
                        ),
                    );
                }
            }

            if modes.initialized == Some(OneVarMode::Always) && facts.initialized > 0 {
                if separate_requires && has_requires {
                    state.required = true;
                } else {
                    state.initialized = true;
                }
            }
            if modes.uninitialized == Some(OneVarMode::Always) && facts.uninitialized > 0 {
                state.uninitialized = true;
            }

            if !is_for_statement_init(node, ctx) && declaration.declarations.len() > 1 {
                if modes.initialized == Some(OneVarMode::Never)
                    && modes.uninitialized == Some(OneVarMode::Never)
                {
                    report_split(
                        ctx,
                        node,
                        declaration,
                        format!(
                            "Split '{}' declarations into multiple statements.",
                            declaration.kind.as_str()
                        ),
                    );
                } else {
                    if modes.initialized == Some(OneVarMode::Never) && facts.initialized > 0 {
                        report_split(
                            ctx,
                            node,
                            declaration,
                            format!(
                                "Split initialized '{}' declarations into multiple statements.",
                                declaration.kind.as_str()
                            ),
                        );
                    }
                    if modes.uninitialized == Some(OneVarMode::Never) && facts.uninitialized > 0 {
                        report_split(
                            ctx,
                            node,
                            declaration,
                            format!(
                                "Split uninitialized '{}' declarations into multiple statements.",
                                declaration.kind.as_str()
                            ),
                        );
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct EffectiveModes {
    initialized: Option<OneVarMode>,
    uninitialized: Option<OneVarMode>,
}

impl OneVar {
    fn modes(&self, kind: VariableDeclarationKind) -> EffectiveModes {
        match &self.0 {
            OneVarConfig::Mode(mode) => {
                EffectiveModes { initialized: Some(*mode), uninitialized: Some(*mode) }
            }
            OneVarConfig::Options(options) => {
                let per_kind = match kind {
                    VariableDeclarationKind::Var => options.var,
                    VariableDeclarationKind::Let => options.r#let,
                    VariableDeclarationKind::Const => options.r#const,
                    VariableDeclarationKind::Using => options.using,
                    VariableDeclarationKind::AwaitUsing => options.await_using,
                };
                EffectiveModes {
                    initialized: options.initialized.or(per_kind),
                    uninitialized: options.uninitialized.or(per_kind),
                }
            }
        }
    }

    fn separate_requires(&self) -> bool {
        matches!(&self.0, OneVarConfig::Options(options) if options.separate_requires)
    }
}

#[derive(Default, Clone, Copy)]
struct ScopeState {
    initialized: bool,
    uninitialized: bool,
    required: bool,
}

struct DeclarationFacts {
    initialized: usize,
    uninitialized: usize,
    requires: usize,
}

impl DeclarationFacts {
    fn new(declaration: &VariableDeclaration<'_>) -> Self {
        let mut initialized = 0;
        let mut requires = 0;
        for declarator in &declaration.declarations {
            if let Some(initializer) = &declarator.init {
                initialized += 1;
                requires += usize::from(is_require(initializer));
            }
        }
        Self { initialized, uninitialized: declaration.declarations.len() - initialized, requires }
    }

    fn mixed_requires_with(&self, other: &Self) -> bool {
        let requires = self.requires + other.requires;
        requires > 0
            && requires
                != self.initialized + self.uninitialized + other.initialized + other.uninitialized
    }
}

fn is_require(expression: &Expression<'_>) -> bool {
    matches!(
        expression,
        Expression::CallExpression(call)
            if matches!(&call.callee, Expression::Identifier(identifier) if identifier.name == "require")
    )
}

fn declaration_scope(node: &AstNode<'_>, ctx: &LintContext<'_>) -> ScopeId {
    if let AstKind::VariableDeclaration(declaration) = node.kind()
        && declaration.kind == VariableDeclarationKind::Var
    {
        return ctx
            .scoping()
            .scope_ancestors(node.scope_id())
            .find(|&scope_id| ctx.scoping().scope_flags(scope_id).is_var())
            .unwrap_or_else(|| ctx.scoping().root_scope_id());
    }
    node.scope_id()
}

fn is_statement_list_parent(kind: AstKind<'_>) -> bool {
    matches!(
        kind,
        AstKind::Program(_)
            | AstKind::BlockStatement(_)
            | AstKind::FunctionBody(_)
            | AstKind::StaticBlock(_)
            | AstKind::SwitchCase(_)
            | AstKind::TSModuleBlock(_)
    )
}

fn statement_node_id(node: &AstNode<'_>, ctx: &LintContext<'_>) -> Option<NodeId> {
    let parent = ctx.nodes().parent_node(node.id());
    if is_statement_list_parent(parent.kind()) {
        Some(node.id())
    } else if matches!(parent.kind(), AstKind::ExportNamedDeclaration(_)) {
        let grandparent = ctx.nodes().parent_node(parent.id());
        is_statement_list_parent(grandparent.kind()).then_some(parent.id())
    } else {
        None
    }
}

fn previous_declaration<'a, 'c>(
    node: &AstNode<'a>,
    ctx: &'c LintContext<'a>,
) -> Option<&'c VariableDeclaration<'a>> {
    let nodes = ctx.nodes();
    let parent = nodes.parent_node(node.id());
    let (statement_span, statement_parent) = if is_statement_list_parent(parent.kind()) {
        (node.span(), parent)
    } else if matches!(parent.kind(), AstKind::ExportNamedDeclaration(_)) {
        let grandparent = nodes.parent_node(parent.id());
        if !is_statement_list_parent(grandparent.kind()) {
            return None;
        }
        (parent.span(), grandparent)
    } else {
        return None;
    };

    let statements: &[Statement<'a>] = match statement_parent.kind() {
        AstKind::Program(program) => &program.body,
        AstKind::BlockStatement(block) => &block.body,
        AstKind::FunctionBody(body) => &body.statements,
        AstKind::StaticBlock(block) => &block.body,
        AstKind::SwitchCase(case) => &case.consequent,
        AstKind::TSModuleBlock(block) => &block.body,
        _ => unreachable!(),
    };
    let index = statements
        .binary_search_by_key(&statement_span.start, |statement| statement.span().start)
        .ok()?;
    match index.checked_sub(1).and_then(|index| statements.get(index))? {
        Statement::VariableDeclaration(declaration) => Some(declaration),
        _ => None,
    }
}

fn is_for_statement_init(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    matches!(ctx.nodes().parent_kind(node.id()), AstKind::ForStatement(statement) if statement.init.as_ref().is_some_and(|init| init.span() == node.span()))
}

fn is_for_in_or_of_left(node: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    matches!(
        ctx.nodes().parent_kind(node.id()),
        AstKind::ForInStatement(_) | AstKind::ForOfStatement(_)
    )
}

fn report_join_with_optional_previous<'a>(
    ctx: &LintContext<'a>,
    node: &AstNode<'a>,
    declaration: &VariableDeclaration<'a>,
    message: String,
) {
    if let Some(previous) =
        previous_declaration(node, ctx).filter(|previous| previous.kind == declaration.kind)
    {
        report_join(ctx, declaration, previous, message);
    } else {
        ctx.diagnostic(one_var_diagnostic(declaration.span, message));
    }
}

fn report_join(
    ctx: &LintContext<'_>,
    declaration: &VariableDeclaration<'_>,
    previous: &VariableDeclaration<'_>,
    message: String,
) {
    ctx.diagnostic_with_fix(one_var_diagnostic(declaration.span, message), |_fixer| {
        let source = ctx.source_text();
        let previous_source = previous.span.source_text(source);
        let mut fixes = Vec::with_capacity(3);
        if let Some(index) = previous_source.rfind(';') {
            let start = previous.span.start + u32::try_from(index).unwrap();
            fixes.push(Fix::new(",", Span::sized(start, 1)));
        } else {
            fixes.push(Fix::new(",", Span::empty(previous.span.end)));
        }
        let keyword = declaration.kind.as_str();
        let declaration_source = declaration.span.source_text(source);
        if declaration.kind == VariableDeclarationKind::AwaitUsing {
            fixes.push(Fix::delete(Span::sized(declaration.span.start, 5)));
            let using_offset = declaration_source.find("using").unwrap();
            fixes.push(Fix::delete(Span::sized(
                declaration.span.start + u32::try_from(using_offset).unwrap(),
                5,
            )));
        } else if let Some(offset) = declaration_source.find(keyword) {
            fixes.push(Fix::delete(Span::sized(
                declaration.span.start + u32::try_from(offset).unwrap(),
                u32::try_from(keyword.len()).unwrap(),
            )));
        }
        fixes.into_iter().collect::<RuleFix>().with_message("Combine variable declarations")
    });
}

fn report_split(
    ctx: &LintContext<'_>,
    node: &AstNode<'_>,
    declaration: &VariableDeclaration<'_>,
    message: String,
) {
    let diagnostic = one_var_diagnostic(declaration.span, message);
    if statement_node_id(node, ctx).is_none() {
        ctx.diagnostic(diagnostic);
        return;
    }
    ctx.diagnostic_with_fix(diagnostic, |_fixer| {
        let source = ctx.source_text();
        let keyword = declaration.kind.as_str();
        let exported =
            matches!(ctx.nodes().parent_kind(node.id()), AstKind::ExportNamedDeclaration(_));
        let prefix = if exported { format!("export {keyword} ") } else { format!("{keyword} ") };
        let mut fixes = Vec::with_capacity((declaration.declarations.len() - 1) * 2);
        for pair in declaration.declarations.windows(2) {
            let left = &pair[0];
            let right = &pair[1];
            let gap = &source[left.span.end as usize..right.span.start as usize];
            if let Some(index) = gap.find(',') {
                let comma = left.span.end + u32::try_from(index).unwrap();
                let separator = if comma + 1 == right.span.start { "; " } else { ";" };
                fixes.push(Fix::new(separator, Span::sized(comma, 1)));
                fixes.push(Fix::new(prefix.clone(), Span::empty(right.span.start)));
            }
        }
        fixes.into_iter().collect::<RuleFix>().with_message("Split variable declarations")
    });
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function foo() { var bar = true; }", None),
        ("function foo() { var bar = true, baz = 1; if (qux) { bar = false; } }", None),
        ("var foo = function() { var bar = true; baz(); }", None),
        ("function foo() { var bar = true, baz = false; }", Some(serde_json::json!(["always"]))),
        ("function foo() { var bar = true; var baz = false; }", Some(serde_json::json!(["never"]))),
        ("for (var i = 0, len = arr.length; i < len; i++) {}", Some(serde_json::json!(["never"]))),
        ("var bar = true; var baz = false;", Some(serde_json::json!([{ "initialized": "never" }]))),
        ("var bar = true, baz = false;", Some(serde_json::json!([{ "initialized": "always" }]))),
        ("var bar, baz;", Some(serde_json::json!([{ "initialized": "never" }]))),
        ("var bar; var baz;", Some(serde_json::json!([{ "uninitialized": "never" }]))),
        ("var bar, baz;", Some(serde_json::json!([{ "uninitialized": "always" }]))),
        ("var bar = true, baz = false;", Some(serde_json::json!([{ "uninitialized": "never" }]))),
        (
            "var bar = true, baz = false, a, b;",
            Some(serde_json::json!([{ "uninitialized": "always", "initialized": "always" }])),
        ),
        (
            "var bar = true; var baz = false; var a; var b;",
            Some(serde_json::json!([{ "uninitialized": "never", "initialized": "never" }])),
        ),
        (
            "var bar, baz; var a = true; var b = false;",
            Some(serde_json::json!([{ "uninitialized": "always", "initialized": "never" }])),
        ),
        (
            "var bar = true, baz = false; var a; var b;",
            Some(serde_json::json!([{ "uninitialized": "never", "initialized": "always" }])),
        ),
        (
            "var bar; var baz; var a = true, b = false;",
            Some(serde_json::json!([{ "uninitialized": "never", "initialized": "always" }])),
        ),
        (
            "function foo() { var a = [1, 2, 3]; var [b, c, d] = a; }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "function foo() { let a = 1; var c = true; if (a) {let c = true; } }",
            Some(serde_json::json!(["always"])),
        ),
        (
            "function foo() { const a = 1; var c = true; if (a) {const c = true; } }",
            Some(serde_json::json!(["always"])),
        ),
        (
            "function foo() { if (true) { const a = 1; }; if (true) {const a = true; } }",
            Some(serde_json::json!(["always"])),
        ),
        ("function foo() { let a = 1; let b = true; }", Some(serde_json::json!(["never"]))),
        ("function foo() { const a = 1; const b = true; }", Some(serde_json::json!(["never"]))),
        (
            "function foo() { let a = 1; const b = false; var c = true; }",
            Some(serde_json::json!(["always"])),
        ),
        (
            "function foo() { let a = 1, b = false; var c = true; }",
            Some(serde_json::json!(["always"])),
        ),
        (
            "function foo() { let a = 1; let b = 2; const c = false; const d = true; var e = true, f = false; }",
            Some(serde_json::json!([{ "var": "always", "let": "never", "const": "never" }])),
        ),
        (
            "let foo = true; for (let i = 0; i < 1; i++) { let foo = false; }",
            Some(serde_json::json!([{ "var": "always", "let": "always", "const": "never" }])),
        ),
        (
            "let foo = true; for (let i = 0; i < 1; i++) { let foo = false; }",
            Some(serde_json::json!([{ "var": "always" }])),
        ),
        ("let foo = true, bar = false;", Some(serde_json::json!([{ "var": "never" }]))),
        ("let foo = true, bar = false;", Some(serde_json::json!([{ "const": "never" }]))),
        ("let foo = true, bar = false;", Some(serde_json::json!([{ "uninitialized": "never" }]))),
        ("let foo, bar", Some(serde_json::json!([{ "initialized": "never" }]))),
        (
            "let foo = true, bar = false; let a; let b;",
            Some(serde_json::json!([{ "uninitialized": "never" }])),
        ),
        (
            "let foo, bar; let a = true; let b = true;",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ),
        (
            "var foo, bar; const a=1; const b=2; let c, d",
            Some(serde_json::json!([{ "var": "always", "let": "always" }])),
        ),
        (
            "var foo; var bar; const a=1, b=2; let c; let d",
            Some(serde_json::json!([{ "const": "always" }])),
        ),
        (
            "for (let x of foo) {}; for (let y of foo) {}",
            Some(serde_json::json!([{ "uninitialized": "always" }])),
        ),
        (
            "for (let x in foo) {}; for (let y in foo) {}",
            Some(serde_json::json!([{ "uninitialized": "always" }])),
        ),
        (
            "var x; for (var y in foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var x, y; for (y in foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var x, y; for (var z in foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var x; for (var y in foo) {var bar = y; for (var z in bar) {}}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var a = 1; var b = 2; var x, y; for (var z in foo) {var baz = z; for (var d in baz) {}}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var x; for (var y of foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var x, y; for (y of foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var x, y; for (var z of foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var x; for (var y of foo) {var bar = y; for (var z of bar) {}}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var a = 1; var b = 2; var x, y; for (var z of foo) {var baz = z; for (var d of baz) {}}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var foo = require('foo'), bar;",
            Some(serde_json::json!([{ "separateRequires": false, "var": "always" }])),
        ),
        (
            "var foo = require('foo'), bar = require('bar');",
            Some(serde_json::json!([{ "separateRequires": true, "var": "always" }])),
        ),
        (
            "var bar = 'bar'; var foo = require('foo');",
            Some(serde_json::json!([{ "separateRequires": true, "var": "always" }])),
        ),
        (
            "var foo = require('foo'); var bar = 'bar';",
            Some(serde_json::json!([{ "separateRequires": true, "var": "always" }])),
        ),
        ("var a = 0, b, c;", Some(serde_json::json!(["consecutive"]))),
        ("var a = 0, b = 1, c = 2;", Some(serde_json::json!(["consecutive"]))),
        ("var a = 0, b = 1; foo(); var c = 2;", Some(serde_json::json!(["consecutive"]))),
        ("let a = 0, b, c;", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 6 },
        ("let a = 0, b = 1, c = 2;", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 6 },
        ("let a = 0, b = 1; foo(); let c = 2;", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 6 },
        ("const a = 0, b = 1; foo(); const c = 2;", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 6 },
        ("const a = 0; var b = 1;", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 6 },
        ("const a = 0; let b = 1;", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 6 },
        ("let a = 0; const b = 1; var c = 2;", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 6 },
        (
            "const foo = require('foo'); const bar = 'bar';",
            Some(serde_json::json!([{ "const": "consecutive", "separateRequires": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "var a = 0, b = 1; var c, d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ),
        (
            "var a = 0; var b, c; var d = 1;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ),
        (
            "let a = 0, b = 1; let c, d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a = 0; let b, c; let d = 1;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "const a = 0, b = 1; let c, d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "const a = 0; let b, c; const d = 1;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var a = 0, b = 1; var c; var d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ),
        (
            "var a = 0; var b; var c; var d = 1;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ),
        (
            "let a = 0, b = 1; let c; let d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a = 0; let b; let c; let d = 1;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "const a = 0, b = 1; let c; let d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "const a = 0; let b; let c; const d = 1;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var a, b; var c = 0, d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ),
        (
            "var a; var b = 0, c = 1; var d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ),
        (
            "let a, b; let c = 0, d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a; let b = 0, c = 1; let d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a, b; const c = 0, d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a; const b = 0, c = 1; let d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var a, b; var c = 0; var d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ),
        (
            "var a; var b = 0; var c = 1; var d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ),
        (
            "let a, b; let c = 0; let d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a; let b = 0; let c = 1; let d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a, b; const c = 0; const d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a; const b = 0; const c = 1; let d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ), // { "ecmaVersion": 6 },
        ("var a = 0, b = 1;", Some(serde_json::json!([{ "var": "consecutive" }]))),
        ("var a = 0; foo; var b = 1;", Some(serde_json::json!([{ "var": "consecutive" }]))),
        ("let a = 0, b = 1;", Some(serde_json::json!([{ "let": "consecutive" }]))), // { "ecmaVersion": 6 },
        ("let a = 0; foo; let b = 1;", Some(serde_json::json!([{ "let": "consecutive" }]))), // { "ecmaVersion": 6 },
        ("const a = 0, b = 1;", Some(serde_json::json!([{ "const": "consecutive" }]))), // { "ecmaVersion": 6 },
        ("const a = 0; foo; const b = 1;", Some(serde_json::json!([{ "const": "consecutive" }]))), // { "ecmaVersion": 6 },
        (
            "let a, b; const c = 0, d = 1;",
            Some(serde_json::json!([{ "let": "consecutive", "const": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a; const b = 0, c = 1; let d;",
            Some(serde_json::json!([{ "let": "consecutive", "const": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a, b; const c = 0; const d = 1;",
            Some(serde_json::json!([{ "let": "consecutive", "const": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a; const b = 0; const c = 1; let d;",
            Some(serde_json::json!([{ "let": "consecutive", "const": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "const a = 0, b = 1; let c, d;",
            Some(serde_json::json!([{ "const": "consecutive", "let": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "const a = 0; let b, c; const d = 1;",
            Some(serde_json::json!([{ "const": "consecutive", "let": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "const a = 0, b = 1; let c; let d;",
            Some(serde_json::json!([{ "const": "consecutive", "let": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "const a = 0; let b; let c; const d = 1;",
            Some(serde_json::json!([{ "const": "consecutive", "let": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var a = 1, b = 2; foo(); var c = 3, d = 4;",
            Some(serde_json::json!([{ "initialized": "consecutive" }])),
        ),
        ("var bar, baz;", Some(serde_json::json!(["consecutive"]))),
        (
            "var bar = 1, baz = 2; qux(); var qux = 3, quux;",
            Some(serde_json::json!(["consecutive"])),
        ),
        (
            "let a, b; var c; var d; let e;",
            Some(
                serde_json::json!([ { "var": "never", "let": "consecutive", "const": "consecutive" }, ]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "const a = 1, b = 2; var d; var e; const f = 3;",
            Some(
                serde_json::json!([ { "var": "never", "let": "consecutive", "const": "consecutive" }, ]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "var a, b; const c = 1; const d = 2; let e; let f; ",
            Some(serde_json::json!([{ "var": "consecutive" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var a = 1, b = 2; var c; var d; var e = 3, f = 4;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ),
        ("var a; somethingElse(); var b;", Some(serde_json::json!([{ "var": "never" }]))),
        (
            "var foo = 1;
            let bar = function() { var x; };
            var baz = 2;",
            Some(serde_json::json!([{ "var": "never" }])),
        ),
        ("class C { static { var a; let b; const c = 0; } }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("const a = 0; class C { static { const b = 0; } }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { static { const b = 0; } } const a = 0; ", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("let a; class C { static { let b; } }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { static { let b; } } let a;", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("var a; class C { static { var b; } }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { static { var b; } } var a; ", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("var a; class C { static { if (foo) { var b; } } }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { static { if (foo) { var b; } } } var a; ", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        (
            "class C { static { const a = 0; if (foo) { const b = 0; } } }",
            Some(serde_json::json!(["always"])),
        ), // { "ecmaVersion": 2022 },
        ("class C { static { let a; if (foo) { let b; } } }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { static { const a = 0; const b = 0; } }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { static { let a; let b; } }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { static { var a; var b; } }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { static { let a; foo; let b; } }", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 2022 },
        (
            "class C { static { let a; const b = 0; let c; } }",
            Some(serde_json::json!(["consecutive"])),
        ), // { "ecmaVersion": 2022 },
        ("class C { static { var a; foo; var b; } }", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 2022 },
        ("class C { static { var a; let b; var c; } }", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 2022 },
        (
            "class C { static { let a; if (foo) { let b; } } }",
            Some(serde_json::json!(["consecutive"])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { if (foo) { let b; } let a;  } }",
            Some(serde_json::json!(["consecutive"])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { const a = 0; if (foo) { const b = 0; } } }",
            Some(serde_json::json!(["consecutive"])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { if (foo) { const b = 0; } const a = 0; } }",
            Some(serde_json::json!(["consecutive"])),
        ), // { "ecmaVersion": 2022 },
        ("class C { static { var a; if (foo) var b; } }", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 2022 },
        ("class C { static { if (foo) var b; var a; } }", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 2022 },
        (
            "class C { static { if (foo) { var b; } var a; } }",
            Some(serde_json::json!(["consecutive"])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { let a; let b = 0; } }",
            Some(serde_json::json!([{ "initialized": "consecutive" }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { var a; var b = 0; } }",
            Some(serde_json::json!([{ "initialized": "consecutive" }])),
        ), // { "ecmaVersion": 2022 },
        ("using a = 0; let b = 1; const c = 2;", None), // { "ecmaVersion": 2026, "sourceType": "module", },
        ("await using a = 0; let b = 1; const c = 2;", None), // { "ecmaVersion": 2026, "sourceType": "module", },
        ("using a = 0, b = 1;", None), // { "ecmaVersion": 2026, "sourceType": "module", },
        ("await using a = 0, b = 1;", None), // { "ecmaVersion": 2026, "sourceType": "module", },
        ("function fn() { { using a = 0; } using b = 1; }", None), // { "ecmaVersion": 2026, "sourceType": "module", },
        ("using a = 0; using b = 1;", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2026, "sourceType": "module", },
        ("await using a = 0; await using b = 1;", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2026, "sourceType": "module", },
        ("using a = 0, b = 1;", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 2026, "sourceType": "module", },
        ("await using a = 0, b = 1;", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 2026, "sourceType": "module", },
        ("using a = 0, b = 1;", Some(serde_json::json!([{ "initialized": "always" }]))), // { "ecmaVersion": 2026, "sourceType": "module", },
        ("await using a = 0, b = 1;", Some(serde_json::json!([{ "initialized": "always" }]))), // { "ecmaVersion": 2026, "sourceType": "module", },
        ("using a = 0; using b = 1;", Some(serde_json::json!([{ "initialized": "never" }]))), // { "ecmaVersion": 2026, "sourceType": "module", },
        (
            "await using a = 0; await using b = 1;",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ), // { "ecmaVersion": 2026, "sourceType": "module", },
        (
            "using a = 0, b = 1; foo(); using c = 2, d = 3;",
            Some(serde_json::json!([{ "initialized": "consecutive" }])),
        ), // { "ecmaVersion": 2026, "sourceType": "module", },
        (
            "await using a = 0, b = 1; foo(); await using c = 2, d = 3;",
            Some(serde_json::json!([{ "initialized": "consecutive" }])),
        ), // { "ecmaVersion": 2026, "sourceType": "module", }
    ];
    let fail = vec![
        ("var bar = true, baz = false;", Some(serde_json::json!(["never"]))),
        ("function foo() { var bar = true, baz = false; }", Some(serde_json::json!(["never"]))),
        ("if (foo) { var bar = true, baz = false; }", Some(serde_json::json!(["never"]))),
        (
            "switch (foo) { case bar: var baz = true, quux = false; }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "switch (foo) { default: var baz = true, quux = false; }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "function foo() { var bar = true; var baz = false; }",
            Some(serde_json::json!(["always"])),
        ),
        ("var a = 1; for (var b = 2;;) {}", Some(serde_json::json!(["always"]))),
        (
            "function foo() { var foo = true, bar = false; }",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ),
        (
            "function foo() { var foo, bar; }",
            Some(serde_json::json!([{ "uninitialized": "never" }])),
        ),
        (
            "function foo() { var bar, baz; var a = true; var b = false; var c, d;}",
            Some(serde_json::json!([{ "uninitialized": "always", "initialized": "never" }])),
        ),
        (
            "function foo() { var bar = true, baz = false; var a; var b; var c = true, d = false; }",
            Some(serde_json::json!([{ "uninitialized": "never", "initialized": "always" }])),
        ),
        (
            "function foo() { var bar = true, baz = false; var a, b;}",
            Some(serde_json::json!([{ "uninitialized": "never", "initialized": "never" }])),
        ),
        (
            "function foo() { var bar = true; var baz = false; var a; var b;}",
            Some(serde_json::json!([{ "uninitialized": "always", "initialized": "always" }])),
        ),
        (
            "function foo() { var a = [1, 2, 3]; var [b, c, d] = a; }",
            Some(serde_json::json!(["always"])),
        ),
        ("function foo() { let a = 1; let b = 2; }", Some(serde_json::json!(["always"]))),
        ("function foo() { const a = 1; const b = 2; }", Some(serde_json::json!(["always"]))),
        (
            "function foo() { let a = 1; let b = 2; }",
            Some(serde_json::json!([{ "let": "always" }])),
        ),
        (
            "function foo() { const a = 1; const b = 2; }",
            Some(serde_json::json!([{ "const": "always" }])),
        ),
        ("function foo() { let a = 1, b = 2; }", Some(serde_json::json!([{ "let": "never" }]))),
        (
            "function foo() { let a = 1, b = 2; }",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ),
        ("function foo() { let a, b; }", Some(serde_json::json!([{ "uninitialized": "never" }]))),
        (
            "function foo() { const a = 1, b = 2; }",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ),
        ("function foo() { const a = 1, b = 2; }", Some(serde_json::json!([{ "const": "never" }]))),
        (
            "let foo = true; switch(foo) { case true: let bar = 2; break; case false: let baz = 3; break; }",
            Some(serde_json::json!([{ "var": "always", "let": "always", "const": "never" }])),
        ),
        (
            "var one = 1, two = 2;
            var three;",
            Some(serde_json::json!(["always"])),
        ),
        ("var i = [0], j;", Some(serde_json::json!([{ "initialized": "never" }]))),
        ("var i = [0], j;", Some(serde_json::json!([{ "uninitialized": "never" }]))),
        ("for (var x of foo) {}; for (var y of foo) {}", Some(serde_json::json!(["always"]))),
        ("for (var x in foo) {}; for (var y in foo) {}", Some(serde_json::json!(["always"]))),
        ("var foo = function() { var bar = true; var baz = false; }", None),
        (
            "function foo() { var bar = true; if (qux) { var baz = false; } else { var quxx = 42; } }",
            None,
        ),
        ("var foo = () => { var bar = true; var baz = false; }", None), // { "ecmaVersion": 6 },
        ("var foo = function() { var bar = true; if (qux) { var baz = false; } }", None),
        ("var foo; var bar;", None),
        (
            "var x = 1, y = 2; for (var z in foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var x = 1, y = 2; for (var z of foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var x; var y; for (var z in foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var x; var y; for (var z of foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var x; for (var y in foo) {var bar = y; var a; for (var z of bar) {}}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var a = 1; var b = 2; var x, y; for (var z of foo) {var c = 3, baz = z; for (var d in baz) {}}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        ("var {foo} = 1, [bar] = 2;", Some(serde_json::json!([{ "initialized": "never" }]))), // { "ecmaVersion": 6 },
        (
            "const foo = 1,
                bar = 2;",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = 1,
                bar = 2;",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ),
        (
            "var foo = 1, // comment
                bar = 2;",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ),
        ("var f, k /* test */, l;", Some(serde_json::json!(["never"]))),
        ("var f,          /* test */ l;", Some(serde_json::json!(["never"]))),
        (
            "var f, k /* test 
             some more comment 
             even more */, l = 1, P;",
            Some(serde_json::json!(["never"])),
        ),
        ("var a = 1, b = 2", Some(serde_json::json!(["never"]))),
        (
            "var foo = require('foo'), bar;",
            Some(serde_json::json!([{ "separateRequires": true, "var": "always" }])),
        ),
        (
            "var foo, bar = require('bar');",
            Some(serde_json::json!([{ "separateRequires": true, "var": "always" }])),
        ),
        (
            "let foo, bar = require('bar');",
            Some(serde_json::json!([{ "separateRequires": true, "let": "always" }])),
        ),
        (
            "const foo = 0, bar = require('bar');",
            Some(serde_json::json!([{ "separateRequires": true, "const": "always" }])),
        ),
        (
            "const foo = require('foo'); const bar = require('bar');",
            Some(serde_json::json!([{ "separateRequires": true, "const": "always" }])),
        ),
        ("var a = 1, b; var c;", Some(serde_json::json!(["consecutive"]))),
        ("var a = 0, b = 1; var c = 2;", Some(serde_json::json!(["consecutive"]))),
        ("let a = 1, b; let c;", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 6 },
        ("let a = 0, b = 1; let c = 2;", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 6 },
        ("const a = 0, b = 1; const c = 2;", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 6 },
        (
            "const a = 0; var b = 1; var c = 2; const d = 3;",
            Some(serde_json::json!(["consecutive"])),
        ), // { "ecmaVersion": 6 },
        (
            "var a = true; var b = false;",
            Some(serde_json::json!([{ "separateRequires": true, "var": "always" }])),
        ),
        (
            "const a = 0; let b = 1; let c = 2; const d = 3;",
            Some(serde_json::json!(["consecutive"])),
        ), // { "ecmaVersion": 6 },
        (
            "let a = 0; const b = 1; const c = 1; var d = 2;",
            Some(serde_json::json!(["consecutive"])),
        ), // { "ecmaVersion": 6 },
        (
            "var a = 0; var b; var c; var d = 1",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ),
        (
            "var a = 0; var b = 1; var c; var d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ),
        (
            "let a = 0; let b; let c; let d = 1;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a = 0; let b = 1; let c; let d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "const a = 0; let b; let c; const d = 1;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "const a = 0; const b = 1; let c; let d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var a = 0; var b = 1; var c, d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ),
        (
            "var a = 0; var b, c; var d = 1;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ),
        (
            "let a = 0; let b = 1; let c, d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a = 0; let b, c; let d = 1;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "const a = 0; const b = 1; let c, d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "const a = 0; let b, c; const d = 1;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var a; var b; var c = 0; var d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ),
        (
            "var a; var b = 0; var c = 1; var d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ),
        (
            "let a; let b; let c = 0; let d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a; let b = 0; let c = 1; let d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a; let b; const c = 0; const d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a; const b = 0; const c = 1; let d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var a; var b; var c = 0, d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ),
        (
            "var a; var b = 0, c = 1; var d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ),
        (
            "let a; let b; let c = 0, d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a; let b = 0, c = 1; let d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a; let b; const c = 0, d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a; const b = 0, c = 1; let d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ), // { "ecmaVersion": 6 },
        ("var a = 0; var b = 1;", Some(serde_json::json!([{ "var": "consecutive" }]))),
        ("let a = 0; let b = 1;", Some(serde_json::json!([{ "let": "consecutive" }]))), // { "ecmaVersion": 6 },
        ("const a = 0; const b = 1;", Some(serde_json::json!([{ "const": "consecutive" }]))), // { "ecmaVersion": 6 },
        (
            "let a; let b; const c = 0; const d = 1;",
            Some(serde_json::json!([{ "let": "consecutive", "const": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a; const b = 0; const c = 1; let d;",
            Some(serde_json::json!([{ "let": "consecutive", "const": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a; let b; const c = 0, d = 1;",
            Some(serde_json::json!([{ "let": "consecutive", "const": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "let a; const b = 0, c = 1; let d;",
            Some(serde_json::json!([{ "let": "consecutive", "const": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "const a = 0; const b = 1; let c; let d;",
            Some(serde_json::json!([{ "const": "consecutive", "let": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "const a = 0; let b; let c; const d = 1;",
            Some(serde_json::json!([{ "const": "consecutive", "let": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "const a = 0; const b = 1; let c, d;",
            Some(serde_json::json!([{ "const": "consecutive", "let": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "const a = 0; let b, c; const d = 1;",
            Some(serde_json::json!([{ "const": "consecutive", "let": "never" }])),
        ), // { "ecmaVersion": 6 },
        ("var bar; var baz;", Some(serde_json::json!(["consecutive"]))),
        (
            "var bar = 1; var baz = 2; qux(); var qux = 3; var quux;",
            Some(serde_json::json!(["consecutive"])),
        ),
        (
            "let a, b; let c; var d, e;",
            Some(
                serde_json::json!([ { "var": "never", "let": "consecutive", "const": "consecutive" }, ]),
            ),
        ), // { "ecmaVersion": 6 },
        ("var a; var b;", Some(serde_json::json!([{ "var": "consecutive" }]))),
        (
            "var a = 1; var b = 2; var c, d; var e = 3; var f = 4;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ),
        (
            "var a = 1; var b = 2; foo(); var c = 3; var d = 4;",
            Some(serde_json::json!([{ "initialized": "consecutive" }])),
        ),
        (
            "var a
            var b",
            Some(serde_json::json!(["always"])),
        ),
        ("export const foo=1, bar=2;", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2021, "sourceType": "module" },
        (
            "const foo=1,
             bar=2;",
            Some(serde_json::json!(["never"])),
        ), // { "ecmaVersion": 2021, "sourceType": "module" },
        (
            "export const foo=1,
             bar=2;",
            Some(serde_json::json!(["never"])),
        ), // { "ecmaVersion": 2021, "sourceType": "module" },
        (
            "export const foo=1
            , bar=2;",
            Some(serde_json::json!(["never"])),
        ), // { "ecmaVersion": 2021, "sourceType": "module" },
        ("export const foo= a, bar=2;", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2021, "sourceType": "module" },
        ("export const foo=() => a, bar=2;", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2021, "sourceType": "module" },
        ("export const foo= a, bar=2, bar2=2;", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2021, "sourceType": "module" },
        ("export const foo = 1,bar = 2;", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2021, "sourceType": "module" },
        ("if (foo) var x, y;", Some(serde_json::json!(["never"]))),
        ("if (foo) var x, y;", Some(serde_json::json!([{ "var": "never" }]))),
        ("if (foo) var x, y;", Some(serde_json::json!([{ "uninitialized": "never" }]))),
        ("if (foo) var x = 1, y = 1;", Some(serde_json::json!([{ "initialized": "never" }]))),
        ("if (foo) {} else var x, y;", Some(serde_json::json!(["never"]))),
        ("while (foo) var x, y;", Some(serde_json::json!(["never"]))),
        ("do var x, y; while (foo);", Some(serde_json::json!(["never"]))),
        ("do var x = f(), y = b(); while (x < y);", Some(serde_json::json!(["never"]))),
        ("for (;;) var x, y;", Some(serde_json::json!(["never"]))),
        ("for (foo in bar) var x, y;", Some(serde_json::json!(["never"]))),
        ("for (foo of bar) var x, y;", Some(serde_json::json!(["never"]))),
        ("with (foo) var x, y;", Some(serde_json::json!(["never"]))),
        ("label: var x, y;", Some(serde_json::json!(["never"]))),
        ("class C { static { let x, y; } }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { static { var x, y; } }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { static { let x; let y; } }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { static { var x; var y; } }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { static { let x; foo; let y; } }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { static { var x; foo; var y; } }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { static { var x; if (foo) { var y; } } }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { static { let x; let y; } }", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 2022 },
        ("class C { static { var x; var y; } }", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 2022 },
        (
            "class C { static { let a = 0; let b = 1; } }",
            Some(serde_json::json!([{ "initialized": "consecutive" }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { static { var a = 0; var b = 1; } }",
            Some(serde_json::json!([{ "initialized": "consecutive" }])),
        ), // { "ecmaVersion": 2022 },
        ("using a = 0; using b = 1;", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2026, "sourceType": "module", },
        ("await using a = 0; await using b = 1;", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2026, "sourceType": "module", },
        ("using a = 0, b = 1;", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2026, "sourceType": "module", },
        ("await using a = 0, b = 1;", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2026, "sourceType": "module", },
        ("using a = 0; using b = 1;", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 2026, "sourceType": "module", },
        ("await using a = 0; await using b = 1;", Some(serde_json::json!(["consecutive"]))), // { "ecmaVersion": 2026, "sourceType": "module", },
        ("using a = 0, b = 1;", Some(serde_json::json!([{ "initialized": "never" }]))), // { "ecmaVersion": 2026, "sourceType": "module", },
        ("await using a = 0, b = 1;", Some(serde_json::json!([{ "initialized": "never" }]))), // { "ecmaVersion": 2026, "sourceType": "module", }
    ];
    let fix = vec![
        (
            "var bar = true, baz = false;",
            "var bar = true; var baz = false;",
            Some(serde_json::json!(["never"])),
        ),
        (
            "function foo() { var bar = true, baz = false; }",
            "function foo() { var bar = true; var baz = false; }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "if (foo) { var bar = true, baz = false; }",
            "if (foo) { var bar = true; var baz = false; }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "switch (foo) { case bar: var baz = true, quux = false; }",
            "switch (foo) { case bar: var baz = true; var quux = false; }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "switch (foo) { default: var baz = true, quux = false; }",
            "switch (foo) { default: var baz = true; var quux = false; }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "function foo() { var bar = true; var baz = false; }",
            "function foo() { var bar = true,  baz = false; }",
            Some(serde_json::json!(["always"])),
        ),
        (
            "function foo() { var foo = true, bar = false; }",
            "function foo() { var foo = true; var bar = false; }",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ),
        (
            "function foo() { var foo, bar; }",
            "function foo() { var foo; var bar; }",
            Some(serde_json::json!([{ "uninitialized": "never" }])),
        ),
        (
            "function foo() { var bar, baz; var a = true; var b = false; var c, d;}",
            "function foo() { var bar, baz; var a = true; var b = false,  c, d;}",
            Some(serde_json::json!([{ "uninitialized": "always", "initialized": "never" }])),
        ),
        (
            "function foo() { var bar = true, baz = false; var a; var b; var c = true, d = false; }",
            "function foo() { var bar = true, baz = false; var a; var b,  c = true, d = false; }",
            Some(serde_json::json!([{ "uninitialized": "never", "initialized": "always" }])),
        ),
        (
            "function foo() { var bar = true, baz = false; var a, b;}",
            "function foo() { var bar = true; var baz = false; var a; var b;}",
            Some(serde_json::json!([{ "uninitialized": "never", "initialized": "never" }])),
        ),
        (
            "function foo() { var bar = true; var baz = false; var a; var b;}",
            "function foo() { var bar = true,  baz = false,  a,  b;}",
            Some(serde_json::json!([{ "uninitialized": "always", "initialized": "always" }])),
        ),
        (
            "function foo() { var a = [1, 2, 3]; var [b, c, d] = a; }",
            "function foo() { var a = [1, 2, 3],  [b, c, d] = a; }",
            Some(serde_json::json!(["always"])),
        ),
        (
            "function foo() { let a = 1; let b = 2; }",
            "function foo() { let a = 1,  b = 2; }",
            Some(serde_json::json!(["always"])),
        ),
        (
            "function foo() { const a = 1; const b = 2; }",
            "function foo() { const a = 1,  b = 2; }",
            Some(serde_json::json!(["always"])),
        ),
        (
            "function foo() { let a = 1; let b = 2; }",
            "function foo() { let a = 1,  b = 2; }",
            Some(serde_json::json!([{ "let": "always" }])),
        ),
        (
            "function foo() { const a = 1; const b = 2; }",
            "function foo() { const a = 1,  b = 2; }",
            Some(serde_json::json!([{ "const": "always" }])),
        ),
        (
            "function foo() { let a = 1, b = 2; }",
            "function foo() { let a = 1; let b = 2; }",
            Some(serde_json::json!([{ "let": "never" }])),
        ),
        (
            "function foo() { let a = 1, b = 2; }",
            "function foo() { let a = 1; let b = 2; }",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ),
        (
            "function foo() { let a, b; }",
            "function foo() { let a; let b; }",
            Some(serde_json::json!([{ "uninitialized": "never" }])),
        ),
        (
            "function foo() { const a = 1, b = 2; }",
            "function foo() { const a = 1; const b = 2; }",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ),
        (
            "function foo() { const a = 1, b = 2; }",
            "function foo() { const a = 1; const b = 2; }",
            Some(serde_json::json!([{ "const": "never" }])),
        ),
        (
            "var one = 1, two = 2;
            var three;",
            "var one = 1, two = 2,
             three;",
            Some(serde_json::json!(["always"])),
        ),
        (
            "var i = [0], j;",
            "var i = [0]; var j;",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ),
        (
            "var i = [0], j;",
            "var i = [0]; var j;",
            Some(serde_json::json!([{ "uninitialized": "never" }])),
        ),
        (
            "var foo = function() { var bar = true; var baz = false; }",
            "var foo = function() { var bar = true,  baz = false; }",
            None,
        ),
        (
            "var foo = () => { var bar = true; var baz = false; }",
            "var foo = () => { var bar = true,  baz = false; }",
            None,
        ),
        ("var foo; var bar;", "var foo,  bar;", None),
        (
            "var x = 1, y = 2; for (var z in foo) {}",
            "var x = 1; var y = 2; for (var z in foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var x = 1, y = 2; for (var z of foo) {}",
            "var x = 1; var y = 2; for (var z of foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var x; var y; for (var z in foo) {}",
            "var x,  y; for (var z in foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var x; var y; for (var z of foo) {}",
            "var x,  y; for (var z of foo) {}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var x; for (var y in foo) {var bar = y; var a; for (var z of bar) {}}",
            "var x; for (var y in foo) {var bar = y,  a; for (var z of bar) {}}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var a = 1; var b = 2; var x, y; for (var z of foo) {var c = 3, baz = z; for (var d in baz) {}}",
            "var a = 1; var b = 2; var x, y; for (var z of foo) {var c = 3; var baz = z; for (var d in baz) {}}",
            Some(serde_json::json!([{ "initialized": "never", "uninitialized": "always" }])),
        ),
        (
            "var {foo} = 1, [bar] = 2;",
            "var {foo} = 1; var [bar] = 2;",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ),
        (
            "const foo = 1,
                bar = 2;",
            "const foo = 1;
                const bar = 2;",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ),
        (
            "var foo = 1,
                bar = 2;",
            "var foo = 1;
                var bar = 2;",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ),
        (
            "var foo = 1, // comment
                bar = 2;",
            "var foo = 1; // comment
                var bar = 2;",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ),
        (
            "var f, k /* test */, l;",
            "var f; var k /* test */; var l;",
            Some(serde_json::json!(["never"])),
        ),
        (
            "var f,          /* test */ l;",
            "var f;          /* test */ var l;",
            Some(serde_json::json!(["never"])),
        ),
        (
            "var f, k /* test 
             some more comment 
             even more */, l = 1, P;",
            "var f; var k /* test 
             some more comment 
             even more */; var l = 1; var P;",
            Some(serde_json::json!(["never"])),
        ),
        ("var a = 1, b = 2", "var a = 1; var b = 2", Some(serde_json::json!(["never"]))),
        (
            "const foo = require('foo'); const bar = require('bar');",
            "const foo = require('foo'),  bar = require('bar');",
            Some(serde_json::json!([{ "separateRequires": true, "const": "always" }])),
        ),
        (
            "const foo = obj.require('foo'); const bar = 1;",
            "const foo = obj.require('foo'),  bar = 1;",
            Some(serde_json::json!([{ "separateRequires": true, "const": "always" }])),
        ),
        ("var a = 1, b; var c;", "var a = 1, b,  c;", Some(serde_json::json!(["consecutive"]))),
        (
            "var a = 0, b = 1; var c = 2;",
            "var a = 0, b = 1,  c = 2;",
            Some(serde_json::json!(["consecutive"])),
        ),
        ("let a = 1, b; let c;", "let a = 1, b,  c;", Some(serde_json::json!(["consecutive"]))),
        (
            "let a = 0, b = 1; let c = 2;",
            "let a = 0, b = 1,  c = 2;",
            Some(serde_json::json!(["consecutive"])),
        ),
        (
            "const a = 0, b = 1; const c = 2;",
            "const a = 0, b = 1,  c = 2;",
            Some(serde_json::json!(["consecutive"])),
        ),
        (
            "const a = 0; var b = 1; var c = 2; const d = 3;",
            "const a = 0; var b = 1,  c = 2; const d = 3;",
            Some(serde_json::json!(["consecutive"])),
        ),
        (
            "var a = true; var b = false;",
            "var a = true,  b = false;",
            Some(serde_json::json!([{ "separateRequires": true, "var": "always" }])),
        ),
        (
            "const a = 0; let b = 1; let c = 2; const d = 3;",
            "const a = 0; let b = 1,  c = 2; const d = 3;",
            Some(serde_json::json!(["consecutive"])),
        ),
        (
            "let a = 0; const b = 1; const c = 1; var d = 2;",
            "let a = 0; const b = 1,  c = 1; var d = 2;",
            Some(serde_json::json!(["consecutive"])),
        ),
        (
            "var a = 0; var b; var c; var d = 1",
            "var a = 0; var b,  c; var d = 1",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ),
        (
            "var a = 0; var b = 1; var c; var d;",
            "var a = 0,  b = 1; var c,  d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ),
        (
            "let a = 0; let b; let c; let d = 1;",
            "let a = 0; let b,  c; let d = 1;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ),
        (
            "let a = 0; let b = 1; let c; let d;",
            "let a = 0,  b = 1; let c,  d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ),
        (
            "const a = 0; let b; let c; const d = 1;",
            "const a = 0; let b,  c; const d = 1;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ),
        (
            "const a = 0; const b = 1; let c; let d;",
            "const a = 0,  b = 1; let c,  d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "always" }])),
        ),
        (
            "var a = 0; var b = 1; var c, d;",
            "var a = 0,  b = 1; var c; var d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ),
        (
            "var a = 0; var b, c; var d = 1;",
            "var a = 0; var b; var c; var d = 1;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ),
        (
            "let a = 0; let b = 1; let c, d;",
            "let a = 0,  b = 1; let c; let d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ),
        (
            "let a = 0; let b, c; let d = 1;",
            "let a = 0; let b; let c; let d = 1;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ),
        (
            "const a = 0; const b = 1; let c, d;",
            "const a = 0,  b = 1; let c; let d;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ),
        (
            "const a = 0; let b, c; const d = 1;",
            "const a = 0; let b; let c; const d = 1;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ),
        (
            "var a; var b; var c = 0; var d = 1;",
            "var a,  b; var c = 0,  d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ),
        (
            "var a; var b = 0; var c = 1; var d;",
            "var a; var b = 0,  c = 1; var d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ),
        (
            "let a; let b; let c = 0; let d = 1;",
            "let a,  b; let c = 0,  d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ),
        (
            "let a; let b = 0; let c = 1; let d;",
            "let a; let b = 0,  c = 1; let d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ),
        (
            "let a; let b; const c = 0; const d = 1;",
            "let a,  b; const c = 0,  d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ),
        (
            "let a; const b = 0; const c = 1; let d;",
            "let a; const b = 0,  c = 1; let d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "always" }])),
        ),
        (
            "var a; var b; var c = 0, d = 1;",
            "var a,  b; var c = 0; var d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ),
        (
            "var a; var b = 0, c = 1; var d;",
            "var a; var b = 0; var c = 1; var d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ),
        (
            "let a; let b; let c = 0, d = 1;",
            "let a,  b; let c = 0; let d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ),
        (
            "let a; let b = 0, c = 1; let d;",
            "let a; let b = 0; let c = 1; let d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ),
        (
            "let a; let b; const c = 0, d = 1;",
            "let a,  b; const c = 0; const d = 1;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ),
        (
            "let a; const b = 0, c = 1; let d;",
            "let a; const b = 0; const c = 1; let d;",
            Some(serde_json::json!([{ "uninitialized": "consecutive", "initialized": "never" }])),
        ),
        (
            "var a = 0; var b = 1;",
            "var a = 0,  b = 1;",
            Some(serde_json::json!([{ "var": "consecutive" }])),
        ),
        (
            "let a = 0; let b = 1;",
            "let a = 0,  b = 1;",
            Some(serde_json::json!([{ "let": "consecutive" }])),
        ),
        (
            "const a = 0; const b = 1;",
            "const a = 0,  b = 1;",
            Some(serde_json::json!([{ "const": "consecutive" }])),
        ),
        (
            "let a; let b; const c = 0; const d = 1;",
            "let a,  b; const c = 0,  d = 1;",
            Some(serde_json::json!([{ "let": "consecutive", "const": "always" }])),
        ),
        (
            "let a; const b = 0; const c = 1; let d;",
            "let a; const b = 0,  c = 1; let d;",
            Some(serde_json::json!([{ "let": "consecutive", "const": "always" }])),
        ),
        (
            "let a; let b; const c = 0, d = 1;",
            "let a,  b; const c = 0; const d = 1;",
            Some(serde_json::json!([{ "let": "consecutive", "const": "never" }])),
        ),
        (
            "let a; const b = 0, c = 1; let d;",
            "let a; const b = 0; const c = 1; let d;",
            Some(serde_json::json!([{ "let": "consecutive", "const": "never" }])),
        ),
        (
            "const a = 0; const b = 1; let c; let d;",
            "const a = 0,  b = 1; let c,  d;",
            Some(serde_json::json!([{ "const": "consecutive", "let": "always" }])),
        ),
        (
            "const a = 0; let b; let c; const d = 1;",
            "const a = 0; let b,  c; const d = 1;",
            Some(serde_json::json!([{ "const": "consecutive", "let": "always" }])),
        ),
        (
            "const a = 0; const b = 1; let c, d;",
            "const a = 0,  b = 1; let c; let d;",
            Some(serde_json::json!([{ "const": "consecutive", "let": "never" }])),
        ),
        (
            "const a = 0; let b, c; const d = 1;",
            "const a = 0; let b; let c; const d = 1;",
            Some(serde_json::json!([{ "const": "consecutive", "let": "never" }])),
        ),
        ("var bar; var baz;", "var bar,  baz;", Some(serde_json::json!(["consecutive"]))),
        (
            "var bar = 1; var baz = 2; qux(); var qux = 3; var quux;",
            "var bar = 1,  baz = 2; qux(); var qux = 3,  quux;",
            Some(serde_json::json!(["consecutive"])),
        ),
        (
            "let a, b; let c; var d, e;",
            "let a, b,  c; var d; var e;",
            Some(
                serde_json::json!([ { "var": "never", "let": "consecutive", "const": "consecutive" }, ]),
            ),
        ),
        ("var a; var b;", "var a,  b;", Some(serde_json::json!([{ "var": "consecutive" }]))),
        (
            "var a = 1; var b = 2; var c, d; var e = 3; var f = 4;",
            "var a = 1,  b = 2; var c; var d; var e = 3,  f = 4;",
            Some(serde_json::json!([{ "initialized": "consecutive", "uninitialized": "never" }])),
        ),
        (
            "var a = 1; var b = 2; foo(); var c = 3; var d = 4;",
            "var a = 1,  b = 2; foo(); var c = 3,  d = 4;",
            Some(serde_json::json!([{ "initialized": "consecutive" }])),
        ),
        (
            "var a
            var b",
            "var a,
             b",
            Some(serde_json::json!(["always"])),
        ),
        (
            "export const foo=1, bar=2;",
            "export const foo=1; export const bar=2;",
            Some(serde_json::json!(["never"])),
        ),
        (
            "const foo=1,
             bar=2;",
            "const foo=1;
             const bar=2;",
            Some(serde_json::json!(["never"])),
        ),
        (
            "export const foo=1,
             bar=2;",
            "export const foo=1;
             export const bar=2;",
            Some(serde_json::json!(["never"])),
        ),
        (
            "export const foo=1
            , bar=2;",
            "export const foo=1
            ; export const bar=2;",
            Some(serde_json::json!(["never"])),
        ),
        (
            "export const foo= a, bar=2;",
            "export const foo= a; export const bar=2;",
            Some(serde_json::json!(["never"])),
        ),
        (
            "export const foo=() => a, bar=2;",
            "export const foo=() => a; export const bar=2;",
            Some(serde_json::json!(["never"])),
        ),
        (
            "export const foo= a, bar=2, bar2=2;",
            "export const foo= a; export const bar=2; export const bar2=2;",
            Some(serde_json::json!(["never"])),
        ),
        (
            "export const foo = 1,bar = 2;",
            "export const foo = 1; export const bar = 2;",
            Some(serde_json::json!(["never"])),
        ),
        (
            "class C { static { let x, y; } }",
            "class C { static { let x; let y; } }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "class C { static { var x, y; } }",
            "class C { static { var x; var y; } }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "class C { static { let x; let y; } }",
            "class C { static { let x,  y; } }",
            Some(serde_json::json!(["always"])),
        ),
        (
            "class C { static { var x; var y; } }",
            "class C { static { var x,  y; } }",
            Some(serde_json::json!(["always"])),
        ),
        (
            "class C { static { let x; let y; } }",
            "class C { static { let x,  y; } }",
            Some(serde_json::json!(["consecutive"])),
        ),
        (
            "class C { static { var x; var y; } }",
            "class C { static { var x,  y; } }",
            Some(serde_json::json!(["consecutive"])),
        ),
        (
            "class C { static { let a = 0; let b = 1; } }",
            "class C { static { let a = 0,  b = 1; } }",
            Some(serde_json::json!([{ "initialized": "consecutive" }])),
        ),
        (
            "class C { static { var a = 0; var b = 1; } }",
            "class C { static { var a = 0,  b = 1; } }",
            Some(serde_json::json!([{ "initialized": "consecutive" }])),
        ),
        ("using a = 0; using b = 1;", "using a = 0,  b = 1;", Some(serde_json::json!(["always"]))),
        (
            "await using a = 0; await using b = 1;",
            "await using a = 0,   b = 1;",
            Some(serde_json::json!(["always"])),
        ),
        ("using a = 0, b = 1;", "using a = 0; using b = 1;", Some(serde_json::json!(["never"]))),
        (
            "await using a = 0, b = 1;",
            "await using a = 0; await using b = 1;",
            Some(serde_json::json!(["never"])),
        ),
        (
            "using a = 0; using b = 1;",
            "using a = 0,  b = 1;",
            Some(serde_json::json!(["consecutive"])),
        ),
        (
            "await using a = 0; await using b = 1;",
            "await using a = 0,   b = 1;",
            Some(serde_json::json!(["consecutive"])),
        ),
        (
            "using a = 0, b = 1;",
            "using a = 0; using b = 1;",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ),
        (
            "await using a = 0, b = 1;",
            "await using a = 0; await using b = 1;",
            Some(serde_json::json!([{ "initialized": "never" }])),
        ),
    ];
    Tester::new(OneVar::NAME, OneVar::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
