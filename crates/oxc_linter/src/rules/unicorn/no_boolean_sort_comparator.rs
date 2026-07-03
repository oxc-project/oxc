use oxc_ast::{
    AstKind,
    ast::{
        Argument, ArrowFunctionExpression, BinaryExpression, BindingPattern, CallExpression,
        Expression, Function, FunctionBody, IdentifierReference, MemberExpression, Statement,
        TSLiteral, TSType, TSTypeName, VariableDeclarator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{IsGlobalReference, SymbolId};
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, UnaryOperator};
use rustc_hash::FxHashSet;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_boolean_sort_comparator_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not return a boolean from a sort comparator.")
        .with_help(
            "A sort comparator must return a number: negative, zero, or positive. Returning a boolean sorts inconsistently.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoBooleanSortComparator;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows passing a comparator that returns a boolean to `Array#sort()` or
    /// `Array#toSorted()`.
    ///
    /// ### Why is this bad?
    ///
    /// `Array#sort()` expects its comparator to return a number: a negative value if the first
    /// argument should come first, a positive value if the second should come first, and `0` if
    /// they are equal. A boolean-returning comparator such as `(a, b) => a > b` only ever returns
    /// `true`/`false`, which coerce to `1`/`0`. The engine never learns that an element should move
    /// *before* another, so the array is sorted incorrectly and the result is implementation
    /// dependent.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// array.sort((a, b) => a > b);
    /// array.sort((a, b) => a.score >= b.score);
    /// array.sort(Boolean);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// array.sort((a, b) => a - b);
    /// array.sort((a, b) => a.score - b.score);
    /// array.sort((a, b) => a.localeCompare(b));
    /// ```
    NoBooleanSortComparator,
    unicorn,
    correctness,
    suggestion,
    version = "next",
    short_description = "Disallow boolean-returning sort comparators.",
);

impl Rule for NoBooleanSortComparator {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call) = node.kind() else {
            return;
        };

        let Some(member) = call.callee.get_member_expr() else {
            return;
        };

        if !is_sort_method(member, ctx) {
            return;
        }

        // Skip receivers that are provably not arrays, e.g. `new Set().sort(...)`.
        if is_known_non_array(member.object().get_inner_expression(), ctx) {
            return;
        }

        let Some(comparator) = call.arguments.first().and_then(Argument::as_expression) else {
            return;
        };

        report_boolean_comparator(comparator, ctx);
    }
}

const SORT_METHODS: [&str; 2] = ["sort", "toSorted"];

/// Is `member` a `.sort`/`.toSorted` access, resolving computed keys that reference a `const`
/// string (e.g. `const method = "sort"; array[method](...)`)?
fn is_sort_method<'a>(member: &MemberExpression<'a>, ctx: &LintContext<'a>) -> bool {
    if let Some(name) = member.static_property_name() {
        return SORT_METHODS.contains(&name);
    }

    if let MemberExpression::ComputedMemberExpression(computed) = member
        && let Expression::Identifier(ident) = computed.expression.get_inner_expression()
        && let Some(Expression::StringLiteral(lit)) = resolve_const_init(ident, ctx)
    {
        return SORT_METHODS.contains(&lit.value.as_str());
    }

    false
}

/// Mirrors eslint-plugin-unicorn's `isKnownNonArray` for the syntactic cases oxlint can determine
/// without type information.
fn is_known_non_array<'a>(expr: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    match expr {
        Expression::ObjectExpression(_)
        | Expression::FunctionExpression(_)
        | Expression::ArrowFunctionExpression(_)
        | Expression::ClassExpression(_)
        | Expression::TemplateLiteral(_) => true,
        // Any `new X()` that is not `new Array()` is not an array.
        Expression::NewExpression(new_expr) => !matches!(
            new_expr.callee.get_inner_expression(),
            Expression::Identifier(ident)
                if ident.name == "Array" && ident.is_global_reference(ctx.scoping())
        ),
        Expression::Identifier(ident) => resolve_const_init(ident, ctx)
            .is_some_and(|init| is_known_non_array(init.get_inner_expression(), ctx)),
        _ => false,
    }
}

