use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{
        AssignmentExpression, AssignmentTarget, Class, ClassElement, Expression, FormalParameter,
        MethodDefinition, MethodDefinitionKind, PropertyDefinition, PropertyKey, Statement,
        TSAccessibility,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, GetSpan, Span};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

fn prefer_class_property_diagnostic(parameter: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Property {parameter} should be declared as a class property."))
        .with_help("Remove the parameter modifier and declare this member on the class instead.")
        .with_label(span)
}

fn prefer_parameter_property_diagnostic(parameter: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Property {parameter} should be declared as a parameter property."))
        .with_help("Declare this member as a constructor parameter property and remove the class field plus assignment.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum Prefer {
    #[default]
    ClassProperty,
    ParameterProperty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
enum Modifier {
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "private readonly")]
    PrivateReadonly,
    #[serde(rename = "protected")]
    Protected,
    #[serde(rename = "protected readonly")]
    ProtectedReadonly,
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "public readonly")]
    PublicReadonly,
    #[serde(rename = "readonly")]
    Readonly,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct ParameterPropertiesConfig {
    /// Modifiers that are allowed to be used with parameter properties or class properties, depending on the `prefer` option.
    allow: Vec<Modifier>,
    /// Whether to prefer parameter properties or class properties.
    prefer: Prefer,
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
pub struct ParameterProperties(Box<ParameterPropertiesConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires or disallows parameter properties in class constructors.
    ///
    /// ### Why is this bad?
    ///
    /// Mixing parameter properties and class property declarations can make
    /// class style inconsistent and harder to maintain.
    ///
    /// ### Examples
    ///
    /// #### `{ "prefer": "class-property" }` (default)
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// class Foo {
    ///   constructor(private name: string) {}
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// class Foo {
    ///   name: string;
    ///   constructor(name: string) {
    ///     this.name = name;
    ///   }
    /// }
    /// ```
    ///
    /// #### `{ "prefer": "parameter-property" }`
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// class Foo {
    ///   name: string;
    ///   constructor(name: string) {
    ///     this.name = name;
    ///   }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// class Foo {
    ///   constructor(private name: string) {}
    /// }
    /// ```
    ParameterProperties,
    typescript,
    style,
    config = ParameterPropertiesConfig,
);

impl Rule for ParameterProperties {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::MethodDefinition(method)
                if method.kind == MethodDefinitionKind::Constructor
                    && self.0.prefer == Prefer::ClassProperty =>
            {
                self.check_prefer_class_property(method, ctx);
            }
            AstKind::Class(class) if self.0.prefer == Prefer::ParameterProperty => {
                self.check_prefer_parameter_property(class, ctx);
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

#[derive(Default)]
struct PropertyNodes<'a> {
    class_property: Option<&'a PropertyDefinition<'a>>,
    constructor_assignment: Option<&'a AssignmentExpression<'a>>,
    constructor_parameter: Option<&'a FormalParameter<'a>>,
}

impl ParameterProperties {
    fn check_prefer_class_property<'a>(
        &self,
        method: &MethodDefinition<'a>,
        ctx: &LintContext<'a>,
    ) {
        for parameter in &method.value.params.items {
            if !parameter.has_modifier() {
                continue;
            }

            if self.is_allowed_modifier(modifier_from_parameter(parameter)) {
                continue;
            }

            let Some(name) = parameter.pattern.get_binding_identifier().map(|id| id.name) else {
                continue;
            };

            ctx.diagnostic(prefer_class_property_diagnostic(name.as_str(), parameter.span));
        }
    }

    fn check_prefer_parameter_property<'a>(&self, class: &Class<'a>, ctx: &LintContext<'a>) {
        let mut property_nodes_by_name = FxHashMap::<Atom<'a>, PropertyNodes<'a>>::default();

        for element in &class.body.body {
            let ClassElement::PropertyDefinition(property) = element else { continue };
            if property.computed || property.value.is_some() {
                continue;
            }
            let PropertyKey::StaticIdentifier(identifier) = &property.key else { continue };
            if self.is_allowed_modifier(modifier_from_property(property)) {
                continue;
            }

            property_nodes_by_name.entry(identifier.name.into()).or_default().class_property =
                Some(property);
        }

        for element in &class.body.body {
            let ClassElement::MethodDefinition(method) = element else { continue };
            if method.kind != MethodDefinitionKind::Constructor {
                continue;
            }

            for parameter in &method.value.params.items {
                if parameter.initializer.is_some() {
                    continue;
                }
                let Some(identifier) = parameter.pattern.get_binding_identifier() else { continue };
                property_nodes_by_name
                    .entry(identifier.name.into())
                    .or_default()
                    .constructor_parameter = Some(parameter);
            }

            for statement in
                method.value.body.as_ref().map_or([].as_slice(), |body| &body.statements)
            {
                let Some((assignment, name)) = constructor_assignment(statement) else { break };
                property_nodes_by_name.entry(name).or_default().constructor_assignment =
                    Some(assignment);
            }
        }

        for (name, nodes) in property_nodes_by_name {
            let (Some(class_property), Some(_assignment), Some(constructor_parameter)) =
                (nodes.class_property, nodes.constructor_assignment, nodes.constructor_parameter)
            else {
                continue;
            };

            if !type_annotations_match(class_property, constructor_parameter, ctx) {
                continue;
            }

            ctx.diagnostic(prefer_parameter_property_diagnostic(
                name.as_str(),
                class_property.span,
            ));
        }
    }

    fn is_allowed_modifier(&self, modifier: Option<Modifier>) -> bool {
        modifier.is_some_and(|modifier| self.0.allow.contains(&modifier))
    }
}

