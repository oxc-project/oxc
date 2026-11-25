use std::fmt::Debug;

use oxc_ast::{
    AstKind,
    ast::{
        Expression, ForOfStatement, ForStatementLeft, IdentifierReference, IfStatement, JSXText,
        LogicalExpression, Statement, StringLiteral, TemplateElement, VariableDeclarationKind,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::BoundNames;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{
    identifier::is_white_space,
    line_terminator::is_line_terminator,
    operator::{BinaryOperator, LogicalOperator},
};

use crate::{
    AstNode,
    ast_util::{get_symbol_id_of_variable, variable_declaration_kind},
    context::LintContext,
    rule::Rule,
    utils::is_same_expression,
};

fn some(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Found a useless array length check")
        .with_help(
            "The non-empty check is useless as `Array#some()` returns `false` for an empty array.",
        )
        .with_label(span)
}

fn every(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Found a useless array length check")
        .with_help(
            "The empty check is useless as `Array#every()` returns `true` for an empty array.",
        )
        .with_label(span)
}

fn for_of(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Found a useless array length check")
        .with_help(
            "The non-empty check is useless as `for...of` does not iterate over an empty array.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessLengthCheck;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// It checks for an unnecessary array length check in a logical expression or before a `for...of` loop.
    ///
    /// The cases are:
    /// - `array.length === 0 || array.every(Boolean)` (`array.every` returns `true` if array is empty)
    /// - `array.length > 0 && array.some(Boolean)` (`array.some` returns `false` if array is empty)
    /// - `if (array.length > 0) { for (const x of array) {} }` (`for...of` skips empty arrays)
    ///
    /// ### Why is this bad?
    ///
    /// An extra unnecessary length check is done.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// if (array.length === 0 || array.every(Boolean)) {
    ///    // do something!
    /// }
    /// if (array.length > 0) { for (const x of array) { use(x); } }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// if (array.every(Boolean)) {
    ///    // do something!
    /// }
    /// for (const x of array) { use(x); }
    /// ```
    NoUselessLengthCheck,
    unicorn,
    correctness,
    recommended,
    conditional_fix,
    version = "0.0.19",
    short_description = "Disallow useless array length check.",
);

struct ConditionDTO {
    property_name: &'static str,
    binary_operators: &'static [BinaryOperator],
}

fn is_useless_check<'a>(
    left: &'a Expression<'a>,
    right: &'a Expression<'a>,
    operator: LogicalOperator,
) -> Option<OxcDiagnostic> {
    let every_condition = ConditionDTO {
        property_name: "every",
        binary_operators: &[BinaryOperator::StrictEquality],
    };
    let some_condition = ConditionDTO {
        property_name: "some",
        binary_operators: &[BinaryOperator::StrictInequality, BinaryOperator::GreaterThan],
    };

    let mut array_name: &str = "";
    let active_condition =
        { if operator == LogicalOperator::Or { every_condition } else { some_condition } };
    let mut binary_expression_span: Option<Span> = None;
    let mut call_expression_span: Option<Span> = None;

    let l = match left.without_parentheses() {
        Expression::BinaryExpression(expr) => {
            let left_expr = expr.left.get_inner_expression().as_member_expression()?;
            array_name = left_expr.object().get_identifier_reference()?.name.as_str();
            binary_expression_span = Some(expr.span);

            active_condition.binary_operators.contains(&expr.operator)
                && left_expr.is_specific_member_access(array_name, "length")
                && expr.right.is_specific_raw_number_literal("0")
        }
        Expression::CallExpression(expr) => {
            array_name =
                expr.callee.get_member_expr()?.object().get_identifier_reference()?.name.as_str();
            let property_name = expr.callee.get_member_expr()?.static_property_name()?;
            call_expression_span = Some(expr.span);

            let is_same_method = property_name == active_condition.property_name;
            let is_optional = expr.optional;

            is_same_method && !is_optional
        }
        _ => false,
    };

    let r = match right.without_parentheses() {
        Expression::BinaryExpression(expr) => {
            let left_expr = expr.left.get_inner_expression().as_member_expression()?;
            let ident_name = left_expr.object().get_identifier_reference()?.name.as_str();
            if binary_expression_span.is_some() {
                return None;
            }
            binary_expression_span = Some(expr.span);

            active_condition.binary_operators.contains(&expr.operator)
                && left_expr.is_specific_member_access(array_name, "length")
                && expr.right.is_specific_raw_number_literal("0")
                && ident_name == array_name
        }
        Expression::CallExpression(expr) => {
            let is_same_name =
                expr.callee.get_member_expr()?.object().get_identifier_reference()?.name.as_str()
                    == array_name;

            if call_expression_span.is_some() {
                return None;
            }
            let property_name = expr.callee.get_member_expr()?.static_property_name()?;
            let is_same_method = property_name == active_condition.property_name;
            let is_optional = expr.optional;

            is_same_method && !is_optional && is_same_name
        }
        _ => false,
    };

    if l && r {
        Some(if active_condition.property_name == "every" {
            every(binary_expression_span?)
        } else {
            some(binary_expression_span?)
        })
    } else {
        None
    }
}

impl Rule for NoUselessLengthCheck {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::LogicalExpression(log_expr) => {
                if ![LogicalOperator::And, LogicalOperator::Or].contains(&log_expr.operator) {
                    return;
                }
                let mut flat_exprs = Vec::new();
                make_flat_logical_expression(log_expr, &mut flat_exprs);
                for window in flat_exprs.windows(2) {
                    if let Some(diag) = is_useless_check(window[0], window[1], log_expr.operator) {
                        ctx.diagnostic(diag);
                    }
                }
            }
            AstKind::ForOfStatement(for_of_stmt) => {
                check_for_of_statement(node, for_of_stmt, ctx);
            }
            _ => {}
        }
    }
}