fn report_boolean_comparator<'a>(comparator: &'a Expression<'a>, ctx: &LintContext<'a>) {
    // Parentheses are not part of the ESTree AST the upstream rule operates on, so peel them off.
    let comparator = comparator.without_parentheses();
    let unwrapped = unwrap_typescript_expression(comparator);

    if let Some(func) = as_function(unwrapped) {
        let mut visited = FxHashSet::default();
        if is_boolean_function(&func, ctx, &mut visited)
            || has_boolean_function_type_assertion(comparator, ctx)
        {
            // Only offer a fix when the comparator has no wrapping type assertion.
            let suggestion = if std::ptr::eq(comparator, unwrapped) {
                build_suggestion(&func, ctx)
            } else {
                None
            };
            report(comparator, suggestion, ctx);
        }
        return;
    }

    let mut visited = FxHashSet::default();
    if is_boolean_function_reference(unwrapped, ctx, &mut visited)
        || is_known_boolean_function_reference(unwrapped, ctx)
        || has_boolean_function_type_assertion(comparator, ctx)
    {
        report(comparator, None, ctx);
    }
}

fn report<'a>(comparator: &Expression<'a>, suggestion: Option<String>, ctx: &LintContext<'a>) {
    let span = comparator.span();
    let diagnostic = no_boolean_sort_comparator_diagnostic(span);
    if let Some(replacement) = suggestion {
        ctx.diagnostic_with_suggestion(diagnostic, move |fixer| fixer.replace(span, replacement));
    } else {
        ctx.diagnostic(diagnostic);
    }
}

enum FunctionLike<'a, 'b> {
    Arrow(&'b ArrowFunctionExpression<'a>),
    Function(&'b Function<'a>),
}

fn as_function<'a, 'b>(expr: &'b Expression<'a>) -> Option<FunctionLike<'a, 'b>> {
    match expr {
        Expression::ArrowFunctionExpression(arrow) => Some(FunctionLike::Arrow(arrow)),
        Expression::FunctionExpression(func) => Some(FunctionLike::Function(func)),
        _ => None,
    }
}