fn modifier_from_accessibility(
    accessibility: Option<TSAccessibility>,
    readonly: bool,
) -> Option<Modifier> {
    match (accessibility, readonly) {
        (Some(TSAccessibility::Private), true) => Some(Modifier::PrivateReadonly),
        (Some(TSAccessibility::Private), false) => Some(Modifier::Private),
        (Some(TSAccessibility::Protected), true) => Some(Modifier::ProtectedReadonly),
        (Some(TSAccessibility::Protected), false) => Some(Modifier::Protected),
        (Some(TSAccessibility::Public), true) => Some(Modifier::PublicReadonly),
        (Some(TSAccessibility::Public), false) => Some(Modifier::Public),
        (None, true) => Some(Modifier::Readonly),
        (None, false) => None,
    }
}

fn modifier_from_parameter(parameter: &FormalParameter<'_>) -> Option<Modifier> {
    modifier_from_accessibility(parameter.accessibility, parameter.readonly)
}

fn modifier_from_property(property: &PropertyDefinition<'_>) -> Option<Modifier> {
    modifier_from_accessibility(property.accessibility, property.readonly)
}

fn constructor_assignment<'a>(
    statement: &'a Statement<'a>,
) -> Option<(&'a AssignmentExpression<'a>, Atom<'a>)> {
    let Statement::ExpressionStatement(expression_statement) = statement else {
        return None;
    };
    let Expression::AssignmentExpression(assignment) = &expression_statement.expression else {
        return None;
    };
    let AssignmentTarget::StaticMemberExpression(member_expression) = &assignment.left else {
        return None;
    };
    if !matches!(member_expression.object.get_inner_expression(), Expression::ThisExpression(_)) {
        return None;
    }
    let Expression::Identifier(identifier) = assignment.right.get_inner_expression() else {
        return None;
    };
    if member_expression.property.name != identifier.name {
        return None;
    }
    Some((assignment, identifier.name.into()))
}

