use std::ops::Deref;

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::Semantic;
use oxc_span::{GetSpan, Span};
use serde_json::Value;

use crate::{
    AstNode,
    ast_util::{get_function_name_with_kind, is_function_node, iter_outer_expressions},
    context::LintContext,
    rule::Rule,
    utils::count_comment_lines,
};

fn max_lines_per_function_diagnostic(
    name: &str,
    count: usize,
    max: usize,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "The {name} has too many lines ({count}). Maximum allowed is {max}."
    ))
    .with_help("Consider splitting it into smaller functions.")
    .with_label(span)
}

#[derive(Debug, Clone)]
pub struct MaxLinesPerFunctionConfig {
    max: usize,
    skip_comments: bool,
    skip_blank_lines: bool,
    iifes: bool,
}

const DEFAULT_MAX_LINES_PER_FUNCTION: usize = 50;

impl Default for MaxLinesPerFunctionConfig {
    fn default() -> Self {
        Self {
            max: DEFAULT_MAX_LINES_PER_FUNCTION,
            skip_comments: false,
            skip_blank_lines: false,
            iifes: false,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MaxLinesPerFunction(Box<MaxLinesPerFunctionConfig>);

impl Deref for MaxLinesPerFunction {
    type Target = MaxLinesPerFunctionConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce a maximum number of lines of code in a function. This rule ensures
    /// that functions do not exceed a specified line count, promoting smaller,
    /// more focused functions that are easier to maintain and understand.
    ///
    /// ### Why is this bad?
    ///
    /// Some people consider large functions a code smell. Large functions tend to
    /// do a lot of things and can make it hard to follow what’s going on. Many coding
    /// style guides dictate a limit to the number of lines that a function can
    /// comprise of. This rule can help enforce that style.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with a particular max value:
    /// ```js
    /// /* { "eslint/max-lines-per-function": ["error", 2] } */
    /// function foo() {
    ///     const x = 0;
    /// }
    ///
    /// /* { "eslint/max-lines-per-function": ["error", 4] } */
    /// function foo() {
    ///     // a comment followed by a blank line
    ///
    ///     const x = 0;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule with a particular max value:
    /// ```js
    /// /* { "eslint/max-lines-per-function": ["error", 3] } */
    /// function foo() {
    ///     const x = 0;
    /// }
    ///
    /// /* { "eslint/max-lines-per-function": ["error", 5] } */
    /// function foo() {
    ///     // a comment followed by a blank line
    ///
    ///     const x = 0;
    /// }
    /// ```
    ///
    /// ### Options
    ///
    /// #### max
    ///
    /// { type: number, default: 50 }
    ///
    /// The `max` enforces a maximum number of lines in a function.
    ///
    /// #### skipBlankLines
    ///
    /// { type: boolean, default: false }
    ///
    /// The `skipBlankLines` ignore lines made up purely of whitespace.
    ///
    /// #### skipComments
    ///
    /// { type: boolean, default: false }
    ///
    /// The `skipComments` ignore lines containing just comments.
    ///
    /// #### IIFEs
    ///
    /// { type: boolean, default: false }
    ///
    /// The `IIFEs` option controls whether IIFEs are included in the line count.
    /// By default, IIFEs are not considered, but when set to `true`, they will
    /// be included in the line count for the function.
    ///
    /// Example:
    /// ```json
    /// "eslint/max-lines-per-function": [
    ///   "error",
    ///   {
    ///     "max": 50,
    ///     "skipBlankLines": false,
    ///     "skipComments": false,
    ///     "IIFEs": false
    ///   }
    /// ]
    /// ```
    MaxLinesPerFunction,
    eslint,
    pedantic
);

impl Rule for MaxLinesPerFunction {
    fn from_configuration(value: Value) -> Self {
        let config = value.get(0);
        let config = if let Some(max) = config
            .and_then(Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .and_then(|v| usize::try_from(v).ok())
        {
            MaxLinesPerFunctionConfig {
                max,
                skip_comments: false,
                skip_blank_lines: false,
                iifes: false,
            }
        } else {
            let max = config
                .and_then(|config| config.get("max"))
                .and_then(Value::as_number)
                .and_then(serde_json::Number::as_u64)
                .map_or(DEFAULT_MAX_LINES_PER_FUNCTION, |v| {
                    usize::try_from(v).unwrap_or(DEFAULT_MAX_LINES_PER_FUNCTION)
                });
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

            MaxLinesPerFunctionConfig { max, skip_comments, skip_blank_lines, iifes }
        };

        Self(Box::new(config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !is_function_node(node) || (!self.iifes && is_iife(node, ctx.semantic())) {
            return;
        }
        let span = node.span();
        let source_text = ctx.source_text();

        let comment_lines = if self.skip_comments {
            ctx.comments_range(span.start..span.end)
                .map(|comment| count_comment_lines(comment, source_text))
                .sum()
        } else {
            0
        };

        let code = &source_text[span.start as usize..span.end as usize];
        let lines_in_function = if self.skip_blank_lines {
            code.lines().filter(|&line| !line.trim().is_empty()).count()
        } else {
            // Intentionally counting newline bytes instead of using .lines() for performance (see PR 11242)
            let newlines = code.bytes().filter(|ch| *ch == b'\n').count();
            if code.ends_with('\n') { newlines } else { newlines + 1 }
        };

        let final_lines = lines_in_function.saturating_sub(comment_lines);
        if final_lines > self.max {
            let name = get_function_name_with_kind(node, ctx.nodes().parent_node(node.id()));
            ctx.diagnostic(max_lines_per_function_diagnostic(&name, final_lines, self.max, span));
        }
    }
}

fn is_iife<'a>(node: &AstNode<'a>, semantic: &Semantic<'a>) -> bool {
    let Some(AstKind::CallExpression(call)) =
        iter_outer_expressions(semantic.nodes(), node.id()).next()
    else {
        return false;
    };
    call.callee.span().contains_inclusive(node.span())
}

#[test]
fn test() {
    use crate::tester::Tester;

    let defaults = MaxLinesPerFunction::default();
    assert_eq!(defaults.max, 50);
    assert!(!defaults.skip_comments);
    assert!(!defaults.skip_blank_lines);
    assert!(!defaults.iifes);

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
