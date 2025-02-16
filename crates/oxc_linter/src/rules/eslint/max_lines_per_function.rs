use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::Semantic;
use oxc_span::{GetSpan, Span};
use serde_json::Value;

use crate::{
    ast_util::{is_function_node, iter_outer_expressions},
    context::LintContext,
    rule::Rule,
    utils::count_comment_lines,
    AstNode,
};

fn max_lines_per_function_diagnostic(count: usize, max: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Function has too many lines ({count})."))
        .with_help(format!("Maximum allowed is {max}."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct MaxLinesPerFunction {
    max: usize,
    skip_comments: bool,
    skip_blank_lines: bool,
    iifes: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Enforce a maximum number of lines of code in a function
    ///
    /// ### Why is this bad?
    /// Some people consider large functions a code smell. Large functions tend to
    /// do a lot of things and can make it hard following what’s going on. Many coding
    /// style guides dictate a limit of the number of lines that a function can
    /// comprise of. This rule can help enforce that style.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with a particular max value:
    /// ```js
    /// /*eslint max-lines-per-function: ["error", 2]*/
    /// function foo() {
    ///     const x = 0;
    /// }
    /// ```
    ///
    /// ```js
    /// /*eslint max-lines-per-function: ["error", 3]*/
    /// function foo() {
    ///     // a comment
    ///     const x = 0;
    /// }
    /// ```
    ///
    /// ```js
    /// /*eslint max-lines-per-function: ["error", 4]*/
    /// function foo() {
    ///     // a comment followed by a blank line
    ///
    ///     const x = 0;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule with a particular max value:
    /// ```js
    /// /*eslint max-lines-per-function: ["error", 3]*/
    /// function foo() {
    ///     const x = 0;
    /// }
    /// ```
    ///
    /// ```js
    /// /*eslint max-lines-per-function: ["error", 4]*/
    /// function foo() {
    ///     // a comment
    ///     const x = 0;
    /// }
    /// ```
    ///
    /// ```js
    /// /*eslint max-lines-per-function: ["error", 5]*/
    /// function foo() {
    ///     // a comment followed by a blank line
    ///
    ///     const x = 0;
    /// }
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the `{ "skipBlankLines": true }` option:
    /// ```js
    /// /*eslint max-lines-per-function: ["error", {"max": 2, "skipBlankLines": true}]*/
    /// function foo() {
    ///
    ///     const x = 0;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `{ "skipBlankLines": true }` option:
    /// ```js
    /// /*eslint max-lines-per-function: ["error", {"max": 3, "skipBlankLines": true}]*/
    /// function foo() {
    ///
    ///     const x = 0;
    /// }
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the `{ "skipComments": true }` option:
    /// ```js
    /// /*eslint max-lines-per-function: ["error", {"max": 2, "skipComments": true}]*/
    /// function foo() {
    ///     // a comment
    ///     const x = 0;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `{ "skipComments": true }` option:
    /// ```js
    /// /*eslint max-lines-per-function: ["error", {"max": 3, "skipComments": true}]*/
    /// function foo() {
    ///     // a comment
    ///     const x = 0;
    /// }
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the `{ "IIFEs": true }` option:
    /// ```js
    /// /*eslint max-lines-per-function: ["error", {"max": 2, "IIFEs": true}]*/
    /// (function(){
    ///     const x = 0;
    /// }());
    ///
    /// (() => {
    ///     const x = 0;
    /// })();
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `{ "IIFEs": true }` option:
    /// ```js
    /// /*eslint max-lines-per-function: ["error", {"max": 3, "IIFEs": true}]*/
    /// (function(){
    ///     const x = 0;
    /// }());
    ///
    /// (() => {
    ///     const x = 0;
    /// })();
    /// ```
    ///
    MaxLinesPerFunction,
    eslint,
    pedantic
);

impl Rule for MaxLinesPerFunction {
    fn from_configuration(value: Value) -> Self {
        let config = value.get(0);
        if let Some(max) = config
            .and_then(Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .and_then(|v| usize::try_from(v).ok())
        {
            Self { max, skip_comments: false, skip_blank_lines: false, iifes: false }
        } else {
            let max = config
                .and_then(|config| config.get("max"))
                .and_then(Value::as_number)
                .and_then(serde_json::Number::as_u64)
                .map_or(50, |v| usize::try_from(v).unwrap_or(50));
            let skip_comments = config
                .and_then(|config| config.get("skipComments"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let skip_blank_lines = config
                .and_then(|config| config.get("skipBlankLines"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let iifes = config
                .and_then(|config| config.get("IIFEs"))
                .and_then(Value::as_bool)
                .unwrap_or(false);

            Self { max, skip_comments, skip_blank_lines, iifes }
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !is_function_node(node) || (!self.iifes && is_iife(node, ctx.semantic())) {
            return;
        }
        let span = node.span();
        let source_text = ctx.source_text();

        let comment_lines = if self.skip_comments {
            ctx.semantic()
                .comments_range(span.start..span.end)
                .map(|comment| count_comment_lines(comment, source_text))
                .sum()
        } else {
            0
        };

        let code = &source_text[span.start as usize..span.end as usize];
        let lines_in_function = code.lines().count();
        let blank_lines = if self.skip_blank_lines {
            code.lines().filter(|&line| line.trim().is_empty()).count()
        } else {
            0
        };
        let result_lines =
            lines_in_function.saturating_sub(blank_lines).saturating_sub(comment_lines);
        if result_lines > self.max {
            ctx.diagnostic(max_lines_per_function_diagnostic(result_lines, self.max, span));
        }
    }
}

fn is_iife<'a>(node: &AstNode<'a>, semantic: &Semantic<'a>) -> bool {
    let Some(AstKind::CallExpression(call)) = iter_outer_expressions(semantic, node.id()).next()
    else {
        return false;
    };
    call.callee.span().contains_inclusive(node.span())
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "var x = 5;
			var x = 2;
			",
            Some(serde_json::json!([1])),
        ),
        ("function name() {}", Some(serde_json::json!([1]))),
        (
            "function name() {
			var x = 5;
			var x = 2;
			}",
            Some(serde_json::json!([4])),
        ),
        ("const bar = () => 2", Some(serde_json::json!([1]))),
        (
            "const bar = () => {
			const x = 2 + 1;
			return x;
			}",
            Some(serde_json::json!([4])),
        ),
        (
            "function name() {
			var x = 5;



			var x = 2;
			}",
            Some(serde_json::json!([{ "max": 7, "skipComments": false, "skipBlankLines": false }])),
        ),
        (
            "function name() {
			var x = 5;



			var x = 2;
			}",
            Some(serde_json::json!([{ "max": 4, "skipComments": false, "skipBlankLines": true }])),
        ),
        (
            "function name() {
			var x = 5;
			var x = 2; // end of line comment
			}",
            Some(serde_json::json!([{ "max": 4, "skipComments": true, "skipBlankLines": false }])),
        ),
        (
            "function name() {
			var x = 5;
			// a comment on it's own line
			var x = 2; // end of line comment
			}",
            Some(serde_json::json!([{ "max": 4, "skipComments": true, "skipBlankLines": false }])),
        ),
        (
            "function name() {
			var x = 5;
			// a comment on it's own line
			// and another line comment
			var x = 2; // end of line comment
			}",
            Some(serde_json::json!([{ "max": 4, "skipComments": true, "skipBlankLines": false }])),
        ),
        (
            "function name() {
			var x = 5;
			/* a
			 multi
			 line
			 comment
			*/

			var x = 2; // end of line comment
			}",
            Some(serde_json::json!([{ "max": 5, "skipComments": true, "skipBlankLines": false }])),
        ),
        (
            "function name() {
			var x = 5;
				/* a comment with leading whitespace */
			/* a comment with trailing whitespace */
				/* a comment with trailing and leading whitespace */
			/* a
			 multi
			 line
			 comment
			*/

			var x = 2; // end of line comment
			}",
            Some(serde_json::json!([{ "max": 5, "skipComments": true, "skipBlankLines": false }])),
        ),
        (
            "function foo(
			    aaa = 1,
			    bbb = 2,
			    ccc = 3
			) {
			    return aaa + bbb + ccc
			}",
            Some(serde_json::json!([{ "max": 7, "skipComments": true, "skipBlankLines": false }])),
        ),
        (
            "(
			function
			()
			{
			}
			)
			()",
            Some(
                serde_json::json!([{ "max": 4, "skipComments": true, "skipBlankLines": false, "IIFEs": true }]),
            ),
        ),
        (
            "function parent() {
			var x = 0;
			function nested() {
			    var y = 0;
			    x = 2;
			}
			if ( x === y ) {
			    x++;
			}
			}",
            Some(serde_json::json!([{ "max": 10, "skipComments": true, "skipBlankLines": false }])),
        ),
        (
            "class foo {
			    method() {
			        let y = 10;
			        let x = 20;
			        return y + x;
			    }
			}",
            Some(serde_json::json!([{ "max": 5, "skipComments": true, "skipBlankLines": false }])),
        ),
        (
            "(function(){
			    let x = 0;
			    let y = 0;
			    let z = x + y;
			    let foo = {};
			    return bar;
			}());",
            Some(
                serde_json::json!([{ "max": 7, "skipComments": true, "skipBlankLines": false, "IIFEs": true }]),
            ),
        ),
        (
            "(function(){
			    let x = 0;
			    let y = 0;
			    let z = x + y;
			    let foo = {};
			    return bar;
			}());",
            Some(
                serde_json::json!([{ "max": 2, "skipComments": true, "skipBlankLines": false, "IIFEs": false }]),
            ),
        ),
        (
            "(() => {
			    let x = 0;
			    let y = 0;
			    let z = x + y;
			    let foo = {};
			    return bar;
			})();",
            Some(
                serde_json::json!([{ "max": 7, "skipComments": true, "skipBlankLines": false, "IIFEs": true }]),
            ),
        ),
        (
            "(() => {
			    let x = 0;
			    let y = 0;
			    let z = x + y;
			    let foo = {};
			    return bar;
			})();",
            Some(
                serde_json::json!([{ "max": 2, "skipComments": true, "skipBlankLines": false, "IIFEs": false }]),
            ),
        ),
    ];

    let repeat_60 = format!("() => {{{}}}", &"foo\n".repeat(60));

    let fail = vec![
        (
            "function name() {
			}",
            Some(serde_json::json!([1])),
        ),
        (
            "var func = function() {
			}",
            Some(serde_json::json!([1])),
        ),
        (
            "const bar = () => {
			const x = 2 + 1;
			return x;
			}",
            Some(serde_json::json!([3])),
        ),
        (
            "const bar = () =>
			 2",
            Some(serde_json::json!([1])),
        ),
        (&repeat_60, Some(serde_json::json!([{}]))),
        (
            "function name() {
			var x = 5;



			var x = 2;
			}",
            Some(serde_json::json!([{ "max": 6, "skipComments": false, "skipBlankLines": false }])),
        ),
        (
            "function name() {
			var x = 5;



			var x = 2;
			}",
            Some(serde_json::json!([{ "max": 6, "skipComments": true, "skipBlankLines": false }])),
        ),
        (
            "function name() {
			var x = 5;



			var x = 2;
			}",
            Some(serde_json::json!([{ "max": 2, "skipComments": true, "skipBlankLines": true }])),
        ),
        (
            "function name() {
			var x = 5;



			var x = 2;
			}",
            Some(serde_json::json!([{ "max": 2, "skipComments": true, "skipBlankLines": true }])),
        ),
        (
            "function name() { // end of line comment
			var x = 5; /* mid line comment */
				// single line comment taking up whole line



			var x = 2;
			}",
            Some(serde_json::json!([{ "max": 6, "skipComments": true, "skipBlankLines": false }])),
        ),
        (
            "function name() { // end of line comment
			var x = 5; /* mid line comment */
				// single line comment taking up whole line



			var x = 2;
			}",
            Some(serde_json::json!([{ "max": 1, "skipComments": true, "skipBlankLines": true }])),
        ),
        (
            "function name() { // end of line comment
			var x = 5; /* mid line comment */
				// single line comment taking up whole line



			var x = 2;
			}",
            Some(serde_json::json!([{ "max": 1, "skipComments": false, "skipBlankLines": true }])),
        ),
        (
            "function foo(
			    aaa = 1,
			    bbb = 2,
			    ccc = 3
			) {
			    return aaa + bbb + ccc
			}",
            Some(serde_json::json!([{ "max": 2, "skipComments": true, "skipBlankLines": false }])),
        ),
        (
            "(
			function
			()
			{
			}
			)
			()",
            Some(
                serde_json::json!([{ "max": 2, "skipComments": true, "skipBlankLines": false, "IIFEs": true }]),
            ),
        ),
        (
            "function parent() {
			var x = 0;
			function nested() {
			    var y = 0;
			    x = 2;
			}
			if ( x === y ) {
			    x++;
			}
			}",
            Some(serde_json::json!([{ "max": 9, "skipComments": true, "skipBlankLines": false }])),
        ),
        (
            "function parent() {
			var x = 0;
			function nested() {
			    var y = 0;
			    x = 2;
			}
			if ( x === y ) {
			    x++;
			}
			}",
            Some(serde_json::json!([{ "max": 2, "skipComments": true, "skipBlankLines": false }])),
        ),
        (
            "class foo {
			    method() {
			        let y = 10;
			        let x = 20;
			        return y + x;
			    }
			}",
            Some(serde_json::json!([{ "max": 2, "skipComments": true, "skipBlankLines": false }])),
        ),
        (
            "class A {
			    static
			    foo
			    (a) {
			        return a
			    }
			}",
            Some(serde_json::json!([{ "max": 2, "skipComments": true, "skipBlankLines": false }])),
        ),
        (
            "var obj = {
			    get
			    foo
			    () {
			        return 1
			    }
			}",
            Some(serde_json::json!([{ "max": 2, "skipComments": true, "skipBlankLines": false }])),
        ),
        (
            "var obj = {
			    set
			    foo
			    ( val ) {
			        this._foo = val;
			    }
			}",
            Some(serde_json::json!([{ "max": 2, "skipComments": true, "skipBlankLines": false }])),
        ),
        (
            "class A {
			    static
			    [
			        foo +
			            bar
			    ]
			    (a) {
			        return a
			    }
			}",
            Some(serde_json::json!([{ "max": 2, "skipComments": true, "skipBlankLines": false }])),
        ),
        (
            "(function(){
			    let x = 0;
			    let y = 0;
			    let z = x + y;
			    let foo = {};
			    return bar;
			}());",
            Some(
                serde_json::json!([{ "max": 2, "skipComments": true, "skipBlankLines": false, "IIFEs": true }]),
            ),
        ),
        (
            "(() => {
			    let x = 0;
			    let y = 0;
			    let z = x + y;
			    let foo = {};
			    return bar;
			})();",
            Some(
                serde_json::json!([{ "max": 2, "skipComments": true, "skipBlankLines": false, "IIFEs": true }]),
            ),
        ),
    ];

    Tester::new(MaxLinesPerFunction::NAME, MaxLinesPerFunction::PLUGIN, pass, fail)
        .test_and_snapshot();
}
