use std::{borrow::Cow, ops::Deref};

use oxc_allocator::{Address, UnstableAddress};
use oxc_ast::{AstKind, ast::*};
use oxc_ast_visit::{
    Visit,
    walk::{self, walk_expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::{CompactStr, GetSpan, Span};
use oxc_syntax::node::NodeId;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use smallvec::SmallVec;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn func_missing_return_type(fn_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing return type on function").with_label(fn_span)
}

fn func_missing_argument_type(param_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing argument type on function").with_label(param_span)
}

fn func_argument_is_explicitly_any(param_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Argument is explicitly typed as `any`").with_label(param_span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ExplicitModuleBoundaryTypes(Box<ExplicitModuleBoundaryTypesConfig>);

impl Deref for ExplicitModuleBoundaryTypes {
    type Target = ExplicitModuleBoundaryTypesConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct ExplicitModuleBoundaryTypesConfig {
    /// Whether to ignore arguments that are explicitly typed as `any`.
    allow_arguments_explicitly_typed_as_any: bool,
    /// Whether to ignore return type annotations on body-less arrow functions
    /// that return an `as const` type assertion. You must still type the
    /// parameters of the function.
    allow_direct_const_assertion_in_arrow_functions: bool,
    /// An array of function/method names that will not have their arguments or
    /// return values checked.
    allowed_names: Vec<CompactStr>,
    /// Whether to ignore return type annotations on functions immediately
    /// returning another function expression. You must still type the
    /// parameters of the function.
    allow_higher_order_functions: bool,
    /// Whether to ignore return type annotations on functions with overload
    /// signatures.
    allow_overload_functions: bool,
    /// Whether to ignore type annotations on the variable of a function
    /// expression.
    allow_typed_function_expressions: bool,
}

impl Default for ExplicitModuleBoundaryTypesConfig {
    fn default() -> Self {
        Self {
            allow_arguments_explicitly_typed_as_any: false,
            allow_direct_const_assertion_in_arrow_functions: true,
            allowed_names: vec![],
            allow_overload_functions: false,
            allow_typed_function_expressions: true,
            allow_higher_order_functions: true,
        }
    }
}

impl ExplicitModuleBoundaryTypesConfig {
    fn is_allowed_name(&self, name: &str) -> bool {
        self.allowed_names.iter().any(|n| n == name)
    }

    fn is_some_allowed_name<S: AsRef<str>>(&self, name: Option<S>) -> bool {
        name.is_some_and(|name| self.is_allowed_name(name.as_ref()))
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
    config = ExplicitModuleBoundaryTypesConfig,
);

impl Rule for ExplicitModuleBoundaryTypes {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        Ok(serde_json::from_value::<DefaultRuleConfig<Self>>(value)
            .unwrap_or_default()
            .into_inner())
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            // look for `export function foo() { ... }`, `export const foo = () => { ... }`,
            // etc.
            AstKind::ExportNamedDeclaration(export) => {
                // export { foo } from 'bar';
                if export.source.is_some() {
                    return;
                }
                if let Some(decl) = &export.declaration {
                    let mut checker = ExplicitTypesChecker::new(self, ctx);
                    walk::walk_declaration(&mut checker, decl);
                } else {
                    let mut checker = ExplicitTypesChecker::new(self, ctx);
                    for specifier in &export.specifiers {
                        if let ModuleExportName::IdentifierReference(id) = &specifier.local {
                            Self::run_on_identifier_reference(ctx, id, &mut checker);
                        }
                    }
                }
            }
            AstKind::TSExportAssignment(export) => {
                self.run_on_exported_expression(ctx, &export.expression);
            }
            AstKind::ExportDefaultDeclaration(export) => {
                match &export.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                        let mut checker = ExplicitTypesChecker::new(self, ctx);
                        checker.visit_function(func, ScopeFlags::Function);
                    }
                    ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                        let mut checker = ExplicitTypesChecker::new(self, ctx);
                        walk::walk_class(&mut checker, class);
                    }
                    ExportDefaultDeclarationKind::TSInterfaceDeclaration(_) => {
                        // nada
                    }
                    match_expression!(ExportDefaultDeclarationKind) => {
                        self.run_on_exported_expression(ctx, export.declaration.to_expression());
                    }
                }
            }
            _ => {}
        }
    }
    fn should_run(&self, ctx: &crate::ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}
impl ExplicitModuleBoundaryTypes {
    fn run_on_exported_expression<'a>(&self, ctx: &LintContext<'a>, expr: &Expression<'a>) {
        self.run_on_exported_expression_inner(ctx, expr, true);
    }
    fn run_on_exported_expression_inner<'a>(
        &self,
        ctx: &LintContext<'a>,
        expr: &Expression<'a>,
        inside_export: bool,
    ) {
        let mut checker = ExplicitTypesChecker::new(self, ctx);
        match get_typed_inner_expression(expr) {
            Expression::Identifier(id) => {
                Self::run_on_identifier_reference(ctx, id, &mut checker);
            }
            Expression::ArrowFunctionExpression(arrow) => {
                walk::walk_arrow_function_expression(&mut checker, arrow);
            }
            // const foo = arg => arg;
            // export default [foo];
            Expression::ArrayExpression(arr) if inside_export => {
                for el in arr.elements.iter().filter_map(ArrayExpressionElement::as_expression) {
                    self.run_on_exported_expression_inner(ctx, el, false);
                }
            }
            // const foo = arg => arg;
            // export default { foo };
            Expression::ObjectExpression(obj) if inside_export => {
                for el in obj.properties.iter().filter_map(ObjectPropertyKind::as_property) {
                    self.run_on_exported_expression_inner(ctx, &el.value, false);
                }
            }
            _ => {}
        }
    }

    fn run_on_identifier_reference<'a>(
        ctx: &LintContext<'a>,
        id: &IdentifierReference<'a>,
        checker: &mut ExplicitTypesChecker<'a, '_>,
    ) {
        let s = ctx.scoping();
        let Some(symbol_id) = s.get_reference(id.reference_id()).symbol_id() else {
            return;
        };
        let decl = ctx.nodes().get_node(s.symbol_declaration(symbol_id));
        match decl.kind() {
            AstKind::VariableDeclaration(it) => {
                walk::walk_variable_declaration(checker, it);
            }
            AstKind::VariableDeclarator(it) => {
                checker.visit_variable_declarator(it);
                // walk::walk_variable_declarator(&mut checker, it)
            }
            AstKind::Function(it) => {
                checker.visit_function(it, ScopeFlags::Function);
            }
            AstKind::Class(it) => walk::walk_class(checker, it),
            _ => {}
        }
    }
}

