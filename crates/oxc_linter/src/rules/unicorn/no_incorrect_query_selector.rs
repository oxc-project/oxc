use oxc_ast::{
    AstKind,
    ast::{
        Argument, BindingPattern, CallExpression, Expression, IdentifierReference,
        StaticMemberExpression, VariableDeclarationKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::JSStr;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{
    AstNode,
    ast_util::{outermost_paren_parent, variable_declaration_kind},
    context::LintContext,
    rule::Rule,
    utils::{is_logical_expression, is_node_value_not_dom_node},
};

fn first_match_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `.querySelector()` when only the first match is used.")
        .with_help(
            "Use `.querySelector()` instead of taking the first item of `.querySelectorAll()`.",
        )
        .with_label(span)
}

fn id_selector_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `.querySelector()` for a simple ID selector.")
        .with_help("An ID selector matches a single element, so use `.querySelector()`.")
        .with_label(span)
}

fn length_check_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Check `.length` instead of the `NodeList` object itself.")
        .with_help("A `NodeList` is always truthy, even when it is empty.")
        .with_label(span)
}

fn query_selector_all_nullish_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Check `.length` instead of comparing the `NodeList` object with `null` or `undefined`.",
    )
    .with_help("`.querySelectorAll()` always returns a `NodeList`, never `null` or `undefined`.")
    .with_label(span)
}

fn query_selector_undefined_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Compare the result of `.querySelector()` with `null` instead of `undefined`.",
    )
    .with_help("`.querySelector()` returns `null` when nothing matches.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoIncorrectQuerySelector;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows incorrect or inefficient `querySelector()` and `querySelectorAll()` usage.
    ///
    /// ### Why is this bad?
    ///
    /// `querySelectorAll()` collects every match, which is wasteful when only the first
    /// one is used. It also always returns a `NodeList`, so checking the result for
    /// truthiness or comparing it with `null` never detects an empty result.
    /// `querySelector()` returns `null`, not `undefined`, when nothing matches.
    ///
    /// This rule only checks simple, common cases. It does not validate or simplify selectors.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// document.querySelectorAll('form')[0];
    /// document.querySelectorAll('form').at(0);
    /// document.querySelectorAll('#foo');
    ///
    /// if (document.querySelectorAll('.item')) {}
    ///
    /// const elements = document.querySelectorAll('.item');
    /// if (elements) {}
    ///
    /// document.querySelectorAll('.item') === null;
    /// document.querySelector('.item') === undefined;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// document.querySelector('form');
    /// document.querySelector('#foo');
    ///
    /// if (document.querySelectorAll('.item').length > 0) {}
    ///
    /// const elements = document.querySelectorAll('.item');
    /// if (elements.length > 0) {}
    ///
    /// document.querySelectorAll('.item').length === 0;
    /// document.querySelector('.item') === null;
    /// ```
    NoIncorrectQuerySelector,
    unicorn,
    suspicious,
    conditional_fix,
    version = "next",
    short_description = "Disallow incorrect `querySelector()` and `querySelectorAll()` usage.",
);

impl Rule for NoIncorrectQuerySelector {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ComputedMemberExpression(_) => {
                check_first_element_access(node, ctx);
            }
            AstKind::CallExpression(call_expr) => {
                if check_first_element_access(node, ctx) || check_length(node, ctx) {
                    return;
                }
                if let Some(member_expr) = as_dom_query_member(call_expr, "querySelectorAll")
                    && !is_part_of_first_element_access(node, ctx)
                    && get_static_string_argument(call_expr).is_some_and(is_simple_id_selector)
                {
                    ctx.diagnostic(id_selector_diagnostic(member_expr.property.span));
                }
            }
            AstKind::IdentifierReference(_) => {
                check_length(node, ctx);
            }
            AstKind::BinaryExpression(binary_expr) => {
                if !matches!(
                    binary_expr.operator,
                    BinaryOperator::Equality
                        | BinaryOperator::Inequality
                        | BinaryOperator::StrictEquality
                        | BinaryOperator::StrictInequality
                ) {
                    return;
                }
                let left = binary_expr.left.without_parentheses();
                let right = binary_expr.right.without_parentheses();

                if get_compared_nullish_value(left, right, "querySelectorAll", ctx).is_some() {
                    ctx.diagnostic(query_selector_all_nullish_diagnostic(binary_expr.span));
                } else if matches!(
                    binary_expr.operator,
                    BinaryOperator::StrictEquality | BinaryOperator::StrictInequality
                ) && get_compared_nullish_value(left, right, "querySelector", ctx)
                    .is_some_and(Expression::is_undefined)
                {
                    ctx.diagnostic(query_selector_undefined_diagnostic(binary_expr.span));
                }
            }
            _ => {}
        }
    }
}

