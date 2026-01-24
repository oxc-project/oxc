use std::{collections::VecDeque, fmt::Debug};

use crate::{context::LintContext, rule::Rule};
use itertools::Either;
use lazy_regex::{Lazy, Regex, RegexBuilder, lazy_regex};
use oxc_ast::ast::{
    ArrowFunctionExpression, Expression, Function, ObjectExpression, ObjectProperty,
    ObjectPropertyKind, PropertyKind,
};
use oxc_ast_visit::{Visit, walk};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{ReferenceId, ScopeId};
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;
use schemars::JsonSchema;

fn expected_all_properties_shorthanded(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(object-shorthand): Expected shorthand for all properties.")
        .with_label(span)
}

fn expected_literal_method_longform(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "eslint(object-shorthand): Expected longform method syntax for string literal keys.",
    )
    .with_label(span)
}

fn expected_property_shorthand(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(object-shorthand): Expected property shorthand.").with_label(span)
}

fn expected_property_longform(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(object-shorthand): Expected longform property syntax.")
        .with_label(span)
}

fn expected_method_shorthand(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(object-shorthand): Expected method shorthand.").with_label(span)
}

fn expected_method_longform(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(object-shorthand): Expected longform method syntax.")
        .with_label(span)
}

fn unexpected_mix(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "eslint(object-shorthand): Unexpected mix of shorthand and non-shorthand properties.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ObjectShorthand(Box<ObjectShorthandConfig>);

#[derive(Debug, Default, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ObjectShorthandConfigJSON {
    apply_to_methods: bool,
    apply_to_properties: bool,
    apply_never: bool,
    apply_consistent: bool,
    apply_consistent_as_needed: bool,

    avoid_quotes: bool,
    ignore_constructors: bool,
    avoid_explicit_return_arrows: bool,
    methods_ignore_pattern: Option<String>,
}

impl ObjectShorthandConfigJSON {
    fn into_object_shorthand_config(self) -> ObjectShorthandConfig {
        ObjectShorthandConfig {
            apply_to_methods: self.apply_to_methods,
            apply_to_properties: self.apply_to_properties,
            apply_never: self.apply_never,
            apply_consistent: self.apply_consistent,
            apply_consistent_as_needed: self.apply_consistent_as_needed,
            avoid_quotes: self.avoid_quotes,
            ignore_constructors: self.ignore_constructors,
            avoid_explicit_return_arrows: self.avoid_explicit_return_arrows,
            methods_ignore_pattern: self
                .methods_ignore_pattern
                .and_then(|p| RegexBuilder::new(&format!(r"{p}")).build().ok()),
        }
    }
}

#[derive(Debug, Default, Clone)]
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

impl std::ops::Deref for ObjectShorthand {
    type Target = ObjectShorthandConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// doc: https://github.com/eslint/eslint/blob/main/docs/src/rules/object-shorthand.md
// code: https://github.com/eslint/eslint/blob/main/lib/rules/object-shorthand.js
// test: https://github.com/eslint/eslint/blob/main/tests/lib/rules/object-shorthand.js

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
    config = ObjectShorthandConfigJSON
);

impl Rule for ObjectShorthand {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let obj1 = value.get(0);
        let obj2 = value.get(1);

        let shorthand_type =
            obj1.and_then(serde_json::Value::as_str).map(ShorthandType::from).unwrap_or_default();

        Ok(Self(Box::new(
            ObjectShorthandConfigJSON {
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
                apply_consistent_as_needed: matches!(
                    shorthand_type,
                    ShorthandType::ConsistentAsNeeded
                ),

                avoid_quotes: obj2
                    .and_then(|v| v.get("avoidQuotes"))
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
                ignore_constructors: obj2
                    .and_then(|v| v.get("ignoreConstructors"))
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
                avoid_explicit_return_arrows: obj2
                    .and_then(|v| v.get("avoidExplicitReturnArrows"))
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
                methods_ignore_pattern: obj2
                    .and_then(|v| v.get("methodsIgnorePattern"))
                    .and_then(serde_json::Value::as_str)
                    .map(ToString::to_string),
            }
            .into_object_shorthand_config(),
        )))
    }

    fn run_once<'a>(&self, ctx: &LintContext<'a>) {
        let mut checker = ObjectShorthandChecker::new(self, ctx);
        walk::walk_program(&mut checker, ctx.semantic().nodes().program());
    }
}

struct ObjectShorthandChecker<'a, 'c> {
    rule: &'c ObjectShorthand,
    ctx: &'c LintContext<'a>,
    lexical_scope_stack: VecDeque<FxHashSet<ScopeId>>,
    arrows_with_lexical_identifiers: FxHashSet<ScopeId>,
    arguments_identifiers: FxHashSet<ReferenceId>,
}

