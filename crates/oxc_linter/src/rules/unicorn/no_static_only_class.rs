use oxc_ast::{AstKind, ast::ClassElement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_static_only_class_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use an object instead of a class with only static members.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoStaticOnlyClass;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow classes that only have static members.
    ///
    /// ### Why is this bad?
    ///
    /// A class with only static members could just be an object instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// class A {
    ///     static a() {}
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// class A {
    ///     static a() {}
    ///
    ///     constructor() {}
    /// }
    /// ```
    /// ```javascript
    /// const X = {
    ///     foo: false,
    ///     bar() {}
    /// };
    /// ```
    /// ```javascript
    /// class X {
    ///     static #foo = false; // private field
    ///     static bar() {}
    /// }
    /// ```
    NoStaticOnlyClass,
    unicorn,
    pedantic,
    pending
);

impl Rule for NoStaticOnlyClass {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Class(class) = node.kind() else {
            return;
        };

        if class.super_class.is_some() {
            return;
        }
        if !class.decorators.is_empty() {
            return;
        }
        if class.body.body.is_empty() {
            return;
        }
        if class.body.body.iter().any(|node| {
            match node {
                ClassElement::MethodDefinition(v) => {
                    if v.accessibility.is_some() {
                        return true;
                    }
                }
                ClassElement::PropertyDefinition(v) => {
                    if v.accessibility.is_some() || v.readonly || v.declare {
                        return true;
                    }
                }
                ClassElement::AccessorProperty(_)
                | ClassElement::StaticBlock(_)
                | ClassElement::TSIndexSignature(_) => {}
            }

            if node.r#static() {
                if let Some(k) = node.property_key() {
                    return k.is_private_identifier();
                }
                return false;
            }
            true
        }) {
            return;
        }

        ctx.diagnostic(no_static_only_class_diagnostic(class.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"class A {}",
        r"const A = class {}",
        r"class A extends B { static a() {}; }",
        r"const A = class extends B { static a() {}; }",
        r"class A { a() {} }",
        r"class A { constructor() {} }",
        r"class A { get a() {} }",
        r"class A { set a(value) {} }",
        r"class A3 { static #a() {}; }",
        r"class A3 { static #a = 1; }",
        r"const A3 = class { static #a() {}; }",
        r"const A3 = class { static #a = 1; }",
        r"class A2 { static {}; }",
        r"class A { static #a() {}; }",
        r"class A { static #a = 1; }",
        r"const A = class { static #a() {}; }",
        r"const A = class { static #a = 1; }",
        r"@decorator class A { static  a = 1; }",
        r"class A { static public a = 1; }",
        r"class A { static private a = 1; }",
        r"class A { static readonly a = 1; }",
        r"class A { static declare a = 1; }",
        r"class A { static {}; }",
        r"class A2 { static #a() {}; }",
        r"class A2 { static #a = 1; }",
        r"const A2 = class { static #a() {}; }",
        r"const A2 = class { static #a = 1; }",
        r"class A2 { static {}; }",
    ];

    let fail = vec![
        r"class A { static a() {}; }",
        r"class A { static a() {} }",
        r"const A = class A { static a() {}; }",
        r"const A = class { static a() {}; }",
        r"class A { static constructor() {}; }",
        r"export default class A { static a() {}; }",
        r"export default class { static a() {}; }",
        r"export class A { static a() {}; }",
        r"class A {static [this.a] = 1}",
        r"class A { static a() {} }",
    ];

    Tester::new(NoStaticOnlyClass::NAME, NoStaticOnlyClass::PLUGIN, pass, fail).test_and_snapshot();
}
