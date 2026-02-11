use oxc_ast::{
    AstKind,
    ast::{
        ClassElement, Declaration, ExportDefaultDeclarationKind, Function, FunctionType,
        MethodDefinition, ModuleDeclaration, Statement, TSType, TSUnionType,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

fn invalid_void_for_generic_diagnostic(span: Span, generic: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Do not use `void` as a type argument for `{generic}`."))
        .with_help(
            "Replace this `void` type with an allowed type, or keep `void` only in a valid return position.",
        )
        .with_label(span)
}

fn invalid_void_not_return_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `void` only as a return type.")
        .with_help(
            "Replace this `void` type with an allowed type, or keep `void` only in a valid return position.",
        )
        .with_label(span)
}

fn invalid_void_not_return_or_generic_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `void` only as a return type or generic type argument.")
        .with_help(
            "Replace this `void` type with an allowed type, or keep `void` only in a valid return position.",
        )
        .with_label(span)
}

fn invalid_void_not_return_or_this_param_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use `void` only as a return type or as the type of a `this` parameter.")
        .with_help(
            "Replace this `void` type with an allowed type, or keep `void` only in a valid return position.",
        )
        .with_label(span)
}

fn invalid_void_not_return_or_this_param_or_generic_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Use `void` only as a return type, generic type argument, or the type of a `this` parameter.",
    )
    .with_help(
        "Replace this `void` type with an allowed type, or keep `void` only in a valid return position.",
    )
    .with_label(span)
}

fn invalid_void_union_constituent_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Remove `void` from this union type constituent.")
        .with_help(
            "Replace this `void` type with an allowed type, or keep `void` only in a valid return position.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoInvalidVoidType(Box<NoInvalidVoidTypeConfig>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum AllowInGenericTypeArguments {
    Boolean(bool),
    AllowList(Vec<String>),
}

