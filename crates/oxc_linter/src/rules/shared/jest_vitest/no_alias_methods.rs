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

// TODO: This rule's documentation should be genericized to apply to Vitest as well, probably?
pub const DOCUMENTATION: &str = r#"### What it does

This rule ensures that only the canonical name as used in the Jest documentation is used in the code.

### Why is this bad?

These aliases are going to be removed in the next major version of Jest - see [jestjs/jest#13164](https://github.com/jestjs/jest/issues/13164) for more.
This rule will make it easier to search for all occurrences of the method within code, and it ensures consistency among the method names used.

### Examples

Examples of **incorrect** code for this rule:
```javascript
expect(a).toBeCalled()
expect(a).toBeCalledTimes()
expect(a).toBeCalledWith()
expect(a).lastCalledWith()
expect(a).nthCalledWith()
expect(a).toReturn()
expect(a).toReturnTimes()
expect(a).toReturnWith()
expect(a).lastReturnedWith()
expect(a).nthReturnedWith()
expect(a).toThrowError()
```

Examples of **correct** code for this rule:
```javascript
expect(a).toHaveBeenCalled()
expect(a).toHaveBeenCalledTimes()
expect(a).toHaveBeenCalledWith()
expect(a).toHaveBeenLastCalledWith()
expect(a).toHaveBeenNthCalledWith()
expect(a).toHaveReturned()
expect(a).toHaveReturnedTimes()
expect(a).toHaveReturnedWith()
expect(a).toHaveLastReturnedWith()
expect(a).toHaveNthReturnedWith()
expect(a).toThrow()
```

Examples of **incorrect** code for this rule with vitest:
```javascript
expect(a).toBeCalled()
expect(a).toBeCalledTimes()
expect(a).not["toThrowError"]()
```

Examples of **correct** code for this rule with vitest:
```javascript
expect(a).toHaveBeenCalled()
expect(a).toHaveBeenCalledTimes()
expect(a).toHaveBeenCalledWith()
expect(a).toHaveBeenLastCalledWith()
expect(a).toHaveBeenNthCalledWith()
expect(a).toHaveReturned()
expect(a).toHaveReturnedTimes()
expect(a).toHaveReturnedWith()
expect(a).toHaveLastReturnedWith()
expect(a).toHaveNthReturnedWith()
expect(a).toThrow()
expect(a).rejects
expect(a)
```
"#;

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
