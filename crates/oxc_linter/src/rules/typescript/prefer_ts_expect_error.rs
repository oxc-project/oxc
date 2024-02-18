use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(prefer-ts-expect-error):")]
#[diagnostic(severity(warning), help(""))]
struct PreferTsExpectErrorDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct PreferTsExpectError;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
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