/// Does the function return a boolean, either by its return type annotation or its single-return
/// body?
fn is_boolean_function<'a>(
    func: &FunctionLike<'a, '_>,
    ctx: &LintContext<'a>,
    visited: &mut FxHashSet<SymbolId>,
) -> bool {
    let (is_async, is_generator, return_type, body, is_expression) = match func {
        FunctionLike::Arrow(arrow) => {
            (arrow.r#async, false, &arrow.return_type, Some(&*arrow.body), arrow.expression)
        }
        FunctionLike::Function(func) => {
            (func.r#async, func.generator, &func.return_type, func.body.as_deref(), false)
        }
    };

    if is_async || is_generator {
        return false;
    }

    if let Some(return_type) = return_type
        && is_boolean_type_annotation(&return_type.type_annotation, ctx, &mut FxHashSet::default())
    {
        return true;
    }

    let Some(body) = body else {
        return false;
    };

    let Some(return_expression) = function_return_expression(body, is_expression) else {
        return false;
    };

    is_boolean_expression(return_expression, ctx, visited)
}

/// The single expression a function evaluates to, if it has exactly one.
fn function_return_expression<'a, 'b>(
    body: &'b FunctionBody<'a>,
    is_expression: bool,
) -> Option<&'b Expression<'a>> {
    if is_expression {
        if let [Statement::ExpressionStatement(stmt)] = body.statements.as_slice() {
            return Some(&stmt.expression);
        }
        return None;
    }

    if let [Statement::ReturnStatement(stmt)] = body.statements.as_slice() {
        return stmt.argument.as_ref();
    }

    None
}

fn is_boolean_expression<'a>(
    expr: &Expression<'a>,
    ctx: &LintContext<'a>,
    visited: &mut FxHashSet<SymbolId>,
) -> bool {
    match expr {
        Expression::BooleanLiteral(_) => true,
        Expression::Identifier(ident) => is_boolean_identifier(ident, ctx, visited),
        Expression::UnaryExpression(unary) => {
            matches!(unary.operator, UnaryOperator::LogicalNot | UnaryOperator::Delete)
        }
        Expression::LogicalExpression(logical) => {
            is_boolean_expression(&logical.left, ctx, visited)
                && is_boolean_expression(&logical.right, ctx, visited)
        }
        Expression::BinaryExpression(binary) => is_boolean_binary_expression(binary),
        Expression::ConditionalExpression(conditional) => {
            is_boolean_expression(&conditional.consequent, ctx, visited)
                && is_boolean_expression(&conditional.alternate, ctx, visited)
        }
        Expression::SequenceExpression(sequence) => sequence
            .expressions
            .last()
            .is_some_and(|last| is_boolean_expression(last, ctx, visited)),
        Expression::AssignmentExpression(assignment) => {
            assignment.operator == AssignmentOperator::Assign
                && is_boolean_expression(&assignment.right, ctx, visited)
        }
        Expression::ParenthesizedExpression(paren) => {
            is_boolean_expression(&paren.expression, ctx, visited)
        }
        Expression::AwaitExpression(await_expr) => {
            is_boolean_expression(&await_expr.argument, ctx, visited)
        }
        Expression::TSNonNullExpression(non_null) => {
            is_boolean_expression(&non_null.expression, ctx, visited)
        }
        Expression::TSAsExpression(as_expr) => {
            is_boolean_type_annotation(&as_expr.type_annotation, ctx, &mut FxHashSet::default())
                || is_boolean_expression(&as_expr.expression, ctx, visited)
        }
        Expression::TSSatisfiesExpression(satisfies) => {
            is_boolean_type_annotation(&satisfies.type_annotation, ctx, &mut FxHashSet::default())
                || is_boolean_expression(&satisfies.expression, ctx, visited)
        }
        Expression::TSTypeAssertion(assertion) => {
            is_boolean_type_annotation(&assertion.type_annotation, ctx, &mut FxHashSet::default())
                || is_boolean_expression(&assertion.expression, ctx, visited)
        }
        Expression::CallExpression(call) => is_boolean_call_expression(call, ctx, visited),
        _ => false,
    }
}

fn is_boolean_binary_expression(binary: &BinaryExpression) -> bool {
    binary.operator.is_equality() || binary.operator.is_compare() || binary.operator.is_relational()
}

fn is_boolean_identifier<'a>(
    ident: &IdentifierReference<'a>,
    ctx: &LintContext<'a>,
    visited: &mut FxHashSet<SymbolId>,
) -> bool {
    let Some(symbol_id) = ctx.scoping().get_reference(ident.reference_id()).symbol_id() else {
        return false;
    };
    if !visited.insert(symbol_id) {
        return false;
    }

    let declaration = ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id));
    let result = if let AstKind::VariableDeclarator(declarator) = declaration.kind() {
        declarator.kind.is_const()
            && declarator
                .init
                .as_ref()
                .is_some_and(|init| is_boolean_expression(init, ctx, visited))
    } else {
        false
    };

    visited.remove(&symbol_id);
    result
}

fn is_boolean_call_expression<'a>(
    call: &CallExpression<'a>,
    ctx: &LintContext<'a>,
    visited: &mut FxHashSet<SymbolId>,
) -> bool {
    let callee = call.callee.get_inner_expression();
    is_known_boolean_function_reference(callee, ctx)
        || is_boolean_function_reference(callee, ctx, visited)
}

/// A direct reference to a known boolean-returning global, e.g. `Boolean`, `isNaN`,
/// `Number.isInteger`, `Array.isArray`.
fn is_known_boolean_function_reference<'a>(expr: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    match expr {
        Expression::Identifier(ident) => {
            matches!(ident.name.as_str(), "Boolean" | "isFinite" | "isNaN")
                && ident.is_global_reference(ctx.scoping())
        }
        Expression::StaticMemberExpression(_) | Expression::ComputedMemberExpression(_) => {
            let Some(member) = expr.as_member_expression() else {
                return false;
            };
            let Expression::Identifier(object) = member.object().get_inner_expression() else {
                return false;
            };
            object.is_global_reference(ctx.scoping())
                && member
                    .static_property_name()
                    .is_some_and(|method| is_known_boolean_static_method(&object.name, method))
        }
        _ => false,
    }
}

