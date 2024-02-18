use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(prefer-ts-expect-error):")]
#[diagnostic(severity(warning), help("Use " @ ts - expect - error" to ensure an error is actually being suppressed."))]
struct PreferTsExpectErrorDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct PreferTsExpectError;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce using @ts-expect-error over @ts-ignore.
    ///
    /// ### Why is this bad?
    /// TypeScript allows you to suppress all errors on a line by placing a comment starting with @ts-ignore or @ts-expect-error immediately before the erroring line.
    /// The two directives work the same, except @ts-expect-error causes a type error if placed before a line that's not erroring in the first place.
    ///
    /// This means it's easy for @ts-ignores to be forgotten about, and remain in code even after the error they were suppressing is fixed.
    /// This is dangerous, as if a new error arises on that line it'll be suppressed by the forgotten about @ts-ignore, and so be missed.
    ///
    /// ### Example
    /// ```javascript
    /// // @ts-ignore
    /// const str: string = 1;
    ///
    /// /**
    /// * Explaining comment
    /// *
    /// * @ts-ignore */
    /// const multiLine: number = 'value';
    /// ```
    PreferTsExpectError,
    correctness
);

impl Rule for PreferTsExpectError {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"// @ts-nocheck"#,
        r#"// @ts-check"#,
        r#"// just a comment containing @ts-ignore somewhere"#,
        r#"
			{
			  /*
			        just a comment containing @ts-ignore somewhere in a block
			      */
			}
			    "#,
        r#"// @ts-expect-error"#,
        r#"
			if (false) {
			  // @ts-expect-error: Unreachable code error
			  console.log('hello');
			}
			    "#,
        r#"
			/**
			 * Explaining comment
			 *
			 * @ts-expect-error
			 *
			 * Not last line
			 * */
			    "#,
    ];

    let fail = vec![
        r#"// @ts-ignore"#,
        r#"// @ts-ignore: Suppress next line"#,
        r#"///@ts-ignore: Suppress next line"#,
        r#"
			if (false) {
			  // @ts-ignore: Unreachable code error
			  console.log('hello');
			}
			      "#,
        r#"/* @ts-ignore */"#,
        r#"
			/**
			 * Explaining comment
			 *
			 * @ts-ignore */
			      "#,
        r#"/* @ts-ignore in a single block */"#,
        r#"
			/*
			// @ts-ignore in a block with single line comments */
			      "#,
    ];

    Tester::new(PreferTsExpectError::NAME, pass, fail).test_and_snapshot();
}