fn as_dom_query_member<'a, 'b>(
    call_expr: &'b CallExpression<'a>,
    method: &str,
) -> Option<&'b StaticMemberExpression<'a>> {
    if call_expr.optional
        || call_expr.arguments.len() != 1
        || matches!(call_expr.arguments[0], Argument::SpreadElement(_))
    {
        return None;
    }
    let Expression::StaticMemberExpression(member_expr) = call_expr.callee.without_parentheses()
    else {
        return None;
    };
    (!member_expr.optional
        && member_expr.property.name == method
        && !is_node_value_not_dom_node(member_expr.object.without_parentheses()))
    .then_some(member_expr)
}

fn as_dom_query_call<'a, 'b>(
    expr: &'b Expression<'a>,
    method: &str,
) -> Option<(&'b CallExpression<'a>, &'b StaticMemberExpression<'a>)> {
    let Expression::CallExpression(call_expr) = expr.without_parentheses() else {
        return None;
    };
    as_dom_query_member(call_expr, method).map(|member_expr| (call_expr.as_ref(), member_expr))
}

/// Returns the `.querySelectorAll()` member and the span of the access to remove
/// for `qsa[0]`, `qsa.at(0)` and `qsa.item(0)`.
fn get_first_element_access<'a, 'b>(
    kind: &AstKind<'a>,
) -> Option<(&'b StaticMemberExpression<'a>, Span)>
where
    'a: 'b,
{
    match kind {
        AstKind::ComputedMemberExpression(member_expr) => {
            if member_expr.optional || !member_expr.expression.is_number_0() {
                return None;
            }
            let (_, query_member) = as_dom_query_call(&member_expr.object, "querySelectorAll")?;
            Some((query_member, Span::new(member_expr.object.span().end, member_expr.span.end)))
        }
        AstKind::CallExpression(call_expr) => {
            if call_expr.optional
                || call_expr.arguments.len() != 1
                || !call_expr.arguments[0].as_expression().is_some_and(Expression::is_number_0)
            {
                return None;
            }
            let Expression::StaticMemberExpression(member_expr) =
                call_expr.callee.without_parentheses()
            else {
                return None;
            };
            if member_expr.optional || !matches!(member_expr.property.name.as_str(), "at" | "item")
            {
                return None;
            }
            let (_, query_member) = as_dom_query_call(&member_expr.object, "querySelectorAll")?;
            Some((query_member, Span::new(member_expr.object.span().end, call_expr.span.end)))
        }
        _ => None,
    }
}

fn check_first_element_access<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let Some((query_member, access_span)) = get_first_element_access(&node.kind()) else {
        return false;
    };
    if is_left_hand_side(node, ctx) {
        return false;
    }

    let diagnostic = first_match_diagnostic(node.span());
    if ctx.has_comments_between(access_span) {
        ctx.diagnostic(diagnostic);
    } else {
        ctx.diagnostic_with_fix(diagnostic, |fixer| {
            let fixer = fixer.for_multifix();
            let mut fix = fixer.new_fix_with_capacity(2);
            fix.push(fixer.replace(query_member.property.span, "querySelector"));
            fix.push(fixer.delete_range(access_span));
            fix.with_message("Replace with `.querySelector()`")
        });
    }
    true
}

fn is_part_of_first_element_access<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let Some(parent) = outermost_paren_parent(node, ctx) else {
        return false;
    };
    match parent.kind() {
        AstKind::ComputedMemberExpression(_) => get_first_element_access(&parent.kind()).is_some(),
        AstKind::StaticMemberExpression(_) => outermost_paren_parent(parent, ctx)
            .is_some_and(|grandparent| get_first_element_access(&grandparent.kind()).is_some()),
        _ => false,
    }
}