const TYPED_ARRAY_NAMES: &[&str] = &[
    "Int8Array",
    "Uint8Array",
    "Uint8ClampedArray",
    "Int16Array",
    "Uint16Array",
    "Int32Array",
    "Uint32Array",
    "Float16Array",
    "Float32Array",
    "Float64Array",
    "BigInt64Array",
    "BigUint64Array",
];

fn check_for_of_statement<'a>(
    node: &AstNode<'a>,
    for_of_stmt: &ForOfStatement<'a>,
    ctx: &LintContext<'a>,
) {
    if for_of_stmt.r#await {
        return;
    }

    let Some(if_stmt) = get_enclosing_guard_if_statement(node, for_of_stmt, ctx) else {
        return;
    };

    let Expression::BinaryExpression(test_bin) = if_stmt.test.get_inner_expression() else {
        return;
    };
    if !matches!(test_bin.operator, BinaryOperator::GreaterThan | BinaryOperator::StrictInequality)
        || !test_bin.right.get_inner_expression().is_specific_raw_number_literal("0")
    {
        return;
    }

    let Expression::StaticMemberExpression(length_member) = test_bin.left.get_inner_expression()
    else {
        return;
    };
    if length_member.optional || length_member.property.name != "length" {
        return;
    }

    if !is_same_expression(
        length_member.object.get_inner_expression(),
        for_of_stmt.right.get_inner_expression(),
        ctx,
    ) || has_loop_binding_reference_in_right(for_of_stmt, ctx)
        || is_known_non_indexed_collection(&for_of_stmt.right, ctx)
    {
        return;
    }

    let diag = for_of(test_bin.span);
    let for_of_text = ctx.source_range(for_of_stmt.span);
    let is_single_line = !for_of_text.contains(is_line_terminator);
    let source = ctx.source_text();
    let indent_pair = if is_single_line {
        Some(("", ""))
    } else {
        indent_at_line_start(source, for_of_stmt.span.start)
            .zip(indent_at_line_start(source, if_stmt.span.start))
    };

    let Some((source_indent, target_indent)) = indent_pair else {
        ctx.diagnostic(diag);
        return;
    };

    let has_outer_comments = ctx
        .has_comments_between(Span::new(if_stmt.span.start, for_of_stmt.span.start))
        || ctx.has_comments_between(Span::new(for_of_stmt.span.end, if_stmt.span.end));

    if has_outer_comments || (!is_single_line && has_multiline_token(for_of_stmt, ctx)) {
        ctx.diagnostic(diag);
        return;
    }

    ctx.diagnostic_with_fix(diag, |fixer| {
        let mut replacement = if is_single_line {
            for_of_text.to_owned()
        } else {
            reindent_text(for_of_text, source_indent, target_indent)
        };
        if matches!(if_stmt.consequent, Statement::BlockStatement(_))
            && statement_ends_without_semicolon(&for_of_stmt.body, ctx)
        {
            replacement.push(';');
        }
        fixer.replace(if_stmt.span, replacement)
    });
}

