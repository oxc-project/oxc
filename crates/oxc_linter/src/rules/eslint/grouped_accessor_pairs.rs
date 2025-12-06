use std::borrow::Cow;

use oxc_allocator::Box as OBox;
use oxc_ast::{
    AstKind,
    ast::{
        ClassElement, Expression, MethodDefinition, MethodDefinitionKind, ObjectProperty,
        ObjectPropertyKind, PropertyKey, PropertyKind, TSInterfaceBody, TSMethodSignature,
        TSMethodSignatureKind, TSSignature, TSTypeLiteral,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn grouped_accessor_pairs_diagnostic(
    getter_span: Span,
    getter_key: &str,
    setter_span: Span,
    setter_key: &str,
    msg: String,
) -> OxcDiagnostic {
    let getter_label_span = getter_span.label(format!("{getter_key} is here"));
    let setter_label_span = setter_span.label(format!("{setter_key} is here"));
    OxcDiagnostic::warn(msg)
        .with_help("Require grouped accessor pairs in object literals and classes")
        .with_labels([getter_label_span, setter_label_span])
}

#[derive(Debug, Default, PartialEq, Clone, Copy, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum PairOrder {
    /// Accessors can be in any order. This is the default.
    #[default]
    AnyOrder,
    /// Getters must come before setters.
    GetBeforeSet,
    /// Setters must come before getters.
    SetBeforeGet,
}

#[derive(Debug, Default, Clone, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GroupedAccessorPairs(PairOrder, GroupedAccessorPairsConfig);

#[derive(Debug, Default, Clone, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GroupedAccessorPairsConfig {
    /// When `enforceForTSTypes` is enabled, this rule also applies to TypeScript interfaces
    /// and type aliases.
    ///
    /// Examples of **incorrect** TypeScript code:
    /// ```ts
    /// interface Foo {
    ///     get a(): string;
    ///     someProperty: string;
    ///     set a(value: string);
    /// }
    ///
    /// type Bar = {
    ///     get b(): string;
    ///     someProperty: string;
    ///     set b(value: string);
    /// };
    /// ```
    ///
    /// Examples of **correct** TypeScript code:
    /// ```ts
    /// interface Foo {
    ///     get a(): string;
    ///     set a(value: string);
    ///     someProperty: string;
    /// }
    ///
    /// type Bar = {
    ///     get b(): string;
    ///     set b(value: string);
    ///     someProperty: string;
    /// };
    /// ```
    #[serde(rename = "enforceForTSTypes")]
    enforce_for_ts_types: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require grouped accessor pairs in object literals and classes
    ///
    /// ### Why is this bad?
    ///
    /// While it is allowed to define the pair for a getter or a setter anywhere in an object or class definition,
    /// it’s considered a best practice to group accessor functions for the same property.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const foo = {
    ///     get a() {
    ///         return this.val;
    ///     },
    ///     b: 1,
    ///     set a(value) {
    ///         this.val = value;
    ///     }
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const foo = {
    ///     get a() {
    ///         return this.val;
    ///     },
    ///     set a(value) {
    ///         this.val = value;
    ///     },
    ///     b: 1
    /// };
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the `getBeforeSet` option:
    /// ```js
    /// const foo = {
    ///     set a(value) {
    ///         this.val = value;
    ///     },
    ///     get a() {
    ///         return this.val;
    ///     }
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `getBeforeSet` option:
    /// ```js
    /// const foo = {
    ///     get a() {
    ///         return this.val;
    ///     },
    ///     set a(value) {
    ///         this.val = value;
    ///     }
    /// };
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the `setBeforeGet` option:
    /// ```js
    /// const foo = {
    ///     get a() {
    ///         return this.val;
    ///     },
    ///     set a(value) {
    ///         this.val = value;
    ///     }
    /// };
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `setBeforeGet` option:
    /// ```js
    /// const foo = {
    ///     set a(value) {
    ///         this.val = value;
    ///     },
    ///     get a() {
    ///         return this.val;
    ///     }
    /// };
    /// ```
    GroupedAccessorPairs,
    eslint,
    style,
    pending,
    config = GroupedAccessorPairs,
);

impl Rule for GroupedAccessorPairs {
    fn from_configuration(value: Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<GroupedAccessorPairs>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let GroupedAccessorPairs(pair_order, config) = &self;
        let enforce_for_ts_types = config.enforce_for_ts_types;

        match node.kind() {
            AstKind::ObjectExpression(obj_expr) => {
                let mut prop_map =
                    FxHashMap::<(String, bool), Vec<(usize, &OBox<ObjectProperty>)>>::default();
                let properties = &obj_expr.properties;

                for (idx, v) in properties.iter().enumerate() {
                    let ObjectPropertyKind::ObjectProperty(obj_prop) = v else {
                        continue;
                    };
                    if obj_prop.kind == PropertyKind::Init {
                        continue;
                    }
                    // get the Key name of the Object and verify that it is Literal
                    // we need to consider the following two cases:
                    // 1) var foo = { get a() {}, set [a](v) {} }
                    // in the above example, both a and [a] should not be treated as a pair,
                    // although call get_key_name_and_check_literal method gets the key name 'a'
                    // because the set access property is computed and its key is not Literal
                    // 2) var foo = { get a() {}, set ['a'](v) {} }
                    // in this example, a and ['a'] should be treated as a pair
                    // although the set access property is computed, but its key is Literal
                    let (key_name, is_literal) = get_key_name_and_check_literal(ctx, &obj_prop.key);
                    let is_computed = if is_literal { false } else { obj_prop.computed };
                    prop_map.entry((key_name, is_computed)).or_default().push((idx, obj_prop));
                }

                for ((key, is_computed), val) in prop_map {
                    if val.len() == 2 {
                        let (first_idx, first_node) = val[0];
                        let (second_idx, second_node) = val[1];
                        if first_node.kind == second_node.kind {
                            continue;
                        }
                        let (getter_idx, getter_node, setter_idx, setter_node) =
                            if first_node.kind == PropertyKind::Get {
                                (first_idx, first_node, second_idx, second_node)
                            } else {
                                (second_idx, second_node, first_idx, first_node)
                            };
                        let getter_key =
                            get_diagnostic_access_name("getter", &key, is_computed, false, false);
                        let setter_key =
                            get_diagnostic_access_name("setter", &key, is_computed, false, false);
                        report(
                            ctx,
                            *pair_order,
                            (&getter_key, &setter_key),
                            (
                                Span::new(getter_node.span.start, getter_node.key.span().end),
                                Span::new(setter_node.span.start, setter_node.key.span().end),
                            ),
                            (getter_idx, setter_idx),
                        );
                    }
                }
            }
            AstKind::ClassBody(class_body) => {
                let method_defines = &class_body.body;
                let mut prop_map = FxHashMap::<
                    (String, bool, bool, bool),
                    Vec<(usize, &OBox<MethodDefinition>)>,
                >::default();

                for (idx, v) in method_defines.iter().enumerate() {
                    let ClassElement::MethodDefinition(method_define) = v else {
                        continue;
                    };
                    if !matches!(
                        method_define.kind,
                        MethodDefinitionKind::Get | MethodDefinitionKind::Set
                    ) {
                        continue;
                    }
                    let (key_name, is_literal) =
                        get_key_name_and_check_literal(ctx, &method_define.key);
                    let is_computed = if is_literal { false } else { method_define.computed };
                    let is_private = matches!(method_define.key, PropertyKey::PrivateIdentifier(_));
                    // for Class we need to focus on whether the key is static or private
                    // 1) class foo { static set [a+b](val){} static get [a+b](){} }
                    // 2) class foo { set #abc(val){} static get #abc(){} }
                    prop_map
                        .entry((key_name, is_computed, method_define.r#static, is_private))
                        .or_default()
                        .push((idx, method_define));
                }

                for ((key, is_computed, is_static, is_private), val) in prop_map {
                    if val.len() == 2 {
                        let (first_idx, first_node) = val[0];
                        let (second_idx, second_node) = val[1];
                        if first_node.kind == second_node.kind {
                            continue;
                        }
                        let (getter_idx, getter_node, setter_idx, setter_node) =
                            if first_node.kind == MethodDefinitionKind::Get {
                                (first_idx, first_node, second_idx, second_node)
                            } else {
                                (second_idx, second_node, first_idx, first_node)
                            };
                        let getter_key = get_diagnostic_access_name(
                            "getter",
                            &key,
                            is_computed,
                            is_static,
                            is_private,
                        );
                        let setter_key = get_diagnostic_access_name(
                            "setter",
                            &key,
                            is_computed,
                            is_static,
                            is_private,
                        );
                        report(
                            ctx,
                            *pair_order,
                            (&getter_key, &setter_key),
                            (
                                Span::new(getter_node.span.start, getter_node.key.span().end),
                                Span::new(setter_node.span.start, setter_node.key.span().end),
                            ),
                            (getter_idx, setter_idx),
                        );
                    }
                }
            }
            AstKind::TSInterfaceBody(interface_body) if enforce_for_ts_types => {
                self.check_ts_interface_body(interface_body, ctx);
            }
            AstKind::TSTypeLiteral(type_literal) if enforce_for_ts_types => {
                self.check_ts_type_literal(type_literal, ctx);
            }
            _ => {}
        }
    }
}

impl GroupedAccessorPairs {
    fn check_ts_interface_body<'a>(
        &self,
        interface_body: &TSInterfaceBody<'a>,
        ctx: &LintContext<'a>,
    ) {
        self.check_ts_signatures(&interface_body.body, ctx);
    }

    fn check_ts_type_literal<'a>(&self, type_literal: &TSTypeLiteral<'a>, ctx: &LintContext<'a>) {
        self.check_ts_signatures(&type_literal.members, ctx);
    }

    fn check_ts_signatures<'a>(&self, signatures: &[TSSignature<'a>], ctx: &LintContext<'a>) {
        let GroupedAccessorPairs(pair_order, _config) = &self;

        let mut prop_map =
            FxHashMap::<(String, bool), Vec<(usize, &OBox<TSMethodSignature>)>>::default();

        for (idx, signature) in signatures.iter().enumerate() {
            let TSSignature::TSMethodSignature(method_sig) = signature else {
                continue;
            };
            if !matches!(method_sig.kind, TSMethodSignatureKind::Get | TSMethodSignatureKind::Set) {
                continue;
            }
            let (key_name, is_literal) = get_key_name_and_check_literal(ctx, &method_sig.key);
            let is_computed = if is_literal { false } else { method_sig.computed };
            prop_map.entry((key_name, is_computed)).or_default().push((idx, method_sig));
        }

        for ((key, is_computed), val) in prop_map {
            if val.len() == 2 {
                let (first_idx, first_node) = val[0];
                let (second_idx, second_node) = val[1];
                if first_node.kind == second_node.kind {
                    continue;
                }
                let (getter_idx, getter_node, setter_idx, setter_node) =
                    if first_node.kind == TSMethodSignatureKind::Get {
                        (first_idx, first_node, second_idx, second_node)
                    } else {
                        (second_idx, second_node, first_idx, first_node)
                    };
                let getter_key =
                    get_diagnostic_access_name("getter", &key, is_computed, false, false);
                let setter_key =
                    get_diagnostic_access_name("setter", &key, is_computed, false, false);
                report(
                    ctx,
                    *pair_order,
                    (&getter_key, &setter_key),
                    (
                        Span::new(getter_node.span.start, getter_node.key.span().end),
                        Span::new(setter_node.span.start, setter_node.key.span().end),
                    ),
                    (getter_idx, setter_idx),
                );
            }
        }
    }
}

