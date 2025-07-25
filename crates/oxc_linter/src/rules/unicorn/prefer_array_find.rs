use oxc_ast::{
    AstKind,
    ast::{
        Argument, AssignmentTarget, BindingPatternKind, CallExpression, Expression,
        SimpleAssignmentTarget, UnaryOperator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode, ast_util::is_method_call, context::LintContext, rule::Rule,
    utils::call_expr_member_expr_property_span,
};

fn prefer_array_find_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer 'find' over filtering and accessing the first result")
        .with_help("Use 'find(predicate)' instead of 'filter(predicate)[0]' or similar patterns")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferArrayFind;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Encourages using `Array.prototype.find` instead of `filter(...)[0]` or
    /// similar patterns when only the first matching element is needed.
    ///
    /// ### Why is this bad?
    ///
    /// Using `filter(...)[0]` to get the first match is less efficient and more verbose
    /// than using `find(...)`. `find` short-circuits when a match is found,
    /// whereas `filter` evaluates the entire array.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const match = users.filter(u => u.id === id)[0];
    /// const match = users.filter(fn).shift();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const match = users.find(u => u.id === id);
    /// const match = users.find(fn);
    /// ```
    PreferArrayFind,
    unicorn,
    perf, // Encourages more efficient use of built-in methods
    pending
);

impl Rule for PreferArrayFind {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Zero index access
        if let AstKind::ComputedMemberExpression(computed_member_expr) = node.kind() {
            if computed_member_expr.expression.is_number_0() {
                if let Expression::CallExpression(call_expr) =
                    computed_member_expr.object.get_inner_expression()
                {
                    if is_filter_call(call_expr) && !is_left_hand_side(node, ctx) {
                        ctx.diagnostic(prefer_array_find_diagnostic(
                            call_expr_member_expr_property_span(call_expr),
                        ));
                    }
                }
            }
        }

        if let AstKind::CallExpression(call_expr) = node.kind() {
            if is_method_call(call_expr, None, Some(&["shift"]), Some(0), Some(0)) {
                if let Some(Expression::CallExpression(filter_call_expr)) = call_expr
                    .callee
                    .get_inner_expression()
                    .as_member_expression()
                    .map(|expression| expression.object().get_inner_expression())
                {
                    if is_filter_call(filter_call_expr) {
                        ctx.diagnostic(prefer_array_find_diagnostic(
                            call_expr_member_expr_property_span(filter_call_expr),
                        ));
                    }
                }
            }
        }

        // `const [foo] = array.filter()`
        if let AstKind::VariableDeclarator(var_decl) = node.kind() {
            if let BindingPatternKind::ArrayPattern(array_pat) = &var_decl.id.kind {
                if array_pat.elements.len() == 1 && array_pat.elements[0].is_some() {
                    if let Some(Expression::CallExpression(array_filter)) = &var_decl.init {
                        if is_filter_call(array_filter) {
                            ctx.diagnostic(prefer_array_find_diagnostic(
                                call_expr_member_expr_property_span(array_filter),
                            ));
                        }
                    }
                }
            }
        }

        // `[foo] = array.filter()`
        if let AstKind::AssignmentExpression(assignment_expr) = node.kind() {
            if let AssignmentTarget::ArrayAssignmentTarget(array_assignment_target) =
                &assignment_expr.left
            {
                if array_assignment_target.elements.len() == 1
                    && array_assignment_target.elements[0].is_some()
                {
                    if let Expression::CallExpression(array_filter) = &assignment_expr.right {
                        if is_filter_call(array_filter) {
                            ctx.diagnostic(prefer_array_find_diagnostic(
                                call_expr_member_expr_property_span(array_filter),
                            ));
                        }
                    }
                }
            }
        }