fn get_enclosing_guard_if_statement<'a, 'b>(
    for_of_node: &AstNode<'a>,
    for_of: &ForOfStatement<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b IfStatement<'a>> {
    let parent = ctx.nodes().parent_node(for_of_node.id());
    match parent.kind() {
        AstKind::BlockStatement(block) => {
            if block.body.len() != 1 || block.body[0].span() != for_of.span {
                return None;
            }
            let grandparent = ctx.nodes().parent_node(parent.id());
            let AstKind::IfStatement(if_stmt) = grandparent.kind() else {
                return None;
            };
            (if_stmt.alternate.is_none()
                && matches!(&if_stmt.consequent, Statement::BlockStatement(b) if b.span == block.span))
            .then_some(if_stmt)
        }
        AstKind::IfStatement(if_stmt) => (if_stmt.alternate.is_none()
            && matches!(&if_stmt.consequent, Statement::ForOfStatement(f) if f.span == for_of.span))
        .then_some(if_stmt),
        _ => None,
    }
}

fn statement_ends_without_semicolon(stmt: &Statement, ctx: &LintContext) -> bool {
    if ctx.source_range(stmt.span()).trim_end().ends_with(';') {
        return false;
    }
    match stmt {
        Statement::ExpressionStatement(_)
        | Statement::VariableDeclaration(_)
        | Statement::ReturnStatement(_)
        | Statement::ThrowStatement(_)
        | Statement::BreakStatement(_)
        | Statement::ContinueStatement(_)
        | Statement::DebuggerStatement(_) => true,
        Statement::IfStatement(if_stmt) => statement_ends_without_semicolon(
            if_stmt.alternate.as_ref().unwrap_or(&if_stmt.consequent),
            ctx,
        ),
        Statement::LabeledStatement(s) => statement_ends_without_semicolon(&s.body, ctx),
        Statement::WhileStatement(s) => statement_ends_without_semicolon(&s.body, ctx),
        Statement::ForStatement(s) => statement_ends_without_semicolon(&s.body, ctx),
        Statement::ForInStatement(s) => statement_ends_without_semicolon(&s.body, ctx),
        Statement::ForOfStatement(s) => statement_ends_without_semicolon(&s.body, ctx),
        Statement::WithStatement(s) => statement_ends_without_semicolon(&s.body, ctx),
        _ => false,
    }
}

fn has_loop_binding_reference_in_right<'a>(
    for_of: &ForOfStatement<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    let ForStatementLeft::VariableDeclaration(decl) = &for_of.left else {
        return false;
    };
    if decl.kind == VariableDeclarationKind::Var {
        return false;
    }

    let right_span = for_of.right.span();
    let mut found = false;
    decl.bound_names(&mut |ident| {
        if found {
            return;
        }
        for reference in ctx.semantic().symbol_references(ident.symbol_id()) {
            let ref_span = ctx.nodes().get_node(reference.node_id()).span();
            if right_span.contains_inclusive(ref_span) {
                found = true;
                break;
            }
        }
    });
    found
}