fn get_key_name_and_check_literal<'a>(
    ctx: &LintContext<'a>,
    prop_key: &PropertyKey<'a>,
) -> (String, bool) {
    let key_name = prop_key
        .name()
        .unwrap_or_else(|| {
            Cow::Borrowed(prop_key.as_expression().unwrap().span().source_text(ctx.source_text()))
        })
        .to_string();
    let is_literal =
        if matches!(prop_key, PropertyKey::StaticIdentifier(_) | PropertyKey::PrivateIdentifier(_))
        {
            false
        } else {
            matches!(
                prop_key.as_expression().unwrap(),
                Expression::BooleanLiteral(_)
                    | Expression::NullLiteral(_)
                    | Expression::StringLiteral(_)
                    | Expression::RegExpLiteral(_)
                    | Expression::BigIntLiteral(_)
                    | Expression::NumericLiteral(_)
                    | Expression::TemplateLiteral(_)
            )
        };
    (key_name, is_literal)
}

fn get_diagnostic_access_name(
    access_word: &str,
    base_key: &str,
    is_computed: bool,
    is_static: bool,
    is_private: bool,
) -> String {
    let static_prefix = if is_static { "static " } else { "" };
    if is_computed {
        format!("{static_prefix}{access_word}")
    } else {
        // e.g. "class foo { get #a {} set #a(val) {} }" we should get #a
        let (real_key, private_prefix) = if is_private {
            (format!("#{base_key}"), "private ")
        } else {
            (format!("'{base_key}'"), "")
        };
        format!("{static_prefix}{private_prefix}{access_word} {real_key}")
    }
}

