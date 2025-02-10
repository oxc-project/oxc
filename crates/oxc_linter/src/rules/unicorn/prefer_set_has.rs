use itertools::Itertools;
use oxc_ast::{
    ast::{Expression, MemberExpression, VariableDeclarationKind},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeId;
use oxc_span::Span;
use phf::phf_set;

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

const ARRAY_METHODS_RETURNS_ARRAY: phf::Set<&'static str> = phf_set! {
   "concat",
    "copyWithin",
    "fill",
    "filter",
    "flat",
    "flatMap",
    "map",
    "reverse",
    "slice",
    "sort",
    "splice",
    "toReversed",
    "toSorted",
    "toSpliced",
    "with",
};

fn prefer_set_has_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("should be a `Set`, and use `.has()` to check existence or non-existence.")
        .with_help("Switch to `Set`")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferSetHas;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `Set#has()` over `Array#includes()` when checking for existence or non-existence.
    ///
    /// ### Why is this bad?
    ///
    /// Set#has() is faster than Array#includes().
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const array = [1, 2, 3];
    /// const hasValue = value => array.includes(value);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const set = new Set([1, 2, 3]);
    /// const hasValue = value => set.has(value);
    /// ```
    /// ```js
    /// const array = [1, 2, 3];
    /// const hasOne = array.includes(1);
    /// ```
    PreferSetHas,
    unicorn,
    perf,
    dangerous_fix
);

fn is_array_of_or_from(callee: &MemberExpression) -> bool {
    callee.is_specific_member_access("Array", "of")
        || callee.is_specific_member_access("Array", "from")
}

fn is_kind_of_array_expr(expr: &Expression) -> bool {
    match expr {
        Expression::NewExpression(new_expr) => {
            new_expr.callee.get_identifier_reference().is_some_and(|ident| ident.name == "Array")
        }
        Expression::CallExpression(call_expr) => {
            let Some(callee) = call_expr.callee.get_member_expr() else {
                return call_expr.callee_name().is_some_and(|name| name == "Array");
            };

            if callee.is_computed() || callee.optional() {
                return false;
            }

            let Some(name) = callee.static_property_name() else { return false };

            is_array_of_or_from(callee) || ARRAY_METHODS_RETURNS_ARRAY.contains(name)
        }
        Expression::ArrayExpression(_) => true,
        _ => false,
    }
}

fn is_multiple_calls(node: &AstNode, ctx: &LintContext, root_scope_id: ScopeId) -> bool {
    let mut was_in_root_scope = node.scope_id() == root_scope_id;
    let mut is_multiple = false;
    for parent in ctx.nodes().ancestors(node.id()) {
        let parent_scope = parent.scope_id();
        if was_in_root_scope && parent_scope != root_scope_id {
            is_multiple = false;
            break;
        }
        was_in_root_scope = parent_scope == root_scope_id;
        let parent_kind = parent.kind();
        if matches!(
            parent_kind,
            AstKind::ForOfStatement(_)
                | AstKind::ForInStatement(_)
                | AstKind::ForStatement(_)
                | AstKind::WhileStatement(_)
                | AstKind::DoWhileStatement(_)
                | AstKind::ArrowFunctionExpression(_)
                | AstKind::Function(_)
        ) {
            is_multiple = true;
            break;
        };
    }
    is_multiple
}

impl Rule for PreferSetHas {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::VariableDeclarator(declarator) = node.kind() else {
            return;
        };

        if let VariableDeclarationKind::Var = declarator.kind {
            return;
        }

        let Some(init) = declarator.init.as_ref() else {
            return;
        };

        let is_kind_of_array = is_kind_of_array_expr(init);

        if !is_kind_of_array {
            return;
        }

        let Some(ident) = declarator.id.get_binding_identifier() else {
            return;
        };

        let Some(symbol_id) = ident.symbol_id.get() else {
            return;
        };

        let module_record = ctx.module_record();
        if module_record.exported_bindings.contains_key(ident.name.as_str())
            || module_record.export_default.is_some_and(|default| default == ident.span)
        {
            return;
        }
        let symbol_table = ctx.symbols();