impl<'a, 'c> ObjectShorthandChecker<'a, 'c> {
    fn new(rule: &'c ObjectShorthand, ctx: &'c LintContext<'a>) -> Self {
        let arguments_identifiers = ctx
            .scoping()
            .root_unresolved_references()
            .get("arguments")
            .map(|v| FxHashSet::from_iter(v.iter().map(|&id| id)))
            .unwrap_or_default();

        Self {
            rule,
            ctx,
            lexical_scope_stack: Default::default(),
            arrows_with_lexical_identifiers: Default::default(),
            arguments_identifiers,
        }
    }

    fn make_function_shorthand(
        &self,
        property: &ObjectProperty,
        fn_or_arrow_fn: Either<&Function, &ArrowFunctionExpression>,
    ) {
        let span = match fn_or_arrow_fn {
            Either::Left(func) => func.span(),
            Either::Right(func) => func.span(),
        };
        self.ctx.diagnostic_with_fix(expected_method_shorthand(span), |fixer| {
            let has_comment = self.ctx.semantic().has_comments_between(Span::new(
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
                Either::Right(func) => match func.r#async {
                    true => "async ",
                    false => "",
                },
            };

            let property_key_span = property.key.span();
            let key_text = if property.computed {
                let (Some(paren_start), Some(paren_end_offset)) = (
                    self.ctx.find_prev_token_from(property_key_span.start, "["),
                    self.ctx.find_next_token_from(property_key_span.end, "]"),
                ) else {
                    return fixer.noop();
                };
                self.ctx.source_range(Span::new(
                    paren_start,
                    property_key_span.end + paren_end_offset + 1,
                ))
            } else {
                self.ctx.source_range(property_key_span)
            };

            match fn_or_arrow_fn {
                Either::Left(func) => {
                    let next_token = if func.generator {
                        self.ctx
                            .find_next_token_from(property_key_span.end, "*")
                            .map(|offset| offset + "*".len() as u32)
                    } else {
                        self.ctx
                            .find_next_token_from(property_key_span.end, "function")
                            .map(|offset| offset + "function".len() as u32)
                    };
                    let Some(func_token) = next_token else {
                        return fixer.noop();
                    };
                    let body = self
                        .ctx
                        .source_range(Span::new(property_key_span.end + func_token, func.span.end));
                    let ret = format!("{key_prefix}{key_text}{body}");
                    fixer.replace(property.span, ret)
                }
                Either::Right(func) => {
                    let next_token = self
                        .ctx
                        .find_prev_token_from(func.body.span.start, "=>")
                        .map(|offset| offset + "=>".len() as u32);
                    let Some(arrow_token) = next_token else {
                        return fixer.noop();
                    };
                    let arrow_body = self.ctx.source_range(Span::new(
                        arrow_token,
                        property.value.without_parentheses().span().end,
                    ));
                    let old_param_text = self.ctx.source_range(Span::new(
                        func.params.span.start,
                        func.return_type
                            .as_ref()
                            .map(|p| p.span.end)
                            .unwrap_or(func.params.span.end),
                    ));
                    let should_add_parens = if func.r#async {
                        if let Some(async_token) =
                            self.ctx.find_next_token_from(func.span.start, "async")
                        {
                            if let Some(fist_param) = func.params.items.first() {
                                self.ctx
                                    .find_next_token_within(
                                        func.span.start + async_token,
                                        fist_param.span.start,
                                        "(",
                                    )
                                    .is_none()
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        if let Some(fist_param) = func.params.items.first() {
                            self.ctx
                                .find_next_token_within(func.span.start, fist_param.span.start, "(")
                                .is_none()
                        } else {
                            false
                        }
                    };
                    let new_param_text = if should_add_parens {
                        format!("({old_param_text})")
                    } else {
                        old_param_text.to_string()
                    };
                    let type_param = func
                        .type_parameters
                        .as_ref()
                        .map(|t| self.ctx.source_range(t.span()))
                        .unwrap_or("");
                    let ret =
                        format!("{key_prefix}{key_text}{type_param}{new_param_text}{arrow_body}");
                    fixer.replace(property.span, ret)
                }
            }
        })
    }

    fn make_function_long_form(&self, property: &ObjectProperty) {
        let diagnostic = if self.rule.apply_never {
            expected_method_longform(property.span)
        } else {
            expected_literal_method_longform(property.span)
        };
        self.ctx.diagnostic_with_fix(diagnostic, |fixer| {
            let property_key_span = property.key.span();
            let key_text_range = if property.computed {
                let (Some(paren_start), Some(paren_end_offset)) = (
                    self.ctx.find_prev_token_from(property_key_span.start, "["),
                    self.ctx.find_next_token_from(property_key_span.end, "]"),
                ) else {
                    return fixer.noop();
                };
                Span::new(paren_start, property_key_span.end + paren_end_offset + 1)
            } else {
                property_key_span
            };
            let key_text = self.ctx.source_range(key_text_range);

            let Expression::FunctionExpression(func) = &property.value.without_parentheses() else {
                return fixer.noop();
            };
            let function_header = match (func.r#async, func.generator) {
                (true, true) => "async function*",
                (true, false) => "async function",
                (false, true) => "function*",
                (false, false) => "function",
            };

            // always include async and * in replace range
            let replace_range = Span::new(property.span.start, key_text_range.end);
            fixer.replace(replace_range, format!("{key_text}: {function_header}"))
        });
    }

    fn check_longform_methods(&self, property: &ObjectProperty) {
        if self.rule.ignore_constructors
            && property.key.is_identifier()
            && property.key.name().map(is_constructor).unwrap_or(false)
        {
            return;
        }
        if let (Some(pattern), Some(static_name)) =
            (self.rule.methods_ignore_pattern.as_ref(), property.key.static_name())
        {
            if pattern.is_match(static_name.as_ref()) {
                return;
            }
        }

        let is_key_string_literal = is_property_key_string_literal(property);
        if self.rule.avoid_quotes && is_key_string_literal {
            return;
        }

        if let Expression::FunctionExpression(func) = &property.value.without_parentheses() {
            self.make_function_shorthand(property, Either::Left(func));
        }

        if self.rule.avoid_explicit_return_arrows {
            if let Expression::ArrowFunctionExpression(func) = &property.value.without_parentheses()
                && !self.arrows_with_lexical_identifiers.contains(&func.scope_id())
            {
                if !func.expression {
                    self.make_function_shorthand(property, Either::Right(func));
                }
            }
        }
    }

    fn check_shorthand_properties(&self, property: &ObjectProperty) {
        if let Some(property_name) = property.key.name() {
            self.ctx.diagnostic_with_fix(expected_property_longform(property.span), |fixer| {
                fixer.replace(
                    property.span,
                    property_name.to_string() + ": " + &property_name.to_string(),
                )
            });
        }
    }

    fn check_longform_properties(&self, property: &ObjectProperty) {
        if self.rule.avoid_quotes && is_property_key_string_literal(property) {
            return;
        }

        let Expression::Identifier(value_identifier) = &property.value.without_parentheses() else {
            return;
        };

        if self.ctx.comments().iter().any(|comment| {
            if !property.span.contains_inclusive(comment.span) {
                return false;
            }
            // eslint checks `@type`
            // https://github.com/eslint/eslint/blob/f9c3e7adf7550441341e05dc60ab23bd5307d568/lib/rules/object-shorthand.js#L592
            comment.is_jsdoc() && self.ctx.source_range(comment.span).contains("@type")
        }) {
            return;
        }

        if let Some(property_name) = property.key.name() {
            if property_name == value_identifier.name {
                self.ctx.diagnostic_with_fix(expected_property_shorthand(property.span), |fixer| {
                    // x: /* */ x
                    // x: (/* */ x)
                    // "x": /* */ x
                    // "x": (/* */ x)
                    if self.ctx.semantic().has_comments_between(Span::new(
                        property.key.span().start,
                        value_identifier.span.end,
                    )) {
                        return fixer.noop();
                    }
                    fixer.replace(property.span, property_name.to_string())
                });
            }
        }
    }

    fn check_consistency(&self, obj_expr: &ObjectExpression, check_redundancy: bool) {
        let properties =
            obj_expr.properties.iter().filter_map(|property_kind| match property_kind {
                ObjectPropertyKind::ObjectProperty(property) => {
                    can_property_have_shorthand(property).then(|| property)
                }
                _ => None,
            });

        if properties.clone().count() > 0 {
            let shorthand_properties = properties.clone().filter(|p| is_shorthand_property(p));

            if shorthand_properties.clone().count() != properties.clone().count() {
                if shorthand_properties.count() > 0 {
                    self.ctx.diagnostic(unexpected_mix(obj_expr.span));
                } else if check_redundancy {
                    if properties.clone().all(|p| is_redundant_property(p)) {
                        self.ctx.diagnostic(expected_all_properties_shorthanded(obj_expr.span));
                    }
                }
            }
        }
    }

    fn enter_function(&mut self) {
        self.lexical_scope_stack.push_front(FxHashSet::default());
    }

    fn exit_function(&mut self) {
        self.lexical_scope_stack.pop_front();
    }

    fn report_lexical_identifier(&mut self) {
        let Some(scope) = self.lexical_scope_stack.iter().nth(0) else { return };
        scope.iter().for_each(|item| {
            self.arrows_with_lexical_identifiers.insert(item.clone());
        });
    }
}

impl<'a> Visit<'a> for ObjectShorthandChecker<'a, '_> {
    fn visit_function(&mut self, it: &oxc_ast::ast::Function<'a>, flags: oxc_semantic::ScopeFlags) {
        self.enter_function();
        walk::walk_function(self, it, flags);
        self.exit_function();
    }