fn is_known_non_indexed_collection<'a>(expr: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    let expr = match expr.get_inner_expression() {
        Expression::Identifier(ident) => {
            let Some(init) = const_initializer(ident, ctx) else {
                return false;
            };
            init.get_inner_expression()
        }
        expr => expr,
    };
    match expr {
        Expression::ObjectExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::ClassExpression(_)
        | Expression::TemplateLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_) => true,
        Expression::NewExpression(new_expr) => !matches!(
            new_expr.callee.get_inner_expression(),
            Expression::Identifier(ident)
                if (ident.name == "Array" || TYPED_ARRAY_NAMES.contains(&ident.name.as_str()))
                    && ident.is_global_reference(ctx.scoping())
        ),
        _ => false,
    }
}

fn const_initializer<'a>(
    ident: &IdentifierReference<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'a Expression<'a>> {
    let symbol_id = get_symbol_id_of_variable(ident, ctx)?;
    let AstKind::VariableDeclarator(decl) =
        ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id)).kind()
    else {
        return None;
    };
    if !decl.id.is_binding_identifier()
        || variable_declaration_kind(decl, ctx.nodes()) != Some(VariableDeclarationKind::Const)
    {
        return None;
    }
    decl.init.as_ref()
}

fn indent_at_line_start(source: &str, offset: u32) -> Option<&str> {
    let before = &source[..offset as usize];
    let prefix = match before.rfind(is_line_terminator) {
        Some(pos) => {
            let ch_len = before[pos..].chars().next().map_or(0, char::len_utf8);
            &before[pos + ch_len..]
        }
        None => before.strip_prefix('\u{FEFF}').unwrap_or(before),
    };
    prefix.chars().all(is_white_space).then_some(prefix)
}

fn reindent_text(text: &str, source_indent: &str, target_indent: &str) -> String {
    let Some(first_lb) = text.find(is_line_terminator) else {
        return text.to_owned();
    };
    let mut out = String::with_capacity(text.len());
    out.push_str(&text[..first_lb]);
    let mut rest = &text[first_lb..];
    while !rest.is_empty() {
        let lb_len = if rest.starts_with("\r\n") {
            2
        } else {
            rest.chars().next().map_or(0, char::len_utf8)
        };
        out.push_str(&rest[..lb_len]);
        rest = &rest[lb_len..];
        let next_lb = rest.find(is_line_terminator).unwrap_or(rest.len());
        let line = &rest[..next_lb];
        rest = &rest[next_lb..];
        if !line.chars().all(is_white_space) {
            if let Some(stripped) = line.strip_prefix(source_indent) {
                out.push_str(target_indent);
                out.push_str(stripped);
            } else {
                out.push_str(line);
            }
        }
    }
    out
}

struct MultilineTokenVisitor<'a, 'b> {
    ctx: &'b LintContext<'a>,
    found: bool,
}

impl MultilineTokenVisitor<'_, '_> {
    fn check(&mut self, span: Span) {
        self.found |= self.ctx.source_range(span).contains(is_line_terminator);
    }
}

impl<'a> Visit<'a> for MultilineTokenVisitor<'a, '_> {
    fn visit_statement(&mut self, it: &Statement<'a>) {
        if self.found {
            return;
        }
        walk::walk_statement(self, it);
    }

    fn visit_expression(&mut self, it: &Expression<'a>) {
        if self.found {
            return;
        }
        walk::walk_expression(self, it);
    }

    fn visit_string_literal(&mut self, it: &StringLiteral<'a>) {
        self.check(it.span);
    }

    fn visit_template_element(&mut self, it: &TemplateElement<'a>) {
        self.check(it.span);
    }

    fn visit_jsx_text(&mut self, it: &JSXText<'a>) {
        self.check(it.span);
    }
}

fn has_multiline_token<'a>(for_of: &ForOfStatement<'a>, ctx: &LintContext<'a>) -> bool {
    let mut visitor = MultilineTokenVisitor { ctx, found: false };
    visitor.visit_for_of_statement(for_of);
    visitor.found
}

