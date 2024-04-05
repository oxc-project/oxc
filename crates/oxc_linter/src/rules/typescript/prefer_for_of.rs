use oxc_ast::ast::{
    AssignmentTarget, BindingPatternKind, Expression, ForStatementInit, SimpleAssignmentTarget,
    VariableDeclarationKind,
};
use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, GetSpan, Span};
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, UnaryOperator, UpdateOperator};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(prefer-for-of): Expected a `for-of` loop instead of a `for` loop with this simple iteration.")]
#[diagnostic(severity(warning), help("Consider using a for-of loop for this simple iteration."))]
struct PreferForOfDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct PreferForOf;

declare_oxc_lint!(
    /// ### What it does
    /// Enforces the use of for-of loop instead of a for loop with a simple iteration.
    ///
    /// ### Why is this bad?
    /// Using a for loop with a simple iteration over an array can be replaced with a more concise
    /// and readable for-of loop. For-of loops are easier to read and less error-prone, as they
    /// eliminate the need for an index variable and manual array access.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// for (let i = 0; i < arr.length; i++) {
    ///   console.log(arr[i]);
    /// }
    ///
    /// // Good
    /// for (const item of arr) {
    ///   console.log(item);
    /// }
    /// ```
    PreferForOf,
    style
);

trait SpanExt {
    fn contains(&self, other: Self) -> bool;
}

impl SpanExt for Span {
    fn contains(&self, other: Self) -> bool {
        self.start <= other.start && self.end >= other.end
    }
}

trait ExpressionExt {
    fn is_increment_of(&self, var_name: &Atom) -> bool;
}

