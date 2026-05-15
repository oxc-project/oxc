use std::borrow::Cow;

use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{
    AstKind,
    ast::{
        AccessorProperty, Decorator, FormalParameter, MethodDefinition, MethodDefinitionKind,
        PropertyDefinition, TSAccessibility,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum AccessibilityLevel {
    /// Always require an accessibility modifier.
    #[default]
    Explicit,
    /// Require an accessibility modifier except when public.
    NoPublic,
    /// Never check whether there is an accessibility modifier.
    Off,
}

#[derive(Default, Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct AccessibilityOverrides {
    /// Which member accessibility modifier requirements to apply for accessors (getters/setters).
    accessors: Option<AccessibilityLevel>,
    /// Which member accessibility modifier requirements to apply for constructors.
    constructors: Option<AccessibilityLevel>,
    /// Which member accessibility modifier requirements to apply for methods.
    methods: Option<AccessibilityLevel>,
    /// Which member accessibility modifier requirements to apply for parameter properties.
    parameter_properties: Option<AccessibilityLevel>,
    /// Which member accessibility modifier requirements to apply for properties.
    properties: Option<AccessibilityLevel>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ExplicitMemberAccessibility(Box<ExplicitMemberAccessibilityConfig>);

impl std::ops::Deref for ExplicitMemberAccessibility {
    type Target = ExplicitMemberAccessibilityConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ExplicitMemberAccessibilityConfig {
    /// Which accessibility modifier is required to exist or not exist.
    accessibility: AccessibilityLevel,
    /// Changes to required accessibility modifiers for specific kinds of class members.
    overrides: AccessibilityOverrides,
    /// Specific method names that may be ignored.
    ignored_method_names: Vec<String>,
}

impl Default for ExplicitMemberAccessibilityConfig {
    fn default() -> Self {
        Self {
            accessibility: AccessibilityLevel::Explicit,
            overrides: AccessibilityOverrides::default(),
            ignored_method_names: Vec::new(),
        }
    }
}

fn missing_accessibility_diagnostic(span: Span, member_type: &str, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Missing accessibility modifier on {member_type} {name}."))
        .with_help("Add an explicit 'public', 'private', or 'protected' modifier. Members without a modifier are implicitly public, which may not be intentional.")
        .with_label(span)
}

fn unwanted_public_accessibility_diagnostic(
    span: Span,
    member_type: &str,
    name: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Public accessibility modifier on {member_type} {name}."))
        .with_help("Remove the 'public' modifier. Members are public by default, so the modifier is redundant.")
        .with_label(span)
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require explicit accessibility modifiers on class properties and methods.
    ///
    /// ### Why is this bad?
    ///
    /// TypeScript allows placing explicit `public`, `protected`, and `private`
    /// accessibility modifiers in front of class members. The modifiers exist
    /// solely in the type system and serve to describe who is allowed to access
    /// those members.
    ///
    /// Leaving off accessibility modifiers makes for less code to read and
    /// write. Members are `public` by default. However, adding explicit
    /// modifiers can make code more readable and explicit about who can use
    /// which properties.
    ///
    /// ### Examples
    ///
    /// #### `{ "accessibility": "explicit" }` (default)
    ///
    /// Examples of **incorrect** code:
    /// ```ts
    /// class Animal {
    ///   constructor(name: string) {}
    ///   animalName: string;
    ///   get name(): string { return this.animalName; }
    /// }
    /// ```
    ///
    /// Examples of **correct** code:
    /// ```ts
    /// class Animal {
    ///   public constructor(name: string) {}
    ///   private animalName: string;
    ///   public get name(): string { return this.animalName; }
    /// }
    /// ```
    ///
    /// #### `{ "accessibility": "no-public" }`
    ///
    /// Examples of **incorrect** code:
    /// ```ts
    /// class Animal {
    ///   public constructor(public breed: string, name: string) {}
    ///   public animalName: string;
    ///   public get name(): string { return this.animalName; }
    /// }
    /// ```
    ///
    /// Examples of **correct** code:
    /// ```ts
    /// class Animal {
    ///   constructor(protected breed: string, name: string) {}
    ///   private animalName: string;
    ///   get name(): string { return this.animalName; }
    /// }
    /// ```
    ///
    /// #### `{ "overrides": { "constructors": "no-public" } }`
    ///
    /// Disallow the use of `public` on constructors while requiring explicit
    /// modifiers everywhere else.
    ///
    /// Examples of **incorrect** code:
    /// ```ts
    /// class Animal {
    ///   public constructor(protected animalName: string) {}
    /// }
    /// ```
    ///
    /// Examples of **correct** code:
    /// ```ts
    /// class Animal {
    ///   constructor(protected animalName: string) {}
    ///   public get name(): string { return this.animalName; }
    /// }
    /// ```
    ///
    /// #### `{ "accessibility": "no-public", "overrides": { "properties": "explicit" } }`
    ///
    /// Require explicit modifiers on properties while disallowing `public`
    /// everywhere else.
    ///
    /// Examples of **incorrect** code:
    /// ```ts
    /// class Animal {
    ///   legs: number;
    ///   private hasFleas: boolean;
    /// }
    /// ```
    ///
    /// Examples of **correct** code:
    /// ```ts
    /// class Animal {
    ///   public legs: number;
    ///   private hasFleas: boolean;
    /// }
    /// ```
    ExplicitMemberAccessibility,
    typescript,
    restriction,
    conditional_fix_suggestion,
    config = ExplicitMemberAccessibilityConfig,
    version = "1.61.0",
);

impl Rule for ExplicitMemberAccessibility {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::MethodDefinition(method) => self.check_method(method, ctx),
            AstKind::PropertyDefinition(prop) => self.check_property(prop, ctx),
            AstKind::AccessorProperty(prop) => self.check_accessor_property(prop, ctx),
            AstKind::FormalParameter(param) => self.check_parameter_property(param, ctx),
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

impl ExplicitMemberAccessibility {
    fn check_method<'a>(&self, method: &MethodDefinition<'a>, ctx: &LintContext<'a>) {
        if method.key.is_private_identifier() {
            return;
        }

        let check = match method.kind {
            MethodDefinitionKind::Constructor => self.overrides.constructors,
            MethodDefinitionKind::Get | MethodDefinitionKind::Set => self.overrides.accessors,
            MethodDefinitionKind::Method => self.overrides.methods,
        };

        let check = check.unwrap_or(self.accessibility);

        let method_name = method.key.name().unwrap_or(Cow::Borrowed(""));

        if check == AccessibilityLevel::Off
            || self.ignored_method_names.iter().any(|n| n.as_str() == &*method_name)
        {
            return;
        }

        Self::check_member_accessibility(
            check,
            method.accessibility,
            method_definition_kind_to_str(method.kind),
            &method_name,
            method.span,
            method.key.span(),
            &method.decorators,
            ctx,
        );
    }

    fn check_property<'a>(&self, prop: &PropertyDefinition<'a>, ctx: &LintContext<'a>) {
        if prop.key.is_private_identifier() {
            return;
        }

        let check = self.overrides.properties.unwrap_or(self.accessibility);
        if check == AccessibilityLevel::Off {
            return;
        }

        let name = prop.key.name().unwrap_or(Cow::Borrowed(""));
        Self::check_member_accessibility(
            check,
            prop.accessibility,
            "class property",
            &name,
            prop.span,
            prop.key.span(),
            &prop.decorators,
            ctx,
        );
    }

    fn check_accessor_property<'a>(&self, prop: &AccessorProperty<'a>, ctx: &LintContext<'a>) {
        if prop.key.is_private_identifier() {
            return;
        }

        let check = self.overrides.properties.unwrap_or(self.accessibility);
        if check == AccessibilityLevel::Off {
            return;
        }

        let name = prop.key.name().unwrap_or(Cow::Borrowed(""));
        Self::check_member_accessibility(
            check,
            prop.accessibility,
            "class property",
            &name,
            prop.span,
            prop.key.span(),
            &prop.decorators,
            ctx,
        );
    }

    fn check_parameter_property<'a>(&self, param: &FormalParameter<'a>, ctx: &LintContext<'a>) {
        if !param.has_modifier() {
            return;
        }

        let check = self.overrides.parameter_properties.unwrap_or(self.accessibility);
        if check == AccessibilityLevel::Off {
            return;
        }

        let name = param
            .pattern
            .get_binding_identifier()
            .map_or(Cow::Borrowed(""), |id| Cow::Borrowed(id.name.as_str()));

        match check {
            AccessibilityLevel::Explicit => {
                if param.accessibility.is_none() {
                    let report_span = param.pattern.span();
                    ctx.diagnostic_with_suggestion(
                        missing_accessibility_diagnostic(report_span, "parameter property", &name),
                        |fixer| {
                            let insert_pos = find_insert_position(
                                param.span,
                                &param.decorators,
                                ctx.source_text(),
                            );
                            fixer.insert_text_before_range(
                                Span::new(insert_pos, insert_pos),
                                "public ",
                            )
                        },
                    );
                }
            }
            AccessibilityLevel::NoPublic => {
                if param.accessibility == Some(TSAccessibility::Public) && param.readonly {
                    let search_start = search_start_after_decorators(param.span, &param.decorators);
                    let search_end = param.pattern.span().start;
                    let (public_span, removal_range) =
                        find_public_spans(ctx, search_start, search_end);
                    ctx.diagnostic_with_fix(
                        unwanted_public_accessibility_diagnostic(
                            public_span,
                            "parameter property",
                            &name,
                        ),
                        |fixer| fixer.delete_range(removal_range),
                    );
                }
            }
            AccessibilityLevel::Off => {}
        }
    }

    fn check_member_accessibility(
        check: AccessibilityLevel,
        accessibility: Option<TSAccessibility>,
        node_type: &str,
        name: &str,
        node_span: Span,
        key_span: Span,
        decorators: &[Decorator<'_>],
        ctx: &LintContext<'_>,
    ) {
        if check == AccessibilityLevel::Explicit && accessibility.is_none() {
            ctx.diagnostic_with_suggestion(
                missing_accessibility_diagnostic(key_span, node_type, name),
                |fixer| {
                    let insert_pos = find_insert_position(node_span, decorators, ctx.source_text());
                    fixer.insert_text_before_range(Span::new(insert_pos, insert_pos), "public ")
                },
            );
        } else if check == AccessibilityLevel::NoPublic
            && accessibility == Some(TSAccessibility::Public)
        {
            let search_start = search_start_after_decorators(node_span, decorators);
            let (public_span, removal_range) = find_public_spans(ctx, search_start, key_span.start);
            ctx.diagnostic_with_fix(
                unwanted_public_accessibility_diagnostic(public_span, node_type, name),
                |fixer| fixer.delete_range(removal_range),
            );
        }
    }
}

fn search_start_after_decorators(node_span: Span, decorators: &[Decorator<'_>]) -> u32 {
    decorators.last().map_or(node_span.start, |d| d.span.end)
}

fn find_insert_position(node_span: Span, decorators: &[Decorator<'_>], source: &str) -> u32 {
    skip_ascii_whitespace(source, search_start_after_decorators(node_span, decorators))
}

/// Returns `(keyword_span, removal_span)` where `keyword_span` covers just the `public` keyword
/// and `removal_span` also includes trailing whitespace.
fn find_public_spans(ctx: &LintContext<'_>, search_start: u32, search_end: u32) -> (Span, Span) {
    let start = search_start
        + ctx
            .find_next_token_within(search_start, search_end, "public")
            .expect("Expected 'public' keyword in source");
    let keyword_span = Span::sized(start, 6);
    let end = skip_ascii_whitespace(ctx.source_text(), start + 6);
    (keyword_span, Span::new(start, end))
}

#[expect(clippy::cast_possible_truncation)]
fn skip_ascii_whitespace(source: &str, start: u32) -> u32 {
    let bytes = &source.as_bytes()[start as usize..];
    let offset = bytes.iter().position(|byte| !byte.is_ascii_whitespace()).unwrap_or(bytes.len());
    start + offset as u32
}

fn method_definition_kind_to_str(kind: MethodDefinitionKind) -> &'static str {
    match kind {
        MethodDefinitionKind::Method | MethodDefinitionKind::Constructor => "method definition",
        MethodDefinitionKind::Get => "get property accessor",
        MethodDefinitionKind::Set => "set property accessor",
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
            class Test {
              public constructor(private foo: string) {}
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "explicit", "overrides": { "parameterProperties": "explicit" }, }, ]),
            ),
        ),
        (
            "
            class Test {
              public constructor(private readonly foo: string) {}
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "explicit", "overrides": { "parameterProperties": "explicit" }, }, ]),
            ),
        ),
        (
            "
            class Test {
              public constructor(private foo: string) {}
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "explicit", "overrides": { "parameterProperties": "off" }, }, ]),
            ),
        ),
        (
            "
            class Test {
              public constructor(protected foo: string) {}
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "explicit", "overrides": { "parameterProperties": "off" }, }, ]),
            ),
        ),
        (
            "
            class Test {
              public constructor(public foo: string) {}
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "explicit", "overrides": { "parameterProperties": "off" }, }, ]),
            ),
        ),
        (
            "
            class Test {
              public constructor(readonly foo: string) {}
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "explicit", "overrides": { "parameterProperties": "off" }, }, ]),
            ),
        ),
        (
            "
            class Test {
              public constructor(private readonly foo: string) {}
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "explicit", "overrides": { "parameterProperties": "off" }, }, ]),
            ),
        ),
        (
            "
            class Test {
              protected name: string;
              private x: number;
              public getX() {
                return this.x;
              }
            }
                  ",
            None,
        ),
        (
            "
            class Test {
              protected name: string;
              protected foo?: string;
              public 'foo-bar'?: string;
            }
                  ",
            None,
        ),
        (
            "
            class Test {
              public constructor({ x, y }: { x: number; y: number }) {}
            }
                  ",
            None,
        ),
        (
            "
            class Test {
              protected name: string;
              protected foo?: string;
              public getX() {
                return this.x;
              }
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "explicit" }])),
        ),
        (
            "
            class Test {
              protected name: string;
              protected foo?: string;
              getX() {
                return this.x;
              }
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "no-public" }])),
        ),
        (
            "
            class Test {
              name: string;
              foo?: string;
              getX() {
                return this.x;
              }
              get fooName(): string {
                return this.foo + ' ' + this.name;
              }
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "no-public" }])),
        ),
        (
            "
            class Test {
              private x: number;
              constructor(x: number) {
                this.x = x;
              }
              get internalValue() {
                return this.x;
              }
              private set internalValue(value: number) {
                this.x = value;
              }
              public square(): number {
                return this.x * this.x;
              }
            }
                  ",
            Some(
                serde_json::json!([{ "overrides": { "accessors": "off", "constructors": "off" } }]),
            ),
        ),
        (
            "
            class Test {
              private x: number;
              public constructor(x: number) {
                this.x = x;
              }
              public get internalValue() {
                return this.x;
              }
              public set internalValue(value: number) {
                this.x = value;
              }
              public square(): number {
                return this.x * this.x;
              }
              half(): number {
                return this.x / 2;
              }
            }
                  ",
            Some(serde_json::json!([{ "overrides": { "methods": "off" } }])),
        ),
        (
            "
            class Test {
              constructor(private x: number) {}
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "no-public" }])),
        ),
        (
            "
            class Test {
              constructor(public x: number) {}
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "no-public", "overrides": { "parameterProperties": "off" }, }, ]),
            ),
        ),
        (
            "
            class Test {
              constructor(public foo: number) {}
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "no-public" }])),
        ),
        (
            "
            class Test {
              public getX() {
                return this.x;
              }
            }
                  ",
            Some(serde_json::json!([{ "ignoredMethodNames": ["getX"] }])),
        ),
        (
            "
            class Test {
              public static getX() {
                return this.x;
              }
            }
                  ",
            Some(serde_json::json!([{ "ignoredMethodNames": ["getX"] }])),
        ),
        (
            "
            class Test {
              get getX() {
                return this.x;
              }
            }
                  ",
            Some(serde_json::json!([{ "ignoredMethodNames": ["getX"] }])),
        ),
        (
            "
            class Test {
              getX() {
                return this.x;
              }
            }
                  ",
            Some(serde_json::json!([{ "ignoredMethodNames": ["getX"] }])),
        ),
        (
            "
            class Test {
              x = 2;
            }
                  ",
            Some(serde_json::json!([{ "overrides": { "properties": "off" } }])),
        ),
        (
            "
            class Test {
              private x = 2;
            }
                  ",
            Some(serde_json::json!([{ "overrides": { "properties": "explicit" } }])),
        ),
        (
            "
            class Test {
              x = 2;
              private x = 2;
            }
                  ",
            Some(serde_json::json!([{ "overrides": { "properties": "no-public" } }])),
        ),
        (
            "
            class Test {
              #foo = 1;
              #bar() {}
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "explicit" }])),
        ),
        (
            "
            class Test {
              private accessor foo = 1;
            }
                  ",
            None,
        ),
        (
            "
            abstract class Test {
              private abstract accessor foo: number;
            }
                  ",
            None,
        ),
    ];

    let fail = vec![
        (
            "
            export class XXXX {
              public constructor(readonly value: string) {}
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "off", "overrides": { "parameterProperties": "explicit", }, }, ]),
            ),
        ),
        (
            "
            export class WithParameterProperty {
              public constructor(readonly value: string) {}
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "explicit" }])),
        ),
        (
            "
            export class XXXX {
              public constructor(readonly samosa: string) {}
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "off", "overrides": { "constructors": "explicit", "parameterProperties": "explicit", }, }, ]),
            ),
        ),
        (
            "
            class Test {
              public constructor(readonly foo: string) {}
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "explicit", "overrides": { "parameterProperties": "explicit" }, }, ]),
            ),
        ),
        (
            "
            class Test {
              x: number;
              public getX() {
                return this.x;
              }
            }
                  ",
            None,
        ),
        (
            "
            class Test {
              private x: number;
              getX() {
                return this.x;
              }
            }
                  ",
            None,
        ),
        (
            "
            class Test {
              x?: number;
              getX?() {
                return this.x;
              }
            }
                  ",
            None,
        ),
        (
            "
            class Test {
              protected name: string;
              protected foo?: string;
              public getX() {
                return this.x;
              }
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "no-public" }])),
        ),
        (
            "
            class Test {
              protected name: string;
              public foo?: string;
              getX() {
                return this.x;
              }
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "no-public" }])),
        ),
        (
            "
            class Test {
              public x: number;
              public getX() {
                return this.x;
              }
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "no-public" }])),
        ),
        (
            "
            class Test {
              private x: number;
              constructor(x: number) {
                this.x = x;
              }
              get internalValue() {
                return this.x;
              }
              set internalValue(value: number) {
                this.x = value;
              }
            }
                  ",
            Some(serde_json::json!([{ "overrides": { "constructors": "no-public" } }])),
        ),
        (
            "
            class Test {
              private x: number;
              constructor(x: number) {
                this.x = x;
              }
              get internalValue() {
                return this.x;
              }
              set internalValue(value: number) {
                this.x = value;
              }
            }
                  ",
            None,
        ),
        (
            "
            class Test {
              constructor(public x: number) {}
              public foo(): string {
                return 'foo';
              }
            }
                  ",
            Some(serde_json::json!([ { "overrides": { "parameterProperties": "no-public" }, }, ])),
        ),
        (
            "
            class Test {
              constructor(public x: number) {}
            }
                  ",
            None,
        ),
        (
            "
            class Test {
              constructor(public readonly x: number) {}
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "off", "overrides": { "parameterProperties": "no-public" }, }, ]),
            ),
        ),
        (
            "
            class Test {
              x = 2;
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "off", "overrides": { "properties": "explicit" }, }, ]),
            ),
        ),
        (
            "
            class Test {
              public x = 2;
              private x = 2;
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "off", "overrides": { "properties": "no-public" }, }, ]),
            ),
        ),
        (
            "
            class Test {
              constructor(public x: any[]) {}
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "explicit" }])),
        ),
        (
            "
            class Test {
              public /*public*/constructor(private foo: string) {}
            }
                  ",
            Some(serde_json::json!([ { "accessibility": "no-public", }, ])),
        ),
        (
            "
            class Test {
              @public
              public foo() {}
            }
                  ",
            Some(serde_json::json!([ { "accessibility": "no-public", }, ])),
        ),
        (
            "
            class Test {
              @public
              public foo;
            }
                  ",
            Some(serde_json::json!([ { "accessibility": "no-public", }, ])),
        ),
        (
            "
            class Test {
              public foo = '';
            }
                  ",
            Some(serde_json::json!([ { "accessibility": "no-public", }, ])),
        ),
        (
            "
            class Test {
              constructor(public/* Hi there */ readonly foo) {}
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "no-public", "overrides": { "parameterProperties": "no-public" }, }, ]),
            ),
        ),
        (
            "
            class Test {
              constructor(public readonly foo: string) {}
            }
                  ",
            Some(serde_json::json!([ { "accessibility": "no-public", }, ])),
        ),
        (
            "
            class EnsureWhiteSPaceSpan {
              public constructor() {}
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "no-public", "overrides": { "parameterProperties": "no-public" }, }, ]),
            ),
        ),
        (
            "
            class EnsureWhiteSPaceSpan {
              public /* */ constructor() {}
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "no-public", "overrides": { "parameterProperties": "no-public" }, }, ]),
            ),
        ),
        (
            "
            class Test {
              public 'foo' = 1;
              public 'foo foo' = 2;
              public 'bar'() {}
              public 'bar bar'() {}
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "no-public" }])),
        ),
        (
            "
            abstract class SomeClass {
              abstract method(): string;
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "explicit" }])),
        ),
        (
            "
            abstract class SomeClass {
              public abstract method(): string;
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "no-public", "overrides": { "parameterProperties": "no-public" }, }, ]),
            ),
        ),
        (
            "
            abstract class SomeClass {
              abstract x: string;
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "explicit" }])),
        ),
        (
            "
            abstract class SomeClass {
              public abstract x: string;
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "no-public", "overrides": { "parameterProperties": "no-public" }, }, ]),
            ),
        ),
        (
            "
            class SomeClass {
              accessor foo = 1;
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "explicit" }])),
        ),
        (
            "
            abstract class SomeClass {
              abstract accessor foo: string;
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "explicit" }])),
        ),
        (
            "
            class DecoratedClass {
              constructor(@foo @bar() readonly arg: string) {}
              @foo @bar() x: string;
              @foo @bar() getX() {
                return this.x;
              }
              @foo
              @bar()
              get y() {
                return this.x;
              }
              @foo @bar() set z(@foo @bar() value: x) {
                this.x = x;
              }
            }
                  ",
            None,
        ),
        (
            "
            abstract class SomeClass {
              abstract ['computed-method-name'](): string;
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "explicit" }])),
        ),
    ];

    let fix = vec![
        (
            "
            class Test {
              protected name: string;
              protected foo?: string;
              public getX() {
                return this.x;
              }
            }
                  ",
            "
            class Test {
              protected name: string;
              protected foo?: string;
              getX() {
                return this.x;
              }
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "no-public" }])),
        ),
        (
            "
            class Test {
              protected name: string;
              public foo?: string;
              getX() {
                return this.x;
              }
            }
                  ",
            "
            class Test {
              protected name: string;
              foo?: string;
              getX() {
                return this.x;
              }
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "no-public" }])),
        ),
        (
            "
            class Test {
              public x: number;
              public getX() {
                return this.x;
              }
            }
                  ",
            "
            class Test {
              x: number;
              getX() {
                return this.x;
              }
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "no-public" }])),
        ),
        (
            "
            class Test {
              constructor(public readonly x: number) {}
            }
                  ",
            "
            class Test {
              constructor(readonly x: number) {}
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "off", "overrides": { "parameterProperties": "no-public" }, }, ]),
            ),
        ),
        (
            "
            class Test {
              public x = 2;
              private x = 2;
            }
                  ",
            "
            class Test {
              x = 2;
              private x = 2;
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "off", "overrides": { "properties": "no-public" }, }, ]),
            ),
        ),
        (
            "
            class Test {
              public /*public*/constructor(private foo: string) {}
            }
                  ",
            "
            class Test {
              /*public*/constructor(private foo: string) {}
            }
                  ",
            Some(serde_json::json!([ { "accessibility": "no-public", }, ])),
        ),
        (
            "
            class Test {
              @public
              public foo() {}
            }
                  ",
            "
            class Test {
              @public
              foo() {}
            }
                  ",
            Some(serde_json::json!([ { "accessibility": "no-public", }, ])),
        ),
        (
            "
            class Test {
              @public
              public foo;
            }
                  ",
            "
            class Test {
              @public
              foo;
            }
                  ",
            Some(serde_json::json!([ { "accessibility": "no-public", }, ])),
        ),
        (
            "
            class Test {
              public foo = '';
            }
                  ",
            "
            class Test {
              foo = '';
            }
                  ",
            Some(serde_json::json!([ { "accessibility": "no-public", }, ])),
        ),
        (
            "
            class Test {
              constructor(public/* Hi there */ readonly foo) {}
            }
                  ",
            "
            class Test {
              constructor(/* Hi there */ readonly foo) {}
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "no-public", "overrides": { "parameterProperties": "no-public" }, }, ]),
            ),
        ),
        (
            "
            class Test {
              constructor(public readonly foo: string) {}
            }
                  ",
            "
            class Test {
              constructor(readonly foo: string) {}
            }
                  ",
            Some(serde_json::json!([ { "accessibility": "no-public", }, ])),
        ),
        (
            "
            class EnsureWhiteSPaceSpan {
              public constructor() {}
            }
                  ",
            "
            class EnsureWhiteSPaceSpan {
              constructor() {}
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "no-public", "overrides": { "parameterProperties": "no-public" }, }, ]),
            ),
        ),
        (
            "
            class EnsureWhiteSPaceSpan {
              public /* */ constructor() {}
            }
                  ",
            "
            class EnsureWhiteSPaceSpan {
              /* */ constructor() {}
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "no-public", "overrides": { "parameterProperties": "no-public" }, }, ]),
            ),
        ),
        (
            "
            class Test {
              public 'foo' = 1;
              public 'foo foo' = 2;
              public 'bar'() {}
              public 'bar bar'() {}
            }
                  ",
            "
            class Test {
              'foo' = 1;
              'foo foo' = 2;
              'bar'() {}
              'bar bar'() {}
            }
                  ",
            Some(serde_json::json!([{ "accessibility": "no-public" }])),
        ),
        (
            "
            abstract class SomeClass {
              public abstract method(): string;
            }
                  ",
            "
            abstract class SomeClass {
              abstract method(): string;
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "no-public", "overrides": { "parameterProperties": "no-public" }, }, ]),
            ),
        ),
        (
            "
            abstract class SomeClass {
              public abstract x: string;
            }
                  ",
            "
            abstract class SomeClass {
              abstract x: string;
            }
                  ",
            Some(
                serde_json::json!([ { "accessibility": "no-public", "overrides": { "parameterProperties": "no-public" }, }, ]),
            ),
        ),
        (
            "
            class Test {
              @dec /*public*/ public foo() {}
            }
                  ",
            "
            class Test {
              @dec /*public*/ foo() {}
            }
                  ",
            Some(serde_json::json!([ { "accessibility": "no-public", }, ])),
        ),
    ];

    Tester::new(ExplicitMemberAccessibility::NAME, ExplicitMemberAccessibility::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