fn make_flat_logical_expression<'a>(
    node: &'a LogicalExpression<'a>,
    result: &mut Vec<&'a Expression<'a>>,
) {
    match &node.left.without_parentheses() {
        Expression::LogicalExpression(le) => {
            if le.operator == node.operator {
                make_flat_logical_expression(le, result);
            } else {
                result.push(&node.left);
            }
        }
        _ => result.push(&node.left),
    }

    match &node.right.without_parentheses() {
        Expression::LogicalExpression(le) => {
            if le.operator == node.operator {
                make_flat_logical_expression(le, result);
            } else {
                result.push(&node.right);
            }
        }
        _ => result.push(&node.right),
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "array.length === 0 ?? array.every(Boolean)",
        "array.length === 0 && array.every(Boolean)",
        "(array.length === 0) + (array.every(Boolean))",
        "array.length === 1 || array.every(Boolean)",
        r#"array.length === "0" || array.every(Boolean)"#,
        "array.length === 0. || array.every(Boolean)",
        "array.length === 0x0 || array.every(Boolean)",
        "array.length !== 0 || array.every(Boolean)",
        "array.length == 0 || array.every(Boolean)",
        "0 === array.length || array.every(Boolean)",
        "array?.length === 0 || array.every(Boolean)",
        "array.notLength === 0 || array.every(Boolean)",
        "array[length] === 0 || array.every(Boolean)",
        "array.length === 0 || array.every?.(Boolean)",
        "array.length === 0 || array?.every(Boolean)",
        "array.length === 0 || array.every",
        "array.length === 0 || array[every](Boolean)",
        "array1.length === 0 || array2.every(Boolean)",
        "array.length !== 0 ?? array.some(Boolean)",
        "array.length !== 0 || array.some(Boolean)",
        "(array.length !== 0) - (array.some(Boolean))",
        "array.length !== 1 && array.some(Boolean)",
        r#"array.length !== "0" && array.some(Boolean)"#,
        "array.length !== 0. && array.some(Boolean)",
        "array.length !== 0x0 && array.some(Boolean)",
        "array.length === 0 && array.some(Boolean)",
        "array.length <= 0 && array.some(Boolean)",
        "array.length != 0 && array.some(Boolean)",
        "0 !== array.length && array.some(Boolean)",
        "array?.length !== 0 && array.some(Boolean)",
        "array.notLength !== 0 && array.some(Boolean)",
        "array[length] !== 0 && array.some(Boolean)",
        "array.length !== 0 && array.some?.(Boolean)",
        "array.length !== 0 && array?.some(Boolean)",
        "array.length !== 0 && array.some",
        "array.length !== 0 && array.notSome(Boolean)",
        "array.length !== 0 && array[some](Boolean)",
        "array1.length !== 0 && array2.some(Boolean)",
        "array.length > 0 ?? array.some(Boolean)",
        "array.length > 0 || array.some(Boolean)",
        "(array.length > 0) - (array.some(Boolean))",
        "array.length > 1 && array.some(Boolean)",
        r#"array.length > "0" && array.some(Boolean)"#,
        "array.length > 0. && array.some(Boolean)",
        "array.length > 0x0 && array.some(Boolean)",
        "array.length >= 0 && array.some(Boolean)",
        "0 > array.length && array.some(Boolean)",
        "0 < array.length && array.some(Boolean)",
        "array?.length > 0 && array.some(Boolean)",
        "array.notLength > 0 && array.some(Boolean)",
        "array.length > 0 && array.some?.(Boolean)",
        "array.length > 0 && array?.some(Boolean)",
        "array.length > 0 && array.some",
        "array.length > 0 && array.notSome(Boolean)",
        "array.length > 0 && array[some](Boolean)",
        "array1.length > 0 && array2.some(Boolean)",
        "if (
                foo &&
                array.length !== 0 &&
                bar &&
                array.some(Boolean)
            ) {
                // ...
            }",
        "(foo && array.length === 0) || array.every(Boolean) && foo",
        "array.length === 0 || (array.every(Boolean) && foo)",
        "(foo || array.length > 0) && array.some(Boolean)",
        "array.length > 0 && (array.some(Boolean) || foo)",
        "if (array.length === 0) { for (const element of array) {} }",
        "if (array.length >= 0) { for (const element of array) {} }",
        "if (array.length > 1) { for (const element of array) {} }",
        "if (array.length != 0) { for (const element of array) {} }",
        "if (array.length > 0 && condition) { for (const element of array) {} }",
        "if (array.length > 0) { for (const element of otherArray) {} }",
        "if (array.length > 0) { before(); for (const element of array) {} }",
        "if (array.length > 0) { for (const element of array) {} after(); }",
        "if (array.length > 0) { for (const element of array) {} } else { fallback(); }",
        "if (array.length > 0) { for (const index in array) {} }",
        "if (array.length > 0) { for (let index = 0; index < array.length; index++) {} }",
        "if (array.length > 0) { while (condition) {} }",
        "async function iterate() { if (array.length > 0) { for await (const element of array) {} } }",
        "if (array?.length > 0) { for (const element of array) {} }",
        "if (array[length] > 0) { for (const element of array) {} }",
        r#"if (array["length"] > 0) { for (const element of array) {} }"#,
        "if (array.length > 0.) { for (const element of array) {} }",
        "if (array.length > 0x0) { for (const element of array) {} }",
        "if (getArray().length > 0) { for (const element of getArray()) {} }",
        "const array = []; if (array.length > 0) { for (const array of array) {} }",
        "const array = []; if (array.length > 0) { for (const {value: array} of array) {} }",
        "const object = {array: []}; if (object.array.length > 0) { for (const object of object.array) {} }",
        "const items = new Set(); if (items.length > 0) { for (const item of items) {} }",
        "const items = new Set(); if ((items as any).length > 0) { for (const item of (items as any)) {} }",
        "if (array?.length! > 0) { for (const element of array) {} }",
        "if (array.length > 0) { 'use strict'; for (const element of array) {} }",
    ];

    let fail = vec![
        "array.length === 0 || array.every(Boolean)",
        "array.length > 0 && array.some(Boolean)",
        "array.length !== 0 && array.some(Boolean)",
        // "((
        //         ((
        //             (( array )).length
        //         )) === (( 0 ))
        //         ||
        //         ((
        //             (( array )).every(Boolean)
        //         ))
        //     ))",
        // "((
        //         ((
        //             (( array )).every(Boolean)
        //         ))
        //         ||
        //         ((
        //             (( array )).length
        //         )) === (( 0 ))
        //     ))",
        "if ((( array.length > 0 )) && array.some(Boolean));",
        "if (
                array.length !== 0 &&
                array.some(Boolean) &&
                foo
            ) {
                // ...
            }",
        "(array.length === 0 || array.every(Boolean)) || foo",
        "foo || (array.length === 0 || array.every(Boolean))",
        "(array.length > 0 && array.some(Boolean)) && foo",
        "foo && (array.length > 0 && array.some(Boolean))",
        "array.every(Boolean) || array.length === 0",
        "array.some(Boolean) && array.length !== 0",
        "array.some(Boolean) && array.length > 0",
        "foo && array.length > 0 && array.some(Boolean)",
        "foo || array.length === 0 || array.every(Boolean)",
        "(foo || array.length === 0) || array.every(Boolean)",
        "array.length === 0 || (array.every(Boolean) || foo)",
        "(foo && array.length > 0) && array.some(Boolean)",
        "array.length > 0 && (array.some(Boolean) && foo)",
        "array.every(Boolean) || array.length === 0 || array.every(Boolean)",
        "array.length === 0 || array.every(Boolean) || array.length === 0",
        "array1.every(Boolean)
            || (( array1.length === 0 || array2.length === 0 )) // Both useless
            || array2.every(Boolean)",
        "function isUselessLengthCheckNode({node, operator, siblings}) {
                return (
                    (
                        operator === '||' &&
                        zeroLengthChecks.has(node) &&
                        siblings.length > 0 &&
                        siblings.some(condition =>
                            arrayEveryCalls.has(condition) &&
                            isSameReference(node.left.object, condition.callee.object)
                        )
                    ) ||
                    (
                        operator === '&&' &&
                        nonZeroLengthChecks.has(node) &&
                        siblings.length > 0 &&
                        siblings.some(condition =>
                            arraySomeCalls.has(condition) &&
                            isSameReference(node.left.object, condition.callee.object)
                        )
                    )
                );
            }",
        "if (((array.length > 0))) { for (const element of array) { use(element); } }",
        "if (object.array.length !== 0) { for (const element of object.array) { use(element); } }",
        "if (arrays.length > 0) {\n\tfor (const array of arrays) {\n\t\tif (array.length > 0) {\n\t\t\tfor (const element of array) {\n\t\t\t\tuse(element);\n\t\t\t}\n\t\t}\n\t}\n}",
    ];

    let fix = vec![
        (
            "if (array.length > 0) {\n\tfor (const element of array) {\n\t\tuse(element);\n\t}\n}",
            "for (const element of array) {\n\tuse(element);\n}",
        ),
        (
            "if (array.length !== 0) {\n\tfor (const element of array) {\n\t\t// Do work.\n\t}\n}",
            "for (const element of array) {\n\t// Do work.\n}",
        ),
        (
            "function iterate(array) {\n\tif (array.length > 0) {\n\t\tfor (const element of array) {\n\t\t\tuse(element);\n\t\t}\n\t}\n}",
            "function iterate(array) {\n\tfor (const element of array) {\n\t\tuse(element);\n\t}\n}",
        ),
        (
            "if (array.length > 0) for (const element of array) { use(element); }",
            "for (const element of array) { use(element); }",
        ),
        (
            "if (array.length > 0) { for (const element of array) use(element); }",
            "for (const element of array) use(element);",
        ),
        (
            "if (array.length > 0) { for (const element of array) use(element) } [0].map(f);",
            "for (const element of array) use(element); [0].map(f);",
        ),
        (
            "if (array.length > 0) { for (const element of array) result = () => {} }\n(1);",
            "for (const element of array) result = () => {};\n(1);",
        ),
        (
            "outer: for (const y of b) { if (a.length > 0) { for (const x of a) break } outer; }",
            "outer: for (const y of b) { for (const x of a) break; outer; }",
        ),
        (
            r#"if (a["c"].length > 0) { for (const x of a.c) { use(x); } }"#,
            "for (const x of a.c) { use(x); }",
        ),
        (
            "if (array.length > 0) { for (const [key, value] of array) { use(key, value); } }",
            "for (const [key, value] of array) { use(key, value); }",
        ),
        (
            "if (array.length > 0) { for (const element of array) { /* Keep this comment. */ use(element); } }",
            "for (const element of array) { /* Keep this comment. */ use(element); }",
        ),
        (
            "if (array.length > 0) {\n\tfor (const element of array) {\n\t\t/*\n\t\tKeep this indentation.\n\t\t*/\n\t\tuse(element);\n\t}\n}",
            "for (const element of array) {\n\t/*\n\tKeep this indentation.\n\t*/\n\tuse(element);\n}",
        ),
        (
            "if (array.length > 0)\n\tfor (const element of array) {\n\t\tuse(element);\n\t}",
            "for (const element of array) {\n\tuse(element);\n}",
        ),
        (
            "function iterate(array) { if (array.length > 0) { for (var array of array) {} } }",
            "function iterate(array) { for (var array of array) {} }",
        ),
        (
            "if ((a?.b).length > 0) { for (const element of a?.b) { use(element); } }",
            "for (const element of a?.b) { use(element); }",
        ),
        (
            "if ((array as any[]).length > 0) { for (const element of (array as any[])) { use(element); } }",
            "for (const element of (array as any[])) { use(element); }",
        ),
        (
            "if (array!.length !== 0) { for (const element of array!) { use(element); } }",
            "for (const element of array!) { use(element); }",
        ),
        (
            "if ((array satisfies any[]).length > 0) { for (const element of (array satisfies any[])) { use(element); } }",
            "for (const element of (array satisfies any[])) { use(element); }",
        ),
        (
            "if ((array.length as number) > 0) { for (const element of array) { use(element); } }",
            "for (const element of array) { use(element); }",
        ),
        (
            "if (array.length! > 0) { for (const element of array) { use(element); } }",
            "for (const element of array) { use(element); }",
        ),
        (
            "if ((array.length satisfies number) > 0) { for (const element of array) { use(element); } }",
            "for (const element of array) { use(element); }",
        ),
        (
            "function iterate(array: Int8Array) { if (array.length > 0) { for (const element of array) { use(element); } } }",
            "function iterate(array: Int8Array) { for (const element of array) { use(element); } }",
        ),
        (
            "if (array.length > 0) {\n\tfor (const element of array) {\n\t\t/*\nKeep this column.\n\t\t*/\n\t}\n}",
            "for (const element of array) {\n\t/*\nKeep this column.\n\t*/\n}",
        ),
        (
            "if (array.length > 0) {\n\tfor (const element of array) {\nuse(element);\n\t}\n}",
            "for (const element of array) {\nuse(element);\n}",
        ),
        (
            "if (array.length > 0) {\r\n\tfor (const element of array) {\r\n\r\n\t\tuse(element);\r\n\t}\r\n}",
            "for (const element of array) {\r\n\r\n\tuse(element);\r\n}",
        ),
        (
            "if (array.length > 0) {\r\n\tfor (const element of array) {\r\n\r\n\t\tuse(element);\n\t\tuseAgain(element);\r\n\t}\r\n}",
            "for (const element of array) {\r\n\r\n\tuse(element);\n\tuseAgain(element);\r\n}",
        ),
        (
            "function run() {\n  if (array.length > 0) {\n    for (const element of array) {\n      use({\n        element,\n      });\n    }\n  }\n}",
            "function run() {\n  for (const element of array) {\n    use({\n      element,\n    });\n  }\n}",
        ),
        (
            "\u{FEFF}if (array.length > 0) {\n\tfor (const element of array) {\n\t\tuse(element);\n\t}\n}",
            "\u{FEFF}for (const element of array) {\n\tuse(element);\n}",
        ),
        (
            "if (array.length > 0) { /* Keep this comment. */ for (const element of array) { use(element); } }",
            "if (array.length > 0) { /* Keep this comment. */ for (const element of array) { use(element); } }",
        ),
        (
            "if (array.length /* Keep this comment. */ > 0) { for (const element of array) { use(element); } }",
            "if (array.length /* Keep this comment. */ > 0) { for (const element of array) { use(element); } }",
        ),
        (
            "if (array.length > 0) {\n\tfor (const element of array) {\n\t\tconst value = `\n\t\t\tKeep this indentation.\n\t\t`;\n\t}\n}",
            "if (array.length > 0) {\n\tfor (const element of array) {\n\t\tconst value = `\n\t\t\tKeep this indentation.\n\t\t`;\n\t}\n}",
        ),
        (
            "if (array.length > 0) { for (const element of array) {\n\tuse(element);\n} }",
            "if (array.length > 0) { for (const element of array) {\n\tuse(element);\n} }",
        ),
        (
            "before(); if (array.length > 0) {\n\tfor (const element of array) {\n\t\tuse(element);\n\t}\n}",
            "before(); if (array.length > 0) {\n\tfor (const element of array) {\n\t\tuse(element);\n\t}\n}",
        ),
        (
            "if ((a?.b).length > 0) { for (const element of a.b) {} }",
            "for (const element of a.b) {}",
        ),
        (
            "if (a.b.length > 0) { for (const element of a?.b) {} }",
            "for (const element of a?.b) {}",
        ),
        (
            "if ((a?.b.c).length > 0) { for (const element of a.b?.c) {} }",
            "for (const element of a.b?.c) {}",
        ),
    ];

    Tester::new(NoUselessLengthCheck::NAME, NoUselessLengthCheck::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();

    Tester::new(
        NoUselessLengthCheck::NAME,
        NoUselessLengthCheck::PLUGIN,
        Vec::<&str>::new(),
        Vec::<&str>::new(),
    )
    .change_rule_path_extension("ts")
    .expect_fix(vec![(
        "if ((<any[]>array).length > 0) { for (const element of (<any[]>array)) { use(element); } }",
        "for (const element of (<any[]>array)) { use(element); }",
    )])
    .test();
}