#[derive(Clone, Copy)]
#[expect(clippy::enum_variant_names)]
enum Fn<'a> {
    Fn(&'a Function<'a>),
    Arrow(&'a ArrowFunctionExpression<'a>),
    None,
}

impl Fn<'_> {
    fn address(self) -> Option<Address> {
        // AST is immutable in linter, so `unstable_address` produces stable `Address`es
        match self {
            Fn::Fn(f) => Some(f.unstable_address()),
            Fn::Arrow(a) => Some(a.unstable_address()),
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
    scope_flags: ScopeFlags,
}
impl<'a, 'c> ExplicitTypesChecker<'a, 'c> {
    fn new(rule: &'c ExplicitModuleBoundaryTypes, ctx: &'c LintContext<'a>) -> Self {
        Self {
            rule,
            ctx,
            target_symbol: None,
            fns: smallvec::smallvec![],
            fn_returns: FxHashMap::default(),
            scope_flags: ScopeFlags::empty(),
        }
    }
    // fn target_span(&self) -> Option<Span> {
    //     self.target_symbol.as_ref().map(|id| id.span)
    // }

    fn with_target_binding(&mut self, binding: Option<&BindingIdentifier<'a>>) -> bool {
        if let Some(id) = binding {
            self.target_symbol.replace(IdentifierName {
                span: id.span,
                node_id: NodeId::DUMMY,
                name: id.name,
            });
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
            self.target_symbol.replace(IdentifierName {
                span: id.span(),
                node_id: NodeId::DUMMY,
                name: Atom::from(name),
            });
            true
        } else {
            false
        }
    }
    #[inline]
    fn reset_target(&mut self, had_target: bool) {
        if had_target {
            self.target_symbol = None;
        }
    }

    fn check_function_without_return(&mut self, func: &Function<'a>) {
        debug_assert!(func.return_type.is_none());

        let target_span = self.target_symbol.as_ref();
        let target_name = target_span.map(|t| t.name);
        #[expect(clippy::cast_possible_truncation)]
        let span =
            target_span.map_or(Span::sized(func.span.start, "function".len() as u32), |t| t.span);
        let is_allowed = || self.rule.is_some_allowed_name(func.name().or(target_name));

        // When allow_overload_functions is enabled, skip return type checking for all functions
        // This is a simplified implementation - a proper implementation would only skip
        // functions that are actually part of an overload set
        if self.rule.allow_overload_functions {
            return;
        }

        if func.body.is_none() {
            return;
        }

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

        // AST is immutable in linter, so `unstable_address` produces stable `Address`es
        let is_hof = self.is_higher_order_function(func.unstable_address());
        if !is_hof && !is_allowed() {
            self.ctx.diagnostic(func_missing_return_type(span));
        }
    }

    fn check_arrow_without_return(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        debug_assert!(arrow.return_type.is_none());
        let target_span = self.target_symbol.as_ref();
        let target_name = target_span.map(|t| t.name);
        let span = target_span.map_or(arrow.params.span, |t| t.span);
        let is_allowed = || self.rule.is_some_allowed_name(target_name);

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

            let mut curr = get_typed_inner_expression(expr);
            loop {
                match curr {
                    // `export const foo = () => 1 as const`
                    Expression::TSAsExpression(as_expr)
                        if self.rule.allow_direct_const_assertion_in_arrow_functions
                            && as_expr.type_annotation.is_const_type_reference() =>
                    {
                        return;
                    }
                    Expression::TSSatisfiesExpression(satisfies) => {
                        curr = get_typed_inner_expression(&satisfies.expression);
                    }

                    // `export const foo = () => () => (): number => 1`
                    Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => {
                        debug_assert!(self.rule.allow_higher_order_functions);
                        walk::walk_function_body(self, &arrow.body);
                        return;
                    }
                    _ => {
                        self.ctx.diagnostic(func_missing_return_type(span));
                        return;
                    }
                }
            }
        } else {
            walk::walk_function_body(self, &arrow.body);

            // AST is immutable in linter, so `unstable_address` produces stable `Address`es
            let is_hof = self.is_higher_order_function(arrow.unstable_address());
            if !is_hof && !is_allowed() {
                self.ctx.diagnostic(func_missing_return_type(span));
            }
        }
    }

    fn is_higher_order_function(&self, address: Address) -> bool {
        let Some(returns) = self.fn_returns.get(&address) else {
            return false;
        };
        returns.iter().any(|ret| {
            matches!(
                ret.argument,
                Some(Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_))
            )
        })
    }
}

