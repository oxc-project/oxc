use std::borrow::Cow;
#[allow(dead_code, unused)]
use std::ops::Deref;

use oxc_allocator::{Address, GetAddress};
use oxc_ast::{AstKind, ast::*};
use oxc_ast_visit::{
    Visit,
    walk::{self, walk_expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{ScopeFlags, SymbolId};
use oxc_span::{CompactStr, GetSpan, Span};
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use smallvec::SmallVec;

use crate::{
    AstNode,
    context::LintContext,
    // fixer::{RuleFix, RuleFixer},
    rule::Rule,
    utils::default_true,
};

fn explicit_module_boundary_types_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

fn func_missing_return_type(fn_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing return type on function").with_label(fn_span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ExplicitModuleBoundaryTypes(Box<Config>);
impl From<Config> for ExplicitModuleBoundaryTypes {
    fn from(config: Config) -> Self {
        Self(Box::new(config))
    }
}
impl Deref for ExplicitModuleBoundaryTypes {
    type Target = Config;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Whether to ignore arguments that are explicitly typed as `any`.
    #[serde(default)]
    allow_arguments_explicitly_typed_as_any: bool,
    /// Whether to ignore return type annotations on body-less arrow functions
    /// that return an `as const` type assertion. You must still type the
    /// parameters of the function.
    #[serde(default = "default_true")]
    allow_direct_const_assertion_in_arrow_functions: bool,
    /// An array of function/method names that will not have their arguments or
    /// return values checked.
    #[serde(default)]
    allowed_names: Vec<CompactStr>,
    /// Whether to ignore return type annotations on functions immediately
    /// returning another function expression. You must still type the
    /// parameters of the function.
    #[serde(default = "default_true")]
    allow_higher_order_functions: bool,
    /// Whether to ignore return type annotations on functions with overload
    /// signatures.
    #[serde(default)]
    allow_overload_functions: bool,
    /// Whether to ignore type annotations on the variable of a function
    /// expression.
    #[serde(default = "default_true")]
    allow_typed_function_expressions: bool,
}
impl TryFrom<Value> for Config {
    type Error = serde_json::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}
impl Config {
    fn is_allowed_name(&self, name: &str) -> bool {
        self.allowed_names.iter().any(|n| n == name)
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require explicit return and argument types on exported functions' and classes' public class methods.
    ///
    /// ### Why is this bad?
    ///
    /// Explicit types for function return values and arguments makes it clear
    /// to any calling code what is the module boundary's input and output.
    /// Adding explicit type annotations for those types can help improve code
    /// readability. It can also improve TypeScript type checking performance on
    /// larger codebases.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// // Should indicate that no value is returned (void)
    /// export function test() {
    ///   return;
    /// }
    ///
    /// // Should indicate that a string is returned
    /// export var arrowFn = () => 'test';
    ///
    /// // All arguments should be typed
    /// export var arrowFn = (arg): string => `test ${arg}`;
    /// export var arrowFn = (arg: any): string => `test ${arg}`;
    ///
    /// export class Test {
    ///   // Should indicate that no value is returned (void)
    ///   method() {
    ///     return;
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// // A function with no return value (void)
    /// export function test(): void {
    ///   return;
    /// }
    ///
    /// // A return value of type string
    /// export var arrowFn = (): string => 'test';
    ///
    /// // All arguments should be typed
    /// export var arrowFn = (arg: string): string => `test ${arg}`;
    /// export var arrowFn = (arg: unknown): string => `test ${arg}`;
    ///
    /// export class Test {
    ///   // A class method with no return value (void)
    ///   method(): void {
    ///     return;
    ///   }
    /// }
    ///
    /// // The function does not apply because it is not an exported function.
    /// function test() {
    ///   return;
    /// }
    /// ```
    ExplicitModuleBoundaryTypes,
    typescript,
    restriction,
);

impl Rule for ExplicitModuleBoundaryTypes {
    fn from_configuration(mut value: Value) -> Self {
        let Some(value) = value.get_mut(0).filter(|v| v.is_object()) else {
            return Self::default();
        };
        serde_json::from_value(value.take()).unwrap_or_default()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ExportNamedDeclaration(export) => {
                // export { foo } from 'bar';
                if export.source.is_some() {
                    return;
                }
                if let Some(decl) = &export.declaration {
                    let mut checker = ExplicitTypesChecker::new(self, ctx);
                    walk::walk_declaration(&mut checker, decl);
                }
            }
            _ => {}
        }
    }
    fn should_run(&self, ctx: &crate::ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

#[derive(Clone, Copy)]
enum Fn<'a> {
    Fn(&'a Function<'a>),
    Arrow(&'a ArrowFunctionExpression<'a>),
    None,
}
impl<'a> Fn<'a> {
    fn address(self) -> Option<Address> {
        match self {
            Fn::Fn(f) => Some(Address::from_ptr(f)),
            Fn::Arrow(a) => Some(Address::from_ptr(a)),
            Fn::None => None,
        }
    }
}

struct ExplicitTypesChecker<'a, 'c> {
    rule: &'c ExplicitModuleBoundaryTypes,
    ctx: &'c LintContext<'a>,
    target_symbol: Option<IdentifierName<'a>>,
    // note: we avoid allocations by reserving space on the stack. Yes this
    // struct is large, but reserving space for it is just a offset op from the
    // stack pointer.
    /// function stack
    fns: SmallVec<[Fn<'a>; 8]>,
    /// Return statements we've seen inside visited functions. Keys are the
    /// [`Address`]es of those functions.
    fn_returns: FxHashMap<Address, SmallVec<[&'a ReturnStatement<'a>; 2]>>,
}
impl<'a, 'c> ExplicitTypesChecker<'a, 'c> {
    fn new(rule: &'c ExplicitModuleBoundaryTypes, ctx: &'c LintContext<'a>) -> Self {
        Self {
            rule,
            ctx,
            target_symbol: None,
            fns: smallvec::smallvec![],
            fn_returns: FxHashMap::default(),
        }
    }
    fn target_span(&self) -> Option<Span> {
        self.target_symbol.as_ref().map(|id| id.span)
    }
    fn with_target_binding(&mut self, binding: Option<&BindingIdentifier<'a>>) -> bool {
        if let Some(id) = binding {
            self.target_symbol.replace(IdentifierName { name: id.name, span: id.span });
            true
        } else {
            false
        }
    }
    fn with_target_property(&mut self, prop: Option<&PropertyKey<'a>>) -> bool {
        let Some(id) = prop else {
            return false;
        };
        if let Some(Cow::Borrowed(name)) = id.static_name() {
            self.target_symbol.replace(IdentifierName { name: Atom::from(name), span: id.span() });
            true
        } else {
            false
        }
    }

    fn check_function_without_return(&mut self, func: &Function<'a>) {
        debug_assert!(func.return_type.is_none());

        let target_span = self.target_symbol.as_ref();
        let target_name = target_span.map(|t| t.name);
        let span = target_span
            .map(|t| t.span)
            .unwrap_or(Span::sized(func.span.start, "function".len() as u32));
        let is_allowed = || {
            func.name()
                .map(Into::into)
                .or(target_name)
                .is_some_and(|name| self.rule.is_allowed_name(&name))
        };

        if !self.rule.allow_higher_order_functions {
            if !is_allowed() {
                self.ctx.diagnostic(func_missing_return_type(span));
            }
            return;
        }
        let Some(body) = func.body.as_deref() else {
            return;
        };
        walk::walk_function_body(self, body);
        if let Some(returns) = self.fn_returns.get(&Address::from_ptr(func)) {
            let is_hof = returns.iter().any(|ret| {
                matches!(
                    ret.argument,
                    Some(
                        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
                    )
                )
            });
            if !is_hof && !is_allowed() {
                self.ctx.diagnostic(func_missing_return_type(span))
            }
        } else {
            // NOTE: only arrow functions can have implicit returns
        }
    }

    fn check_arrow_without_return(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        debug_assert!(arrow.return_type.is_none());
        let target_span = self.target_symbol.as_ref();
        let target_name = target_span.map(|t| t.name);
        let span = target_span.map(|t| t.span).unwrap_or(arrow.params.span);
        let is_allowed = || target_name.is_some_and(|name| self.rule.is_allowed_name(&name));

        if !self.rule.allow_higher_order_functions {
            if !is_allowed() {
                self.ctx.diagnostic(func_missing_return_type(span));
            }
            return;
        }

        if arrow.expression {
            let Some(expr) = arrow.get_expression() else {
                debug_assert!(
                    false,
                    "ArrowFunctionExpression claims to have an implicit return but get_expression() returned None. This is a parser bug"
                );
                return;
            };

            // `export const foo = () => 1 as const`
            if self.rule.allow_direct_const_assertion_in_arrow_functions
                && matches!(get_typed_inner_expression(expr), Expression::TSAsExpression(_))
            {
                return;
            }

            self.ctx.diagnostic(func_missing_return_type(span));
            return;
        }

        walk::walk_function_body(self, &arrow.body);
        if let Some(returns) = self.fn_returns.get(&Address::from_ptr(arrow)) {
            let is_hof = returns.iter().any(|ret| {
                matches!(
                    ret.argument,
                    Some(
                        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
                    )
                )
            });
            if !is_hof && !is_allowed() {
                self.ctx.diagnostic(func_missing_return_type(span))
            }
        } else {
            // NOTE: only arrow functions can have implicit returns
        }
    }
}

impl<'a, 'c> Visit<'a> for ExplicitTypesChecker<'a, 'c> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        match dbg!(kind) {
            AstKind::Function(f) => self.fns.push(Fn::Fn(f)),
            AstKind::ArrowFunctionExpression(arrow) => self.fns.push(Fn::Arrow(arrow)),
            AstKind::Class(_) => self.fns.push(Fn::None),
            AstKind::ReturnStatement(ret) => {
                // returns outside of functions are semantic errors
                debug_assert!(!self.fns.is_empty());
                let Some(f) = self.fns.last() else {
                    return;
                };
                let Some(addr) = f.address() else {
                    // e.g. something like
                    // function foo() {
                    //     class C { return; }
                    // }
                    // which also doesn't make sense.
                    debug_assert!(
                        false,
                        "found a return nested somewhere in a function, but due to the current scope, that function is not a valid return target."
                    );
                    return;
                };
                let returns = self.fn_returns.entry(addr).or_default();
                returns.push(ret)
            }
            _ => {}
        }
    }

    fn leave_node(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) | AstKind::Class(_) => {
                let last = self.fns.pop();
                debug_assert!(
                    last.is_some(),
                    "tried to exit a function/class node when it was not on the function stack"
                );
            }
            _ => {}
        }
    }

    fn visit_variable_declarator(&mut self, var: &VariableDeclarator<'a>) {
        if var.id.type_annotation.is_some() {
            return;
        }
        let Some(init) = &var.init else {
            return; // TODO: what do we do here?
        };
        let Some(binding) = var.id.get_binding_identifier() else {
            return;
        };
        match get_typed_inner_expression(init) {
            // we consider these well-typed
            Expression::TSAsExpression(_) | Expression::TSTypeAssertion(_) => return,
            Expression::ObjectExpression(_) | Expression::ArrayExpression(_) => return,
            expr if expr.is_literal() => return,
            expr => {
                self.with_target_binding(Some(binding));
                walk_expression(self, expr);
                self.target_symbol = None;
            }
        }
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        let had_id = self.with_target_binding(class.id.as_ref());
        walk::walk_class_body(self, class.body.as_ref());

        if had_id {
            self.target_symbol = None;
        }
    }

    fn visit_class_element(&mut self, el: &ClassElement<'a>) {
        // dont check non-public members
        if el.accessibility().is_some_and(|a| a != TSAccessibility::Public)
            || el.property_key().is_some_and(|key| matches!(key, PropertyKey::PrivateIdentifier(_)))
        {
            return;
        }

        if let ClassElement::PropertyDefinition(prop) = &el {
            if prop.type_annotation.is_some() {
                return;
            }
        }

        let is_target = self.with_target_property(el.property_key());
        walk::walk_class_element(self, el);
        if is_target {
            self.target_symbol = None;
        }
    }

    fn visit_method_definition(&mut self, m: &MethodDefinition<'a>) {
        if m.kind.is_constructor() {
            // skip return type
            // TODO: adjust target_symbol
            self.visit_formal_parameters(m.value.as_ref().params.as_ref());
            return;
        }
        walk::walk_method_definition(self, m);
    }

    fn visit_function(&mut self, func: &Function<'a>, _flags: ScopeFlags) {
        let f = AstKind::Function(self.alloc(func));
        let had_id = self.with_target_binding(func.id.as_ref());
        self.enter_node(f);

        self.visit_formal_parameters(func.params.as_ref());

        if func.return_type.is_none() {
            self.check_function_without_return(func);
        }

        self.leave_node(f);
        if had_id {
            self.target_symbol = None;
        }
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        let f = AstKind::ArrowFunctionExpression(self.alloc(arrow));
        self.enter_node(f);

        self.visit_formal_parameters(arrow.params.as_ref());

        if arrow.return_type.is_none() {
            self.check_arrow_without_return(arrow);
        }

        self.leave_node(f);
    }
}

