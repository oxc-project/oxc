use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn explicit_module_boundary_types_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ExplicitModuleBoundaryTypes;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ExplicitModuleBoundaryTypes,
    typescript,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for ExplicitModuleBoundaryTypes {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("function test(): void { return; }", None),
        ("export function test(): void { return; }", None),
        ("export var fn = function (): number { return 1; };", None),
        ("export var arrowFn = (): string => 'test';", None),
        (
            "
			class Test {
			  constructor(one) {}
			  get prop(one) {
			    return 1;
			  }
			  set prop(one) {}
			  method(one) {
			    return;
			  }
			  arrow = one => 'arrow';
			  abstract abs(one);
			}
            ",
            None,
        ),
        (
            "
			export class Test {
			  constructor(one: string) {}
			  get prop(one: string): void {
			    return 1;
			  }
			  set prop(one: string): void {}
			  method(one: string): void {
			    return;
			  }
			  arrow = (one: string): string => 'arrow';
			  abstract abs(one: string): void;
			}
            ",
            None,
        ),
        (
            "
			export class Test {
			  private constructor(one) {}
			  private get prop(one) {
			    return 1;
			  }
			  private set prop(one) {}
			  private method(one) {
			    return;
			  }
			  private arrow = one => 'arrow';
			  private abstract abs(one);
			}
            ",
            None,
        ),
        (
            "
			export class PrivateProperty {
			  #property = () => null;
			}
			    ",
            None,
        ),
        (
            "
			export class PrivateMethod {
			  #method() {}
			}
			    ",
            None,
        ),
        (
            "
			export class Test {
			  constructor();
			  constructor(value?: string) {
			    console.log(value);
			  }
			}
            ",
            None,
        ),
        (
            "
			declare class MyClass {
			  constructor(options?: MyClass.Options);
			}
			export { MyClass };
            ",
            None,
        ),
        (
            "
			export function test(): void {
			  nested();
			  return;
			
			  function nested() {}
			}
            ",
            None,
        ),
        (
            "
			export function test(): string {
			  const nested = () => 'value';
			  return nested();
			}
            ",
            None,
        ),
        (
            "
			export function test(): string {
			  class Nested {
			    public method() {
			      return 'value';
			    }
			  }
			  return new Nested().method();
			}
            ",
            None,
        ),
        (
            "export var arrowFn: Foo = () => 'test';",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": true, }])),
        ),
        (
            "
			export var funcExpr: Foo = function () {
			  return 'test';
			};
            ",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": true, }])),
        ),
        (
            "const x = (() => {}) as Foo;",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": true }])),
        ),
        (
            "const x = <Foo>(() => {});",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": true }])),
        ),
        (
            "
			export const x = {
			  foo: () => {},
			} as Foo;
            ",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": true }])),
        ),
        (
            "
			export const x = <Foo>{
			  foo: () => {},
			};
            ",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": true }])),
        ),
        (
            "
			export const x: Foo = {
			  foo: () => {},
			};
            ",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": true }])),
        ),
        (
            "
			export const x = {
			  foo: { bar: () => {} },
			} as Foo;
            ",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": true }])),
        ),
        (
            "
			export const x = <Foo>{
			  foo: { bar: () => {} },
			};
            ",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": true }])),
        ),
        (
            "
			export const x: Foo = {
			  foo: { bar: () => {} },
			};
            ",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": true }])),
        ),
        (
            "
			type MethodType = () => void;
			
			export class App {
			  public method: MethodType = () => {};
			}
            ",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": true }])),
        ),
        (
            "
			export const myObj = {
			  set myProp(val: number) {
			    this.myProp = val;
			  },
			};
            ",
            None,
        ),
        (
            "export default () => (): void => {};",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "export default () => function (): void {};",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "export default () => { return (): void => {}; };",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "export default () => { return function (): void {}; };",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "export function fn() { return (): void => {}; }",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "export function fn() { return function (): void {}; }",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "
			export function FunctionDeclaration() {
			  return function FunctionExpression_Within_FunctionDeclaration() {
			    return function FunctionExpression_Within_FunctionExpression() {
			      return () => {
			        // ArrowFunctionExpression_Within_FunctionExpression
			        return () =>
			          // ArrowFunctionExpression_Within_ArrowFunctionExpression
			          (): number =>
			            1; // ArrowFunctionExpression_Within_ArrowFunctionExpression_WithNoBody
			      };
			    };
			  };
			}
            ",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "
			export default () => () => {
			  return (): void => {
			    return;
			  };
			};
            ",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "
			export default () => () => {
			  const foo = 'foo';
			  return (): void => {
			    return;
			  };
			};
            ",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "
			export default () => () => {
			  const foo = () => (): string => 'foo';
			  return (): void => {
			    return;
			  };
			};
            ",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "
			export class Accumulator {
			  private count: number = 0;
			
			  public accumulate(fn: () => number): void {
			    this.count += fn();
			  }
			}
			
			new Accumulator().accumulate(() => 1);
            ",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": true, }])),
        ),
        (
            "
			export const func1 = (value: number) => ({ type: 'X', value }) as const;
			export const func2 = (value: number) => ({ type: 'X', value }) as const;
			export const func3 = (value: number) => x as const;
			export const func4 = (value: number) => x as const;
            ",
            Some(serde_json::json!([{ "allowDirectConstAssertionInArrowFunctions": true, }])),
        ),
        (
            "
			interface R {
			  type: string;
			  value: number;
			}
			
			export const func = (value: number) =>
			  ({ type: 'X', value }) as const satisfies R;
            ",
            Some(serde_json::json!([{ "allowDirectConstAssertionInArrowFunctions": true, }])),
        ),
        (
            "
			interface R {
			  type: string;
			  value: number;
			}
			
			export const func = (value: number) =>
			  ({ type: 'X', value }) as const satisfies R satisfies R;
            ",
            Some(serde_json::json!([{ "allowDirectConstAssertionInArrowFunctions": true, }])),
        ),
        (
            "
			interface R {
			  type: string;
			  value: number;
			}
			
			export const func = (value: number) =>
			  ({ type: 'X', value }) as const satisfies R satisfies R satisfies R;
            ",
            Some(serde_json::json!([{ "allowDirectConstAssertionInArrowFunctions": true, }])),
        ),
        (
            "
			export const func1 = (value: string) => value;
			export const func2 = (value: number) => ({ type: 'X', value });
            ",
            Some(serde_json::json!([{ "allowedNames": ["func1", "func2"], }])),
        ),
        (
            "
			export function func1() {
			  return 0;
			}
			export const foo = {
			  func2() {
			    return 0;
			  },
			};
            ",
            Some(serde_json::json!([{ "allowedNames": ["func1", "func2"], }])),
        ),
        (
            "
			export class Test {
			  get prop() {
			    return 1;
			  }
			  set prop() {}
			  method() {
			    return;
			  }
			  // prettier-ignore
			  'method'() {}
			  ['prop']() {}
			  [`prop`]() {}
			  [null]() {}
			  [`${v}`](): void {}
			
			  foo = () => {
			    bar: 5;
			  };
			}
            ",
            Some(serde_json::json!([{ "allowedNames": ["prop", "method", "null", "foo"], }])),
        ),
        (
            "
			        export function foo(outer: string) {
			          return function (inner: string): void {};
			        }
            ",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true, }])),
        ),
        (
            "
			        export type Ensurer = (blocks: TFBlock[]) => TFBlock[];
			
			        export const myEnsurer: Ensurer = blocks => {
			          return blocks;
			        };
            ",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": true, }])),
        ),
        (
            "
			export const Foo: FC = () => (
			  <div a={e => {}} b={function (e) {}} c={function foo(e) {}}></div>
			);
            ",
            None,
        ), // {        "parserOptions": {          "ecmaFeatures": { "jsx": true },        },      },
        (
            "
			export const Foo: JSX.Element = (
			  <div a={e => {}} b={function (e) {}} c={function foo(e) {}}></div>
			);
            ",
            None,
        ), // {        "parserOptions": {          "ecmaFeatures": { "jsx": true },        },      },
        (
            "
			const test = (): void => {
			  return;
			};
			export default test;
            ",
            None,
        ),
        (
            "
			function test(): void {
			  return;
			}
			export default test;
            ",
            None,
        ),
        (
            "
			const test = (): void => {
			  return;
			};
			export default [test];
            ",
            None,
        ),
        (
            "
			function test(): void {
			  return;
			}
			export default [test];
            ",
            None,
        ),
        (
            "
			const test = (): void => {
			  return;
			};
			export default { test };
            ",
            None,
        ),
        (
            "
			function test(): void {
			  return;
			}
			export default { test };
            ",
            None,
        ),
        (
            "
			const foo = (arg => arg) as Foo;
			export default foo;
            ",
            None,
        ),
        (
            "
			let foo = (arg => arg) as Foo;
			foo = 3;
			export default foo;
            ",
            None,
        ),
        (
            "
			class Foo {
			  bar = (arg: string): string => arg;
			}
			export default { Foo };
            ",
            None,
        ),
        (
            "
			class Foo {
			  bar(): void {
			    return;
			  }
			}
			export default { Foo };
            ",
            None,
        ),
        (
            "
			export class Foo {
			  accessor bar = (): void => {
			    return;
			  };
			}
            ",
            None,
        ),
        (
            "
			export function foo(): (n: number) => string {
			  return n => String(n);
			}
            ",
            None,
        ),
        (
            "
			export const foo = (a: string): ((n: number) => string) => {
			  return function (n) {
			    return String(n);
			  };
			};
            ",
            None,
        ),
        (
            "
			export function a(): void {
			  function b() {}
			  const x = () => {};
			  (function () {});
			
			  function c() {
			    return () => {};
			  }
			
			  return;
			}
            ",
            None,
        ),
        (
            "
			export function a(): void {
			  function b() {
			    function c() {}
			  }
			  const x = () => {
			    return () => 100;
			  };
			  (function () {
			    (function () {});
			  });
			
			  function c() {
			    return () => {
			      (function () {});
			    };
			  }
			
			  return;
			}
            ",
            None,
        ),
        (
            "
			export function a() {
			  return function b(): () => void {
			    return function c() {};
			  };
			}
            ",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "
			export var arrowFn = () => (): void => {};
            ",
            None,
        ),
        (
            "
			export function fn() {
			  return function (): void {};
			}
            ",
            None,
        ),
        (
            "
			export function foo(outer: string) {
			  return function (inner: string): void {};
			}
            ",
            None,
        ),
        (
            "
			export function foo(): unknown {
			  return new Proxy(apiInstance, {
			    get: (target, property) => {
			      // implementation
			    },
			  });
			}
			    ",
            None,
        ),
        (
            "export default (() => true)();",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": false, }])),
        ),
        (
            "export const x = (() => {}) as Foo;",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": false }])),
        ),
        (
            "
			interface Foo {}
			export const x = {
			  foo: () => {},
			} as Foo;
            ",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": false }])),
        ),
        (
            "export function foo(foo: any): void {}",
            Some(serde_json::json!([{ "allowArgumentsExplicitlyTypedAsAny": true }])),
        ),
        (
            "export function foo({ foo }: any): void {}",
            Some(serde_json::json!([{ "allowArgumentsExplicitlyTypedAsAny": true }])),
        ),
        (
            "export function foo([bar]: any): void {}",
            Some(serde_json::json!([{ "allowArgumentsExplicitlyTypedAsAny": true }])),
        ),
        (
            "export function foo(...bar: any): void {}",
            Some(serde_json::json!([{ "allowArgumentsExplicitlyTypedAsAny": true }])),
        ),
        (
            "export function foo(...[a]: any): void {}",
            Some(serde_json::json!([{ "allowArgumentsExplicitlyTypedAsAny": true }])),
        ),
        ("export function foo(arg = 1): void {}", None),
        ("export const foo = (): ((n: number) => string) => n => String(n);", None),
        (
            "
			export function foo(): (n: number) => (m: number) => string {
			  return function (n) {
			    return function (m) {
			      return String(n + m);
			    };
			  };
			}
			    ",
            None,
        ),
        (
            "
			export const foo = (): ((n: number) => (m: number) => string) => n => m =>
			  String(n + m);
			    ",
            None,
        ),
        ("export const bar: () => (n: number) => string = () => n => String(n);", None),
        (
            "
			type Buz = () => (n: number) => string;
			
			export const buz: Buz = () => n => String(n);
			    ",
            None,
        ),
        (
            "
			export abstract class Foo<T> {
			  abstract set value(element: T);
			}
			    ",
            None,
        ),
        (
            "
			export declare class Foo {
			  set time(seconds: number);
			}
			    ",
            None,
        ),
        ("export class A { b = A; }", None),
        (
            "
			interface Foo {
			  f: (x: boolean) => boolean;
			}
			
			export const a: Foo[] = [
			  {
			    f: (x: boolean) => x,
			  },
			];
			    ",
            None,
        ),
        (
            "
			interface Foo {
			  f: (x: boolean) => boolean;
			}
			
			export const a: Foo = {
			  f: (x: boolean) => x,
			};
			    ",
            None,
        ),
        (
            "
			export function test(a: string): string;
			export function test(a: number): number;
			export function test(a: unknown) {
			  return a;
			}
            ",
            Some(serde_json::json!([{ "allowOverloadFunctions": true, }])),
        ),
        (
            "
			export default function test(a: string): string;
			export default function test(a: number): number;
			export default function test(a: unknown) {
			  return a;
			}
            ",
            Some(serde_json::json!([{ "allowOverloadFunctions": true, }])),
        ),
        (
            "
			export default function (a: string): string;
			export default function (a: number): number;
			export default function (a: unknown) {
			  return a;
			}
            ",
            Some(serde_json::json!([{ "allowOverloadFunctions": true, }])),
        ),
        (
            "
			export class Test {
			  test(a: string): string;
			  test(a: number): number;
			  test(a: unknown) {
			    return a;
			  }
			}
            ",
            Some(serde_json::json!([{ "allowOverloadFunctions": true, }])),
        ),
    ];

    let fail = vec![
        ("export function test(a: number, b: number) { return; }", None),
        ("export function test() { return; }", None),
        ("export var fn = function () { return 1; };", None),
        ("export var arrowFn = () => 'test';", None),
        (
            "
			export class Test {
			  constructor() {}
			  get prop() {
			    return 1;
			  }
			  set prop(value) {}
			  method() {
			    return;
			  }
			  arrow = arg => 'arrow';
			  private method() {
			    return;
			  }
			  abstract abs(arg);
			}
            ",
            None,
        ),
        (
            "
			export class Foo {
			  public a = () => {};
			  public b = function () {};
			  public c = function test() {};
			
			  static d = () => {};
			  static e = function () {};
			}
            ",
            None,
        ),
        ("export default () => (true ? () => {} : (): void => {});", None),
        (
            "export var arrowFn = () => 'test';",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": true }])),
        ),
        (
            "export var funcExpr = function () { return 'test'; };",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": true }])),
        ),
        (
            "
			interface Foo {}
			export const x: Foo = {
			  foo: () => {},
			};
            ",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": false }])),
        ),
        (
            "export default () => () => {};",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "export default () => function () {};",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "export default () => { return () => {}; };",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "
			export default () => {
			  return function () {};
			};
            ",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "export function fn() { return () => {}; }",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "export function fn() { return function () {}; }",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "
			export function FunctionDeclaration() {
			  return function FunctionExpression_Within_FunctionDeclaration() {
			    return function FunctionExpression_Within_FunctionExpression() {
			      return () => {
			        // ArrowFunctionExpression_Within_FunctionExpression
			        return () =>
			          // ArrowFunctionExpression_Within_ArrowFunctionExpression
			          () =>
			            1; // ArrowFunctionExpression_Within_ArrowFunctionExpression_WithNoBody
			      };
			    };
			  };
			}
            ",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "
			export default () => () => {
			  return () => {
			    return;
			  };
			};
            ",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "
			export const func1 = (value: number) => ({ type: 'X', value }) as any;
			export const func2 = (value: number) => ({ type: 'X', value }) as Action;
            ",
            Some(serde_json::json!([{ "allowDirectConstAssertionInArrowFunctions": true, }])),
        ),
        (
            "
			export const func = (value: number) => ({ type: 'X', value }) as const;
            ",
            Some(serde_json::json!([{ "allowDirectConstAssertionInArrowFunctions": false, }])),
        ),
        (
            "
			interface R {
			  type: string;
			  value: number;
			}
			
			export const func = (value: number) =>
			  ({ type: 'X', value }) as const satisfies R;
            ",
            Some(serde_json::json!([{ "allowDirectConstAssertionInArrowFunctions": false, }])),
        ),
        (
            "
			export class Test {
			  constructor() {}
			  get prop() {
			    return 1;
			  }
			  set prop() {}
			  method() {
			    return;
			  }
			  arrow = (): string => 'arrow';
			  foo = () => 'bar';
			}
            ",
            Some(serde_json::json!([{ "allowedNames": ["prop"],        },      ])),
        ),
        (
            "
			export class Test {
			  constructor(
			    public foo,
			    private ...bar,
			  ) {}
			}
            ",
            None,
        ),
        (
            "
			export const func1 = (value: number) => value;
			export const func2 = (value: number) => value;
            ",
            Some(serde_json::json!([{ "allowedNames": ["func2"], }])),
        ),
        (
            "
			export function fn(test): string {
			  return '123';
			}
			",
            None,
        ),
        ("export const fn = (one: number, two): string => '123';", None),
        (
            "
			export function foo(outer) {
			  return function (inner) {};
			}
			",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "export const baz = arg => arg as const;",
            Some(serde_json::json!([{ "allowDirectConstAssertionInArrowFunctions": true }])),
        ),
        (
            "
			const foo = arg => arg;
			export default foo;
			",
            None,
        ),
        (
            "
			const foo = arg => arg;
			export = foo;
			",
            None,
        ),
        (
            "
			let foo = (arg: number): number => arg;
			foo = arg => arg;
			export default foo;
			",
            None,
        ),
        (
            "
			const foo = arg => arg;
			export default [foo];
			",
            None,
        ),
        (
            "
			const foo = arg => arg;
			export default { foo };
			",
            None,
        ),
        (
            "
			function foo(arg) {
			  return arg;
			}
			export default foo;
			",
            None,
        ),
        (
            "
			function foo(arg) {
			  return arg;
			}
			export default [foo];
			",
            None,
        ),
        (
            "
			function foo(arg) {
			  return arg;
			}
			export default { foo };
			",
            None,
        ),
        (
            "
			const bar = function foo(arg) {
			  return arg;
			};
			export default { bar };
			",
            None,
        ),
        (
            "
			class Foo {
			  bool(arg) {
			    return arg;
			  }
			}
			export default Foo;
			",
            None,
        ),
        (
            "
			class Foo {
			  bool = arg => {
			    return arg;
			  };
			}
			export default Foo;
			",
            None,
        ),
        (
            "
			class Foo {
			  bool = function (arg) {
			    return arg;
			  };
			}
			export default Foo;
			",
            None,
        ),
        (
            "
			class Foo {
			  accessor bool = arg => {
			    return arg;
			  };
			}
			export default Foo;
			",
            None,
        ),
        (
            "
			class Foo {
			  accessor bool = function (arg) {
			    return arg;
			  };
			}
			export default Foo;
			",
            None,
        ),
        (
            "
			class Foo {
			  bool = function (arg) {
			    return arg;
			  };
			}
			export default [Foo];
			",
            None,
        ),
        (
            "
			let test = arg => argl;
			test = (): void => {
			  return;
			};
			export default test;
			",
            None,
        ),
        (
            "
			let test = arg => argl;
			test = (): void => {
			  return;
			};
			export { test };
			",
            None,
        ),
        (
            "
			export const foo =
			  () =>
			  (a: string): ((n: number) => string) => {
			    return function (n) {
			      return String(n);
			    };
			  };
",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": false }])),
        ),
        (
            "export var arrowFn = () => () => {};",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "
			export function fn() {
			  return function () {};
			}
",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "
			export function foo(outer) {
			  return function (inner): void {};
			}
            ",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "
			export function foo(outer: boolean) {
			  if (outer) {
			    return 'string';
			  }
			  return function (inner): void {};
			}
            ",
            Some(serde_json::json!([{ "allowHigherOrderFunctions": true }])),
        ),
        (
            "
			export function foo({ foo }): void {}
            ",
            None,
        ),
        (
            "
			export function foo([bar]): void {}
            ",
            None,
        ),
        ("export function foo(...bar): void {}", None),
        ("export function foo(...[a]): void {}", None),
        (
            "export function foo(foo: any): void {}",
            Some(serde_json::json!([{ "allowArgumentsExplicitlyTypedAsAny": false }])),
        ),
        (
            "export function foo({ foo }: any): void {}",
            Some(serde_json::json!([{ "allowArgumentsExplicitlyTypedAsAny": false }])),
        ),
        (
            "export function foo([bar]: any): void {}",
            Some(serde_json::json!([{ "allowArgumentsExplicitlyTypedAsAny": false }])),
        ),
        (
            "export function foo(...bar: any): void {}",
            Some(serde_json::json!([{ "allowArgumentsExplicitlyTypedAsAny": false }])),
        ),
        (
            "export function foo(...[a]: any): void {}",
            Some(serde_json::json!([{ "allowArgumentsExplicitlyTypedAsAny": false }])),
        ),
        (
            "
			export function func1() {
			  return 0;
			}
			export const foo = {
			  func2() {
			    return 0;
			  },
			};
            ",
            Some(serde_json::json!([{ "allowedNames": [],        },      ])),
        ),
        (
            "
			export function test(a: string): string;
			export function test(a: number): number;
			export function test(a: unknown) {
			  return a;
			}
            ",
            None,
        ),
        (
            "
			export default function test(a: string): string;
			export default function test(a: number): number;
			export default function test(a: unknown) {
			  return a;
			}
            ",
            None,
        ),
        (
            "
			export default function (a: string): string;
			export default function (a: number): number;
			export default function (a: unknown) {
			  return a;
			}
            ",
            None,
        ),
        (
            "
			export class Test {
			  test(a: string): string;
			  test(a: number): number;
			  test(a: unknown) {
			    return a;
			  }
			}
            ",
            None,
        ),
    ];

    Tester::new(ExplicitModuleBoundaryTypes::NAME, ExplicitModuleBoundaryTypes::PLUGIN, pass, fail)
        .test_and_snapshot();
}
