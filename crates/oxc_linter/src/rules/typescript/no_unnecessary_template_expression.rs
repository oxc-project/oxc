use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoUnnecessaryTemplateExpression;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows unnecessary template literals.
    ///
    /// ### Why is this bad?
    ///
    /// Template literals should only be used when they are needed for string interpolation or multi-line strings. Using template literals when a simple string would suffice adds unnecessary complexity.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const str1 = `Hello world`;
    ///
    /// const str2 = `42`;
    ///
    /// const str3 = `true`;
    ///
    /// // Template with only literal expressions
    /// const str4 = `${'Hello'} ${'world'}`;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const str1 = 'Hello world';
    ///
    /// const str2 = '42';
    ///
    /// const str3 = 'true';
    ///
    /// // Template with variable interpolation
    /// const name = 'world';
    /// const str4 = `Hello ${name}`;
    ///
    /// // Multi-line string
    /// const multiline = `
    ///   Hello
    ///   world
    /// `;
    ///
    /// // Template with expression
    /// const str5 = `Result: ${1 + 2}`;
    /// ```
    NoUnnecessaryTemplateExpression(tsgolint),
    typescript,
    suspicious,
    pending,
);

impl Rule for NoUnnecessaryTemplateExpression {}
