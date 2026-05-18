use itertools::Either;
use lazy_regex::{Lazy, Regex, lazy_regex};
use schemars::JsonSchema;

use oxc_ast::{
    AstKind,
    ast::{
        ArrowFunctionExpression, Expression, Function, ObjectExpression, ObjectProperty,
        ObjectPropertyKind, PropertyKind,
    },
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{Rule, TupleRuleConfig},
    utils::deserialize_regex_option,
};

fn expected_all_properties_shorthanded(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected shorthand for all properties.").with_label(span)
}

fn expected_literal_method_longform(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected longform method syntax for string literal keys.").with_label(span)
}

fn expected_property_shorthand(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected property shorthand.").with_label(span)
}

fn expected_property_longform(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected longform property syntax.").with_label(span)
}

fn expected_method_shorthand(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected method shorthand.").with_label(span)
}

fn expected_method_longform(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected longform method syntax.").with_label(span)
}

fn unexpected_mix(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected mix of shorthand and non-shorthand properties.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ObjectShorthand(Box<ObjectShorthandConfig>);

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(default)]
pub struct ObjectShorthandTupleConfig(ShorthandType, ObjectShorthandOptions);

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default, deny_unknown_fields)]
pub struct ObjectShorthandOptions {
    avoid_quotes: bool,
    ignore_constructors: bool,
    avoid_explicit_return_arrows: bool,
    #[serde(default, deserialize_with = "deserialize_regex_option")]
    methods_ignore_pattern: Option<Regex>,
}

impl From<ObjectShorthandTupleConfig> for ObjectShorthandConfig {
    fn from(value: ObjectShorthandTupleConfig) -> Self {
        let ObjectShorthandTupleConfig(shorthand_type, options) = value;

        ObjectShorthandConfig {
            apply_to_methods: matches!(
                shorthand_type,
                ShorthandType::Methods | ShorthandType::Always
            ),
            apply_to_properties: matches!(
                shorthand_type,
                ShorthandType::Properties | ShorthandType::Always
            ),
            apply_never: matches!(shorthand_type, ShorthandType::Never),
            apply_consistent: matches!(shorthand_type, ShorthandType::Consistent),
            apply_consistent_as_needed: matches!(shorthand_type, ShorthandType::ConsistentAsNeeded),
            avoid_quotes: options.avoid_quotes,
            ignore_constructors: options.ignore_constructors,
            avoid_explicit_return_arrows: options.avoid_explicit_return_arrows,
            methods_ignore_pattern: options.methods_ignore_pattern,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObjectShorthandConfig {
    apply_to_methods: bool,
    apply_to_properties: bool,
    apply_never: bool,
    apply_consistent: bool,
    apply_consistent_as_needed: bool,
    avoid_quotes: bool,
    ignore_constructors: bool,
    avoid_explicit_return_arrows: bool,
    methods_ignore_pattern: Option<Regex>,
}

impl Default for ObjectShorthandConfig {
    fn default() -> Self {
        Self {
            apply_to_methods: true,
            apply_to_properties: true,
            apply_never: false,
            apply_consistent: false,
            apply_consistent_as_needed: false,
            avoid_quotes: false,
            ignore_constructors: false,
            avoid_explicit_return_arrows: false,
            methods_ignore_pattern: None,
        }
    }
}

impl std::ops::Deref for ObjectShorthand {
    type Target = ObjectShorthandConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// Require or disallow method and property shorthand syntax for object literals
    ///
    /// ### Why is this bad?
    /// Stylistic preference
    ///
    /// ### Example
    /// Here are a few common examples using the ES5 syntax:
    ///
    /// ```javascript
    /// var properties = { x: x, y: y, z: z, };
    /// var methods = { a: function() {}, b: function() {} };
    /// ```
    ///
    /// Now here are ES6 equivalents:
    ///
    /// ```javascript
    /// var properties = { x, y, z };
    /// var methods = { a() {}, b() {} };
    /// ```
    ObjectShorthand,
    eslint,
    style,
    fix,
    config = ObjectShorthandTupleConfig,
    version = "1.59.0",
);

impl Rule for ObjectShorthand {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<TupleRuleConfig<Self>>(value).map(TupleRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ObjectExpression(object_expr) => {
                if self.apply_consistent {
                    check_consistency(ctx, object_expr, false);
                } else if self.apply_consistent_as_needed {
                    check_consistency(ctx, object_expr, true);
                }
            }
            AstKind::ObjectProperty(property) => check_object_property(self, ctx, property),
            _ => {}
        }
    }
}

impl<'de> Deserialize<'de> for ObjectShorthand {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let config = ObjectShorthandTupleConfig::deserialize(deserializer)?;
        Ok(Self(Box::new(config.into())))
    }
}

fn make_function_shorthand<'a>(
    ctx: &LintContext<'a>,
    property: &ObjectProperty<'a>,
    fn_or_arrow_fn: Either<&Function<'a>, &ArrowFunctionExpression<'a>>,
) {
    let span = match fn_or_arrow_fn {
        Either::Left(func) => func.span(),
        Either::Right(func) => func.span(),
    };
    ctx.diagnostic_with_fix(expected_method_shorthand(span), |fixer| {
        let has_comment = ctx.semantic().has_comments_between(Span::new(
            property.key.span().start,
            property.value.span().start,
        ));
        if has_comment {
            return fixer.noop();
        }

        let key_prefix = match fn_or_arrow_fn {
            Either::Left(func) => match (func.r#async, func.generator) {
                (true, true) => "async *",
                (true, false) => "async ",
                (false, true) => "*",
                (false, false) => "",
            },
            Either::Right(func) => {
                if func.r#async {
                    "async "
                } else {
                    ""
                }
            }
        };

        let property_key_span = property.key.span();
        let key_text = if property.computed {
            let (Some(paren_start), Some(paren_end_offset)) = (
                ctx.find_prev_token_from(property_key_span.start, "["),
                ctx.find_next_token_from(property_key_span.end, "]"),
            ) else {
                return fixer.noop();
            };
            ctx.source_range(Span::new(paren_start, property_key_span.end + paren_end_offset + 1))
        } else {
            ctx.source_range(property_key_span)
        };

        match fn_or_arrow_fn {
            Either::Left(func) => {
                let next_token = if func.generator {
                    ctx.find_next_token_from(property_key_span.end, "*")
                        .map(|offset| offset + 1 /* "*".len() */)
                } else {
                    ctx.find_next_token_from(property_key_span.end, "function")
                        .map(|offset| offset + 8 /* "function".len() */)
                };
                let Some(func_token) = next_token else {
                    return fixer.noop();
                };
                let body =
                    ctx.source_range(Span::new(property_key_span.end + func_token, func.span.end));
                let ret = format!("{key_prefix}{key_text}{body}");
                fixer.replace(property.span, ret)
            }
            Either::Right(func) => {
                let next_token = ctx
                    .find_prev_token_from(func.body.span.start, "=>")
                    .map(|offset| offset + 2 /* "=>".len() */);
                let Some(arrow_token) = next_token else {
                    return fixer.noop();
                };
                let arrow_body = ctx.source_range(Span::new(
                    arrow_token,
                    property.value.without_parentheses().span().end,
                ));
                let old_param_text = ctx.source_range(Span::new(
                    func.params.span.start,
                    func.return_type.as_ref().map_or(func.params.span.end, |p| p.span.end),
                ));
                let should_add_parens = if func.r#async {
                    if let Some(async_token) = ctx.find_next_token_from(func.span.start, "async")
                        && let Some(first) = func.params.items.first()
                    {
                        ctx.find_next_token_within(
                            func.span.start + async_token,
                            first.span.start,
                            "(",
                        )
                        .is_none()
                    } else {
                        false
                    }
                } else if let Some(first_param) = func.params.items.first() {
                    ctx.find_next_token_within(func.span.start, first_param.span.start, "(")
                        .is_none()
                } else {
                    false
                };
                let new_param_text = if should_add_parens {
                    format!("({old_param_text})")
                } else {
                    old_param_text.to_string()
                };
                let type_param =
                    func.type_parameters.as_ref().map_or("", |t| ctx.source_range(t.span()));
                let ret = format!("{key_prefix}{key_text}{type_param}{new_param_text}{arrow_body}");
                fixer.replace(property.span, ret)
            }
        }
    });
}

