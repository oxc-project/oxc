use std::ops::Deref;

use oxc_ast::{
    AstKind,
    ast::{
        ArrowFunctionExpression, AssignmentExpression, AssignmentTarget, Class, ClassBody,
        ClassElement, Expression, Function, MethodDefinitionKind, PropertyDefinition, PropertyKey,
        Statement,
    },
};
use oxc_ast_visit::Visit;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeFlags;
use oxc_span::{Atom, GetSpan, Span};
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn prefer_field_style_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Literals should be exposed using readonly fields.")
        .with_help("Replace this getter with a readonly field initialized to the returned literal.")
        .with_label(span)
}

fn prefer_getter_style_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Literals should be exposed using getters.")
        .with_help("Replace this readonly literal field with a getter.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ClassLiteralPropertyStyleOption {
    #[default]
    Fields,
    Getters,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClassLiteralPropertyStyle(Box<ClassLiteralPropertyStyleOption>);

impl Default for ClassLiteralPropertyStyle {
    fn default() -> Self {
        Self(Box::new(ClassLiteralPropertyStyleOption::Fields))
    }
}

impl Deref for ClassLiteralPropertyStyle {
    type Target = ClassLiteralPropertyStyleOption;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a consistent style for exposing literal values on classes.
    ///
    /// ### Why is this bad?
    ///
    /// Mixing readonly fields and trivial literal getters for the same kind of value
    /// makes class APIs inconsistent and harder to scan.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule (default `"fields"`):
    /// ```ts
    /// class C {
    ///   get name() {
    ///     return "oxc";
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// class C {
    ///   readonly name = "oxc";
    /// }
    /// ```
    ClassLiteralPropertyStyle,
    typescript,
    style,
    pending,
    config = ClassLiteralPropertyStyleOption
);

impl Rule for ClassLiteralPropertyStyle {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
        Some(serde_json::to_value(*self.0))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ClassBody(class_body) = node.kind() else {
            return;
        };

        match **self {
            ClassLiteralPropertyStyleOption::Fields => check_fields_mode(class_body, ctx),
            ClassLiteralPropertyStyleOption::Getters => check_getters_mode(class_body, ctx),
        }
    }
}

fn check_fields_mode<'a>(class_body: &ClassBody<'a>, ctx: &LintContext<'a>) {
    for element in &class_body.body {
        if let ClassElement::MethodDefinition(method) = element
            && method.kind == MethodDefinitionKind::Get
            && !method.r#override
            && let Some(body) = &method.value.body
            && let Some(Statement::ReturnStatement(return_statement)) = body.statements.first()
            && let Some(argument) = &return_statement.argument
            && is_supported_literal(argument)
            && !has_duplicate_setter(class_body, method)
        {
            ctx.diagnostic(prefer_field_style_diagnostic(method.key.span()));
        }
    }
}

fn check_getters_mode<'a>(class_body: &ClassBody<'a>, ctx: &LintContext<'a>) {
    let mut excluded_properties: FxHashSet<Atom<'a>> = FxHashSet::default();
    for element in &class_body.body {
        if let ClassElement::MethodDefinition(method) = element
            && method.kind == MethodDefinitionKind::Constructor
            && let Some(body) = &method.value.body
        {
            let mut collector =
                ConstructorAssignmentCollector { excluded_properties: &mut excluded_properties };
            collector.visit_function_body(body);
        }
    }

    for element in &class_body.body {
        if let ClassElement::PropertyDefinition(property) = element
            && is_literal_readonly_property(property)
        {
            if let Some(name) = property.key.name()
                && excluded_properties.contains(&*name)
            {
                continue;
            }

            ctx.diagnostic(prefer_getter_style_diagnostic(property.key.span()));
        }
    }
}

fn has_duplicate_setter<'a>(
    class_body: &ClassBody<'a>,
    getter: &oxc_ast::ast::MethodDefinition<'a>,
) -> bool {
    class_body.body.iter().any(|element| {
        let ClassElement::MethodDefinition(method) = element else {
            return false;
        };
        if method.kind != MethodDefinitionKind::Set || method.r#static != getter.r#static {
            return false;
        }
        property_keys_match(&method.key, &getter.key)
    })
}

fn is_literal_readonly_property(property: &PropertyDefinition<'_>) -> bool {
    property.readonly
        && !property.declare
        && !property.r#override
        && property.value.as_ref().is_some_and(is_supported_literal)
}

fn is_supported_literal(expression: &Expression<'_>) -> bool {
    if expression.is_literal() {
        return true;
    }

    match expression {
        Expression::TaggedTemplateExpression(tagged_template) => {
            tagged_template.quasi.is_no_substitution_template()
        }
        Expression::TemplateLiteral(template_literal) => {
            template_literal.is_no_substitution_template()
        }
        _ => false,
    }
}