        // `const foo = array.filter(); foo[0]; [bar] = foo`
        if let AstKind::VariableDeclarator(var_decl) = node.kind() {
            if let Some(Expression::CallExpression(call_expr)) = &var_decl.init {
                if is_filter_call(call_expr)
                    && !matches!(
                        ctx.nodes().ancestor_kinds(node.id()).nth(1),
                        Some(
                            AstKind::ExportDefaultDeclaration(_)
                                | AstKind::ExportNamedDeclaration(_)
                        )
                    )
                {
                    if let Some(ident) = var_decl.id.kind.get_binding_identifier() {
                        let mut zero_index_nodes = Vec::new();
                        let mut destructuring_nodes = Vec::new();

                        let mut is_used_elsewhere = false;

                        for reference in ctx.symbol_references(ident.symbol_id()) {
                            match ctx.nodes().parent_kind(reference.node_id()) {
                                AstKind::ComputedMemberExpression(c)
                                    if c.expression.is_number_0() =>
                                {
                                    zero_index_nodes.push(reference);
                                }
                                AstKind::VariableDeclarator(var_declarator) => {
                                    if let BindingPatternKind::ArrayPattern(array_pat) =
                                        &var_declarator.id.kind
                                    {
                                        if array_pat.elements.len() == 1
                                            && array_pat.elements[0].is_some()
                                        {
                                            destructuring_nodes.push(reference);
                                        }
                                    }
                                }
                                AstKind::AssignmentExpression(assignment_expr) => {
                                    // Check for array destructuring: [foo] = items
                                    if let AssignmentTarget::ArrayAssignmentTarget(target) =
                                        &assignment_expr.left
                                    {
                                        if target.elements.len() == 1
                                            && target.elements[0].is_some()
                                        {
                                            destructuring_nodes.push(reference);
                                        }
                                    } else if let Some(
                                        SimpleAssignmentTarget::AssignmentTargetIdentifier(ident),
                                    ) = assignment_expr.left.as_simple_assignment_target()
                                    {
                                        // Check for simple reassignment: items = something
                                        if ident.span
                                            == ctx.nodes().get_node(reference.node_id()).span()
                                        {
                                            is_used_elsewhere = true; // Variable is being reassigned
                                        }
                                    }
                                }
                                _ => is_used_elsewhere = true,
                            }
                        }

                        if !is_used_elsewhere
                            && (!zero_index_nodes.is_empty() || !destructuring_nodes.is_empty())
                        {
                            ctx.diagnostic(prefer_array_find_diagnostic(
                                call_expr_member_expr_property_span(call_expr),
                            ));
                        }
                    }
                }
            }
        }

        // `array.filter().at(0)`
        // `array.filter().at(-1)`
        if let AstKind::CallExpression(at_call_expr) = node.kind() {
            if is_method_call(at_call_expr, None, Some(&["at"]), Some(1), Some(1))
                && at_call_expr.arguments.first().is_some_and(|arg| {
                    arg.as_expression().is_some_and(|x| match x {
                        Expression::NumericLiteral(_) if x.is_number_value(0.0) => true,
                        Expression::UnaryExpression(u)
                            if u.operator == UnaryOperator::UnaryNegation =>
                        {
                            u.argument.is_number_value(1.0)
                        }
                        _ => false,
                    })
                })
            {
                if let Some(Expression::CallExpression(filter_call_expr)) = at_call_expr
                    .callee
                    .get_inner_expression()
                    .as_member_expression()
                    .map(|expression| expression.object().get_inner_expression())
                {
                    if is_filter_call(filter_call_expr) {
                        ctx.diagnostic(prefer_array_find_diagnostic(
                            call_expr_member_expr_property_span(filter_call_expr),
                        ));
                    }
                }
            }
        }

        // `array.filter().pop()`
        if let AstKind::CallExpression(pop_call_expr) = node.kind() {
            if is_method_call(pop_call_expr, None, Some(&["pop"]), Some(0), Some(0)) {
                if let Some(Expression::CallExpression(filter_call_expr)) = pop_call_expr
                    .callee
                    .get_inner_expression()
                    .as_member_expression()
                    .map(|expression| expression.object().get_inner_expression())
                {
                    if is_filter_call(filter_call_expr) {
                        ctx.diagnostic(prefer_array_find_diagnostic(
                            call_expr_member_expr_property_span(filter_call_expr),
                        ));
                    }
                }
            }
        }
    }
}

fn is_filter_call(call_expr: &CallExpression) -> bool {
    is_method_call(call_expr, None, Some(&["filter"]), Some(1), Some(2))
        && call_expr.arguments.first().is_some_and(|arg| !matches!(arg, Argument::SpreadElement(_)))
}

