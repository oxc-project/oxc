use oxc_ast::{
    AstKind,
    ast::{CallExpression, Statement},
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

const MAX_VALIDATION_LOOKAHEAD: usize = 3;

fn json_parse_validation_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Validate `JSON.parse()` results before using them.")
        .with_help(
            "Wrap `JSON.parse(...)` with `safeParse(...)`/`parse(...)`, or validate the assigned value within the next few statements.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct JsonParseValidation;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires runtime validation of `JSON.parse()` results.
    ///
    /// ### Why is this bad?
    ///
    /// Parsed JSON is untrusted input. Requiring an immediate schema validation
    /// step makes the data safe to use and prevents unchecked assumptions from
    /// spreading through the program.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const data = JSON.parse(text);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const result = v.safeParse(MySchema, JSON.parse(text));
    /// ```
    JsonParseValidation,
    oxc,
    correctness,
    none
);

impl Rule for JsonParseValidation {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !call_expr.callee.is_specific_member_access("JSON", "parse") {
            return;
        }

        if is_immediately_validated(node.id(), ctx) {
            return;
        }

        if get_assigned_identifier_name(node.id(), ctx)
            .is_some_and(|name| is_validated_soon(node.id(), name, ctx))
        {
            return;
        }

        ctx.diagnostic(json_parse_validation_diagnostic(call_expr.span));
    }
}

fn is_immediately_validated(node_id: oxc_syntax::node::NodeId, ctx: &LintContext<'_>) -> bool {
    match ctx.nodes().parent_kind(node_id) {
        AstKind::CallExpression(parent_call) => is_validation_call(parent_call),
        _ => false,
    }
}

fn get_assigned_identifier_name<'a>(
    node_id: oxc_syntax::node::NodeId,
    ctx: &LintContext<'a>,
) -> Option<&'a str> {
    match ctx.nodes().parent_kind(node_id) {
        AstKind::VariableDeclarator(declarator) => {
            declarator.id.get_identifier_name().map(|ident| ident.as_str())
        }
        AstKind::AssignmentExpression(assignment) => assignment.left.get_identifier_name(),
        _ => None,
    }
}

fn is_validated_soon(node_id: oxc_syntax::node::NodeId, name: &str, ctx: &LintContext<'_>) -> bool {
    let Some((statements, statement_index)) = find_enclosing_statement_body(node_id, ctx) else {
        return false;
    };

    statements
        .iter()
        .skip(statement_index + 1)
        .take(MAX_VALIDATION_LOOKAHEAD)
        .any(|statement| statement_contains_validation_of_identifier(statement, name))
}

fn find_enclosing_statement_body<'a>(
    node_id: oxc_syntax::node::NodeId,
    ctx: &LintContext<'a>,
) -> Option<(&'a [Statement<'a>], usize)> {
    let mut child = ctx.nodes().get_node(node_id);

    for ancestor in ctx.nodes().ancestors(node_id) {
        match ancestor.kind() {
            AstKind::BlockStatement(block) => {
                let body = block.body.as_slice();
                let statement_span = child.kind().span();
                let statement_index =
                    body.iter().position(|statement| statement.span() == statement_span)?;
                return Some((body, statement_index));
            }
            AstKind::Program(program) => {
                let body = program.body.as_slice();
                let statement_span = child.kind().span();
                let statement_index =
                    body.iter().position(|statement| statement.span() == statement_span)?;
                return Some((body, statement_index));
            }
            _ => {
                child = ancestor;
            }
        }
    }

    None
}

fn is_validation_call(call_expr: &CallExpression) -> bool {
    match &call_expr.callee {
        oxc_ast::ast::Expression::Identifier(ident) => {
            matches!(ident.name.as_str(), "safeParse" | "parse")
        }
        callee if callee.is_member_expression() => callee
            .to_member_expression()
            .static_property_name()
            .is_some_and(|name| matches!(name, "safeParse" | "parse")),
        oxc_ast::ast::Expression::ChainExpression(chain) => chain
            .expression
            .as_member_expression()
            .and_then(oxc_ast::ast::MemberExpression::static_property_name)
            .is_some_and(|name| matches!(name, "safeParse" | "parse")),
        _ => false,
    }
}

fn statement_contains_validation_of_identifier(statement: &Statement<'_>, name: &str) -> bool {
    let mut visitor = ValidationUsageVisitor { name, found: false };
    visitor.visit_statement(statement);
    visitor.found
}

struct ValidationUsageVisitor<'a> {
    name: &'a str,
    found: bool,
}

impl ValidationUsageVisitor<'_> {
    fn call_uses_target_identifier(&self, call_expr: &CallExpression<'_>) -> bool {
        call_expr.arguments.iter().filter_map(|argument| argument.as_expression()).any(
            |expression| match expression.get_inner_expression() {
                oxc_ast::ast::Expression::Identifier(ident) => ident.name == self.name,
                _ => false,
            },
        )
    }
}

impl<'a> Visit<'a> for ValidationUsageVisitor<'_> {
    fn visit_call_expression(&mut self, call_expr: &CallExpression<'a>) {
        if self.found {
            return;
        }

        if is_validation_call(call_expr) && self.call_uses_target_identifier(call_expr) {
            self.found = true;
            return;
        }

        walk::walk_call_expression(self, call_expr);
    }
}

#[test]
fn test() {
    use serde_json::Value;
    use serde_json::json;

    use crate::tester::Tester;

    let pass: Vec<(&str, Option<Value>)> = vec![
        ("const result = safeParse(MySchema, JSON.parse(text));", None),
        ("const result = parse(MySchema, JSON.parse(text), 'context label');", None),
        ("const result = v.safeParse(MySchema, JSON.parse(text));", None),
        ("const raw = JSON.parse(text); const result = v.safeParse(MySchema, raw);", None),
        ("let raw; raw = JSON.parse(text); const result = parse(MySchema, raw, 'ctx');", None),
        (
            "const raw = JSON.parse(text); foo(); bar(); v.safeParse(MySchema, raw);",
            Some(json!([])),
        ),
    ];

    let fail: Vec<(&str, Option<Value>)> = vec![
        ("const data = JSON.parse(text);", None),
        ("handle(JSON.parse(text));", None),
        ("const raw = JSON.parse(text); doSomething(raw);", None),
        ("let raw; raw = JSON.parse(text); doSomething(raw);", None),
        (
            "const raw = JSON.parse(text); foo(); bar(); baz(); qux(); v.safeParse(MySchema, raw);",
            None,
        ),
    ];

    Tester::new(JsonParseValidation::NAME, JsonParseValidation::PLUGIN, pass, fail)
        .change_rule_path_extension("ts")
        .test_and_snapshot();
}