    fn visit_arrow_function_expression(&mut self, it: &oxc_ast::ast::ArrowFunctionExpression<'a>) {
        let scope_id = it.scope_id();
        if self.lexical_scope_stack.is_empty() {
            self.enter_function();
        }
        self.lexical_scope_stack.iter_mut().nth(0).map(|scope| scope.insert(scope_id));
        walk::walk_arrow_function_expression(self, it);
        self.lexical_scope_stack.iter_mut().nth(0).map(|scope| scope.remove(&scope_id));
    }

    fn visit_this_expression(&mut self, _it: &oxc_ast::ast::ThisExpression) {
        self.report_lexical_identifier();
    }

    fn visit_super(&mut self, _it: &oxc_ast::ast::Super) {
        self.report_lexical_identifier();
    }

    fn visit_meta_property(&mut self, it: &oxc_ast::ast::MetaProperty<'a>) {
        if it.meta.name == "new" && it.property.name == "target" {
            self.report_lexical_identifier();
        }
    }

    fn visit_identifier_reference(&mut self, it: &oxc_ast::ast::IdentifierReference<'a>) {
        if self.arguments_identifiers.contains(&it.reference_id()) {
            self.report_lexical_identifier();
        }
    }

    fn visit_object_expression(&mut self, it: &ObjectExpression<'a>) {
        if self.rule.apply_consistent {
            self.check_consistency(it, false);
        } else if self.rule.apply_consistent_as_needed {
            self.check_consistency(it, true);
        }
        walk::walk_object_expression(self, it);
    }