        let mut references = symbol_table.get_resolved_references(symbol_id).peekable();

        let Ok(len) = references.try_len() else {
            return;
        };
        if len == 0 {
            return;
        }

        let root_scope = node.scope_id();

        if len == 1 {
            let Some(reference) = references.peek() else {
                return;
            };

            let node = ctx.nodes().get_node(reference.node_id());

            if !is_multiple_calls(node, ctx, root_scope) {
                return;
            }
        }

        if references.any(|reference| {
            let node = ctx.nodes().get_node(reference.node_id());
            let Some(parent_id) = ctx.nodes().parent_id(node.id()) else {
                return true;
            };
            let Some(AstKind::CallExpression(call_expression)) = ctx.nodes().parent_kind(parent_id)
            else {
                return true;
            };
            if call_expression.arguments.len() != 1 || call_expression.optional {
                return true;
            }
            let arg = &call_expression.arguments[0];
            if arg.is_spread() {
                return true;
            }
            let Some(AstKind::MemberExpression(member_expr)) = ctx.nodes().parent_kind(node.id())
            else {
                return true;
            };
            if member_expr.optional() || member_expr.is_computed() {
                return true;
            }
            let is_method = is_method_call(
                call_expression,
                Some(&[ident.name.as_str()]),
                Some(&["includes"]),
                Some(1),
                Some(1),
            );
            !is_method
        }) {
            return;
        }

