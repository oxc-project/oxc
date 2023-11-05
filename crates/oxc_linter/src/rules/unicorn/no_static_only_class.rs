use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error(
    "eslint-plugin-unicorn(no-static-only-class): Disallow classes that only have static members."
)]
#[diagnostic(
    severity(warning),
    help("Change the class to an object instead, or add some instance members.")
)]
struct NoStaticOnlyClassDiagnostic(#[label] pub Span);

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
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// class A {
    ///     static a() {}
    /// }
    ///
    /// // Good
    /// class A {
    ///     static a() {}
    ///
    ///     constructor() {}
    /// }
    /// ```
    NoStaticOnlyClass,
    pedantic
);

impl Rule for NoStaticOnlyClass {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Class(class) = node.kind() else { return };

        if class.super_class.is_some() {
            return;
        }
        if class.decorators.len() > 0 {
            return;
        }
        if class.body.body.len() == 0 {
            return;
        }
        if class.body.body.iter().any(|node| {
            match node {
                oxc_ast::ast::ClassElement::MethodDefinition(v) => {
                    if v.accessibility.is_some() {
                        return true;
                    }
                }
                oxc_ast::ast::ClassElement::PropertyDefinition(v) => {
                    if v.accessibility.is_some() || v.readonly || v.declare {
                        return true;
                    }
                }
                oxc_ast::ast::ClassElement::TSAbstractMethodDefinition(v) => {
                    if v.method_definition.accessibility.is_some() {
                        return true;
                    }
                }
                oxc_ast::ast::ClassElement::TSAbstractPropertyDefinition(v) => {
                    if v.property_definition.accessibility.is_some() {
                        return true;
                    }
                }
                oxc_ast::ast::ClassElement::AccessorProperty(_)
                | oxc_ast::ast::ClassElement::StaticBlock(_)
                | oxc_ast::ast::ClassElement::TSIndexSignature(_) => {}
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

        ctx.diagnostic(NoStaticOnlyClassDiagnostic(class.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"class A {}"#,
        r#"const A = class {}"#,
        r#"class A extends B { static a() {}; }"#,
        r#"const A = class extends B { static a() {}; }"#,
        r#"class A { a() {} }"#,
        r#"class A { constructor() {} }"#,
        r#"class A { get a() {} }"#,
        r#"class A { set a(value) {} }"#,
        r#"class A3 { static #a() {}; }"#,
        r#"class A3 { static #a = 1; }"#,
        r#"const A3 = class { static #a() {}; }"#,
        r#"const A3 = class { static #a = 1; }"#,
        r#"class A2 { static {}; }"#,
        r#"class A { static #a() {}; }"#,
        r#"class A { static #a = 1; }"#,
        r#"const A = class { static #a() {}; }"#,
        r#"const A = class { static #a = 1; }"#,
        r#"@decorator class A { static  a = 1; }"#,
        r#"class A { static public a = 1; }"#,
        r#"class A { static private a = 1; }"#,
        r#"class A { static readonly a = 1; }"#,
        r#"class A { static declare a = 1; }"#,
        r#"class A { static {}; }"#,
        r#"class A2 { static #a() {}; }"#,
        r#"class A2 { static #a = 1; }"#,
        r#"const A2 = class { static #a() {}; }"#,
        r#"const A2 = class { static #a = 1; }"#,
        r#"class A2 { static {}; }"#,
    ];

    let fail = vec![
        r#"class A { static a() {}; }"#,
        r#"class A { static a() {} }"#,
        r#"const A = class A { static a() {}; }"#,
        r#"const A = class { static a() {}; }"#,
        r#"class A { static constructor() {}; }"#,
        r#"export default class A { static a() {}; }"#,
        r#"export default class { static a() {}; }"#,
        r#"export class A { static a() {}; }"#,
        r#"class A {static [this.a] = 1}"#,
        r#"class A { static a() {} }"#,
    ];

    Tester::new_without_config(NoStaticOnlyClass::NAME, pass, fail).test_and_snapshot();
}
