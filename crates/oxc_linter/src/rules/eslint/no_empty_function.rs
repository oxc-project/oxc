use std::borrow::Cow;

use oxc_ast::{
    ast::{IdentifierName, IdentifierReference, MethodDefinitionKind},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_empty_function_diagnostic<S: AsRef<str>>(
    span: Span,
    fn_kind: &str,
    fn_name: Option<S>,
) -> OxcDiagnostic {
    let message = match fn_name {
        Some(name) => Cow::Owned(format!("Unexpected empty {fn_kind} `{}`", name.as_ref())),
        None => Cow::Borrowed("Unexpected empty function"),
    };
    OxcDiagnostic::warn(message)
        .with_help(format!("Consider removing this {fn_kind} or adding logic to it."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoEmptyFunction;

declare_oxc_lint!(
    /// ### What it does
    /// Disallows the usages of empty functions
    ///
    /// ### Why is this bad?
    /// Empty functions can reduce readability because readers need to guess whether it’s
    /// intentional or not. So writing a clear comment for empty functions is a good practice.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// function foo() {
    /// }
    ///
    /// const bar = () => {};
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// function foo() {
    ///     // do nothing
    /// }
    ///
    /// function foo() {
    ///     return;
    /// }
    /// const add = (a, b) => a + b
    /// ```
    NoEmptyFunction,
    restriction,
);

impl Rule for NoEmptyFunction {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::FunctionBody(fb) = node.kind() else {
            return;
        };
        if fb.is_empty() && !ctx.semantic().has_comments_between(fb.span) {
            let (kind, fn_name) = get_function_name_and_kind(node, ctx);
            ctx.diagnostic(no_empty_function_diagnostic(fb.span, kind, fn_name));
        }
    }
}

fn get_function_name_and_kind<'a>(
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> (&'static str, Option<Cow<'a, str>>) {
    for parent in ctx.nodes().iter_parents(node.id()).skip(1).map(AstNode::kind) {
        match parent {
            AstKind::Function(f) => {
                if let Some(name) = f.name() {
                    let kind = if f.generator { "generator function" } else { "function" };
                    return (kind, Some(name.into()));
                }
                continue;
            }
            AstKind::ArrowFunctionExpression(_) => {
                continue;
            }
            AstKind::IdentifierName(IdentifierName { name, .. })
            | AstKind::IdentifierReference(IdentifierReference { name, .. }) => {
                return ("function", Some(Cow::Borrowed(name.as_str())));
            }
            AstKind::PropertyDefinition(prop) => {
                return ("function", prop.key.name());
            }
            AstKind::MethodDefinition(method) => {
                let kind = match method.kind {
                    MethodDefinitionKind::Method => {
                        if method.r#static {
                            "static method"
                        } else {
                            "method"
                        }
                    }
                    MethodDefinitionKind::Get => "getter",
                    MethodDefinitionKind::Set => "setter",
                    MethodDefinitionKind::Constructor => "constructor",
                };
                return (kind, method.key.name());
            }
            AstKind::VariableDeclarator(decl) => {
                return ("function", decl.id.get_identifier().map(Into::into));
            }
            _ => return ("function", None),
        }
    }
    #[cfg(debug_assertions)]
    unreachable!();
    #[cfg(not(debug_assertions))]
    ("function", None)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "
        function foo() {
            // empty
        }
        ",
        "
        function* baz() {
            // empty
        }
        ",
        "
        const bar = () => {
            // empty
        };
        ",
        "
        const obj = {
            foo: function() {
                // empty
            },
            bar: function*() {
                // empty
            },
            foobar() {
                // empty
            }
        };
        ",
        "
        class A {
            constructor() {
                // empty
            }
            foo() {
                // empty
            }
            *foo1() {
                // empty
            }
            get bar() {
                // empty
            }
            set bar(value) {
                // empty
            }
            static bar() {
                // empty
            }
            static *barr() {
                // empty
            }
            static get baz() {
                // empty
            }
            static set baz(value) {
                // empty
            }
        }
        ",
    ];

    let fail = vec![
        "function foo() {}",
        "const bar = () => {};",
        "function* baz() {}",
        "
        const obj = {
            foo: function() {
            },
            bar: function*() {
            },
            foobar() {
            }
        };
        ",
        "
        class A {
            constructor() {
            }
            foo() {
            }
            *foo1() {
            }
            get fooz() {
            }
            set fooz(value) {
            }
            static bar() {
            }
            static *barr() {
            }
            static get baz() {
            }
            static set baz(value) {
            }
        }
    ",
    ];

    Tester::new(NoEmptyFunction::NAME, pass, fail).test_and_snapshot();
}
