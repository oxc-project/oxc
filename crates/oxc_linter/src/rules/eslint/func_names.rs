use oxc_ast::ast::{
    AssignmentTarget, AssignmentTargetProperty, BindingPatternKind, Expression, Function,
    FunctionType, PropertyKind,
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

    match parent_node.unwrap().kind() {
        AstKind::MethodDefinition(_) => true,
        AstKind::ObjectProperty(property) => {
            property.method
                || property.kind == PropertyKind::Get
                || property.kind == PropertyKind::Set
        }

        _ => false,
    }
}
/**
 * Determines whether the current FunctionExpression node has a name that would be
 * inferred from context in a conforming ES6 environment.
 */
fn has_inferred_name(function: &Function, parent_node: Option<&AstNode>) -> bool {
    if is_object_or_class_method(parent_node) {
        return true;
    }

    if parent_node.is_none() {
        return false;
    }

    match parent_node.unwrap().kind() {
        AstKind::VariableDeclarator(declarator) => {
            if let BindingPatternKind::BindingIdentifier(_) = declarator.id.kind {
                if let Expression::FunctionExpression(function_expression) =
                    declarator.init.as_ref().unwrap()
                {
                    return get_function_identifier(function_expression)
                        == get_function_identifier(function);
                }
            }

            false
        }
        AstKind::ObjectProperty(property) => {
            if let Expression::FunctionExpression(function_expression) = &property.value {
                return get_function_identifier(function_expression)
                    == get_function_identifier(function);
            }

            false
        }
        AstKind::PropertyDefinition(definition) => {
            if let Expression::FunctionExpression(function_expression) =
                definition.value.as_ref().unwrap()
            {
                return get_function_identifier(function_expression)
                    == get_function_identifier(function);
            }

            false
        }
        AstKind::AssignmentExpression(expression) => {
            if let AssignmentTarget::AssignmentTargetIdentifier(_) = expression.left {
                if let Expression::FunctionExpression(function_expression) = &expression.right {
                    return get_function_identifier(function_expression)
                        == get_function_identifier(function);
                }
            }

            false
        }
        AstKind::AssignmentTargetWithDefault(target) => {
            if let AssignmentTarget::AssignmentTargetIdentifier(_) = target.binding {
                if let Expression::FunctionExpression(function_expression) = &target.init {
                    return get_function_identifier(function_expression)
                        == get_function_identifier(function);
                }
            }

            false
        }
        AstKind::AssignmentPattern(pattern) => {
            if let BindingPatternKind::BindingIdentifier(_) = pattern.left.kind {
                if let Expression::FunctionExpression(function_expression) = &pattern.right {
                    return get_function_identifier(function_expression)
                        == get_function_identifier(function);
                }
            }

            false
        }
        AstKind::ObjectAssignmentTarget(target) => {
            for property in &target.properties {
                if let AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(identifier) =
                    property
                {
                    if let Expression::FunctionExpression(function_expression) =
                        identifier.init.as_ref().unwrap()
                    {
                        return get_function_identifier(function_expression)
                            == get_function_identifier(function);
                    }
                }
            }

            false
        }
        _ => false,
    }
}

fn get_function_identifier<'a>(func: &'a Function<'a>) -> Option<&'a Span> {
    if let Some(id) = &func.id {
        return Some(&id.span);
    }

    None
}

fn get_function_name<'a>(func: &'a Function<'a>) -> Option<&Atom<'a>> {
    if let Some(id) = &func.id {
        return Some(&id.name);
    }

    None
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
        let mut invalid_funcs: Vec<&Function> = vec![];

        for node in ctx.nodes().iter() {
            match node.kind() {
                // check function if it invalid, do not report it because maybe later the function is calling itself
                AstKind::Function(func) => {
                    let parent_node = ctx.nodes().parent_node(node.id());
                    let config =
                        if func.generator { &self.generators_config } else { &self.default_config };

                    if is_invalid_function(func, config, parent_node) {
                        invalid_funcs.push(func);
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
                            invalid_funcs.retain(|func| func.span != span);
                        }
                    }
                }
                _ => {}
            }
        }

        for func in &invalid_funcs {
            let func_name = get_function_name(func);

            if func_name.is_some() {
                ctx.diagnostic(
                    OxcDiagnostic::warn(format!("Unexpected named {:?}.", func_name.unwrap()))
                        .with_label(Span::new(
                            func.span.start,
                            func.id.clone().unwrap().span.end
                        )),
                );
            } else {
                ctx.diagnostic(OxcDiagnostic::warn("Unexpected unnamed.").with_label(Span::new(
                    func.span.start,
                    func.params.span.start
                )));
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