fn type_annotations_match(
    class_property: &PropertyDefinition<'_>,
    constructor_parameter: &FormalParameter<'_>,
    ctx: &LintContext<'_>,
) -> bool {
    match (&class_property.type_annotation, &constructor_parameter.type_annotation) {
        (None, None) => true,
        (Some(left), Some(right)) => {
            ctx.source_range(left.span()) == ctx.source_range(right.span())
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
            class Foo {
              constructor(name: string) {}
            }
                ",
            None,
        ),
        (
            "
            class Foo {
              constructor(name: string) {}
            }
                  ",
            Some(serde_json::json!([{ "prefer": "class-property" }])),
        ),
        (
            "
            class Foo {
              constructor(...name: string[]) {}
            }
                ",
            None,
        ),
        (
            "
            class Foo {
              constructor(name: string, age: number) {}
            }
                ",
            None,
        ),
        (
            "
            class Foo {
              constructor(name: string) {}
              constructor(name: string, age?: number) {}
            }
                ",
            None,
        ),
        (
            "
            class Foo {
              constructor(readonly name: string) {}
            }
                  ",
            Some(serde_json::json!([{ "allow": ["readonly"] }])),
        ),
        (
            "
            class Foo {
              constructor(private name: string) {}
            }
                  ",
            Some(serde_json::json!([{ "allow": ["private"] }])),
        ),
        (
            "
            class Foo {
              constructor(protected name: string) {}
            }
                  ",
            Some(serde_json::json!([{ "allow": ["protected"] }])),
        ),
        (
            "
            class Foo {
              constructor(public name: string) {}
            }
                  ",
            Some(serde_json::json!([{ "allow": ["public"] }])),
        ),
        (
            "
            class Foo {
              constructor(private readonly name: string) {}
            }
                  ",
            Some(serde_json::json!([{ "allow": ["private readonly"] }])),
        ),
        (
            "
            class Foo {
              constructor(protected readonly name: string) {}
            }
                  ",
            Some(serde_json::json!([{ "allow": ["protected readonly"] }])),
        ),
        (
            "
            class Foo {
              constructor(public readonly name: string) {}
            }
                  ",
            Some(serde_json::json!([{ "allow": ["public readonly"] }])),
        ),
        (
            "
            class Foo {
              constructor(
                readonly name: string,
                private age: number,
              ) {}
            }
                  ",
            Some(serde_json::json!([{ "allow": ["readonly", "private"] }])),
        ),
        (
            "
            class Foo {
              constructor(
                public readonly name: string,
                private age: number,
              ) {}
            }
                  ",
            Some(serde_json::json!([{ "allow": ["public readonly", "private"] }])),
        ),
        (
            "
            class Foo {
              constructor(private name: string[]) {}
            }
                  ",
            Some(serde_json::json!([{ "prefer": "parameter-property" }])),
        ),
        (
            "
            class Foo {
              constructor(...name: string[]) {}
            }
                  ",
            Some(serde_json::json!([{ "prefer": "parameter-property" }])),
        ),
        (
            "
            class Foo {
              constructor(age: string, ...name: string[]) {}
            }
                  ",
            Some(serde_json::json!([{ "prefer": "parameter-property" }])),
        ),
        (
            "
            class Foo {
              constructor(
                private age: string,
                ...name: string[]
              ) {}
            }
                  ",
            Some(serde_json::json!([{ "prefer": "parameter-property" }])),
        ),
        (
            "
            class Foo {
              public age: number;
              constructor(age: string) {
                this.age = age;
              }
            }
                  ",
            Some(serde_json::json!([{ "prefer": "parameter-property" }])),
        ),
        (
            "
            class Foo {
              public age = '';
              constructor(age: string) {
                this.age = age;
              }
            }
                  ",
            Some(serde_json::json!([{ "prefer": "parameter-property" }])),
        ),
        (
            "
            class Foo {
              public age;
              constructor(age: string) {
                this.age = age;
              }
            }
                  ",
            Some(serde_json::json!([{ "prefer": "parameter-property" }])),
        ),
        (
            "
            class Foo {
              public age: string;
              constructor(age) {
                this.age = age;
              }
            }
                  ",
            Some(serde_json::json!([{ "prefer": "parameter-property" }])),
        ),
        (
            "
            class Foo {
              public age: string;
              constructor(age: string) {
                console.log('unrelated');
                this.age = age;
              }
            }
                  ",
            Some(serde_json::json!([{ "prefer": "parameter-property" }])),
        ),
        (
            "
            class Foo {
              other: string;
              constructor(age: string) {
                this.other = age;
              }
            }
                  ",
            Some(serde_json::json!([{ "prefer": "parameter-property" }])),
        ),
        (
            "
            class Foo {
              prop: string;
              other: string;
              constructor(prop: string) {
                this.other = prop;
              }
            }
                  ",
            Some(serde_json::json!([{ "prefer": "parameter-property" }])),
        ),
        (
            "
            class Foo {
              age: string;
              constructor(age: string) {
                this.age = '';
                console.log(age);
              }
            }
                  ",
            Some(serde_json::json!([{ "prefer": "parameter-property" }])),
        ),
        (
            "
            class Foo {
              age() {
                return '';
              }
              constructor(age: string) {
                this.age = age;
              }
            }
                  ",
            Some(serde_json::json!([{ "prefer": "parameter-property" }])),
        ),
        (
            "
            class Foo {
              public age: string;
              constructor(age: string) {
                this.age = age;
              }
            }
                  ",
            Some(serde_json::json!([{ "allow": ["public"], "prefer": "parameter-property" }])),
        ),
        (
            "
            class Foo {
              public readonly age: string;
              constructor(age: string) {
                this.age = age;
              }
            }
                  ",
            Some(
                serde_json::json!([{ "allow": ["public readonly"], "prefer": "parameter-property" }]),
            ),
        ),
        (
            "
            class Foo {
              protected age: string;
              constructor(age: string) {
                this.age = age;
              }
            }
                  ",
            Some(serde_json::json!([{ "allow": ["protected"], "prefer": "parameter-property" }])),
        ),
        (
            "
            class Foo {
              protected readonly age: string;
              constructor(age: string) {
                this.age = age;
              }
            }
                  ",
            Some(
                serde_json::json!([ { "allow": ["protected readonly"], "prefer": "parameter-property" }, ]),
            ),
        ),
        (
            "
            class Foo {
              private age: string;
              constructor(age: string) {
                this.age = age;
              }
            }
                  ",
            Some(serde_json::json!([{ "allow": ["private"], "prefer": "parameter-property" }])),
        ),
        (
            "
            class Foo {
              private readonly age: string;
              constructor(age: string) {
                this.age = age;
              }
            }
                  ",
            Some(
                serde_json::json!([{ "allow": ["private readonly"], "prefer": "parameter-property" }]),
            ),
        ),
    ];

    let fail = vec![
        (
            "
            class Foo {
              constructor(readonly name: string) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(private name: string) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(protected name: string) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(public name: string) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(private readonly name: string) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(protected readonly name: string) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(public readonly name: string) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(
                public name: string,
                age: number,
              ) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(
                private name: string,
                private age: number,
              ) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(
                protected name: string,
                protected age: number,
              ) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(
                public name: string,
                public age: number,
              ) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(name: string) {}
              constructor(
                private name: string,
                age?: number,
              ) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(private name: string) {}
              constructor(
                private name: string,
                age?: number,
              ) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(private name: string) {}
              constructor(
                private name: string,
                private age?: number,
              ) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(name: string) {}
              constructor(
                protected name: string,
                age?: number,
              ) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(protected name: string) {}
              constructor(
                protected name: string,
                age?: number,
              ) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(protected name: string) {}
              constructor(
                protected name: string,
                protected age?: number,
              ) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(name: string) {}
              constructor(
                public name: string,
                age?: number,
              ) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(public name: string) {}
              constructor(
                public name: string,
                age?: number,
              ) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(public name: string) {}
              constructor(
                public name: string,
                public age?: number,
              ) {}
            }
                  ",
            None,
        ),
        (
            "
            class Foo {
              constructor(readonly name: string) {}
            }
                  ",
            Some(serde_json::json!([{ "allow": ["private"] }])),
        ),
        (
            "
            class Foo {
              constructor(private name: string) {}
            }
                  ",
            Some(serde_json::json!([{ "allow": ["readonly"] }])),
        ),
        (
            "
            class Foo {
              constructor(protected name: string) {}
            }
                  ",
            Some(
                serde_json::json!([ { "allow": ["readonly", "private", "public", "protected readonly"], }, ]),
            ),
        ),
        (
            "
            class Foo {
              constructor(public name: string) {}
            }
                  ",
            Some(
                serde_json::json!([ { "allow": [ "readonly", "private", "protected", "protected readonly", "public readonly", ], }, ]),
            ),
        ),
        (
            "
            class Foo {
              constructor(private readonly name: string) {}
            }
                  ",
            Some(serde_json::json!([{ "allow": ["readonly", "private"] }])),
        ),
        (
            "
            class Foo {
              constructor(protected readonly name: string) {}
            }
                  ",
            Some(
                serde_json::json!([ { "allow": [ "readonly", "protected", "private readonly", "public readonly", ], }, ]),
            ),
        ),
        (
            "
            class Foo {
              constructor(private name: string) {}
              constructor(
                private name: string,
                protected age?: number,
              ) {}
            }
                  ",
            Some(serde_json::json!([{ "allow": ["private"] }])),
        ),
        (
            "
            class Foo {
              member: string;

              constructor(member: string) {
                this.member = member;
              }
            }
                  ",
            Some(serde_json::json!([{ "prefer": "parameter-property" }])),
        ),
        (
            "
            class Foo {
              constructor(member: string) {
                this.member = member;
              }

              member: string;
            }
                  ",
            Some(serde_json::json!([{ "prefer": "parameter-property" }])),
        ),
        (
            "
            class Foo {
              member;
              constructor(member) {
                this.member = member;
              }
            }
                  ",
            Some(serde_json::json!([{ "prefer": "parameter-property" }])),
        ),
        (
            "
            class Foo {
              public member: string;
              constructor(member: string) {
                this.member = member;
              }
            }
                  ",
            Some(
                serde_json::json!([ { "allow": ["protected", "private", "readonly"], "prefer": "parameter-property", }, ]),
            ),
        ),
    ];

    Tester::new(ParameterProperties::NAME, ParameterProperties::PLUGIN, pass, fail)
        .test_and_snapshot();
}