fn is_left_hand_side<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    match ctx.nodes().parent_kind(node.id()) {
        AstKind::AssignmentExpression(expr) => expr.left.span() == node.span(),
        AstKind::AssignmentPattern(expr) => expr.left.span() == node.span(),
        AstKind::UpdateExpression(expr) => expr.argument.span() == node.span(),
        AstKind::UnaryExpression(expr) => expr.operator == UnaryOperator::Delete,
        AstKind::ArrayAssignmentTarget(_)
        | AstKind::ObjectAssignmentTarget(_)
        | AstKind::AssignmentTargetWithDefault(_)
        | AstKind::ArrayPattern(_)
        | AstKind::IdentifierReference(_) => true,
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "array.find(foo)",
        "array.filter(foo)",
        "array.filter(foo)[+0]",
        "array.filter(foo)[-0]",
        "array.filter(foo)[1-1]",
        r#"array.filter(foo)["0"]"#,
        "array.filter(foo).first",
        "array.filter[0]",
        "filter(foo)[0]",
        "array[filter](foo)[0]",
        "array.notFilter(foo)[0]",
        "array.filter()[0]",
        "array.filter(foo, thisArgument, extraArgument)[0]",
        "array.filter(...foo)[0]",
        "array.filter(foo)[0] += 1",
        "++ array.filter(foo)[0]",
        "array.filter(foo)[0]--",
        "delete array.filter(foo)[0]",
        "[array.filter(foo)[0] = 1] = []",
        "array.filter(foo).shift",
        "shift(array.filter(foo))",
        "array.filter(foo)[shift]()",
        "array.filter(foo).notShift()",
        "array.filter(foo).shift(extraArgument)",
        "array.filter(foo).shift(...[])",
        "array.filter.shift()",
        "filter(foo).shift()",
        "array[filter](foo).shift()",
        "array.notFilter(foo).shift()",
        "array.filter().shift()",
        "array.filter(foo, thisArgument, extraArgument).shift()",
        "array.filter(...foo).shift()",
        "function a([foo] = array.filter(bar1)) {}",
        "const foo = array.filter(bar)",
        "const items = array.filter(bar)",
        "const {0: foo} = array.filter(bar)",
        "const [] = array.filter(bar)",
        "const [foo, another] = array.filter(bar)",
        "const [, foo] = array.filter(bar)",
        "const [,] = array.filter(bar)",
        "const [...foo] = array.filter(bar)",
        "const [foo] = array.filter",
        "const [foo] = filter(bar)",
        "const [foo] = array[filter](bar)",
        "const [foo] = array.notFilter(bar)",
        "const [foo] = array.filter()",
        "const [foo] = array.filter(bar, thisArgument, extraArgument)",
        "const [foo] = array.filter(...bar)",
        "function a([foo] = array.filter(bar)) {}",
        "foo = array.filter(bar)",
        "items = array.filter(bar)",
        "({foo} = array.filter(bar))",
        "[] = array.filter(bar)",
        "[foo, another] = array.filter(bar)",
        "[, foo] = array.filter(bar)",
        "[,] = array.filter(bar)",
        "[...foo] = array.filter(bar)",
        "[foo] = array.filter",
        "[foo] = filter(bar)",
        "[foo] = array[filter](bar)",
        "[foo] = array.notFilter(bar)",
        "[foo] = array.filter()",
        "[foo] = array.filter(bar, thisArgument, extraArgument)",
        "[foo] = array.filter(...bar)",
        "const foo = array.find(bar), first = foo[0];",
        "const foo = array.filter(bar), first = notFoo[0];",
        "const foo = array.filter(bar), first = foo[+0];",
        "const foo2 = array.filter(bar); first = foo;",
        "const foo = array.filter(bar), first = a[foo][0];",
        "const foo = array.filter(bar), first = foo[-0];",
        "const foo = array.filter(bar), first = foo[1-1];",
        r#"const foo = array.filter(bar), first = foo["0"];"#,
        "const foo = array.filter(bar), first = foo.first;",
        "foo = array.filter(bar); const first = foo[+0];",
        "const {foo} = array.filter(bar), first = foo[0];",
        "const foo = array.filter(bar);
			doSomething(foo);
			const first = foo[0];",
        // "var foo = array.filter(bar);
        // 	var foo = array.filter(bar);
        // 	const first = foo[0];",
        "export const foo = array.filter(bar);
			const first = foo[0];",
        "const foo = array.find(bar); const [first] = foo;",
        "const foo = array.find(bar); [first] = foo;",
        "const foo = array.filter(bar); const [first] = notFoo;",
        "const foo = array.filter(bar); [first] = notFoo;",
        "const foo = array.filter(bar); const first = foo;",
        "const foo = array.filter(bar); first = foo;",
        "const foo = array.filter(bar); const {0: first} = foo;",
        "const foo = array.filter(bar); ({0: first} = foo);",
        "const foo = array.filter(bar); const [] = foo;",
        "const foo = array.filter(bar); const [first, another] = foo;",
        "const foo = array.filter(bar); [first, another] = foo;",
        "const foo = array.filter(bar); const [,first] = foo;",
        "const foo = array.filter(bar); [,first] = foo;",
        "const foo = array.filter(bar); const [,] = foo;",
        "const foo = array.filter(bar); [,] = foo;",
        "const foo = array.filter(bar); const [...first] = foo;",
        "const foo = array.filter(bar); [...first] = foo;",
        "const foo = array.filter(bar);
			function a([bar] = foo) {}",
        "const foo = array.filter; const first = foo[0]",
        "const foo = filter(bar); const first = foo[0]",
        "const foo = array[filter](bar); const first = foo[0]",
        "const foo = array.notFilter(bar); const first = foo[0]",
        "const foo = array.filter(); const first = foo[0]",
        "const foo = array.filter(bar, thisArgument, extraArgument); const first = foo[0]",
        "const foo = array.filter(...bar); const first = foo[0]",
        "const item = array.find(bar), first = item;",
        "let items = array.filter(bar); console.log(items[0]); items = [1,2,3]; console.log(items[0]);",
        "array.filter(foo).pop",
        "pop(array.filter(foo))",
        "array.filter(foo)[pop]()",
        "array.filter(foo).notPop()",
        "array.filter(foo).pop(extraArgument)",
        "array.filter(foo).pop(...[])",
        "array.filter.pop()",
        "filter(foo).pop()",
        "array[filter](foo).pop()",
        "array.notFilter(foo).pop()",
        "array.filter().pop()",
        "array.filter(foo, thisArgument, extraArgument).pop()",
        "array.filter(...foo).pop()",
        "array.filter(foo).at",
        "at(array.filter(foo), -1)",
        "array.filter(foo)[at](-1)",
        "array.filter(foo).notAt(-1)",
        "array.filter(foo).at()",
        "array.filter(foo).at(-1, extraArgument)",
        "array.filter(foo).at(...[-1])",
        "array.filter(foo).at(1)",
        "array.filter(foo).at(+1)",
        "const ONE = 1; array.filter(foo).at(-ONE)",
        "const MINUS_ONE = -1; array.filter(foo).at(MINUS_ONE)",
        "const a = {b: 1}; array.filter(foo).at(-a.b)",
        "const a = {b: -1}; array.filter(foo).at(a.b)",
        "array.filter(foo).at(-2)",
        "array.filter(foo).at(-(-1))",
        //"array.filter(foo).at(-1.)",
        //"array.filter(foo).at(-0b1)",
        r#"array.filter(foo).at(-"1")"#,
        "array.filter(foo).at(-null)",
        "array.filter(foo).at(-false)",
        "array.filter(foo).at(-true)",
        "array.filter.at(-1)",
        "filter(foo).at(-1)",
        "array[filter](foo).at(-1)",
        "array.notFilter(foo).at(-1)",
        "array.filter().at(-1)",
        "array.filter(foo, thisArgument, extraArgument).at(-1)",
        "array.filter(...foo).at(-1)",
        "array2.filter(foo).at",
        "at(array.filter(foo), 0)",
        "array.filter(foo)[at](0)",
        "array.filter(foo).notAt(0)",
        "array2.filter(foo).at()",
        "array.filter(foo).at(0, extraArgument)",
        "array.filter(foo).at(...[0])",
        "array.filter(foo).at(100)",
        "array.filter(foo).at(+0)",
        "const ZERO = 0; array.filter(foo).at(ZERO)",
        "const a = {b: 0}; array.filter(foo).at(a.b)",
        //"array.filter(foo).at(0b0)",
        r#"array.filter(foo).at("0")"#,
        "array.filter.at(0)",
        "filter(foo).at(0)",
        "array[filter](foo).at(0)",
        "array.notFilter(foo).at(0)",
        "array.filter().at(0)",
        "array.filter(foo, thisArgument, extraArgument).at(0)",
        "array.filter(...foo).at(0)",
        // oxc-project/oxc#12399
        "{a.pop!()}",
    ];

    let fail = vec![
        "array.filter(foo)[0]",
        "array.filter(foo, thisArgument)[0]",
        "array.filter(foo).shift()",
        "array.filter(foo, thisArgument).shift()",
        "const item = array
				// comment 1
				.filter(
					// comment 2
					x => x === '🦄'
				)
				// comment 3
				.shift()
				// comment 4
				;",
        "const [foo] = array.filter(bar)",
        "const [items] = array.filter(bar)",
        "const [foo] = array.filter(bar, thisArgument)",
        "const [{foo}] = array.filter(fn);",
        "const [{foo = bar}] = array.filter(fn);",
        "const [[foo]] = array.filter(fn);",
        "const [[foo = bar]] = array.filter(fn);",
        "const [foo, ] = array.filter(bar)",
        "var [foo, ] = array.filter(bar)",
        "let [foo, ] = array.filter(bar)",
        "let a = 1, [foo, ] = array.filter(bar)",
        "let a = 1, [{foo}] = array.filter(bar)",
        "for (let [i] = array.filter(bar); i< 10; i++) {}",
        "const [
				// comment 1
				item
				]
				// comment 2
				= array
				// comment 3
				.filter(
					// comment 4
					x => x === '🦄'
				)
				// comment 5
				;",
        "const [foo = baz] = array.filter(bar)",
        "const [foo = (bar)] = array.filter(bar)",
        "const [foo = a ? b : c] = array.filter(bar)",
        "const [foo = a ?? b] = array.filter(bar)",
        "const [foo = a || b] = array.filter(bar)",
        "const [foo = a && b] = array.filter(bar)",
        "[foo] = array.filter(bar)",
        "[foo] = array.filter(bar, thisArgument)",
        "[foo.bar().baz] = array.filter(fn)",
        "[{foo}] = array.filter(fn);",
        "[[foo]] = array.filter(fn);",
        "[{foo = baz}] = array.filter(fn);",
        "[foo, ] = array.filter(bar)",
        "for ([i] = array.filter(bar); i< 10; i++) {}",
        "let foo
			const bar = []
			;[foo] = array.filter(bar)",
        "[foo = baz] = array.filter(bar)",
        "[{foo} = baz] = array.filter(bar)",
        ";([{foo} = baz] = array.filter(bar))",
        "[foo = (bar)] = array.filter(bar)",
        "[foo = a ? b : c] = array.filter(bar)",
        "[foo = a || b] = array.filter(bar)",
        // "const foo = array.filter(bar); const first = foo[0];",
        // "const foo = array.filter(bar), first = foo[0];",
        // "var foo = array.filter(bar), first = foo[0];",
        // "let foo = array.filter(bar), first = foo[0];",
        "const foo = array.filter(bar); const [first] = foo;",
        "const foo = array.filter(bar); [first] = foo;",
        "const foo = array.filter(bar); const [{propOfFirst = unicorn}] = foo;",
        "const foo = array.filter(bar); [{propOfFirst = unicorn}] = foo;",
        "const items = array.filter(bar);
			const first = items[0];
			console.log(items[0]);
			function foo() { return items[0]; }",
        "const item = {}; const items = array.filter(bar); console.log(items[0]);",
        "let items = array.filter(bar); console.log(items[0]);",
        "const item = 1;
			function f() {
				const items = array.filter(bar);
				console.log(items[0]);
			}",
        "const items = array.filter(bar);
			function f() {
				const item = 1;
				const item_ = 2;
				console.log(items[0]);
			}",
        "const items = array.filter(bar);
			function f() {
				console.log(items[0], item);
			}",
        "const items = array.filter(bar);
			console.log(items[0]);
			function f(item) {
				return item;
			}",
        "function f() {
				const items = array.filter(bar);
				console.log(items[0]);
			}
			function f2(item) {
				return item;
			}",
        "const packages = array.filter(bar);
			console.log(packages[0]);",
        "const symbols = array.filter(bar);
			console.log(symbols[0]);",
        "const foo = array.filter(bar); const [first = bar] = foo;",
        "const foo = array.filter(bar); [first = bar] = foo;",
        "let foo = array.filter(bar);foo[0](foo[0])[foo[0]];",
        "let baz;
			const foo = array.filter(bar);
			const [bar] = foo;
			[{bar}] = foo;
			function getValueOfFirst() {
				return foo[0].value;
			}
			function getPropertyOfFirst(property) {
				return foo[0][property];
			}",
        "const quz = array.filter(fn);
			const [foo] = array.filter(quz[0]);
			[{bar: baz}] = foo[
				array.filter(fn)[0]
			].filter(
				array.filter(fn).shift()
			);",
        "const quz = array.find(fn);
			const [foo] = array.filter(quz);
			({bar: baz} = foo[
				array.filter(fn)[0]
			].find(
				array.filter(fn).shift()
			));",
        "array.filter(foo).pop()",
        "array.filter(foo, thisArgument).pop()",
        "const item = array
				// comment 1
				.filter(
					// comment 2
					x => x === '🦄'
				)
				// comment 3
				.pop()
				// comment 4
				;",
        "array.filter(foo).at(-1)",
        "array.filter(foo, thisArgument).at(-1)",
        "const item = array
				// comment 1
				.filter(
					// comment 2
					x => x === '🦄'
				)
				// comment 3
				.at(
					// comment 4
					-1
					// comment 5
				)
				// comment 6
				;",
        "array.filter(foo).at(0)",
        "array.filter(foo, thisArgument).at(0)",
        "const item = array
				// comment 1
				.filter(
					// comment 2
					x => x === '🦄'
				)
				// comment 3
				.at(
					// comment 4
					0
					// comment 5
				)
				// comment 6
				;",
        // oxc-project/oxc#12399
        "array.filter(foo).pop!()",
        "array.filter(foo)?.pop()",
    ];

    let _fix: Vec<(&'static str, &'static str, Option<serde_json::Value>)> = vec![
        ("array.filter(foo)[0]", "array.find(foo)", None),
        ("array.filter(foo, thisArgument)[0]", "array.find(foo, thisArgument)", None),
        ("array.filter(foo).shift()", "array.find(foo)", None),
        ("array.filter(foo, thisArgument).shift()", "array.find(foo, thisArgument)", None),
        (
            "const item = array
				// comment 1
				.filter(
					// comment 2
					x => x === '🦄'
				)
				// comment 3
				.shift()
				// comment 4
				;",
            "const item = array
				// comment 1
				.find(
					// comment 2
					x => x === '🦄'
				)
				// comment 4
				;",
            None,
        ),
        ("const [foo] = array.filter(bar)", "const foo = array.find(bar)", None),
        ("const [items] = array.filter(bar)", "const items = array.find(bar)", None),
        (
            "const [foo] = array.filter(bar, thisArgument)",
            "const foo = array.find(bar, thisArgument)",
            None,
        ),
        ("const [{foo}] = array.filter(fn);", "const {foo} = array.find(fn);", None),
        ("const [{foo = bar}] = array.filter(fn);", "const {foo = bar} = array.find(fn);", None),
        ("const [[foo]] = array.filter(fn);", "const [foo] = array.find(fn);", None),
        ("const [[foo = bar]] = array.filter(fn);", "const [foo = bar] = array.find(fn);", None),
        ("const [foo, ] = array.filter(bar)", "const foo = array.find(bar)", None),
        ("var [foo, ] = array.filter(bar)", "var foo = array.find(bar)", None),
        ("let [foo, ] = array.filter(bar)", "let foo = array.find(bar)", None),
        ("let a = 1, [foo, ] = array.filter(bar)", "let a = 1, foo = array.find(bar)", None),
        ("let a = 1, [{foo}] = array.filter(bar)", "let a = 1, {foo} = array.find(bar)", None),
        (
            "for (let [i] = array.filter(bar); i< 10; i++) {}",
            "for (let i = array.find(bar); i< 10; i++) {}",
            None,
        ),
        (
            "const [
				// comment 1
				item
				]
				// comment 2
				= array
				// comment 3
				.filter(
					// comment 4
					x => x === '🦄'
				)
				// comment 5
				;",
            "const item
				// comment 2
				= array
				// comment 3
				.find(
					// comment 4
					x => x === '🦄'
				)
				// comment 5
				;",
            None,
        ),
        ("[foo] = array.filter(bar)", "foo = array.find(bar)", None),
        ("[foo] = array.filter(bar, thisArgument)", "foo = array.find(bar, thisArgument)", None),
        ("[foo.bar().baz] = array.filter(fn)", "foo.bar().baz = array.find(fn)", None),
        ("[{foo}] = array.filter(fn);", "({foo} = array.find(fn));", None),
        ("[[foo]] = array.filter(fn);", "[foo] = array.find(fn);", None),
        ("[{foo = baz}] = array.filter(fn);", "({foo = baz} = array.find(fn));", None),
        ("[foo, ] = array.filter(bar)", "foo = array.find(bar)", None),
        (
            "for ([i] = array.filter(bar); i< 10; i++) {}",
            "for (i = array.find(bar); i< 10; i++) {}",
            None,
        ),
        (
            "let foo
			const bar = []
			;[foo] = array.filter(bar)",
            "let foo
			const bar = []
			;foo = array.find(bar)",
            None,
        ),
        (
            "const foo = array.filter(bar); const first = foo[0];",
            "const foo = array.find(bar); const first = foo;",
            None,
        ),
        (
            "const foo = array.filter(bar), first = foo[0];",
            "const foo = array.find(bar), first = foo;",
            None,
        ),
        (
            "var foo = array.filter(bar), first = foo[0];",
            "var foo = array.find(bar), first = foo;",
            None,
        ),
        (
            "let foo = array.filter(bar), first = foo[0];",
            "let foo = array.find(bar), first = foo;",
            None,
        ),
        (
            "const foo = array.filter(bar); const [first] = foo;",
            "const foo = array.find(bar); const first = foo;",
            None,
        ),
        (
            "const foo = array.filter(bar); [first] = foo;",
            "const foo = array.find(bar); first = foo;",
            None,
        ),
        (
            "const foo = array.filter(bar); const [{propOfFirst = unicorn}] = foo;",
            "const foo = array.find(bar); const {propOfFirst = unicorn} = foo;",
            None,
        ),
        (
            "const foo = array.filter(bar); [{propOfFirst = unicorn}] = foo;",
            "const foo = array.find(bar); ({propOfFirst = unicorn} = foo);",
            None,
        ),
        (
            "const items = array.filter(bar);
			const first = items[0];
			console.log(items[0]);
			function foo() { return items[0]; }",
            "const item = array.find(bar);
			const first = item;
			console.log(item);
			function foo() { return item; }",
            None,
        ),
        (
            "const item = {}; const items = array.filter(bar); console.log(items[0]);",
            "const item = {}; const item_ = array.find(bar); console.log(item_);",
            None,
        ),
        (
            "let items = array.filter(bar); console.log(items[0]);",
            "let item = array.find(bar); console.log(item);",
            None,
        ),
        (
            "const item = 1;
			function f() {
				const items = array.filter(bar);
				console.log(items[0]);
			}",
            "const item = 1;
			function f() {
				const item_ = array.find(bar);
				console.log(item_);
			}",
            None,
        ),
        (
            "const items = array.filter(bar);
			function f() {
				const item = 1;
				const item_ = 2;
				console.log(items[0]);
			}",
            "const item__ = array.find(bar);
			function f() {
				const item = 1;
				const item_ = 2;
				console.log(item__);
			}",
            None,
        ),
        (
            "const items = array.filter(bar);
			function f() {
				console.log(items[0], item);
			}",
            "const item_ = array.find(bar);
			function f() {
				console.log(item_, item);
			}",
            None,
        ),
        (
            "const items = array.filter(bar);
			console.log(items[0]);
			function f(item) {
				return item;
			}",
            "const item_ = array.find(bar);
			console.log(item_);
			function f(item) {
				return item;
			}",
            None,
        ),
        (
            "function f() {
				const items = array.filter(bar);
				console.log(items[0]);
			}
			function f2(item) {
				return item;
			}",
            "function f() {
				const item = array.find(bar);
				console.log(item);
			}
			function f2(item) {
				return item;
			}",
            None,
        ),
        (
            "const packages = array.filter(bar);
			console.log(packages[0]);",
            "const package_ = array.find(bar);
			console.log(package_);",
            None,
        ),
        (
            "const symbols = array.filter(bar);
			console.log(symbols[0]);",
            "const symbol_ = array.find(bar);
			console.log(symbol_);",
            None,
        ),
        (
            "let foo = array.filter(bar);foo[0](foo[0])[foo[0]];",
            "let foo = array.find(bar);foo(foo)[foo];",
            None,
        ),
        (
            "let baz;
			const foo = array.filter(bar);
			const [bar] = foo;
			[{bar}] = foo;
			function getValueOfFirst() {
				return foo[0].value;
			}
			function getPropertyOfFirst(property) {
				return foo[0][property];
			}",
            "let baz;
			const foo = array.find(bar);
			const bar = foo;
			({bar} = foo);
			function getValueOfFirst() {
				return foo.value;
			}
			function getPropertyOfFirst(property) {
				return foo[property];
			}",
            None,
        ),
        (
            "const quz = array.filter(fn);
			const [foo] = array.filter(quz[0]);
			[{bar: baz}] = foo[
				array.filter(fn)[0]
			].filter(
				array.filter(fn).shift()
			);",
            "const quz = array.find(fn);
			const [foo] = array.filter(quz);
			({bar: baz} = foo[
				array.filter(fn)[0]
			].find(
				array.filter(fn).shift()
			));",
            None,
        ),
        (
            "const quz = array.find(fn);
			const [foo] = array.filter(quz);
			({bar: baz} = foo[
				array.filter(fn)[0]
			].find(
				array.filter(fn).shift()
			));",
            "const quz = array.find(fn);
			const foo = array.find(quz);
			({bar: baz} = foo[
				array.find(fn)
			].find(
				array.find(fn)
			));",
            None,
        ),
        ("array.filter(foo).pop()", "array.findLast(foo)", None),
        ("array.filter(foo, thisArgument).pop()", "array.findLast(foo, thisArgument)", None),
        (
            "const item = array
				// comment 1
				.filter(
					// comment 2
					x => x === '🦄'
				)
				// comment 3
				.pop()
				// comment 4
				;",
            "const item = array
				// comment 1
				.findLast(
					// comment 2
					x => x === '🦄'
				)
				// comment 4
				;",
            None,
        ),
        ("array.filter(foo).at(-1)", "array.findLast(foo)", None),
        ("array.filter(foo, thisArgument).at(-1)", "array.findLast(foo, thisArgument)", None),
        (
            "const item = array
				// comment 1
				.filter(
					// comment 2
					x => x === '🦄'
				)
				// comment 3
				.at(
					// comment 4
					-1
					// comment 5
				)
				// comment 6
				;",
            "const item = array
				// comment 1
				.findLast(
					// comment 2
					x => x === '🦄'
				)
				// comment 6
				;",
            None,
        ),
        ("array.filter(foo).at(0)", "array.find(foo)", None),
        ("array.filter(foo, thisArgument).at(0)", "array.find(foo, thisArgument)", None),
        (
            "const item = array
				// comment 1
				.filter(
					// comment 2
					x => x === '🦄'
				)
				// comment 3
				.at(
					// comment 4
					0
					// comment 5
				)
				// comment 6
				;",
            "const item = array
				// comment 1
				.find(
					// comment 2
					x => x === '🦄'
				)
				// comment 6
				;",
            None,
        ),
    ];
    Tester::new(PreferArrayFind::NAME, PreferArrayFind::PLUGIN, pass, fail).test_and_snapshot();
}