fn is_left_hand_side<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let span = node.span();
    match ctx.nodes().parent_kind(node.id()) {
        AstKind::AssignmentExpression(expr) => expr.left.span() == span,
        AstKind::AssignmentTargetWithDefault(target) => target.binding.span() == span,
        AstKind::AssignmentTargetPropertyProperty(property) => property.binding.span() == span,
        AstKind::UpdateExpression(expr) => expr.argument.span() == span,
        AstKind::UnaryExpression(expr) => expr.operator == UnaryOperator::Delete,
        AstKind::ForInStatement(stmt) => stmt.left.span() == span,
        AstKind::ForOfStatement(stmt) => stmt.left.span() == span,
        AstKind::ArrayAssignmentTarget(_) | AstKind::AssignmentTargetRest(_) => true,
        _ => false,
    }
}

fn get_static_string_argument<'a>(call_expr: &CallExpression<'a>) -> Option<&'a str> {
    match call_expr.arguments[0].as_expression()?.without_parentheses() {
        Expression::StringLiteral(literal) => literal.value.as_str(),
        Expression::TemplateLiteral(literal) => literal.single_quasi().and_then(JSStr::as_str),
        _ => None,
    }
}

fn is_simple_id_selector(selector: &str) -> bool {
    let Some(name) = selector.strip_prefix('#') else {
        return false;
    };
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '-' || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn get_const_initializer<'a>(
    ident: &IdentifierReference<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'a Expression<'a>> {
    let symbol_id = ctx.scoping().get_reference(ident.reference_id()).symbol_id()?;
    let AstKind::VariableDeclarator(declarator) =
        ctx.nodes().kind(ctx.scoping().symbol_declaration(symbol_id))
    else {
        return None;
    };
    if variable_declaration_kind(declarator, ctx.nodes()) != Some(VariableDeclarationKind::Const)
        || !matches!(declarator.id, BindingPattern::BindingIdentifier(_))
    {
        return None;
    }
    declarator.init.as_ref()
}

fn is_dom_query_result<'a>(expr: &Expression<'a>, method: &str, ctx: &LintContext<'a>) -> bool {
    match expr.without_parentheses() {
        Expression::CallExpression(call_expr) => as_dom_query_member(call_expr, method).is_some(),
        Expression::Identifier(ident) => get_const_initializer(ident, ctx)
            .is_some_and(|init| as_dom_query_call(init, method).is_some()),
        _ => false,
    }
}

fn is_nullish(expr: &Expression, ctx: &LintContext) -> bool {
    match expr {
        Expression::NullLiteral(_) => true,
        Expression::Identifier(ident) => {
            ident.name == "undefined" && ctx.is_reference_to_global_variable(ident)
        }
        _ => false,
    }
}

fn get_compared_nullish_value<'a, 'b>(
    left: &'b Expression<'a>,
    right: &'b Expression<'a>,
    method: &str,
    ctx: &LintContext<'a>,
) -> Option<&'b Expression<'a>> {
    if is_dom_query_result(left, method, ctx) && is_nullish(right, ctx) {
        Some(right)
    } else if is_dom_query_result(right, method, ctx) && is_nullish(left, ctx) {
        Some(left)
    } else {
        None
    }
}

fn is_global_boolean_call(call_expr: &CallExpression, ctx: &LintContext) -> bool {
    !call_expr.optional
        && call_expr.arguments.len() == 1
        && !matches!(call_expr.arguments[0], Argument::SpreadElement(_))
        && matches!(
            &call_expr.callee,
            Expression::Identifier(ident)
                if ident.name == "Boolean" && ctx.is_reference_to_global_variable(ident)
        )
}