        ctx.diagnostic_with_fix(prefer_set_has_diagnostic(declarator.span), |fixer| {
            let fixer = fixer.for_multifix();
            let mut declaration_fix = fixer.new_fix_with_capacity(2);
            let new_string = "new";
            let set_start_string = "Set(";
            let set_end_string = ")";
            let set_array_string = format!("{set_start_string}[");
            let net_set_array_string = format!("{new_string} {set_array_string}");
            let new_set_start_string = format!("{new_string} {set_start_string}");
            let array_end_string = format!("]{set_end_string}");
            let array_parenthesis_len = 6; // Array( => 6
            let new_space_len = 4; // new  => 4
            match init {
                Expression::ArrayExpression(init_node) => {
                    declaration_fix
                        .push(fixer.insert_text_before(&init_node.span, new_set_start_string));
                    declaration_fix.push(fixer.insert_text_after(&init_node.span, set_end_string));
                }
                Expression::CallExpression(call_expr) => {
                    if call_expr.callee.is_identifier_reference() {
                        let start = call_expr.span.start;
                        let end = start + 6;
                        let span = Span::new(start, end);
                        declaration_fix.push(fixer.replace(span, net_set_array_string));
                        let start = call_expr.span.end - 1;
                        let end = start + 1;
                        let span = Span::new(start, end);
                        declaration_fix.push(fixer.replace(span, array_end_string));
                    } else {
                        declaration_fix
                            .push(fixer.insert_text_before(&call_expr.span, new_set_start_string));
                        declaration_fix
                            .push(fixer.insert_text_after(&call_expr.span, set_end_string));
                    }
                }
                Expression::NewExpression(new_expr) => {
                    let start = new_expr.span.start + new_space_len;
                    let end = start + array_parenthesis_len;
                    let span = Span::new(start, end);
                    declaration_fix.push(fixer.replace(span, set_array_string));
                    let start = new_expr.span.end - 1;
                    let end = start + 1;
                    let span = Span::new(start, end);
                    declaration_fix.push(fixer.replace(span, array_end_string));
                }
                _ => {}
            }

            let mut references_fix = fixer.new_fix_with_capacity(len);
            let references = symbol_table.get_resolved_references(symbol_id);
            for reference in references {
                let node = ctx.nodes().get_node(reference.node_id());
                let Some(parent) = ctx.nodes().parent_node(node.id()) else {
                    continue;
                };
                let AstKind::MemberExpression(member_expr) = parent.kind() else {
                    continue;
                };
                let Some(property_info) = member_expr.static_property_info() else {
                    continue;
                };
                references_fix.push(fixer.replace(property_info.0, "has"));
            }
            declaration_fix.extend(references_fix)
        });
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "
        	const foo = new Set([1, 2, 3]);
        	function unicorn() {
        		return foo.has(1);
        	}
        ",
        "
            const foo = [1, 2, 3];
            const isExists = foo.includes(1);
        ",
        "
            while (a) {
            	const foo = [1, 2, 3];
            	const isExists = foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            (() => {})(foo.includes(1));
        ",
        "
            foo = [1, 2, 3];
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const exists = foo.includes(1);
        ",
        "
            const exists = [1, 2, 3].includes(1);
        ",
        "
            const foo = [1, 2, 3];
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
            	return foo.includes;
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
            	return includes(foo);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
            	return bar.includes(foo);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
            	return foo[includes](1);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
            	return foo.indexOf(1) !== -1;
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
            	foo.includes(1);
            	foo.length = 1;
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
            	if (foo.includes(1)) {}
            	return foo;
            				}
        ",
        "
            var foo = [1, 2, 3];
            var foo = [4, 5, 6];
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = bar;
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
            	return foo.includes();
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
            	return foo.includes(1, 1);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
            	return foo.includes(1, 0);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
            	return foo.includes(1, undefined);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
            	return foo.includes(...[1]);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
            	return foo?.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
            	return foo.includes?.(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
            	return foo?.includes?.(1);
            }
        ",
        "
            function unicorn() {
            	const foo = [1, 2, 3];
            }
            function unicorn2() {
            	return foo.includes(1);
            }
        ",
        "
            export const foo = [1, 2, 3];
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            module.exports = [1, 2, 3];
            function unicorn() {
            	return module.exports.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            export {foo};
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            export default foo;
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            export {foo as bar};
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            module.exports = foo;
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            exports = foo;
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            module.exports.foo = foo;
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = NotArray(1, 2);
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = new NotArray(1, 2);
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = NotArray.from({length: 1}, (_, index) => index);
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = NotArray.of(1, 2);
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = Array.notListed();
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = Array[from]({length: 1}, (_, index) => index);
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = Array[of](1, 2);
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = 'Array'.from({length: 1}, (_, index) => index);
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = 'Array'.of(1, 2);
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = Array['from']({length: 1}, (_, index) => index);
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = Array['of'](1, 2);
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = of(1, 2);
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = from({length: 1}, (_, index) => index);
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = bar.notListed();
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = _.map([1, 2, 3], value => value);
            function unicorn() {
            	return _.includes(foo, 1);
            }
        ",
        "
            @connect(
            	state => {
            		const availableComponents = ['is']
            		if (nsConfig.enabled) availableComponents.push('ns')
            		if (jsConfig.enabled) availableComponents.push('js')
            		if (asConfig.enabled) availableComponents.push('as')
            		return {
            			availableComponents,
            		}
            	},
            )
            export default class A {}
        ",
        "
            @connect(
            	state => {
            		const availableComponents = ['is']
            		if (nsConfig.enabled) availableComponents.push('ns')
            		if (jsConfig.enabled) availableComponents.push('js')
            		return {
            			availableComponents,
            		}
            	},
            )
            export default class A {}
        ",
    ];

