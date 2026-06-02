use std::borrow::Cow;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de};
use serde_json::Value;

use oxc_ast::{
    AstKind,
    ast::{
        Argument, AssignmentTarget, CallExpression, Expression, Function, ObjectProperty,
        PropertyDefinition, PropertyKey,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::{identifier::is_identifier_name, keyword::is_reserved_keyword};

use crate::{
    AstNode,
    context::LintContext,
    rule::{Rule, TupleRuleConfig},
};

fn func_name_matching_diagnostic(
    name: &str,
    func_name: &str,
    is_property: bool,
    mode: FuncNameMatchingMode,
    span: Span,
) -> OxcDiagnostic {
    let msg = match (mode, is_property) {
        (FuncNameMatchingMode::Always, true) => {
            format!("Function name `{func_name}` should match property name `{name}`.")
        }
        (FuncNameMatchingMode::Always, false) => {
            format!("Function name `{func_name}` should match variable name `{name}`.")
        }
        (FuncNameMatchingMode::Never, true) => {
            format!("Function name `{func_name}` should not match property name `{name}`.")
        }
        (FuncNameMatchingMode::Never, false) => {
            format!("Function name `{func_name}` should not match variable name `{name}`.")
        }
    };

    OxcDiagnostic::warn(msg)
        .with_help("Rename the function or the variable/property so the names satisfy this rule.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum FuncNameMatchingMode {
    #[default]
    Always,
    Never,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FuncNameMatchingConfig {
    #[serde(default)]
    /// If `considerPropertyDescriptor` is set to `true`, the check will take into account the use of `Object.create`, `Object.defineProperty`, `Object.defineProperties`, and `Reflect.defineProperty`.
    consider_property_descriptor: bool,
    #[serde(default, rename = "includeCommonJSModuleExports")]
    /// If `includeCommonJSModuleExports` is set to `true`, `module.exports` and `module["exports"]` will be checked by this rule.
    include_common_js_module_exports: bool,
}

#[derive(Debug, Default, Clone, Serialize, JsonSchema)]
/// This rule takes an optional string of `"always"` or `"never"` (when omitted, it defaults to `"always"`), and an optional options object with two properties `considerPropertyDescriptor` and `includeCommonJSModuleExports`.
pub struct FuncNameMatching(FuncNameMatchingMode, FuncNameMatchingConfig);

impl<'de> Deserialize<'de> for FuncNameMatching {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let values = Vec::<Value>::deserialize(deserializer)?;
        let mut mode = FuncNameMatchingMode::Always;
        let mut options = FuncNameMatchingConfig::default();

        match values.as_slice() {
            [] => {}
            [first] if first.is_string() => {
                mode = serde_json::from_value(first.clone()).map_err(de::Error::custom)?;
            }
            [first] if first.is_object() => {
                options = serde_json::from_value(first.clone()).map_err(de::Error::custom)?;
            }
            [first, second] if first.is_string() && second.is_object() => {
                mode = serde_json::from_value(first.clone()).map_err(de::Error::custom)?;
                options = serde_json::from_value(second.clone()).map_err(de::Error::custom)?;
            }
            _ => {
                return Err(de::Error::custom(
                    r#"expected [], ["always"|"never"], [options], or ["always"|"never", options]"#,
                ));
            }
        }

        Ok(Self(mode, options))
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires function expression names to match the variable or property
    /// names they are assigned to, or disallows such matches with `"never"`.
    ///
    /// ### Why is this bad?
    ///
    /// Matching names keep stack traces and source code easier to connect.
    /// If a project prefers distinct names, the `"never"` option enforces that
    /// convention consistently.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /*eslint func-name-matching: "error"*/
    ///
    /// let foo = function bar() {};
    /// foo = function bar() {};
    /// const obj = {foo: function bar() {}};
    /// obj.foo = function bar() {};
    /// obj['foo'] = function bar() {};
    ///
    /// class C {
    ///     foo = function bar() {};
    /// }
    ///
    /// /*eslint func-name-matching: ["error", "never"] */
    ///
    /// let foo = function foo() {};
    /// foo = function foo() {};
    /// const obj = {foo: function foo() {}};
    /// obj.foo = function foo() {};
    /// obj['foo'] = function foo() {};
    ///
    /// class C {
    ///     foo = function foo() {};
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /*eslint func-name-matching: "error"*/
    /// // equivalent to /*eslint func-name-matching: ["error", "always"]*/
    ///
    /// const foo = function foo() {};
    /// const foo1 = function() {};
    /// const foo2 = () => {};
    /// foo = function foo() {};
    ///
    /// const obj = {foo: function foo() {}};
    /// obj.foo = function foo() {};
    /// obj['foo'] = function foo() {};
    ///
    /// const obj1 = {[foo]: function bar() {}};
    /// const obj3 = {foo: function() {}};
    ///
    /// obj['x' + 2] = function bar(){};
    /// const [ bar ] = [ function bar(){} ];
    ///
    /// class C {
    ///     foo = function foo() {};
    ///     baz = function() {};
    /// }
    ///
    /// // private names are ignored
    /// class D {
    ///     #foo = function foo() {};
    ///     #bar = function foo() {};
    ///     baz() {
    ///         this.#foo = function foo() {};
    ///         this.#foo = function bar() {};
    ///     }
    /// }
    ///
    /// module.exports = function foo(name) {};
    ///
    /// /*eslint func-name-matching: ["error", "never"] */
    ///
    /// let foo = function bar() {};
    /// const foo1 = function() {};
    /// const foo2 = () => {};
    /// foo = function bar() {};
    ///
    /// const obj = {foo: function bar() {}};
    /// obj.foo = function bar() {};
    /// obj['foo'] = function bar() {};
    ///
    /// const obj1 = {foo: function bar() {}};
    /// const obj2 = {[foo]: function foo() {}};
    /// const obj4 = {foo: function() {}};
    ///
    /// obj['x' + 2] = function bar(){};
    /// const [ bar ] = [ function bar(){} ];
    ///
    /// class C {
    ///     foo = function bar() {};
    ///     baz = function() {};
    /// }
    ///
    /// // private names are ignored
    /// class D {
    ///     #foo = function foo() {};
    ///     #bar = function foo() {};
    ///     baz() {
    ///         this.#foo = function foo() {};
    ///         this.#foo = function bar() {};
    ///     }
    /// }
    ///
    /// module.exports = function foo(name) {};
    /// ```
    FuncNameMatching,
    eslint,
    style,
    none,
    config = FuncNameMatching,
    version = "1.62.0",
);

impl Rule for FuncNameMatching {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<TupleRuleConfig<Self>>(value).map(TupleRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::VariableDeclarator(var_decl) => {
                let Some(init) = var_decl.init.as_ref() else {
                    return;
                };
                let Some(name) = var_decl.id.get_identifier_name().map(|name| name.as_str()) else {
                    return;
                };
                let Some(func_name) = function_expression_name(init) else { return };

                self.report_if_should_warn(name, &func_name, false, ctx);
            }
            AstKind::AssignmentExpression(assign_expr) => {
                let Some(func_name) = function_expression_name(&assign_expr.right) else { return };
                let Some((name, is_property)) = assignment_target_name(&assign_expr.left) else {
                    return;
                };

                if !self.1.include_common_js_module_exports && is_module_exports(&assign_expr.left)
                {
                    return;
                }

                if is_property && !is_valid_identifier(name) {
                    return;
                }

                self.report_if_should_warn(name, &func_name, is_property, ctx);
            }
            AstKind::ObjectProperty(property) => {
                self.check_object_property(property, node, ctx);
            }
            AstKind::PropertyDefinition(property_def) => {
                self.check_property_definition(property_def, ctx);
            }
            _ => {}
        }
    }
}

impl FuncNameMatching {
    fn should_warn(&self, name: &str, func_name: &str) -> bool {
        match self.0 {
            FuncNameMatchingMode::Always => name != func_name,
            FuncNameMatchingMode::Never => name == func_name,
        }
    }

    fn report_if_should_warn(
        &self,
        name: &str,
        func_name: &FunctionName<'_>,
        is_property: bool,
        ctx: &LintContext,
    ) {
        if self.should_warn(name, func_name.name) {
            ctx.diagnostic(func_name_matching_diagnostic(
                name,
                func_name.name,
                is_property,
                self.0,
                func_name.span,
            ));
        }
    }

    fn check_object_property<'a>(
        &self,
        property: &ObjectProperty<'a>,
        node: &AstNode<'a>,
        ctx: &LintContext<'a>,
    ) {
        let Some(function) = as_named_function(&property.value) else { return };
        let Some(function_name) = function_name(function) else { return };

        if property_key_is_identifier(&property.key) && !property.computed {
            let Some(property_name) = property.key.static_name() else { return };

            if self.1.consider_property_descriptor && property_name == "value" {
                match property_descriptor_name(node, ctx) {
                    DescriptorName::Name(descriptor_name) => {
                        self.report_if_should_warn(
                            descriptor_name.as_ref(),
                            &function_name,
                            true,
                            ctx,
                        );
                    }
                    DescriptorName::Unresolved => {}
                    DescriptorName::NotDescriptor => {
                        self.report_if_should_warn("value", &function_name, true, ctx);
                    }
                }
            } else {
                self.report_if_should_warn(property_name.as_ref(), &function_name, true, ctx);
            }

            return;
        }

        if let Some(property_name) = string_literal_key_name(&property.key)
            && is_valid_identifier(property_name)
        {
            self.report_if_should_warn(property_name, &function_name, true, ctx);
        }
    }

    fn check_property_definition<'a>(
        &self,
        property: &PropertyDefinition<'a>,
        ctx: &LintContext<'a>,
    ) {
        let Some(value) = property.value.as_ref() else { return };
        let Some(function_name) = function_expression_name(value) else { return };

        if property_key_is_identifier(&property.key) && !property.computed {
            let Some(property_name) = property.key.static_name() else { return };

            self.report_if_should_warn(property_name.as_ref(), &function_name, true, ctx);
            return;
        }

        if let Some(property_name) = string_literal_key_name(&property.key)
            && is_valid_identifier(property_name)
        {
            self.report_if_should_warn(property_name, &function_name, true, ctx);
        }
    }
}

fn property_descriptor_name<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> DescriptorName<'a> {
    let mut ancestors = ctx.nodes().ancestors(node.id());

    let Some(parent) = ancestors.next() else {
        return DescriptorName::NotDescriptor;
    };
    let AstKind::ObjectExpression(_) = parent.kind() else {
        return DescriptorName::NotDescriptor;
    };
    let Some(grandparent) = ancestors.next() else {
        return DescriptorName::NotDescriptor;
    };

    match grandparent.kind() {
        AstKind::CallExpression(call_expr)
            if is_property_call(call_expr, "Object", "defineProperty")
                || is_property_call(call_expr, "Reflect", "defineProperty") =>
        {
            call_expr
                .arguments
                .get(1)
                .and_then(string_literal_argument)
                .map_or(DescriptorName::Unresolved, |name| DescriptorName::Name(name.into()))
        }
        AstKind::ObjectProperty(descriptor_property) => {
            if descriptor_property.computed || !property_key_is_identifier(&descriptor_property.key)
            {
                return DescriptorName::Unresolved;
            }

            let Some(properties_object) = ancestors.next() else {
                return DescriptorName::NotDescriptor;
            };
            let AstKind::ObjectExpression(_) = properties_object.kind() else {
                return DescriptorName::NotDescriptor;
            };
            let Some(call) = ancestors.next() else {
                return DescriptorName::NotDescriptor;
            };
            let AstKind::CallExpression(call_expr) = call.kind() else {
                return DescriptorName::NotDescriptor;
            };

            if is_property_call(call_expr, "Object", "defineProperties")
                || is_property_call(call_expr, "Object", "create")
            {
                descriptor_property
                    .key
                    .static_name()
                    .map_or(DescriptorName::Unresolved, DescriptorName::Name)
            } else {
                DescriptorName::NotDescriptor
            }
        }
        _ => DescriptorName::NotDescriptor,
    }
}

