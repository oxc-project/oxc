use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use crate::{
    context::LintContext,
    utils::{PossibleJestNode, parse_expect_jest_fn_call},
};

fn no_alias_methods_diagnostic(name: &str, canonical_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected alias {name:?}"))
        .with_help(format!("Replace {name:?} with its canonical name of {canonical_name:?}"))
        .with_label(span)
}

pub fn run_on_jest_node<'a, 'c>(jest_node: &PossibleJestNode<'a, 'c>, ctx: &'c LintContext<'a>) {
    let node = jest_node.node;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };
    let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, jest_node, ctx) else {
        return;
    };
    let Some(matcher) = jest_fn_call.matcher() else {
        return;
    };
    let Some(alias) = matcher.name() else {
        return;
    };

    let Some(method_name) = BadAliasMethodName::from_str(alias.as_ref()) else {
        return;
    };
    let (name, canonical_name) = method_name.name_with_canonical();

    let mut span = matcher.span;
    // For quoted matchers, replace only the method name inside the quotes.
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
