use oxc_ast::{
    AstKind,
    ast::{
        AssignmentTarget, BindingPatternKind, Expression, ForStatementInit, SimpleAssignmentTarget,
        VariableDeclarationKind, match_member_expression,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, UnaryOperator, UpdateOperator};

use crate::{AstNode, context::LintContext, rule::Rule, utils::is_same_expression};

fn prefer_for_of_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Expected a `for-of` loop instead of a `for` loop with this simple iteration.",
    )
    .with_help("Consider using a for-of loop for this simple iteration.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferForOf;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of for-of loop instead of a for loop with a simple iteration.
    ///
    /// ### Why is this bad?
    ///
    /// Using a for loop with a simple iteration over an array can be replaced with a more concise
    /// and readable for-of loop. For-of loops are easier to read and less error-prone, as they
    /// eliminate the need for an index variable and manual array access.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// for (let i = 0; i < arr.length; i++) {
    ///   console.log(arr[i]);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// for (const item of arr) {
    ///   console.log(item);
    /// }
    /// ```
    PreferForOf,
    typescript,
    style,
    pending
);

trait ExpressionExt {
    fn is_increment_of(&self, var_name: &str) -> bool;
}

impl ExpressionExt for Expression<'_> {
    fn is_increment_of(&self, var_name: &str) -> bool {
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
                    AssignmentTarget::AssignmentTargetIdentifier(id)
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
        let AstKind::ForStatement(for_stmt) = node.kind() else {
            return;
        };

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
            BindingPatternKind::BindingIdentifier(id) => (&id.name, id.symbol_id()),
            _ => return,
        };

        if !matches!(&decl.init,
            Some(Expression::NumericLiteral(literal)) if literal.value == 0f64
        ) {
            return;
        }

        let Some(Expression::BinaryExpression(test_expr)) = &for_stmt.test else {
            return;
        };

        if !matches!((&test_expr.left, test_expr.operator),
            (Expression::Identifier(id), BinaryOperator::LessThan) if id.name == var_name
        ) {
            return;
        }

        let (array_name, array_expr) = {
            let Some(mem_expr) = test_expr.right.as_member_expression() else {
                return;
            };
            if !matches!(mem_expr.static_property_name(), Some(prop_name) if prop_name == "length")
            {
                return;
            }

            let array_expr = mem_expr.object();
            let array_name = match mem_expr.object() {
                Expression::Identifier(id) => id.name.as_str(),
                expr @ match_member_expression!(Expression) => {
                    match expr.to_member_expression().static_property_name() {
                        Some(array_name) => array_name,
                        None => return,
                    }
                }
                _ => return,
            };

            (array_name, array_expr)
        };

        let Some(update_expr) = &for_stmt.update else {
            return;
        };
        if !update_expr.is_increment_of(var_name) {
            return;
        }

        let nodes = ctx.nodes();
        let body_span = for_stmt.body.span();

        if ctx.semantic().symbol_references(var_symbol_id).any(|reference| {
            let ref_id = reference.node_id();

            let symbol_span = nodes.get_node(ref_id).kind().span();
            if !body_span.contains_inclusive(symbol_span) {
                return false;
            }

            let parent = nodes.parent_node(ref_id);
            let grand_parent = nodes.parent_node(parent.id());

            // Check for direct uses of the loop variable that prevent for-of conversion
            if prevents_for_of_conversion_direct_usage(grand_parent, parent) {
                return true;
            }

            // Check if arr[i] usage prevents for-of conversion
            if prevents_for_of_array_access(parent, grand_parent, array_name, nodes) {
                return true;
            }

            // Check if this is a non-array access that prevents conversion
            prevents_for_of_non_array_access(parent, array_expr, ctx)
        }) {
            return;
        }

        let span = for_stmt_init.span.merge(test_expr.span).merge(update_expr.span());
        ctx.diagnostic(prefer_for_of_diagnostic(span));
    }
}

/// Check if direct usage of the loop variable prevents for-of conversion
fn prevents_for_of_conversion_direct_usage(grand_parent: &AstNode, parent: &AstNode) -> bool {
    match grand_parent.kind() {
        AstKind::UnaryExpression(unary_expr) if unary_expr.operator == UnaryOperator::Delete => {
            true
        }
        AstKind::UpdateExpression(_) => true,
        // Check if the loop variable itself is being assigned to (like i = something)
        AstKind::AssignmentExpression(assign_expr) => assign_expr.left.span() == parent.span(),
        _ => false,
    }
}

/// Check if array item access prevents for-of conversion
fn prevents_for_of_array_access(
    parent: &AstNode,
    grand_parent: &AstNode,
    array_name: &str,
    nodes: &oxc_semantic::AstNodes,
) -> bool {
    let Some(mem_expr) = parent.kind().as_member_expression_kind() else {
        return false;
    };

    let Expression::Identifier(id) = mem_expr.object() else {
        return false;
    };

    if id.name.as_str() != array_name {
        return false;
    }

    // Check for direct assignment: arr[i] = value
    if let AstKind::AssignmentExpression(assign_expr) = grand_parent.kind()
        && assign_expr.left.span() == parent.span()
    {
        return true;
    }

    // Check if arr[i] is a direct element in destructuring
    if is_direct_assignment_target(&grand_parent.kind()) {
        return true;
    }

    // Check one level deeper for nested destructuring
    let great_grand_parent = nodes.parent_node(grand_parent.id());
    if is_direct_assignment_target(&great_grand_parent.kind()) {
        // Only prevent for-of if grand_parent is NOT a member expression
        // This distinguishes [arr[i]] from [obj[arr[i]]]
        return !grand_parent.kind().is_member_expression_kind();
    }

    false
}

