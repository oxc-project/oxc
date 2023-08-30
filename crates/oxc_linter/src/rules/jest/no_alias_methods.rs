use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext, fixer::Fix, jest_ast_util::parse_expect_jest_fn_call, rule::Rule, AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(jest/no-alias-methods): Unexpected alias {0:?}()")]
#[diagnostic(severity(warning), help("Replace {0:?} with its canonical name of {1:?}"))]
struct NoAliasMethodsDiagnostic(pub &'static str, pub &'static str, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoAliasMethods;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule ensures that only the canonical name as used in the Jest documentation is used in the code.
    ///
    /// ### Why is this bad?
    ///
    /// These aliases are going to be removed in the next major version of Jest - see [jestjs/jest#13164](https://github.com/jestjs/jest/issues/13164) for more.
    /// This rule will makes it easier to search for all occurrences of the method within code, and it ensures consistency among the method names used.
    ///
    /// ### Example
    /// ```javascript
    /// expect(a).toBeCalled();
    /// expect(a).toBeCalledTimes();
    /// expect(a).toBeCalledWith();
    /// expect(a).lastCalledWith();
    /// expect(a).nthCalledWith();
    /// expect(a).toReturn();
    /// expect(a).toReturnTimes();
    /// expect(a).toReturnWith();
    /// expect(a).lastReturnedWith();
    /// expect(a).nthReturnedWith();
    /// expect(a).toThrowError();
    /// ```
    NoAliasMethods,
    correctness
);

impl Rule for NoAliasMethods {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::CallExpression(call_expr) = node.kind() {
            if let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, node, ctx) {
                let parsed_expect_call = jest_fn_call;
                let Some(matcher) = parsed_expect_call.matcher() else {
                    return;
                };
                let Some(alias) = matcher.name() else {
                    return;
                };

                if let Some(method_name) = BadAliasMethodName::from_str(alias.as_ref()) {
                    let (name, canonical_name) = method_name.name_with_canonical();

                    let Span { mut start, mut end } = matcher.span;
                    // expect(a).not['toThrowError']()
                    // matcher is the node of `toThrowError`, we only what to replace the content in the quotes.
                    if matcher.element.is_string_literal() {
                        start += 1;
                        end -= 1;
                    }

                    ctx.diagnostic_with_fix(
                        NoAliasMethodsDiagnostic(name, canonical_name, matcher.span),
                        || Fix::new(canonical_name, Span { start, end }),
                    );
                }
            }
        }
    }
}

enum BadAliasMethodName {
    ToBeCalled,
    ToBeCalledTimes,
    ToBeCalledWith,
    LastCalledWith,
    NthCalledWith,
    ToReturn,
    ToReturnTimes,
    ToReturnWith,
    LastReturnedWith,
    NthReturnedWith,
    ToThrowError,
}

impl BadAliasMethodName {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "toBeCalled" => Some(Self::ToBeCalled),
            "toBeCalledTimes" => Some(Self::ToBeCalledTimes),
            "toBeCalledWith" => Some(Self::ToBeCalledWith),
            "lastCalledWith" => Some(Self::LastCalledWith),
            "nthCalledWith" => Some(Self::NthCalledWith),
            "toReturn" => Some(Self::ToReturn),
            "toReturnTimes" => Some(Self::ToReturnTimes),
            "toReturnWith" => Some(Self::ToReturnWith),
            "lastReturnedWith" => Some(Self::LastReturnedWith),
            "nthReturnedWith" => Some(Self::NthReturnedWith),
            "toThrowError" => Some(Self::ToThrowError),
            _ => None,
        }
    }
    fn name_with_canonical(&self) -> (&'static str, &'static str) {
        match self {
            Self::ToBeCalled => ("toBeCalled", "toHaveBeenCalled"),
            Self::ToBeCalledTimes => ("toBeCalledTimes", "toHaveBeenCalledTimes"),
            Self::ToBeCalledWith => ("toBeCalledWith", "toHaveBeenCalledWith"),
            Self::LastCalledWith => ("lastCalledWith", "toHaveBeenLastCalledWith"),
            Self::NthCalledWith => ("nthCalledWith", "toHaveBeenNthCalledWith"),
            Self::ToReturn => ("toReturn", "toHaveReturned"),
            Self::ToReturnTimes => ("toReturnTimes", "toHaveReturnedTimes"),
            Self::ToReturnWith => ("toReturnWith", "toHaveReturnedWith"),
            Self::LastReturnedWith => ("lastReturnedWith", "toHaveLastReturnedWith"),
            Self::NthReturnedWith => ("nthReturnedWith", "toHaveNthReturnedWith"),
            Self::ToThrowError => ("toThrowError", "toThrow"),
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("expect(a).toHaveBeenCalled()", None),
        ("expect(a).toHaveBeenCalledTimes()", None),
        ("expect(a).toHaveBeenCalledWith()", None),
        ("expect(a).toHaveBeenLastCalledWith()", None),
        ("expect(a).toHaveBeenNthCalledWith()", None),
        ("expect(a).toHaveReturned()", None),
        ("expect(a).toHaveReturnedTimes()", None),
        ("expect(a).toHaveReturnedWith()", None),
        ("expect(a).toHaveLastReturnedWith()", None),
        ("expect(a).toHaveNthReturnedWith()", None),
        ("expect(a).toThrow()", None),
        ("expect(a).rejects;", None),
        ("expect(a);", None),
    ];

    let fail = vec![
        ("expect(a).toBeCalled()", None),
        ("expect(a).toBeCalledTimes()", None),
        ("expect(a).toBeCalledWith()", None),
        ("expect(a).lastCalledWith()", None),
        ("expect(a).nthCalledWith()", None),
        ("expect(a).toReturn()", None),
        ("expect(a).toReturnTimes()", None),
        ("expect(a).toReturnWith()", None),
        ("expect(a).lastReturnedWith()", None),
        ("expect(a).nthReturnedWith()", None),
        ("expect(a).toThrowError()", None),
        ("expect(a).resolves.toThrowError()", None),
        ("expect(a).rejects.toThrowError()", None),
        ("expect(a).not.toThrowError()", None),
        ("expect(a).not['toThrowError']()", None),
    ];

    let fix = vec![
        ("expect(a).toBeCalled()", "expect(a).toHaveBeenCalled()", None),
        ("expect(a).not['toThrowError']()", "expect(a).not['toThrow']()", None),
        ("expect(a).not[`toThrowError`]()", "expect(a).not[`toThrow`]()", None),
    ];

    Tester::new(NoAliasMethods::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
