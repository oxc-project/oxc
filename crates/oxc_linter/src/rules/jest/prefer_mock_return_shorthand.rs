use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, IdentifierReference, Statement, VariableDeclarationKind},
};
use oxc_ast_visit::Visit;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{ReferenceId, SymbolId};
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_mock_return_shorthand_diagnostic(
    span: Span,
    current_property: &str,
    replacement: &str,
) -> OxcDiagnostic {
    let help = format!("Replace `{current_property}` with `{replacement}`.");

    OxcDiagnostic::warn("Mock functions that return simple values should use `mockReturnValue/mockReturnValueOnce`.")
        .with_help(help)
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferMockReturnShorthand;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// When working with mocks of functions that return simple values, Jest provides some API sugar functions to reduce the amount of boilerplate you have to write.
    ///
    /// ### Why is this bad?
    ///
    /// Not using Jest’s API sugar functions adds unnecessary boilerplate and makes tests harder to read. These helpers clearly express intent
    /// and reduce errors, keeping tests simple and maintainable.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// jest.fn().mockImplementation(() => 'hello world');
    ///
    /// jest
    ///   .spyOn(fs.promises, 'readFile')
    ///   .mockImplementationOnce(() => Promise.reject(new Error('oh noes!')));
    ///
    /// myFunction
    ///   .mockImplementationOnce(() => 42)
    ///   .mockImplementationOnce(() => Promise.resolve(42))
    ///   .mockReturnValue(0);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// jest.fn().mockResolvedValue(123);
    ///
    /// jest
    ///   .spyOn(fs.promises, 'readFile')
    ///   .mockReturnValue(Promise.reject(new Error('oh noes!')));
    /// jest.spyOn(fs.promises, 'readFile').mockRejectedValue(new Error('oh noes!'));
    ///
    /// jest.spyOn(fs, 'readFileSync').mockImplementationOnce(() => {
    ///   throw new Error('oh noes!');
    /// });
    ///
    /// myFunction
    ///   .mockResolvedValueOnce(42)
    ///   .mockResolvedValueOnce(42)
    ///   .mockReturnValue(0);
    /// ```
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/vitest-dev/eslint-plugin-vitest/blob/main/docs/rules/prefer-mock-return-shorthand.md),
    /// to use it, add the following configuration to your `.oxlintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/prefer-mock-return-shorthand": "error"
    ///   }
    /// }
    /// ```
    PreferMockReturnShorthand,
    jest,
    style,
    fix,
);

impl Rule for PreferMockReturnShorthand {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(mem_expr) = call_expr.callee.as_member_expression() else {
            return;
        };

        if call_expr.arguments.is_empty() {
            return;
        }

        let Some((property_span, property_name)) = mem_expr.static_property_info() else {
            return;
        };

        let Some(expr) = call_expr.arguments.first().and_then(Argument::as_expression) else {
            return;
        };

        let is_once = property_name.ends_with("Once");

        if !property_name.eq("mockImplementation") && !property_name.eq("mockImplementationOnce") {
            return;
        }

        let Some(return_expression) = get_mock_return(expr) else {
            return;
        };

        if let Expression::UpdateExpression(_) = return_expression {
            return;
        }

        let mut visitor = IdentifierCollectorVisitor::new();

        visitor.visit_expression(return_expression);

        for reference_id in visitor.references {
            if let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id()
                && Self::is_mutable(symbol_id, ctx)
            {
                return;
            }
        }

        let new_property_name = if is_once { "mockReturnValueOnce" } else { "mockReturnValue" };
        ctx.diagnostic_with_fix(
            prefer_mock_return_shorthand_diagnostic(
                property_span,
                property_name,
                new_property_name,
            ),
            |fixer| {
                let return_text = ctx
                    .source_range(GetSpan::span(return_expression.without_parentheses()))
                    .to_owned();
                let argument_span = GetSpan::span(expr);

                let mut multifixer = fixer.for_multifix().new_fix_with_capacity(2);

                multifixer.push(fixer.replace(property_span, new_property_name));
                multifixer.push(fixer.replace(argument_span, return_text));

                multifixer.with_message("Replaced successfully")
            },
        );
    }
}

