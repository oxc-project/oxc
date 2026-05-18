use oxc_macros::declare_oxc_lint;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    rules::shared::prefer_mock_return_shorthand::{DOCUMENTATION, run},
};

#[derive(Debug, Default, Clone)]
pub struct PreferMockReturnShorthand;

declare_oxc_lint!(
    PreferMockReturnShorthand,
    jest,
    style,
    fix,
    docs = DOCUMENTATION,
    version = "1.49.0",
);

impl Rule for PreferMockReturnShorthand {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        run(node, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "describe()",
        "it()",
        "describe.skip()",
        "it.skip()",
        "test()",
        "test.skip()",
        "var appliedOnly = describe.only; appliedOnly.apply(describe)",
        "var calledOnly = it.only; calledOnly.call(it)",
        "it.each()()",
        "it.each`table`()",
        "test.each()()",
        "test.each`table`()",
        "test.concurrent()",
        "jest.fn().mockReturnValue(42)",
        "jest.fn(() => Promise.resolve(42))",
        "jest.fn(() => 42)",
        "jest.fn(() => ({}))",
        "aVariable.mockImplementation",
        "aVariable.mockImplementation()",
        "jest.fn().mockImplementation(async () => 1);",
        "jest.fn().mockImplementation(async function () {});",
        "jest.fn().mockImplementation(async function () {
              return 42;
            });",
        "jest.fn().mockImplementation((...args) => console.log(...args));",
        "aVariable.mockImplementation(() => {
              if (true) {
                return 1;
              }
              return 2;
            });",
        "aVariable.mockImplementation(() => value++)",
        "aVariable.mockImplementationOnce(() => --value)",
        "const aValue = 0;
            aVariable.mockImplementation(() => {
              return aValue++;
            });",
        "aVariable.mockImplementation(() => {
              aValue += 1;
              return aValue;
            });",
        "aVariable.mockImplementation(() => {
              aValue++;
              return aValue;
            });",
        "aVariable.mockReturnValue()",
        "aVariable.mockReturnValue(1)",
        r#"aVariable.mockReturnValue("hello world")"#,
        "jest.spyOn(Thingy, 'method').mockImplementation(param => param * 2);",
        "jest.spyOn(Thingy, 'method').mockImplementation(param => true ? param : 0);",
        "aVariable.mockImplementation(() => {
              const value = new Date();
              return Promise.resolve(value);
            });",
        "aVariable.mockImplementation(() => {
              throw new Error('oh noes!');
            });",
        "aVariable.mockImplementation(() => { /* do something */ });",
        "aVariable.mockImplementation(() => {
              const x = 1;
              console.log(x + 2);
            });",
        "aVariable.mockReturnValue(Promise.all([1, 2, 3]));",
        "let currentX = 0;
            jest.spyOn(X, getCount).mockImplementation(() => currentX);
            // stuff happens
            currentX++;
            // more stuff happens",
        "let currentX = 0;
            jest.spyOn(X, getCount).mockImplementation(() => currentX);",
        "let currentX = 0;
            currentX = 0;
            jest.spyOn(X, getCount).mockImplementation(() => currentX);",
        "var currentX = 0;
            currentX = 0;
            jest.spyOn(X, getCount).mockImplementation(() => currentX);",
        "var currentX = 0;
            var currentX = 0;
            jest.spyOn(X, getCount).mockImplementation(() => currentX);",
        "let doSomething = () => {};
            jest.spyOn(X, getCount).mockImplementation(() => doSomething);",
        "let currentX = 0;
            jest.spyOn(X, getCount).mockImplementation(() => {
              currentX += 1;
              return currentX;
            });",
        "const currentX = 0;
            jest.spyOn(X, getCount).mockImplementation(() => {
              console.log('returning', currentX);
              return currentX;
            });",
        "let value = 1;
            jest.fn().mockImplementation(() => ({ value }));",
        "let value = 1;
            aVariable.mockImplementation(() => [value]);",
        "var value = 1;
            aVariable.mockImplementation(() => [0, value, 2]);",
        "let value = 1;
            aVariable.mockImplementation(() => value + 1);",
        "let value = 1;
            aVariable.mockImplementation(() => 1 - value);",
        "var value = 1;
            aVariable.mockImplementation(() => {
              return { value: value + 1 };
            });",
        "var value = 1;
            aVariable.mockImplementation(() => value * value + 1);
            aVariable.mockImplementation(() => 1 + value / 2);
            aVariable.mockImplementation(() => (1 + value) / 2);
            aVariable.mockImplementation(() => {
              return { value: value + 1 };
            });",
        "let value = 1;
            aVariable.mockImplementation(function () {
              return { items: [value] };
            });",
        "let value = 1;
            aVariable.mockImplementation(() => {
              return {
                type: 'object',
                with: { value },
              }
            });",
        "let value = 1;
            jest.fn().mockImplementationOnce(() => {
              return [{
                type: 'object',
                with: [1, 2, value],
              }]
            });",
        "let value = 1;
            jest.fn().mockImplementationOnce(() => {
              return [
                1,
                {type: 'object', with: [1, 2, 3]},
                {type: 'object', with: [1, 2, value]}
              ];
            });",
        "let value = 1;
            jest.fn().mockImplementationOnce(() => {
              return [
                1,
                {type: 'object', with: [1, 3]},
                {type: 'object', with: [1, value]}
              ];
            });",
        "let value = 1;
            aVariable.mockImplementation(() => {
              return {
                type: 'object',
                with: {
                  inner: {
                    value,
                  },
                },
              }
            });",
        "let value = 1;
            aVariable.mockImplementation(() => {
              return {
                type: 'object',
                with: {
                  inner: {
                    items: [1, 2, value],
                  },
                },
              }
            });",
        "let value = 1;
            aVariable.mockImplementation(() => {
              return [{
                type: 'object',
                with: {
                  inner: {
                    items: [1, 2, value],
                  },
                },
              }]
            });",
        "let value = 1;
            aVariable.mockImplementation(() => value & 1);
            aVariable.mockImplementation(() => value | 1);
            aVariable.mockImplementation(() => 1 & value);
            aVariable.mockImplementation(() => 1 | value);",
        "let value = 1;
            aVariable.mockImplementation(() => !value);
            aVariable.mockImplementation(() => ~value);
            aVariable.mockImplementation(() => typeof value);",
        "const mx = 1
            let my = 2;
            aVariable.mockImplementation(() => mx & my);
            aVariable.mockImplementation(() => my | mx);",
        "let value = 1;
            aVariable.mockImplementation(() => value || 0);
            aVariable.mockImplementation(() => 1 && value);
            aVariable.mockImplementation(() => 1 ?? value);
            aVariable.mockImplementation(() => 1 ?? (value && 0));",
        "const mx = 1
            let my = 2;
            aVariable.mockImplementation(() => mx || my);
            aVariable.mockImplementation(() => my && mx);
            aVariable.mockImplementation(() => my ?? mx);
            aVariable.mockImplementation(() => mx ?? (7 && my));",
        "let value = [1];
            aVariable.mockImplementation(() => {
              return [{
                type: 'object',
                with: {
                  inner: {
                    items: [1, 2, ...value],
                  },
                },
              }]
            });",
        "let value = 1;
            aVariable.mockImplementation(() => {
              return [{
                type: 'object',
                with: {
                  inner: {
                    items: [1, 2, ...[value]],
                  },
                },
              }]
            });",
        "let obj = {};
            aVariable.mockImplementation(() => {
              return {
                type: 'object',
                ...obj,
              }
            });",
        "let value = 1;
            aVariable.mockImplementation(function () {
              function mx() {
                return value;
              }
              return mx();
            });",
        "let value = 1;
            jest.fn().mockImplementation(() => new Mx(value));
            jest.fn().mockImplementation(() => new Mx(() => value));
            jest.fn().mockImplementation(() => new Mx(() => { return value }));",
        "let value = 1;
            jest.fn().mockImplementation(() => mx(value));
            jest.fn().mockImplementation(() => mx(value));
            jest.fn().mockImplementation(() => mx?.(value));
            jest.fn().mockImplementation(() => mx(value).my());
            jest.fn().mockImplementation(() => mx(value).my);
            jest.fn().mockImplementation(() => mx.my(value));
            jest.fn().mockImplementation(() => mx?.my(value));
            jest.fn().mockImplementation(() => mx?.my?.(value));
            jest.fn().mockImplementation(() => mx.my?.(value));
            jest.fn().mockImplementation(() => mx().my(value));
            jest.fn().mockImplementation(() => mx()?.my(value));
            jest.fn().mockImplementation(() => mx.my(value));
            jest.fn().mockImplementation(() => mx(value).my(value));
            jest.fn().mockImplementation(() => mx?.(value)?.my?.(value));
            jest.fn().mockImplementation(() => new Mx().add(value));
            jest.fn().mockImplementation(() => {
              return mx([{
                type: 'object',
                with: {
                  inner: {
                    items: [1, 2, value],
                  },
                },
              }])
            });",
        "let propName = 'world';
            aVariable.mockImplementation(() => mx[propName]());
            aVariable.mockImplementation(() => mx[propName]);
            aVariable.mockImplementation(() => ({ [propName]: 1 }));",
        "const x = true;
            let value = 1;
            aVariable.mockImplementation(() => value ? true : false);
            aVariable.mockImplementation(() => x ? value : false);
            aVariable.mockImplementation(() => x ? true : value);
            aVariable.mockImplementation(() => true ? true : value);
            aVariable.mockImplementation(() => true ? true : value ? true : false);
            aVariable.mockImplementation(() => true ? true : true ? value : false);
            aVariable.mockImplementation(() => true ? true : true ? false : value);
            aVariable.mockImplementation(function() {
              if (x) {
                return value;
              } else {
                return 0;
              }
            });",
    ];

    let fail = vec![
        r#"jest.fn().mockImplementation(() => "hello sunshine")"#,
        "jest.fn().mockImplementation(() => ({}))",
        "jest.fn().mockImplementation(() => x)",
        "jest.fn().mockImplementation(() => true ? x : y)",
        r#"jest.fn().mockImplementation(function () {
              return "hello world";
            })"#,
        "jest.fn().mockImplementation(() => console.log(123));",
        r#"jest.fn().mockImplementation(() => "hello world")"#,
        r#"jest.fn().mockImplementation(() => {
              return "hello world";
            })"#,
        r#"aVariable.mockImplementation(() => "hello world")"#,
        r#"aVariable.mockImplementation(() => {
              return "hello world";
            })"#,
        r#"jest.fn().mockImplementationOnce(() => "hello world")"#,
        r#"aVariable.mockImplementationOnce(() => "hello world")"#,
        "aVariable.mockImplementation(() => ({
              target: 'world',
              message: 'hello'
            }))",
        r#"aVariable
              .mockImplementation(() => 42)
              .mockImplementation(async () => 42)
              .mockImplementation(() => Promise.resolve(42))
              .mockReturnValue("hello world")"#,
        r#"aVariable
              .mockImplementationOnce(() => Promise.reject(42))
              .mockImplementation(() => "hello sunshine")
              .mockReturnValueOnce(Promise.reject(42))"#,
        "jest.fn().mockImplementation(() => (input: number | Record<string, number[]>) => typeof input === 'number' ? input.toFixed(2) : JSON.stringify(input))",
        "jest.fn().mockImplementation(() => [], xyz)",
        r#"jest.spyOn(fs, "readFile").mockImplementation(() => new Error("oh noes!"))"#,
        "aVariable.mockImplementation(() => {
              return Promise.resolve(value)
                .then(value => value + 1);
            });",
        "aVariable.mockImplementation(() => {
              return Promise.all([1, 2, 3]);
            });",
        "const currentX = 0;
            jest.spyOn(X, getCount).mockImplementation(() => currentX);",
        "import { currentX } from './elsewhere';
            jest.spyOn(X, getCount).mockImplementation(() => currentX);",
        "const currentX = 0;
            describe('some tests', () => {
              it('works', () => {
                jest.spyOn(X, getCount).mockImplementation(() => currentX);
              });
            });",
        "function doSomething() {};
            jest.spyOn(X, getCount).mockImplementation(() => doSomething);",
        "const doSomething = () => {};
            jest.spyOn(X, getCount).mockImplementation(() => doSomething);",
        "const value = 1;
            aVariable.mockImplementation(() => [value]);",
        "const value = 1;
            aVariable.mockImplementation(() => [0, value, 2]);",
        "const value = 1;
            aVariable.mockImplementation(() => [0,, value, 2]);",
        "const value = 1;
            jest.fn().mockImplementation(() => ({ value }));",
        "const value = 1;
            aVariable.mockImplementation(() => ({ items: [value] }));",
        "const value = 1;
            aVariable.mockImplementation(() => {
              return {
                type: 'object',
                with: { value },
              }
            });",
        "const vX = 1;
            let vY = 1;
            getPoint.mockImplementation(() => vX + vY);
            getPoint.mockImplementation(() => {
              return { x: vX, y: 1 }
            });",
        "const value = 1;
            aVariable.mockImplementation(() => value & 0);
            aVariable.mockImplementation(() => 0 & value);
            aVariable.mockImplementation(() => value | 1);
            aVariable.mockImplementation(() => 1 | value);",
        "const value = 1;
            aVariable.mockImplementation(() => ~value);
            aVariable.mockImplementation(() => !value);",
        "const value = 1;
            aVariable.mockImplementation(() => value + 1);
            aVariable.mockImplementation(() => 1 + value);
            aVariable.mockImplementation(() => value * value + 1);
            aVariable.mockImplementation(() => 1 + value / 2);
            aVariable.mockImplementation(() => (1 + value) / 2);",
        "const value = 1;
            aVariable.mockImplementation(() => {
              return {
                type: 'object',
                with: [1, 2, value],
              }
            });",
        "const obj = {};
            aVariable.mockImplementation(() => {
              return {
                type: 'object',
                ...obj,
              }
            });",
        "const value = 1;
            jest.fn().mockImplementationOnce(() => {
              return [
                1,
                {type: 'object', with: [1, 2, 3]},
                {type: 'object', with: [1, 2, value]}
              ];
            });",
        "const value = 1;
            jest.fn().mockImplementationOnce(() => {
              return [
                1,
                {type: 'object', with: [1, 2, 3]},
                {type: 'object', with: [1, 2, 0 + value]}
              ];
            });",
        "const value = 1;
            aVariable.mockImplementationOnce(() => {
              return {
                type: 'object',
                with: {
                  inner: {
                    value,
                  },
                },
              }
            });",
        "const value = 1;
            aVariable.mockImplementationOnce(() => {
              return {
                type: 'object',
                with: {
                  inner: {
                    ...{ value },
                  },
                },
              }
            });",
        "const value = 1;
            jest.fn().mockImplementation(() => {
              return {
                type: 'object',
                with: {
                  inner: {
                    items: [1, 2, value],
                  },
                },
              }
            });",
        "const value = 1;
            jest.fn().mockImplementation(() => {
              return [{
                type: 'object',
                with: {
                  inner: {
                    items: [1, 2, value],
                  },
                },
              }]
            });",
        "const mx = 1
            let my = 2;
            aVariable.mockImplementation(() => mx || my);
            aVariable.mockImplementation(() => mx || 0);
            aVariable.mockImplementation(() => my && mx);
            aVariable.mockImplementation(() => mx ?? (7 && my));
            aVariable.mockImplementation(() => mx ?? (7 && 0));",
        "const value = 1;
            jest.fn().mockImplementation(() => new Mx(value));
            jest.fn().mockImplementation(() => new Mx(() => value));
            jest.fn().mockImplementation(() => new Mx(() => { return value }));",
        "const value = 1;
            jest.fn().mockImplementation(() => mx(value));
            jest.fn().mockImplementation(() => mx?.(value));
            jest.fn().mockImplementation(() => mx().my());
            jest.fn().mockImplementation(() => mx().my);
            jest.fn().mockImplementation(() => mx.my());
            jest.fn().mockImplementation(() => mx?.my());
            jest.fn().mockImplementation(() => mx.my);
            jest.fn().mockImplementation(() => mx(value).my());
            jest.fn().mockImplementation(() => mx(value)?.my());
            jest.fn().mockImplementation(() => mx(value).my);
            jest.fn().mockImplementation(() => mx.my(value));
            jest.fn().mockImplementation(() => mx().my(value));
            jest.fn().mockImplementation(() => mx.my(value));
            jest.fn().mockImplementation(() => mx.my?.(value));
            jest.fn().mockImplementation(() => mx(value).my(value));
            jest.fn().mockImplementation(() => mx?.(value)?.my?.(value));
            jest.fn().mockImplementation(() => new Mx().add(value));
            jest.fn().mockImplementation(() => {
              return mx([{
                type: 'object',
                with: {
                  inner: {
                    items: [1, 2, value],
                  },
                },
              }])
            });",
        "const propName = 'world';
            aVariable.mockImplementation(() => mx[propName]());
            aVariable.mockImplementation(() => mx[propName]);
            aVariable.mockImplementation(() => ({ [propName]: 1 }));",
        "const x = true;
            let value = 1;
            aVariable.mockImplementation(() => value ? true : false);
            aVariable.mockImplementation(() => x ? true : false);
            aVariable.mockImplementation(() => x ? true : value);
            aVariable.mockImplementation(() => true ? true : value);
            aVariable.mockImplementation(() => true ? true : true ? value : false);
            aVariable.mockImplementation(() => true ? true : true ? x : false);
            aVariable.mockImplementation(() => true ? true : true ? true : false);",
    ];

    let fix = vec![
        (
            r#"jest.fn().mockImplementation(() => "hello sunshine")"#,
            r#"jest.fn().mockReturnValue("hello sunshine")"#,
        ),
        ("jest.fn().mockImplementation(() => ({}))", "jest.fn().mockReturnValue({})"),
        ("jest.fn().mockImplementation(() => x)", "jest.fn().mockReturnValue(x)"),
        (
            "jest.fn().mockImplementation(() => true ? x : y)",
            "jest.fn().mockReturnValue(true ? x : y)",
        ),
        (
            r#"jest.fn().mockImplementation(function () {
              return "hello world";
            })"#,
            r#"jest.fn().mockReturnValue("hello world")"#,
        ),
        (
            r#"jest.fn().mockImplementation(() => "hello world")"#,
            r#"jest.fn().mockReturnValue("hello world")"#,
        ),
        (
            r#"jest.fn().mockImplementation(() => {
              return "hello world";
            })"#,
            r#"jest.fn().mockReturnValue("hello world")"#,
        ),
        (
            r#"aVariable.mockImplementation(() => "hello world")"#,
            r#"aVariable.mockReturnValue("hello world")"#,
        ),
        (
            r#"aVariable.mockImplementation(() => {
              return "hello world";
            })"#,
            r#"aVariable.mockReturnValue("hello world")"#,
        ),
        (
            r#"jest.fn().mockImplementationOnce(() => "hello world")"#,
            r#"jest.fn().mockReturnValueOnce("hello world")"#,
        ),
        (
            r#"aVariable.mockImplementationOnce(() => "hello world")"#,
            r#"aVariable.mockReturnValueOnce("hello world")"#,
        ),
        (
            "aVariable.mockImplementation(() => ({
              target: 'world',
              message: 'hello'
            }))",
            "aVariable.mockReturnValue({
              target: 'world',
              message: 'hello'
            })",
        ),
        ("jest.fn().mockImplementation(() => [], xyz)", "jest.fn().mockReturnValue([], xyz)"),
        (
            "const currentX = 0;
            jest.spyOn(X, getCount).mockImplementation(() => currentX);",
            "const currentX = 0;
            jest.spyOn(X, getCount).mockReturnValue(currentX);",
        ),
        (
            "import { currentX } from './elsewhere';
            jest.spyOn(X, getCount).mockImplementation(() => currentX);",
            "import { currentX } from './elsewhere';
            jest.spyOn(X, getCount).mockReturnValue(currentX);",
        ),
        (
            "const currentX = 0;
            describe('some tests', () => {
              it('works', () => {
                jest.spyOn(X, getCount).mockImplementation(() => currentX);
              });
            });",
            "const currentX = 0;
            describe('some tests', () => {
              it('works', () => {
                jest.spyOn(X, getCount).mockReturnValue(currentX);
              });
            });",
        ),
        (
            "function doSomething() {};
            jest.spyOn(X, getCount).mockImplementation(() => doSomething);",
            "function doSomething() {};
            jest.spyOn(X, getCount).mockReturnValue(doSomething);",
        ),
        (
            "const doSomething = () => {};
            jest.spyOn(X, getCount).mockImplementation(() => doSomething);",
            "const doSomething = () => {};
            jest.spyOn(X, getCount).mockReturnValue(doSomething);",
        ),
        (
            "const value = 1;
            aVariable.mockImplementation(() => [value]);",
            "const value = 1;
            aVariable.mockReturnValue([value]);",
        ),
        (
            "const value = 1;
            aVariable.mockImplementation(() => [0, value, 2]);",
            "const value = 1;
            aVariable.mockReturnValue([0, value, 2]);",
        ),
        (
            "const value = 1;
            aVariable.mockImplementation(() => [0,, value, 2]);",
            "const value = 1;
            aVariable.mockReturnValue([0,, value, 2]);",
        ),
        (
            "const value = 1;
            jest.fn().mockImplementation(() => ({ value }));",
            "const value = 1;
            jest.fn().mockReturnValue({ value });",
        ),
        (
            "const value = 1;
            aVariable.mockImplementation(() => ({ items: [value] }));",
            "const value = 1;
            aVariable.mockReturnValue({ items: [value] });",
        ),
        (
            "const value = 1;
            aVariable.mockImplementation(() => {
              return {
                type: 'object',
                with: { value },
              }
            });",
            "const value = 1;
            aVariable.mockReturnValue({
                type: 'object',
                with: { value },
              });",
        ),
        (
            "const vX = 1;
            let vY = 1;
            getPoint.mockImplementation(() => vX + vY);
            getPoint.mockImplementation(() => {
              return { x: vX, y: 1 }
            });",
            "const vX = 1;
            let vY = 1;
            getPoint.mockImplementation(() => vX + vY);
            getPoint.mockReturnValue({ x: vX, y: 1 });",
        ),
        (
            "const value = 1;
            aVariable.mockImplementation(() => value & 0);
            aVariable.mockImplementation(() => 0 & value);
            aVariable.mockImplementation(() => value | 1);
            aVariable.mockImplementation(() => 1 | value);",
            "const value = 1;
            aVariable.mockReturnValue(value & 0);
            aVariable.mockReturnValue(0 & value);
            aVariable.mockReturnValue(value | 1);
            aVariable.mockReturnValue(1 | value);",
        ),
        (
            "const value = 1;
            aVariable.mockImplementation(() => ~value);
            aVariable.mockImplementation(() => !value);",
            "const value = 1;
            aVariable.mockReturnValue(~value);
            aVariable.mockReturnValue(!value);",
        ),
        (
            "const value = 1;
            aVariable.mockImplementation(() => value + 1);
            aVariable.mockImplementation(() => 1 + value);
            aVariable.mockImplementation(() => value * value + 1);
            aVariable.mockImplementation(() => 1 + value / 2);
            aVariable.mockImplementation(() => (1 + value) / 2);",
            "const value = 1;
            aVariable.mockReturnValue(value + 1);
            aVariable.mockReturnValue(1 + value);
            aVariable.mockReturnValue(value * value + 1);
            aVariable.mockReturnValue(1 + value / 2);
            aVariable.mockReturnValue((1 + value) / 2);",
        ),
        (
            "const value = 1;
            aVariable.mockImplementation(() => {
              return {
                type: 'object',
                with: [1, 2, value],
              }
            });",
            "const value = 1;
            aVariable.mockReturnValue({
                type: 'object',
                with: [1, 2, value],
              });",
        ),
        (
            "const obj = {};
            aVariable.mockImplementation(() => {
              return {
                type: 'object',
                ...obj,
              }
            });",
            "const obj = {};
            aVariable.mockReturnValue({
                type: 'object',
                ...obj,
              });",
        ),
        (
            "const value = 1;
            jest.fn().mockImplementationOnce(() => {
              return [
                1,
                {type: 'object', with: [1, 2, 3]},
                {type: 'object', with: [1, 2, value]}
              ];
            });",
            "const value = 1;
            jest.fn().mockReturnValueOnce([
                1,
                {type: 'object', with: [1, 2, 3]},
                {type: 'object', with: [1, 2, value]}
              ]);",
        ),
        (
            "const value = 1;
            jest.fn().mockImplementationOnce(() => {
              return [
                1,
                {type: 'object', with: [1, 2, 3]},
                {type: 'object', with: [1, 2, 0 + value]}
              ];
            });",
            "const value = 1;
            jest.fn().mockReturnValueOnce([
                1,
                {type: 'object', with: [1, 2, 3]},
                {type: 'object', with: [1, 2, 0 + value]}
              ]);",
        ),
        (
            "const value = 1;
            aVariable.mockImplementationOnce(() => {
              return {
                type: 'object',
                with: {
                  inner: {
                    value,
                  },
                },
              }
            });",
            "const value = 1;
            aVariable.mockReturnValueOnce({
                type: 'object',
                with: {
                  inner: {
                    value,
                  },
                },
              });",
        ),
        (
            "const value = 1;
            aVariable.mockImplementationOnce(() => {
              return {
                type: 'object',
                with: {
                  inner: {
                    ...{ value },
                  },
                },
              }
            });",
            "const value = 1;
            aVariable.mockReturnValueOnce({
                type: 'object',
                with: {
                  inner: {
                    ...{ value },
                  },
                },
              });",
        ),
        (
            "const value = 1;
            jest.fn().mockImplementation(() => {
              return {
                type: 'object',
                with: {
                  inner: {
                    items: [1, 2, value],
                  },
                },
              }
            });",
            "const value = 1;
            jest.fn().mockReturnValue({
                type: 'object',
                with: {
                  inner: {
                    items: [1, 2, value],
                  },
                },
              });",
        ),
        (
            "const value = 1;
            jest.fn().mockImplementation(() => {
              return [{
                type: 'object',
                with: {
                  inner: {
                    items: [1, 2, value],
                  },
                },
              }]
            });",
            "const value = 1;
            jest.fn().mockReturnValue([{
                type: 'object',
                with: {
                  inner: {
                    items: [1, 2, value],
                  },
                },
              }]);",
        ),
        (
            "const mx = 1
            let my = 2;
            aVariable.mockImplementation(() => mx || my);
            aVariable.mockImplementation(() => mx || 0);
            aVariable.mockImplementation(() => my && mx);
            aVariable.mockImplementation(() => mx ?? (7 && my));
            aVariable.mockImplementation(() => mx ?? (7 && 0));",
            "const mx = 1
            let my = 2;
            aVariable.mockImplementation(() => mx || my);
            aVariable.mockReturnValue(mx || 0);
            aVariable.mockImplementation(() => my && mx);
            aVariable.mockImplementation(() => mx ?? (7 && my));
            aVariable.mockReturnValue(mx ?? (7 && 0));",
        ),
        (
            "const x = true;
            let value = 1;
            aVariable.mockImplementation(() => value ? true : false);
            aVariable.mockImplementation(() => x ? true : false);
            aVariable.mockImplementation(() => x ? true : value);
            aVariable.mockImplementation(() => true ? true : value);
            aVariable.mockImplementation(() => true ? true : true ? value : false);
            aVariable.mockImplementation(() => true ? true : true ? x : false);
            aVariable.mockImplementation(() => true ? true : true ? true : false);",
            "const x = true;
            let value = 1;
            aVariable.mockImplementation(() => value ? true : false);
            aVariable.mockReturnValue(x ? true : false);
            aVariable.mockImplementation(() => x ? true : value);
            aVariable.mockImplementation(() => true ? true : value);
            aVariable.mockImplementation(() => true ? true : true ? value : false);
            aVariable.mockReturnValue(true ? true : true ? x : false);
            aVariable.mockReturnValue(true ? true : true ? true : false);",
        ),
    ];

    Tester::new(PreferMockReturnShorthand::NAME, PreferMockReturnShorthand::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