fn make_function_long_form<'a>(
    rule: &ObjectShorthand,
    ctx: &LintContext<'a>,
    property: &ObjectProperty<'a>,
) {
    let diagnostic = if rule.apply_never {
        expected_method_longform(property.span)
    } else {
        expected_literal_method_longform(property.span)
    };
    ctx.diagnostic_with_fix(diagnostic, |fixer| {
        let property_key_span = property.key.span();
        let key_text_range = if property.computed {
            let (Some(paren_start), Some(paren_end_offset)) = (
                ctx.find_prev_token_from(property_key_span.start, "["),
                ctx.find_next_token_from(property_key_span.end, "]"),
            ) else {
                return fixer.noop();
            };
            Span::new(paren_start, property_key_span.end + paren_end_offset + 1)
        } else {
            property_key_span
        };
        let key_text = ctx.source_range(key_text_range);

        let Expression::FunctionExpression(func) = &property.value.without_parentheses() else {
            return fixer.noop();
        };
        let function_header = match (func.r#async, func.generator) {
            (true, true) => "async function*",
            (true, false) => "async function",
            (false, true) => "function*",
            (false, false) => "function",
        };

        let replace_range = Span::new(property.span.start, key_text_range.end);
        fixer.replace(replace_range, format!("{key_text}: {function_header}"))
    });
}

fn check_longform_methods<'a>(
    rule: &ObjectShorthand,
    ctx: &LintContext<'a>,
    property: &ObjectProperty<'a>,
) {
    if rule.ignore_constructors
        && property.key.is_identifier()
        && property.key.name().is_some_and(is_constructor)
    {
        return;
    }

    if let (Some(pattern), Some(static_name)) =
        (rule.methods_ignore_pattern.as_ref(), property.key.static_name())
        && pattern.is_match(static_name.as_ref())
    {
        return;
    }

    let is_key_string_literal = is_property_key_string_literal(property);
    if rule.avoid_quotes && is_key_string_literal {
        return;
    }

    if let Expression::FunctionExpression(func) = &property.value.without_parentheses() {
        make_function_shorthand(ctx, property, Either::Left(func));
    }

    if rule.avoid_explicit_return_arrows
        && let Expression::ArrowFunctionExpression(func) = &property.value.without_parentheses()
        && !arrow_uses_lexical_identifiers(ctx, func)
        && !func.expression
    {
        make_function_shorthand(ctx, property, Either::Right(func));
    }
}

fn check_shorthand_properties<'a>(ctx: &LintContext<'a>, property: &ObjectProperty<'a>) {
    if let Some(property_name) = property.key.name() {
        ctx.diagnostic_with_fix(expected_property_longform(property.span), |fixer| {
            fixer.replace(property.span, format!("{property_name}: {property_name}"))
        });
    }
}

fn check_longform_properties<'a>(
    rule: &ObjectShorthand,
    ctx: &LintContext<'a>,
    property: &ObjectProperty<'a>,
) {
    if rule.avoid_quotes && is_property_key_string_literal(property) {
        return;
    }

    let Expression::Identifier(value_identifier) = &property.value.without_parentheses() else {
        return;
    };

    if ctx.comments().iter().any(|comment| {
        if !property.span.contains_inclusive(comment.span) {
            return false;
        }
        comment.is_jsdoc() && ctx.source_range(comment.span).contains("@type")
    }) {
        return;
    }

    if let Some(property_name) = property.key.name()
        && property_name == value_identifier.name
    {
        ctx.diagnostic_with_fix(expected_property_shorthand(property.span), |fixer| {
            if ctx.semantic().has_comments_between(Span::new(
                property.key.span().start,
                value_identifier.span.end,
            )) {
                return fixer.noop();
            }
            fixer.replace(property.span, property_name.to_string())
        });
    }
}

fn check_consistency<'a>(
    ctx: &LintContext<'a>,
    obj_expr: &ObjectExpression<'a>,
    check_redundancy: bool,
) {
    let properties = obj_expr.properties.iter().filter_map(|property_kind| match property_kind {
        ObjectPropertyKind::ObjectProperty(property) => {
            can_property_have_shorthand(property).then_some(property)
        }
        ObjectPropertyKind::SpreadProperty(_) => None,
    });

    let properties_count = properties.clone().count();
    if properties_count > 0 {
        let shorthand_properties_count =
            properties.clone().filter(|p| is_shorthand_property(p)).count();

        if shorthand_properties_count != properties_count {
            if shorthand_properties_count > 0 {
                ctx.diagnostic(unexpected_mix(obj_expr.span));
            } else if check_redundancy && properties.clone().all(|p| is_redundant_property(p)) {
                ctx.diagnostic(expected_all_properties_shorthanded(obj_expr.span));
            }
        }
    }
}

fn check_object_property<'a>(
    rule: &ObjectShorthand,
    ctx: &LintContext<'a>,
    property: &ObjectProperty<'a>,
) {
    let is_concise_property = property.shorthand || property.method;

    if !can_property_have_shorthand(property) {
        return;
    }

    if is_concise_property {
        if property.method
            && (rule.apply_never || rule.avoid_quotes && is_property_key_string_literal(property))
        {
            make_function_long_form(rule, ctx, property);
        } else if rule.apply_never {
            check_shorthand_properties(ctx, property);
        }
    } else if rule.apply_to_methods && is_property_value_anonymous_function(property) {
        check_longform_methods(rule, ctx, property);
    } else if rule.apply_to_properties {
        check_longform_properties(rule, ctx, property);
    }
}

