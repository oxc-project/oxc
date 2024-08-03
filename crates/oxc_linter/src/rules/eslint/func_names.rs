use oxc_ast::ast::{
    AssignmentTarget, AssignmentTargetProperty, BindingPatternKind, Expression, Function,
    FunctionType, MethodDefinitionKind, PropertyKey, PropertyKind,
};
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct FuncNames {
    default_config: FuncNamesConfig,
    generators_config: FuncNamesConfig,
}

#[derive(Debug, Default, Clone, PartialEq)]
enum FuncNamesConfig {
    #[default]
    Always,
    AsNeeded,
    Never,
}

impl FuncNamesConfig {
    pub fn from(raw: &str) -> Self {
        match raw {
            "always" => FuncNamesConfig::Always,
            "as-needed" => FuncNamesConfig::AsNeeded,
            "never" => FuncNamesConfig::Never,
            _ => FuncNamesConfig::default(),
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require or disallow named function expressions
    ///
    /// ### Why is this bad?
    ///
    /// If you leave off the function name then when the function throws an exception you are likely
    /// to get something similar to anonymous function in the stack trace. If you provide the optional
    /// name for a function expression then you will get the name of the function expression in the stack trace.
    ///
    /// /// ### Example
    ///
    /// ```javascript
    /// Foo.prototype.bar = function bar() {};
    /// ```
    FuncNames,
    style,
    pending
);
/**
 * Determines whether the current FunctionExpression node is a get, set, or
 * shorthand method in an object literal or a class.
 */
fn is_object_or_class_method(parent_node: Option<&AstNode>) -> bool {
    if parent_node.is_none() {
        return false;
    }

    let unwrapped_kind = parent_node.unwrap().kind();

    if matches!(unwrapped_kind, AstKind::MethodDefinition(_)) {
        return true;
    }

    if let AstKind::ObjectProperty(property) = unwrapped_kind {
        return property.method
            || property.kind == PropertyKind::Get
            || property.kind == PropertyKind::Set;
    }

    false
}
/**
 * Determines whether the current FunctionExpression node has a name that would be
 * inferred from context in a conforming ES6 environment.
 */
fn has_inferred_name(function: &Function, parent_node: Option<&AstNode>) -> bool {
    if is_object_or_class_method(parent_node) {
        return true;
    }

    // unwrap is safe because of is_object_or_class_method
    match parent_node.unwrap().kind() {
        AstKind::VariableDeclarator(declarator) => {
            matches!(declarator.id.kind, BindingPatternKind::BindingIdentifier(_))
                && matches!(declarator.init.as_ref().unwrap(), Expression::FunctionExpression(function_expression)
                        if get_function_identifier(function_expression) == get_function_identifier(function)
                )
        }
        AstKind::ObjectProperty(property) => {
            matches!(&property.value, Expression::FunctionExpression(function_expression)
                if get_function_identifier(function_expression) == get_function_identifier(function)
            )
        }
        AstKind::PropertyDefinition(definition) => {
            matches!(&definition.value.as_ref().unwrap(), Expression::FunctionExpression(function_expression)
                if get_function_identifier(function_expression) == get_function_identifier(function)
            )
        }
        AstKind::AssignmentExpression(expression) => {
            matches!(expression.left, AssignmentTarget::AssignmentTargetIdentifier(_))
                && matches!(&expression.right, Expression::FunctionExpression(function_expression)
                        if get_function_identifier(function_expression) == get_function_identifier(function)
                )
        }
        AstKind::AssignmentTargetWithDefault(target) => {
            matches!(target.binding, AssignmentTarget::AssignmentTargetIdentifier(_))
                && matches!(&target.init, Expression::FunctionExpression(function_expression)
                    if get_function_identifier(function_expression) == get_function_identifier(function)
                )
        }
        AstKind::AssignmentPattern(pattern) => {
            matches!(pattern.left.kind, BindingPatternKind::BindingIdentifier(_))
                && matches!(&pattern.right, Expression::FunctionExpression(function_expression)
                    if get_function_identifier(function_expression) == get_function_identifier(function)
                )
        }
        AstKind::ObjectAssignmentTarget(target) => {
            for property in &target.properties {
                if matches!(property, AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(identifier)
                    if matches!(identifier.init.as_ref().unwrap(), Expression::FunctionExpression(function_expression)
                        if get_function_identifier(function_expression) == get_function_identifier(function)
                    )
                ) {
                    return true;
                }
            }

            false
        }
        _ => false,
    }
}

/**
 * Gets the identifier for the function
 */
fn get_function_identifier<'a>(func: &'a Function<'a>) -> Option<&'a Span> {
    func.id.as_ref().map(|id| &id.span)
}

/**
 * Gets the identifier name of the function
 */
fn get_function_name<'a>(func: &'a Function<'a>) -> Option<&Atom<'a>> {
    func.id.as_ref().map(|id| &id.name)
}

fn get_property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<String> {
    if matches!(key, PropertyKey::NullLiteral(_)) {
        return Some("null".to_string());
    }
    
    match key {
        PropertyKey::RegExpLiteral(regex) => {
            Some(format!("/{}/{}", regex.regex.pattern, regex.regex.flags))
        }
        PropertyKey::BigIntLiteral(bigint) => Some(bigint.raw.to_string()),
        PropertyKey::TemplateLiteral(template) => {
            if template.expressions.len() == 0 && template.quasis.len() == 1 {
                if let Some(cooked) = &template.quasis[0].value.cooked {
                    return Some(cooked.to_string());
                }
            }

            None
        }
        _ => None,
    }
}

fn get_static_property_name<'a>(parent_node: Option<&'a AstNode<'a>>) -> Option<String> {
    parent_node?;

    let result_key = match parent_node.unwrap().kind() {
        AstKind::PropertyDefinition(definition) => Some((&definition.key, definition.computed)),
        AstKind::MethodDefinition(method_definition) => {
            Some((&method_definition.key, method_definition.computed))
        }
        AstKind::ObjectProperty(property) => Some((&property.key, property.computed)),
        _ => None,
    };

    result_key?;

    let prop = result_key.unwrap().0;

    if prop.is_identifier() && !result_key.unwrap().1 {
        prop.name()?;

        return Some(prop.name().unwrap().to_string());
    }

    get_property_key_name(prop)
}

/**
 * Gets the name and kind of the given function node.
 * @see https://github.com/eslint/eslint/blob/48117b27e98639ffe7e78a230bfad9a93039fb7f/lib/rules/utils/ast-utils.js#L1762
 */
fn get_function_name_with_kind(func: &Function, parent_node: Option<&AstNode>) -> String {
    let mut tokens: Vec<String> = vec![];

    if parent_node.is_some() {
        match parent_node.unwrap().kind() {
            AstKind::MethodDefinition(definition) => {
                if definition.r#static {
                    tokens.push("static".to_owned());
                }

                if !definition.computed && definition.key.is_private_identifier() {
                    tokens.push("private".to_owned());
                }
            }
            AstKind::PropertyDefinition(definition) => {
                if definition.r#static {
                    tokens.push("static".to_owned());
                }

                if !definition.computed && definition.key.is_private_identifier() {
                    tokens.push("private".to_owned());
                }
            }
            _ => {}
        }
    }

    if func.r#async {
        tokens.push("async".to_owned());
    }

    if func.generator {
        tokens.push("generator".to_owned());
    }

    if parent_node.is_some() {
        let kind = parent_node.unwrap().kind();

        match kind {
            AstKind::MethodDefinition(method_definition) => match method_definition.kind {
                MethodDefinitionKind::Constructor => tokens.push("constructor".to_owned()),
                MethodDefinitionKind::Get => tokens.push("getter".to_owned()),
                MethodDefinitionKind::Set => tokens.push("setter".to_owned()),
                MethodDefinitionKind::Method => tokens.push("method".to_owned()),
            },
            AstKind::PropertyDefinition(_) => tokens.push("method".to_owned()),
            _ => {
                tokens.push("function".to_owned());
            }
        }

        match kind {
            AstKind::MethodDefinition(method_definition) => {
                if !method_definition.computed && method_definition.key.is_private_identifier() {
                    let name = method_definition.key.name();

                    if let Some(name) = name {
                        tokens.push(name.to_string());
                    }
                }
            }
            AstKind::PropertyDefinition(definition) => {
                if !definition.computed && definition.key.is_private_identifier() {
                    let name = definition.key.name();

                    if let Some(name) = name {
                        tokens.push(name.to_string());
                    }
                } else if let Some(static_name) = get_static_property_name(parent_node) {
                    tokens.push(static_name);
                } else if let Some(name) = get_function_name(func) {
                    tokens.push(name.to_string());
                }
            }
            _ => {
                if let Some(static_name) = get_static_property_name(parent_node) {
                    tokens.push(static_name);
                } else if let Some(name) = get_function_name(func) {
                    tokens.push(name.to_string());
                }
            }
        }
    }

    tokens.join(" ")
}

