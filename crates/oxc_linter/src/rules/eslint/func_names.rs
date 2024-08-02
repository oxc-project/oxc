use oxc_ast::ast::{
    AssignmentTarget, BindingPatternKind, Expression, Function, FunctionType, PropertyKind,
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
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    FuncNames,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details

    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix-dangerous', 'suggestion', and 'suggestion-dangerous'
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
        _ => false,
    }
}

fn get_function_identifier<'a>(func: &'a Function<'a>) -> Option<&Span> {
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

impl Rule for FuncNames {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj1 = value.get(0);
        let obj2: Option<&serde_json::Value> = value.get(1);

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
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::Function(func) = node.kind() {
            let parent_node = ctx.nodes().parent_node(node.id());

            let func_name = get_function_name(func);
            let config =
                if func.generator { &self.generators_config } else { &self.default_config };

            if *config == FuncNamesConfig::Never
                && func_name.is_some()
                && func.r#type != FunctionType::FunctionDeclaration
            {
                ctx.diagnostic(
                    OxcDiagnostic::warn(format!("Unexpected named {:?}.", func_name.unwrap()))
                        .with_label(Span::new(
                            func.span.start,
                            func.body.as_ref().unwrap().span.start,
                        )),
                );
            } else if (*config == FuncNamesConfig::AsNeeded
                && func_name.is_none()
                && !has_inferred_name(func, parent_node))
                // always
                || (*config == FuncNamesConfig::Always
                    && func_name.is_none()
                    && !is_object_or_class_method(parent_node))
            {
                ctx.diagnostic(OxcDiagnostic::warn("Unexpected unnamed.").with_label(Span::new(
                    func.span.start,
                    func.body.as_ref().unwrap().span.start,
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
        // ("({foo = function(){}} = {});", Some(serde_json::json!(["as-needed"]))), // { "ecmaVersion": 6 }, -- ToDo: expecting AstKind::AssignmentPattern, but getting AstKind::ObjectAssignmentTarget
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