fn is_known_boolean_static_method(object: &str, method: &str) -> bool {
    matches!(
        (object, method),
        ("Array", "isArray")
            | ("ArrayBuffer", "isView")
            | ("Atomics", "isLockFree")
            | ("Error", "isError")
            | ("Number", "isFinite" | "isInteger" | "isNaN" | "isSafeInteger")
            | ("Object", "hasOwn" | "is" | "isExtensible" | "isFrozen" | "isSealed")
            | ("Reflect", "deleteProperty" | "has")
            | ("URL", "canParse")
    )
}

/// A reference to a locally-declared function/const that returns a boolean, or a variable typed as
/// a boolean-returning function.
fn is_boolean_function_reference<'a>(
    expr: &Expression<'a>,
    ctx: &LintContext<'a>,
    visited: &mut FxHashSet<SymbolId>,
) -> bool {
    let Expression::Identifier(ident) = expr else {
        return false;
    };
    let Some(symbol_id) = ctx.scoping().get_reference(ident.reference_id()).symbol_id() else {
        return false;
    };
    if !visited.insert(symbol_id) {
        return false;
    }

    let declaration = ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id));
    let result = match declaration.kind() {
        AstKind::VariableDeclarator(declarator) => {
            is_boolean_function_declarator(declarator, ctx, visited)
        }
        AstKind::Function(func) => is_boolean_function(&FunctionLike::Function(func), ctx, visited),
        _ => false,
    };

    visited.remove(&symbol_id);
    result
}

fn is_boolean_function_declarator<'a>(
    declarator: &VariableDeclarator<'a>,
    ctx: &LintContext<'a>,
    visited: &mut FxHashSet<SymbolId>,
) -> bool {
    if let Some(type_annotation) = &declarator.type_annotation
        && is_boolean_function_type_annotation(
            &type_annotation.type_annotation,
            ctx,
            &mut FxHashSet::default(),
        )
    {
        return true;
    }

    declarator.kind.is_const()
        && declarator
            .init
            .as_ref()
            .and_then(as_function)
            .is_some_and(|func| is_boolean_function(&func, ctx, visited))
}

fn unwrap_typescript_expression<'a, 'b>(mut expr: &'b Expression<'a>) -> &'b Expression<'a> {
    loop {
        expr = match expr {
            Expression::TSAsExpression(e) => &e.expression,
            Expression::TSSatisfiesExpression(e) => &e.expression,
            Expression::TSNonNullExpression(e) => &e.expression,
            Expression::TSTypeAssertion(e) => &e.expression,
            Expression::ParenthesizedExpression(e) => &e.expression,
            _ => return expr,
        };
    }
}

fn has_boolean_function_type_assertion<'a>(
    mut expr: &Expression<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    loop {
        let type_annotation = match expr {
            Expression::TSAsExpression(e) => {
                expr = &e.expression;
                Some(&e.type_annotation)
            }
            Expression::TSSatisfiesExpression(e) => {
                expr = &e.expression;
                Some(&e.type_annotation)
            }
            Expression::TSTypeAssertion(e) => {
                expr = &e.expression;
                Some(&e.type_annotation)
            }
            Expression::TSNonNullExpression(e) => {
                expr = &e.expression;
                None
            }
            Expression::ParenthesizedExpression(e) => {
                expr = &e.expression;
                None
            }
            _ => return false,
        };
        if let Some(type_annotation) = type_annotation
            && is_boolean_function_type_annotation(type_annotation, ctx, &mut FxHashSet::default())
        {
            return true;
        }
    }
}

