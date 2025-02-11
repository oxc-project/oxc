use oxc_ast::{ast::{ObjectPropertyKind, PropertyKind}, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use serde_json::Value;

use crate::{
    context::LintContext,
    rule::Rule,
    AstNode,
};

fn grouped_accessor_pairs_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, PartialEq, Clone)]
enum OrderStyle {
    #[default]
    AnyOrder,
    GetBeforeSet,
    SetBeforeGet,
}

impl OrderStyle {
    pub fn from(raw: &str) -> Self {
        match raw {
            "getBeforeSet" => Self::GetBeforeSet,
            "setBeforeGet" => Self::SetBeforeGet,
            _ => Self::AnyOrder,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GroupedAccessorPairs {
    order_style: OrderStyle,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    GroupedAccessorPairs,
    eslint,
    style,
);

impl Rule for GroupedAccessorPairs {
    fn from_configuration(value: Value) -> Self {
        Self {
            order_style: value.get(0).and_then(Value::as_str).map(OrderStyle::from).unwrap_or_default(),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ObjectExpression(obj_expr) => {
                // let mut accessors = vec![];

                for v in &obj_expr.properties {
                    let ObjectPropertyKind::ObjectProperty(obj_prop) = v else {
                        continue;
                    };
                    if obj_prop.kind == PropertyKind::Init {
                        continue;
                    }
                    // let key_name = obj_prop.key.name().unwrap_or_else(|| {
                    //     if let Some(expression) = obj_prop.key.as_expression() {
                    //         return Cow::Borrowed(expression.span().source_text(ctx.source_text()));
                    //     }
                    //     Cow::Borrowed("")
                    // });
                    // let key_name = obj_prop.key.name().unwrap_or_else(|| obj_prop.key.as_expression().unwrap().span().source_text(ctx.source_text())).as_ref();

                }
            },
            _ => {},
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "const boo = {
                set [v](value) {
                    
                }
            }",
            None
        ),
    ];
    let fail = vec![
        (
            "const o = {
                set [a+b](value) {
                    this.val = value;
                },
                get [a+b]() {
                    return this.val;
                },
                set a(value) {
                }
            };",
            None
        )
    ];

//     let pass = vec![
//         ("({})", None),
// ("({ a })", None),
// ("({ a(){}, b(){}, a(){} })", None),
// ("({ a: 1, b: 2 })", None),
// ("({ a, ...b, c: 1 })", None),
// ("({ a, b, ...a })", None),
// ("({ a: 1, [b]: 2, a: 3, [b]: 4 })", None),
// ("({ a: function get(){}, b, a: function set(foo){} })", None),
// ("({ get(){}, a, set(){} })", None),
// ("class A {}", None),
// ("(class { a(){} })", None),
// ("class A { a(){} [b](){} a(){} [b](){} }", None),
// ("(class { a(){} b(){} static a(){} static b(){} })", None),
// ("class A { get(){} a(){} set(){} }", None),
// ("({ get a(){} })", None),
// ("({ set a(foo){} })", None),
// ("({ a: 1, get b(){}, c, ...d })", None),
// ("({ get a(){}, get b(){}, set c(foo){}, set d(foo){} })", None),
// ("({ get a(){}, b: 1, set c(foo){} })", None),
// ("({ set a(foo){}, b: 1, a: 2 })", None),
// ("({ get a(){}, b: 1, a })", None),
// ("({ set a(foo){}, b: 1, a(){} })", None),
// ("({ get a(){}, b: 1, set [a](foo){} })", None),
// ("({ set a(foo){}, b: 1, get 'a '(){} })", None),
// ("({ get a(){}, b: 1, ...a })", None),
// ("({ set a(foo){}, b: 1 }, { get a(){} })", None),
// ("({ get a(){}, b: 1, ...{ set a(foo){} } })", None),
// ("({ set a(foo){}, get b(){} })", Some(serde_json::json!(["getBeforeSet"]))),
// ("({ get a(){}, set b(foo){} })", Some(serde_json::json!(["setBeforeGet"]))),
// ("class A { get a(){} }", None),
// ("(class { set a(foo){} })", None),
// ("class A { static set a(foo){} }", None),
// ("(class { static get a(){} })", None),
// ("class A { a(){} set b(foo){} c(){} }", None),
// ("(class { a(){} get b(){} c(){} })", None),
// ("class A { get a(){} static get b(){} set c(foo){} static set d(bar){} }", None),
// ("(class { get a(){} b(){} a(foo){} })", None),
// ("class A { static set a(foo){} b(){} static a(){} }", None),
// ("(class { get a(){} static b(){} set [a](foo){} })", None),
// ("class A { static set a(foo){} b(){} static get ' a'(){} }", None),
// ("(class { set a(foo){} b(){} static get a(){} })", None),
// ("class A { static set a(foo){} b(){} get a(){} }", None),
// ("(class { get a(){} }, class { b(){} set a(foo){} })", None),
// ("({ get a(){}, set a(foo){} })", None),
// ("({ a: 1, set b(foo){}, get b(){}, c: 2 })", None),
// ("({ get a(){}, set a(foo){}, set b(bar){}, get b(){} })", None),
// ("({ get [a](){}, set [a](foo){} })", None),
// ("({ set a(foo){}, get 'a'(){} })", None),
// ("({ a: 1, b: 2, get a(){}, set a(foo){}, c: 3, a: 4 })", None),
// ("({ get a(){}, set a(foo){}, set b(bar){} })", None),
// ("({ get a(){}, get b(){}, set b(bar){} })", None),
// ("class A { get a(){} set a(foo){} }", None),
// ("(class { set a(foo){} get a(){} })", None),
// ("class A { static set a(foo){} static get a(){} }", None),
// ("(class { static get a(){} static set a(foo){} })", None),
// ("class A { a(){} set b(foo){} get b(){} c(){} get d(){} set d(bar){} }", None),
// ("(class { set a(foo){} get a(){} get b(){} set b(bar){} })", None),
// ("class A { static set [a](foo){} static get [a](){} }", None),
// ("(class { get a(){} set [`a`](foo){} })", None),
// ("class A { static get a(){} static set a(foo){} set a(bar){} static get a(){} }", None),
// ("(class { static get a(){} get a(){} set a(foo){} })", None),
// ("({ get a(){}, set a(foo){} })", Some(serde_json::json!(["anyOrder"]))),
// ("({ set a(foo){}, get a(){} })", Some(serde_json::json!(["anyOrder"]))),
// ("({ get a(){}, set a(foo){} })", Some(serde_json::json!(["getBeforeSet"]))),
// ("({ set a(foo){}, get a(){} })", Some(serde_json::json!(["setBeforeGet"]))),
// ("class A { get a(){} set a(foo){} }", Some(serde_json::json!(["anyOrder"]))),
// ("(class { set a(foo){} get a(){} })", Some(serde_json::json!(["anyOrder"]))),
// ("class A { get a(){} set a(foo){} }", Some(serde_json::json!(["getBeforeSet"]))),
// ("(class { static set a(foo){} static get a(){} })", Some(serde_json::json!(["setBeforeGet"]))),
// ("({ get a(){}, b: 1, get a(){} })", None),
// ("({ set a(foo){}, b: 1, set a(foo){} })", None),
// ("({ get a(){}, b: 1, set a(foo){}, c: 2, get a(){} })", None),
// ("({ set a(foo){}, b: 1, set 'a'(bar){}, c: 2, get a(){} })", None),
// ("class A { get [a](){} b(){} get [a](){} c(){} set [a](foo){} }", None),
// ("(class { static set a(foo){} b(){} static get a(){} static c(){} static set a(bar){} })", None),
// ("class A { get '#abc'(){} b(){} set #abc(foo){} }", None),
// ("class A { get #abc(){} b(){} set '#abc'(foo){} }", None),
// ("class A { set '#abc'(foo){} get #abc(){} }", Some(serde_json::json!(["getBeforeSet"]))),
// ("class A { set #abc(foo){} get '#abc'(){} }", Some(serde_json::json!(["getBeforeSet"])))
//     ];

//     let fail = vec![
//         ("({ get a(){}, b:1, set a(foo){} })", None),
// ("({ set 'abc'(foo){}, b:1, get 'abc'(){} })", None),
// ("({ get [a](){}, b:1, set [a](foo){} })", None),
// ("class A { get abc(){} b(){} set abc(foo){} }", None),
// ("(class { set abc(foo){} b(){} get abc(){} })", None),
// ("class A { static set a(foo){} b(){} static get a(){} }", None),
// ("(class { static get 123(){} b(){} static set 123(foo){} })", None),
// ("class A { static get [a](){} b(){} static set [a](foo){} }", None),
// ("class A { get '#abc'(){} b(){} set '#abc'(foo){} }", None),
// ("class A { get #abc(){} b(){} set #abc(foo){} }", None),
// ("({ set a(foo){}, get a(){} })", Some(serde_json::json!(["getBeforeSet"]))),
// ("({ get 123(){}, set 123(foo){} })", Some(serde_json::json!(["setBeforeGet"]))),
// ("({ get [a](){}, set [a](foo){} })", Some(serde_json::json!(["setBeforeGet"]))),
// ("class A { set abc(foo){} get abc(){} }", Some(serde_json::json!(["getBeforeSet"]))),
// ("(class { get [`abc`](){} set [`abc`](foo){} })", Some(serde_json::json!(["setBeforeGet"]))),
// ("class A { static get a(){} static set a(foo){} }", Some(serde_json::json!(["setBeforeGet"]))),
// ("(class { static set 'abc'(foo){} static get 'abc'(){} })", Some(serde_json::json!(["getBeforeSet"]))),
// ("class A { static set [abc](foo){} static get [abc](){} }", Some(serde_json::json!(["getBeforeSet"]))),
// ("class A { set '#abc'(foo){} get '#abc'(){} }", Some(serde_json::json!(["getBeforeSet"]))),
// ("class A { set #abc(foo){} get #abc(){} }", Some(serde_json::json!(["getBeforeSet"]))),
// ("({ get a(){}, b: 1, set a(foo){} })", Some(serde_json::json!(["anyOrder"]))),
// ("({ get a(){}, b: 1, set a(foo){} })", Some(serde_json::json!(["setBeforeGet"]))),
// ("({ get a(){}, b: 1, set a(foo){} })", Some(serde_json::json!(["getBeforeSet"]))),
// ("class A { set a(foo){} b(){} get a(){} }", Some(serde_json::json!(["getBeforeSet"]))),
// ("(class { static set a(foo){} b(){} static get a(){} })", Some(serde_json::json!(["setBeforeGet"]))),
// ("({ get 'abc'(){}, d(){}, set 'abc'(foo){} })", None),
// ("({ set ''(foo){}, get [''](){} })", Some(serde_json::json!(["getBeforeSet"]))),
// ("class A { set abc(foo){} get 'abc'(){} }", Some(serde_json::json!(["getBeforeSet"]))),
// ("(class { set [`abc`](foo){} get abc(){} })", Some(serde_json::json!(["getBeforeSet"]))),
// ("({ set ['abc'](foo){}, get [`abc`](){} })", Some(serde_json::json!(["getBeforeSet"]))),
// ("({ set 123(foo){}, get [123](){} })", Some(serde_json::json!(["getBeforeSet"]))),
// ("class A { static set '123'(foo){} static get 123(){} }", Some(serde_json::json!(["getBeforeSet"]))),
// ("(class { set [a+b](foo){} get [a+b](){} })", Some(serde_json::json!(["getBeforeSet"]))),
// ("({ set [f(a)](foo){}, get [f(a)](){} })", Some(serde_json::json!(["getBeforeSet"]))),
// ("({ get a(){}, b: 1, set a(foo){}, set c(foo){}, d(){}, get c(){} })", None),
// ("({ get a(){}, set b(foo){}, set a(bar){}, get b(){} })", None),
// ("({ get a(){}, set [a](foo){}, set a(bar){}, get [a](){} })", None),
// ("({ a(){}, set b(foo){}, ...c, get b(){}, set c(bar){}, get c(){} })", Some(serde_json::json!(["getBeforeSet"]))),
// ("({ set [a](foo){}, get [a](){}, set [-a](bar){}, get [-a](){} })", Some(serde_json::json!(["getBeforeSet"]))),
// ("class A { get a(){} constructor (){} set a(foo){} get b(){} static c(){} set b(bar){} }", None),
// ("(class { set a(foo){} static get a(){} get a(){} static set a(bar){} })", None),
// ("class A { get a(){} set a(foo){} static get b(){} static set b(bar){} }", Some(serde_json::json!(["setBeforeGet"]))),
// ("(class { set [a+b](foo){} get [a-b](){} get [a+b](){} set [a-b](bar){} })", None),
// ("({ get a(){}, set a(foo){}, get b(){}, c: function(){}, set b(bar){} })", None),
// ("({ get a(){}, get b(){}, set a(foo){} })", None),
// ("({ set a(foo){}, get [a](){}, get a(){} })", None),
// ("({ set [a](foo){}, set a(bar){}, get [a](){} })", None),
// ("({ get a(){}, set a(foo){}, set b(bar){}, get b(){} })", Some(serde_json::json!(["getBeforeSet"]))),
// ("class A { get a(){} static set b(foo){} static get b(){} set a(foo){} }", None),
// ("(class { static get a(){} set a(foo){} static set a(bar){} })", None),
// ("class A { set a(foo){} get a(){} static get a(){} static set a(bar){} }", Some(serde_json::json!(["setBeforeGet"]))),
// ("({ get a(){}, a: 1, set a(foo){} })", None),
// ("({ a(){}, set a(foo){}, get a(){} })", Some(serde_json::json!(["getBeforeSet"]))),
// ("class A { get a(){} a(){} set a(foo){} }", None),
// ("class A { get a(){} a; set a(foo){} }", None), // { "ecmaVersion": 2022 },
// ("({ get a(){},
// 			    b: 1,
// 			    set a(foo){}
// 			})", None),
// ("class A { static set a(foo){} b(){} static get 
// 			 a(){}
// 			}", None)
//     ];

    Tester::new(GroupedAccessorPairs::NAME, GroupedAccessorPairs::PLUGIN, pass, fail)
        .test_and_snapshot();
}