/// like [`Expression::get_inner_expression`], but does not skip over most ts syntax
fn get_typed_inner_expression<'a, 'e>(expr: &'e Expression<'a>) -> &'e Expression<'a> {
    match expr {
        Expression::ParenthesizedExpression(expr) => get_typed_inner_expression(&expr.expression),
        Expression::TSNonNullExpression(expr) => get_typed_inner_expression(&expr.expression),
        _ => expr,
    }
}

#[test]
fn test_debug() {
    use crate::tester::Tester;
    use serde_json::{Value, json};
    let pass: Vec<(&'static str, Option<Value>)> = vec![
        //
        // (
        //     "export function fn() { return (): void => {}; }",
        //     Some(json!([{ "allowHigherOrderFunctions": true }])),
        // ),
        (
            "
            export class Test {
            
               method() {
                 return;
               }
            //   foo = () => {
            //     bar: 5;
            //   };
            }
            ",
            Some(serde_json::json!([{ "allowedNames": ["prop", "method", "null", "foo"], }])),
        ),
        // (
        //     "
        //     export class Test {
        //       get prop() {
        //         return 1;
        //       }
        //       set prop(p) {}
        //       method() {
        //         return;
        //       }
        //       // prettier-ignore
        //       'method'() {}
        //       ['prop']() {}
        //       [`prop`]() {}
        //       [null]() {}
        //       [`${v}`](): void {}

        //       foo = () => {
        //         bar: 5;
        //       };
        //     }
        //     ",
        //     Some(serde_json::json!([{ "allowedNames": ["prop", "method", "null", "foo"], }])),
        // ),
    ];
    let fail: Vec<(&'static str, Option<Value>)> = vec![];
    Tester::new(ExplicitModuleBoundaryTypes::NAME, ExplicitModuleBoundaryTypes::PLUGIN, pass, fail)
        .test();
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
              get prop() {
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
              get prop(): void {
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
              private get prop() {
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
        // FIXME: move to Tester using ".ts"
        // (
        //     "const x = <Foo,>(() => {});",
        //     Some(serde_json::json!([{ "allowTypedFunctionExpressions": true }])),
        // ),
        (
            "
            export const x = {
              foo: () => {},
            } as Foo;
            ",
            Some(serde_json::json!([{ "allowTypedFunctionExpressions": true }])),
        ),
        // FIXME: move to Tester using ".ts"
        // (
        //     "
        //     export const x = <Foo>{
        //       foo: () => {},
        //     };
        //     ",
        //     Some(serde_json::json!([{ "allowTypedFunctionExpressions": true }])),
        // ),
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
        // (
        //     "
        //     export const x = <Foo>{
        //       foo: { bar: () => {} },
        //     };
        //     ",
        //     Some(serde_json::json!([{ "allowTypedFunctionExpressions": true }])),
        // ),
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
              set prop(p) {}
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
              set prop(p) {}
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
        ("export function foo({ foo }): void {}", None),
        ("export function foo([bar]): void {}", None),
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