    fn visit_object_property(&mut self, it: &ObjectProperty<'a>) {
        walk::walk_object_property(self, it);
        let property = it;
        let is_concise_property = property.shorthand || property.method;

        if !can_property_have_shorthand(property) {
            return;
        }

        if is_concise_property {
            if property.method
                && (self.rule.apply_never
                    || self.rule.avoid_quotes && is_property_key_string_literal(property))
            {
                // from { x() {} } to { x: function() {} }
                self.make_function_long_form(property);
            } else if self.rule.apply_never {
                // from { x } to { x: x }
                self.check_shorthand_properties(property);
            }
        } else if self.rule.apply_to_methods && is_property_value_anonymous_function(property) {
            // from { x: function() {} }   to { x() {} }
            // from { [x]: function() {} } to { [x]() {} }
            // from { x: () => {} }        to { x() {} }
            // from { [x]: () => {} }      to { [x]() {} }
            self.check_longform_methods(property);
        } else if self.rule.apply_to_properties {
            // from { x: x }   to { x }
            // from { "x": x } to { x }
            self.check_longform_properties(property);
        }
    }
}

#[derive(Debug, Default, Clone)]
enum ShorthandType {
    #[default]
    Always,
    Methods,
    Properties,
    Consistent,
    ConsistentAsNeeded,
    Never,
}

impl ShorthandType {
    pub fn from(raw: &str) -> Self {
        match raw {
            "methods" => Self::Methods,
            "properties" => Self::Properties,
            "consistent" => Self::Consistent,
            "consistent-as-needed" => Self::ConsistentAsNeeded,
            "never" => Self::Never,
            _ => Self::Always,
        }
    }
}

static CTOR_PREFIX_REGEX: Lazy<Regex> = lazy_regex!(r"[^_$0-9]");

/// Determines if the first character of the name
/// is a capital letter.
/// * `name` - The name of the node to evaluate.
/// Returns true if the first character of the property name is a capital letter, false if not.
fn is_constructor<N: AsRef<str>>(name: N) -> bool {
    // Not a constructor if name has no characters apart from '_', '$' and digits e.g. '_', '$$', '_8'
    let Some(matched) = CTOR_PREFIX_REGEX.find(name.as_ref()) else {
        return false;
    };

    name.as_ref().chars().nth(matched.start()).map(|ch| ch.is_uppercase()).unwrap_or(false)
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

    return true;
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

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
        // arrow functions are still alright by default
        ("var x = {y: (x)=>x}", None),
        ("doSomething({y: (x)=>x})", None),
        ("var x = {y: (x)=>x, y: a}", None),
        ("doSomething({x, y: (x)=>x})", None),
        ("({ foo: x => { return; }})", None),
        ("({ foo: (x) => { return; }})", None),
        ("({ foo: () => { return; }})", None),
        // getters and setters
        ("var x = {get y() {}}", None),
        ("var x = {set y(z) {}}", None),
        ("var x = {get y() {}, set y(z) {}}", None),
        ("doSomething({get y() {}})", None),
        ("doSomething({set y(z) {}})", None),
        ("doSomething({get y() {}, set y(z) {}})", None),
        // object literal computed properties
        ("var x = {[y]: y}", Some(json!(["properties"]))),
        ("var x = {['y']: 'y'}", Some(json!(["properties"]))),
        ("var x = {['y']: y}", Some(json!(["properties"]))),
        // object literal computed methods
        ("var x = {[y]() {}}", Some(json!(["methods"]))),
        ("var x = {[y]: function x() {}}", Some(json!(["methods"]))),
        ("var x = {[y]: y}", Some(json!(["methods"]))),
        // options
        ("var x = {y() {}}", Some(json!(["methods"]))),
        ("var x = {x, y() {}, a:b}", Some(json!(["methods"]))),
        ("var x = {y}", Some(json!(["properties"]))),
        ("var x = {y: {b}}", Some(json!(["properties"]))),
        // consistent
        ("var x = {a: a, b: b}", Some(json!(["consistent"]))),
        ("var x = {a: b, c: d, f: g}", Some(json!(["consistent"]))),
        ("var x = {a, b}", Some(json!(["consistent"]))),
        ("var x = {a, b, get test() { return 1; }}", Some(json!(["consistent"]))),
        ("var x = {foo, bar, ...baz}", Some(json!(["consistent"]))),
        ("var x = {bar: baz, ...qux}", Some(json!(["consistent"]))),
        ("var x = {...foo, bar: bar, baz: baz}", Some(json!(["consistent"]))),
        // consistent-as-needed
        ("var x = {...bar}", Some(json!(["consistent-as-needed"]))),
        ("var x = {a, b}", Some(json!(["consistent-as-needed"]))),
        ("var x = {a, b, get test(){return 1;}}", Some(json!(["consistent-as-needed"]))),
        ("var x = {0: 'foo'}", Some(json!(["consistent-as-needed"]))),
        ("var x = {'key': 'baz'}", Some(json!(["consistent-as-needed"]))),
        ("var x = {foo: 'foo'}", Some(json!(["consistent-as-needed"]))),
        ("var x = {[foo]: foo}", Some(json!(["consistent-as-needed"]))),
        ("var x = {foo: function foo() {}}", Some(json!(["consistent-as-needed"]))),
        ("var x = {[foo]: 'foo'}", Some(json!(["consistent-as-needed"]))),
        ("var x = {bar, ...baz}", Some(json!(["consistent-as-needed"]))),
        ("var x = {bar: baz, ...qux}", Some(json!(["consistent-as-needed"]))),
        ("var x = {...foo, bar, baz}", Some(json!(["consistent-as-needed"]))),
    ];