/// Walks up through `!` and global `Boolean()` wrappers.
fn get_boolean_ancestor<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> (&'b AstNode<'a>, bool) {
    let mut current = node;
    let mut is_negative = false;
    while let Some(parent) = outermost_paren_parent(current, ctx) {
        match parent.kind() {
            AstKind::UnaryExpression(expr) if expr.operator == UnaryOperator::LogicalNot => {
                is_negative = !is_negative;
            }
            AstKind::CallExpression(call_expr)
                if is_global_boolean_call(call_expr, ctx)
                    && call_expr.arguments[0]
                        .as_expression()
                        .is_some_and(|arg| arg.without_parentheses().span() == current.span()) => {}
            _ => break,
        }
        current = parent;
    }
    (current, is_negative)
}

fn is_test_of(test: &Expression, node: &AstNode) -> bool {
    test.without_parentheses().span() == node.span()
}

fn is_control_flow_test<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let Some(parent) = outermost_paren_parent(node, ctx) else {
        return false;
    };
    match parent.kind() {
        AstKind::IfStatement(stmt) => is_test_of(&stmt.test, node),
        AstKind::ConditionalExpression(expr) => is_test_of(&expr.test, node),
        AstKind::WhileStatement(stmt) => is_test_of(&stmt.test, node),
        AstKind::DoWhileStatement(stmt) => is_test_of(&stmt.test, node),
        AstKind::ForStatement(stmt) => {
            stmt.test.as_ref().is_some_and(|test| is_test_of(test, node))
        }
        _ => is_logical_expression(parent) && is_control_flow_test(parent, ctx),
    }
}

fn check_length<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    if !outermost_paren_parent(node, ctx).is_some_and(|parent| {
        matches!(
            parent.kind(),
            AstKind::IfStatement(_)
                | AstKind::ConditionalExpression(_)
                | AstKind::WhileStatement(_)
                | AstKind::DoWhileStatement(_)
                | AstKind::ForStatement(_)
                | AstKind::LogicalExpression(_)
                | AstKind::UnaryExpression(_)
                | AstKind::CallExpression(_)
        )
    }) {
        return false;
    }

    let (ancestor, is_negative) = get_boolean_ancestor(node, ctx);
    if !is_control_flow_test(ancestor, ctx) {
        return false;
    }

    let is_query_selector_all = match node.kind() {
        AstKind::CallExpression(call_expr) => {
            as_dom_query_member(call_expr, "querySelectorAll").is_some()
        }
        AstKind::IdentifierReference(ident) => get_const_initializer(ident, ctx)
            .is_some_and(|init| as_dom_query_call(init, "querySelectorAll").is_some()),
        _ => false,
    };
    if !is_query_selector_all {
        return false;
    }

    let node_span = node.span();
    let ancestor_span = ancestor.span();
    let diagnostic = length_check_diagnostic(node_span);
    if would_remove_comments(ancestor_span, node_span, ctx)
        || has_comment_right_after(ancestor_span, ctx)
    {
        ctx.diagnostic(diagnostic);
    } else {
        ctx.diagnostic_with_fix(diagnostic, |fixer| {
            let comparison = if is_negative { "=== 0" } else { "> 0" };
            fixer.replace(
                ancestor_span,
                format!("{}.length {comparison}", fixer.source_range(node_span)),
            )
        });
    }
    true
}

fn would_remove_comments(replaced: Span, preserved: Span, ctx: &LintContext) -> bool {
    ctx.comments().iter().any(|comment| {
        let span = comment.span;
        span.start >= replaced.start
            && span.end <= replaced.end
            && !(span.start >= preserved.start && span.end <= preserved.end)
    })
}