fn is_boolean_type_annotation<'a>(
    ts_type: &TSType<'a>,
    ctx: &LintContext<'a>,
    visited: &mut FxHashSet<SymbolId>,
) -> bool {
    match ts_type {
        TSType::TSBooleanKeyword(_) => true,
        TSType::TSLiteralType(literal) => matches!(literal.literal, TSLiteral::BooleanLiteral(_)),
        TSType::TSTypePredicate(predicate) => !predicate.asserts,
        TSType::TSParenthesizedType(paren) => {
            is_boolean_type_annotation(&paren.type_annotation, ctx, visited)
        }
        TSType::TSUnionType(union) => {
            union.types.iter().all(|ty| is_boolean_type_annotation(ty, ctx, visited))
        }
        TSType::TSTypeReference(reference) => {
            resolve_type_alias(&reference.type_name, ctx, visited)
                .is_some_and(|ty| is_boolean_type_annotation(ty, ctx, visited))
        }
        _ => false,
    }
}

fn is_boolean_function_type_annotation<'a>(
    ts_type: &TSType<'a>,
    ctx: &LintContext<'a>,
    visited: &mut FxHashSet<SymbolId>,
) -> bool {
    match ts_type {
        TSType::TSFunctionType(func) => {
            is_boolean_type_annotation(&func.return_type.type_annotation, ctx, visited)
        }
        TSType::TSParenthesizedType(paren) => {
            is_boolean_function_type_annotation(&paren.type_annotation, ctx, visited)
        }
        TSType::TSUnionType(union) => {
            union.types.iter().all(|ty| is_boolean_function_type_annotation(ty, ctx, visited))
        }
        TSType::TSTypeReference(reference) => {
            resolve_type_alias(&reference.type_name, ctx, visited)
                .is_some_and(|ty| is_boolean_function_type_annotation(ty, ctx, visited))
        }
        _ => false,
    }
}

/// Resolve a `TSTypeReference` name to the `type X = ...` it aliases, guarding against cycles.
fn resolve_type_alias<'a>(
    type_name: &TSTypeName<'a>,
    ctx: &LintContext<'a>,
    visited: &mut FxHashSet<SymbolId>,
) -> Option<&'a TSType<'a>> {
    let TSTypeName::IdentifierReference(ident) = type_name else {
        return None;
    };
    let symbol_id = ctx.scoping().get_reference(ident.reference_id()).symbol_id()?;
    if !visited.insert(symbol_id) {
        return None;
    }

    let declaration = ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id));
    let AstKind::TSTypeAliasDeclaration(alias) = declaration.kind() else {
        return None;
    };
    Some(&alias.type_annotation)
}

fn resolve_const_init<'a>(
    ident: &IdentifierReference<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'a Expression<'a>> {
    let symbol_id = ctx.scoping().get_reference(ident.reference_id()).symbol_id()?;
    let declaration = ctx.nodes().get_node(ctx.scoping().symbol_declaration(symbol_id));
    let AstKind::VariableDeclarator(declarator) = declaration.kind() else {
        return None;
    };
    if !declarator.kind.is_const() {
        return None;
    }
    declarator.init.as_ref()
}

// ---------------------------------------------------------------------------
// Suggestion: rewrite `(a, b) => a > b` into `(a, b) => a - b`.
// ---------------------------------------------------------------------------

fn build_suggestion<'a>(func: &FunctionLike<'a, '_>, ctx: &LintContext<'a>) -> Option<String> {
    let (span, params, body, is_expression) = match func {
        FunctionLike::Arrow(arrow) => {
            (arrow.span, &arrow.params, Some(&*arrow.body), arrow.expression)
        }
        FunctionLike::Function(function) => {
            (function.span, &function.params, function.body.as_deref(), false)
        }
    };
    // Rewriting would drop a comment, so leave it to the user.
    if ctx.comments_range(span.start..span.end).count() > 0 {
        return None;
    }

    let [first, second] = params.items.as_slice() else {
        return None;
    };
    if params.rest.is_some()
        || first.type_annotation.is_some()
        || second.type_annotation.is_some()
        || first.optional
        || second.optional
    {
        return None;
    }
    let (first_name, second_name) = (binding_name(&first.pattern)?, binding_name(&second.pattern)?);
    if first_name == second_name {
        return None;
    }

    let body = body?;
    let Expression::BinaryExpression(binary) = function_return_expression(body, is_expression)?
    else {
        return None;
    };
    if !binary.operator.is_compare() {
        return None;
    }
    if !is_parameter_mirror(&binary.left, &binary.right, first_name, second_name) {
        return None;
    }

    let (minuend, subtrahend) = match binary.operator {
        BinaryOperator::GreaterThan | BinaryOperator::GreaterEqualThan => {
            (&binary.left, &binary.right)
        }
        _ => (&binary.right, &binary.left),
    };
    Some(format!(
        "({first_name}, {second_name}) => {} - {}",
        ctx.source_range(minuend.span()),
        ctx.source_range(subtrahend.span())
    ))
}