fn report(
    ctx: &LintContext,
    pair_order: PairOrder,
    (getter_key, setter_key): (&str, &str),
    (getter_span, setter_span): (Span, Span),
    (getter_idx, setter_idx): (usize, usize),
) {
    if getter_idx.abs_diff(setter_idx) > 1 {
        ctx.diagnostic(grouped_accessor_pairs_diagnostic(
            getter_span,
            getter_key,
            setter_span,
            setter_key,
            format!("Accessor pair {getter_key} and {setter_key} should be grouped."),
        ));
    }
    match pair_order {
        PairOrder::GetBeforeSet if getter_idx > setter_idx => {
            ctx.diagnostic(grouped_accessor_pairs_diagnostic(
                getter_span,
                getter_key,
                setter_span,
                setter_key,
                format!("Expected {getter_key} to be before {setter_key}."),
            ));
        }
        PairOrder::SetBeforeGet if setter_idx > getter_idx => {
            ctx.diagnostic(grouped_accessor_pairs_diagnostic(
                getter_span,
                getter_key,
                setter_span,
                setter_key,
                format!("Expected {setter_key} to be before {getter_key}."),
            ));
        }
        _ => {}
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("({})", None),
        ("({ a })", None),
        ("({ a(){}, b(){}, a(){} })", None),
        ("({ a: 1, b: 2 })", None),
        ("({ a, ...b, c: 1 })", None),
        ("({ a, b, ...a })", None),
        ("({ a: 1, [b]: 2, a: 3, [b]: 4 })", None),
        ("({ a: function get(){}, b, a: function set(foo){} })", None),
        ("({ get(){}, a, set(){} })", None),
        ("class A {}", None),
        ("(class { a(){} })", None),
        ("class A { a(){} [b](){} a(){} [b](){} }", None),
        ("(class { a(){} b(){} static a(){} static b(){} })", None),
        ("class A { get(){} a(){} set(){} }", None),
        ("({ get a(){} })", None),
        ("({ set a(foo){} })", None),
        ("({ a: 1, get b(){}, c, ...d })", None),
        ("({ get a(){}, get b(){}, set c(foo){}, set d(foo){} })", None),
        ("({ get a(){}, b: 1, set c(foo){} })", None),
        ("({ set a(foo){}, b: 1, a: 2 })", None),
        ("({ get a(){}, b: 1, a })", None),
        ("({ set a(foo){}, b: 1, a(){} })", None),
        ("({ get a(){}, b: 1, set [a](foo){} })", None),
        ("({ set a(foo){}, b: 1, get 'a '(){} })", None),
        ("({ get a(){}, b: 1, ...a })", None),
        ("({ set a(foo){}, b: 1 }, { get a(){} })", None),
        ("({ get a(){}, b: 1, ...{ set a(foo){} } })", None),
        ("({ set a(foo){}, get b(){} })", Some(serde_json::json!(["getBeforeSet"]))),
        ("({ get a(){}, set b(foo){} })", Some(serde_json::json!(["setBeforeGet"]))),
        ("class A { get a(){} }", None),
        ("(class { set a(foo){} })", None),
        ("class A { static set a(foo){} }", None),
        ("(class { static get a(){} })", None),
        ("class A { a(){} set b(foo){} c(){} }", None),
        ("(class { a(){} get b(){} c(){} })", None),
        ("class A { get a(){} static get b(){} set c(foo){} static set d(bar){} }", None),
        ("(class { get a(){} b(){} a(foo){} })", None),
        ("class A { static set a(foo){} b(){} static a(){} }", None),
        ("(class { get a(){} static b(){} set [a](foo){} })", None),
        ("class A { static set a(foo){} b(){} static get ' a'(){} }", None),
        ("(class { set a(foo){} b(){} static get a(){} })", None),
        ("class A { static set a(foo){} b(){} get a(){} }", None),
        ("(class { get a(){} }, class { b(){} set a(foo){} })", None),
        ("({ get a(){}, set a(foo){} })", None),
        ("({ a: 1, set b(foo){}, get b(){}, c: 2 })", None),
        ("({ get a(){}, set a(foo){}, set b(bar){}, get b(){} })", None),
        ("({ get [a](){}, set [a](foo){} })", None),
        ("({ set a(foo){}, get 'a'(){} })", None),
        ("({ a: 1, b: 2, get a(){}, set a(foo){}, c: 3, a: 4 })", None),
        ("({ get a(){}, set a(foo){}, set b(bar){} })", None),
        ("({ get a(){}, get b(){}, set b(bar){} })", None),
        ("class A { get a(){} set a(foo){} }", None),
        ("(class { set a(foo){} get a(){} })", None),
        ("class A { static set a(foo){} static get a(){} }", None),
        ("(class { static get a(){} static set a(foo){} })", None),
        ("class A { a(){} set b(foo){} get b(){} c(){} get d(){} set d(bar){} }", None),
        ("(class { set a(foo){} get a(){} get b(){} set b(bar){} })", None),
        ("class A { static set [a](foo){} static get [a](){} }", None),
        ("(class { get a(){} set [`a`](foo){} })", None),
        ("class A { static get a(){} static set a(foo){} set a(bar){} static get a(){} }", None),
        ("(class { static get a(){} get a(){} set a(foo){} })", None),
        ("({ get a(){}, set a(foo){} })", Some(serde_json::json!(["anyOrder"]))),
        ("({ set a(foo){}, get a(){} })", Some(serde_json::json!(["anyOrder"]))),
        ("({ get a(){}, set a(foo){} })", Some(serde_json::json!(["getBeforeSet"]))),
        ("({ set a(foo){}, get a(){} })", Some(serde_json::json!(["setBeforeGet"]))),
        ("class A { get a(){} set a(foo){} }", Some(serde_json::json!(["anyOrder"]))),
        ("(class { set a(foo){} get a(){} })", Some(serde_json::json!(["anyOrder"]))),
        ("class A { get a(){} set a(foo){} }", Some(serde_json::json!(["getBeforeSet"]))),
        (
            "(class { static set a(foo){} static get a(){} })",
            Some(serde_json::json!(["setBeforeGet"])),
        ),
        ("({ get a(){}, b: 1, get a(){} })", None),
        ("({ set a(foo){}, b: 1, set a(foo){} })", None),
        ("({ get a(){}, b: 1, set a(foo){}, c: 2, get a(){} })", None),
        ("({ set a(foo){}, b: 1, set 'a'(bar){}, c: 2, get a(){} })", None),
        ("class A { get [a](){} b(){} get [a](){} c(){} set [a](foo){} }", None),
        (
            "(class { static set a(foo){} b(){} static get a(){} static c(){} static set a(bar){} })",
            None,
        ),
        ("class A { get '#abc'(){} b(){} set #abc(foo){} }", None),
        ("class A { get #abc(){} b(){} set '#abc'(foo){} }", None),
        ("class A { set '#abc'(foo){} get #abc(){} }", Some(serde_json::json!(["getBeforeSet"]))),
        ("class A { set #abc(foo){} get '#abc'(){} }", Some(serde_json::json!(["getBeforeSet"]))),
        ("class faoo { set abc(val){} get #abc(){} }", Some(serde_json::json!(["getBeforeSet"]))),
        (
            "class foo {
            static set ['#a+b'](val) {

            }
            static get ['#a+b']() {

            }
            static set ['#a+b'](val) {

            }
        }",
            Some(serde_json::json!(["getBeforeSet"])),
        ),
        (
            "class foo {
            set [() => {}](val) {

            }
            get ['() => {}']() {

            }
        }",
            Some(serde_json::json!(["getBeforeSet"])),
        ),
        (
            "class foo {
            get aa() {

            }
            get aa() {}
            set aa(val) {

            }
        }",
            Some(serde_json::json!(["setBeforeGet"])),
        ),
        (
            "interface I { set prop(value: any), get prop(): any }",
            Some(serde_json::json!(["anyOrder", { "enforceForTSTypes": true }])),
        ),
        (
            "interface I { get a(): any, between: true, set b(value: any) }",
            Some(serde_json::json!(["anyOrder", { "enforceForTSTypes": true }])),
        ),
        (
            "interface I { before: true, get prop(): any, set prop(value: any), after: true }",
            Some(serde_json::json!(["getBeforeSet", { "enforceForTSTypes": true }])),
        ),
        (
            "interface I { set prop(value: any), get prop(): any }",
            Some(serde_json::json!(["setBeforeGet", { "enforceForTSTypes": true }])),
        ),
        (
            "type T = { get prop(): any, set prop(value: any) }",
            Some(serde_json::json!(["anyOrder", { "enforceForTSTypes": true }])),
        ),
        (
            "type T = { set prop(value: any), get prop(): any }",
            Some(serde_json::json!(["setBeforeGet", { "enforceForTSTypes": true }])),
        ),
    ];

    let fail = vec![
        ("({ get a(){}, b:1, set a(foo){} })", None),
        ("({ set 'abc'(foo){}, b:1, get 'abc'(){} })", None),
        ("({ get [a](){}, b:1, set [a](foo){} })", None),
        ("class A { get abc(){} b(){} set abc(foo){} }", None),
        ("(class { set abc(foo){} b(){} get abc(){} })", None),
        ("class A { static set a(foo){} b(){} static get a(){} }", None),
        ("(class { static get 123(){} b(){} static set 123(foo){} })", None),
        ("class A { static get [a](){} b(){} static set [a](foo){} }", None),
        ("class A { get '#abc'(){} b(){} set '#abc'(foo){} }", None),
        ("class A { get #abc(){} b(){} set #abc(foo){} }", None),
        ("({ set a(foo){}, get a(){} })", Some(serde_json::json!(["getBeforeSet"]))),
        ("({ get 123(){}, set 123(foo){} })", Some(serde_json::json!(["setBeforeGet"]))),
        ("({ get [a](){}, set [a](foo){} })", Some(serde_json::json!(["setBeforeGet"]))),
        ("class A { set abc(foo){} get abc(){} }", Some(serde_json::json!(["getBeforeSet"]))),
        (
            "(class { get [`abc`](){} set [`abc`](foo){} })",
            Some(serde_json::json!(["setBeforeGet"])),
        ),
        (
            "class A { static get a(){} static set a(foo){} }",
            Some(serde_json::json!(["setBeforeGet"])),
        ),
        (
            "(class { static set 'abc'(foo){} static get 'abc'(){} })",
            Some(serde_json::json!(["getBeforeSet"])),
        ),
        (
            "class A { static set [abc](foo){} static get [abc](){} }",
            Some(serde_json::json!(["getBeforeSet"])),
        ),
        ("class A { set '#abc'(foo){} get '#abc'(){} }", Some(serde_json::json!(["getBeforeSet"]))),
        ("class A { set #abc(foo){} get #abc(){} }", Some(serde_json::json!(["getBeforeSet"]))),
        ("({ get a(){}, b: 1, set a(foo){} })", Some(serde_json::json!(["anyOrder"]))),
        ("({ get a(){}, b: 1, set a(foo){} })", Some(serde_json::json!(["setBeforeGet"]))),
        ("({ get a(){}, b: 1, set a(foo){} })", Some(serde_json::json!(["getBeforeSet"]))),
        ("class A { set a(foo){} b(){} get a(){} }", Some(serde_json::json!(["getBeforeSet"]))),
        (
            "(class { static set a(foo){} b(){} static get a(){} })",
            Some(serde_json::json!(["setBeforeGet"])),
        ),
        ("({ get 'abc'(){}, d(){}, set 'abc'(foo){} })", None),
        ("({ set ''(foo){}, get [''](){} })", Some(serde_json::json!(["getBeforeSet"]))),
        ("class A { set abc(foo){} get 'abc'(){} }", Some(serde_json::json!(["getBeforeSet"]))),
        ("(class { set [`abc`](foo){} get abc(){} })", Some(serde_json::json!(["getBeforeSet"]))),
        ("({ set ['abc'](foo){}, get [`abc`](){} })", Some(serde_json::json!(["getBeforeSet"]))),
        ("({ set 123(foo){}, get [123](){} })", Some(serde_json::json!(["getBeforeSet"]))),
        (
            "class A { static set '123'(foo){} static get 123(){} }",
            Some(serde_json::json!(["getBeforeSet"])),
        ),
        ("(class { set [a+b](foo){} get [a+b](){} })", Some(serde_json::json!(["getBeforeSet"]))),
        ("({ set [f(a)](foo){}, get [f(a)](){} })", Some(serde_json::json!(["getBeforeSet"]))),
        ("({ get a(){}, b: 1, set a(foo){}, set c(foo){}, d(){}, get c(){} })", None),
        ("({ get a(){}, set b(foo){}, set a(bar){}, get b(){} })", None),
        ("({ get a(){}, set [a](foo){}, set a(bar){}, get [a](){} })", None),
        (
            "({ a(){}, set b(foo){}, ...c, get b(){}, set c(bar){}, get c(){} })",
            Some(serde_json::json!(["getBeforeSet"])),
        ),
        (
            "({ set [a](foo){}, get [a](){}, set [-a](bar){}, get [-a](){} })",
            Some(serde_json::json!(["getBeforeSet"])),
        ),
        (
            "class A { get a(){} constructor (){} set a(foo){} get b(){} static c(){} set b(bar){} }",
            None,
        ),
        ("(class { set a(foo){} static get a(){} get a(){} static set a(bar){} })", None),
        (
            "class A { get a(){} set a(foo){} static get b(){} static set b(bar){} }",
            Some(serde_json::json!(["setBeforeGet"])),
        ),
        ("(class { set [a+b](foo){} get [a-b](){} get [a+b](){} set [a-b](bar){} })", None),
        ("({ get a(){}, set a(foo){}, get b(){}, c: function(){}, set b(bar){} })", None),
        ("({ get a(){}, get b(){}, set a(foo){} })", None),
        ("({ set a(foo){}, get [a](){}, get a(){} })", None),
        ("({ set [a](foo){}, set a(bar){}, get [a](){} })", None),
        (
            "({ get a(){}, set a(foo){}, set b(bar){}, get b(){} })",
            Some(serde_json::json!(["getBeforeSet"])),
        ),
        ("class A { get a(){} static set b(foo){} static get b(){} set a(foo){} }", None),
        ("(class { static get a(){} set a(foo){} static set a(bar){} })", None),
        (
            "class A { set a(foo){} get a(){} static get a(){} static set a(bar){} }",
            Some(serde_json::json!(["setBeforeGet"])),
        ),
        ("({ get a(){}, a: 1, set a(foo){} })", None),
        ("({ a(){}, set a(foo){}, get a(){} })", Some(serde_json::json!(["getBeforeSet"]))),
        ("class A { get a(){} a(){} set a(foo){} }", None),
        ("class A { get a(){} a; set a(foo){} }", None), // { "ecmaVersion": 2022 },
        (
            "class faoo { static set #abc(foo){} static get #abc(){} }",
            Some(serde_json::json!(["getBeforeSet"])),
        ),
        (
            "({ get a(){},
			    b: 1,
			    set a(foo){}
			})",
            None,
        ),
        (
            "class A { static set a(foo){} b(){} static get
			 a(){}
			}",
            None,
        ),
        (
            "const foo = {
            set [false](value) {
                this.val = value;
            },
            get 'false'() {
                return this.val;
            },
        }",
            Some(serde_json::json!(["getBeforeSet"])),
        ),
        (
            "const foo = {
            get '/a/g'() {
                return this.val;
            },
            set [/a/g](value) {
                this.val = value;
            },
        };",
            Some(serde_json::json!(["setBeforeGet"])),
        ),
        (
            "class foo { static set #abc(foo){} static get #abc(){} }",
            Some(serde_json::json!(["getBeforeSet"])),
        ),
        (
            "class foo {
            static set ['#a+b'](val) {

            }
            static get ['#a+b']() {

            }
        }",
            Some(serde_json::json!(["getBeforeSet"])),
        ),
        (
            "class foo {
            set [() => {}](val) {

            }
            get [() => {}]() {

            }
        }",
            Some(serde_json::json!(["getBeforeSet"])),
        ),
        (
            "class foo {
            static set [23](val) {

            }
            static get 23() {

            }
        }",
            Some(serde_json::json!(["getBeforeSet"])),
        ),
        (
            "class jj {
            static set [23](val) {

            }
            static get 23() {

            }
            set 23(val) {

            }
            get 23() {

            }
        }",
            Some(serde_json::json!(["getBeforeSet"])),
        ),
        (
            "interface I { get a(): any, set a(value: any) }",
            Some(serde_json::json!(["setBeforeGet", { "enforceForTSTypes": true }])),
        ),
        (
            "interface I { set a(value: any), get a(): any }",
            Some(serde_json::json!(["getBeforeSet", { "enforceForTSTypes": true }])),
        ),
        (
            "type T = { get a(): any, between: true, set a(value: any) }",
            Some(serde_json::json!(["anyOrder", { "enforceForTSTypes": true }])),
        ),
        (
            "type T = { get a(): any, set a(value: any) }",
            Some(serde_json::json!(["setBeforeGet", { "enforceForTSTypes": true }])),
        ),
    ];

    Tester::new(GroupedAccessorPairs::NAME, GroupedAccessorPairs::PLUGIN, pass, fail)
        .test_and_snapshot();
}