enum DescriptorName<'a> {
    Name(Cow<'a, str>),
    Unresolved,
    NotDescriptor,
}

fn as_named_function<'a>(expr: &'a Expression<'a>) -> Option<&'a Function<'a>> {
    let Expression::FunctionExpression(function) = expr.without_parentheses() else {
        return None;
    };
    function.id.as_ref()?;
    Some(function)
}

struct FunctionName<'a> {
    name: &'a str,
    span: Span,
}

fn function_expression_name<'a>(expr: &'a Expression<'a>) -> Option<FunctionName<'a>> {
    as_named_function(expr).and_then(function_name)
}

fn function_name<'a>(function: &'a Function<'a>) -> Option<FunctionName<'a>> {
    let id = function.id.as_ref()?;
    Some(FunctionName { name: id.name.as_str(), span: id.span })
}

fn assignment_target_name<'a>(target: &'a AssignmentTarget<'a>) -> Option<(&'a str, bool)> {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(identifier) => {
            Some((identifier.name.as_str(), false))
        }
        target => target.as_member_expression()?.static_property_name().map(|name| (name, true)),
    }
}

fn is_module_exports(target: &AssignmentTarget) -> bool {
    let Some(member_expr) = target.as_member_expression() else {
        return false;
    };
    let Some(object) = member_expr.object().get_identifier_reference() else {
        return false;
    };

    object.name == "module" && member_expr.static_property_name() == Some("exports")
}