fn binding_name<'a>(pattern: &'a BindingPattern<'a>) -> Option<&'a str> {
    if let BindingPattern::BindingIdentifier(ident) = pattern {
        Some(ident.name.as_str())
    } else {
        None
    }
}

/// Do `left` and `right` reference the two comparator parameters in the same shape, e.g.
/// `a.foo.bar` vs `b.foo.bar`?
fn is_parameter_mirror(left: &Expression, right: &Expression, first: &str, second: &str) -> bool {
    match (left.get_inner_expression(), right.get_inner_expression()) {
        (Expression::Identifier(left_ident), Expression::Identifier(right_ident)) => {
            let (l, r) = (left_ident.name.as_str(), right_ident.name.as_str());
            (l == first && r == second) || (l == second && r == first)
        }
        (
            Expression::StaticMemberExpression(left_member),
            Expression::StaticMemberExpression(right_member),
        ) => {
            left_member.property.name == right_member.property.name
                && is_parameter_mirror(&left_member.object, &right_member.object, first, second)
        }
        (
            Expression::ComputedMemberExpression(left_member),
            Expression::ComputedMemberExpression(right_member),
        ) => {
            is_same_property(&left_member.expression, &right_member.expression)
                && is_parameter_mirror(&left_member.object, &right_member.object, first, second)
        }
        _ => false,
    }
}