    let fail = vec![
        ("var x = {a: /* comment */ a}", None),
        ("var x = {a /* comment */: a}", None),
        ("var x = {a: (a /* comment */)}", None),
        ("var x = {'a': /* comment */ a}", None),
        ("var x = {'a' /* comment */: a}", None),
        ("var x = {'a': (a /* comment */)}", None),
        ("var x = {f: /* comment */ function() {}}", None),
        ("var x = {f /* comment */: function() {}}", None),
        ("var x = {a: a, b}", Some(json!(["consistent"]))),
        ("var x = {b, c: d, f: g}", Some(json!(["consistent"]))),
        ("var x = {foo, bar: baz, ...qux}", Some(json!(["consistent"]))),
        ("var x = {a: a, b: b}", Some(json!(["consistent-as-needed"]))),
        ("var x = {a, z: function z(){}}", Some(json!(["consistent-as-needed"]))),
        ("var x = {foo: function() {}}", Some(json!(["consistent-as-needed"]))),
        ("var x = {a: a, b: b, ...baz}", Some(json!(["consistent-as-needed"]))),
        ("var x = {foo, bar: bar, ...qux}", Some(json!(["consistent-as-needed"]))),
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
        // options
        ("var x = {y: function() {}}", "var x = {y() {}}", Some(json!(["methods"]))),
        (
            "var x = {x, y() {}, z: function() {}}",
            "var x = {x, y() {}, z() {}}",
            Some(json!(["methods"])),
        ),
        (
            "var x = {ConstructorFunction: function(){}, a: b}",
            "var x = {ConstructorFunction(){}, a: b}",
            Some(json!(["methods"])),
        ),
        ("var x = {[y]: function() {}}", "var x = {[y]() {}}", Some(json!(["methods"]))),
        ("({ [(foo)]: function() { return; } })", "({ [(foo)]() { return; } })", None),
        ("({ [(foo)]: async function() { return; } })", "({ async [(foo)]() { return; } })", None),
        (
            "({ [(((((((foo)))))))]: function() { return; } })",
            "({ [(((((((foo)))))))]() { return; } })",
            None,
        ),
        ("var x = {x: x}", "var x = {x}", Some(json!(["properties"]))),
        ("var x = {a, b, c(){}, x: x}", "var x = {a, b, c(){}, x}", Some(json!(["properties"]))),
        ("({ a: (function(){ return foo; }) })", "({ a(){ return foo; } })", None),
        ("({ a: async function*() {} })", "({ async *a() {} })", Some(json!(["always"]))),
        (
            "var x = {foo: foo, bar: baz, ...qux}",
            "var x = {foo, bar: baz, ...qux}",
            Some(json!(["always"])),
        ),
    ];

