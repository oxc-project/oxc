use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn max_nested_calls_diagnostic(max: u32, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Call is nested too deeply. Maximum allowed is {max}."))
        .with_help("Extract intermediate results into named variables to reduce nesting.")
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct MaxNestedCalls {
    /// The maximum allowed nested call depth.
    max: u32,
}

const DEFAULT_MAX_NESTED_CALLS: u32 = 3;

impl Default for MaxNestedCalls {
    fn default() -> Self {
        Self { max: DEFAULT_MAX_NESTED_CALLS }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Limit the depth of nested calls.
    ///
    /// This rule counts calls and constructor calls passed into other calls or
    /// constructors. Fluent receiver chains and JSX wrappers are ignored.
    ///
    /// ### Why is this bad?
    ///
    /// Deeply nested calls make code hard to read. Extracting intermediate
    /// results into named variables improves readability.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// foo(bar(baz(qux())));
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const value = baz(qux());
    /// foo(bar(value));
    ///
    /// // Fluent chains are ignored.
    /// query().filter().map().toArray();
    /// ```
    MaxNestedCalls,
    unicorn,
    style,
    config = MaxNestedCalls,
    version = "next",
    short_description = "Limit the depth of nested calls.",
);

impl Rule for MaxNestedCalls {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let node_span = match node.kind() {
            AstKind::CallExpression(call) => call.span,
            AstKind::NewExpression(new_expression) => new_expression.span,
            _ => return,
        };

        let mut depth = 1u32;
        let mut child_span = node_span;

        for ancestor in ctx.nodes().ancestors(node.id()) {
            match ancestor.kind() {
                // Calls inside a new scope are counted independently.
                AstKind::Function(_)
                | AstKind::ArrowFunctionExpression(_)
                | AstKind::Class(_)
                | AstKind::StaticBlock(_)
                | AstKind::JSXElement(_)
                | AstKind::JSXFragment(_) => break,
                AstKind::CallExpression(call)
                    if call.arguments.iter().any(|argument| argument.span() == child_span) =>
                {
                    depth += 1;
                }
                AstKind::NewExpression(new_expression)
                    if new_expression
                        .arguments
                        .iter()
                        .any(|argument| argument.span() == child_span) =>
                {
                    depth += 1;
                }
                _ => {}
            }
            child_span = ancestor.span();
        }

        if depth > self.max {
            ctx.diagnostic(max_nested_calls_diagnostic(self.max, node_span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("foo();", None),
        ("foo(bar(baz()));", None),
        ("foo(bar(), baz(), qux());", None),
        ("query().filter().map().toArray();", None),
        ("foo()[bar()]().baz();", None),
        ("foo(() => bar(baz(qux())));", None),
        ("foo(bar(class {field = baz(qux());}));", None),
        ("foo(bar(class {static {baz(qux());}}));", None),
        ("new Foo(new Bar(new Baz()));", None),
        ("await foo(await bar(await baz()));", None),
        ("foo?.(bar?.(baz?.()));", None),
        ("foo(...bar(baz()));", None),
        ("foo(condition ? bar(baz()) : qux());", None),
        (
            "nativeDiffButtons.parentElement.after(
                tooltipped(
                    {},
                    <a className={cx('btn', isHidingWhitespace() && 'color-fg-subtle')} />,
                ),
            );",
            None,
        ),
        (
            "nativeDiffButtons.parentElement.after(
                tooltipped(
                    {},
                    <>{cx('btn', isHidingWhitespace() && 'color-fg-subtle')}</>,
                ),
            );",
            None,
        ),
        ("foo(bar(baz(qux())));", Some(serde_json::json!([{"max": 4}]))),
    ];

    let fail = vec![
        ("foo(bar(baz(qux())));", None),
        ("foo(bar(baz()));", Some(serde_json::json!([{"max": 2}]))),
        ("new Foo(new Bar(new Baz(new Qux())));", None),
        ("await foo(await bar(await baz(await qux())));", None),
        ("foo?.(bar?.(baz?.(qux?.())));", None),
        ("foo(...bar(baz(qux())));", None),
        ("foo(condition ? bar(baz(qux())) : zed());", None),
        ("foo(class {field = bar(baz(qux(zed())));});", None),
        ("<Component value={foo(bar(baz(qux())))} />;", None),
        (
            "mergeReports(await pMap(
                await mergeWithFileConfigs(uniq(paths), inputOptions, configFiles),
                async ({files, options, prettierOptions}) => runEslint(files, buildConfig(options, prettierOptions), {isQuiet: options.quiet}),
            ));",
            None,
        ),
    ];

    Tester::new(MaxNestedCalls::NAME, MaxNestedCalls::PLUGIN, pass, fail).test_and_snapshot();
}
