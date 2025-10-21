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
use serde_json::Value;

use crate::{AstNode, context::LintContext, rule::Rule};

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
    /// Locations and kinds of functions that are allowed to be empty.
    allow: Allowed,
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
impl TryFrom<&str> for Allowed {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // NOTE: typescript-eslint extends options from eslint. Their docs dont list
        // originals
        // NOTE: typescript-eslint uses kebab-case instead of camelCase for some of its additional options.
        // This is confusing, so we support both kinds
        match value {
            "functions" | "function" => Ok(Self::Function),
            "arrowFunctions" | "arrow-functions" => Ok(Self::ArrowFunction),
            "generatorFunctions" | "generator-functions" => Ok(Self::GeneratorFunctions),
            "methods" | "method" => Ok(Self::Methods),
            "generatorMethods" | "generator-methods" => Ok(Self::GeneratorMethods),
            "getters" | "getter" => Ok(Self::Getters),
            "setters" | "setter" => Ok(Self::Setters),
            "constructors" | "constructor" => Ok(Self::Constructors),
            "asyncFunctions" | "async-functions" => Ok(Self::AsyncFunctions),
            "asyncMethods" | "async-methods" => Ok(Self::AsyncMethods),
            "privateConstructors" | "private-constructors" => Ok(Self::PrivateConstructor),
            "protectedConstructors" | "protected-constructors" => Ok(Self::ProtectedConstructor),
            "decoratedFunctions" | "decorated-functions" => Ok(Self::DecoratedFunction),
            "overrideMethods" | "override-methods" => Ok(Self::OverrideMethod),
            _ => Err(()),
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the usages of empty functions
    ///
    /// ### Why is this bad?
    ///
    /// Empty functions can reduce readability because readers need to guess whether it's
    /// intentional or not. So writing a clear comment for empty functions is a good practice.
    ///
    /// ### Options
    ///
    /// #### allow
    ///
    /// `{ type: string[], default: [] }`
    ///
    /// You may pass a list of allowed function kinds, which will allow functions of
    /// these kinds to be empty.
    ///
    /// Example:
    /// ```json
    /// {
    ///   "no-empty-function": [
    ///     "error",
    ///     { "allow": ["functions"] }
    ///   ]
    /// }
    /// ```
    ///
    /// `allow` accepts the following values:
    /// - `"functions"`
    /// - `"arrowFunctions"`
    /// - `"generatorFunctions"`
    /// - `"methods"`
    /// - `"generatorMethods"`
    /// - `"getters"`
    /// - `"setters"`
    /// - `"constructors"`
    /// - `"privateConstructors"`
    /// - `"protectedConstructors"`
    /// - `"asyncFunctions"`
    /// - `"asyncMethods"`
    /// - `"decoratedFunctions"`
    /// - `"overrideMethods"`
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
);

impl Rule for NoEmptyFunction {
    fn from_configuration(value: Value) -> Self {
        let config = match value {
            Value::Object(ref obj) => Some(obj),
            Value::Array(ref arr) => arr.first().and_then(Value::as_object),
            _ => None,
        };
        let Some(config) = config else { return NoEmptyFunction::default() };
        let Some(allow) = config.get("allow").and_then(Value::as_array) else {
            return NoEmptyFunction::default();
        };
        let mut allow_option = Allowed::None;
        for allowed in allow {
            let Some(allowed) = allowed.as_str() else { continue };
            let Some(allowed) = Allowed::try_from(allowed).ok() else { continue };
            allow_option |= allowed;
        }

        Self { allow: allow_option }
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
        ("async function foo() {}", Some(serde_json::json!([{ "allow": ["asyncFunctions"] }]))),
        ("const foo = async () => {};", Some(serde_json::json!([{ "allow": ["asyncFunctions"] }]))),
        ("class Foo { async bar() {} }", Some(serde_json::json!([{ "allow": ["asyncMethods"] }]))),
        ("class Foo { *gen() {} }", Some(serde_json::json!([{ "allow": ["generatorMethods"] }]))),
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
