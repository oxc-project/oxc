use oxc_ast::{
    ast::{Expression, TSType, TSTypeAnnotation, TSTypeName},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn consistent_generic_constructors_diagnostic_prefer_annotation(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "The generic type arguments should be specified as part of the type annotation.",
    )
    .with_help("Move the generic type to the type annotation")
    .with_label(span)
}
fn consistent_generic_constructors_diagnostic_prefer_constructor(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "The generic type arguments should be specified as part of the constructor type arguments.",
    )
    .with_help("Move the type annotation to the constructor")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ConsistentGenericConstructors(Box<ConsistentGenericConstructorsConfig>);

#[derive(Debug, Default, Clone)]
pub struct ConsistentGenericConstructorsConfig {
    option: PreferGenericType,
}

#[derive(Debug, Default, Clone)]
enum PreferGenericType {
    #[default]
    Constructor,
    TypeAnnotation,
}

impl TryFrom<&str> for PreferGenericType {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "constructor" => Ok(Self::Constructor),
            "type-annotation" => Ok(Self::TypeAnnotation),
            _ => Err("Invalid value"),
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// When constructing a generic class, you can specify the type arguments on either the left-hand side (as a type annotation) or the right-hand side (as part of the constructor call).
    ///
    /// This rule enforces consistency in the way generic constructors are used.
    ///
    /// ### Why is this bad?
    ///
    /// Inconsistent usage of generic constructors can make the code harder to read and maintain.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const a: Foo<string> = new Foo();
    /// const a = new Foo<string>(); // prefer type annotation
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const a = new Foo<string>();
    /// const a: Foo<string> = new Foo(); // prefer type annotation
    /// ```
    ConsistentGenericConstructors,
    typescript,
    style,
    pending
);

impl Rule for ConsistentGenericConstructors {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::VariableDeclarator(variable_declarator) => {
                let type_ann = variable_declarator.id.type_annotation.as_ref();
                let init = variable_declarator.init.as_ref();
                self.check(type_ann, init, ctx);
            }
            AstKind::AssignmentPattern(assignment_pattern) => {
                let Some(parent) = ctx.nodes().parent_kind(node.id()) else {
                    return;
                };

                if !matches!(parent, AstKind::FormalParameter(_)) {
                    return;
                }

                let type_ann = assignment_pattern.left.type_annotation.as_ref();
                let init = &assignment_pattern.right;
                self.check(type_ann, Some(init), ctx);
            }
            AstKind::PropertyDefinition(property_definition) => {
                let type_ann = property_definition.type_annotation.as_ref();
                let init = property_definition.value.as_ref();
                self.check(type_ann, init, ctx);
            }
            _ => {}
        }
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(ConsistentGenericConstructorsConfig {
            option: value
                .get(0)
                .and_then(|v| v.as_str())
                .and_then(|s| PreferGenericType::try_from(s).ok())
                .unwrap_or_default(),
        }))
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

impl ConsistentGenericConstructors {
    fn check(
        &self,
        type_annotation: Option<&oxc_allocator::Box<TSTypeAnnotation>>,
        init: Option<&Expression>,
        ctx: &LintContext,
    ) {
        let Some(init) = init else { return };
        let Expression::NewExpression(new_expression) = init.get_inner_expression() else {
            return;
        };
        let Expression::Identifier(identifier) = &new_expression.callee else {
            return;
        };
        if let Some(type_annotation) = type_annotation {
            if let TSType::TSTypeReference(type_annotation) = &type_annotation.type_annotation {
                if let TSTypeName::IdentifierReference(ident) = &type_annotation.type_name {
                    if ident.name != identifier.name {
                        return;
                    }
                } else {
                    return;
                }
            } else {
                return;
            }
        }

        if matches!(self.0.option, PreferGenericType::TypeAnnotation) {
            if type_annotation.is_none() {
                if let Some(type_arguments) = &new_expression.type_parameters {
                    ctx.diagnostic(consistent_generic_constructors_diagnostic_prefer_annotation(
                        type_arguments.span,
                    ));
                }
            }
            return;
        }

        if let Some(type_arguments) = &type_annotation {
            if has_type_parameters(&type_arguments.type_annotation)
                && new_expression.type_parameters.is_none()
            {
                ctx.diagnostic(consistent_generic_constructors_diagnostic_prefer_constructor(
                    type_arguments.span,
                ));
            }
        }
    }
}

