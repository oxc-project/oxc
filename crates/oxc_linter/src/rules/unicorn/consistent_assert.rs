use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn consistent_assert_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ConsistentAssert;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ConsistentAssert,
    unicorn,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for ConsistentAssert {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "assert(foo)",
        r#"import assert from "assert";"#,
        "import assert from 'node:assert';
			assert;",
        "import customAssert from 'node:assert';
			assert(foo);",
        "function foo (assert) {
				assert(bar);
			}",
        "import assert from 'node:assert';
			function foo (assert) {
				assert(bar);
			}",
        "import {strict} from 'node:assert/strict';
			strict(foo);",
        "import * as assert from 'node:assert';
			assert(foo);",
        "export * as assert from 'node:assert';
			assert(foo);",
        "export {default as assert} from 'node:assert';
			export {assert as strict} from 'node:assert';
			assert(foo);",
        "import assert from 'node:assert/strict';
			console.log(assert)",
    ];

    let fail = vec![
        "import assert from 'assert';
			assert(foo)",
        "import assert from 'node:assert';
			assert(foo)",
        "import assert from 'assert/strict';
			assert(foo)",
        "import assert from 'node:assert/strict';
			assert(foo)",
        "import customAssert from 'assert';
			customAssert(foo)",
        "import customAssert from 'node:assert';
			customAssert(foo)",
        "import assert from 'assert';
			assert(foo)
			assert(bar)
			assert(baz)",
        "import {strict} from 'assert';
			strict(foo)",
        "import {strict as assert} from 'assert';
			assert(foo)",
        "import a, {strict as b, default as c} from 'node:assert';
			import d, {strict as e, default as f} from 'assert';
			import g, {default as h} from 'node:assert/strict';
			import i, {default as j} from 'assert/strict';
			a(foo);
			b(foo);
			c(foo);
			d(foo);
			e(foo);
			f(foo);
			g(foo);
			h(foo);
			i(foo);
			j(foo);",
        "import assert from 'node:assert';
			assert?.(foo)",
        "import assert from 'assert';
			((
				/* comment */ ((
					/* comment */
					assert
					/* comment */
					)) /* comment */
					(/* comment */ typeof foo === 'string', 'foo must be a string' /** after comment */)
			));",
    ];

    Tester::new(ConsistentAssert::NAME, ConsistentAssert::PLUGIN, pass, fail).test_and_snapshot();
}