fn is_invalid_function(
    func: &Function,
    config: &FuncNamesConfig,
    parent_node: Option<&AstNode>,
) -> bool {
    let func_name = get_function_name(func);

    if
    // never
    (*config == FuncNamesConfig::Never
        && func_name.is_some()
        && func.r#type != FunctionType::FunctionDeclaration)
        // as needed
    || (*config == FuncNamesConfig::AsNeeded
        && func_name.is_none()
        && !has_inferred_name(func, parent_node))
        // always
    || (*config == FuncNamesConfig::Always
        && func_name.is_none()
        && !is_object_or_class_method(parent_node))
    {
        return true;
    }

    false
}

impl Rule for FuncNames {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj1 = value.get(0);
        let obj2 = value.get(1);

        let default_config =
            obj1.and_then(serde_json::Value::as_str).map(FuncNamesConfig::from).unwrap_or_default();

        return Self {
            default_config: default_config.clone(),

            generators_config: obj2
                .and_then(|v| v.get("generators"))
                .and_then(serde_json::Value::as_str)
                .map(FuncNamesConfig::from)
                .unwrap_or(default_config),
        };
    }
    fn run_once(&self, ctx: &LintContext<'_>) {
        let mut invalid_funcs: Vec<(&Function, Option<&AstNode>)> = vec![];

        for node in ctx.nodes().iter() {
            match node.kind() {
                // check function if it invalid, do not report it because maybe later the function is calling itself
                AstKind::Function(func) => {
                    let parent_node = ctx.nodes().parent_node(node.id());
                    let config =
                        if func.generator { &self.generators_config } else { &self.default_config };

                    if is_invalid_function(func, config, parent_node) {
                        invalid_funcs.push((func, parent_node));
                    }
                }

                // check if the calling function is inside of its own body
                // when yes remove it from invalid_funcs because recursion are always named
                AstKind::CallExpression(expression) => {
                    if let Expression::Identifier(identifier) = &expression.callee {
                        let ast_span = ctx.nodes().iter_parents(node.id()).skip(1).find_map(|p| {
                            match p.kind() {
                                AstKind::Function(func) => {
                                    let func_name = get_function_name(func);

                                    func_name?;

                                    if *func_name.unwrap() == identifier.name {
                                        return Some(func.span);
                                    }

                                    None
                                }
                                _ => None,
                            }
                        });

                        if let Some(span) = ast_span {
                            invalid_funcs.retain(|(func, _)| func.span != span);
                        }
                    }
                }
                _ => {}
            }
        }

        for (func, parent_node) in &invalid_funcs {
            let func_name = get_function_name(func);
            let func_name_complete = get_function_name_with_kind(func, *parent_node);

            if func_name.is_some() {
                ctx.diagnostic(
                    OxcDiagnostic::warn(format!("Unexpected named {func_name_complete}."))
                        .with_label(Span::new(func.span.start, func.params.span.start)),
                );
            } else {
                ctx.diagnostic(
                    OxcDiagnostic::warn(format!("Unexpected unnamed {func_name_complete}."))
                        .with_label(Span::new(func.span.start, func.params.span.start)),
                );
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("Foo.prototype.bar = function bar(){};", None),
        ("Foo.prototype.bar = () => {}", None), // { "ecmaVersion": 6 },
        ("function foo(){}", None),
        ("function test(d, e, f) {}", None),
        ("new function bar(){}", None),
        ("exports = { get foo() { return 1; }, set bar(val) { return val; } };", None),
        ("({ foo() { return 1; } });", None), // { "ecmaVersion": 6 },
        ("class A { constructor(){} foo(){} get bar(){} set baz(value){} static qux(){}}", None), // { "ecmaVersion": 6 },
        ("function foo() {}", Some(serde_json::json!(["always"]))),
        ("var a = function foo() {};", Some(serde_json::json!(["always"]))),
        (
            "class A { constructor(){} foo(){} get bar(){} set baz(value){} static qux(){}}",
            Some(serde_json::json!(["as-needed"])),
        ), // { "ecmaVersion": 6 },
        ("({ foo() {} });", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 6 },
        ("var foo = function(){};", Some(serde_json::json!(["as-needed"]))),
        ("({foo: function(){}});", Some(serde_json::json!(["as-needed"]))),
        ("(foo = function(){});", Some(serde_json::json!(["as-needed"]))),
        ("({foo = function(){}} = {});", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 6 },
        ("({key: foo = function(){}} = {});", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 6 },
        ("[foo = function(){}] = [];", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 6 },
        ("function fn(foo = function(){}) {}", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 6 },
        ("function foo() {}", Some(serde_json::json!(["never"]))),
        ("var a = function() {};", Some(serde_json::json!(["never"]))),
        ("var a = function foo() { foo(); };", Some(serde_json::json!(["never"]))),
        ("var foo = {bar: function() {}};", Some(serde_json::json!(["never"]))),
        ("$('#foo').click(function() {});", Some(serde_json::json!(["never"]))),
        ("Foo.prototype.bar = function() {};", Some(serde_json::json!(["never"]))),
        (
            "class A { constructor(){} foo(){} get bar(){} set baz(value){} static qux(){}}",
            Some(serde_json::json!(["never"])),
        ), // { "ecmaVersion": 6 },
        ("({ foo() {} });", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 },
        ("export default function foo() {}", Some(serde_json::json!(["always"]))), // { "sourceType": "module", "ecmaVersion": 6 },
        ("export default function foo() {}", Some(serde_json::json!(["as-needed"]))), // { "sourceType": "module", "ecmaVersion": 6 },
        ("export default function foo() {}", Some(serde_json::json!(["never"]))), // { "sourceType": "module", "ecmaVersion": 6 },
        ("export default function() {}", Some(serde_json::json!(["never"]))), // { "sourceType": "module", "ecmaVersion": 6 },
        ("var foo = bar(function *baz() {});", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *baz() {});",
            Some(serde_json::json!(["always", { "generators": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *baz() {});",
            Some(serde_json::json!(["always", { "generators": "as-needed" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = function*() {};",
            Some(serde_json::json!(["always", { "generators": "as-needed" }])),
        ), // { "ecmaVersion": 6 },
        ("var foo = bar(function *baz() {});", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 6 },
        ("var foo = function*() {};", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *baz() {});",
            Some(serde_json::json!(["as-needed", { "generators": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *baz() {});",
            Some(serde_json::json!(["as-needed", { "generators": "as-needed" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = function*() {};",
            Some(serde_json::json!(["as-needed", { "generators": "as-needed" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *baz() {});",
            Some(serde_json::json!(["never", { "generators": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *baz() {});",
            Some(serde_json::json!(["never", { "generators": "as-needed" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = function*() {};",
            Some(serde_json::json!(["never", { "generators": "as-needed" }])),
        ), // { "ecmaVersion": 6 },
        ("var foo = bar(function *() {});", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 },
        ("var foo = function*() {};", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(serde_json::json!(["never"]))),        // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *() {});",
            Some(serde_json::json!(["never", { "generators": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = function*() {};",
            Some(serde_json::json!(["never", { "generators": "never" }])),
        ), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(serde_json::json!(["never", { "generators": "never" }]))), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *() {});",
            Some(serde_json::json!(["always", { "generators": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = function*() {};",
            Some(serde_json::json!(["always", { "generators": "never" }])),
        ), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(serde_json::json!(["always", { "generators": "never" }]))), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *() {});",
            Some(serde_json::json!(["as-needed", { "generators": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = function*() {};",
            Some(serde_json::json!(["as-needed", { "generators": "never" }])),
        ), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(serde_json::json!(["as-needed", { "generators": "never" }]))), // { "ecmaVersion": 6 },
        ("class C { foo = function() {}; }", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 2022 },
        ("class C { [foo] = function() {}; }", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 2022 },
        ("class C { #foo = function() {}; }", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 2022 }
    ];

    let fail = vec![
        ("Foo.prototype.bar = function() {};", None),
        ("(function(){}())", None),
        ("f(function(){})", None),
        ("var a = new Date(function() {});", None),
        ("var test = function(d, e, f) {};", None),
        ("new function() {}", None),
        ("Foo.prototype.bar = function() {};", Some(serde_json::json!(["as-needed"]))),
        ("(function(){}())", Some(serde_json::json!(["as-needed"]))),
        ("f(function(){})", Some(serde_json::json!(["as-needed"]))),
        ("var a = new Date(function() {});", Some(serde_json::json!(["as-needed"]))),
        ("new function() {}", Some(serde_json::json!(["as-needed"]))),
        ("var {foo} = function(){};", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 6 },
        // ("({key: foo = function(){}} = {});", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 6 },
        ("({ a: obj.prop = function(){} } = foo);", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 6 },
        ("[obj.prop = function(){}] = foo;", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 6 },
        ("var { a: [b] = function(){} } = foo;", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 6 },
        ("function foo({ a } = function(){}) {};", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 6 },
        ("var x = function foo() {};", Some(serde_json::json!(["never"]))),
        ("Foo.prototype.bar = function foo() {};", Some(serde_json::json!(["never"]))),
        ("({foo: function foo() {}})", Some(serde_json::json!(["never"]))),
        ("export default function() {}", Some(serde_json::json!(["always"]))), // { "sourceType": "module", "ecmaVersion": 6 },
        ("export default function() {}", Some(serde_json::json!(["as-needed"]))), // { "sourceType": "module", "ecmaVersion": 6 },
        ("export default (function(){});", Some(serde_json::json!(["as-needed"]))), // { "sourceType": "module", "ecmaVersion": 6 },
        ("var foo = bar(function *() {});", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 6 },
        ("var foo = function*() {};", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(serde_json::json!(["always"]))),        // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *() {});",
            Some(serde_json::json!(["always", { "generators": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = function*() {};",
            Some(serde_json::json!(["always", { "generators": "always" }])),
        ), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(serde_json::json!(["always", { "generators": "always" }]))), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *() {});",
            Some(serde_json::json!(["always", { "generators": "as-needed" }])),
        ), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(serde_json::json!(["always", { "generators": "as-needed" }]))), // { "ecmaVersion": 6 },
        ("var foo = bar(function *() {});", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *() {});",
            Some(serde_json::json!(["as-needed", { "generators": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = function*() {};",
            Some(serde_json::json!(["as-needed", { "generators": "always" }])),
        ), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(serde_json::json!(["as-needed", { "generators": "always" }]))), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *() {});",
            Some(serde_json::json!(["as-needed", { "generators": "as-needed" }])),
        ), // { "ecmaVersion": 6 },
        (
            "(function*() {}())",
            Some(serde_json::json!(["as-needed", { "generators": "as-needed" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *() {});",
            Some(serde_json::json!(["never", { "generators": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = function*() {};",
            Some(serde_json::json!(["never", { "generators": "always" }])),
        ), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(serde_json::json!(["never", { "generators": "always" }]))), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *() {});",
            Some(serde_json::json!(["never", { "generators": "as-needed" }])),
        ), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(serde_json::json!(["never", { "generators": "as-needed" }]))), // { "ecmaVersion": 6 },
        ("var foo = bar(function *baz() {});", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *baz() {});",
            Some(serde_json::json!(["never", { "generators": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *baz() {});",
            Some(serde_json::json!(["always", { "generators": "never" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *baz() {});",
            Some(serde_json::json!(["as-needed", { "generators": "never" }])),
        ), // { "ecmaVersion": 6 },
        ("class C { foo = function() {} }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { [foo] = function() {} }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { #foo = function() {} }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2022 },
        ("class C { foo = bar(function() {}) }", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 2022 },
        ("class C { foo = function bar() {} }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2022 }
    ];

    Tester::new(FuncNames::NAME, pass, fail).test_and_snapshot();
}