    Tester::new(ObjectShorthand::NAME, ObjectShorthand::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

#[test]
fn test_jsdoc() {
    use crate::tester::Tester;

    // JSDoc @type annotation
    let pass = vec![
        ("({ val: /** @type {number} */ (val) })", None),
        ("({ 'prop': /** @type {string} */ (prop) })", None),
        ("({ val: /**\n * @type {number}\n */ (val) })", None),
        ("({ val: /**\n  * @type {number}\n  */ (val) })", None),
        ("({ val: /**\n   * @type {number}\n   */ (val) })", None),
        ("({ val: /**\n\t* @type {number}\n\t*/ (val) })", None),
        ("({ val: /**\n\t * @type {number}\n\t */ (val) })", None),
        ("({ val: /**\n  *  @type   {number}  \n  */ (val) })", None),
        ("({ val: /**\n *  @type   {string} myParam\n */ (val) })", None),
        ("({ val: /**\n  *  @type   {Object} options\n  */ (val) })", None),
        ("({ val: /**\n\t *\t@type\t{Array}\n\t */ (val) })", None),
        (
            "({ val: /**\n   *\n   * @type {Function}\n   * @param {string} name\n   */ (val) })",
            None,
        ),
    ];

    let fail = vec![
        ("({ val: /** regular comment */ (val) })", None),
        ("({ val: /** @param {string} name */ (val) })", None),
        ("({ val: /** @returns {number} */ (val) })", None),
        ("({ val: /** @description some text */ (val) })", None),
        ("({ val: /**\n * @param {string} name\n */ (val) })", None),
        ("({ val: /**\n  * @returns {number}\n  */ (val) })", None),
        ("({ val: /**\n   * @description some text\n   */ (val) })", None),
        ("({ val: /**\n\t* @param {string} name\n\t*/ (val) })", None),
        ("({ val: /**\n\t * @returns {number}\n\t */ (val) })", None),
        ("({ val: /**\n  *  @param   {string}  name  \n  */ (val) })", None),
        ("({ val: /**\n *  @returns   {number} result\n */ (val) })", None),
        (
            "({ val: /**\n   *\n   * @param {string} name\n   * @returns {number}\n   */ (val) })",
            None,
        ),
    ];

    Tester::new(ObjectShorthand::NAME, ObjectShorthand::PLUGIN, pass, fail)
        .intentionally_allow_no_fix_tests()
        .test();
}

#[test]
fn test_never() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("var x = {a: n, c: d, f: g}", Some(json!(["never"]))),
        ("var x = {a: function(){}, b: {c: d}}", Some(json!(["never"]))),
        ("let {a, b} = o;", Some(json!(["never"]))),
        ("var x = {foo: foo, bar: bar, ...baz}", Some(json!(["never"]))),
    ];

    let fail = vec![];

    let fix = vec![
        ("var x = {y}", "var x = {y: y}", Some(json!(["never"]))),
        ("var x = {y: {x}}", "var x = {y: {x: x}}", Some(json!(["never"]))),
        (
            "var x = {foo, bar: baz, ...qux}",
            "var x = {foo: foo, bar: baz, ...qux}",
            Some(json!(["never"])),
        ),
        (
            "({ [(foo)]() { return; } })",
            "({ [(foo)]: function() { return; } })",
            Some(json!(["never"])),
        ),
        (
            "({ async [(foo)]() { return; } })",
            "({ [(foo)]: async function() { return; } })",
            Some(json!(["never"])),
        ),
        (
            "({ *[((foo))]() { return; } })",
            "({ [((foo))]: function*() { return; } })",
            Some(json!(["never"])),
        ),
        (
            "({ [(((((((foo)))))))]() { return; } })",
            "({ [(((((((foo)))))))]: function() { return; } })",
            Some(json!(["never"])),
        ),
        (
            "({ 'foo bar'() { return; } })",
            "({ 'foo bar': function() { return; } })",
            Some(json!(["never"])),
        ),
        ("({ *foo() { return; } })", "({ foo: function*() { return; } })", Some(json!(["never"]))),
        ("({ async* a() {} })", "({ a: async function*() {} })", Some(json!(["never"]))),
        (
            "({ async foo() { return; } })",
            "({ foo: async function() { return; } })",
            Some(json!(["never"])),
        ),
        (
            "({ *['foo bar']() { return; } })",
            "({ ['foo bar']: function*() { return; } })",
            Some(json!(["never"])),
        ),
        ("var x = {y() {}}", "var x = {y: function() {}}", Some(json!(["never"]))),
        ("var x = {*y() {}}", "var x = {y: function*() {}}", Some(json!(["never"]))),
        (
            "var x = {y, a: b, *x(){}}",
            "var x = {y: y, a: b, x: function*(){}}",
            Some(json!(["never"])),
        ),
        (
            "var x = {ConstructorFunction(){}, a: b}",
            "var x = {ConstructorFunction: function(){}, a: b}",
            Some(json!(["never"])),
        ),
        (
            "var x = {notConstructorFunction(){}, b: c}",
            "var x = {notConstructorFunction: function(){}, b: c}",
            Some(json!(["never"])),
        ),
    ];