impl PreferMockReturnShorthand {
    fn is_mutable(symbol_id: SymbolId, ctx: &LintContext<'_>) -> bool {
        let scoping = ctx.scoping();

        if scoping.symbol_is_mutated(symbol_id) {
            return true;
        }

        let decl_node_id = scoping.symbol_declaration(symbol_id);
        if let AstKind::VariableDeclarator(_) = ctx.nodes().kind(decl_node_id)
            && let AstKind::VariableDeclaration(parent) = ctx.nodes().parent_kind(decl_node_id)
        {
            return parent.kind != VariableDeclarationKind::Const;
        }

        false
    }
}

fn get_mock_return<'a>(argument_expression: &'a Expression<'a>) -> Option<&'a Expression<'a>> {
    match argument_expression {
        Expression::ArrowFunctionExpression(arrow_func) => {
            if arrow_func.r#async
                || arrow_func.body.statements.len() > 1
                || !arrow_func.params.is_empty()
            {
                return None;
            }

            let stmt = arrow_func.body.statements.first()?;

            match stmt {
                Statement::ExpressionStatement(stmt_expr) => Some(&stmt_expr.expression),
                Statement::ReturnStatement(return_statement) => {
                    let Some(arg_expr) = &return_statement.argument else {
                        return None;
                    };

                    Some(arg_expr)
                }
                _ => None,
            }
        }
        Expression::FunctionExpression(function) => {
            if function.r#async || !function.params.is_empty() {
                return None;
            }

            let Some(body) = &function.body else {
                return None;
            };

            if body.statements.len() > 1 {
                return None;
            }

            let stmt = body.statements.first()?;

            match stmt {
                Statement::ExpressionStatement(stmt_expr) => Some(&stmt_expr.expression),
                Statement::ReturnStatement(return_statement) => {
                    let Some(arg_expr) = &return_statement.argument else {
                        return None;
                    };

                    Some(arg_expr)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

struct IdentifierCollectorVisitor {
    references: FxHashSet<ReferenceId>,
}

impl IdentifierCollectorVisitor {
    fn new() -> Self {
        Self { references: FxHashSet::default() }
    }
}

impl<'a> Visit<'a> for IdentifierCollectorVisitor {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        self.references.insert(ident.reference_id());
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
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

    let mut fail = vec![
        r#"jest.fn().mockImplementation(() => "hello sunshine")"#,
        "jest.fn().mockImplementation(() => ({}))",
        "jest.fn().mockImplementation(() => x)",
        "jest.fn().mockImplementation(() => true ? x : y)",
        r#"jest.fn().mockImplementation(function () {
              return "hello world";
            })"#,
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

    let mut fix = vec![
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
        (
            r#"aVariable
              .mockImplementation(() => 42)
              .mockImplementation(async () => 42)
              .mockImplementation(() => Promise.resolve(42))
              .mockReturnValue("hello world")"#,
            r#"aVariable
              .mockReturnValue(42)
              .mockImplementation(async () => 42)
              .mockReturnValue(Promise.resolve(42))
              .mockReturnValue("hello world")"#,
        ),
        (
            r#"aVariable
              .mockImplementationOnce(() => Promise.reject(42))
              .mockImplementation(() => "hello sunshine")
              .mockReturnValueOnce(Promise.reject(42))"#,
            r#"aVariable
              .mockReturnValueOnce(Promise.reject(42))
              .mockReturnValue("hello sunshine")
              .mockReturnValueOnce(Promise.reject(42))"#,
        ),
        (
            "jest.fn().mockImplementation(() => (input: number | Record<string, number[]>) => typeof input === 'number' ? input.toFixed(2) : JSON.stringify(input))",
            "jest.fn().mockReturnValue((input: number | Record<string, number[]>) => typeof input === 'number' ? input.toFixed(2) : JSON.stringify(input))",
        ),
        ("jest.fn().mockImplementation(() => [], xyz)", "jest.fn().mockReturnValue([], xyz)"),
        (
            r#"jest.spyOn(fs, "readFile").mockImplementation(() => new Error("oh noes!"))"#,
            r#"jest.spyOn(fs, "readFile").mockReturnValue(new Error("oh noes!"))"#,
        ),
        (
            "aVariable.mockImplementation(() => {
              return Promise.resolve(value)
                .then(value => value + 1);
            });",
            "aVariable.mockReturnValue(Promise.resolve(value)
                .then(value => value + 1));",
        ),
        (
            "aVariable.mockImplementation(() => {
              return Promise.all([1, 2, 3]);
            });",
            "aVariable.mockReturnValue(Promise.all([1, 2, 3]));",
        ),
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
            "const value = 1;
            jest.fn().mockImplementation(() => new Mx(value));
            jest.fn().mockImplementation(() => new Mx(() => value));
            jest.fn().mockImplementation(() => new Mx(() => { return value }));",
            "const value = 1;
            jest.fn().mockReturnValue(new Mx(value));
            jest.fn().mockReturnValue(new Mx(() => value));
            jest.fn().mockReturnValue(new Mx(() => { return value }));",
        ),
        (
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
            "const value = 1;
            jest.fn().mockReturnValue(mx(value));
            jest.fn().mockReturnValue(mx?.(value));
            jest.fn().mockReturnValue(mx().my());
            jest.fn().mockReturnValue(mx().my);
            jest.fn().mockReturnValue(mx.my());
            jest.fn().mockReturnValue(mx?.my());
            jest.fn().mockReturnValue(mx.my);
            jest.fn().mockReturnValue(mx(value).my());
            jest.fn().mockReturnValue(mx(value)?.my());
            jest.fn().mockReturnValue(mx(value).my);
            jest.fn().mockReturnValue(mx.my(value));
            jest.fn().mockReturnValue(mx().my(value));
            jest.fn().mockReturnValue(mx.my(value));
            jest.fn().mockReturnValue(mx.my?.(value));
            jest.fn().mockReturnValue(mx(value).my(value));
            jest.fn().mockReturnValue(mx?.(value)?.my?.(value));
            jest.fn().mockReturnValue(new Mx().add(value));
            jest.fn().mockReturnValue(mx([{
                type: 'object',
                with: {
                  inner: {
                    items: [1, 2, value],
                  },
                },
              }]));",
        ),
        (
            "const propName = 'world';
            aVariable.mockImplementation(() => mx[propName]());
            aVariable.mockImplementation(() => mx[propName]);
            aVariable.mockImplementation(() => ({ [propName]: 1 }));",
            "const propName = 'world';
            aVariable.mockReturnValue(mx[propName]());
            aVariable.mockReturnValue(mx[propName]);
            aVariable.mockReturnValue({ [propName]: 1 });",
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

    let vitest_pass = vec![
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
        "vi.fn().mockReturnValue(42)",
        "vi.fn(() => Promise.resolve(42))",
        "vi.fn(() => 42)",
        "vi.fn(() => ({}))",
        "aVariable.mockImplementation",
        "aVariable.mockImplementation()",
        "jest.fn().mockImplementation(async () => 1);", // { "parserOptions": { "ecmaVersion": 2017 } },
        "jest.fn().mockImplementation(async function () {});", // { "parserOptions": { "ecmaVersion": 2017 } },
        "
                    jest.fn().mockImplementation(async function () {
                      return 42;
                    });
                  ", // { "parserOptions": { "ecmaVersion": 2017 } },
        "
                  aVariable.mockImplementation(() => {
                    if (true) {
                      return 1;
                    }

                    return 2;
                  });
                ",
        "aVariable.mockImplementation(() => value++)",
        "aVariable.mockImplementationOnce(() => --value)",
        "
                  const aValue = 0;
                  aVariable.mockImplementation(() => {
                    return aValue++;
                  });
                ",
        "
                  aVariable.mockImplementation(() => {
                    aValue += 1;

                    return aValue;
                  });
                ",
        "
                  aVariable.mockImplementation(() => {
                    aValue++;

                    return aValue;
                  });
                ",
        "aVariable.mockReturnValue()",
        "aVariable.mockReturnValue(1)",
        r#"aVariable.mockReturnValue("hello world")"#,
        "vi.spyOn(Thingy, 'method').mockImplementation(param => param * 2);",
        "vi.spyOn(Thingy, 'method').mockImplementation(param => true ? param : 0);",
        "
                  aVariable.mockImplementation(() => {
                    const value = new Date();

                    return Promise.resolve(value);
                  });
                ",
        "
                  aVariable.mockImplementation(() => {
                    throw new Error('oh noes!');
                  });
                ",
        "aVariable.mockImplementation(() => { /* do something */ });",
        "
                  aVariable.mockImplementation(() => {
                    const x = 1;

                    console.log(x + 2);
                  });
                ",
        "aVariable.mockReturnValue(Promise.all([1, 2, 3]));",
        "
                  let currentX = 0;
                  jest.spyOn(X, getCount).mockImplementation(() => currentX);

                  // stuff happens

                  currentX++;

                  // more stuff happens
                ",
        "
                  let currentX = 0;
                  jest.spyOn(X, getCount).mockImplementation(() => currentX);
                ",
        "
                  let currentX = 0;
                  currentX = 0;
                  jest.spyOn(X, getCount).mockImplementation(() => currentX);
                ",
        "
                  var currentX = 0;
                  currentX = 0;
                  jest.spyOn(X, getCount).mockImplementation(() => currentX);
                ",
        "
                  var currentX = 0;
                  var currentX = 0;
                  jest.spyOn(X, getCount).mockImplementation(() => currentX);
                ",
        "
                  let doSomething = () => {};

                  jest.spyOn(X, getCount).mockImplementation(() => doSomething);
                ",
        "
                  let currentX = 0;
                  jest.spyOn(X, getCount).mockImplementation(() => {
                    currentX += 1;

                    return currentX;
                  });
                ",
        "
                  const currentX = 0;
                  jest.spyOn(X, getCount).mockImplementation(() => {
                    console.log('returning', currentX);

                    return currentX;
                  });
                ",
    ];

    let vitest_fail = vec![
        r#"vi.fn().mockImplementation(() => "hello sunshine")"#,
        "vi.fn().mockImplementation(() => ({}))",
        "vi.fn().mockImplementation(() => x)",
        "vi.fn().mockImplementation(() => true ? x : y)",
        r#"vi.fn().mockImplementation(() => "hello world")"#,
        r#"aVariable.mockImplementation(() => "hello world")"#,
        r#"vi.fn().mockImplementationOnce(() => "hello world")"#,
        r#"aVariable.mockImplementationOnce(() => "hello world")"#,
        "vi.fn().mockImplementation(() => [], xyz)",
        r#"vi.spyOn(fs, "readFile").mockImplementation(() => new Error("oh noes!"))"#,
    ];

    let vitest_fix = vec![
        (
            r#"vi.fn().mockImplementation(() => "hello sunshine")"#,
            r#"vi.fn().mockReturnValue("hello sunshine")"#,
        ),
        ("vi.fn().mockImplementation(() => ({}))", "vi.fn().mockReturnValue({})"),
        ("vi.fn().mockImplementation(() => x)", "vi.fn().mockReturnValue(x)"),
        ("vi.fn().mockImplementation(() => true ? x : y)", "vi.fn().mockReturnValue(true ? x : y)"),
        (
            r#"vi.fn().mockImplementation(() => "hello world")"#,
            r#"vi.fn().mockReturnValue("hello world")"#,
        ),
        (
            r#"aVariable.mockImplementation(() => "hello world")"#,
            r#"aVariable.mockReturnValue("hello world")"#,
        ),
        (
            r#"vi.fn().mockImplementationOnce(() => "hello world")"#,
            r#"vi.fn().mockReturnValueOnce("hello world")"#,
        ),
        (
            r#"aVariable.mockImplementationOnce(() => "hello world")"#,
            r#"aVariable.mockReturnValueOnce("hello world")"#,
        ),
        ("vi.fn().mockImplementation(() => [], xyz)", "vi.fn().mockReturnValue([], xyz)"),
        (
            r#"vi.spyOn(fs, "readFile").mockImplementation(() => new Error("oh noes!"))"#,
            r#"vi.spyOn(fs, "readFile").mockReturnValue(new Error("oh noes!"))"#,
        ),
    ];

    pass.extend(vitest_pass);
    fail.extend(vitest_fail);
    fix.extend(vitest_fix);

    Tester::new(PreferMockReturnShorthand::NAME, PreferMockReturnShorthand::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
