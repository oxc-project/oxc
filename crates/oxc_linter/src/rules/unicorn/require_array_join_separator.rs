use oxc_ast::{
    ast::{CallExpression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(require-array-join-separator): Missing the separator argument.")]
#[diagnostic(severity(warning), help("Enforce using the separator argument with `Array#join()`."))]
struct RequireArrayJoinSeparatorDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct RequireArrayJoinSeparator;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// It's better to make it clear what the separator is when calling Array#join(), instead of relying on the default comma (',') separator.
    ///
    /// ### Example
    /// ```javascript
    /// // Invalid
    /// const string = array.join();
    /// const string = Array.prototype.join.call(arrayLike);
    /// const string = [].join.call(arrayLike);
    ///
    /// // Valid
    /// const string = array.join(',');
    /// const string = array.join('|');
    /// const string = Array.prototype.join.call(arrayLike, '');
    /// const string = [].join.call(arrayLike, '\n');
    /// ```
    RequireArrayJoinSeparator,
    correctness
);

fn is_empty_join_call(call_expr: &CallExpression) -> bool {
    // Not optional `arr?.join()`
    !call_expr.optional
    // Must be join call with no args `arr.join()`
        && is_method_call(call_expr, None, Some(&["join"]), None, Some(0))
    // Not be a computed property call `arr["join"]()`
        && !call_expr
            .callee
            .get_member_expr()
            .map(|member_expr| member_expr.is_computed())
            .unwrap_or(false)
}

impl Rule for RequireArrayJoinSeparator {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::CallExpression(callExpression) = node.kind() {
            dbg!(callExpression);
            if is_empty_join_call(&callExpression)
                || is_method_call(
                    callExpression,
                    Some(&["join"]),
                    Some(&["call"]),
                    Some(1),
                    Some(1),
                )
            {
                ctx.diagnostic(RequireArrayJoinSeparatorDiagnostic(callExpression.span));
            }

            // dbg!(is_join_call_with_0_args, callExpression.span.source_text(ctx.source_text()));
        }

        // CallExpression(node) {
        // 		if (!(
        // 			// `foo.join()`
        // 			isMethodCall(node, {
        // 				method: 'join',
        // 				argumentsLength: 0,
        // 				optionalCall: false,
        // 			})
        // 			// `[].join.call(foo)` and `Array.prototype.join.call(foo)`
        // 			|| (
        // 				isMethodCall(node, {
        // 					method: 'call',
        // 					argumentsLength: 1,
        // 					optionalCall: false,
        // 					optionalMember: false,
        // 				})
        // 				&& isArrayPrototypeProperty(node.callee.object, {
        // 					property: 'join',
        // 				})
        // 			)
        // 		)) {
        // 			return;
        // 		}

        // 		const {sourceCode} = context;
        // 		const [penultimateToken, lastToken] = sourceCode.getLastTokens(node, 2);
        // 		const isPrototypeMethod = node.arguments.length === 1;
        // 		return {
        // 			loc: {
        // 				start: penultimateToken.loc[isPrototypeMethod ? 'end' : 'start'],
        // 				end: lastToken.loc.end,
        // 			},
        // 			messageId: MESSAGE_ID,
        // 			/** @param {import('eslint').Rule.RuleFixer} fixer */
        // 			fix: fixer => appendArgument(fixer, node, '\',\'', sourceCode),
        // 		};
        // 	},
    }
}

#[test]
fn test_require_array_join_separator() {
    use crate::tester::Tester;

    let pass = vec![];
    let _pass = vec![
        r#"foo.join(",")"#,
        r#"join()"#,
        r#"foo.join(...[])"#,
        r#"foo.join?.()"#,
        r#"foo?.join?.()"#,
        r#"foo[join]()"#,
        r#"foo["join"]()"#,
        r#"[].join.call(foo, ",")"#,
        r#"[].join.call()"#,
        r#"[].join.call(...[foo])"#,
        r#"[].join?.call(foo)"#,
        r#"[]?.join.call(foo)"#,
        r#"[].join[call](foo)"#,
        r#"[][join].call(foo)"#,
        r#"[,].join.call(foo)"#,
        r#"[].join.notCall(foo)"#,
        r#"[].notJoin.call(foo)"#,
        r#"Array.prototype.join.call(foo, "")"#,
        r#"Array.prototype.join.call()"#,
        r#"Array.prototype.join.call(...[foo])"#,
        r#"Array.prototype.join?.call(foo)"#,
        r#"Array.prototype?.join.call(foo)"#,
        r#"Array?.prototype.join.call(foo)"#,
        r#"Array.prototype.join[call](foo, "")"#,
        r#"Array.prototype[join].call(foo)"#,
        r#"Array[prototype].join.call(foo)"#,
        r#"Array.prototype.join.notCall(foo)"#,
        r#"Array.prototype.notJoin.call(foo)"#,
        r#"Array.notPrototype.join.call(foo)"#,
        r#"NotArray.prototype.join.call(foo)"#,
        r#"path.join(__dirname, "./foo.js")"#,
    ];

    let fail = vec![
        r#"foo.join()"#,
        r#"[].join.call(foo)"#,
        r#"[].join.call(foo,)"#,
        r#"[].join.call(foo , );"#,
        r#"Array.prototype.join.call(foo)"#,
        r#"Array.prototype.join.call(foo, )"#,
        r#"foo?.join()"#,
    ];

    Tester::new_without_config(RequireArrayJoinSeparator::NAME, pass, fail).test_and_snapshot();
}