impl Default for AllowInGenericTypeArguments {
    fn default() -> Self {
        Self::Boolean(true)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoInvalidVoidTypeConfig {
    /// Whether a `this` parameter of a function may be `void`.
    pub allow_as_this_parameter: bool,
    /// Whether `void` can be used as generic type arguments.
    /// Can be `true` / `false`, or an allowlist of generic type names.
    pub allow_in_generic_type_arguments: AllowInGenericTypeArguments,
}

impl NoInvalidVoidTypeConfig {
    fn allow_in_generic_type_arguments_enabled(&self) -> bool {
        !matches!(self.allow_in_generic_type_arguments, AllowInGenericTypeArguments::Boolean(false))
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `void` type usage outside return types and configured generic contexts.
    ///
    /// ### Why is this bad?
    ///
    /// In TypeScript, `void` is primarily meaningful in return positions. Using `void` in other
    /// type locations (parameters, properties, aliases, and most unions) is usually confusing and
    /// often indicates a mistaken type design.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// function takeVoid(arg: void) {}
    /// type Alias = void;
    /// type Union = string | void;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// function f(): void {}
    /// type P = Promise<void>;
    /// type U = void | never;
    /// ```
    NoInvalidVoidType,
    typescript,
    restriction,
    none,
    config = NoInvalidVoidTypeConfig,
);

impl Rule for NoInvalidVoidType {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(&*self.0))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSVoidKeyword(keyword) = node.kind() else {
            return;
        };

        let parent = ctx.nodes().parent_node(node.id());
        let grand_parent = ctx.nodes().ancestors(node.id()).nth(1);

        if let AstKind::TSTypeParameterInstantiation(_) = parent.kind() {
            let grand_parent_node = ctx.nodes().parent_node(parent.id());
            if let AstKind::TSTypeReference(type_reference) = grand_parent_node.kind() {
                self.check_generic_type_argument(
                    keyword.span,
                    ctx.source_range(type_reference.type_name.span()),
                    ctx,
                );
                return;
            }

            if matches!(grand_parent_node.kind(), AstKind::NewExpression(_)) {
                match self.0.allow_in_generic_type_arguments {
                    AllowInGenericTypeArguments::Boolean(true) => return,
                    AllowInGenericTypeArguments::Boolean(false) => {
                        ctx.diagnostic(self.not_return_for_disabled_generics(keyword.span));
                        return;
                    }
                    AllowInGenericTypeArguments::AllowList(_) => {}
                }
            }
        }

        if self.0.allow_in_generic_type_arguments_enabled()
            && let AstKind::TSTypeParameter(type_parameter) = parent.kind()
        {
            if type_parameter.default.as_ref().is_some_and(|default| default.span() == keyword.span)
            {
                return;
            }

            ctx.diagnostic(if matches!(parent.kind(), AstKind::TSUnionType(_)) {
                invalid_void_union_constituent_diagnostic(keyword.span)
            } else {
                invalid_void_not_return_or_generic_diagnostic(keyword.span)
            });
            return;
        }

        if let AstKind::TSUnionType(union_type) = parent.kind() {
            if is_valid_union_type(union_type) {
                return;
            }

            if let Some(decl) = get_parent_function_declaration_node(node, ctx)
                && is_union_within_function_return_type(union_type.span, decl)
                && has_overload_signatures(decl, ctx)
            {
                return;
            }
        }

        if self.0.allow_as_this_parameter
            && matches!(parent.kind(), AstKind::TSTypeAnnotation(_))
            && matches!(grand_parent.map(AstNode::kind), Some(AstKind::TSThisParameter(_)))
        {
            return;
        }

        if is_valid_return_type_annotation(parent.kind(), grand_parent.map(AstNode::kind)) {
            return;
        }

        let in_union = matches!(parent.kind(), AstKind::TSUnionType(_));
        ctx.diagnostic(self.not_return_for_options(keyword.span, in_union));
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

impl NoInvalidVoidType {
    fn check_generic_type_argument(
        &self,
        span: Span,
        fully_qualified_name: &str,
        ctx: &LintContext<'_>,
    ) {
        match &self.0.allow_in_generic_type_arguments {
            AllowInGenericTypeArguments::AllowList(allow_list) => {
                let normalized_name = remove_spaces(fully_qualified_name);
                if allow_list.iter().all(|allow| remove_spaces(allow.as_str()) != normalized_name) {
                    ctx.diagnostic(invalid_void_for_generic_diagnostic(span, &normalized_name));
                }
            }
            AllowInGenericTypeArguments::Boolean(false) => {
                ctx.diagnostic(self.not_return_for_disabled_generics(span));
            }
            AllowInGenericTypeArguments::Boolean(true) => {}
        }
    }

    fn not_return_for_disabled_generics(&self, span: Span) -> OxcDiagnostic {
        if self.0.allow_as_this_parameter {
            invalid_void_not_return_or_this_param_diagnostic(span)
        } else {
            invalid_void_not_return_diagnostic(span)
        }
    }

    fn not_return_for_options(&self, span: Span, in_union: bool) -> OxcDiagnostic {
        if self.0.allow_in_generic_type_arguments_enabled() && self.0.allow_as_this_parameter {
            invalid_void_not_return_or_this_param_or_generic_diagnostic(span)
        } else if self.0.allow_in_generic_type_arguments_enabled() {
            if in_union {
                invalid_void_union_constituent_diagnostic(span)
            } else {
                invalid_void_not_return_or_generic_diagnostic(span)
            }
        } else if self.0.allow_as_this_parameter {
            invalid_void_not_return_or_this_param_diagnostic(span)
        } else {
            invalid_void_not_return_diagnostic(span)
        }
    }
}

fn is_valid_return_type_annotation(parent: AstKind<'_>, grand_parent: Option<AstKind<'_>>) -> bool {
    let AstKind::TSTypeAnnotation(type_annotation) = parent else {
        return false;
    };

    let Some(grand_parent) = grand_parent else {
        return false;
    };

    match grand_parent {
        AstKind::Function(function) => {
            function.return_type.as_ref().is_some_and(|ret| ret.span == type_annotation.span)
        }
        AstKind::ArrowFunctionExpression(function) => {
            function.return_type.as_ref().is_some_and(|ret| ret.span == type_annotation.span)
        }
        AstKind::TSFunctionType(function) => function.return_type.span == type_annotation.span,
        AstKind::TSConstructorType(function) => function.return_type.span == type_annotation.span,
        AstKind::TSCallSignatureDeclaration(signature) => {
            signature.return_type.as_ref().is_some_and(|ret| ret.span == type_annotation.span)
        }
        AstKind::TSMethodSignature(signature) => {
            signature.return_type.as_ref().is_some_and(|ret| ret.span == type_annotation.span)
        }
        AstKind::TSConstructSignatureDeclaration(signature) => {
            signature.return_type.as_ref().is_some_and(|ret| ret.span == type_annotation.span)
        }
        _ => false,
    }
}

fn is_valid_union_type(union_type: &TSUnionType<'_>) -> bool {
    union_type.types.iter().all(|member| {
        matches!(member, TSType::TSVoidKeyword(_) | TSType::TSNeverKeyword(_))
            || matches!(
                member,
                TSType::TSTypeReference(type_reference)
                    if type_reference.type_arguments.as_ref().is_some_and(|type_args| {
                        type_args.params.iter().any(|param| matches!(param, TSType::TSVoidKeyword(_)))
                    })
            )
    })
}

#[derive(Clone, Copy)]
enum ParentFunctionDeclarationNode<'n, 'a> {
    Function(&'n AstNode<'a>, &'n Function<'a>),
    Method(&'n AstNode<'a>, &'n MethodDefinition<'a>),
}

fn get_parent_function_declaration_node<'n, 'a>(
    node: &'n AstNode<'a>,
    ctx: &'n LintContext<'a>,
) -> Option<ParentFunctionDeclarationNode<'n, 'a>> {
    for ancestor in ctx.nodes().ancestors(node.id()) {
        match ancestor.kind() {
            AstKind::MethodDefinition(method) if method.value.body.is_some() => {
                return Some(ParentFunctionDeclarationNode::Method(ancestor, method));
            }
            AstKind::Function(function) if function.body.is_some() => {
                if matches!(
                    ctx.nodes().parent_node(ancestor.id()).kind(),
                    AstKind::MethodDefinition(_)
                ) {
                    continue;
                }
                return Some(ParentFunctionDeclarationNode::Function(ancestor, function));
            }
            _ => {}
        }
    }
    None
}

fn is_union_within_function_return_type(
    union_span: Span,
    declaration: ParentFunctionDeclarationNode<'_, '_>,
) -> bool {
    match declaration {
        ParentFunctionDeclarationNode::Function(_, function) => {
            function.return_type.as_ref().is_some_and(|ret| span_contains(ret.span, union_span))
        }
        ParentFunctionDeclarationNode::Method(_, method) => {
            method.value.return_type.as_ref().is_some_and(|ret| span_contains(ret.span, union_span))
        }
    }
}

fn has_overload_signatures(
    declaration: ParentFunctionDeclarationNode<'_, '_>,
    ctx: &LintContext<'_>,
) -> bool {
    match declaration {
        ParentFunctionDeclarationNode::Function(function_node, function) => {
            has_function_overload_signatures(function_node, function, ctx)
        }
        ParentFunctionDeclarationNode::Method(method_node, method) => {
            has_method_overload_signatures(method_node, method, ctx)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FunctionStatementWrapper {
    Plain,
    ExportNamed,
    ExportDefault,
}

fn has_function_overload_signatures(
    function_node: &AstNode<'_>,
    function: &Function<'_>,
    ctx: &LintContext<'_>,
) -> bool {
    let wrapper = match ctx.nodes().parent_kind(function_node.id()) {
        AstKind::ExportNamedDeclaration(_) => FunctionStatementWrapper::ExportNamed,
        AstKind::ExportDefaultDeclaration(_) => FunctionStatementWrapper::ExportDefault,
        _ => FunctionStatementWrapper::Plain,
    };
    let impl_name = function.id.as_ref().map(|id| id.name.as_str());

    for ancestor in ctx.nodes().ancestors(function_node.id()) {
        let has_match = match ancestor.kind() {
            AstKind::Program(program) => {
                has_overload_in_statements(&program.body, wrapper, impl_name)
            }
            AstKind::TSModuleBlock(block) => {
                has_overload_in_statements(&block.body, wrapper, impl_name)
            }
            AstKind::BlockStatement(block) => {
                has_overload_in_statements(&block.body, wrapper, impl_name)
            }
            AstKind::FunctionBody(body) => {
                has_overload_in_statements(&body.statements, wrapper, impl_name)
            }
            _ => false,
        };
        if has_match {
            return true;
        }
    }

    false
}

fn has_overload_in_statements(
    statements: &[Statement<'_>],
    impl_wrapper: FunctionStatementWrapper,
    impl_name: Option<&str>,
) -> bool {
    statements.iter().any(|statement| {
        let Some((candidate, candidate_wrapper)) = statement_function_candidate(statement) else {
            return false;
        };
        if candidate_wrapper != impl_wrapper {
            return false;
        }
        if candidate.body.is_some() && !matches!(candidate.r#type, FunctionType::TSDeclareFunction)
        {
            return false;
        }

        match (impl_wrapper, impl_name, candidate.id.as_ref()) {
            (FunctionStatementWrapper::ExportDefault, None, None) => true,
            (_, Some(name), Some(candidate_name)) => candidate_name.name == name,
            _ => false,
        }
    })
}

fn statement_function_candidate<'a>(
    statement: &'a Statement<'a>,
) -> Option<(&'a Function<'a>, FunctionStatementWrapper)> {
    if let Some(declaration) = statement.as_declaration()
        && let Declaration::FunctionDeclaration(function) = declaration
    {
        return Some((function, FunctionStatementWrapper::Plain));
    }

    if let Some(module_decl) = statement.as_module_declaration() {
        match module_decl {
            ModuleDeclaration::ExportNamedDeclaration(named_decl) => {
                if let Some(Declaration::FunctionDeclaration(function)) = &named_decl.declaration {
                    return Some((function, FunctionStatementWrapper::ExportNamed));
                }
            }
            ModuleDeclaration::ExportDefaultDeclaration(default_decl) => {
                if let ExportDefaultDeclarationKind::FunctionDeclaration(function) =
                    &default_decl.declaration
                {
                    return Some((function, FunctionStatementWrapper::ExportDefault));
                }
            }
            _ => {}
        }
    }

    None
}

fn has_method_overload_signatures(
    method_node: &AstNode<'_>,
    method: &MethodDefinition<'_>,
    ctx: &LintContext<'_>,
) -> bool {
    let parent = ctx.nodes().parent_node(method_node.id());
    let AstKind::ClassBody(class_body) = parent.kind() else {
        return false;
    };

    class_body.body.iter().any(|element| {
        let ClassElement::MethodDefinition(candidate) = element else {
            return false;
        };
        if candidate.span == method.span {
            return false;
        }
        if candidate.value.body.is_some()
            && !matches!(candidate.value.r#type, FunctionType::TSEmptyBodyFunctionExpression)
        {
            return false;
        }
        is_same_method_signature(method, candidate, ctx)
    })
}

fn is_same_method_signature(
    current: &MethodDefinition<'_>,
    candidate: &MethodDefinition<'_>,
    ctx: &LintContext<'_>,
) -> bool {
    current.r#static == candidate.r#static
        && remove_spaces(ctx.source_range(current.key.span()))
            == remove_spaces(ctx.source_range(candidate.key.span()))
}

fn span_contains(outer: Span, inner: Span) -> bool {
    inner.start >= outer.start && inner.end <= outer.end
}

fn remove_spaces(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "type Generic<T> = [T];",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": false }])),
        ),
        (
            "
            function foo(): void | never {
              throw new Error('Test');
            }
                  ",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": false }])),
        ),
        (
            "type voidNeverUnion = void | never;",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": false }])),
        ),
        (
            "type neverVoidUnion = never | void;",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": false }])),
        ),
        ("function func(): void {}", None),
        ("type NormalType = () => void;", None),
        ("let normalArrow = (): void => {};", None),
        ("let ughThisThing = void 0;", None),
        ("function takeThing(thing: undefined) {}", None),
        ("takeThing(void 0);", None),
        ("let voidPromise: Promise<void> = new Promise<void>(() => {});", None),
        ("let voidMap: Map<string, void> = new Map<string, void>();", None),
        (
            "
                  function returnsVoidPromiseDirectly(): Promise<void> {
                    return Promise.resolve();
                  }
                ",
            None,
        ),
        ("async function returnsVoidPromiseAsync(): Promise<void> {}", None),
        ("type UnionType = string | number;", None),
        ("type GenericVoid = Generic<void>;", None),
        ("type Generic<T> = [T];", None),
        ("type voidPromiseUnion = void | Promise<void>;", None),
        ("type promiseNeverUnion = Promise<void> | never;", None),
        ("const arrowGeneric1 = <T = void,>(arg: T) => {};", None),
        ("declare function functionDeclaration1<T = void>(arg: T): void;", None),
        (
            "
                  class ClassName {
                    accessor propName: number;
                  }
                ",
            None,
        ),
        (
            "
            function f(): void;
            function f(x: string): string;
            function f(x?: string): string | void {
              if (x !== undefined) {
                return x;
              }
            }
                ",
            None,
        ),
        (
            "
            class SomeClass {
              f(): void;
              f(x: string): string;
              f(x?: string): string | void {
                if (x !== undefined) {
                  return x;
                }
              }
            }
                ",
            None,
        ),
        (
            "
            class SomeClass {
              ['f'](): void;
              ['f'](x: string): string;
              ['f'](x?: string): string | void {
                if (x !== undefined) {
                  return x;
                }
              }
            }
                ",
            None,
        ),
        (
            "
            class SomeClass {
              [Symbol.iterator](): void;
              [Symbol.iterator](x: string): string;
              [Symbol.iterator](x?: string): string | void {
                if (x !== undefined) {
                  return x;
                }
              }
            }
                ",
            None,
        ),
        (
            "
            class SomeClass {
              'f'(): void;
              'f'(x: string): string;
              'f'(x?: string): string | void {
                if (x !== undefined) {
                  return x;
                }
              }
            }
                ",
            None,
        ),
        (
            "
            class SomeClass {
              1(): void;
              1(x: string): string;
              1(x?: string): string | void {
                if (x !== undefined) {
                  return x;
                }
              }
            }
                ",
            None,
        ),
        (
            "
            const staticSymbol = Symbol.for('static symbol');

            class SomeClass {
              [staticSymbol](): void;
              [staticSymbol](x: string): string;
              [staticSymbol](x?: string): string | void {
                if (x !== undefined) {
                  return x;
                }
              }
            }
                ",
            None,
        ),
        (
            "
            declare module foo {
              function f(): void;
              function f(x: string): string;
              function f(x?: string): string | void {
                if (x !== undefined) {
                  return x;
                }
              }
            }
                ",
            None,
        ),
        (
            "
            {
              function f(): void;
              function f(x: string): string;
              function f(x?: string): string | void {
                if (x !== undefined) {
                  return x;
                }
              }
            }
                ",
            None,
        ),
        (
            "
            function f(): Promise<void>;
            function f(x: string): Promise<string>;
            async function f(x?: string): Promise<void | string> {
              if (x !== undefined) {
                return x;
              }
            }
                ",
            None,
        ),
        (
            "
            class SomeClass {
              f(): Promise<void>;
              f(x: string): Promise<string>;
              async f(x?: string): Promise<void | string> {
                if (x !== undefined) {
                  return x;
                }
              }
            }
                ",
            None,
        ),
        (
            "
            function f(): void;

            const a = 5;

            function f(x: string): string;
            function f(x?: string): string | void {
              if (x !== undefined) {
                return x;
              }
            }
                ",
            None,
        ),
        (
            "
            export default function (): void;
            export default function (x: string): string;
            export default function (x?: string): string | void {
              if (x !== undefined) {
                return x;
              }
            }
                ",
            None,
        ),
        (
            "
            export function f(): void;
            export function f(x: string): string;
            export function f(x?: string): string | void {
              if (x !== undefined) {
                return x;
              }
            }
                ",
            None,
        ),
        (
            "
            export {};

            export function f(): void;
            export function f(x: string): string;
            export function f(x?: string): string | void {
              if (x !== undefined) {
                return x;
              }
            }
                ",
            None,
        ),
        ("type Allowed<T> = [T];", None),
        ("type Banned<T> = [T];", None),
        (
            "type AllowedVoid = Allowed<void>;",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": ["Allowed"] }])),
        ),
        (
            "type AllowedVoid = Ex.Mx.Tx<void>;",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": ["Ex.Mx.Tx"] }])),
        ),
        (
            "type AllowedVoid = Ex . Mx . Tx<void>;",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": ["Ex.Mx.Tx"] }])),
        ),
        (
            "type AllowedVoid = Ex . Mx . Tx<void>;",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": ["Ex.Mx . Tx"] }])),
        ),
        (
            "type AllowedVoid = Ex.Mx.Tx<void>;",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": ["Ex . Mx . Tx"] }])),
        ),
        (
            "type voidPromiseUnion = void | Promise<void>;",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": ["Promise"] }])),
        ),
        (
            "type promiseVoidUnion = Promise<void> | void;",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": ["Promise"] }])),
        ),
        (
            "
            async function foo(bar: () => void | Promise<void>) {
              await bar();
            }
                  ",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": ["Promise"] }])),
        ),
        (
            "type promiseNeverUnion = Promise<void> | never;",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": ["Promise"] }])),
        ),
        (
            "type voidPromiseNeverUnion = void | Promise<void> | never;",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": ["Promise"] }])),
        ),
        ("function f(this: void) {}", Some(serde_json::json!([{ "allowAsThisParameter": true }]))),
        (
            "
            class Test {
              public static helper(this: void) {}
              method(this: void) {}
            }
                  ",
            Some(serde_json::json!([{ "allowAsThisParameter": true }])),
        ),
    ];

    let fail = vec![
        (
            "type GenericVoid = Generic<void>;",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": false }])),
        ),
        (
            "function takeVoid(thing: void) {}",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": false }])),
        ),
        (
            "let voidPromise: Promise<void> = new Promise<void>(() => {});",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": false }])),
        ),
        (
            "let voidMap: Map<string, void> = new Map<string, void>();",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": false }])),
        ),
        (
            "type invalidVoidUnion = void | number;",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": false }])),
        ),
        ("function takeVoid(thing: void) {}", None),
        ("const arrowGeneric = <T extends void>(arg: T) => {};", None),
        ("const arrowGeneric2 = <T extends void = void>(arg: T) => {};", None),
        ("function functionGeneric<T extends void>(arg: T) {}", None),
        ("function functionGeneric2<T extends void = void>(arg: T) {}", None),
        ("declare function functionDeclaration<T extends void>(arg: T): void;", None),
        ("declare function functionDeclaration2<T extends void = void>(arg: T): void;", None),
        ("functionGeneric<void>(undefined);", None),
        ("declare function voidArray(args: void[]): void[];", None),
        ("let value = undefined as void;", None),
        ("let value = <void>undefined;", None),
        ("function takesThings(...things: void[]): void {}", None),
        ("type KeyofVoid = keyof void;", None),
        (
            "
                    interface Interface {
                      lambda: () => void;
                      voidProp: void;
                    }
                  ",
            None,
        ),
        (
            "
                    class ClassName {
                      private readonly propName: void;
                    }
                  ",
            None,
        ),
        (
            "
                    class ClassName {
                      accessor propName: void;
                    }
                  ",
            None,
        ),
        ("let letVoid: void;", None),
        (
            "
                    type VoidType = void;
                    class OtherClassName {
                      private propName: VoidType;
                    }
                  ",
            None,
        ),
        ("type UnionType2 = string | number | void;", None),
        ("type UnionType3 = string | ((number & any) | (string | void));", None),
        ("declare function test(): number | void;", None),
        ("declare function test<T extends number | void>(): T;", None),
        ("type IntersectionType = string & number & void;", None),
        (
            "
                    type MappedType<T> = {
                      [K in keyof T]: void;
                    };
                  ",
            None,
        ),
        (
            "
                    type ConditionalType<T> = {
                      [K in keyof T]: T[K] extends string ? void : string;
                    };
                  ",
            None,
        ),
        ("type ManyVoid = readonly void[];", None),
        ("function foo(arr: readonly void[]) {}", None),
        ("type invalidVoidUnion = void | Map<string, number>;", None),
        ("type invalidVoidUnion = void | Map;", None),
        (
            "
            class SomeClass {
              f(x?: string): string | void {
                if (x !== undefined) {
                  return x;
                }
              }
            }
                  ",
            None,
        ),
        ("export default function (x?: string): string | void {}", None),
        ("export function f(x?: string): string | void {}", None),
        (
            "
            function f(): void;
            function f(x: string): string | void;
            function f(x?: string): string | void {
              if (x !== undefined) {
                return x;
              }
            }
                  ",
            None,
        ),
        (
            "
            class SomeClass {
              f(): void;
              f(x: string): string | void;
              f(x?: string): string | void {
                if (x !== undefined) {
                  return x;
                }
              }
            }
                  ",
            None,
        ),
        (
            "type BannedVoid = Banned<void>;",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": ["Allowed"] }])),
        ),
        (
            "type BannedVoid = Ex.Mx.Tx<void>;",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": ["Tx"] }])),
        ),
        (
            "function takeVoid(thing: void) {}",
            Some(serde_json::json!([{ "allowInGenericTypeArguments": ["Allowed"] }])),
        ),
        (
            "type alias = void;",
            Some(
                serde_json::json!([ { "allowAsThisParameter": true, "allowInGenericTypeArguments": true }, ]),
            ),
        ),
        (
            "type alias = void;",
            Some(
                serde_json::json!([ { "allowAsThisParameter": true, "allowInGenericTypeArguments": false }, ]),
            ),
        ),
        (
            "type alias = Array<void>;",
            Some(
                serde_json::json!([ { "allowAsThisParameter": true, "allowInGenericTypeArguments": false }, ]),
            ),
        ),
    ];

    Tester::new(NoInvalidVoidType::NAME, NoInvalidVoidType::PLUGIN, pass, fail).test_and_snapshot();
}
