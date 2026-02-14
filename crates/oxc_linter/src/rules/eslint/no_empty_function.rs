use std::borrow::Cow;

use bitflags::bitflags;
use oxc_ast::{
    AstKind,
    ast::{
        FormalParameter, IdentifierName, IdentifierReference, MethodDefinition,
        MethodDefinitionKind, TSAccessibility,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_empty_function_diagnostic<S: AsRef<str>>(
    span: Span,
    fn_kind: &str,
    fn_name: Option<S>,
) -> OxcDiagnostic {
    let message = match fn_name {
        Some(name) => Cow::Owned(format!("Unexpected empty {fn_kind} `{}`", name.as_ref())),
        None => Cow::Borrowed("Unexpected empty function"),
    };
    OxcDiagnostic::warn(message)
        .with_help(format!("Consider removing this {fn_kind} or adding logic to it."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoEmptyFunction {
    allow: Allowed,
}

impl From<NoEmptyFunctionConfig> for NoEmptyFunction {
    fn from(config: NoEmptyFunctionConfig) -> Self {
        let mut flags = Allowed::None;
        for kind in &config.allow {
            flags |= Allowed::from(*kind);
        }
        Self { allow: flags }
    }
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct NoEmptyFunctionConfig {
    /// Types of functions that are allowed to be empty.
    ///
    /// By default, no function kinds are allowed to be empty, but this option can be used to
    /// permit specific kinds of functions.
    ///
    /// Example:
    /// ```json
    /// {
    ///   "no-empty-function": ["error", { "allow": ["constructors"] }]
    /// }
    /// ```
    allow: Vec<AllowKind>,
}

/// Kinds of functions that can be allowed to be empty.
// NOTE: typescript-eslint extends options from eslint. Their docs don't list originals.
// NOTE: typescript-eslint uses kebab-case instead of camelCase for some of its additional options.
// This is confusing, so we support both kinds.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AllowKind {
    /// Allow empty regular functions.
    ///
    /// ```js
    /// function foo() {}
    /// ```
    #[serde(alias = "function")]
    Functions,
    /// Allow empty arrow functions.
    ///
    /// ```js
    /// const foo = () => {};
    /// ```
    #[serde(alias = "arrow-functions")]
    ArrowFunctions,
    /// Allow empty generator functions.
    ///
    /// ```js
    /// function* foo() {}
    /// ```
    #[serde(alias = "generator-functions")]
    GeneratorFunctions,
    /// Allow empty methods.
    ///
    /// ```js
    /// class Foo {
    ///   bar() {}
    /// }
    /// ```
    #[serde(alias = "method")]
    Methods,
    /// Allow empty generator methods.
    ///
    /// ```js
    /// class Foo {
    ///   *bar() {}
    /// }
    /// ```
    #[serde(alias = "generator-methods")]
    GeneratorMethods,
    /// Allow empty getters.
    ///
    /// ```js
    /// class Foo {
    ///   get bar() {}
    /// }
    /// ```
    #[serde(alias = "getter")]
    Getters,
    /// Allow empty setters.
    ///
    /// ```js
    /// class Foo {
    ///   set bar(value) {}
    /// }
    /// ```
    #[serde(alias = "setter")]
    Setters,
    /// Allow empty constructors.
    ///
    /// ```js
    /// class Foo {
    ///   constructor() {}
    /// }
    /// ```
    #[serde(alias = "constructor")]
    Constructors,
    /// Allow empty async functions.
    ///
    /// ```js
    /// async function foo() {}
    /// ```
    #[serde(alias = "async-functions")]
    AsyncFunctions,
    /// Allow empty async methods.
    ///
    /// ```js
    /// class Foo {
    ///   async bar() {}
    /// }
    /// ```
    #[serde(alias = "async-methods")]
    AsyncMethods,
    /// Allow empty private constructors.
    ///
    /// ```ts
    /// class Foo {
    ///   private constructor() {}
    /// }
    /// ```
    #[serde(alias = "private-constructors")]
    PrivateConstructors,
    /// Allow empty protected constructors.
    ///
    /// ```ts
    /// class Foo {
    ///   protected constructor() {}
    /// }
    /// ```
    #[serde(alias = "protected-constructors")]
    ProtectedConstructors,
    /// Allow empty decorated functions.
    ///
    /// ```js
    /// class Foo {
    ///   @decorator()
    ///   bar() {}
    /// }
    /// ```
    #[serde(alias = "decorated-functions")]
    DecoratedFunctions,
    /// Allow empty override methods.
    ///
    /// ```ts
    /// class Foo extends Base {
    ///   override bar() {}
    /// }
    /// ```
    #[serde(alias = "override-methods")]
    OverrideMethods,
}

impl From<AllowKind> for Allowed {
    fn from(kind: AllowKind) -> Self {
        match kind {
            AllowKind::Functions => Allowed::Function,
            AllowKind::ArrowFunctions => Allowed::ArrowFunction,
            AllowKind::GeneratorFunctions => Allowed::GeneratorFunctions,
            AllowKind::Methods => Allowed::Methods,
            AllowKind::GeneratorMethods => Allowed::GeneratorMethods,
            AllowKind::Getters => Allowed::Getters,
            AllowKind::Setters => Allowed::Setters,
            AllowKind::Constructors => Allowed::Constructors,
            AllowKind::AsyncFunctions => Allowed::AsyncFunctions,
            AllowKind::AsyncMethods => Allowed::AsyncMethods,
            AllowKind::PrivateConstructors => Allowed::PrivateConstructor,
            AllowKind::ProtectedConstructors => Allowed::ProtectedConstructor,
            AllowKind::DecoratedFunctions => Allowed::DecoratedFunction,
            AllowKind::OverrideMethods => Allowed::OverrideMethod,
        }
    }
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Allowed: u16 {
        const None = 0;
        /// `'functions'`
        const Function = 1 << 0;
        /// `'arrowFunctions'`
        const ArrowFunction = 1 << 1;
        /// `'generatorFunctions'`
        const GeneratorFunctions = 1 << 2;
        /// `'methods'`
        const Methods = 1 << 3;
        /// `'generatorMethods'`
        const GeneratorMethods = 1 << 4;
        /// `'getters'`
        const Getters = 1 << 5;
        /// `'setters'`
        const Setters = 1 << 6;
        /// `'constructors'`
        const Constructors = 1 << 7;
        /// `'private-constructors'`
        const PrivateConstructor = 1 << 8;
        /// `'protected-constructors'`
        const ProtectedConstructor = 1 << 9;
        /// `'asyncFunctions'`
        const AsyncFunctions = 1 << 10;
        /// `'asyncMethods'`
        const AsyncMethods = 1 << 11;
        /// `'decoratedFunctions`
        const DecoratedFunction = 1 << 12;
        /// `'overrideMethods'`
        const OverrideMethod = 1 << 13;
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the usage of empty functions.
    ///
    /// ### Why is this bad?
    ///
    /// Empty functions can reduce readability because readers need to guess whether it's
    /// intentional or not. So writing a clear comment for empty functions is a good practice.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// function foo() {
    /// }
    ///
    /// const bar = () => {};
    ///
    /// class Foo {
    ///   constructor()
    ///   someMethod() {}
    ///   set bar(value) {}
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// function foo() {
    ///     // do nothing
    /// }
    ///
    /// function foo() {
    ///   return;
    /// }
    /// const add = (a, b) => a + b
    ///
    /// class Foo {
    ///   // constructor body is empty, but it declares a private property named
    ///   // `_name`
    ///   constructor(private _name: string) {}
    ///
    ///   public get name() {
    ///     return this._name;
    ///   }
    /// }
    /// ```
    NoEmptyFunction,
    eslint,
    restriction,
    pending,
    config = NoEmptyFunctionConfig,
);

impl Rule for NoEmptyFunction {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let config = serde_json::from_value::<DefaultRuleConfig<NoEmptyFunctionConfig>>(value)
            .map(DefaultRuleConfig::into_inner)?;
        Ok(NoEmptyFunction::from(config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::FunctionBody(fb) = node.kind() else {
            return;
        };
        if !fb.is_empty() || ctx.has_comments_between(fb.span) {
            return;
        }
        let ViolationInfo(Some((kind, fn_name))) = self.get_function_name_and_kind(node, ctx)
        else {
            return;
        };
        ctx.diagnostic(no_empty_function_diagnostic(fb.span, kind, fn_name));
    }
}

#[derive(Default, Debug, Clone)]
struct ViolationInfo<'a>(pub Option<(&'static str, Option<Cow<'a, str>>)>);
impl<'a> From<(&'static str, Option<Cow<'a, str>>)> for ViolationInfo<'a> {
    fn from(value: (&'static str, Option<Cow<'a, str>>)) -> Self {
        debug_assert!(!value.0.is_empty());
        Self(Some(value))
    }
}

impl NoEmptyFunction {
    fn get_function_name_and_kind<'a>(
        &self,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) -> ViolationInfo<'a> {
        for parent in ctx.nodes().ancestor_kinds(node.id()) {
            match parent {
                AstKind::Function(f) => {
                    if let Some(name) = f.name() {
                        let is_generator = f.generator;
                        let is_async = f.r#async;

                        if is_generator && self.allow.contains(Allowed::GeneratorFunctions) {
                            return ViolationInfo::default();
                        }
                        if is_async && self.allow.contains(Allowed::AsyncFunctions) {
                            return ViolationInfo::default();
                        }
                        if !is_generator && !is_async && self.allow.contains(Allowed::Function) {
                            return ViolationInfo::default();
                        }

                        let kind = if is_async {
                            "async function"
                        } else if is_generator {
                            "generator function"
                        } else {
                            "function"
                        };
                        return (kind, Some(name.into())).into();
                    }
                }
                AstKind::ArrowFunctionExpression(arrow) => {
                    if self.allow.contains(Allowed::ArrowFunction) {
                        return ViolationInfo::default();
                    }
                    if arrow.r#async && self.allow.contains(Allowed::AsyncFunctions) {
                        return ViolationInfo::default();
                    }
                }
                AstKind::IdentifierName(IdentifierName { name, .. })
                | AstKind::IdentifierReference(IdentifierReference { name, .. }) => {
                    return ("function", Some(Cow::Borrowed(name.as_str()))).into();
                }
                AstKind::PropertyDefinition(prop) => {
                    if self.allow_decorated_function() && !prop.decorators.is_empty() {
                        return ViolationInfo::default();
                    }
                    return ("function", prop.key.name()).into();
                }
                AstKind::MethodDefinition(method) => {
                    if self.is_allowed_method(method) {
                        return ViolationInfo::default();
                    }
                    let kind = match method.kind {
                        MethodDefinitionKind::Method => {
                            if method.r#static {
                                "static method"
                            } else {
                                "method"
                            }
                        }
                        MethodDefinitionKind::Get => "getter",
                        MethodDefinitionKind::Set => "setter",
                        MethodDefinitionKind::Constructor => "constructor",
                    };
                    return (kind, method.key.name()).into();
                }
                AstKind::VariableDeclarator(decl) => {
                    return ("function", decl.id.get_identifier_name().map(Into::into)).into();
                }
                _ => return ("function", None).into(),
            }
        }
        #[cfg(debug_assertions)]
        unreachable!();
        #[cfg(not(debug_assertions))]
        ("function", None).into()
    }

    fn is_allowed_method(&self, method: &MethodDefinition) -> bool {
        if self.allow.contains(Allowed::DecoratedFunction) && !method.decorators.is_empty() {
            return true;
        }
        match method.kind {
            MethodDefinitionKind::Constructor => {
                if self.allow.contains(Allowed::Constructors) {
                    return true;
                }
                // `constructor(private name: string) {}` is allowed b/c it declares
                // a private property
                if method.value.params.items.iter().any(FormalParameter::has_modifier) {
                    return true;
                }
                match method.accessibility {
                    Some(TSAccessibility::Private) => {
                        self.allow.contains(Allowed::PrivateConstructor)
                    }
                    Some(TSAccessibility::Protected) => {
                        self.allow.contains(Allowed::ProtectedConstructor)
                    }
                    _ => false,
                }
            }
            MethodDefinitionKind::Get => self.allow.contains(Allowed::Getters),
            MethodDefinitionKind::Set => self.allow.contains(Allowed::Setters),
            MethodDefinitionKind::Method => {
                if method.value.r#async && self.allow.contains(Allowed::AsyncMethods)
                    || method.value.generator && self.allow.contains(Allowed::GeneratorMethods)
                {
                    return true;
                }
                self.allow.contains(Allowed::Methods)
                    || (method.r#override && self.allow.contains(Allowed::OverrideMethod))
            }
        }
    }

    fn allow_decorated_function(&self) -> bool {
        self.allow.contains(Allowed::DecoratedFunction)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
            function foo() {
                // empty
            }
            ",
            None,
        ),
        (
            "
            function* baz() {
                // empty
            }
            ",
            None,
        ),
        (
            "
            const bar = () => {
                // empty
            };
            ",
            None,
        ),
        (
            "
            const obj = {
                foo: function() {
                    // empty
                },
                bar: function*() {
                    // empty
                },
                foobar() {
                    // empty
                }
            };
            ",
            None,
        ),
        (
            "
            class A {
                constructor() {
                    // empty
                }
                foo() {
                    // empty
                }
                *foo1() {
                    // empty
                }
                get bar() {
                    // empty
                }
                set bar(value) {
                    // empty
                }
                static bar() {
                    // empty
                }
                static *barr() {
                    // empty
                }
                static get baz() {
                    // empty
                }
                static set baz(value) {
                    // empty
                }
            }
            ",
            None,
        ),
        // ported from typescript-eslint
        (
            "
            class Person {
              private name: string;
              constructor(name: string) {
                this.name = name;
              }
            }
            ",
            None,
        ),
        ("class Person { constructor(private name: string) {} }", None),
        (
            "class Person { constructor(name: string) {} }",
            Some(serde_json::json!([{ "allow": ["constructors"] }])),
        ),
        (
            "class Person { otherMethod(name: string) {} }",
            Some(serde_json::json!([{ "allow": ["methods"] }])),
        ),
        (
            "class Foo { private constructor() {} }",
            Some(serde_json::json!([{ "allow": ["private-constructors"] }])),
        ),
        (
            "class Foo { protected constructor() {} }",
            Some(serde_json::json!([{ "allow": ["protected-constructors"] }])),
        ),
        ("function foo() { const a = null; }", None),
        (
            "
            class Foo {
              @decorator()
              foo() {}
            }
            ",
            Some(serde_json::json!([{ "allow": ["decoratedFunctions"] }])),
        ),
        (
            "class Foo extends Base { override foo() {} }",
            Some(serde_json::json!([{ "allow": ["overrideMethods"] }])),
        ),
        // Test allow option for functions and arrow functions
        ("function foo() {}", Some(serde_json::json!([{ "allow": ["functions"] }]))),
        ("const bar = () => {};", Some(serde_json::json!([{ "allow": ["arrowFunctions"] }]))),
        (
            "const foo = () => {}; function bar() {}",
            Some(serde_json::json!([{ "allow": ["arrowFunctions", "functions"] }])),
        ),
        ("function* gen() {}", Some(serde_json::json!([{ "allow": ["generatorFunctions"] }]))),
        ("function* gen() {}", Some(serde_json::json!([{ "allow": ["generator-functions"] }]))),
        ("async function foo() {}", Some(serde_json::json!([{ "allow": ["asyncFunctions"] }]))),
        ("async function foo() {}", Some(serde_json::json!([{ "allow": ["async-functions"] }]))),
        ("const foo = async () => {};", Some(serde_json::json!([{ "allow": ["asyncFunctions"] }]))),
        ("class Foo { async bar() {} }", Some(serde_json::json!([{ "allow": ["asyncMethods"] }]))),
        ("class Foo { async bar() {} }", Some(serde_json::json!([{ "allow": ["async-methods"] }]))),
        ("class Foo { *gen() {} }", Some(serde_json::json!([{ "allow": ["generatorMethods"] }]))),
        ("class Foo { *gen() {} }", Some(serde_json::json!([{ "allow": ["generator-methods"] }]))),
        // getters
        // TODO: Fix these two. They are from the original tests.
        // ("var obj = {get foo() {}};", Some(serde_json::json!([{ "allow": ["getters"] }]))),
        // ("var obj = {get foo() {}};", Some(serde_json::json!([{ "allow": ["getter"] }]))),
        ("class A {get foo() {}}", Some(serde_json::json!([{ "allow": ["getters"] }]))),
        ("class A {get foo() {}}", Some(serde_json::json!([{ "allow": ["getter"] }]))),
        ("class A {static get foo() {}}", Some(serde_json::json!([{ "allow": ["getters"] }]))),
        ("class A {static get foo() {}}", Some(serde_json::json!([{ "allow": ["getter"] }]))),
        // setters
        // TODO: Fix these two. They are from the original tests.
        // ("var obj = {set foo(value) {}};", Some(serde_json::json!([{ "allow": ["setters"] }]))),
        // ("var obj = {set foo(value) {}};", Some(serde_json::json!([{ "allow": ["setter"] }]))),
        ("class A {set foo(value) {}}", Some(serde_json::json!([{ "allow": ["setters"] }]))),
        ("class A {set foo(value) {}}", Some(serde_json::json!([{ "allow": ["setter"] }]))),
        ("class A {static set foo(value) {}}", Some(serde_json::json!([{ "allow": ["setters"] }]))),
        ("class A {static set foo(value) {}}", Some(serde_json::json!([{ "allow": ["setter"] }]))),
        ("var A = class {set foo(value) {}};", Some(serde_json::json!([{ "allow": ["setters"] }]))),
        ("var A = class {set foo(value) {}};", Some(serde_json::json!([{ "allow": ["setter"] }]))),
        (
            "var A = class {static set foo(value) {}};",
            Some(serde_json::json!([{ "allow": ["setters"] }])),
        ),
        (
            "var A = class {static set foo(value) {}};",
            Some(serde_json::json!([{ "allow": ["setter"] }])),
        ),
        // extras added by oxc team
        ("declare function foo(x: number): void;", None),
    ];

    let fail = vec![
        ("function foo() {}", None),
        ("const bar = () => {};", None),
        ("function* baz() {}", None),
        (
            "
        const obj = {
            foo: function() {
            },
            bar: function*() {
            },
            foobar() {
            }
        };
        ",
            None,
        ),
        (
            "
        class A {
            constructor() {
            }
            foo() {
            }
            *foo1() {
            }
            get fooz() {
            }
            set fooz(value) {
            }
            static bar() {
            }
            static *barr() {
            }
            static get baz() {
            }
            static set baz(value) {
            }
        }
        ",
            None,
        ),
        // ported from typescript-eslint
        ("class Person { constructor(name: string) {} }", None),
        ("class Person { otherMethod(name: string) {} }", None),
        ("class Foo { private constructor() {} }", None),
        ("class Foo { protected constructor() {} }", None),
        ("function foo() {}", None),
        (
            "
        class Foo {
          @decorator()
          foo() {}
        }
        ",
            None,
        ),
        ("class Foo extends Base { override foo() {} }", None),
    ];

    Tester::new(NoEmptyFunction::NAME, NoEmptyFunction::PLUGIN, pass, fail).test_and_snapshot();
}