impl<'a> ExpressionExt for Expression<'a> {
    fn is_increment_of(&self, var_name: &Atom) -> bool {
        match self {
            Expression::UpdateExpression(expr) => match (&expr.argument, &expr.operator) {
                (
                    SimpleAssignmentTarget::AssignmentTargetIdentifier(id),
                    UpdateOperator::Increment,
                ) => id.name == var_name,
                _ => false,
            },
            Expression::AssignmentExpression(expr) => {
                if !matches!(&expr.left,
                    AssignmentTarget::SimpleAssignmentTarget(
                        SimpleAssignmentTarget::AssignmentTargetIdentifier(id)
                    )
                    if id.name == var_name
                ) {
                    return false;
                }

                match expr.operator {
                    AssignmentOperator::Addition => {
                        matches!(&expr.right, Expression::NumericLiteral(lit)
                            if (lit.value - 1f64).abs() < f64::EPSILON)
                    }
                    AssignmentOperator::Assign => {
                        let Expression::BinaryExpression(bin_expr) = &expr.right else {
                            return false;
                        };

                        if bin_expr.operator != BinaryOperator::Addition {
                            return false;
                        }

                        match (&bin_expr.left, &bin_expr.right) {
                            (Expression::Identifier(id), Expression::NumericLiteral(lit))
                            | (Expression::NumericLiteral(lit), Expression::Identifier(id)) => {
                                id.name == var_name && (lit.value - 1f64).abs() < f64::EPSILON
                            }
                            _ => false,
                        }
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

impl Rule for PreferForOf {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ForStatement(for_stmt) = node.kind() else { return };

        let Some(ForStatementInit::VariableDeclaration(for_stmt_init)) = &for_stmt.init else {
            return;
        };

        if for_stmt_init.declarations.len() != 1
            || for_stmt_init.kind == VariableDeclarationKind::Const
        {
            return;
        }

        let decl = &for_stmt_init.declarations[0];
        let (var_name, var_symbol_id) = match &decl.id.kind {
            BindingPatternKind::BindingIdentifier(id) => (&id.name, id.symbol_id.get()),
            _ => return,
        };

        if !matches!(&decl.init,
            Some(Expression::NumericLiteral(literal)) if literal.value == 0f64
        ) {
            return;
        }

        let Some(Expression::BinaryExpression(test_expr)) = &for_stmt.test else { return };

        if !matches!((&test_expr.left, test_expr.operator),
            (Expression::Identifier(id), BinaryOperator::LessThan) if id.name == var_name
        ) {
            return;
        }

        let array_name = {
            let Expression::MemberExpression(mem_expr) = &test_expr.right else { return };
            if !matches!(mem_expr.static_property_name(), Some(prop_name) if prop_name == "length")
            {
                return;
            }

            match &mem_expr.object() {
                Expression::Identifier(id) => id.name.as_str(),
                Expression::MemberExpression(mem_expr) => match mem_expr.static_property_name() {
                    Some(array_name) => array_name,
                    None => return,
                },
                _ => return,
            }
        };

        let Some(update_expr) = &for_stmt.update else { return };
        if !update_expr.is_increment_of(var_name) {
            return;
        }

        let nodes = ctx.nodes();
        let body_span = for_stmt.body.span();

        let Some(var_symbol_id) = var_symbol_id else { return };
        if ctx.semantic().symbol_references(var_symbol_id).any(|reference| {
            let ref_id = reference.node_id();

            let symbol_span = nodes.get_node(ref_id).kind().span();
            if !body_span.contains(symbol_span) {
                return false;
            }

            let Some(ref_parent) = nodes.parent_node(ref_id) else { return true };

            if let Some(ref_grand_parent) = nodes.parent_node(ref_parent.id()) {
                match ref_grand_parent.kind() {
                    AstKind::SimpleAssignmentTarget(_) => {
                        return true;
                    }
                    AstKind::UnaryExpression(unary_expr)
                        if unary_expr.operator == UnaryOperator::Delete =>
                    {
                        return true;
                    }
                    _ => {}
                }
            }

            let parent_kind = ref_parent.kind();
            let AstKind::MemberExpression(mem_expr) = parent_kind else { return true };
            match mem_expr.object() {
                Expression::Identifier(id) => id.name.as_str() != array_name,
                Expression::MemberExpression(mem_expr) => match mem_expr.static_property_name() {
                    Some(prop_name) => prop_name != array_name,
                    None => true,
                },
                _ => true,
            }
        }) {
            return;
        }

        let span = for_stmt_init.span.merge(&test_expr.span).merge(&update_expr.span());
        ctx.diagnostic(PreferForOfDiagnostic(span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "for (let i = 0; i < arr1.length; i++) { const x = arr1[i] === arr2[i]; }",
        "for (let i = 0; i < arr.length; i++) { arr[i] = 0; }",
        "for (var c = 0; c < arr.length; c++) { doMath(c); }",
        "for (var d = 0; d < arr.length; d++) doMath(d);",
        "for (var e = 0; e < arr.length; e++) { if (e > 5) { doMath(e); } console.log(arr[e]); }",
        "for (var f = 0; f <= 40; f++) { doMath(f); }",
        "for (var g = 0; g <= 40; g++) doMath(g);",
        "for (var h = 0, len = arr.length; h < len; h++) {}",
        "for (var i = 0, len = arr.length; i < len; i++) arr[i];",
        "var m = 0; for (;;) { if (m > 3) break; console.log(m); m++; }",
        "var n = 0; for (; n < 9; n++) { console.log(n); }",
        "var o = 0; for (; o < arr.length; o++) { console.log(arr[o]); }",
        "for (; x < arr.length; x++) {}",
        "for (let x = 0; ; x++) {}",
        "for (let x = 0; x < arr.length; ) {}",
        "for (let x = 0; NOTX < arr.length; x++) {}",
        "for (let x = 0; x < arr.length; NOTX++) {}",
        "for (let NOTX = 0; x < arr.length; x++) {}",
        "for (let x = 0; x < arr.length; x--) {}",
        "for (let x = 0; x <= arr.length; x++) {}",
        "for (let x = 1; x < arr.length; x++) {}",
        "for (let x = 0; x < arr.length(); x++) {}",
        "for (let x = 0; x < arr.length; x += 11) {}",
        "for (let x = arr.length; x > 1; x -= 1) {}",
        "for (let x = 0; x < arr.length; x *= 2) {}",
        "for (let x = 0; x < arr.length; x = x + 11) {}",
        "for (let x = 0; x < arr.length; x++) { x++; }",
        "for (let x = 0; true; x++) {}",
        "for (var q in obj) { if (obj.hasOwnProperty(q)) { console.log(q); } }",
        "for (var r of arr) { console.log(r); }",
        "for (let x = 0; x < arr.length; x++) { let y = arr[x + 1]; }",
        "for (let i = 0; i < arr.length; i++) { delete arr[i]; }",
        "for (let i = 0; i < arr.length; i++) { [arr[i]] = [1]; }",
        "for (let i = 0; i < arr.length; i++) { [...arr[i]] = [1]; }",
        "for (let i = 0; i < arr1?.length; i++) { const x = arr1[i] === arr2[i]; }",
        "for (let i = 0; i < arr?.length; i++) { arr[i] = 0; }",
        "for (var c = 0; c < arr?.length; c++) { doMath(c); }",
        "for (var d = 0; d < arr?.length; d++) doMath(d);",
        "for (var c = 0; c < arr.length; c++) { doMath?.(c); }",
        "for (var d = 0; d < arr.length; d++) doMath?.(d);",
        "for (let i = 0; i < test.length; ++i) { this[i]; }",
        "for (let i = 0; i < arr.length; i++) { ({ foo: arr[i] } = { foo: 1 }); }",
        "for (let i = 0; i < arr.length; i++) { arr[i]++; }",
        "function* gen() { for (let i = 0; i < this.length; ++i) { yield this[i]; } }",
    ];

    let fail = vec![
        "for (var a = 0; a < obj.arr.length; a++) { console.log(obj.arr[a]); }",
        "for (var b = 0; b < arr.length; b++) console.log(arr[b]);",
        "for (let a = 0; a < arr.length; a++) { console.log(arr[a]); }",
        "for (var b = 0; b < arr.length; b++) console?.log(arr[b]);",
        "for (let a = 0; a < arr.length; a++) { console?.log(arr[a]); }",
        "for (let a = 0; a < arr.length; ++a) { arr[a].whatever(); }",
        "for (let x = 0; x < arr.length; x++) {}",
        "for (let x = 0; x < arr.length; x += 1) {}",
        "for (let x = 0; x < arr.length; x = x + 1) {}",
        "for (let x = 0; x < arr.length; x = 1 + x) {}",
        "for (let shadow = 0; shadow < arr.length; shadow++) {
            for (let shadow = 0; shadow < arr.length; shadow++) {}
        }",
        "for (let i = 0; i < arr.length; i++) { obj[arr[i]] = 1; }",
        "for (let i = 0; i < arr.length; i++) { delete obj[arr[i]]; }",
        "for (let i = 0; i < arr.length; i++) { [obj[arr[i]]] = [1]; }",
        "for (let i = 0; i < arr.length; i++) { [...obj[arr[i]]] = [1]; }",
        "for (let i = 0; i < arr.length; i++) { ({ foo: obj[arr[i]] } = { foo: 1 }); }",
        "for (let i = 0; i < this.item.length; ++i) { this.item[i]; }",
        "function* gen() { for (let i = 0; i < this.array.length; ++i) { yield this.array[i]; } }",
    ];

    Tester::new(PreferForOf::NAME, pass, fail).test_and_snapshot();
}
