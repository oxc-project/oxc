use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::UnaryOperator;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct NoDynamicDelete;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow using the delete operator on computed key expressions.
    ///
    /// ### Why is this bad?
    /// Deleting dynamically computed keys can be dangerous and in some cases not well optimized.
    /// Using the delete operator on keys that aren't runtime constants could be a sign that you're using the wrong data structures.
    /// Consider using a Map or Set if you’re using an object as a key-value collection.
    ///
    /// ### Example
    /// ```ts
    /// const container: { [i: string]: 0 } = {};
    /// delete container['aa' + 'b'];
    /// ```
    NoDynamicDelete,
    typescript,
    restriction,
);

fn no_dynamic_delete_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not delete dynamically computed property keys.").with_label(span)
}

impl Rule for NoDynamicDelete {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::UnaryExpression(expr) = node.kind() else { return };
        if !matches!(expr.operator, UnaryOperator::Delete) {
            return;
        }

        let Expression::ComputedMemberExpression(computed_expr) = &expr.argument else { return };
        let inner_expression = computed_expr.expression.get_inner_expression();
        if inner_expression.is_string_literal() || inner_expression.is_number_literal() {
            return;
        }

        if let Expression::UnaryExpression(unary_expr) = &inner_expression {
            if unary_expr.operator == UnaryOperator::UnaryNegation
                && unary_expr.argument.is_number_literal()
            {
                return;
            }
        }
        ctx.diagnostic(no_dynamic_delete_diagnostic(expr.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "
        	const container: { [i: string]: 0 } = {};
        	delete container.aaa;
        	    ",
        "
        	const container: { [i: string]: 0 } = {};
        	delete container.delete;
        	    ",
        "
        	const container: { [i: string]: 0 } = {};
        	delete container[7];
        	    ",
        "
        	const container: { [i: string]: 0 } = {};
        	delete container[-7];
        	    ",
        "
        	const container: { [i: string]: 0 } = {};
        	delete container['-Infinity'];
        	    ",
        "
        	const container: { [i: string]: 0 } = {};
        	delete container['+Infinity'];
        	    ",
        "
        	const value = 1;
        	delete value;
        	    ",
        "
        	const value = 1;
        	delete -value;
        	    ",
        "
        	const container: { [i: string]: 0 } = {};
        	delete container['aaa'];
        	    ",
        "
        	const container: { [i: string]: 0 } = {};
        	delete container['delete'];
        	    ",
        "
        	const container: { [i: string]: 0 } = {};
        	delete container['NaN'];
        	    ",
        "
        	const container = {};
        	delete container[('aaa')]
        	    ",
    ];

    let fail = vec![
        "
        	const container: { [i: string]: 0 } = {};
        	delete container['aa' + 'b'];
        	      ",
        "
        	const container: { [i: string]: 0 } = {};
        	delete container[+7];
        	      ",
        "
        	const container: { [i: string]: 0 } = {};
        	delete container[-Infinity];
        	      ",
        "
        	const container: { [i: string]: 0 } = {};
        	delete container[+Infinity];
        	      ",
        "
        	const container: { [i: string]: 0 } = {};
        	delete container[NaN];
        	      ",
        "
        	const container: { [i: string]: 0 } = {};
        	const name = 'name';
        	delete container[name];
        	      ",
        "
        	const container: { [i: string]: 0 } = {};
        	const getName = () => 'aaa';
        	delete container[getName()];
        	      ",
        "
        	const container: { [i: string]: 0 } = {};
        	const name = { foo: { bar: 'bar' } };
        	delete container[name.foo.bar];
        	      ",
        "
        	const container: { [i: string]: 0 } = {};
        	delete container[+'Infinity'];
        	      ",
        "
        	const container: { [i: string]: 0 } = {};
        	delete container[typeof 1];
        	      ",
    ];

    Tester::new(NoDynamicDelete::NAME, NoDynamicDelete::PLUGIN, pass, fail).test_and_snapshot();
}
