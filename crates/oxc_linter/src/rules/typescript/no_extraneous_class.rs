use oxc_ast::{
    ast::{ClassElement, FormalParameter},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct NoExtraneousClass {
    allow_constructor_only: bool,
    allow_empty: bool,
    allow_static_only: bool,
    allow_with_decorator: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule reports when a class has no non-static members,
    /// such as for a class used exclusively as a static namespace.
    /// This rule also reports classes that have only a constructor and no fields.
    /// Those classes can generally be replaced with a standalone function.
    ///
    /// ### Why is this bad?
    ///
    /// Users who come from a OOP paradigm may wrap their utility functions in an extra class,
    /// instead of putting them at the top level of an ECMAScript module.
    /// Doing so is generally unnecessary in JavaScript and TypeScript projects.
    ///
    /// Wrapper classes add extra cognitive complexity to code without adding any structural improvements
    ///
    /// Whatever would be put on them, such as utility functions, are already organized by virtue of being in a module.
    ///
    /// As an alternative, you can import * as ... the module to get all of them in a single object.
    /// IDEs can't provide as good suggestions for static class or namespace imported properties when you start typing property names
    ///
    /// It's more difficult to statically analyze code for unused variables, etc.
    /// when they're all on the class (see: Finding dead code (and dead types) in TypeScript).
    ///
    /// ### Example
    /// ```ts
    /// class StaticConstants {
    /// 	static readonly version = 42;
    ///
    /// 	static isProduction() {
    /// 	  return process.env.NODE_ENV === 'production';
    /// 	}
    ///   }
    ///
    ///   class HelloWorldLogger {
    /// 	constructor() {
    /// 	  console.log('Hello, world!');
    /// 	}
    ///   }
    ///
    ///   abstract class Foo {}
    /// ```
    NoExtraneousClass,
    typescript,
    suspicious
);

fn empty_no_extraneous_class_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected empty class.").with_label(span)
}

fn only_static_no_extraneous_class_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected class with only static properties.").with_label(span)
}

fn only_constructor_no_extraneous_class_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected class with only a constructor.").with_label(span)
}

impl Rule for NoExtraneousClass {
    fn from_configuration(value: serde_json::Value) -> Self {
        use serde_json::Value;
        let Some(config) = value.get(0).and_then(Value::as_object) else {
            return Self::default();
        };
        Self {
            allow_constructor_only: config
                .get("allowConstructorOnly")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            allow_empty: config
                .get("allowEmpty") // lb
                .and_then(Value::as_bool)
                .unwrap_or(false),
            allow_static_only: config
                .get("allowStaticOnly")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            allow_with_decorator: config
                .get("allowWithDecorator")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Class(class) = node.kind() else {
            return;
        };
        if class.super_class.is_some()
            || (self.allow_with_decorator && !class.decorators.is_empty())
        {
            return;
        }
        let span = class.id.as_ref().map_or(class.span, |id| id.span);
        let body = &class.body.body;
        match body.as_slice() {
            [] => {
                if !self.allow_empty {
                    ctx.diagnostic(empty_no_extraneous_class_diagnostic(class.span));
                }
            }
            [ClassElement::MethodDefinition(constructor)] if constructor.kind.is_constructor() => {
                let only_constructor =
                    !constructor.value.params.items.iter().any(FormalParameter::has_modifier);
                if only_constructor && !self.allow_constructor_only {
                    ctx.diagnostic(only_constructor_no_extraneous_class_diagnostic(span));
                }
            }
            _ => {
                let only_static = body.iter().all(|prop| prop.r#static() && !prop.is_abstract());
                if only_static && !self.allow_static_only {
                    ctx.diagnostic(only_static_no_extraneous_class_diagnostic(span));
                }
            }
        };
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
			class Foo {
			  public prop = 1;
			  constructor() {}
			}
			    ",
            None,
        ),
        (
            "
			export class CClass extends BaseClass {
			  public static helper(): void {}
			  private static privateHelper(): boolean {
			    return true;
			  }
			  constructor() {}
			}
			    ",
            None,
        ),
        (
            "
			class Foo {
			  constructor(public bar: string) {}
			}
			    ",
            None,
        ),
        ("class Foo {}", Some(serde_json::json!([{ "allowEmpty": true }]))),
        (
            "
			class Foo {
			  constructor() {}
			}
			      ",
            Some(serde_json::json!([{ "allowConstructorOnly": true }])),
        ),
        (
            "
			export class Bar {
			  public static helper(): void {}
			  private static privateHelper(): boolean {
			    return true;
			  }
			}
			      ",
            Some(serde_json::json!([{ "allowStaticOnly": true }])),
        ),
        (
            "
			export default class {
			  hello() {
			    return 'I am foo!';
			  }
			}
			    ",
            None,
        ),
        (
            "
			@FooDecorator
			class Foo {}
			      ",
            Some(serde_json::json!([{ "allowWithDecorator": true }])),
        ),
        (
            "
			@FooDecorator
			class Foo {
			  constructor(foo: Foo) {
			    foo.subscribe(a => {
			      console.log(a);
			    });
			  }
			}
			      ",
            Some(serde_json::json!([{ "allowWithDecorator": true }])),
        ),
        (
            "
			abstract class Foo {
			  abstract property: string;
			}
			    ",
            None,
        ),
        (
            "
			abstract class Foo {
			  abstract method(): string;
			}
			    ",
            None,
        ),
    ];

    let fail = vec![
        ("class Foo {}", None),
        (
            "
			class Foo {
			  public prop = 1;
			  constructor() {
			    class Bar {
			      static PROP = 2;
			    }
			  }
			}
			export class Bar {
			  public static helper(): void {}
			  private static privateHelper(): boolean {
			    return true;
			  }
			}
			      ",
            None,
        ),
        (
            "
			class Foo {
			  constructor() {}
			}
			      ",
            None,
        ),
        (
            "
			export class AClass {
			  public static helper(): void {}
			  private static privateHelper(): boolean {
			    return true;
			  }
			  constructor() {
			    class nestedClass {}
			  }
			}
			      ",
            None,
        ),
        (
            "
			export default class {
			  static hello() {}
			}
			      ",
            None,
        ),
        (
            "
			@FooDecorator
			class Foo {}
			      ",
            Some(serde_json::json!([{ "allowWithDecorator": false }])),
        ),
        (
            "
			@FooDecorator
			class Foo {
			  constructor(foo: Foo) {
			    foo.subscribe(a => {
			      console.log(a);
			    });
			  }
			}
			      ",
            Some(serde_json::json!([{ "allowWithDecorator": false }])),
        ),
        (
            "
			abstract class Foo {}
			      ",
            None,
        ),
        (
            "
			abstract class Foo {
			  static property: string;
			}
			      ",
            None,
        ),
        (
            "
			abstract class Foo {
			  constructor() {}
			}
			      ",
            None,
        ),
    ];

    Tester::new(NoExtraneousClass::NAME, NoExtraneousClass::PLUGIN, pass, fail).test_and_snapshot();
}