/// Check if this is a non-array access that prevents conversion
fn prevents_for_of_non_array_access(
    parent: &AstNode,
    array_expr: &Expression,
    ctx: &LintContext,
) -> bool {
    let parent_kind = parent.kind();

    if let Some(mem_expr) = parent_kind.as_member_expression_kind() {
        !is_same_expression(mem_expr.object(), array_expr, ctx)
    } else {
        true
    }
}

/// Check if the AST kind represents a direct assignment target
fn is_direct_assignment_target(kind: &AstKind) -> bool {
    matches!(kind, AstKind::ArrayAssignmentTarget(_) | AstKind::ObjectAssignmentTarget(_))
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
        // subsection of eslint-plugin-unicotn test cases
        "for (;;);",
        "for (;;) {}",
        "for (a;; c) { d }",
        "for (a; b;) { d }",
        "for (the; love; of) { god }",
        "for ([a] = b; f(c); d--) { arr[d] }",
        "for (var a = b; c < arr.length; d++) { arr[e] }",
        "for (const x of xs) {}",
        "for (var j = 0; j < 10; j++) {}",
        "for (i = 0; i < arr.length; i++) { el = arr[i]; console.log(i, el); }",
        "for (let i = 0, j = 0; i < arr.length; i++) { const el = arr[i]; console.log(i, el); }",
        "for (let {i} = 0; i < arr.length; i++) { const el = arr[i]; console.log(i, el); }",
        "for (let i = 0; f(i, arr.length); i++) { const el = arr[i]; console.log(i, el); }",
        "for (let i = 0; i < arr.size; i++) { const el = arr[i]; console.log(i, el); }",
        "for (let i = 0; j < arr.length; i++) { const el = arr[i]; console.log(i, el); }",
        "for (let i = 0; i <= arr.length; i++) { const el = arr[i]; console.log(i, el); }",
        "for (let i = 0; arr.length > i;) { let el = arr[i]; console.log(i, el); }",
        "for (let i = 0; arr.length > i; i--) { let el = arr[i]; console.log(i, el); }",
        "for (let i = 0; arr.length > i; f(i)) { let el = arr[i]; console.log(i, el); }",
        "for (let i = 0; arr.length > i; i = f(i)) { let el = arr[i]; console.log(i, el); }",
        "const arr = []; for (let i = 0; arr.length > i; i ++);",
        "const arr = []; for (let i = 0; arr.length > i; i ++) console.log(NaN)",
        "const arr = []; for (let i = 0; i < arr.length; ++i) { const el = f(i); console.log(i, el); }",
        "const arr = []; for (let i = 0; i < arr.length; i++) { console.log(i); }",
        "const input = []; for (let i = 0; i < input.length; i++) { const el = input[i]; i++; console.log(i, el); }",
        "const input = []; for (let i = 0; i < input.length; i++) { const el = input[i]; i = 4; console.log(i, el); }",
        "const arr = []; for (let i = 0; i < arr.length; i++) { arr[i] = i + 2; }",
        "for (;;);",
        "for (;;) {}",
        "for (var j = 0; j < 10; j++) {}",
        "const arr = [];
        for (i = 0; i < arr.length; i++) { el = arr[i]; console.log(i, el); }",
        "for (let x = 0; x < series.data.length; x++) { let newValue = series.data[x].y; for (const otherSeries of subseries) { newValue -= otherSeries.data[x].y; } series.data[x].y = newValue; }",
        // Deep nesting test cases
        "const a = { b: { c: { d: { e: [1, 2, 3] } } } };
         const x = { b: { c: { d: { e: [4, 5, 6] } } } };
         for (let i = 0; i < a.b.c.d.e.length; i++) {
             console.log(x.b.c.d.e[i]); // Different object with same path
         }",
        "const obj1 = { a: { b: [1, 2, 3] } };
         const obj2 = { a: { b: [4, 5, 6] } };
         for (let i = 0; i < obj1.a.b.length; i++) {
             console.log(obj2.a.b[i]); // Different object
         }",
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
        // subsection of eslint-plugin-unicorn test cases
        "const positions = []; for (let i = 0; i < positions.length; i++) { let last: vscode.Position | vscode.Range = positions[i]; }",
        "const arr = []; for (let i = 0; i < arr.length; i += 1) { console.log(arr[i]) }",
        "const plugins = []; for (let i = 0; i < plugins.length; i++) { let plugin = plugins[i]; plugin = calculateSomeNewValue(); }",
        "const array = []; for (let i = 0; i < array.length; i++) { var foo = array[i]; foo = bar(); }",
        "const array = []; for (let i = 0; i < array.length; i++) { let foo = array[i]; }",
        "const array = []; for (let i = 0; i < array.length; i++) { const foo = array[i]; }",
        "const array = []; for (let i = 0; i < array.length; i++) { var foo = array[i], bar = 1; }",
        // Deep nesting test cases that should trigger warning
        "const a = { b: { c: { d: { e: [1, 2, 3] } } } };
         for (let i = 0; i < a.b.c.d.e.length; i++) {
             console.log(a.b.c.d.e[i]); // Same deeply nested array
         }",
        "const obj = { a: { b: [1, 2, 3] } };
         for (let i = 0; i < obj.a.b.length; i++) {
             console.log(obj.a.b[i]); // Same nested array
         }",
    ];

    Tester::new(PreferForOf::NAME, PreferForOf::PLUGIN, pass, fail).test_and_snapshot();
}