    let fail = vec![
        "
			const foo = [1, 2, 3];
			function unicorn() {
				return foo.includes(1);
			}
		",
        "
            const foo = [1, 2, 3];
            const isExists = foo.includes(1);
            const isExists2 = foo.includes(2);
        ",
        "
            const foo = [1, 2, 3];
            for (const a of b) {
            	foo.includes(1);
            }
        ",
        "
            async function unicorn() {
            	const foo = [1, 2, 3];
            	for await (const a of b) {
            		foo.includes(1);
            	}
            }
        ",
        "
            const foo = [1, 2, 3];
            for (let i = 0; i < n; i++) {
            	foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            for (let a in b) {
            	foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            while (a)  {
            	foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            do {
            	foo.includes(1);
            } while (a)
        ",
        "
            const foo = [1, 2, 3];
            do {
            	// …
            } while (foo.includes(1))
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            function * unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            async function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            async function * unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            const unicorn = function () {
            	return foo.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            const unicorn = () => foo.includes(1);
        ",
        "
            const foo = [1, 2, 3];
            const a = {
            	b() {
            		return foo.includes(1);
            	}
            };
        ",
        "
            const foo = [1, 2, 3];
            class A {
            	b() {
            		return foo.includes(1);
            	}
            }
        ",
        "
            const foo = [...bar];
            function unicorn() {
            	return foo.includes(1);
            }
            bar.pop();
        ",
        "
            const foo = [1, 2, 3];
            function unicorn() {
            	const exists = foo.includes(1);
            	function isExists(find) {
            		return foo.includes(find);
            	}
            }
        ",
        "
            function wrap() {
            	const foo = [1, 2, 3];
            	function unicorn() {
            		return foo.includes(1);
            	}
            }
            const bar = [4, 5, 6];
            function unicorn() {
            	return bar.includes(1);
            }
        ",
        "
            const foo = [1, 2, 3];
            function wrap() {
            	const exists = foo.includes(1);
            	const bar = [1, 2, 3];
            	function outer(find) {
            		const foo = [1, 2, 3];
            		while (a) {
            			foo.includes(1);
            		}
            		function inner(find) {
            			const bar = [1, 2, 3];
            			while (a) {
            				const exists = bar.includes(1);
            			}
            		}
            	}
            }
        ",
        "
            const foo = Array(1, 2);
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = new Array(1, 2);
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = Array.from({length: 1}, (_, index) => index);
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = Array.of(1, 2);
            function unicorn() {
            	return foo.includes(1);
            }
        ",
        "
            const foo = _([1,2,3]);
            const bar = foo.map(value => value);
            function unicorn() {
            	return bar.includes(1);
            }
        ",
        "
            const a: Array<'foo' | 'bar'> = ['foo', 'bar']

            for (let i = 0; i < 3; i++) {
            	if (a.includes(someString)) {
            		console.log(123)
            	}
            }
        ",
    ];

    let fix = vec![
        (
            "
                const foo = [1, 2, 3];
                function unicorn() {
                    return foo.includes(1);
                }
            ",
            "
                const foo = new Set([1, 2, 3]);
                function unicorn() {
                    return foo.has(1);
                }
            ",
            None,
        ),
        (
            "
                const foo = [...bar];
                function unicorn() {
                	return foo.includes(1);
                }
                bar.pop();
            ",
            "
                const foo = new Set([...bar]);
                function unicorn() {
                	return foo.has(1);
                }
                bar.pop();
            ",
            None,
        ),
        (
            "
                const foo = Array(1, 2);
                function unicorn() {
                	return foo.includes(1);
                }
            ",
            "
                const foo = new Set([1, 2]);
                function unicorn() {
                	return foo.has(1);
                }
            ",
            None,
        ),
        (
            "
                const foo = new Array(1, 2);
                function unicorn() {
                	return foo.includes(1);
                }
            ",
            "
                const foo = new Set([1, 2]);
                function unicorn() {
                	return foo.has(1);
                }
            ",
            None,
        ),
        (
            "
                const foo = _([1,2,3]);
                const bar = foo.map(value => value);
                function unicorn() {
                	return bar.includes(1);
                }
            ",
            "
                const foo = _([1,2,3]);
                const bar = new Set(foo.map(value => value));
                function unicorn() {
                	return bar.has(1);
                }
            ",
            None,
        ),
        (
            "
                const foo = Array.of(1, 2);
                function unicorn() {
                    return foo.includes(1);
                }
            ",
            "
                const foo = new Set(Array.of(1, 2));
                function unicorn() {
                    return foo.has(1);
                }
            ",
            None,
        ),
    ];

    Tester::new(PreferSetHas::NAME, PreferSetHas::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
