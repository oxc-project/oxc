use oxc_macros::declare_oxc_lint;

use crate::rule::Rule;

#[derive(Debug, Default, Clone)]
pub struct NoBaseToString;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule requires toString() and toLocaleString() calls to only be called on objects which provide useful information when stringified.
    ///
    /// ### Why is this bad?
    ///
    /// JavaScript's toString() method returns '[object Object]' on plain objects, which is not useful information. This rule prevents toString() and toLocaleString() from being called on objects that return less useful strings.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// // These will evaluate to '[object Object]'
    /// ({}).toString();
    /// ({foo: 'bar'}).toString();
    /// ({foo: 'bar'}).toLocaleString();
    ///
    /// // This will evaluate to 'Symbol()'
    /// Symbol('foo').toString();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const someString = 'Hello world';
    /// someString.toString();
    ///
    /// const someNumber = 42;
    /// someNumber.toString();
    ///
    /// const someBoolean = true;
    /// someBoolean.toString();
    ///
    /// class CustomToString {
    ///   toString() {
    ///     return 'CustomToString';
    ///   }
    /// }
    /// new CustomToString().toString();
    /// ```
    NoBaseToString(tsgolint),
    typescript,
    correctness,
    pending,
);

impl Rule for NoBaseToString {}