fn property_key_is_identifier(key: &PropertyKey) -> bool {
    matches!(key, PropertyKey::StaticIdentifier(_))
}

fn string_literal_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StringLiteral(lit) => Some(lit.value.as_str()),
        _ => None,
    }
}

fn string_literal_argument<'a>(argument: &'a Argument<'a>) -> Option<&'a str> {
    match argument.as_expression()?.without_parentheses() {
        Expression::StringLiteral(lit) => Some(lit.value.as_str()),
        _ => None,
    }
}

fn is_property_call(call_expr: &CallExpression, object: &str, property: &str) -> bool {
    call_expr.callee.is_specific_member_access(object, property)
}

fn is_valid_identifier(name: &str) -> bool {
    is_identifier_name(name) && !is_reserved_keyword(name)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var foo;", None),
        ("var foo = function foo() {};", None),
        ("var foo = function foo() {};", Some(serde_json::json!(["always"]))),
        ("var foo = function bar() {};", Some(serde_json::json!(["never"]))),
        ("var foo = function() {}", None),
        ("var foo = () => {}", None), // { "ecmaVersion": 6 },
        ("foo = function foo() {};", None),
        ("foo = function foo() {};", Some(serde_json::json!(["always"]))),
        ("foo = function bar() {};", Some(serde_json::json!(["never"]))),
        ("foo &&= function foo() {};", None), // { "ecmaVersion": 2021 },
        ("obj.foo ||= function foo() {};", None), // { "ecmaVersion": 2021 },
        ("obj['foo'] ??= function foo() {};", None), // { "ecmaVersion": 2021 },
        ("obj.foo = function foo() {};", None),
        ("obj.foo = function foo() {};", Some(serde_json::json!(["always"]))),
        ("obj.foo = function bar() {};", Some(serde_json::json!(["never"]))),
        ("obj.foo = function() {};", None),
        ("obj.foo = function() {};", Some(serde_json::json!(["always"]))),
        ("obj.foo = function() {};", Some(serde_json::json!(["never"]))),
        ("obj.bar.foo = function foo() {};", None),
        ("obj.bar.foo = function foo() {};", Some(serde_json::json!(["always"]))),
        ("obj.bar.foo = function baz() {};", Some(serde_json::json!(["never"]))),
        ("obj['foo'] = function foo() {};", None),
        ("obj['foo'] = function foo() {};", Some(serde_json::json!(["always"]))),
        ("obj['foo'] = function bar() {};", Some(serde_json::json!(["never"]))),
        ("obj['foo//bar'] = function foo() {};", None),
        ("obj['foo//bar'] = function foo() {};", Some(serde_json::json!(["always"]))),
        ("obj['foo//bar'] = function foo() {};", Some(serde_json::json!(["never"]))),
        ("obj[foo] = function bar() {};", None),
        ("obj[foo] = function bar() {};", Some(serde_json::json!(["always"]))),
        ("obj[foo] = function bar() {};", Some(serde_json::json!(["never"]))),
        ("var obj = {foo: function foo() {}};", None),
        ("var obj = {foo: function foo() {}};", Some(serde_json::json!(["always"]))),
        ("var obj = {foo: function bar() {}};", Some(serde_json::json!(["never"]))),
        ("var obj = {'foo': function foo() {}};", None),
        ("var obj = {'foo': function foo() {}};", Some(serde_json::json!(["always"]))),
        ("var obj = {'foo': function bar() {}};", Some(serde_json::json!(["never"]))),
        ("var obj = {'foo//bar': function foo() {}};", None),
        ("var obj = {'foo//bar': function foo() {}};", Some(serde_json::json!(["always"]))),
        ("var obj = {'foo//bar': function foo() {}};", Some(serde_json::json!(["never"]))),
        ("var obj = {foo: function() {}};", None),
        ("var obj = {foo: function() {}};", Some(serde_json::json!(["always"]))),
        ("var obj = {foo: function() {}};", Some(serde_json::json!(["never"]))),
        ("var obj = {[foo]: function bar() {}} ", None), // { "ecmaVersion": 6 },
        ("var obj = {['x' + 2]: function bar(){}};", None), // { "ecmaVersion": 6 },
        ("obj['x' + 2] = function bar(){};", None),
        ("var [ bar ] = [ function bar(){} ];", None), // { "ecmaVersion": 6 },
        ("function a(foo = function bar() {}) {}", None), // { "ecmaVersion": 6 },
        ("module.exports = function foo(name) {};", None),
        ("module['exports'] = function foo(name) {};", None),
        (
            "module.exports = function foo(name) {};",
            Some(serde_json::json!([{ "includeCommonJSModuleExports": false }])),
        ), // { "ecmaVersion": 6 },
        (
            "module.exports = function foo(name) {};",
            Some(serde_json::json!(["always", { "includeCommonJSModuleExports": false }])),
        ), // { "ecmaVersion": 6 },
        (
            "module.exports = function foo(name) {};",
            Some(serde_json::json!(["never", { "includeCommonJSModuleExports": false }])),
        ), // { "ecmaVersion": 6 },
        (
            "module['exports'] = function foo(name) {};",
            Some(serde_json::json!([{ "includeCommonJSModuleExports": false }])),
        ), // { "ecmaVersion": 6 },
        (
            "module['exports'] = function foo(name) {};",
            Some(serde_json::json!(["always", { "includeCommonJSModuleExports": false }])),
        ), // { "ecmaVersion": 6 },
        (
            "module['exports'] = function foo(name) {};",
            Some(serde_json::json!(["never", { "includeCommonJSModuleExports": false }])),
        ), // { "ecmaVersion": 6 },
        ("({['foo']: function foo() {}})", None), // { "ecmaVersion": 6 },
        ("({['foo']: function foo() {}})", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 6 },
        ("({['foo']: function bar() {}})", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 },
        ("({['❤']: function foo() {}})", None), // { "ecmaVersion": 6 },
        ("({[foo]: function bar() {}})", None), // { "ecmaVersion": 6 },
        ("({[null]: function foo() {}})", None), // { "ecmaVersion": 6 },
        ("({[1]: function foo() {}})", None),   // { "ecmaVersion": 6 },
        ("({[true]: function foo() {}})", None), // { "ecmaVersion": 6 },
        ("({[`x`]: function foo() {}})", None), // { "ecmaVersion": 6 },
        ("({[/abc/]: function foo() {}})", None), // { "ecmaVersion": 6 },
        ("({[[1, 2, 3]]: function foo() {}})", None), // { "ecmaVersion": 6 },
        ("({[{x: 1}]: function foo() {}})", None), // { "ecmaVersion": 6 },
        ("[] = function foo() {}", None),       // { "ecmaVersion": 6 },
        ("({} = function foo() {})", None),     // { "ecmaVersion": 6 },
        ("[a] = function foo() {}", None),      // { "ecmaVersion": 6 },
        ("({a} = function foo() {})", None),    // { "ecmaVersion": 6 },
        ("var [] = function foo() {}", None),   // { "ecmaVersion": 6 },
        ("var {} = function foo() {}", None),   // { "ecmaVersion": 6 },
        ("var [a] = function foo() {}", None),  // { "ecmaVersion": 6 },
        ("var {a} = function foo() {}", None),  // { "ecmaVersion": 6 },
        (
            "({ value: function value() {} })",
            Some(serde_json::json!([{ "considerPropertyDescriptor": true }])),
        ),
        (
            "obj.foo = function foo() {};",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ),
        (
            "obj.bar.foo = function foo() {};",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ),
        (
            "var obj = {foo: function foo() {}};",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ),
        (
            "var obj = {foo: function() {}};",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ),
        (
            "var obj = { value: function value() {} }",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ),
        (
            "Object.defineProperty(foo, 'bar', { value: function bar() {} })",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ),
        (
            "Object.defineProperties(foo, { bar: { value: function bar() {} } })",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ),
        (
            "Object.create(proto, { bar: { value: function bar() {} } })",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ),
        (
            "Object.defineProperty(foo, 'b' + 'ar', { value: function bar() {} })",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ),
        (
            "Object.defineProperties(foo, { ['bar']: { value: function bar() {} } })",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "Object.create(proto, { ['bar']: { value: function bar() {} } })",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "Object.defineProperty(foo, 'bar', { value() {} })",
            Some(serde_json::json!(["never", { "considerPropertyDescriptor": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "Object.defineProperties(foo, { bar: { value() {} } })",
            Some(serde_json::json!(["never", { "considerPropertyDescriptor": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "Object.create(proto, { bar: { value() {} } })",
            Some(serde_json::json!(["never", { "considerPropertyDescriptor": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "Reflect.defineProperty(foo, 'bar', { value: function bar() {} })",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ),
        (
            "Reflect.defineProperty(foo, 'b' + 'ar', { value: function baz() {} })",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ),
        (
            "Reflect.defineProperty(foo, 'bar', { value() {} })",
            Some(serde_json::json!(["never", { "considerPropertyDescriptor": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "foo({ value: function value() {} })",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ),
        ("class C { x = function () {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { x = function () {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { 'x' = function () {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { 'x' = function () {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { #x = function () {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { #x = function () {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { [x] = function () {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { [x] = function () {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { ['x'] = function () {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { ['x'] = function () {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { x = function x() {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { x = function y() {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { 'x' = function x() {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { 'x' = function y() {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { #x = function x() {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { #x = function x() {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { #x = function y() {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { #x = function y() {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { [x] = function x() {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { [x] = function x() {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { [x] = function y() {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { [x] = function y() {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { ['x'] = function x() {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { ['x'] = function y() {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { 'xy ' = function foo() {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { 'xy ' = function xy() {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { ['xy '] = function foo() {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { ['xy '] = function xy() {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { 1 = function x0() {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { 1 = function x1() {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { [1] = function x0() {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { [1] = function x1() {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { [f()] = function g() {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { [f()] = function f() {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { static x = function x() {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { static x = function y() {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { x = (function y() {})(); }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { x = (function x() {})(); }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("(class { x = function x() {}; })", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("(class { x = function y() {}; })", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        (
            "class C { #x; foo() { this.#x = function x() {}; } }",
            Some(serde_json::json!(["always"])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { #x; foo() { this.#x = function x() {}; } }",
            Some(serde_json::json!(["never"])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { #x; foo() { this.#x = function y() {}; } }",
            Some(serde_json::json!(["always"])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { #x; foo() { this.#x = function y() {}; } }",
            Some(serde_json::json!(["never"])),
        ), // { "ecmaVersion": 2022 },
        (
            "class C { #x; foo() { a.b.#x = function x() {}; } }",
            Some(serde_json::json!(["always"])),
        ), // { "ecmaVersion": 2022 },
        ("class C { #x; foo() { a.b.#x = function x() {}; } }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        (
            "class C { #x; foo() { a.b.#x = function y() {}; } }",
            Some(serde_json::json!(["always"])),
        ), // { "ecmaVersion": 2022 },
        ("class C { #x; foo() { a.b.#x = function y() {}; } }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
    ];

    let fail = vec![
        ("let foo = function bar() {};", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 6 },
        ("let foo = function bar() {};", None), // { "ecmaVersion": 6 },
        ("foo = function bar() {};", None),     // { "ecmaVersion": 6 },
        ("foo &&= function bar() {};", None),   // { "ecmaVersion": 2021 },
        ("obj.foo ||= function bar() {};", None), // { "ecmaVersion": 2021 },
        ("obj['foo'] ??= function bar() {};", None), // { "ecmaVersion": 2021 },
        ("obj.foo = function bar() {};", None), // { "ecmaVersion": 6 },
        ("obj.bar.foo = function bar() {};", None), // { "ecmaVersion": 6 },
        ("obj['foo'] = function bar() {};", None), // { "ecmaVersion": 6 },
        ("let obj = {foo: function bar() {}};", None), // { "ecmaVersion": 6 },
        ("let obj = {'foo': function bar() {}};", None), // { "ecmaVersion": 6 },
        ("({['foo']: function bar() {}})", None), // { "ecmaVersion": 6 },
        (
            "module.exports = function foo(name) {};",
            Some(serde_json::json!([{ "includeCommonJSModuleExports": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "module.exports = function foo(name) {};",
            Some(serde_json::json!(["always", { "includeCommonJSModuleExports": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "module.exports = function exports(name) {};",
            Some(serde_json::json!(["never", { "includeCommonJSModuleExports": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "module['exports'] = function foo(name) {};",
            Some(serde_json::json!([{ "includeCommonJSModuleExports": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "module['exports'] = function foo(name) {};",
            Some(serde_json::json!(["always", { "includeCommonJSModuleExports": true }])),
        ), // { "ecmaVersion": 6 },
        (
            "module['exports'] = function exports(name) {};",
            Some(serde_json::json!(["never", { "includeCommonJSModuleExports": true }])),
        ), // { "ecmaVersion": 6 },
        ("var foo = function foo(name) {};", Some(serde_json::json!(["never"]))),
        ("obj.foo = function foo(name) {};", Some(serde_json::json!(["never"]))),
        (
            "Object.defineProperty(foo, 'bar', { value: function baz() {} })",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ),
        (
            "Object.defineProperties(foo, { bar: { value: function baz() {} } })",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ),
        (
            "Object.create(proto, { bar: { value: function baz() {} } })",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ),
        (
            "var obj = { value: function foo(name) {} }",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ),
        (
            "Object.defineProperty(foo, 'bar', { value: function bar() {} })",
            Some(serde_json::json!(["never", { "considerPropertyDescriptor": true }])),
        ),
        (
            "Object.defineProperties(foo, { bar: { value: function bar() {} } })",
            Some(serde_json::json!(["never", { "considerPropertyDescriptor": true }])),
        ),
        (
            "Object.create(proto, { bar: { value: function bar() {} } })",
            Some(serde_json::json!(["never", { "considerPropertyDescriptor": true }])),
        ),
        (
            "Reflect.defineProperty(foo, 'bar', { value: function baz() {} })",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ),
        (
            "Reflect.defineProperty(foo, 'bar', { value: function bar() {} })",
            Some(serde_json::json!(["never", { "considerPropertyDescriptor": true }])),
        ),
        (
            "foo({ value: function bar() {} })",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ),
        ("(obj?.aaa).foo = function bar() {};", None), // { "ecmaVersion": 2020 },
        (
            "Object?.defineProperty(foo, 'bar', { value: function baz() {} })",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ), // { "ecmaVersion": 2020 },
        (
            "(Object?.defineProperty)(foo, 'bar', { value: function baz() {} })",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ), // { "ecmaVersion": 2020 },
        (
            "Object?.defineProperty(foo, 'bar', { value: function bar() {} })",
            Some(serde_json::json!(["never", { "considerPropertyDescriptor": true }])),
        ), // { "ecmaVersion": 2020 },
        (
            "(Object?.defineProperty)(foo, 'bar', { value: function bar() {} })",
            Some(serde_json::json!(["never", { "considerPropertyDescriptor": true }])),
        ), // { "ecmaVersion": 2020 },
        (
            "Object?.defineProperties(foo, { bar: { value: function baz() {} } })",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ), // { "ecmaVersion": 2020 },
        (
            "(Object?.defineProperties)(foo, { bar: { value: function baz() {} } })",
            Some(serde_json::json!(["always", { "considerPropertyDescriptor": true }])),
        ), // { "ecmaVersion": 2020 },
        (
            "Object?.defineProperties(foo, { bar: { value: function bar() {} } })",
            Some(serde_json::json!(["never", { "considerPropertyDescriptor": true }])),
        ), // { "ecmaVersion": 2020 },
        (
            "(Object?.defineProperties)(foo, { bar: { value: function bar() {} } })",
            Some(serde_json::json!(["never", { "considerPropertyDescriptor": true }])),
        ), // { "ecmaVersion": 2020 },
        ("class C { x = function y() {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { x = function x() {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { 'x' = function y() {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { 'x' = function x() {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { ['x'] = function y() {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { ['x'] = function x() {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("class C { static x = function y() {}; }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { static x = function x() {}; }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        ("(class { x = function y() {}; })", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("(class { x = function x() {}; })", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 },
        (r"var obj = { '\u1885': function foo() {} };", None), // { "ecmaVersion": 6 }
    ];

    Tester::new(FuncNameMatching::NAME, FuncNameMatching::PLUGIN, pass, fail).test_and_snapshot();
}