fn has_type_parameters(ts_type: &TSType) -> bool {
    match ts_type {
        TSType::TSTypeReference(type_ref) => type_ref.type_parameters.is_some(),
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("const a = new Foo();", None),
        ("const a = new Foo<string>();", None),
        ("const a: Foo<string> = new Foo<string>();", None),
        ("const a: Foo = new Foo();", None),
        ("const a: Bar<string> = new Foo();", None),
        ("const a: Foo = new Foo<string>();", None),
        ("const a: Bar = new Foo<string>();", None),
        ("const a: Bar<string> = new Foo<string>();", None),
        ("const a: Foo<string> = Foo<string>();", None),
        ("const a: Foo<string> = Foo();", None),
        ("const a: Foo = Foo<string>();", None),
        (
            "
			class Foo {
			  a = new Foo<string>();
			}
			    ",
            None,
        ),
        (
            "
			function foo(a: Foo = new Foo<string>()) {}
			    ",
            None,
        ),
        (
            "
			function foo({ a }: Foo = new Foo<string>()) {}
			    ",
            None,
        ),
        (
            "
			function foo([a]: Foo = new Foo<string>()) {}
			    ",
            None,
        ),
        (
            "
			class A {
			  constructor(a: Foo = new Foo<string>()) {}
			}
			    ",
            None,
        ),
        (
            "
			const a = function (a: Foo = new Foo<string>()) {};
			    ",
            None,
        ),
        ("const a = new Foo();", Some(serde_json::json!(["type-annotation"]))),
        ("const a: Foo<string> = new Foo();", Some(serde_json::json!(["type-annotation"]))),
        ("const a: Foo<string> = new Foo<string>();", Some(serde_json::json!(["type-annotation"]))),
        ("const a: Foo = new Foo();", Some(serde_json::json!(["type-annotation"]))),
        ("const a: Bar = new Foo<string>();", Some(serde_json::json!(["type-annotation"]))),
        ("const a: Bar<string> = new Foo<string>();", Some(serde_json::json!(["type-annotation"]))),
        ("const a: Foo<string> = Foo<string>();", Some(serde_json::json!(["type-annotation"]))),
        ("const a: Foo<string> = Foo();", Some(serde_json::json!(["type-annotation"]))),
        ("const a: Foo = Foo<string>();", Some(serde_json::json!(["type-annotation"]))),
        ("const a = new (class C<T> {})<string>();", Some(serde_json::json!(["type-annotation"]))),
        (
            "
			class Foo {
			  a: Foo<string> = new Foo();
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function foo(a: Foo<string> = new Foo()) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function foo({ a }: Foo<string> = new Foo()) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function foo([a]: Foo<string> = new Foo()) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			class A {
			  constructor(a: Foo<string> = new Foo()) {}
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			const a = function (a: Foo<string> = new Foo()) {};
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			const [a = new Foo<string>()] = [];
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function a([a = new Foo<string>()]) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
    ];

    let fail = vec![
        ("const a: Foo<string> = new Foo();", None),
        ("const a: Map<string, number> = new Map();", None),
        ("const a: Map <string, number> = new Map();", None),
        ("const a: Map< string, number > = new Map();", None),
        ("const a: Map<string, number> = new Map ();", None),
        ("const a: Foo<number> = new Foo;", None),
        ("const a: /* comment */ Foo/* another */ <string> = new Foo();", None),
        ("const a: Foo/* comment */ <string> = new Foo /* another */();", None),
        (
            "const a: Foo<string> = new
			 Foo
			 ();",
            None,
        ),
        (
            "
			class Foo {
			  a: Foo<string> = new Foo();
			}
			      ",
            None,
        ),
        (
            "
			class Foo {
			  [a]: Foo<string> = new Foo();
			}
			      ",
            None,
        ),
        (
            "
			function foo(a: Foo<string> = new Foo()) {}
			      ",
            None,
        ),
        (
            "
			function foo({ a }: Foo<string> = new Foo()) {}
			      ",
            None,
        ),
        (
            "
			function foo([a]: Foo<string> = new Foo()) {}
			      ",
            None,
        ),
        (
            "
			class A {
			  constructor(a: Foo<string> = new Foo()) {}
			}
			      ",
            None,
        ),
        (
            "
			const a = function (a: Foo<string> = new Foo()) {};
			      ",
            None,
        ),
        ("const a = new Foo<string>();", Some(serde_json::json!(["type-annotation"]))),
        ("const a = new Map<string, number>();", Some(serde_json::json!(["type-annotation"]))),
        ("const a = new Map <string, number> ();", Some(serde_json::json!(["type-annotation"]))),
        ("const a = new Map< string, number >();", Some(serde_json::json!(["type-annotation"]))),
        (
            "const a = new
			 Foo<string>
			 ();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "const a = new Foo/* comment */ <string> /* another */();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "const a = new Foo</* comment */ string, /* another */ number>();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			class Foo {
			  a = new Foo<string>();
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			class Foo {
			  [a] = new Foo<string>();
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			class Foo {
			  [a + b] = new Foo<string>();
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function foo(a = new Foo<string>()) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function foo({ a } = new Foo<string>()) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function foo([a] = new Foo<string>()) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			class A {
			  constructor(a = new Foo<string>()) {}
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			const a = function (a = new Foo<string>()) {};
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
    ];

    let _fix = vec![
        ("const a: Foo<string> = new Foo();", "const a = new Foo<string>();", None),
        ("const a: Map<string, number> = new Map();", "const a = new Map<string, number>();", None),
        (
            "const a: Map <string, number> = new Map();",
            "const a = new Map<string, number>();",
            None,
        ),
        (
            "const a: Map< string, number > = new Map();",
            "const a = new Map< string, number >();",
            None,
        ),
        (
            "const a: Map<string, number> = new Map ();",
            "const a = new Map<string, number> ();",
            None,
        ),
        ("const a: Foo<number> = new Foo;", "const a = new Foo<number>();", None),
        (
            "const a: /* comment */ Foo/* another */ <string> = new Foo();",
            "const a = new Foo/* comment *//* another */<string>();",
            None,
        ),
        (
            "const a: Foo/* comment */ <string> = new Foo /* another */();",
            "const a = new Foo/* comment */<string> /* another */();",
            None,
        ),
        (
            "const a: Foo<string> = new
			 Foo
			 ();",
            "const a = new
			 Foo<string>
			 ();",
            None,
        ),
        (
            "
			class Foo {
			  a: Foo<string> = new Foo();
			}
			      ",
            "
			class Foo {
			  a = new Foo<string>();
			}
			      ",
            None,
        ),
        (
            "
			class Foo {
			  [a]: Foo<string> = new Foo();
			}
			      ",
            "
			class Foo {
			  [a] = new Foo<string>();
			}
			      ",
            None,
        ),
        (
            "
			function foo(a: Foo<string> = new Foo()) {}
			      ",
            "
			function foo(a = new Foo<string>()) {}
			      ",
            None,
        ),
        (
            "
			function foo({ a }: Foo<string> = new Foo()) {}
			      ",
            "
			function foo({ a } = new Foo<string>()) {}
			      ",
            None,
        ),
        (
            "
			function foo([a]: Foo<string> = new Foo()) {}
			      ",
            "
			function foo([a] = new Foo<string>()) {}
			      ",
            None,
        ),
        (
            "
			class A {
			  constructor(a: Foo<string> = new Foo()) {}
			}
			      ",
            "
			class A {
			  constructor(a = new Foo<string>()) {}
			}
			      ",
            None,
        ),
        (
            "
			const a = function (a: Foo<string> = new Foo()) {};
			      ",
            "
			const a = function (a = new Foo<string>()) {};
			      ",
            None,
        ),
        (
            "const a = new Foo<string>();",
            "const a: Foo<string> = new Foo();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "const a = new Map<string, number>();",
            "const a: Map<string, number> = new Map();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "const a = new Map <string, number> ();",
            "const a: Map<string, number> = new Map  ();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "const a = new Map< string, number >();",
            "const a: Map< string, number > = new Map();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "const a = new
			 Foo<string>
			 ();",
            "const a: Foo<string> = new
			 Foo
			 ();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "const a = new Foo/* comment */ <string> /* another */();",
            "const a: Foo<string> = new Foo/* comment */  /* another */();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "const a = new Foo</* comment */ string, /* another */ number>();",
            "const a: Foo</* comment */ string, /* another */ number> = new Foo();",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			class Foo {
			  a = new Foo<string>();
			}
			      ",
            "
			class Foo {
			  a: Foo<string> = new Foo();
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			class Foo {
			  [a] = new Foo<string>();
			}
			      ",
            "
			class Foo {
			  [a]: Foo<string> = new Foo();
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			class Foo {
			  [a + b] = new Foo<string>();
			}
			      ",
            "
			class Foo {
			  [a + b]: Foo<string> = new Foo();
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function foo(a = new Foo<string>()) {}
			      ",
            "
			function foo(a: Foo<string> = new Foo()) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function foo({ a } = new Foo<string>()) {}
			      ",
            "
			function foo({ a }: Foo<string> = new Foo()) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			function foo([a] = new Foo<string>()) {}
			      ",
            "
			function foo([a]: Foo<string> = new Foo()) {}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			class A {
			  constructor(a = new Foo<string>()) {}
			}
			      ",
            "
			class A {
			  constructor(a: Foo<string> = new Foo()) {}
			}
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
        (
            "
			const a = function (a = new Foo<string>()) {};
			      ",
            "
			const a = function (a: Foo<string> = new Foo()) {};
			      ",
            Some(serde_json::json!(["type-annotation"])),
        ),
    ];
    Tester::new(
        ConsistentGenericConstructors::NAME,
        ConsistentGenericConstructors::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
