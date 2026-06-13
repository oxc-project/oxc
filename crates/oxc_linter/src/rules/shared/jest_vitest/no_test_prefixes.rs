use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    utils::{
        JestFnKind, JestGeneralFnKind, KnownMemberExpressionProperty, ParsedGeneralJestFnCall,
        PossibleJestNode, parse_general_jest_fn_call,
    },
};

fn no_test_prefixes_diagnostic(preferred_node_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use `{preferred_node_name}` instead.")).with_label(span)
}

pub const DOCUMENTATION: &str = r"### What it does

Require using `.only` and `.skip` over `f` and `x`.

### Why is this bad?

Jest allows you to choose how you want to define focused and skipped tests,
with multiple permutations for each:
- only & skip: it.only, test.only, describe.only, it.skip, test.skip, describe.skip.
- 'f' & 'x': fit, fdescribe, xit, xtest, xdescribe.

This rule enforces usages from the only & skip list.

### Examples

Examples of **incorrect** code for this rule:
```javascript
fit('foo'); // invalid
fdescribe('foo'); // invalid
xit('foo'); // invalid
xtest('foo'); // invalid
xdescribe('foo'); // invalid
```
";

struct ParsedNoTestPrefixesCall {
    name: String,
    member_names: Vec<String>,
}

pub fn run_on_jest_node<'a, 'c>(jest_node: &PossibleJestNode<'a, 'c>, ctx: &'c LintContext<'a>) {
    run(jest_node, ctx);
}

fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
    let node = possible_jest_node.node;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };
    let Some(jest_fn_call) = parse_no_test_prefixes_call(call_expr, possible_jest_node, ctx) else {
        return;
    };

    let ParsedNoTestPrefixesCall { name, member_names } = jest_fn_call;
    let kind = JestFnKind::from(&name);
    let Some(kind) = kind.to_general() else {
        return;
    };

    if !matches!(kind, JestGeneralFnKind::Describe | JestGeneralFnKind::Test) {
        return;
    }

    if !name.starts_with('f') && !name.starts_with('x') {
        return;
    }

    let span = match &call_expr.callee {
        Expression::TaggedTemplateExpression(tagged_template_expr) => {
            tagged_template_expr.tag.span()
        }
        Expression::CallExpression(child_call_expr) => child_call_expr.callee.span(),
        _ => call_expr.callee.span(),
    };

    let preferred_node_name = get_preferred_node_names(&name, &member_names);

    ctx.diagnostic_with_fix(no_test_prefixes_diagnostic(&preferred_node_name, span), |fixer| {
        fixer.replace(span, preferred_node_name)
    });
}

fn parse_no_test_prefixes_call<'a>(
    call_expr: &'a oxc_ast::ast::CallExpression<'a>,
    possible_jest_node: &PossibleJestNode<'a, '_>,
    ctx: &LintContext<'a>,
) -> Option<ParsedNoTestPrefixesCall> {
    if let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, possible_jest_node, ctx) {
        return Some(ParsedNoTestPrefixesCall {
            name: jest_fn_call.name.to_string(),
            member_names: get_member_names(&jest_fn_call),
        });
    }

    if !ctx.frameworks().is_vitest() {
        return None;
    }

    if matches!(
        ctx.nodes().parent_kind(possible_jest_node.node.id()),
        AstKind::CallExpression(_)
            | AstKind::StaticMemberExpression(_)
            | AstKind::ComputedMemberExpression(_)
    ) {
        return None;
    }

    let mut member_names = Vec::new();
    let local_name = get_call_chain(&call_expr.callee, &mut member_names)?;
    let name = possible_jest_node.original.unwrap_or(&local_name).to_string();

    Some(ParsedNoTestPrefixesCall { name, member_names })
}

fn get_member_names(jest_fn_call: &ParsedGeneralJestFnCall) -> Vec<String> {
    jest_fn_call
        .members
        .iter()
        .filter_map(KnownMemberExpressionProperty::name)
        .map(|name| name.to_string())
        .collect()
}

fn get_call_chain(expr: &Expression, member_names: &mut Vec<String>) -> Option<String> {
    match expr {
        Expression::Identifier(ident) => Some(ident.name.to_string()),
        Expression::StaticMemberExpression(member_expr) => {
            let name = get_call_chain(&member_expr.object, member_names)?;
            member_names.push(member_expr.property.name.to_string());
            Some(name)
        }
        Expression::CallExpression(call_expr) => get_call_chain(&call_expr.callee, member_names),
        Expression::TaggedTemplateExpression(tagged_template_expr) => {
            get_call_chain(&tagged_template_expr.tag, member_names)
        }
        _ => None,
    }
}

fn get_preferred_node_names(name: &str, member_names: &[String]) -> String {
    let preferred_modifier = if name.starts_with('f') { "only" } else { "skip" };
    let member_names = member_names.join(".");
    let name_slice = &name[1..];

    if member_names.is_empty() {
        format!("{name_slice}.{preferred_modifier}")
    } else {
        format!("{name_slice}.{preferred_modifier}.{member_names}")
    }
}