fn arrow_uses_lexical_identifiers<'a>(
    ctx: &LintContext<'a>,
    arrow: &ArrowFunctionExpression<'a>,
) -> bool {
    let mut visitor = ArrowFunctionLexicalIdentifierVisitor::new(ctx);
    visitor.visit_arrow_function_expression(arrow);
    visitor.has_lexical_identifier
}

struct ArrowFunctionLexicalIdentifierVisitor<'a, 'c> {
    ctx: &'c LintContext<'a>,
    has_lexical_identifier: bool,
}

impl<'a, 'c> ArrowFunctionLexicalIdentifierVisitor<'a, 'c> {
    fn new(ctx: &'c LintContext<'a>) -> Self {
        Self { ctx, has_lexical_identifier: false }
    }
}

impl<'a> Visit<'a> for ArrowFunctionLexicalIdentifierVisitor<'a, '_> {
    fn visit_function(
        &mut self,
        _it: &oxc_ast::ast::Function<'a>,
        _flags: oxc_semantic::ScopeFlags,
    ) {
    }

    fn visit_arrow_function_expression(&mut self, it: &ArrowFunctionExpression<'a>) {
        if self.has_lexical_identifier {
            return;
        }

        walk::walk_arrow_function_expression(self, it);
    }

    fn visit_this_expression(&mut self, _it: &oxc_ast::ast::ThisExpression) {
        self.has_lexical_identifier = true;
    }

    fn visit_super(&mut self, _it: &oxc_ast::ast::Super) {
        self.has_lexical_identifier = true;
    }

    fn visit_meta_property(&mut self, it: &oxc_ast::ast::MetaProperty<'a>) {
        if it.meta.name == "new" && it.property.name == "target" {
            self.has_lexical_identifier = true;
        }
    }

