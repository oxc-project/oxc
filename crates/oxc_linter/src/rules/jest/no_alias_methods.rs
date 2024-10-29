use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{parse_expect_jest_fn_call, PossibleJestNode},
};

fn no_alias_methods_diagnostic(x1: &str, x2: &str, span3: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected alias {x1:?}"))
        .with_help(format!("Replace {x1:?} with its canonical name of {x2:?}"))
        .with_label(span3)
}

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
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/veritem/eslint-plugin-vitest/blob/main/docs/rules/no-alias-methods.md),
    /// to use it, add the following configuration to your `.eslintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/no-alias-methods": "error"
    ///   }
    /// }
    /// ```
    NoAliasMethods,
    style,
    fix
);

impl Rule for NoAliasMethods {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(jest_node, ctx);
    }
}

fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
    let node = possible_jest_node.node;
    if let AstKind::CallExpression(call_expr) = node.kind() {
        if let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx) {
            let parsed_expect_call = jest_fn_call;
            let Some(matcher) = parsed_expect_call.matcher() else {
                return;
            };
            let Some(alias) = matcher.name() else {
                return;
            };

            if let Some(method_name) = BadAliasMethodName::from_str(alias.as_ref()) {
                let (name, canonical_name) = method_name.name_with_canonical();

                let mut span = matcher.span;
                // expect(a).not['toThrowError']()
                // matcher is the node of `toThrowError`, we only what to replace the content in the quotes.
                if matcher.element.is_string_literal() {
                    span.start += 1;
                    span.end -= 1;
                }

                ctx.diagnostic_with_fix(
                    no_alias_methods_diagnostic(name, canonical_name, matcher.span),
                    // || Fix::new(canonical_name, Span::new(start, end)),
                    |fixer| fixer.replace(span, canonical_name),
                );
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

    let mut pass = vec![
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

    let mut fail = vec![
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

    let mut fix = vec![
        ("expect(a).toBeCalled()", "expect(a).toHaveBeenCalled()", None),
        ("expect(a).not['toThrowError']()", "expect(a).not['toThrow']()", None),
        ("expect(a).not[`toThrowError`]()", "expect(a).not[`toThrow`]()", None),
    ];

    let pass_vitest = vec![
        "expect(a).toHaveBeenCalled()",
        "expect(a).toHaveBeenCalledTimes()",
        "expect(a).toHaveBeenCalledWith()",
        "expect(a).toHaveBeenLastCalledWith()",
        "expect(a).toHaveBeenNthCalledWith()",
        "expect(a).toHaveReturned()",
        "expect(a).toHaveReturnedTimes()",
        "expect(a).toHaveReturnedWith()",
        "expect(a).toHaveLastReturnedWith()",
        "expect(a).toHaveNthReturnedWith()",
        "expect(a).toThrow()",
        "expect(a).rejects;",
        "expect(a);",
    ];

    let fail_vitest = vec![
        "expect(a).toBeCalled()",
        "expect(a).toBeCalledTimes()",
        r#"expect(a).not["toThrowError"]()"#,
    ];

    let fix_vitest = vec![
        ("expect(a).toBeCalled()", "expect(a).toHaveBeenCalled()", None),
        ("expect(a).toBeCalledTimes()", "expect(a).toHaveBeenCalledTimes()", None),
        ("expect(a).not['toThrowError']()", "expect(a).not['toThrow']()", None),
    ];

    pass.extend(pass_vitest.into_iter().map(|x| (x, None)));
    fail.extend(fail_vitest.into_iter().map(|x| (x, None)));
    fix.extend(fix_vitest);

    Tester::new(NoAliasMethods::NAME, pass, fail)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
