use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{
        TSMethodSignature, TSMethodSignatureKind, TSPropertySignature, TSSignature, TSThisType,
        TSType, TSTypeAnnotation,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

fn method_signature_style_error_method_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Shorthand method signature is forbidden. Use a function property instead.")
        .with_label(span)
}

fn method_signature_style_error_property_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Function property signature is forbidden. Use a method shorthand instead.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum MethodSignatureStyleConfig {
    /// Enforce method signatures to use the method shorthand syntax (e.g., `foo(): void;`).
    Method,
    /// Enforce method signatures to use the function property syntax (e.g., `foo: () => void;`).
    #[default]
    Property,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct MethodSignatureStyle(MethodSignatureStyleConfig);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a consistent method signature syntax in TypeScript types.
    ///
    /// ### Why is this bad?
    ///
    /// Mixing shorthand method signatures and function-property signatures in type members
    /// reduces readability and consistency.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with default `"property"`:
    /// ```ts
    /// interface Foo {
    ///   bar(a: string): number;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// interface Foo {
    ///   bar: (a: string) => number;
    /// }
    /// ```
    MethodSignatureStyle,
    typescript,
    style,
    conditional_fix,
    config = MethodSignatureStyleConfig,
);

impl Rule for MethodSignatureStyle {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::TSMethodSignature(method_node)
                if self.0 == MethodSignatureStyleConfig::Property =>
            {
                check_method_signature_style_property(method_node, node, ctx);
            }
            AstKind::TSPropertySignature(property_node)
                if self.0 == MethodSignatureStyleConfig::Method =>
            {
                check_method_signature_style_method(property_node, ctx);
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn check_method_signature_style_property<'a>(
    method_node: &TSMethodSignature<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) {
    if method_node.kind != TSMethodSignatureKind::Method {
        return;
    }

    let source_text = ctx.source_text();
    let method_key = get_method_key(
        method_node.key.span(),
        method_node.computed,
        method_node.optional,
        false,
        source_text,
    );
    let skip_fix = return_type_references_this_type(method_node.return_type.as_deref());
    let is_parent_module = is_parent_module_declaration(node.id(), ctx);

    let (method_signatures, current_index) = sibling_method_signatures(node.id(), method_node, ctx);
    let duplicated_method_nodes = method_signatures
        .into_iter()
        .filter(|(_, method)| {
            !std::ptr::eq(*method, method_node)
                && get_method_key(
                    method.key.span(),
                    method.computed,
                    method.optional,
                    false,
                    source_text,
                ) == method_key
        })
        .collect::<Vec<_>>();

    if is_parent_module || skip_fix {
        ctx.diagnostic(method_signature_style_error_method_diagnostic(method_node.span));
        return;
    }

    if duplicated_method_nodes.is_empty() {
        ctx.diagnostic_with_fix(
            method_signature_style_error_method_diagnostic(method_node.span),
            |fixer| {
                let key = get_method_key(
                    method_node.key.span(),
                    method_node.computed,
                    method_node.optional,
                    false,
                    source_text,
                );
                let params = get_method_params(
                    method_node.params.span,
                    method_node
                        .type_parameters
                        .as_ref()
                        .map(|type_parameters| type_parameters.span),
                    source_text,
                );
                let return_type =
                    get_method_return_type(method_node.return_type.as_deref(), source_text);
                let delimiter = get_delimiter(method_node.span, source_text);

                fixer.replace(
                    method_node.span,
                    format!("{key}: {params} => {return_type}{delimiter}"),
                )
            },
        );
        return;
    }

    ctx.diagnostic_with_fix(
        method_signature_style_error_method_diagnostic(method_node.span),
        |fixer| {
            let mut method_nodes = Vec::with_capacity(duplicated_method_nodes.len() + 1);
            method_nodes.push((current_index, method_node));
            method_nodes
                .extend(duplicated_method_nodes.iter().map(|(idx, method)| (*idx, *method)));
            method_nodes.sort_unstable_by_key(|(_, method)| method.span.start);

            let type_string = method_nodes
                .iter()
                .map(|(_, node)| {
                    let params = get_method_params(
                        node.params.span,
                        node.type_parameters.as_ref().map(|type_parameters| type_parameters.span),
                        source_text,
                    );
                    let return_type =
                        get_method_return_type(node.return_type.as_deref(), source_text);
                    format!("({params} => {return_type})")
                })
                .collect::<Vec<_>>()
                .join(" & ");

            let key = get_method_key(
                method_node.key.span(),
                method_node.computed,
                method_node.optional,
                false,
                source_text,
            );
            let delimiter = get_delimiter(method_node.span, source_text);

            let is_contiguous =
                method_nodes.windows(2).all(|window| window[1].0 == window[0].0 + 1);
            if is_contiguous {
                let first = method_nodes.first().unwrap().1;
                let last = method_nodes.last().unwrap().1;
                fixer.replace(
                    Span::new(first.span.start, last.span.end),
                    format!("{key}: {type_string}{delimiter}"),
                )
            } else {
                let multifix = fixer.for_multifix();
                let mut fix = multifix.new_fix_with_capacity(duplicated_method_nodes.len() + 1);
                fix.push(
                    multifix.replace(method_node.span, format!("{key}: {type_string}{delimiter}")),
                );
                for (_, duplicated_node) in &duplicated_method_nodes {
                    fix.push(multifix.delete(*duplicated_node));
                }
                fix
            }
        },
    );
}

fn check_method_signature_style_method(
    property_node: &TSPropertySignature<'_>,
    ctx: &LintContext<'_>,
) {
    let Some(type_annotation) = &property_node.type_annotation else {
        return;
    };
    let TSType::TSFunctionType(function_type) = &type_annotation.type_annotation else {
        return;
    };

    let source_text = ctx.source_text();
    ctx.diagnostic_with_fix(
        method_signature_style_error_property_diagnostic(property_node.span),
        |fixer| {
            let key = get_method_key(
                property_node.key.span(),
                property_node.computed,
                property_node.optional,
                property_node.readonly,
                source_text,
            );
            let params = get_method_params(
                function_type.params.span,
                function_type.type_parameters.as_ref().map(|type_parameters| type_parameters.span),
                source_text,
            );
            let return_type =
                function_type.return_type.type_annotation.span().source_text(source_text);
            let delimiter = get_delimiter(property_node.span, source_text);

            fixer.replace(property_node.span, format!("{key}{params}: {return_type}{delimiter}"))
        },
    );
}

fn sibling_method_signatures<'a>(
    node_id: oxc_semantic::NodeId,
    current_method: &TSMethodSignature<'a>,
    ctx: &LintContext<'a>,
) -> (Vec<(usize, &'a TSMethodSignature<'a>)>, usize) {
    let mut current_index = 0usize;
    let mut methods = vec![];

    match ctx.nodes().parent_kind(node_id) {
        AstKind::TSInterfaceBody(body) => {
            for (index, member) in body.body.iter().enumerate() {
                let TSSignature::TSMethodSignature(method) = member else {
                    continue;
                };
                if method.kind != TSMethodSignatureKind::Method {
                    continue;
                }
                if std::ptr::eq(method.as_ref(), current_method) {
                    current_index = index;
                }
                methods.push((index, method.as_ref()));
            }
        }
        AstKind::TSTypeLiteral(type_literal) => {
            for (index, member) in type_literal.members.iter().enumerate() {
                let TSSignature::TSMethodSignature(method) = member else {
                    continue;
                };
                if method.kind != TSMethodSignatureKind::Method {
                    continue;
                }
                if std::ptr::eq(method.as_ref(), current_method) {
                    current_index = index;
                }
                methods.push((index, method.as_ref()));
            }
        }
        _ => {}
    }

    (methods, current_index)
}