impl<'a> Visit<'a> for ExplicitTypesChecker<'a, '_> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::Function(f) => self.fns.push(Fn::Fn(f)),
            AstKind::ArrowFunctionExpression(arrow) => self.fns.push(Fn::Arrow(arrow)),
            AstKind::Class(_) => self.fns.push(Fn::None),
            AstKind::ReturnStatement(ret) => {
                let Some(f) = self.fns.last() else {
                    return;
                };
                let Some(addr) = f.address() else {
                    // e.g. something like
                    //
                    //     function foo() {
                    //         class C { return; }
                    //     }
                    //
                    // which also doesn't make sense.
                    debug_assert!(
                        false,
                        "found a return nested somewhere in a function, but due to the current scope, that function is not a valid return target."
                    );
                    return;
                };
                let returns = self.fn_returns.entry(addr).or_default();
                returns.push(ret);
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

    fn visit_statements(&mut self, it: &oxc_allocator::Vec<'a, Statement<'a>>) {
        for stmt in it {
            match stmt {
                Statement::ReturnStatement(_) => {
                    self.visit_statement(stmt);
                }
                // Only process expression statements when they are the sole statement in the block.
                // This is typically to handle cases like concise arrow functions or modules with a single expression.
                // If this logic needs to be expanded to handle more cases, revisit this condition.
                Statement::ExpressionStatement(_) if it.len() == 1 => {
                    self.visit_statement(stmt);
                }
                _ => {}
            }
        }
    }

    fn visit_variable_declarator(&mut self, var: &VariableDeclarator<'a>) {
        if self.rule.allow_typed_function_expressions && var.type_annotation.is_some() {
            return;
        }
        let Some(init) = &var.init else {
            return; // TODO: what do we do here?
        };
        let Some(binding) = var.id.get_binding_identifier() else {
            return;
        };
        if self.rule.is_allowed_name(&binding.name) {
            return;
        }

        match get_typed_inner_expression(init) {
            // we consider these well-typed
            Expression::TSAsExpression(_) | Expression::TSTypeAssertion(_) => {}
            expr if expr.is_literal() => {}
            expr => {
                self.with_target_binding(Some(binding));
                walk_expression(self, expr);
                self.target_symbol = None;
            }
        }
    }

    fn visit_call_expression(&mut self, _it: &CallExpression<'a>) {
        // ignore
    }

    fn visit_jsx_element(&mut self, _it: &JSXElement<'a>) {
        // ignore
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        let had_id = self.with_target_binding(class.id.as_ref());
        walk::walk_class_body(self, class.body.as_ref());
        self.reset_target(had_id);
    }

    fn visit_class_element(&mut self, el: &ClassElement<'a>) {
        // dont check non-public members
        if el.accessibility().is_some_and(|a| a != TSAccessibility::Public)
            || el.property_key().is_some_and(|key| matches!(key, PropertyKey::PrivateIdentifier(_)))
        {
            return;
        }

        if let ClassElement::PropertyDefinition(prop) = &el
            && prop.type_annotation.is_some()
        {
            return;
        }
        if self.rule.is_some_allowed_name(el.static_name()) {
            return;
        }

        let is_target = self.with_target_property(el.property_key());
        walk::walk_class_element(self, el);
        self.reset_target(is_target);
    }
    fn visit_object_property(&mut self, it: &ObjectProperty<'a>) {
        let is_set = it.kind == PropertyKind::Set;
        let prev_flags = self.scope_flags;
        if is_set {
            self.scope_flags.set(ScopeFlags::SetAccessor, true);
        }
        let had_name = self.with_target_property(Some(&it.key));
        walk::walk_object_property(self, it);
        self.scope_flags = prev_flags;
        self.reset_target(had_name);
    }

    fn visit_method_definition(&mut self, m: &MethodDefinition<'a>) {
        match m.kind {
            MethodDefinitionKind::Constructor | MethodDefinitionKind::Set => {
                // skip return type
                // TODO: adjust target_symbol
                self.visit_formal_parameters(m.value.as_ref().params.as_ref());
            }
            _ => walk::walk_method_definition(self, m),
        }
    }

    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        let f = AstKind::Function(self.alloc(func));
        let id = func.id.as_ref();
        let had_id = self.with_target_binding(id);
        self.enter_node(f);

        self.visit_formal_parameters(func.params.as_ref());

        if func.return_type.is_none() && !flags.union(self.scope_flags).is_set_accessor() {
            self.check_function_without_return(func);
        }

        self.leave_node(f);
        self.reset_target(had_id);
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

    fn visit_formal_parameters(&mut self, it: &FormalParameters<'a>) {
        for param in &it.items {
            self.visit_formal_parameter(param);
        }
        if let Some(rest) = &it.rest {
            if let Some(ty) = &rest.type_annotation {
                if !self.rule.allow_arguments_explicitly_typed_as_any
                    && matches!(ty.type_annotation, TSType::TSAnyKeyword(_))
                {
                    self.ctx.diagnostic(func_argument_is_explicitly_any(it.span));
                }
                return;
            }
            self.ctx.diagnostic(func_missing_argument_type(it.span));
        }
    }
    fn visit_formal_parameter(&mut self, it: &FormalParameter<'a>) {
        // let name = param.get_identifier_name();
        if let Some(ty) = &it.type_annotation {
            if !self.rule.allow_arguments_explicitly_typed_as_any
                && matches!(ty.type_annotation, TSType::TSAnyKeyword(_))
            {
                self.ctx.diagnostic(func_argument_is_explicitly_any(it.span));
            }
            return;
        }

        if it.initializer.is_some() {
            return;
        }

        self.ctx.diagnostic(func_missing_argument_type(it.span));
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

#[cfg(test)]
mod test {
    use super::{ExplicitModuleBoundaryTypes, ExplicitModuleBoundaryTypesConfig};
    use crate::{RuleMeta as _, rule::Rule as _, tester::Tester};
    use serde_json::{Value, json};
    use std::path::PathBuf;

    #[test]
    fn config() {
        let cases: Vec<(ExplicitModuleBoundaryTypesConfig, Value)> =
            vec![(ExplicitModuleBoundaryTypesConfig::default(), json!({}))];
        for (config, expected) in cases {
            let actual: ExplicitModuleBoundaryTypesConfig =
                serde_json::from_value(expected).unwrap();
            assert_eq!(config, actual);
        }

        // test from_configuration, which suppresses invalid configs
        let cases: Vec<(ExplicitModuleBoundaryTypesConfig, Value)> =
            vec![(ExplicitModuleBoundaryTypesConfig::default(), json!([{}]))];
        for (expected, value) in cases {
            let actual = ExplicitModuleBoundaryTypes::from_configuration(value).unwrap();
            assert_eq!(*actual, expected);
        }
    }

    #[test]
    fn debug_test() {
        let pass: Vec<(&'static str, Option<Value>)> = vec![
            //
        ];
        let fail: Vec<(&'static str, Option<Value>)> = vec![
            // line break
            // ("export default () => (true ? () => {} : (): void => {});", None),
            (
                "
            const foo = arg => arg;
            export default foo;
            ",
                None,
            ),
            // (
            //     "export default () => () => () => 1",
            //     Some(json!([{ "allowHigherOrderFunctions": true }])),
            // ),
        ];
        Tester::new(
            ExplicitModuleBoundaryTypes::NAME,
            ExplicitModuleBoundaryTypes::PLUGIN,
            pass,
            fail,
        )
        .test();
    }

    #[test]
    fn rule() {
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
                Some(json!([{ "allowTypedFunctionExpressions": true, }])),
            ),
            (
                "
            export var funcExpr: Foo = function () {
              return 'test';
            };
            ",
                Some(json!([{ "allowTypedFunctionExpressions": true, }])),
            ),
            (
                "const x = (() => {}) as Foo;",
                Some(json!([{ "allowTypedFunctionExpressions": true }])),
            ),
            (
                "
            export const x = {
              foo: () => {},
            } as Foo;
            ",
                Some(json!([{ "allowTypedFunctionExpressions": true }])),
            ),
            (
                "
            export const x: Foo = {
              foo: () => {},
            };
            ",
                Some(json!([{ "allowTypedFunctionExpressions": true }])),
            ),
            (
                "
            export const x = {
              foo: { bar: () => {} },
            } as Foo;
            ",
                Some(json!([{ "allowTypedFunctionExpressions": true }])),
            ),
            (
                "
            export const x: Foo = {
              foo: { bar: () => {} },
            };
            ",
                Some(json!([{ "allowTypedFunctionExpressions": true }])),
            ),
            (
                "
            type MethodType = () => void;

            export class App {
              public method: MethodType = () => {};
            }
            ",
                Some(json!([{ "allowTypedFunctionExpressions": true }])),
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
                Some(json!([{ "allowHigherOrderFunctions": true }])),
            ),
            (
                "export default () => function (): void {};",
                Some(json!([{ "allowHigherOrderFunctions": true }])),
            ),
            (
                "export default () => { return (): void => {}; };",
                Some(json!([{ "allowHigherOrderFunctions": true }])),
            ),
            (
                "export default () => { return function (): void {}; };",
                Some(json!([{ "allowHigherOrderFunctions": true }])),
            ),
            (
                "export function fn() { return (): void => {}; }",
                Some(json!([{ "allowHigherOrderFunctions": true }])),
            ),
            (
                "export function fn() { return function (): void {}; }",
                Some(json!([{ "allowHigherOrderFunctions": true }])),
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
                Some(json!([{ "allowHigherOrderFunctions": true }])),
            ),
            (
                "
            export default () => () => {
              return (): void => {
                return;
              };
            };
            ",
                Some(json!([{ "allowHigherOrderFunctions": true }])),
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
                Some(json!([{ "allowHigherOrderFunctions": true }])),
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
                Some(json!([{ "allowHigherOrderFunctions": true }])),
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
                Some(json!([{ "allowTypedFunctionExpressions": true, }])),
            ),
            (
                "
            export const func1 = (value: number) => ({ type: 'X', value }) as const;
            export const func2 = (value: number) => ({ type: 'X', value }) as const;
            export const func3 = (value: number) => x as const;
            export const func4 = (value: number) => x as const;
            ",
                Some(json!([{ "allowDirectConstAssertionInArrowFunctions": true, }])),
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
                Some(json!([{ "allowDirectConstAssertionInArrowFunctions": true, }])),
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
                Some(json!([{ "allowDirectConstAssertionInArrowFunctions": true, }])),
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
                Some(json!([{ "allowDirectConstAssertionInArrowFunctions": true, }])),
            ),
            (
                "
            export const func1 = (value: string) => value;
            export const func2 = (value: number) => ({ type: 'X', value });
            ",
                Some(json!([{ "allowedNames": ["func1", "func2"], }])),
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
                Some(json!([{ "allowedNames": ["func1", "func2"], }])),
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
                Some(json!([{ "allowedNames": ["prop", "method", "null", "foo"], }])),
            ),
            (
                "
                    export function foo(outer: string) {
                      return function (inner: string): void {};
                    }
            ",
                Some(json!([{ "allowHigherOrderFunctions": true, }])),
            ),
            (
                "
                    export type Ensurer = (blocks: TFBlock[]) => TFBlock[];

                    export const myEnsurer: Ensurer = blocks => {
                      return blocks;
                    };
            ",
                Some(json!([{ "allowTypedFunctionExpressions": true, }])),
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
                Some(json!([{ "allowHigherOrderFunctions": true }])),
            ),
            ("export var arrowFn = () => (): void => {};", None),
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
                Some(json!([{ "allowTypedFunctionExpressions": false, }])),
            ),
            (
                "export const x = (() => {}) as Foo;",
                Some(json!([{ "allowTypedFunctionExpressions": false }])),
            ),
            (
                "
            interface Foo {}
            export const x = {
              foo: () => {},
            } as Foo;
            ",
                Some(json!([{ "allowTypedFunctionExpressions": false }])),
            ),
            (
                "export function foo(foo: any): void {}",
                Some(json!([{ "allowArgumentsExplicitlyTypedAsAny": true }])),
            ),
            (
                "export function foo({ foo }: any): void {}",
                Some(json!([{ "allowArgumentsExplicitlyTypedAsAny": true }])),
            ),
            (
                "export function foo([bar]: any): void {}",
                Some(json!([{ "allowArgumentsExplicitlyTypedAsAny": true }])),
            ),
            (
                "export function foo(...bar: any): void {}",
                Some(json!([{ "allowArgumentsExplicitlyTypedAsAny": true }])),
            ),
            (
                "export function foo(...[a]: any): void {}",
                Some(json!([{ "allowArgumentsExplicitlyTypedAsAny": true }])),
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
                Some(json!([{ "allowOverloadFunctions": true, }])),
            ),
            (
                "
            export default function test(a: string): string;
            export default function test(a: number): number;
            export default function test(a: unknown) {
              return a;
            }
            ",
                Some(json!([{ "allowOverloadFunctions": true, }])),
            ),
            (
                "
            export default function (a: string): string;
            export default function (a: number): number;
            export default function (a: unknown) {
              return a;
            }
            ",
                Some(json!([{ "allowOverloadFunctions": true, }])),
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
                Some(json!([{ "allowOverloadFunctions": true, }])),
            ),
            ("React.useEffect(() => { test() }, []);", None),
            (
                "const ex = () => (args: { fn: (arg: string) => void }) => args; export const Test = ex()({ fn: () => {} });",
                None,
            ),
            (
                "function ErrorTrackingRules(): JSX.Element { return (<BindLogic><DndContext onDragEnd={({ active, over }) => { /**/ }}></DndContext></BindLogic>); }; export default ErrorTrackingRules;",
                None,
            ),
            ("export namespace B{return}", None),
            ("function Test(): void { const _x = () => {}; } export default Test;", None),
            (
                "function Test(): void { const _x = () => { }; } function Test2() { return (): void => { }; } export { Test2 };",
                None,
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
                Some(json!([{ "allowTypedFunctionExpressions": true }])),
            ),
            (
                "export var funcExpr = function () { return 'test'; };",
                Some(json!([{ "allowTypedFunctionExpressions": true }])),
            ),
            (
                "
            interface Foo {}
            export const x: Foo = {
              foo: () => {},
            };
            ",
                Some(json!([{ "allowTypedFunctionExpressions": false }])),
            ),
            (
                "export default () => () => {};",
                Some(json!([{ "allowHigherOrderFunctions": true }])),
            ),
            (
                "export default () => function () {};",
                Some(json!([{ "allowHigherOrderFunctions": true }])),
            ),
            (
                "export default () => { return () => {}; };",
                Some(json!([{ "allowHigherOrderFunctions": true }])),
            ),
            (
                "
            export default () => {
              return function () {};
            };
            ",
                Some(json!([{ "allowHigherOrderFunctions": true }])),
            ),
            (
                "export function fn() { return () => {}; }",
                Some(json!([{ "allowHigherOrderFunctions": true }])),
            ),
            (
                "export function fn() { return function () {}; }",
                Some(json!([{ "allowHigherOrderFunctions": true }])),
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
                Some(json!([{ "allowHigherOrderFunctions": true }])),
            ),
            (
                "
            export default () => () => {
              return () => {
                return;
              };
            };
            ",
                Some(json!([{ "allowHigherOrderFunctions": true }])),
            ),
            (
                "
            export const func1 = (value: number) => ({ type: 'X', value }) as any;
            export const func2 = (value: number) => ({ type: 'X', value }) as Action;
            ",
                Some(json!([{ "allowDirectConstAssertionInArrowFunctions": true, }])),
            ),
            (
                "
            export const func = (value: number) => ({ type: 'X', value }) as const;
            ",
                Some(json!([{ "allowDirectConstAssertionInArrowFunctions": false, }])),
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
                Some(json!([{ "allowDirectConstAssertionInArrowFunctions": false, }])),
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
                Some(json!([{ "allowedNames": ["prop"],        },      ])),
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
                Some(json!([{ "allowedNames": ["func2"], }])),
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
                Some(json!([{ "allowHigherOrderFunctions": true }])),
            ),
            (
                "export const baz = arg => arg as const;",
                Some(json!([{ "allowDirectConstAssertionInArrowFunctions": true }])),
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
            // FIXME: resolve to last write reference
            // This test case requires tracking variable reassignments and checking the
            // type of the last assignment before export, not the first assignment.
            // Currently not implemented due to complexity.
            // (
            //     "
            // let foo = (arg: number): number => arg;
            // foo = arg => arg;
            // export default foo;
            // ",
            //     None,
            // ),
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
                Some(json!([{ "allowHigherOrderFunctions": false }])),
            ),
            (
                "export var arrowFn = () => () => {};",
                Some(json!([{ "allowHigherOrderFunctions": true }])),
            ),
            (
                "
            export function fn() {
              return function () {};
            }
",
                Some(json!([{ "allowHigherOrderFunctions": true }])),
            ),
            (
                "
            export function foo(outer) {
              return function (inner): void {};
            }
            ",
                Some(json!([{ "allowHigherOrderFunctions": true }])),
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
                Some(json!([{ "allowHigherOrderFunctions": true }])),
            ),
            ("export function foo({ foo }): void {}", None),
            ("export function foo([bar]): void {}", None),
            ("export function foo(...bar): void {}", None),
            ("export function foo(...[a]): void {}", None),
            (
                "export function foo(foo: any): void {}",
                Some(json!([{ "allowArgumentsExplicitlyTypedAsAny": false }])),
            ),
            (
                "export function foo({ foo }: any): void {}",
                Some(json!([{ "allowArgumentsExplicitlyTypedAsAny": false }])),
            ),
            (
                "export function foo([bar]: any): void {}",
                Some(json!([{ "allowArgumentsExplicitlyTypedAsAny": false }])),
            ),
            (
                "export function foo(...bar: any): void {}",
                Some(json!([{ "allowArgumentsExplicitlyTypedAsAny": false }])),
            ),
            (
                "export function foo(...[a]: any): void {}",
                Some(json!([{ "allowArgumentsExplicitlyTypedAsAny": false }])),
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
                Some(json!([{ "allowedNames": [],        },      ])),
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
            (
                "function Test(): void { const _x = () => { }; } function Test2() { return () => { }; } export { Test2 };",
                None,
            ),
            ("function App() { return 42; } export default App", None),
        ];

        Tester::new(
            ExplicitModuleBoundaryTypes::NAME,
            ExplicitModuleBoundaryTypes::PLUGIN,
            pass,
            fail,
        )
        .test_and_snapshot();
    }

    #[test]
    fn rule_typescript_angle_bracket_type_assertions() {
        let pass = vec![
            (
                "const x = <Foo>(() => {});",
                Some(json!([{ "allowTypedFunctionExpressions": true }])),
                None,
                Some(PathBuf::from("test.ts")),
            ),
            (
                "
                export const x = <Foo>{
                  foo: () => {},
                };
                ",
                Some(json!([{ "allowTypedFunctionExpressions": true }])),
                None,
                Some(PathBuf::from("test.ts")),
            ),
            (
                "
                export const x = <Foo>{
                  foo: { bar: () => {} },
                };
                ",
                Some(json!([{ "allowTypedFunctionExpressions": true }])),
                None,
                Some(PathBuf::from("test.ts")),
            ),
        ];

        Tester::new(
            ExplicitModuleBoundaryTypes::NAME,
            ExplicitModuleBoundaryTypes::PLUGIN,
            pass,
            vec![],
        )
        .test();
    }
}