fn property_keys_match(a: &PropertyKey<'_>, b: &PropertyKey<'_>) -> bool {
    match (a.name(), b.name()) {
        (Some(a_name), Some(b_name)) => a_name == b_name,
        _ => match (a.as_expression(), b.as_expression()) {
            (Some(Expression::Identifier(a_ident)), Some(Expression::Identifier(b_ident))) => {
                a_ident.name == b_ident.name
            }
            _ => false,
        },
    }
}

fn assigned_this_property_name<'a>(left: &AssignmentTarget<'a>) -> Option<Atom<'a>> {
    let is_this_object =
        |expr: &Expression<'_>| matches!(expr.without_parentheses(), Expression::ThisExpression(_));

    match left {
        AssignmentTarget::StaticMemberExpression(expr) if is_this_object(&expr.object) => {
            Some(expr.property.name.as_atom())
        }
        AssignmentTarget::ComputedMemberExpression(expr) if is_this_object(&expr.object) => {
            expr.static_property_name()
        }
        AssignmentTarget::PrivateFieldExpression(expr) if is_this_object(&expr.object) => {
            Some(expr.field.name.as_atom())
        }
        _ => None,
    }
}

struct ConstructorAssignmentCollector<'set, 'a> {
    excluded_properties: &'set mut FxHashSet<Atom<'a>>,
}