fn get_method_key(
    key_span: Span,
    computed: bool,
    optional: bool,
    readonly: bool,
    source_text: &str,
) -> String {
    let mut key = key_span.source_text(source_text).to_string();
    if computed {
        key = format!("[{key}]");
    }
    if optional {
        key.push('?');
    }
    if readonly {
        key = format!("readonly {key}");
    }
    key
}

fn get_method_params(
    params_span: Span,
    type_parameters_span: Option<Span>,
    source_text: &str,
) -> String {
    let mut params = params_span.source_text(source_text).to_string();
    if let Some(type_parameters_span) = type_parameters_span {
        params = format!("{}{}", type_parameters_span.source_text(source_text), params);
    }
    params
}

fn get_method_return_type(return_type: Option<&TSTypeAnnotation<'_>>, source_text: &str) -> String {
    return_type.map_or_else(
        || "any".to_string(),
        |return_type| return_type.type_annotation.span().source_text(source_text).to_string(),
    )
}

fn get_delimiter(node_span: Span, source_text: &str) -> &'static str {
    let text = node_span.source_text(source_text).trim_end();
    if text.ends_with(';') {
        ";"
    } else if text.ends_with(',') {
        ","
    } else {
        ""
    }
}

fn is_parent_module_declaration(node_id: oxc_semantic::NodeId, ctx: &LintContext<'_>) -> bool {
    ctx.nodes()
        .ancestors(node_id)
        .any(|ancestor| matches!(ancestor.kind(), AstKind::TSModuleDeclaration(_)))
}