fn has_comment_right_after(span: Span, ctx: &LintContext) -> bool {
    ctx.comments().iter().find(|comment| comment.span.start >= span.end).is_some_and(|comment| {
        ctx.source_range(Span::new(span.end, comment.span.start)).trim().is_empty()
    })
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"new document.querySelectorAll("form");"#,
        r#"querySelectorAll("form")[0];"#,
        r#"document["querySelectorAll"]("form")[0];"#,
        r#"document[querySelectorAll]("form")[0];"#,
        r#"document.querySelector("form");"#,
        "document.querySelectorAll();",
        r#"document.querySelectorAll("form", root);"#,
        r#"document.querySelectorAll(...["form"]);"#,
        r#"([]).querySelectorAll("form")[0]"#,
        r#"([element]).querySelectorAll("form")[0]"#,
        r#"([...elements]).querySelectorAll("form")[0]"#,
        r#"(() => {}).querySelectorAll("form")[0]"#,
        r#"(class Node {}).querySelectorAll("form")[0]"#,
        r#"(function() {}).querySelectorAll("form")[0]"#,
        r#"(0).querySelectorAll("form")[0]"#,
        r#"(1).querySelectorAll("form")[0]"#,
        r#"(0.1).querySelectorAll("form")[0]"#,
        r#"("").querySelectorAll("form")[0]"#,
        r#"("string").querySelectorAll("form")[0]"#,
        r#"(/regex/).querySelectorAll("form")[0]"#,
        r#"(null).querySelectorAll("form")[0]"#,
        r#"(0n).querySelectorAll("form")[0]"#,
        r#"(1n).querySelectorAll("form")[0]"#,
        r#"(true).querySelectorAll("form")[0]"#,
        r#"(false).querySelectorAll("form")[0]"#,
        r#"({}).querySelectorAll("form")[0]"#,
        r#"(`templateLiteral`).querySelectorAll("form")[0]"#,
        r#"(undefined).querySelectorAll("form")[0]"#,
        r#"document.querySelectorAll("form");"#,
        r#"document.querySelectorAll("form")[1];"#,
        r#"document.querySelectorAll("form")[index];"#,
        r#"document.querySelectorAll("form").at(1);"#,
        r#"document.querySelectorAll("form").at(index);"#,
        r#"document.querySelectorAll("form").item(1);"#,
        r#"document.querySelectorAll("form").item(index);"#,
        r#"document.querySelectorAll("form")?.[0];"#,
        r#"document.querySelectorAll("form")?.at(0);"#,
        r#"document.querySelectorAll("form").at?.(0);"#,
        r#"document.querySelectorAll("form")?.item(0);"#,
        r#"document.querySelectorAll("form").item?.(0);"#,
        r#"document.querySelectorAll("form")[0] = form;"#,
        r#"document.querySelectorAll("form")[0]++;"#,
        r#"delete document.querySelectorAll("form")[0];"#,
        r#"delete document.querySelectorAll("form").at(0);"#,
        r#"delete document.querySelectorAll("form").item(0);"#,
        r#"for (document.querySelectorAll("form")[0] in object) {}"#,
        r#"for (document.querySelectorAll("form")[0] of iterable) {}"#,
        r##"document.querySelectorAll("#foo .bar");"##,
        r##"document.querySelectorAll("#foo, #bar");"##,
        r##"document.querySelectorAll("#foo:checked");"##,
        r##"document.querySelectorAll("#foo bar");"##,
        r##"document.querySelectorAll("#1");"##,
        "document.querySelectorAll(`#${foo}`);",
        r#"const elements = document.querySelectorAll(".item"); if (elements.length > 0) {}"#,
        r#"const elements = document.querySelectorAll(".item"); if (elements.length) {}"#,
        r#"let elements = document.querySelectorAll(".item"); if (elements) {}"#,
        r#"const elements = document.querySelectorAll(".item"); const alias = elements; if (alias) {}"#,
        "function foo(elements) { if (elements) {} }",
        r#"function foo(Boolean) { if (Boolean(document.querySelectorAll(".item"))) {} }"#,
        r#"const Boolean = value => value; if (Boolean(document.querySelectorAll(".item"))) {}"#,
        r#"const Boolean = value => value; if (!Boolean(document.querySelectorAll(".item"))) {}"#,
        r#"const Boolean = value => value;
            const elements = document.querySelectorAll(".item");
            if (Boolean(elements)) {}"#,
        r#"document.querySelector("form")?.[0];"#,
        r#"document.querySelector("form")?.at(0);"#,
        r#"document.querySelector("form").at?.(0);"#,
        r#"document.querySelector("form")?.item(0);"#,
        r#"document.querySelector("form").item?.(0);"#,
        r#"document.querySelector("form")[0];"#,
        r#"document.querySelector("form").at(0);"#,
        r#"document.querySelector("form").item(0);"#,
        r#"document.querySelector("form")[0] = form;"#,
        r#"document.querySelector("form")[0]++;"#,
        r#"delete document.querySelector("form")[0];"#,
        r#"for (document.querySelector("form")[0] in object) {}"#,
        r#"for (document.querySelector("form")[0] of iterable) {}"#,
        r#"document.querySelector("form") === null;"#,
        r#"document.querySelector("form") !== null;"#,
        r#"document.querySelector("form") == undefined;"#,
        r#"document.querySelector("form") != undefined;"#,
        r#"const undefined = null; document.querySelector("form") === undefined;"#,
        r#"const undefined = null; document.querySelectorAll("form") === undefined;"#,
        r#"document.querySelectorAll("form") === nodes;"#,
        r#"document.querySelectorAll("form").length === 0;"#,
        r#"let elements = document.querySelectorAll("form"); elements === null;"#,
        r#"const elements = document.querySelectorAll("form"); const alias = elements; alias === null;"#,
    ];

    let fail = vec![
        r#"document.querySelectorAll("form")[0].addEventListener("submit", submitFunction);"#,
        r#"document.querySelectorAll("form").at(0).addEventListener("submit", submitFunction);"#,
        r#"document.querySelectorAll("form").item(0).addEventListener("submit", submitFunction);"#,
        r##"document.querySelectorAll("#foo").at(0);"##,
        r#"(document.querySelectorAll("form"))[0];"#,
        r#"(document.querySelectorAll("form")).at(0);"#,
        r##"document.querySelectorAll("#foo");"##,
        "document.querySelectorAll(`#foo`);",
        r#"if (document.querySelectorAll(".item")) {}"#,
        r#"if (!document.querySelectorAll(".item")) {}"#,
        r#"while (document.querySelectorAll(".item")) {}"#,
        r#"const hasItems = document.querySelectorAll(".item") ? true : false;"#,
        r#"if (document.querySelectorAll(".item") && ready) {}"#,
        r#"const elements = document.querySelectorAll(".item");
            if (elements && ready) {}"#,
        r#"if (Boolean(document.querySelectorAll(".item"))) {}"#,
        r#"if (!Boolean(document.querySelectorAll(".item"))) {}"#,
        r#"if (!!Boolean(document.querySelectorAll(".item"))) {}"#,
        r#"const elements = document.querySelectorAll(".item");
            if (elements) {}"#,
        r#"const elements = document.querySelectorAll(".item");
            if (!elements) {}"#,
        r#"if (Boolean(/* keep */ document.querySelectorAll(".item"))) {}"#,
        r#"if (!/* keep */ document.querySelectorAll(".item")) {}"#,
        r#"document.querySelectorAll(".item") === null;"#,
        r#"document.querySelectorAll(".item") !== null;"#,
        r#"document.querySelectorAll(".item") == null;"#,
        r#"document.querySelectorAll(".item") != null;"#,
        r#"document.querySelectorAll(".item") === undefined;"#,
        r#"document.querySelectorAll(".item") !== undefined;"#,
        r#"null === document.querySelectorAll(".item");"#,
        r#"undefined !== document.querySelectorAll(".item");"#,
        r#"document.querySelectorAll(".item") /* keep */ === null;"#,
        r#"document.querySelector(".item") === undefined;"#,
        r#"document.querySelector(".item") !== undefined;"#,
        r#"undefined === document.querySelector(".item");"#,
        r#"const elements = document.querySelectorAll(".item"); elements === null;"#,
        r#"const elements = document.querySelectorAll(".item"); elements !== undefined;"#,
        r#"const element = document.querySelector(".item"); element === undefined;"#,
        r#"for (const key in document.querySelectorAll("form")[0]) {}"#,
        r#"document.querySelectorAll("form") /* keep */ [0];"#,
        r#"document.querySelectorAll("form").at(/* keep */ 0);"#,
        r#"document.querySelectorAll("form").item(/* keep */ 0);"#,
        "if (document.querySelectorAll(/* c */ a)) {}",
        "if (!/* c */ document.querySelectorAll(a)) {}",
        "if (document.querySelectorAll(a) /* c */) {}",
        "if (document.querySelectorAll(a) // c
            ) {}",
        "if (document.querySelectorAll(a)) {
                // c
                foo();
            }",
        "const elements = document.querySelectorAll(a);
            if (!elements) {
                // c
            }",
        "const elements = document.querySelectorAll(a);
            if (!(/* c */ elements)) {}",
        "const elements = document.querySelectorAll(a);
            if ((/* c */ elements)) {}",
    ];

    let fix = vec![
        (
            r#"document.querySelectorAll("form")[0].addEventListener("submit", submitFunction);"#,
            r#"document.querySelector("form").addEventListener("submit", submitFunction);"#,
        ),
        (
            r#"document.querySelectorAll("form").at(0).addEventListener("submit", submitFunction);"#,
            r#"document.querySelector("form").addEventListener("submit", submitFunction);"#,
        ),
        (
            r#"document.querySelectorAll("form").item(0).addEventListener("submit", submitFunction);"#,
            r#"document.querySelector("form").addEventListener("submit", submitFunction);"#,
        ),
        (r##"document.querySelectorAll("#foo").at(0);"##, r##"document.querySelector("#foo");"##),
        (r#"(document.querySelectorAll("form"))[0];"#, r#"(document.querySelector("form"));"#),
        (r#"(document.querySelectorAll("form")).at(0);"#, r#"(document.querySelector("form"));"#),
        (
            r#"if (document.querySelectorAll(".item")) {}"#,
            r#"if (document.querySelectorAll(".item").length > 0) {}"#,
        ),
        (
            r#"if (!document.querySelectorAll(".item")) {}"#,
            r#"if (document.querySelectorAll(".item").length === 0) {}"#,
        ),
        (
            r#"while (document.querySelectorAll(".item")) {}"#,
            r#"while (document.querySelectorAll(".item").length > 0) {}"#,
        ),
        (
            r#"const hasItems = document.querySelectorAll(".item") ? true : false;"#,
            r#"const hasItems = document.querySelectorAll(".item").length > 0 ? true : false;"#,
        ),
        (
            r#"if (document.querySelectorAll(".item") && ready) {}"#,
            r#"if (document.querySelectorAll(".item").length > 0 && ready) {}"#,
        ),
        (
            r#"const elements = document.querySelectorAll(".item");
            if (elements && ready) {}"#,
            r#"const elements = document.querySelectorAll(".item");
            if (elements.length > 0 && ready) {}"#,
        ),
        (
            r#"if (Boolean(document.querySelectorAll(".item"))) {}"#,
            r#"if (document.querySelectorAll(".item").length > 0) {}"#,
        ),
        (
            r#"if (!Boolean(document.querySelectorAll(".item"))) {}"#,
            r#"if (document.querySelectorAll(".item").length === 0) {}"#,
        ),
        (
            r#"if (!!Boolean(document.querySelectorAll(".item"))) {}"#,
            r#"if (document.querySelectorAll(".item").length > 0) {}"#,
        ),
        (
            r#"const elements = document.querySelectorAll(".item");
            if (elements) {}"#,
            r#"const elements = document.querySelectorAll(".item");
            if (elements.length > 0) {}"#,
        ),
        (
            r#"const elements = document.querySelectorAll(".item");
            if (!elements) {}"#,
            r#"const elements = document.querySelectorAll(".item");
            if (elements.length === 0) {}"#,
        ),
        (
            r#"for (const key in document.querySelectorAll("form")[0]) {}"#,
            r#"for (const key in document.querySelector("form")) {}"#,
        ),
        (
            "if (document.querySelectorAll(/* c */ a)) {}",
            "if (document.querySelectorAll(/* c */ a).length > 0) {}",
        ),
        (
            "if (document.querySelectorAll(a)) {
                // c
                foo();
            }",
            "if (document.querySelectorAll(a).length > 0) {
                // c
                foo();
            }",
        ),
        (
            "const elements = document.querySelectorAll(a);
            if (!elements) {
                // c
            }",
            "const elements = document.querySelectorAll(a);
            if (elements.length === 0) {
                // c
            }",
        ),
        (
            "const elements = document.querySelectorAll(a);
            if ((/* c */ elements)) {}",
            "const elements = document.querySelectorAll(a);
            if ((/* c */ elements.length > 0)) {}",
        ),
    ];

    Tester::new(NoIncorrectQuerySelector::NAME, NoIncorrectQuerySelector::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