    Tester::new(ObjectShorthand::NAME, ObjectShorthand::PLUGIN, pass, fail).expect_fix(fix).test();
}

#[test]
fn test_ignore_constructors() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        (
            "var x = {ConstructorFunction: function(){}, a: b}",
            Some(json!(["always", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_ConstructorFunction: function(){}, a: b}",
            Some(json!(["always", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {$ConstructorFunction: function(){}, a: b}",
            Some(json!(["always", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {__ConstructorFunction: function(){}, a: b}",
            Some(json!(["always", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_0ConstructorFunction: function(){}, a: b}",
            Some(json!(["always", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {notConstructorFunction(){}, b: c}",
            Some(json!(["always", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {ConstructorFunction: function(){}, a: b}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_ConstructorFunction: function(){}, a: b}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {$ConstructorFunction: function(){}, a: b}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {__ConstructorFunction: function(){}, a: b}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_0ConstructorFunction: function(){}, a: b}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {notConstructorFunction(){}, b: c}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        ("({ [foo.bar]: () => {} })", Some(json!(["always", { "ignoreConstructors": true }]))),
        ("var x = {ConstructorFunction: function(){}, a: b}", Some(json!(["never"]))),
        ("var x = {notConstructorFunction: function(){}, b: c}", Some(json!(["never"]))),
    ];

    let fix = vec![
        (
            "var x = {y: function() {}}",
            "var x = {y() {}}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_y: function() {}}",
            "var x = {_y() {}}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {$y: function() {}}",
            "var x = {$y() {}}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {__y: function() {}}",
            "var x = {__y() {}}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
        (
            "var x = {_0y: function() {}}",
            "var x = {_0y() {}}",
            Some(json!(["methods", { "ignoreConstructors": true }])),
        ),
    ];

    Tester::new(ObjectShorthand::NAME, ObjectShorthand::PLUGIN, pass, vec![])
        .expect_fix(fix)
        .with_snapshot_suffix("ignore-constructors")
        .test_and_snapshot();
}

#[test]
fn test_methods_ignore_pattern() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        (
            "var x = { foo: function() {}  }",
            Some(json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { foo: function() {}  }",
            Some(json!(["methods", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { foo: function*() {}  }",
            Some(json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { foo: async function() {}  }",
            Some(json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { foo: () => { return 5; }  }",
            Some(
                json!(["always", { "methodsIgnorePattern": "^foo$", "avoidExplicitReturnArrows": true }]),
            ),
        ),
        (
            "var x = { 'foo': function() {}  }",
            Some(json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { ['foo']: function() {}  }",
            Some(json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { 123: function() {}  }",
            Some(json!(["always", { "methodsIgnorePattern": "^123$" }])),
        ),
        (
            "var x = { afoob: function() {}  }",
            Some(json!(["always", { "methodsIgnorePattern": "foo" }])),
        ),
        (
            "var x = { afoob: function() {}  }",
            Some(json!(["always", { "methodsIgnorePattern": "^.foo.$" }])),
        ),
        (
            "var x = { '👍foo👍': function() {}  }",
            Some(json!(["always", { "methodsIgnorePattern": "^.foo.$" }])),
        ),
    ];

    let fix = vec![
        (
            "var x = { afoob: function() {} }",
            "var x = { afoob() {} }",
            Some(json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { afoob: function() {} }",
            "var x = { afoob() {} }",
            Some(json!(["methods", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { 'afoob': function() {} }",
            "var x = { 'afoob'() {} }",
            Some(json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
        (
            "var x = { 1234: function() {} }",
            "var x = { 1234() {} }",
            Some(json!(["always", { "methodsIgnorePattern": "^123$" }])),
        ),
        (
            "var x = { bar: function() {} }",
            "var x = { bar() {} }",
            Some(json!(["always", { "methodsIgnorePattern": "foo" }])),
        ),
        (
            "var x = { [foo]: function() {} }",
            "var x = { [foo]() {} }",
            Some(json!(["always", { "methodsIgnorePattern": "foo" }])),
        ),
        (
            "var x = { foo: foo }", // does not apply to properties
            "var x = { foo }",
            Some(json!(["always", { "methodsIgnorePattern": "^foo$" }])),
        ),
    ];

    Tester::new(ObjectShorthand::NAME, ObjectShorthand::PLUGIN, pass, vec![])
        .expect_fix(fix)
        .with_snapshot_suffix("ignore-pattern")
        .test_and_snapshot();
}

#[test]
fn test_avoid_quotes() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("var x = {'a': function(){}}", Some(json!(["always", { "avoidQuotes": true }]))),
        ("var x = {['a']: function(){}}", Some(json!(["methods", { "avoidQuotes": true }]))),
        ("var x = {'y': y}", Some(json!(["properties", { "avoidQuotes": true }]))),
    ];

    let fix = vec![
        ("var x = {a: a}", "var x = {a}", Some(json!(["always", { "avoidQuotes": true }]))),
        (
            "var x = {a: function(){}}",
            "var x = {a(){}}",
            Some(json!(["methods", { "avoidQuotes": true }])),
        ),
        (
            "var x = {[a]: function(){}}",
            "var x = {[a](){}}",
            Some(json!(["methods", { "avoidQuotes": true }])),
        ),
        (
            "var x = {'a'(){}}",
            "var x = {'a': function(){}}",
            Some(json!(["always", { "avoidQuotes": true }])),
        ),
        (
            "var x = {['a'](){}}",
            "var x = {['a']: function(){}}",
            Some(json!(["methods", { "avoidQuotes": true }])),
        ),
    ];

    Tester::new(ObjectShorthand::NAME, ObjectShorthand::PLUGIN, pass, vec![])
        .expect_fix(fix)
        .with_snapshot_suffix("avoid-quotes")
        .test_and_snapshot();
}

#[test]
fn test_avoid_explicit_return_arrows() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("({ x: () => foo })", Some(json!(["always", { "avoidExplicitReturnArrows": false }]))),
        (
            "({ x: () => { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": false }])),
        ),
        ("({ x: () => foo })", Some(json!(["always", { "avoidExplicitReturnArrows": true }]))),
        ("({ x() { return; } })", Some(json!(["always", { "avoidExplicitReturnArrows": true }]))),
        (
            "({ x() { return; }, y() { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x() { return; }, y: () => foo })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => foo, y() { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { this; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "function foo() { ({ x: () => { arguments; } }) }",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                    class Foo extends Bar {
                        constructor() {
                            var foo = { x: () => { super(); } };
                        }
                    }
                ",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                    class Foo extends Bar {
                        baz() {
                            var foo = { x: () => { super.baz(); } };
                        }
                    }
                ",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                    function foo() {
                        var x = { x: () => { new.target; } };
                    }
                ",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
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
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
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
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
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
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
    ];

    let fix = vec![
        (
            "({ x: (arg => { return; }) })",
            "({ x(arg) { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { return; } })",
            "({ x() { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x() { return; }, y: () => { return; } })",
            "({ x() { return; }, y() { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { return; }, y: () => foo })",
            "({ x() { return; }, y: () => foo })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { return; }, y: () => { return; } })",
            "({ x() { return; }, y() { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: foo => { return; } })",
            "({ x(foo) { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: (foo = 1) => { return; } })",
            "({ x(foo = 1) { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: ({ foo: bar = 1 } = {}) => { return; } })",
            "({ x({ foo: bar = 1 } = {}) { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { function foo() { this; } } })",
            "({ x() { function foo() { this; } } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { var foo = function() { arguments; } } })",
            "({ x() { var foo = function() { arguments; } } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ x: () => { function foo() { arguments; } } })",
            "({ x() { function foo() { arguments; } } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
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
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
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
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ 'foo bar': () => { return; } })",
            "({ 'foo bar'() { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ [foo]: () => { return; } })",
            "({ [foo]() { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: 1, foo: async (bar = 1) => { return; } })",
            "({ a: 1, async foo(bar = 1) { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ [ foo ]: async bar => { return; } })",
            "({ async [ foo ](bar) { return; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ key: (arg = () => {}) => {} })",
            "({ key(arg = () => {}) {} })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
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
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
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
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: (() => { return foo; }) })",
            "({ a() { return foo; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: ((arg) => { return foo; }) })",
            "({ a(arg) { return foo; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: ((arg, arg2) => { return foo; }) })",
            "({ a(arg, arg2) { return foo; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: (async () => { return foo; }) })",
            "({ async a() { return foo; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: (async (arg) => { return foo; }) })",
            "({ async a(arg) { return foo; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "({ a: (async (arg, arg2) => { return foo; }) })",
            "({ async a(arg, arg2) { return foo; } })",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
        (
            "
                    const test = {
                        key: <T,>(): void => { },
                        key: async <T,>(): Promise<void> => { },
                        key: <T,>(arg: T): T => { return arg },
                        key: async <T,>(arg: T): Promise<T> => { return arg },
                    }
                ",
            "
                    const test = {
                        key<T,>(): void { },
                        async key<T,>(): Promise<void> { },
                        key<T,>(arg: T): T { return arg },
                        async key<T,>(arg: T): Promise<T> { return arg },
                    }
                ",
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
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
            Some(json!(["always", { "avoidExplicitReturnArrows": true }])),
        ),
    ];

    Tester::new(ObjectShorthand::NAME, ObjectShorthand::PLUGIN, pass, vec![])
        .expect_fix(fix)
        .with_snapshot_suffix("avoid-explicit-return-arrows")
        .test_and_snapshot();
}
