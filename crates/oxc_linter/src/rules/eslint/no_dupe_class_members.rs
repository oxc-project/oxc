use std::hash::BuildHasherDefault;

use oxc_ast::{
    ast::{ClassElement, MethodDefinitionKind},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, GetSpan, Span};
use rustc_hash::FxHashMap;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-dupe-class-members): Duplicate class member: {0:?}")]
#[diagnostic(
    severity(warning),
    help(
        "The last declaration overwrites previous ones, remove one of them or rename if both should be retained"
    )
)]
struct NoDupeClassMembersDiagnostic(
    Atom, /*Class member name */
    #[label("{0:?} is previously declared here")] pub Span,
    #[label("{0:?} is re-declared here")] pub Span,
);

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
    correctness
);

impl Rule for NoDupeClassMembers {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Class(class) = node.kind() else {
            return;
        };

        let num_element = class.body.body.len();
        let mut property_table = PropertyTable::with_capacity(num_element);
        for element in &class.body.body {
            if ctx.source_type().is_typescript() && element.is_ts_empty_body_function() {
                // Skip functions with no function bodies, which are Typescript's overload signatures
                continue;
            }

            if let Some(dup_span) = property_table.insert(element) {
                let property_name = element.property_key().unwrap().static_name().unwrap();
                let cur_span = element.property_key().unwrap().span();
                ctx.diagnostic(NoDupeClassMembersDiagnostic(property_name, dup_span, cur_span));
            }
        }
    }
}

/// (static, name)
type PropertyTableKey = (bool, Atom);
/// (Definition kind, span of last declaration)
type PropertyTableEntry = (Option<MethodDefinitionKind>, Span);
/// Table to track whether a name is defined in static/non-static context as a getter/setter/normal class members
/// Maps (static, name) -> (kind -> span of last declaration)
#[derive(Debug, Clone, Default)]
struct PropertyTable(FxHashMap<PropertyTableKey, Vec<PropertyTableEntry>>);

impl PropertyTable {
    /// Return the last duplicate span if element's property key is duplicate,
    /// otherwise return None and insert the property key into the table.
    pub fn insert(&mut self, element: &ClassElement) -> Option<Span> {
        // It is valid to have a normal method named 'constructor'
        if element.method_definition_kind() == Some(MethodDefinitionKind::Constructor) {
            return None;
        }

        let Some((property_name, property_span)) =
            element.property_key().and_then(|key| key.static_name().map(|name| (name, key.span())))
        else {
            return None;
        };
        let property_kind = element.method_definition_kind();

        let key = (element.r#static(), property_name);
        let entry = self.0.entry(key).or_default();
        for (kind, span) in &*entry {
            if Self::conflict(*kind, property_kind) {
                return Some(*span);
            }
        }

        entry.push((property_kind, property_span));
        None
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(FxHashMap::with_capacity_and_hasher(capacity, BuildHasherDefault::default()))
    }

    fn conflict(kind: Option<MethodDefinitionKind>, other: Option<MethodDefinitionKind>) -> bool {
        // getter and setter can share the same name
        !matches!(
            (kind, other),
            (Some(MethodDefinitionKind::Get), Some(MethodDefinitionKind::Set))
                | (Some(MethodDefinitionKind::Set), Some(MethodDefinitionKind::Get))
        )
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
        "class A { [123n]() {} 123() {} }",
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

    Tester::new(NoDupeClassMembers::NAME, pass, fail).test_and_snapshot();
}
