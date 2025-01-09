use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::FxHashMap;

use crate::{context::LintContext, rule::Rule};

fn no_dupe_class_members_diagnostic(
    member_name: &str, /*Class member name */
    decl_span: Span,
    re_decl_span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Duplicate class member: {member_name:?}"))
        .with_help("The last declaration overwrites previous ones, remove one of them or rename if both should be retained")
        .with_labels([
            decl_span.label(format!("{member_name:?} is previously declared here")),
            re_decl_span.label(format!("{member_name:?} is re-declared here")),
        ])
}

#[derive(Debug, Default, Clone)]
pub struct NoDupeClassMembers;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow duplicate class members
    ///
    /// ### Why is this bad?
    /// If there are declarations of the same name in class members,
    /// the last declaration overwrites other declarations silently. It can cause unexpected behaviors.
    ///
    /// ### Example
    /// ```javascript
    /// class A {
    ///   foo() { console.log("foo") }
    ///   foo = 123;
    /// }
    /// let a = new A();
    /// a.foo() // Uncaught TypeError: a.foo is not a function
    /// ```
    NoDupeClassMembers,
    eslint,
    correctness
);

impl Rule for NoDupeClassMembers {
    fn run_once(&self, ctx: &LintContext) {
        ctx.semantic().classes().iter_enumerated().for_each(|(class_id, _)| {
            let mut defined_elements = FxHashMap::default();
            let elements = &ctx.semantic().classes().elements[class_id];
            for (element_id, element) in elements.iter_enumerated() {
                if let Some(prev_element_id) = defined_elements.insert(&element.name, element_id) {
                    let prev_element = &elements[prev_element_id];
                    if element.r#static == prev_element.r#static
                        && element.is_private == prev_element.is_private
                        && (!(element.kind.is_setter_or_getter()
                            && prev_element.kind.is_setter_or_getter())
                            || element.kind == prev_element.kind)
                    {
                        ctx.diagnostic(no_dupe_class_members_diagnostic(
                            &element.name,
                            prev_element.span,
                            element.span,
                        ));
                    }
                }
            }
        });
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "class A { foo() {} bar() {} }",
        "class A { static foo() {} foo() {} }",
        "class A { get foo() {} set foo(value) {} }",
        "class A { static foo() {} get foo() {} set foo(value) {} }",
        "class A { foo() { } } class B { foo() { } }",
        "class A { [foo]() {} foo() {} }",
        "class A { 'foo'() {} 'bar'() {} baz() {} }",
        "class A { *'foo'() {} *'bar'() {} *baz() {} }",
        "class A { get 'foo'() {} get 'bar'() {} get baz() {} }",
        "class A { 1() {} 2() {} }",
        "class A { ['foo']() {} ['bar']() {} }",
        "class A { [`foo`]() {} [`bar`]() {} }",
        "class A { [12]() {} [123]() {} }",
        "class A { [1.0]() {} ['1.0']() {} }",
        "class A { [0x1]() {} [`0x1`]() {} }",
        "class A { [null]() {} ['']() {} }",
        "class A { get ['foo']() {} set ['foo'](value) {} }",
        "class A { ['foo']() {} static ['foo']() {} }",
        // computed "constructor" key doesn't create constructor
        "class A { ['constructor']() {} constructor() {} }",
        "class A { 'constructor'() {} [`constructor`]() {} }",
        "class A { constructor() {} get [`constructor`]() {} }",
        "class A { 'constructor'() {} set ['constructor'](value) {} }",
        // not assumed to be statically-known values
        "class A { ['foo' + '']() {} ['foo']() {} }",
        "class A { [`foo${''}`]() {} [`foo`]() {} }",
        "class A { [-1]() {} ['-1']() {} }",
        // not supported by this rule
        "class A { [foo]() {} [foo]() {} }",
        // private and public
        "class A { foo; static foo; }",
        "class A { static foo() {}; foo() {}; }",
        "class A { foo; #foo; }",
        "class A { '#foo'; #foo; }",
        // typescript-eslint
        "class A { foo() {} bar() {} }",
        "class A { static foo() {} foo() {} }",
        "class A { get foo() {} set foo(value) {} }",
        "class A { static foo() {} get foo() {} set foo(value) {} }",
        "class A { foo() {} } class B { foo() {} }",
        "class A { [foo]() {} foo() {} } ",
        "class A { foo() {} bar() {} baz() {} }",
        "class A { *foo() {} *bar() {} *baz() {} }",
        "class A { get foo() {} get bar() {} get baz() {} }",
        "class A { 1() {} 2() {} }",
        "class Foo { foo(a: string): string; foo(a: number): number; foo(a: any): any {} }",
        // NOTE: This should fail when we get read the big int value
        "class A { [123n]() {} 123() {} }",
    ];

    let fail = vec![
        "class A { foo() {} foo() {} }",
        "!class A { foo() {} foo() {} };",
        "class A { 'foo'() {} 'foo'() {} }",
        "class A { 10() {} 1e1() {} }",
        "class A { ['foo']() {} ['foo']() {} }",
        "class A { static ['foo']() {} static foo() {} }",
        "class A { set 'foo'(value) {} set ['foo'](val) {} }",
        "class A { ''() {} ['']() {} }",
        "class A { [`foo`]() {} [`foo`]() {} }",
        "class A { static get [`foo`]() {} static get ['foo']() {} }",
        "class A { foo() {} [`foo`]() {} }",
        "class A { get [`foo`]() {} 'foo'() {} }",
        "class A { static 'foo'() {} static [`foo`]() {} }",
        "class A { ['constructor']() {} ['constructor']() {} }",
        "class A { static [`constructor`]() {} static constructor() {} }",
        "class A { static constructor() {} static 'constructor'() {} }",
        "class A { [123]() {} [123]() {} }",
        "class A { [0x10]() {} 16() {} }",
        "class A { [100]() {} [1e2]() {} }",
        "class A { [123.00]() {} [`123`]() {} }",
        "class A { static '65'() {} static [0o101]() {} }",
        "class A { [null]() {} 'null'() {} }",
        "class A { foo() {} foo() {} foo() {} }",
        "class A { static foo() {} static foo() {} }",
        "class A { foo() {} get foo() {} }",
        "class A { set foo(value) {} foo() {} }",
        "class A { foo; foo; }",
        // typescript-eslint
        "class A { foo() {}  foo() {}}",
        "!class A { foo() {}  foo() {}};",
        "class A { 'foo'() {}  'foo'() {}}",
        "class A { 10() {}  1e1() {}}",
        "class A { foo() {}  foo() {}  foo() {}}",
        "class A { static foo() {}  static foo() {}}",
        "class A { foo() {}  get foo() {}}",
        "class A { set foo(value) {}  foo() {}}",
        "class A { foo;  foo = 42;}",
        "class A { foo;  foo() {}}",
    ];

    Tester::new(NoDupeClassMembers::NAME, NoDupeClassMembers::PLUGIN, pass, fail)
        .test_and_snapshot();
}