impl<'a> Visit<'a> for ConstructorAssignmentCollector<'_, 'a> {
    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        if let Some(name) = assigned_this_property_name(&assignment.left) {
            self.excluded_properties.insert(name);
        }
    }

    fn visit_function(&mut self, _it: &Function<'a>, _flags: ScopeFlags) {
        // Ignore nested functions to stay in constructor "this" context.
    }

    fn visit_arrow_function_expression(&mut self, _it: &ArrowFunctionExpression<'a>) {}

    fn visit_class(&mut self, _it: &Class<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
            class Mx {
              declare readonly p1 = 1;
            }
                ",
            None,
        ),
        (
            "
            class Mx {
              readonly p1 = 'hello world';
            }
                ",
            None,
        ),
        (
            "
            class Mx {
              p1 = 'hello world';
            }
                ",
            None,
        ),
        (
            "
            class Mx {
              static p1 = 'hello world';
            }
                ",
            None,
        ),
        (
            "
            class Mx {
              p1: string;
            }
                ",
            None,
        ),
        (
            "
            class Mx {
              get p1();
            }
                ",
            None,
        ),
        (
            "
            class Mx {
              get p1() {}
            }
                ",
            None,
        ),
        (
            "
            abstract class Mx {
              abstract get p1(): string;
            }
                ",
            None,
        ),
        (
            "
                  class Mx {
                    get mySetting() {
                      if (this._aValue) {
                        return 'on';
                      }

                      return 'off';
                    }
                  }
                ",
            None,
        ),
        (
            "
                  class Mx {
                    get mySetting() {
                      return `build-${process.env.build}`;
                    }
                  }
                ",
            None,
        ),
        (
            "
                  class Mx {
                    getMySetting() {
                      if (this._aValue) {
                        return 'on';
                      }

                      return 'off';
                    }
                  }
                ",
            None,
        ),
        (
            "
                  class Mx {
                    public readonly myButton = styled.button`
                      color: ${props => (props.primary ? 'hotpink' : 'turquoise')};
                    `;
                  }
                ",
            None,
        ),
        (
            "
                  class Mx {
                    set p1(val) {}
                    get p1() {
                      return '';
                    }
                  }
                ",
            None,
        ),
        (
            "
                  let p1 = 'p1';
                  class Mx {
                    set [p1](val) {}
                    get [p1]() {
                      return '';
                    }
                  }
                ",
            None,
        ),
        (
            "
                  let p1 = 'p1';
                  class Mx {
                    set [/* before set */ p1 /* after set */](val) {}
                    get [/* before get */ p1 /* after get */]() {
                      return '';
                    }
                  }
                ",
            None,
        ),
        (
            "
                  class Mx {
                    set ['foo'](val) {}
                    get foo() {
                      return '';
                    }
                    set bar(val) {}
                    get ['bar']() {
                      return '';
                    }
                    set ['baz'](val) {}
                    get baz() {
                      return '';
                    }
                  }
                ",
            None,
        ),
        (
            "
                    class Mx {
                      public get myButton() {
                        return styled.button`
                          color: ${props => (props.primary ? 'hotpink' : 'turquoise')};
                        `;
                      }
                    }
                  ",
            Some(serde_json::json!(["fields"])),
        ),
        (
            "
            class Mx {
              declare public readonly foo = 1;
            }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
            class Mx {
              get p1() {
                return 'hello world';
              }
            }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
            class Mx {
              p1 = 'hello world';
            }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
            class Mx {
              p1: string;
            }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
            class Mx {
              readonly p1 = [1, 2, 3];
            }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
            class Mx {
              static p1: string;
            }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
            class Mx {
              static get p1() {
                return 'hello world';
              }
            }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
                    class Mx {
                      public readonly myButton = styled.button`
                        color: ${props => (props.primary ? 'hotpink' : 'turquoise')};
                      `;
                    }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
                    class Mx {
                      public get myButton() {
                        return styled.button`
                          color: ${props => (props.primary ? 'hotpink' : 'turquoise')};
                        `;
                      }
                    }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
                    class A {
                      private readonly foo: string = 'bar';
                      constructor(foo: string) {
                        this.foo = foo;
                      }
                    }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
                    class A {
                      private readonly foo: string = 'bar';
                      constructor(foo: string) {
                        this['foo'] = foo;
                      }
                    }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
                    class A {
                      private readonly foo: string = 'bar';
                      constructor(foo: string) {
                        const bar = new (class {
                          private readonly foo: string = 'baz';
                          constructor() {
                            this.foo = 'qux';
                          }
                        })();
                        this['foo'] = foo;
                      }
                    }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
            declare abstract class BaseClass {
              get cursor(): string;
            }

            class ChildClass extends BaseClass {
              override get cursor() {
                return 'overridden value';
              }
            }
                  ",
            None,
        ),
        (
            "
            declare abstract class BaseClass {
              protected readonly foo: string;
            }

            class ChildClass extends BaseClass {
              protected override readonly foo = 'bar';
            }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
    ];

    let fail = vec![
        (
            "
            class Mx {
              get p1() {
                return 'hello world';
              }
            }
                  ",
            None,
        ),
        (
            "
            class Mx {
              get p1() {
                return `hello world`;
              }
            }
                  ",
            None,
        ),
        (
            "
            class Mx {
              static get p1() {
                return 'hello world';
              }
            }
                  ",
            None,
        ),
        (
            "
            class Mx {
              public static get foo() {
                return 1;
              }
            }
                  ",
            None,
        ),
        (
            "
            class Mx {
              public get [myValue]() {
                return 'a literal value';
              }
            }
                  ",
            None,
        ),
        (
            "
            class Mx {
              public get [myValue]() {
                return 12345n;
              }
            }
                  ",
            None,
        ),
        (
            "
            class Mx {
              public readonly [myValue] = 'a literal value';
            }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
            class Mx {
              readonly p1 = 'hello world';
            }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
            class Mx {
              readonly p1 = `hello world`;
            }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
            class Mx {
              static readonly p1 = 'hello world';
            }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
            class Mx {
              protected get p1() {
                return 'hello world';
              }
            }
                  ",
            Some(serde_json::json!(["fields"])),
        ),
        (
            "
            class Mx {
              protected readonly p1 = 'hello world';
            }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
            class Mx {
              public static get p1() {
                return 'hello world';
              }
            }
                  ",
            None,
        ),
        (
            "
            class Mx {
              public static readonly p1 = 'hello world';
            }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
            class Mx {
              public get myValue() {
                return gql`
                  {
                    user(id: 5) {
                      firstName
                      lastName
                    }
                  }
                `;
              }
            }
                  ",
            None,
        ),
        (
            "
            class Mx {
              public readonly myValue = gql`
                {
                  user(id: 5) {
                    firstName
                    lastName
                  }
                }
              `;
            }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
            class A {
              private readonly foo: string = 'bar';
              constructor(foo: string) {
                const bar = new (class {
                  private readonly foo: string = 'baz';
                  constructor() {
                    this.foo = 'qux';
                  }
                })();
              }
            }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
            class A {
              private readonly ['foo']: string = 'bar';
              constructor(foo: string) {
                const bar = new (class {
                  private readonly foo: string = 'baz';
                  constructor() {}
                })();

                if (bar) {
                  this.foo = 'baz';
                }
              }
            }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
        (
            "
            class A {
              private readonly foo: string = 'bar';
              constructor(foo: string) {
                function func() {
                  this.foo = 'aa';
                }
              }
            }
                  ",
            Some(serde_json::json!(["getters"])),
        ),
    ];

    Tester::new(ClassLiteralPropertyStyle::NAME, ClassLiteralPropertyStyle::PLUGIN, pass, fail)
        .test_and_snapshot();
}