fn is_same_property(left: &Expression, right: &Expression) -> bool {
    match (left.get_inner_expression(), right.get_inner_expression()) {
        (Expression::Identifier(l), Expression::Identifier(r)) => l.name == r.name,
        (Expression::NumericLiteral(l), Expression::NumericLiteral(r)) => {
            l.value.to_bits() == r.value.to_bits()
        }
        (Expression::StringLiteral(l), Expression::StringLiteral(r)) => l.value == r.value,
        _ => false,
    }
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::{TestCase, Tester};

    let pass: Vec<TestCase> = vec![
        "array.sort()".into(),
        "array.toSorted()".into(),
        "array.sort(compareFunction)".into(),
        "array.sort((a, b) => a - b)".into(),
        "array.sort((a, b) => b - a)".into(),
        "array.sort((a, b) => a.score - b.score)".into(),
        "array.sort((a, b) => a.localeCompare(b))".into(),
        "array.toSorted((a, b) => a.localeCompare(b))".into(),
        "array.sort((a, b) => a > b ? 1 : -1)".into(),
        "array.sort((a, b) => a > b ? 1 : a < b ? -1 : 0)".into(),
        "array.sort((a, b) => a.foo - b.foo || a.bar - b.bar)".into(),
        "array.sort((a, b) => Math.random() - 0.5)".into(),
        "array.sort(...comparators)".into(),
        "array.sort(...comparators, extraArgument)".into(),
        "array[sort]((a, b) => a > b)".into(),
        "new Set().sort((a, b) => a > b)".into(),
        "({sort() {}}).sort((a, b) => a > b)".into(),
        "const collection = new Set(); collection.sort((a, b) => a > b)".into(),
        "const object = {}; object.sort((a, b) => a > b)".into(),
        "const function_ = () => {}; function_.sort((a, b) => a > b)".into(),
        "array.sort(async (a, b) => a > b)".into(),
        "array.sort(function * (a, b) { return a > b; })".into(),
        "array.sort((a, b) => unknownBoolean)".into(),
        "const Boolean = () => 0;\n array.sort(Boolean)".into(),
        "const isFinite = () => 0;\n array.sort(isFinite)".into(),
        "const Number = {isFinite() { return 0; }};\n array.sort(Number.isFinite)".into(),
        r#"const Number = {isNaN() { return 0; }};
            array.sort(Number["isNaN"])"#
            .into(),
        "const Array = {isArray() { return 0; }};\n array.sort(Array.isArray)".into(),
        "const Object = {is() { return 0; }};\n array.sort(Object.is)".into(),
        "array.sort((a, b) => {\n if (a > b) { return true; }\n return false;\n })".into(),
        "function compare(a, b) { return a - b; }\n array.sort(compare)".into(),
    ];

    let fail: Vec<TestCase> = vec![
        "array.sort((a, b) => a > b)".into(),
        "[].sort((a, b) => a > b)".into(),
        "Array.from(iterable).sort((a, b) => a > b)".into(),
        "new Array().toSorted((a, b) => a > b)".into(),
        "array.sort((a, b) => a >= b)".into(),
        "array.sort((a, b) => a < b)".into(),
        "array.sort((a, b) => a <= b)".into(),
        "array.sort((a, b) => b > a)".into(),
        "array.sort((a, b) => b >= a)".into(),
        "array.sort((a, b) => b < a)".into(),
        "array.sort((a, b) => b <= a)".into(),
        "array.sort((a, b) => a.score > b.score)".into(),
        "array.sort((a, b) => a.score >= b.score)".into(),
        "array.sort((a, b) => a[0] < b[0])".into(),
        "array.sort((a, b) => a[i] <= b[i])".into(),
        "array.sort((a, b) => a.foo.bar > b.foo.bar)".into(),
        "array.sort((a, b) => a > c)".into(),
        "array.sort((a, b) => a.foo > b.bar)".into(),
        "array.sort((a, b) => a[i] > b[j])".into(),
        "array.sort((a, b) => a > b, extraArgument)".into(),
        r#"array["sort"]((a, b) => a > b)"#.into(),
        r#"array["toSorted"]((a, b) => a > b)"#.into(),
        r#"const method = "sort";
            array[method]((a, b) => a > b)"#
            .into(),
        "array.toSorted((a, b) => a > b)".into(),
        "array.sort?.((a, b) => a > b)".into(),
        "array?.sort((a, b) => a > b)".into(),
        "array.sort(function (a, b) { return a > b; })".into(),
        ("array.sort(function (a, a) { return a > a; })", None, None, Some(PathBuf::from("test.cjs")))
            .into(),
        "array.sort((a, b) => {\n return a > b;\n })".into(),
        "array.sort((a, b) => (a) > (b))".into(),
        "array.sort((a, b) => {\n // Compare\n return a > b;\n })".into(),
        "array.sort((a, b) => a === b)".into(),
        "strings.toSorted((a, b) => a === b)".into(),
        "array.sort((a, b) => a !== b)".into(),
        "array.sort((a, b) => true)".into(),
        "array.sort((a, b) => false)".into(),
        "array.sort((a, b) => Boolean(a - b))".into(),
        "array.sort((a, b) => !(a - b))".into(),
        "array.sort((a, b) => a > b && c > d)".into(),
        "array.sort((a, b) => a > b ? c > d : e > f)".into(),
        "array.sort((a, b) => a in b)".into(),
        "array.sort((a, b) => a instanceof b)".into(),
        "array.sort(Boolean)".into(),
        "array.toSorted(Boolean)".into(),
        "array.sort(isFinite)".into(),
        "array.sort(isNaN)".into(),
        "array.sort(Array.isArray)".into(),
        "array.sort(ArrayBuffer.isView)".into(),
        "array.sort(Atomics.isLockFree)".into(),
        "array.sort(Number.isFinite)".into(),
        "array.sort(Number.isInteger)".into(),
        "array.sort(Number.isNaN)".into(),
        r#"array.sort(Number["isNaN"])"#.into(),
        "array.sort(Number.isSafeInteger)".into(),
        "array.sort(Object.hasOwn)".into(),
        "array.sort(Object.is)".into(),
        "array.sort(Object.isExtensible)".into(),
        "array.sort(Object.isFrozen)".into(),
        "array.sort(Object.isSealed)".into(),
        "array.sort(Reflect.deleteProperty)".into(),
        "array.sort(Reflect.has)".into(),
        "array.sort(URL.canParse)".into(),
        "const compare = (a, b) => a > b;\n array.sort(compare)".into(),
        "const compare = function (a, b) { return a > b; };\n array.sort(compare)".into(),
        "function compare(a, b) { return a > b; }\n array.sort(compare)".into(),
        "array.sort((a: number, b: number): boolean => a > b)".into(),
        "array.sort((Boolean as (value: unknown) => boolean))".into(),
        "const compare: (a: number, b: number) => boolean = (a, b) => compareUnknown(a, b);\n array.sort(compare)".into(),
        "type Comparator = (a: number, b: number) => boolean;\n array.sort(((a, b) => a > b) as Comparator)".into(),
        "type Comparator = (a: number, b: number) => boolean;\n array.sort(compare as Comparator)".into(),
        "array.sort(((a: number, b: number) => a > b) as (a: number, b: number) => boolean)".into(),
        "array.sort(compare as (a: number, b: number) => boolean)".into(),
        "array.sort((a: number, b: number): boolean => compare(a, b))".into(),
        "array.sort(function (a: number, b: number): boolean {\n return compare(a, b);\n })".into(),
        "array.sort((a: number, b: number) => (a > b) as boolean)".into(),
        ("array.sort((a: number, b: number) => <boolean>(a > b))", None, None, Some(PathBuf::from("test.ts"))).into(),
        "array.sort((a: number, b: number) => (a > b) satisfies boolean)".into(),
        "array.sort((a: number, b: number) => (a > b)!)".into(),
        ("array.sort(<T extends {score: number}>(a, b) => a.score > b.score)", None, None, Some(PathBuf::from("test.ts"))).into(),
        "array.sort((a?, b?) => a > b)".into(),
        "array.sort((a: number, b: number): boolean => {\n if (a > b) { return true; }\n return false;\n })".into(),
        "array.sort((a: unknown, b: unknown) => (a as number) > (b as number))".into(),
        "array.sort(Boolean!)".into(),
        "array.sort(([a], [b]) => a > b)".into(),
    ];

    let fix = vec![
        ("array.sort((a, b) => a > b)", "array.sort((a, b) => a - b)"),
        ("array.sort((a, b) => a >= b)", "array.sort((a, b) => a - b)"),
        ("array.sort((a, b) => a < b)", "array.sort((a, b) => b - a)"),
        ("array.sort((a, b) => a <= b)", "array.sort((a, b) => b - a)"),
        ("array.sort((a, b) => b > a)", "array.sort((a, b) => b - a)"),
        ("array.sort((a, b) => a.score > b.score)", "array.sort((a, b) => a.score - b.score)"),
        ("array.sort((a, b) => a[0] < b[0])", "array.sort((a, b) => b[0] - a[0])"),
        (
            "array.sort((a, b) => a.foo.bar > b.foo.bar)",
            "array.sort((a, b) => a.foo.bar - b.foo.bar)",
        ),
        ("array.sort((a, b) => { return a > b; })", "array.sort((a, b) => a - b)"),
        ("array.sort(function (a, b) { return a > b; })", "array.sort((a, b) => a - b)"),
        // No suggestion offered — output is unchanged.
        ("array.sort((a, b) => a === b)", "array.sort((a, b) => a === b)"),
        ("array.sort((a, b) => a > c)", "array.sort((a, b) => a > c)"),
        ("array.sort(Boolean)", "array.sort(Boolean)"),
        ("array.sort(([a], [b]) => a > b)", "array.sort(([a], [b]) => a > b)"),
        // A comment inside the comparator disables the fix.
        (
            "array.sort((a, b) => { /* keep */ return a > b; })",
            "array.sort((a, b) => { /* keep */ return a > b; })",
        ),
    ];

    Tester::new(NoBooleanSortComparator::NAME, NoBooleanSortComparator::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