    fn visit_identifier_reference(&mut self, it: &oxc_ast::ast::IdentifierReference<'a>) {
        if self.ctx.scoping().root_unresolved_references().get("arguments").is_some_and(
            |references| references.iter().any(|&reference_id| reference_id == it.reference_id()),
        ) {
            self.has_lexical_identifier = true;
        }
    }
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ShorthandType {
    #[default]
    Always,
    Methods,
    Properties,
    Consistent,
    ConsistentAsNeeded,
    Never,
}

static CTOR_PREFIX_REGEX: Lazy<Regex> = lazy_regex!(r"[^_$0-9]");

/// Determines if the first character of the name
/// is a capital letter.
/// * `name` - The name of the node to evaluate.
///
/// Returns true if the first character of the property name is a capital letter, false if not.
fn is_constructor<N: AsRef<str>>(name: N) -> bool {
    // Not a constructor if name has no characters apart from '_', '$' and digits e.g. '_', '$$', '_8'
    let Some(matched) = CTOR_PREFIX_REGEX.find(name.as_ref()) else {
        return false;
    };

    name.as_ref().chars().nth(matched.start()).is_some_and(char::is_uppercase)
}

fn is_property_value_function(property: &ObjectProperty) -> bool {
    matches!(
        property.value,
        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
    )
}

fn is_property_value_anonymous_function(property: &ObjectProperty) -> bool {
    match &property.value.without_parentheses() {
        Expression::FunctionExpression(func) => func.id.is_none(),
        Expression::ArrowFunctionExpression(_) => true,
        _ => false,
    }
}

fn is_property_key_string_literal(property: &ObjectProperty) -> bool {
    matches!(property.key.as_expression(), Some(Expression::StringLiteral(_)))
}

fn is_shorthand_property(property: &ObjectProperty) -> bool {
    property.shorthand || property.method
}

fn is_redundant_property(property: &ObjectProperty) -> bool {
    match &property.value {
        Expression::FunctionExpression(func) => func.id.is_none(),
        Expression::Identifier(value_identifier) => {
            if let Some(property_name) = property.key.name() {
                property_name == value_identifier.name
            } else {
                false
            }
        }
        _ => false,
    }
}

fn can_property_have_shorthand(property: &ObjectProperty) -> bool {
    // Ignore getters and setters
    if property.kind != PropertyKind::Init {
        return false;
    }

    // Ignore computed properties, unless they are functions
    if property.computed && !is_property_value_function(property) {
        return false;
    }

    true
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var x = {y() {}}", None),
        ("var x = {y}", None),
        ("var x = {a: b}", None),
        ("var x = {a: 'a'}", None),
        ("var x = {'a': 'a'}", None),
        ("var x = {'a': b}", None),
        ("var x = {y(x) {}}", None),
        ("var {x,y,z} = x", None),
        ("var {x: {y}} = z", None),
        ("var x = {*x() {}}", None),
        ("var x = {x: y}", None),
        ("var x = {x: y, y: z}", None),
        ("var x = {x: y, y: z, z: 'z'}", None),
        ("var x = {x() {}, y: z, l(){}}", None),
        ("var x = {x: y, y: z, a: b}", None),
        ("var x = {x: y, y: z, 'a': b}", None),
        ("var x = {x: y, y() {}, z: a}", None),
        ("var x = {[y]: y}", None),
        ("doSomething({x: y})", None),
        ("doSomething({'x': y})", None),
        ("doSomething({x: 'x'})", None),
        ("doSomething({'x': 'x'})", None),
        ("doSomething({y() {}})", None),
        ("doSomething({x: y, y() {}})", None),
        ("doSomething({y() {}, z: a})", None),
        ("!{ a: function a(){} };", None),
        ("var x = {y: (x)=>x}", None),
        ("doSomething({y: (x)=>x})", None),
        ("var x = {y: (x)=>x, y: a}", None),
        ("doSomething({x, y: (x)=>x})", None),
        ("({ foo: x => { return; }})", None),
        ("({ foo: (x) => { return; }})", None),
        ("({ foo: () => { return; }})", None),
        ("var x = {get y() {}}", None),
        ("var x = {set y(z) {}}", None),
        ("var x = {get y() {}, set y(z) {}}", None),
        ("doSomething({get y() {}})", None),
        ("doSomething({set y(z) {}})", None),
        ("doSomething({get y() {}, set y(z) {}})", None),
        ("var x = {[y]: y}", Some(serde_json::json!(["properties"]))),
        ("var x = {['y']: 'y'}", Some(serde_json::json!(["properties"]))),
        ("var x = {['y']: y}", Some(serde_json::json!(["properties"]))),
        ("var x = {[y]() {}}", Some(serde_json::json!(["methods"]))),
        ("var x = {[y]: function x() {}}", Some(serde_json::json!(["methods"]))),
        ("var x = {[y]: y}", Some(serde_json::json!(["methods"]))),
        ("var x = {y() {}}", Some(serde_json::json!(["methods"]))),
        ("var x = {x, y() {}, a:b}", Some(serde_json::json!(["methods"]))),
        ("var x = {y}", Some(serde_json::json!(["properties"]))),
        ("var x = {y: {b}}", Some(serde_json::json!(["properties"]))),
        ("var x = {a: n, c: d, f: g}", Some(serde_json::json!(["never"]))),
        ("var x = {a: function(){}, b: {c: d}}", Some(serde_json::json!(["never"]))),
        (
            "var x = {ConstructorFunction: function(){}, a: b}",
            Some(serde_json::json!(["always", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_ConstructorFunction: function(){}, a: b}",
            Some(serde_json::json!(["always", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {$ConstructorFunction: function(){}, a: b}",
            Some(serde_json::json!(["always", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {__ConstructorFunction: function(){}, a: b}",
            Some(serde_json::json!(["always", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_0ConstructorFunction: function(){}, a: b}",
            Some(serde_json::json!(["always", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {notConstructorFunction(){}, b: c}",
            Some(serde_json::json!(["always", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {ConstructorFunction: function(){}, a: b}",
            Some(serde_json::json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_ConstructorFunction: function(){}, a: b}",
            Some(serde_json::json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {$ConstructorFunction: function(){}, a: b}",
            Some(serde_json::json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {__ConstructorFunction: function(){}, a: b}",
            Some(serde_json::json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_0ConstructorFunction: function(){}, a: b}",
            Some(serde_json::json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {notConstructorFunction(){}, b: c}",
            Some(serde_json::json!(["methods", { "ignoreConstructors": true }])),
        ),
        ("var x = {ConstructorFunction: function(){}, a: b}", Some(serde_json::json!(["never"]))),
        (
            "var x = {notConstructorFunction: function(){}, b: c}",
            Some(serde_json::json!(["never"])),
        ),
        (
            "var x = { foo: function() {}  }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { foo: function() {}  }",
            Some(serde_json::json!(["methods", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { foo: function*() {}  }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { foo: async function() {}  }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { foo: () => { return 5; }  }",
            Some(
                serde_json::json!([ "always", { "methodsIgnorePattern": "^foo$", "avoidExplicitReturnArrows": true, }, ]),
            ),
        ),
        (
            "var x = { 'foo': function() {}  }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { ['foo']: function() {}  }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { 123: function() {}  }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "^123$" }])),
        ),
        (
            "var x = { afoob: function() {}  }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "foo" }])),
        ),
        (
            "var x = { afoob: function() {}  }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "^.foo.$" }])),
        ),
        (
            "var x = { '👍foo👍': function() {}  }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "^.foo.$" }])),
        ),
        (
            "var x = {'a': function(){}}",
            Some(serde_json::json!(["always", { "avoidQuotes": true }])),
        ),
        (
            "var x = {['a']: function(){}}",
            Some(serde_json::json!(["methods", { "avoidQuotes": true }])),
        ),
        ("var x = {'y': y}", Some(serde_json::json!(["properties", { "avoidQuotes": true }]))),
        ("let {a, b} = o;", Some(serde_json::json!(["never"]))),
        ("var x = {foo: foo, bar: bar, ...baz}", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2018 },
        ("var x = {a: a, b: b}", Some(serde_json::json!(["consistent"]))),
        ("var x = {a: b, c: d, f: g}", Some(serde_json::json!(["consistent"]))),
        ("var x = {a, b}", Some(serde_json::json!(["consistent"]))),
        ("var x = {a, b, get test() { return 1; }}", Some(serde_json::json!(["consistent"]))),
        ("var x = {...bar}", Some(serde_json::json!(["consistent-as-needed"]))), // { "ecmaVersion": 2018 },
        ("var x = {foo, bar, ...baz}", Some(serde_json::json!(["consistent"]))), // { "ecmaVersion": 2018 },
        ("var x = {bar: baz, ...qux}", Some(serde_json::json!(["consistent"]))), // { "ecmaVersion": 2018 },
        ("var x = {...foo, bar: bar, baz: baz}", Some(serde_json::json!(["consistent"]))), // { "ecmaVersion": 2018 },
        ("var x = {a, b}", Some(serde_json::json!(["consistent-as-needed"]))),
        (
            "var x = {a, b, get test(){return 1;}}",
            Some(serde_json::json!(["consistent-as-needed"])),
        ),
        ("var x = {0: 'foo'}", Some(serde_json::json!(["consistent-as-needed"]))),
        ("var x = {'key': 'baz'}", Some(serde_json::json!(["consistent-as-needed"]))),
        ("var x = {foo: 'foo'}", Some(serde_json::json!(["consistent-as-needed"]))),
        ("var x = {[foo]: foo}", Some(serde_json::json!(["consistent-as-needed"]))),
        ("var x = {foo: function foo() {}}", Some(serde_json::json!(["consistent-as-needed"]))),
        ("var x = {[foo]: 'foo'}", Some(serde_json::json!(["consistent-as-needed"]))),
        ("var x = {bar, ...baz}", Some(serde_json::json!(["consistent-as-needed"]))), // { "ecmaVersion": 2018 },
        ("var x = {bar: baz, ...qux}", Some(serde_json::json!(["consistent-as-needed"]))), // { "ecmaVersion": 2018 },
        ("var x = {...foo, bar, baz}", Some(serde_json::json!(["consistent-as-needed"]))), // { "ecmaVersion": 2018 },
        (
            "({ x: () => foo })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": false }])),
        ),
        (
            "({ x: () => { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": false }])),
        ),
        (
            "({ x: () => foo })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x() { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x() { return; }, y() { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x() { return; }, y: () => foo })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => foo, y() { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { this; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "function foo() { ({ x: () => { arguments; } }) }",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                            class Foo extends Bar {
                              constructor() {
                                  var foo = { x: () => { super(); } };
                              }
                          }
                        ",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                            class Foo extends Bar {
                                baz() {
                                    var foo = { x: () => { super.baz(); } };
                                }
                            }
                        ",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                            function foo() {
                                var x = { x: () => { new.target; } };
                            }
                        ",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                            function foo() {
                                var x = {
                                    x: () => {
                                        var y = () => { this; };
                                    }
                                };
                            }
                        ",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                            function foo() {
                                var x = {
                                    x: () => {
                                        var y = () => { this; };
                                        function foo() { this; }
                                    }
                                };
                            }
                        ",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                            function foo() {
                                var x = {
                                    x: () => {
                                        return { y: () => { this; } };
                                    }
                                };
                            }
                        ",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ [foo.bar]: () => {} })",
            Some(serde_json::json!(["always", { "ignoreConstructors": true }])),
        ),
        ("({ val: /** @type {number} */ (val) })", None),
        ("({ 'prop': /** @type {string} */ (prop) })", None),
        (
            "({ val: /**
             * @type {number}
             */ (val) })",
            None,
        ),
        (
            "({ val: /**
              * @type {number}
              */ (val) })",
            None,
        ),
        (
            "({ val: /**
               * @type {number}
               */ (val) })",
            None,
        ),
        (
            "({ val: /**
                * @type {number}
                */ (val) })",
            None,
        ),
        (
            "({ val: /**
                 * @type {number}
                 */ (val) })",
            None,
        ),
        (
            "({ val: /**
              *  @type   {number}
              */ (val) })",
            None,
        ),
        (
            "({ val: /**
             *  @type   {string} myParam
             */ (val) })",
            None,
        ),
        (
            "({ val: /**
              *  @type   {Object} options
              */ (val) })",
            None,
        ),
        (
            "({ val: /**
                 *	@type	{Array}
                 */ (val) })",
            None,
        ),
        (
            "({ val: /**
               *
               * @type {Function}
               * @param {string} name
               */ (val) })",
            None,
        ),
    ];

    let fail = vec![
        ("var x = {x: x}", None),
        ("var x = {'x': x}", None),
        ("var x = {y: y, x: x}", None),
        ("var x = {y: z, x: x, a: b}", None),
        (
            "var x = {y: z,
             x: x,
             a: b
             // comment
            }",
            None,
        ),
        (
            "var x = {y: z,
             a: b,
             // comment
            f: function() {}}",
            None,
        ),
        (
            "var x = {a: b,
            /* comment */
            y: y
             }",
            None,
        ),
        (
            "var x = {
              a: b,
              /* comment */
              y: y
            }",
            None,
        ),
        (
            "var x = {
              f: function() {
                /* comment */
                a(b);
                }
              }",
            None,
        ),
        (
            "var x = {
              [f]: function() {
                /* comment */
                a(b);
                }
              }",
            None,
        ),
        (
            "var x = {
              f: function*() {
                /* comment */
                a(b);
                }
              }",
            None,
        ),
        (
            "var x = {
              f: /* comment */ function() {
              }
              }",
            None,
        ),
        (
            "var x = {
             f /* comment */: function() {
              }
              }",
            None,
        ),
        ("var x = {a: /* comment */ a}", None),
        ("var x = {a /* comment */: a}", None),
        ("var x = {a: (a /* comment */)}", None),
        ("var x = {'a': /* comment */ a}", None),
        ("var x = {'a': (a /* comment */)}", None),
        ("var x = {'a' /* comment */: a}", None),
        ("var x = {y: function() {}}", None),
        ("var x = {y: function*() {}}", None),
        ("var x = {x: y, y: z, a: a}", None),
        ("var x = {ConstructorFunction: function(){}, a: b}", None),
        ("var x = {x: y, y: z, a: function(){}, b() {}}", None),
        ("var x = {x: x, y: function() {}}", None),
        ("doSomething({x: x})", None),
        ("doSomething({'x': x})", None),
        ("doSomething({a: 'a', 'x': x})", None),
        ("doSomething({y: function() {}})", None),
        ("doSomething({[y]: function() {}})", None),
        ("doSomething({['y']: function() {}})", None),
        ("({ foo: async function () {} })", None), // { "ecmaVersion": 8 },
        ("({ 'foo': async function() {} })", None), // { "ecmaVersion": 8 },
        ("({ [foo]: async function() {} })", None), // { "ecmaVersion": 8 },
        ("({ [foo.bar]: function*() {} })", None),
        ("({ [foo   ]: function() {} })", None),
        ("({ [ foo ]: async function() {} })", None), // { "ecmaVersion": 8 },
        ("({ foo: function *() {} })", None),
        ("({ [  foo   ]: function() {} })", None),
        ("({ [  foo]: function() {} })", None),
        ("var x = {y: function() {}}", Some(serde_json::json!(["methods"]))),
        ("var x = {x, y() {}, z: function() {}}", Some(serde_json::json!(["methods"]))),
        ("var x = {ConstructorFunction: function(){}, a: b}", Some(serde_json::json!(["methods"]))),
        ("var x = {[y]: function() {}}", Some(serde_json::json!(["methods"]))),
        ("({ [(foo)]: function() { return; } })", None),
        ("({ [(foo)]: async function() { return; } })", None), // { "ecmaVersion": 8 },
        ("({ [(((((((foo)))))))]: function() { return; } })", None),
        ("({ [(foo)]() { return; } })", Some(serde_json::json!(["never"]))),
        ("({ async [(foo)]() { return; } })", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 8 },
        ("({ *[((foo))]() { return; } })", Some(serde_json::json!(["never"]))),
        ("({ [(((((((foo)))))))]() { return; } })", Some(serde_json::json!(["never"]))),
        ("({ 'foo bar'() { return; } })", Some(serde_json::json!(["never"]))),
        ("({ *foo() { return; } })", Some(serde_json::json!(["never"]))),
        ("({ async foo() { return; } })", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 8 },
        ("({ *['foo bar']() { return; } })", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 8 },
        ("var x = {x: x}", Some(serde_json::json!(["properties"]))),
        ("var x = {a, b, c(){}, x: x}", Some(serde_json::json!(["properties"]))),
        ("var x = {y() {}}", Some(serde_json::json!(["never"]))),
        ("var x = {*y() {}}", Some(serde_json::json!(["never"]))),
        ("var x = {y}", Some(serde_json::json!(["never"]))),
        ("var x = {y, a: b, *x(){}}", Some(serde_json::json!(["never"]))),
        ("var x = {y: {x}}", Some(serde_json::json!(["never"]))),
        ("var x = {ConstructorFunction(){}, a: b}", Some(serde_json::json!(["never"]))),
        ("var x = {notConstructorFunction(){}, b: c}", Some(serde_json::json!(["never"]))),
        ("var x = {foo: foo, bar: baz, ...qux}", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2018 },
        ("var x = {foo, bar: baz, ...qux}", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2018 },
        (
            "var x = {y: function() {}}",
            Some(serde_json::json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_y: function() {}}",
            Some(serde_json::json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {$y: function() {}}",
            Some(serde_json::json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {__y: function() {}}",
            Some(serde_json::json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_0y: function() {}}",
            Some(serde_json::json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = { afoob: function() {} }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { afoob: function() {} }",
            Some(serde_json::json!(["methods", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { 'afoob': function() {} }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { 1234: function() {} }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "^123$" }])),
        ),
        (
            "var x = { bar: function() {} }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "foo" }])),
        ),
        (
            "var x = { [foo]: function() {} }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "foo" }])),
        ),
        (
            "var x = { foo: foo }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        ("var x = {a: a}", Some(serde_json::json!(["always", { "avoidQuotes": true }]))),
        (
            "var x = {a: function(){}}",
            Some(serde_json::json!(["methods", { "avoidQuotes": true }])),
        ),
        (
            "var x = {[a]: function(){}}",
            Some(serde_json::json!(["methods", { "avoidQuotes": true }])),
        ),
        ("var x = {'a'(){}}", Some(serde_json::json!(["always", { "avoidQuotes": true }]))),
        ("var x = {['a'](){}}", Some(serde_json::json!(["methods", { "avoidQuotes": true }]))),
        ("var x = {a: a, b}", Some(serde_json::json!(["consistent"]))),
        ("var x = {b, c: d, f: g}", Some(serde_json::json!(["consistent"]))),
        ("var x = {foo, bar: baz, ...qux}", Some(serde_json::json!(["consistent"]))), // { "ecmaVersion": 2018 },
        ("var x = {a: a, b: b}", Some(serde_json::json!(["consistent-as-needed"]))),
        ("var x = {a, z: function z(){}}", Some(serde_json::json!(["consistent-as-needed"]))),
        ("var x = {foo: function() {}}", Some(serde_json::json!(["consistent-as-needed"]))),
        ("var x = {a: a, b: b, ...baz}", Some(serde_json::json!(["consistent-as-needed"]))), // { "ecmaVersion": 2018 },
        ("var x = {foo, bar: bar, ...qux}", Some(serde_json::json!(["consistent-as-needed"]))), // { "ecmaVersion": 2018 },
        (
            "({ x: (arg => { return; }) })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x() { return; }, y: () => { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { return; }, y: () => foo })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { return; }, y: () => { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: foo => { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: (foo = 1) => { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: ({ foo: bar = 1 } = {}) => { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { function foo() { this; } } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { var foo = function() { arguments; } } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { function foo() { arguments; } } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                            ({
                                x: () => {
                                    class Foo extends Bar {
                                        constructor() {
                                            super();
                                        }
                                    }
                                }
                            })
                        ",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                            ({
                                x: () => {
                                    function foo() {
                                        new.target;
                                    }
                                }
                            })
                        ",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ 'foo bar': () => { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ [foo]: () => { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: 1, foo: async (bar = 1) => { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ), // { "ecmaVersion": 8 },
        (
            "({ [ foo ]: async bar => { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ), // { "ecmaVersion": 8 },
        (
            "({ key: (arg = () => {}) => {} })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                            function foo() {
                                var x = {
                                    x: () => {
                                        this;
                                        return { y: () => { foo; } };
                                    }
                                };
                            }
                        ",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                            function foo() {
                                var x = {
                                    x: () => {
                                        ({ y: () => { foo; } });
                                        this;
                                    }
                                };
                            }
                        ",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        ("({ a: (function(){ return foo; }) })", None),
        (
            "({ a: (() => { return foo; }) })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: ((arg) => { return foo; }) })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: ((arg, arg2) => { return foo; }) })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: (async () => { return foo; }) })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: (async (arg) => { return foo; }) })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: (async (arg, arg2) => { return foo; }) })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        ("({ a: async function*() {} })", Some(serde_json::json!(["always"]))),
        ("({ async* a() {} })", Some(serde_json::json!(["never"]))),
        (
            "
                            const test = {
                                key: <T,>(): void => { },
                                key: async <T,>(): Promise<void> => { },

                                key: <T,>(arg: T): T => { return arg },
                                key: async <T,>(arg: T): Promise<T> => { return arg },
                            }
                        ",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ), // { "parser": require("../../fixtures/parsers/typescript-parsers/object-with-generic-arrow-fn-props"), },
        (
            "
                            const test = {
                                key: (): void => {x()},
                                key: ( (): void => {x()} ),
                                key: ( (): (void) => {x()} ),

                                key: (arg: t): void => {x()},
                                key: ( (arg: t): void => {x()} ),
                                key: ( (arg: t): (void) => {x()} ),

                                key: (arg: t, arg2: t): void => {x()},
                                key: ( (arg: t, arg2: t): void => {x()} ),
                                key: ( (arg: t, arg2: t): (void) => {x()} ),

                                key: async (): void => {x()},
                                key: ( async (): void => {x()} ),
                                key: ( async (): (void) => {x()} ),

                                key: async (arg: t): void => {x()},
                                key: ( async (arg: t): void => {x()} ),
                                key: ( async (arg: t): (void) => {x()} ),

                                key: async (arg: t, arg2: t): void => {x()},
                                key: ( async (arg: t, arg2: t): void => {x()} ),
                                key: ( async (arg: t, arg2: t): (void) => {x()} ),
                            }
                        ",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ), // { "parser": require("../../fixtures/parsers/typescript-parsers/object-with-arrow-fn-props"), },
        ("({ val: /** regular comment */ (val) })", None),
        ("({ val: /** @param {string} name */ (val) })", None),
        ("({ val: /** @returns {number} */ (val) })", None),
        ("({ val: /** @description some text */ (val) })", None),
        (
            "({ val: /**
             * @param {string} name
             */ (val) })",
            None,
        ),
        (
            "({ val: /**
              * @returns {number}
              */ (val) })",
            None,
        ),
        (
            "({ val: /**
               * @description some text
               */ (val) })",
            None,
        ),
        (
            "({ val: /**
                * @param {string} name
                */ (val) })",
            None,
        ),
        (
            "({ val: /**
                 * @returns {number}
                 */ (val) })",
            None,
        ),
        (
            "({ val: /**
              *  @param   {string}  name
              */ (val) })",
            None,
        ),
        (
            "({ val: /**
             *  @returns   {number} result
             */ (val) })",
            None,
        ),
        (
            "({ val: /**
               *
               * @param {string} name
               * @returns {number}
               */ (val) })",
            None,
        ),
    ];

    let fix = vec![
        ("var x = {x: x}", "var x = {x}", None),
        ("var x = {'x': x}", "var x = {x}", None),
        ("var x = {y: y, x: x}", "var x = {y, x}", None),
        ("var x = {y: z, x: x, a: b}", "var x = {y: z, x, a: b}", None),
        (
            "var x = {y: z,
             x: x,
             a: b
             // comment
            }",
            "var x = {y: z,
             x,
             a: b
             // comment
            }",
            None,
        ),
        (
            "var x = {y: z,
             a: b,
             // comment
            f: function() {}}",
            "var x = {y: z,
             a: b,
             // comment
            f() {}}",
            None,
        ),
        (
            "var x = {a: b,
            /* comment */
            y: y
             }",
            "var x = {a: b,
            /* comment */
            y
             }",
            None,
        ),
        (
            "var x = {
              a: b,
              /* comment */
              y: y
            }",
            "var x = {
              a: b,
              /* comment */
              y
            }",
            None,
        ),
        (
            "var x = {
              f: function() {
                /* comment */
                a(b);
                }
              }",
            "var x = {
              f() {
                /* comment */
                a(b);
                }
              }",
            None,
        ),
        (
            "var x = {
              [f]: function() {
                /* comment */
                a(b);
                }
              }",
            "var x = {
              [f]() {
                /* comment */
                a(b);
                }
              }",
            None,
        ),
        (
            "var x = {
              f: function*() {
                /* comment */
                a(b);
                }
              }",
            "var x = {
              *f() {
                /* comment */
                a(b);
                }
              }",
            None,
        ),
        ("var x = {y: function() {}}", "var x = {y() {}}", None),
        ("var x = {y: function*() {}}", "var x = {*y() {}}", None),
        ("var x = {x: y, y: z, a: a}", "var x = {x: y, y: z, a}", None),
        (
            "var x = {ConstructorFunction: function(){}, a: b}",
            "var x = {ConstructorFunction(){}, a: b}",
            None,
        ),
        (
            "var x = {x: y, y: z, a: function(){}, b() {}}",
            "var x = {x: y, y: z, a(){}, b() {}}",
            None,
        ),
        ("var x = {x: x, y: function() {}}", "var x = {x, y() {}}", None),
        ("doSomething({x: x})", "doSomething({x})", None),
        ("doSomething({'x': x})", "doSomething({x})", None),
        ("doSomething({a: 'a', 'x': x})", "doSomething({a: 'a', x})", None),
        ("doSomething({y: function() {}})", "doSomething({y() {}})", None),
        ("doSomething({[y]: function() {}})", "doSomething({[y]() {}})", None),
        ("doSomething({['y']: function() {}})", "doSomething({['y']() {}})", None),
        ("({ foo: async function () {} })", "({ async foo () {} })", None),
        ("({ 'foo': async function() {} })", "({ async 'foo'() {} })", None),
        ("({ [foo]: async function() {} })", "({ async [foo]() {} })", None),
        ("({ [foo.bar]: function*() {} })", "({ *[foo.bar]() {} })", None),
        ("({ [foo   ]: function() {} })", "({ [foo   ]() {} })", None),
        ("({ [ foo ]: async function() {} })", "({ async [ foo ]() {} })", None),
        ("({ foo: function *() {} })", "({ *foo() {} })", None),
        ("({ [  foo   ]: function() {} })", "({ [  foo   ]() {} })", None),
        ("({ [  foo]: function() {} })", "({ [  foo]() {} })", None),
        ("var x = {y: function() {}}", "var x = {y() {}}", Some(serde_json::json!(["methods"]))),
        (
            "var x = {x, y() {}, z: function() {}}",
            "var x = {x, y() {}, z() {}}",
            Some(serde_json::json!(["methods"])),
        ),
        (
            "var x = {ConstructorFunction: function(){}, a: b}",
            "var x = {ConstructorFunction(){}, a: b}",
            Some(serde_json::json!(["methods"])),
        ),
        (
            "var x = {[y]: function() {}}",
            "var x = {[y]() {}}",
            Some(serde_json::json!(["methods"])),
        ),
        ("({ [(foo)]: function() { return; } })", "({ [(foo)]() { return; } })", None),
        ("({ [(foo)]: async function() { return; } })", "({ async [(foo)]() { return; } })", None),
        (
            "({ [(((((((foo)))))))]: function() { return; } })",
            "({ [(((((((foo)))))))]() { return; } })",
            None,
        ),
        (
            "({ [(foo)]() { return; } })",
            "({ [(foo)]: function() { return; } })",
            Some(serde_json::json!(["never"])),
        ),
        (
            "({ async [(foo)]() { return; } })",
            "({ [(foo)]: async function() { return; } })",
            Some(serde_json::json!(["never"])),
        ),
        (
            "({ *[((foo))]() { return; } })",
            "({ [((foo))]: function*() { return; } })",
            Some(serde_json::json!(["never"])),
        ),
        (
            "({ [(((((((foo)))))))]() { return; } })",
            "({ [(((((((foo)))))))]: function() { return; } })",
            Some(serde_json::json!(["never"])),
        ),
        (
            "({ 'foo bar'() { return; } })",
            "({ 'foo bar': function() { return; } })",
            Some(serde_json::json!(["never"])),
        ),
        (
            "({ *foo() { return; } })",
            "({ foo: function*() { return; } })",
            Some(serde_json::json!(["never"])),
        ),
        (
            "({ async foo() { return; } })",
            "({ foo: async function() { return; } })",
            Some(serde_json::json!(["never"])),
        ),
        (
            "({ *['foo bar']() { return; } })",
            "({ ['foo bar']: function*() { return; } })",
            Some(serde_json::json!(["never"])),
        ),
        ("var x = {x: x}", "var x = {x}", Some(serde_json::json!(["properties"]))),
        (
            "var x = {a, b, c(){}, x: x}",
            "var x = {a, b, c(){}, x}",
            Some(serde_json::json!(["properties"])),
        ),
        ("var x = {y() {}}", "var x = {y: function() {}}", Some(serde_json::json!(["never"]))),
        ("var x = {*y() {}}", "var x = {y: function*() {}}", Some(serde_json::json!(["never"]))),
        ("var x = {y}", "var x = {y: y}", Some(serde_json::json!(["never"]))),
        (
            "var x = {y, a: b, *x(){}}",
            "var x = {y: y, a: b, x: function*(){}}",
            Some(serde_json::json!(["never"])),
        ),
        ("var x = {y: {x}}", "var x = {y: {x: x}}", Some(serde_json::json!(["never"]))),
        (
            "var x = {ConstructorFunction(){}, a: b}",
            "var x = {ConstructorFunction: function(){}, a: b}",
            Some(serde_json::json!(["never"])),
        ),
        (
            "var x = {notConstructorFunction(){}, b: c}",
            "var x = {notConstructorFunction: function(){}, b: c}",
            Some(serde_json::json!(["never"])),
        ),
        (
            "var x = {foo: foo, bar: baz, ...qux}",
            "var x = {foo, bar: baz, ...qux}",
            Some(serde_json::json!(["always"])),
        ),
        (
            "var x = {foo, bar: baz, ...qux}",
            "var x = {foo: foo, bar: baz, ...qux}",
            Some(serde_json::json!(["never"])),
        ),
        (
            "var x = {y: function() {}}",
            "var x = {y() {}}",
            Some(serde_json::json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_y: function() {}}",
            "var x = {_y() {}}",
            Some(serde_json::json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {$y: function() {}}",
            "var x = {$y() {}}",
            Some(serde_json::json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {__y: function() {}}",
            "var x = {__y() {}}",
            Some(serde_json::json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_0y: function() {}}",
            "var x = {_0y() {}}",
            Some(serde_json::json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = { afoob: function() {} }",
            "var x = { afoob() {} }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { afoob: function() {} }",
            "var x = { afoob() {} }",
            Some(serde_json::json!(["methods", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { 'afoob': function() {} }",
            "var x = { 'afoob'() {} }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { 1234: function() {} }",
            "var x = { 1234() {} }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "^123$" }])),
        ),
        (
            "var x = { bar: function() {} }",
            "var x = { bar() {} }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "foo" }])),
        ),
        (
            "var x = { [foo]: function() {} }",
            "var x = { [foo]() {} }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "foo" }])),
        ),
        (
            "var x = { foo: foo }",
            "var x = { foo }",
            Some(serde_json::json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = {a: a}",
            "var x = {a}",
            Some(serde_json::json!(["always", { "avoidQuotes": true }])),
        ),
        (
            "var x = {a: function(){}}",
            "var x = {a(){}}",
            Some(serde_json::json!(["methods", { "avoidQuotes": true }])),
        ),
        (
            "var x = {[a]: function(){}}",
            "var x = {[a](){}}",
            Some(serde_json::json!(["methods", { "avoidQuotes": true }])),
        ),
        (
            "var x = {'a'(){}}",
            "var x = {'a': function(){}}",
            Some(serde_json::json!(["always", { "avoidQuotes": true }])),
        ),
        (
            "var x = {['a'](){}}",
            "var x = {['a']: function(){}}",
            Some(serde_json::json!(["methods", { "avoidQuotes": true }])),
        ),
        (
            "({ x: (arg => { return; }) })",
            "({ x(arg) { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { return; } })",
            "({ x() { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x() { return; }, y: () => { return; } })",
            "({ x() { return; }, y() { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { return; }, y: () => foo })",
            "({ x() { return; }, y: () => foo })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { return; }, y: () => { return; } })",
            "({ x() { return; }, y() { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: foo => { return; } })",
            "({ x(foo) { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: (foo = 1) => { return; } })",
            "({ x(foo = 1) { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: ({ foo: bar = 1 } = {}) => { return; } })",
            "({ x({ foo: bar = 1 } = {}) { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { function foo() { this; } } })",
            "({ x() { function foo() { this; } } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { var foo = function() { arguments; } } })",
            "({ x() { var foo = function() { arguments; } } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { function foo() { arguments; } } })",
            "({ x() { function foo() { arguments; } } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                            ({
                                x: () => {
                                    class Foo extends Bar {
                                        constructor() {
                                            super();
                                        }
                                    }
                                }
                            })
                        ",
            "
                            ({
                                x() {
                                    class Foo extends Bar {
                                        constructor() {
                                            super();
                                        }
                                    }
                                }
                            })
                        ",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                            ({
                                x: () => {
                                    function foo() {
                                        new.target;
                                    }
                                }
                            })
                        ",
            "
                            ({
                                x() {
                                    function foo() {
                                        new.target;
                                    }
                                }
                            })
                        ",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ 'foo bar': () => { return; } })",
            "({ 'foo bar'() { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ [foo]: () => { return; } })",
            "({ [foo]() { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: 1, foo: async (bar = 1) => { return; } })",
            "({ a: 1, async foo(bar = 1) { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ [ foo ]: async bar => { return; } })",
            "({ async [ foo ](bar) { return; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ key: (arg = () => {}) => {} })",
            "({ key(arg = () => {}) {} })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                            function foo() {
                                var x = {
                                    x: () => {
                                        this;
                                        return { y: () => { foo; } };
                                    }
                                };
                            }
                        ",
            "
                            function foo() {
                                var x = {
                                    x: () => {
                                        this;
                                        return { y() { foo; } };
                                    }
                                };
                            }
                        ",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                            function foo() {
                                var x = {
                                    x: () => {
                                        ({ y: () => { foo; } });
                                        this;
                                    }
                                };
                            }
                        ",
            "
                            function foo() {
                                var x = {
                                    x: () => {
                                        ({ y() { foo; } });
                                        this;
                                    }
                                };
                            }
                        ",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        ("({ a: (function(){ return foo; }) })", "({ a(){ return foo; } })", None),
        (
            "({ a: (() => { return foo; }) })",
            "({ a() { return foo; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: ((arg) => { return foo; }) })",
            "({ a(arg) { return foo; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: ((arg, arg2) => { return foo; }) })",
            "({ a(arg, arg2) { return foo; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: (async () => { return foo; }) })",
            "({ async a() { return foo; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: (async (arg) => { return foo; }) })",
            "({ async a(arg) { return foo; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: (async (arg, arg2) => { return foo; }) })",
            "({ async a(arg, arg2) { return foo; } })",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: async function*() {} })",
            "({ async *a() {} })",
            Some(serde_json::json!(["always"])),
        ),
        (
            "({ async* a() {} })",
            "({ a: async function*() {} })",
            Some(serde_json::json!(["never"])),
        ),
        // FIXME
        // (
        //     "
        //                     const test = {
        //                         key: <T>(): void => { },
        //                         key: async <T>(): Promise<void> => { },

        //                         key: <T>(arg: T): T => { return arg },
        //                         key: async <T>(arg: T): Promise<T> => { return arg },
        //                     }
        //                 ",
        //     "
        //                     const test = {
        //                         key<T>(): void { },
        //                         async key<T>(): Promise<void> { },

        //                         key<T>(arg: T): T { return arg },
        //                         async key<T>(arg: T): Promise<T> { return arg },
        //                     }
        //                 ",
        //     Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        // ),
        (
            "
                            const test = {
                                key: (): void => {x()},
                                key: ( (): void => {x()} ),
                                key: ( (): (void) => {x()} ),

                                key: (arg: t): void => {x()},
                                key: ( (arg: t): void => {x()} ),
                                key: ( (arg: t): (void) => {x()} ),

                                key: (arg: t, arg2: t): void => {x()},
                                key: ( (arg: t, arg2: t): void => {x()} ),
                                key: ( (arg: t, arg2: t): (void) => {x()} ),

                                key: async (): void => {x()},
                                key: ( async (): void => {x()} ),
                                key: ( async (): (void) => {x()} ),

                                key: async (arg: t): void => {x()},
                                key: ( async (arg: t): void => {x()} ),
                                key: ( async (arg: t): (void) => {x()} ),

                                key: async (arg: t, arg2: t): void => {x()},
                                key: ( async (arg: t, arg2: t): void => {x()} ),
                                key: ( async (arg: t, arg2: t): (void) => {x()} ),
                            }
                        ",
            "
                            const test = {
                                key(): void {x()},
                                key(): void {x()},
                                key(): (void) {x()},

                                key(arg: t): void {x()},
                                key(arg: t): void {x()},
                                key(arg: t): (void) {x()},

                                key(arg: t, arg2: t): void {x()},
                                key(arg: t, arg2: t): void {x()},
                                key(arg: t, arg2: t): (void) {x()},

                                async key(): void {x()},
                                async key(): void {x()},
                                async key(): (void) {x()},

                                async key(arg: t): void {x()},
                                async key(arg: t): void {x()},
                                async key(arg: t): (void) {x()},

                                async key(arg: t, arg2: t): void {x()},
                                async key(arg: t, arg2: t): void {x()},
                                async key(arg: t, arg2: t): (void) {x()},
                            }
                        ",
            Some(serde_json::json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
    ];

    Tester::new(ObjectShorthand::NAME, ObjectShorthand::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
