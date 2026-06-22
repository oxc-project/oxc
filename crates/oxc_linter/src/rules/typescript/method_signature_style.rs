use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{TSMethodSignatureKind, TSType},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

fn method_signature_style_diagnostic(
    config: MethodSignatureStyleConfig,
    span: Span,
) -> OxcDiagnostic {
    match config {
        MethodSignatureStyleConfig::Property => method_signature_style_property_diagnostic(span),
        MethodSignatureStyleConfig::Method => method_signature_style_method_diagnostic(span),
    }
}

fn method_signature_style_property_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use a property signature instead of a method signature.")
        .with_help("Replace the method signature with a property whose type is a function type.")
        .with_label(span)
}
fn method_signature_style_method_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use a method signature instead of a property signature.")
        .with_help("Replace the property signature with method shorthand syntax.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum MethodSignatureStyleConfig {
    #[default]
    /// Enforce using property signature for functions. Use this to enforce maximum correctness together with TypeScript's strict mode.
    Property,
    /// Enforce using method signature for functions. Use this if you aren't using TypeScript's strict mode and prefer this style.
    Method,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct MethodSignatureStyle(MethodSignatureStyleConfig);

impl MethodSignatureStyle {
    fn is_property_style(&self) -> bool {
        self.0 == MethodSignatureStyleConfig::Property
    }

    fn is_method_style(&self) -> bool {
        self.0 == MethodSignatureStyleConfig::Method
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce using a particular method signature syntax.
    ///
    /// ### Why is this bad?
    ///
    /// TypeScript provides two ways to define an object/interface function property:
    ///
    /// ```ts
    /// interface Example {
    ///   // method shorthand syntax
    ///   func(arg: string): number;
    ///
    ///   // regular property with function type
    ///   func: (arg: string) => number;
    /// }
    /// ```
    ///
    /// The two are very similar; most of the time it doesn't matter which one you use.
    /// However, when TypeScript's `strictFunctionTypes` option is enabled, there is an important difference: methods are always bivariant in their arguments, while function properties are contravariant.
    /// This means that switching from method syntax to property syntax (or vice versa) can cause TypeScript to report new type errors or stop reporting existing ones.
    ///
    /// A good practice is to use the TypeScript's `strict` option (which implies `strictFunctionTypes`) which enables correct typechecking for function properties only (method signatures get old behavior).
    ///
    /// TypeScript FAQ:
    ///
    /// > A method and a function property of the same type behave differently.
    /// > Methods are always bivariant in their argument, while function properties are contravariant in their argument under `strictFunctionTypes`.
    ///
    /// See the reasoning behind that in the [TypeScript PR for the compiler option](https://github.com/microsoft/TypeScript/pull/18654).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with `property` option:
    /// ```ts
    /// interface T1 {
    ///   func(arg: string): number;
    /// }
    /// type T2 = {
    ///   func(arg: boolean): void;
    /// };
    /// interface T3 {
    ///   func(arg: number): void;
    ///   func(arg: string): void;
    ///   func(arg: boolean): void;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule with `property` option:
    /// ```ts
    /// interface T1 {
    ///   func: (arg: string) => number;
    /// }
    /// type T2 = {
    ///   func: (arg: boolean) => void;
    /// };
    /// // this is equivalent to the overload
    /// interface T3 {
    ///   func: ((arg: number) => void) &
    ///     ((arg: string) => void) &
    ///     ((arg: boolean) => void);
    /// }
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with `method` option:
    /// ```ts
    /// interface T1 {
    ///   func: (arg: string) => number;
    /// }
    /// type T2 = {
    ///   func: (arg: boolean) => void;
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule with `method` option:
    /// ```ts
    /// interface T1 {
    ///   func(arg: string): number;
    /// }
    /// type T2 = {
    ///   func(arg: boolean): void;
    /// };
    /// ```
    MethodSignatureStyle,
    typescript,
    style,
    pending,
    version = "1.68.0",
    short_description = "Enforce using a particular method signature syntax.",
    config = MethodSignatureStyleConfig
);

impl Rule for MethodSignatureStyle {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::TSMethodSignature(ts_method)
                if self.is_property_style() && ts_method.kind == TSMethodSignatureKind::Method =>
            {
                ctx.diagnostic(method_signature_style_diagnostic(self.0, ts_method.span));
            }
            AstKind::TSPropertySignature(ts_property) if self.is_method_style() => {
                if !ts_property.type_annotation.as_ref().is_some_and(|type_annotation| {
                    matches!(
                        type_annotation.type_annotation.without_parenthesized(),
                        TSType::TSFunctionType(_)
                    )
                }) {
                    return;
                }

                ctx.diagnostic(method_signature_style_diagnostic(self.0, ts_property.span));
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
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
              set f(value: number);
            }
                ",
            None,
        ),
        ("type Test = { readonly f: (a: string) => number };", None),
        ("type Test = { ['f']?: (a: boolean) => void };", None),
        ("type Test = { readonly f?: <T>(a?: T) => T };", None),
        ("type Test = { readonly ['f']?: <T>(a: T, b: T) => T };", None),
        ("type Test = { get f(): number };", None),
        ("type Test = { set f(value: number) };", None),
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
                    set f(value: number);
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
                    type Test = { set f(value: number) };
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
                      f: (((a: string) => number));
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

    let _fix = vec![
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
        .test_and_snapshot();
}