fn return_type_references_this_type(return_type: Option<&TSTypeAnnotation<'_>>) -> bool {
    let Some(return_type) = return_type else {
        return false;
    };

    let mut finder = ThisTypeFinder::default();
    finder.visit_ts_type(&return_type.type_annotation);
    finder.found
}

#[derive(Default)]
struct ThisTypeFinder {
    found: bool,
}

impl<'a> Visit<'a> for ThisTypeFinder {
    fn visit_ts_this_type(&mut self, _it: &TSThisType) {
        self.found = true;
    }

    fn visit_ts_type(&mut self, it: &TSType<'a>) {
        if self.found {
            return;
        }
        walk::walk_ts_type(self, it);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
            interface Test {
              f: (a: string) => number;
            }
                ",
            None,
        ),
        (
            "
            interface Test {
              ['f']: (a: boolean) => void;
            }
                ",
            None,
        ),
        (
            "
            interface Test {
              f: <T>(a: T) => T;
            }
                ",
            None,
        ),
        (
            "
            interface Test {
              ['f']: <T extends {}>(a: T, b: T) => T;
            }
                ",
            None,
        ),
        (
            "
            interface Test {
              'f!': </* a */ T>(/* b */ x: any /* c */) => void;
            }
                ",
            None,
        ),
        (
            "
            interface Test {
              get f(): number;
            }
                ",
            None,
        ),
        (
            "
            interface Test {
              set f(value: number): void;
            }
                ",
            None,
        ),
        ("type Test = { readonly f: (a: string) => number };", None),
        ("type Test = { ['f']?: (a: boolean) => void };", None),
        ("type Test = { readonly f?: <T>(a?: T) => T };", None),
        ("type Test = { readonly ['f']?: <T>(a: T, b: T) => T };", None),
        ("type Test = { get f(): number };", None),
        ("type Test = { set f(value: number): void };", None),
        (
            "
                    interface Test {
                      f(a: string): number;
                    }
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
                    interface Test {
                      ['f'](a: boolean): void;
                    }
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
                    interface Test {
                      f<T>(a: T): T;
                    }
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
                    interface Test {
                      ['f']<T extends {}>(a: T, b: T): T;
                    }
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
                    interface Test {
                      'f!'</* a */ T>(/* b */ x: any /* c */): void;
                    }
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
                    type Test = { f(a: string): number };
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
                    type Test = { ['f']?(a: boolean): void };
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
                    type Test = { f?<T>(a?: T): T };
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
                    type Test = { ['f']?<T>(a: T, b: T): T };
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
                  interface Test {
                    get f(): number;
                  }
                ",
            None,
        ),
        (
            "
                  interface Test {
                    set f(value: number): void;
                  }
                ",
            None,
        ),
        (
            "
                    type Test = { get f(): number };
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
                    type Test = { set f(value: number): void };
                  ",
            Some(serde_json::json!(["method"])),
        ),
    ];

    let fail = vec![
        ("
                    interface Test {
                      f(a: string): number;
                    }
                  ", None),
("
                    interface Test {
                      ['f'](a: boolean): void;
                    }
                  ", None),
("
                    interface Test {
                      f<T>(a: T): T;
                    }
                  ", None),
("
                    interface Test {
                      ['f']<T extends {}>(a: T, b: T): T;
                    }
                  ", None),
("
                    interface Test {
                      'f!'</* a */ T>(/* b */ x: any /* c */): void;
                    }
                  ", None),
("
                    type Test = { f(a: string): number };
                  ", None),
("
                    type Test = { ['f']?(a: boolean): void };
                  ", None),
("
                    type Test = { f?<T>(a?: T): T };
                  ", None),
("
                    type Test = { ['f']?<T>(a: T, b: T): T };
                  ", None),
("
                    interface Test {
                      f: (a: string) => number;
                    }
                  ", Some(serde_json::json!(["method"]))),
("
                    interface Test {
                      ['f']: (a: boolean) => void;
                    }
                  ", Some(serde_json::json!(["method"]))),
("
                    interface Test {
                      f: <T>(a: T) => T;
                    }
                  ", Some(serde_json::json!(["method"]))),
("
                    interface Test {
                      ['f']: <T extends {}>(a: T, b: T) => T;
                    }
                  ", Some(serde_json::json!(["method"]))),
("
                    interface Test {
                      'f!': </* a */ T>(/* b */ x: any /* c */) => void;
                    }
                  ", Some(serde_json::json!(["method"]))),
("
                    type Test = { f: (a: string) => number };
                  ", Some(serde_json::json!(["method"]))),
("
                    type Test = { ['f']?: (a: boolean) => void };
                  ", Some(serde_json::json!(["method"]))),
("
                    type Test = { f?: <T>(a?: T) => T };
                  ", Some(serde_json::json!(["method"]))),
("
                    type Test = { ['f']?: <T>(a: T, b: T) => T };
                  ", Some(serde_json::json!(["method"]))),
("
            interface Foo {
              semi(arg: string): void;
              comma(arg: string): void,
              none(arg: string): void
            }
                  ", None),
("
            interface Foo {
              semi: (arg: string) => void;
              comma: (arg: string) => void,
              none: (arg: string) => void
            }
                  ", Some(serde_json::json!(["method"]))),
("
            interface Foo {
              x(
                args: Pick<
                  Bar,
                  'one' | 'two' | 'three'
                >,
              ): Baz;
              y(
                foo: string,
                bar: number,
              ): void;
            }
                  ", None),
("
            interface Foo {
              foo(): one;
              foo(): two;
              foo(): three;
            }
                  ", None),
("
            interface Foo {
              foo(bar: string): one;
              foo(bar: number, baz: string): two;
              foo(): three;
            }
                  ", None),
("
            interface Foo {
              [foo](bar: string): one;
              [foo](bar: number, baz: string): two;
              [foo](): three;
            }
                  ", None),
("
            interface Foo {
              [foo](bar: string): one;
              [foo](bar: number, baz: string): two;
              [foo](): three;
              bar(arg: string): void;
              bar(baz: number): Foo;
            }
                  ", None),
("
                    declare global {
                      namespace jest {
                        interface Matchers<R, T> {
                          // Add overloads specific to the DOM
                          toHaveProp<K extends keyof DomPropsOf<T>>(name: K, value?: DomPropsOf<T>[K]): R;
                          toHaveProps(props: Partial<DomPropsOf<T>>): R;
                        }
                      }
                    }
                  ", None),
("
            type Foo = {
              foo(): one;
              foo(): two;
              foo(): three;
            }
                  ", None),
("
            declare const Foo: {
              foo(): one;
              foo(): two;
              foo(): three;
            }
                  ", None),
("
            interface MyInterface {
              methodReturningImplicitAny();
            }
                  ", None),
("
            interface Test {
              f(value: number): this;
            }
                  ", None),
("
            interface Test {
              foo(): this;
              foo(): Promise<this>;
            }
                  ", None),
("
            interface Test {
              f(value: number): this | undefined;
            }
                  ", None),
("
            interface Test {
              f(value: number): Promise<this>;
            }
                  ", None),
("
            interface Test {
              f(value: number): Promise<this | undefined>;
            }
                  ", None)
    ];

    let fix = vec![
        (
            "
                    interface Test {
                      f(a: string): number;
                    }
                  ",
            "
                    interface Test {
                      f: (a: string) => number;
                    }
                  ",
            None,
        ),
        (
            "
                    interface Test {
                      ['f'](a: boolean): void;
                    }
                  ",
            "
                    interface Test {
                      ['f']: (a: boolean) => void;
                    }
                  ",
            None,
        ),
        (
            "
                    interface Test {
                      f<T>(a: T): T;
                    }
                  ",
            "
                    interface Test {
                      f: <T>(a: T) => T;
                    }
                  ",
            None,
        ),
        (
            "
                    interface Test {
                      ['f']<T extends {}>(a: T, b: T): T;
                    }
                  ",
            "
                    interface Test {
                      ['f']: <T extends {}>(a: T, b: T) => T;
                    }
                  ",
            None,
        ),
        (
            "
                    interface Test {
                      'f!'</* a */ T>(/* b */ x: any /* c */): void;
                    }
                  ",
            "
                    interface Test {
                      'f!': </* a */ T>(/* b */ x: any /* c */) => void;
                    }
                  ",
            None,
        ),
        (
            "
                    type Test = { f(a: string): number };
                  ",
            "
                    type Test = { f: (a: string) => number };
                  ",
            None,
        ),
        (
            "
                    type Test = { ['f']?(a: boolean): void };
                  ",
            "
                    type Test = { ['f']?: (a: boolean) => void };
                  ",
            None,
        ),
        (
            "
                    type Test = { f?<T>(a?: T): T };
                  ",
            "
                    type Test = { f?: <T>(a?: T) => T };
                  ",
            None,
        ),
        (
            "
                    type Test = { ['f']?<T>(a: T, b: T): T };
                  ",
            "
                    type Test = { ['f']?: <T>(a: T, b: T) => T };
                  ",
            None,
        ),
        (
            "
                    interface Test {
                      f: (a: string) => number;
                    }
                  ",
            "
                    interface Test {
                      f(a: string): number;
                    }
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
                    interface Test {
                      ['f']: (a: boolean) => void;
                    }
                  ",
            "
                    interface Test {
                      ['f'](a: boolean): void;
                    }
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
                    interface Test {
                      f: <T>(a: T) => T;
                    }
                  ",
            "
                    interface Test {
                      f<T>(a: T): T;
                    }
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
                    interface Test {
                      ['f']: <T extends {}>(a: T, b: T) => T;
                    }
                  ",
            "
                    interface Test {
                      ['f']<T extends {}>(a: T, b: T): T;
                    }
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
                    interface Test {
                      'f!': </* a */ T>(/* b */ x: any /* c */) => void;
                    }
                  ",
            "
                    interface Test {
                      'f!'</* a */ T>(/* b */ x: any /* c */): void;
                    }
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
                    type Test = { f: (a: string) => number };
                  ",
            "
                    type Test = { f(a: string): number };
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
                    type Test = { ['f']?: (a: boolean) => void };
                  ",
            "
                    type Test = { ['f']?(a: boolean): void };
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
                    type Test = { f?: <T>(a?: T) => T };
                  ",
            "
                    type Test = { f?<T>(a?: T): T };
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
                    type Test = { ['f']?: <T>(a: T, b: T) => T };
                  ",
            "
                    type Test = { ['f']?<T>(a: T, b: T): T };
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
            interface Foo {
              semi(arg: string): void;
              comma(arg: string): void,
              none(arg: string): void
            }
                  ",
            "
            interface Foo {
              semi: (arg: string) => void;
              comma: (arg: string) => void,
              none: (arg: string) => void
            }
                  ",
            None,
        ),
        (
            "
            interface Foo {
              semi: (arg: string) => void;
              comma: (arg: string) => void,
              none: (arg: string) => void
            }
                  ",
            "
            interface Foo {
              semi(arg: string): void;
              comma(arg: string): void,
              none(arg: string): void
            }
                  ",
            Some(serde_json::json!(["method"])),
        ),
        (
            "
            interface Foo {
              x(
                args: Pick<
                  Bar,
                  'one' | 'two' | 'three'
                >,
              ): Baz;
              y(
                foo: string,
                bar: number,
              ): void;
            }
                  ",
            "
            interface Foo {
              x: (
                args: Pick<
                  Bar,
                  'one' | 'two' | 'three'
                >,
              ) => Baz;
              y: (
                foo: string,
                bar: number,
              ) => void;
            }
                  ",
            None,
        ),
        (
            "
            interface Foo {
              foo(): one;
              foo(): two;
              foo(): three;
            }
                  ",
            "
            interface Foo {
              foo: (() => one) & (() => two) & (() => three);
            }
                  ",
            None,
        ),
        (
            "
            interface Foo {
              foo(bar: string): one;
              foo(bar: number, baz: string): two;
              foo(): three;
            }
                  ",
            "
            interface Foo {
              foo: ((bar: string) => one) & ((bar: number, baz: string) => two) & (() => three);
            }
                  ",
            None,
        ),
        (
            "
            interface Foo {
              [foo](bar: string): one;
              [foo](bar: number, baz: string): two;
              [foo](): three;
            }
                  ",
            "
            interface Foo {
              [foo]: ((bar: string) => one) & ((bar: number, baz: string) => two) & (() => three);
            }
                  ",
            None,
        ),
        (
            "
            interface Foo {
              [foo](bar: string): one;
              [foo](bar: number, baz: string): two;
              [foo](): three;
              bar(arg: string): void;
              bar(baz: number): Foo;
            }
                  ",
            "
            interface Foo {
              [foo]: ((bar: string) => one) & ((bar: number, baz: string) => two) & (() => three);
              bar: ((arg: string) => void) & ((baz: number) => Foo);
            }
                  ",
            None,
        ),
        (
            "
            type Foo = {
              foo(): one;
              foo(): two;
              foo(): three;
            }
                  ",
            "
            type Foo = {
              foo: (() => one) & (() => two) & (() => three);
            }
                  ",
            None,
        ),
        (
            "
            declare const Foo: {
              foo(): one;
              foo(): two;
              foo(): three;
            }
                  ",
            "
            declare const Foo: {
              foo: (() => one) & (() => two) & (() => three);
            }
                  ",
            None,
        ),
        (
            "
            interface MyInterface {
              methodReturningImplicitAny();
            }
                  ",
            "
            interface MyInterface {
              methodReturningImplicitAny: () => any;
            }
                  ",
            None,
        ),
    ];

    Tester::new(MethodSignatureStyle::NAME, MethodSignatureStyle::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
