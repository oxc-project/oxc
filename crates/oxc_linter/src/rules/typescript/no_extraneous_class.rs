use oxc_ast::{
    AstKind,
    ast::{ClassElement, FormalParameter},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;

use crate::{AstNode, context::LintContext, rule::Rule};

#[derive(Debug, Clone, Default, JsonSchema)]
#[serde(renameAll = "camelCase", default)]
pub struct NoExtraneousClass {
    allow_constructor_only: bool,
    allow_empty: bool,
    allow_static_only: bool,
    allow_with_decorator: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule reports when a class has no non-static members, such as for a
    /// class used exclusively as a static namespace.  This rule also reports
    /// classes that have only a constructor and no fields.  Those classes can
    /// generally be replaced with a standalone function.
    ///
    /// ### Why is this bad?
    ///
    /// Users who come from a OOP paradigm may wrap their utility functions in
    /// an extra class, instead of putting them at the top level of an
    /// ECMAScript module.  Doing so is generally unnecessary in JavaScript and
    /// TypeScript projects.
    ///
    /// * Wrapper classes add extra cognitive complexity to code without adding
    ///   any structural improvements
    ///   * Whatever would be put on them, such as utility functions, are already
    ///     organized by virtue of being in a module.
    ///   * As an alternative, you can `import * as ...` the module to get all of them
    ///     in a single object.
    /// * IDEs can't provide as good suggestions for static class or namespace
    ///   imported properties when you start typing property names
    /// * It's more difficult to statically analyze code for unused variables,
    ///   etc.  when they're all on the class (see: [Finding dead code (and dead
    ///   types) in TypeScript](https://effectivetypescript.com/2020/10/20/tsprune/)).
    ///
    /// This rule also reports classes that have only a constructor and no
    /// fields. Those classes can generally be replaced with a standalone
    /// function.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
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
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const version = 42;
    /// const isProduction = () => process.env.NODE_ENV === 'production';
    /// ```
    NoExtraneousClass,
    typescript,
    suspicious,
    dangerous_suggestion
);

fn empty_class_diagnostic(span: Span, has_decorators: bool) -> OxcDiagnostic {
    let help = if has_decorators {
        r#"Set "allowWithDecorator": true in your config to allow empty decorated classes"#
    } else {
        "Delete this class"
    };
    OxcDiagnostic::warn("Unexpected empty class.").with_label(span).with_help(help)
}

fn only_static_no_extraneous_class_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected class with only static properties.")
        .with_label(span)
        .with_help("Try using standalone functions instead of static methods")
}

fn only_constructor_no_extraneous_class_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected class with only a constructor.")
        .with_label(span)
        .with_help("Try replacing this class with a standalone function or deleting it entirely")
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
                    let mut span = class.span;
                    #[expect(clippy::checked_conversions, clippy::cast_possible_truncation)]
                    if let Some(decorator) = class.decorators.last() {
                        span = Span::new(decorator.span.end, span.end);
                        // NOTE: there will always be a 'c' because of 'class' keyword.
                        let start = ctx.source_range(span).find('c').unwrap();
                        // SAFETY: source files are guaranteed to be less than
                        // 2^32 characters, so conversion will never fail. Using
                        // unchecked assert here removes a useless bounds check.
                        unsafe { std::hint::assert_unchecked(start <= u32::MAX as usize) };
                        span = span.shrink_left(start as u32);
                    }
                    let has_decorators = !class.decorators.is_empty();
                    ctx.diagnostic_with_suggestion(
                        empty_class_diagnostic(span, has_decorators),
                        |fixer| {
                            if has_decorators {
                                return fixer.noop();
                            }
                            if let AstKind::ExportNamedDeclaration(decl) =
                                ctx.nodes().parent_kind(node.id())
                            {
                                fixer.delete(decl)
                            } else {
                                fixer.delete(class)
                            }
                        },
                    );
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
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

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
        ("class Foo { constructor(public bar: string) {} }", None),
        ("class Foo {}", Some(json!([{ "allowEmpty": true }]))),
        ("class Foo { constructor() {} }", Some(json!([{ "allowConstructorOnly": true }]))),
        (
            "
			export class Bar {
			  public static helper(): void {}
			  private static privateHelper(): boolean {
			    return true;
			  }
			}
			      ",
            Some(json!([{ "allowStaticOnly": true }])),
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
        ("@FooDecorator class Foo {} ", Some(json!([{ "allowWithDecorator": true }]))),
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
            Some(json!([{ "allowWithDecorator": true }])),
        ),
        ("abstract class Foo { abstract property: string; }", None),
        ("abstract class Foo { abstract method(): string; }", None),
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
        ("class Foo { constructor() {} }", None),
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
        ("export default class { static hello() {} }", None),
        (
            "
			@FooDecorator
			class Foo {}
            ",
            Some(json!([{ "allowWithDecorator": false }])),
        ),
        (
            "
			@FooDecorator({
              wowThisDecoratorIsQuiteLarge: true,
              itShouldNotBeIncludedIn: 'the diagnostic span',
            })
			class Foo {}
            ",
            Some(json!([{ "allowWithDecorator": false }])),
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
            Some(json!([{ "allowWithDecorator": false }])),
        ),
        ("abstract class Foo {}", None),
        ("abstract class Foo { static property: string; }", None),
        ("abstract class Foo { constructor() {} }", None),
    ];

    let fix = vec![
        ("class Foo {}", "", None, FixKind::DangerousSuggestion),
        ("export class Foo {}", "", None, FixKind::DangerousSuggestion),
        (
            "@foo class Foo {}",
            "@foo class Foo {}",
            Some(json!([{ "allowWithDecorator": false }])),
            FixKind::DangerousSuggestion,
        ),
    ];

    Tester::new(NoExtraneousClass::NAME, NoExtraneousClass::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
