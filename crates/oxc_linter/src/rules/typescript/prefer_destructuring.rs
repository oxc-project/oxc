use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn prefer_destructuring_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong.")
        .with_help("Should be a command-like statement that tells the user how to fix the issue.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferDestructuring;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// FIXME: Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// FIXME: Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// FIXME: Add at least one example of code that violates the rule.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// FIXME: Add at least one example of code that is allowed with the rule.
    /// ```
    PreferDestructuring,
    typescript,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    pending, // TODO: describe fix capabilities. Remove or set to `none` if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
    version = "next",
    short_description = "FIXME: One-sentence description of the rule.",
);

impl Rule for PreferDestructuring {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
                  declare const object: { foo: string };
                  var foo: string = object.foo;
                ",
            None,
        ),
        (
            "
                  declare const array: number[];
                  const bar: number = array[0];
                ",
            None,
        ),
        (
            "
                    declare const object: { foo: string };
                    var { foo } = object;
                  ",
            Some(
                serde_json::json!([ { "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                    declare const object: { foo: string };
                    var { foo }: { foo: number } = object;
                  ",
            Some(
                serde_json::json!([ { "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                    declare const array: number[];
                    var [foo] = array;
                  ",
            Some(
                serde_json::json!([ { "array": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                    declare const array: number[];
                    var [foo]: [foo: number] = array;
                  ",
            Some(
                serde_json::json!([ { "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                    declare const object: { bar: string };
                    var foo: unknown = object.bar;
                  ",
            Some(
                serde_json::json!([ { "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                    declare const object: { foo: string };
                    var { foo: bar } = object;
                  ",
            Some(
                serde_json::json!([ { "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                    declare const object: { foo: boolean };
                    var { foo: bar }: { foo: boolean } = object;
                  ",
            Some(
                serde_json::json!([ { "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                    declare class Foo {
                      foo: string;
                    }
            
                    class Bar extends Foo {
                      static foo() {
                        var foo: any = super.foo;
                      }
                    }
                  ",
            Some(
                serde_json::json!([ { "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                  let x: { 0: unknown };
                  let y = x[0];
                ",
            None,
        ),
        (
            "
                  let x: { 0: unknown };
                  y = x[0];
                ",
            None,
        ),
        (
            "
                  let x: unknown;
                  let y = x[0];
                ",
            None,
        ),
        (
            "
                  let x: unknown;
                  y = x[0];
                ",
            None,
        ),
        (
            "
                  let x: { 0: unknown } | unknown[];
                  let y = x[0];
                ",
            None,
        ),
        (
            "
                  let x: { 0: unknown } | unknown[];
                  y = x[0];
                ",
            None,
        ),
        (
            "
                  let x: { 0: unknown } & (() => void);
                  let y = x[0];
                ",
            None,
        ),
        (
            "
                  let x: { 0: unknown } & (() => void);
                  y = x[0];
                ",
            None,
        ),
        (
            "
                  let x: Record<number, unknown>;
                  let y = x[0];
                ",
            None,
        ),
        (
            "
                  let x: Record<number, unknown>;
                  y = x[0];
                ",
            None,
        ),
        (
            "
                    let x: { 0: unknown };
                    let { 0: y } = x;
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: { 0: unknown };
                    ({ 0: y } = x);
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: { 0: unknown };
                    let y = x[0];
                  ",
            Some(serde_json::json!([{ "array": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "
                    let x: { 0: unknown };
                    y = x[0];
                  ",
            Some(serde_json::json!([{ "array": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "
                    let x: { 0: unknown };
                    let y = x[0];
                  ",
            Some(
                serde_json::json!([ { "AssignmentExpression": { "array": true, "object": true }, "VariableDeclarator": { "array": true, "object": false }, }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: { 0: unknown };
                    y = x[0];
                  ",
            Some(
                serde_json::json!([ { "AssignmentExpression": { "array": true, "object": false }, "VariableDeclarator": { "array": true, "object": true }, }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: Record<number, unknown>;
                    let i: number = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": false }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: Record<number, unknown>;
                    let i: 0 = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": false }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: Record<number, unknown>;
                    let i: 0 | 1 | 2 = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": false }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: unknown[];
                    let i: number = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": false }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: unknown[];
                    let i: 0 = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": false }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: unknown[];
                    let i: 0 | 1 | 2 = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": false }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: unknown[];
                    let i: number = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": false }, ]),
            ),
        ),
        (
            "
                    let x: { 0: unknown };
                    y += x[0];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    class Bar {
                      public [0]: unknown;
                    }
                    class Foo extends Bar {
                      static foo() {
                        let y = super[0];
                      }
                    }
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    class Bar {
                      public [0]: unknown;
                    }
                    class Foo extends Bar {
                      static foo() {
                        y = super[0];
                      }
                    }
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                  let xs: unknown[] = [1];
                  let [x] = xs;
                ",
            None,
        ),
        (
            "
                  const obj: { x: unknown } = { x: 1 };
                  const { x } = obj;
                ",
            None,
        ),
        (
            "
                  var obj: { x: unknown } = { x: 1 };
                  var { x: y } = obj;
                ",
            None,
        ),
        (
            "
                  let obj: { x: unknown } = { x: 1 };
                  let key: 'x' = 'x';
                  let { [key]: foo } = obj;
                ",
            None,
        ),
        (
            "
                  const obj: { x: unknown } = { x: 1 };
                  let x: unknown;
                  ({ x } = obj);
                ",
            None,
        ),
        (
            "
                  let obj: { x: unknown } = { x: 1 };
                  let y = obj.x;
                ",
            None,
        ),
        (
            "
                  var obj: { x: unknown } = { x: 1 };
                  var y: unknown;
                  y = obj.x;
                ",
            None,
        ),
        (
            "
                  const obj: { x: unknown } = { x: 1 };
                  const y = obj['x'];
                ",
            None,
        ),
        (
            "
                  let obj: Record<string, unknown> = {};
                  let key = 'abc';
                  var y = obj[key];
                ",
            None,
        ),
        (
            "
                  let obj: { x: number } = { x: 1 };
                  let x = 10;
                  x += obj.x;
                ",
            None,
        ),
        (
            "
                  let obj: { x: boolean } = { x: false };
                  let x = true;
                  x ||= obj.x;
                ",
            None,
        ),
        (
            "
                  const xs: number[] = [1];
                  let x = 3;
                  x *= xs[0];
                ",
            None,
        ),
        (
            "
                  let xs: unknown[] | undefined;
                  let x = xs?.[0];
                ",
            None,
        ),
        (
            "
                  let obj: Record<string, unknown> | undefined;
                  let x = obj?.x;
                ",
            None,
        ),
        (
            "
                  class C {
                    #foo: string;
            
                    method() {
                      const foo: unknown = this.#foo;
                    }
                  }
                ",
            None,
        ),
        (
            "
                  class C {
                    #foo: string;
            
                    method() {
                      let foo: unknown;
                      foo = this.#foo;
                    }
                  }
                ",
            None,
        ),
        (
            "
                    class C {
                      #foo: string;
            
                      method() {
                        const bar: unknown = this.#foo;
                      }
                    }
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                    class C {
                      #foo: string;
            
                      method(another: C) {
                        let bar: unknown;
                        bar: unknown = another.#foo;
                      }
                    }
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "
                    class C {
                      #foo: string;
            
                      method() {
                        const foo: unknown = this.#foo;
                      }
                    }
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
    ];

    let fail = vec![
        (
            "var foo: string = object.foo;",
            Some(
                serde_json::json!([ { "object": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "var foo: string = array[0];",
            Some(
                serde_json::json!([ { "array": true }, { "enforceForDeclarationWithTypeAnnotation": true }, ]),
            ),
        ),
        (
            "var foo: unknown = object.bar;",
            Some(
                serde_json::json!([ { "object": true }, { "enforceForDeclarationWithTypeAnnotation": true, "enforceForRenamedProperties": true, }, ]),
            ),
        ),
        (
            "
                    let x: { [Symbol.iterator]: unknown };
                    let y = x[0];
                  ",
            None,
        ),
        (
            "
                    let x: { [Symbol.iterator]: unknown };
                    y = x[0];
                  ",
            None,
        ),
        (
            "
                    let x: [1, 2, 3];
                    let y = x[0];
                  ",
            None,
        ),
        (
            "
                    let x: [1, 2, 3];
                    y = x[0];
                  ",
            None,
        ),
        (
            "
                    function* it() {
                      yield 1;
                    }
                    let y = it()[0];
                  ",
            None,
        ),
        (
            "
                    function* it() {
                      yield 1;
                    }
                    y = it()[0];
                  ",
            None,
        ),
        (
            "
                    let x: any;
                    let y = x[0];
                  ",
            None,
        ),
        (
            "
                    let x: any;
                    y = x[0];
                  ",
            None,
        ),
        (
            "
                    let x: string[] | { [Symbol.iterator]: unknown };
                    let y = x[0];
                  ",
            None,
        ),
        (
            "
                    let x: string[] | { [Symbol.iterator]: unknown };
                    y = x[0];
                  ",
            None,
        ),
        (
            "
                    let x: object & unknown[];
                    let y = x[0];
                  ",
            None,
        ),
        (
            "
                    let x: object & unknown[];
                    y = x[0];
                  ",
            None,
        ),
        (
            "
                    let x: { 0: string };
                    let y = x[0];
                  ",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "
                    let x: { 0: string };
                    y = x[0];
                  ",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "
                    let x: { 0: string };
                    let y = x[0];
                  ",
            Some(
                serde_json::json!([ { "AssignmentExpression": { "array": false, "object": false }, "VariableDeclarator": { "array": false, "object": true }, }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: { 0: string };
                    y = x[0];
                  ",
            Some(
                serde_json::json!([ { "AssignmentExpression": { "array": false, "object": true }, "VariableDeclarator": { "array": false, "object": false }, }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: Record<number, unknown>;
                    let i: number = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: Record<number, unknown>;
                    let i: 0 = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: Record<number, unknown>;
                    let i: 0 | 1 | 2 = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: unknown[];
                    let i: number = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: unknown[];
                    let i: 0 = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: unknown[];
                    let i: 0 | 1 | 2 = 0;
                    y = x[i];
                  ",
            Some(
                serde_json::json!([ { "array": true, "object": true }, { "enforceForRenamedProperties": true }, ]),
            ),
        ),
        (
            "
                    let x: { 0: unknown } | unknown[];
                    let y = x[0];
                  ",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "
                    let x: { 0: unknown } | unknown[];
                    y = x[0];
                  ",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "
                    let obj = { foo: 'bar' };
                    const foo = obj.foo;
                  ",
            None,
        ),
        (
            "
                    let obj = { foo: 'bar' };
                    var x: null = null;
                    const foo = (x, obj).foo;
                  ",
            None,
        ),
        ("const call = (() => null).call;", None),
        (
            "
                    const obj = { foo: 'bar' };
                    let a: any;
                    var foo = (a = obj).foo;
                  ",
            None,
        ),
        (
            "
                    const obj = { asdf: { qwer: null } };
                    const qwer = obj.asdf.qwer;
                  ",
            None,
        ),
        (
            "
                    const obj = { foo: 100 };
                    const /* comment */ foo = obj.foo;
                  ",
            None,
        ),
        (
            "
                    let obj = { foo: 'bar' };
                    const x = obj.foo;
                  ",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "
                    let obj = { foo: 'bar' };
                    let x: unknown;
                    x = obj.foo;
                  ",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "
                    let obj: Record<string, unknown>;
                    let key = 'abc';
                    const x = obj[key];
                  ",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
        (
            "
                    let obj: Record<string, unknown>;
                    let key = 'abc';
                    let x: unknown;
                    x = obj[key];
                  ",
            Some(serde_json::json!([{ "object": true }, { "enforceForRenamedProperties": true }])),
        ),
    ];

    let fix = vec![
        (
            "
                    let obj = { foo: 'bar' };
                    const foo = obj.foo;
                  ",
            "
                    let obj = { foo: 'bar' };
                    const {foo} = obj;
                  ",
        ),
        (
            "
                    let obj = { foo: 'bar' };
                    var x: null = null;
                    const foo = (x, obj).foo;
                  ",
            "
                    let obj = { foo: 'bar' };
                    var x: null = null;
                    const {foo} = (x, obj);
                  ",
        ),
        ("const call = (() => null).call;", "const {call} = () => null;"),
        (
            "
                    const obj = { foo: 'bar' };
                    let a: any;
                    var foo = (a = obj).foo;
                  ",
            "
                    const obj = { foo: 'bar' };
                    let a: any;
                    var {foo} = a = obj;
                  ",
        ),
        (
            "
                    const obj = { asdf: { qwer: null } };
                    const qwer = obj.asdf.qwer;
                  ",
            "
                    const obj = { asdf: { qwer: null } };
                    const {qwer} = obj.asdf;
                  ",
        ),
        (
            "
                    const obj = { foo: 100 };
                    const /* comment */ foo = obj.foo;
                  ",
            "
                    const obj = { foo: 100 };
                    const /* comment */ {foo} = obj;
                  ",
        ),
    ];

    Tester::new(PreferDestructuring::NAME, PreferDestructuring::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
