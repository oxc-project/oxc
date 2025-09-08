use oxc_ast::{
    AstKind,
    ast::ObjectAssignmentTarget,
    ast::{
        AssignmentTarget, AssignmentTargetProperty, BindingPatternKind, Expression, Function,
        FunctionType, PropertyKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{Atom, GetSpan, Span};
use oxc_syntax::identifier::is_identifier_name;
use std::borrow::Cow;

use crate::fixer::{RuleFix, RuleFixer};
use crate::{AstNode, ast_util::get_function_name_with_kind, context::LintContext, rule::Rule};

fn named_diagnostic(function_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected named {function_name}."))
        .with_label(span)
        .with_help("Remove the name on this function expression.")
}

fn unnamed_diagnostic(inferred_name_or_description: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected unnamed {inferred_name_or_description}."))
        .with_label(span)
        .with_help("Consider giving this function expression a name.")
}

#[derive(Debug, Clone, Default)]
struct FuncNamesConfig {
    functions: FuncNamesConfigType,
    generators: FuncNamesConfigType,
}

#[derive(Debug, Default, Clone)]
pub struct FuncNames {
    config: FuncNamesConfig,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum FuncNamesConfigType {
    #[default]
    Always,
    AsNeeded,
    Never,
}

impl From<&serde_json::Value> for FuncNamesConfigType {
    fn from(raw: &serde_json::Value) -> Self {
        match raw.as_str() {
            Some("as-needed") => Self::AsNeeded,
            Some("never") => Self::Never,
            _ => Self::Always,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require or disallow named function expressions.
    ///
    /// ### Why is this bad?
    ///
    /// Leaving the name off a function will cause `<anonymous>` to appear in
    /// stack traces of errors thrown in it or any function called within it.
    /// This makes it more difficult to find where an error is thrown.
    /// Providing an explicit name also improves readability and consistency.
    ///
    /// ### Options
    ///
    /// First option:
    /// - Type: `string`
    /// - Default: `"always"`
    /// - Possible values:
    ///   - `"always"` - requires all function expressions to have a name.
    ///   - `"as-needed"` - requires a name only if one is not automatically inferred.
    ///   - `"never"` - disallows names for function expressions.
    ///
    /// Second option:
    /// - Type: `object`
    /// - Properties:
    ///   - `generators`: `("always" | "as-needed" | "never")` (default: falls back to first option)
    ///     - `"always"` - require named generator function expressions.
    ///     - `"as-needed"` - require a name only when not inferred.
    ///     - `"never"` - disallow names for generator function expressions.
    ///
    /// Example configuration:
    /// ```json
    /// {
    ///   "func-names": ["error", "as-needed", { "generators": "never" }]
    /// }
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /* func-names: ["error", "always"] */
    ///
    /// Foo.prototype.bar = function() {};
    /// const cat = { meow: function() {} };
    /// (function() { /* ... */ }());
    /// export default function() {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /* func-names: ["error", "always"] */
    ///
    /// Foo.prototype.bar = function bar() {};
    /// const cat = { meow() {} };
    /// (function bar() { /* ... */ }());
    /// export default function foo() {}
    /// ```
    ///
    /// #### `as-needed`
    ///
    /// Examples of **incorrect** code for this rule with the `"as-needed"` option:
    /// ```js
    /// /* func-names: ["error", "as-needed"] */
    ///
    /// Foo.prototype.bar = function() {};
    /// (function() { /* ... */ }());
    /// export default function() {}
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `"as-needed"` option:
    /// ```js
    /// /* func-names: ["error", "as-needed"] */
    ///
    /// const bar = function() {};
    /// const cat = { meow: function() {} };
    /// class C { #bar = function() {}; baz = function() {}; }
    /// quux ??= function() {};
    /// (function bar() { /* ... */ }());
    /// export default function foo() {}
    /// ```
    ///
    /// #### `never`
    ///
    /// Examples of **incorrect** code for this rule with the `"never"` option:
    /// ```js
    /// /* func-names: ["error", "never"] */
    ///
    /// Foo.prototype.bar = function bar() {};
    /// (function bar() { /* ... */ }());
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `"never"` option:
    /// ```js
    /// /* func-names: ["error", "never"] */
    ///
    /// Foo.prototype.bar = function() {};
    /// (function() { /* ... */ }());
    /// ```
    ///
    /// #### `generators`
    ///
    /// Examples of **incorrect** code for this rule with the `"always", { "generators": "as-needed" }` options:
    /// ```js
    /// /* func-names: ["error", "always", { "generators": "as-needed" }] */
    ///
    /// (function*() { /* ... */ }());
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `"always", { "generators": "as-needed" }` options:
    /// ```js
    /// /* func-names: ["error", "always", { "generators": "as-needed" }] */
    ///
    /// const foo = function*() {};
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the `"always", { "generators": "never" }` options:
    /// ```js
    /// /* func-names: ["error", "always", { "generators": "never" }] */
    ///
    /// const foo = bar(function *baz() {});
    /// ```
    /// Examples of **correct** code for this rule with the `"always", { "generators": "never" }` options:
    /// ```js
    /// /* func-names: ["error", "always", { "generators": "never" }] */
    ///
    /// const foo = bar(function *() {});
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the `"as-needed", { "generators": "never" }` options:
    /// ```js
    /// /* func-names: ["error", "as-needed", { "generators": "never" }] */
    ///
    /// const foo = bar(function *baz() {});
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `"as-needed", { "generators": "never" }` options:
    /// ```js
    /// /* func-names: ["error", "as-needed", { "generators": "never" }] */
    ///
    /// const foo = bar(function *() {});
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the `"never", { "generators": "always" }` options:
    /// ```js
    /// /* func-names: ["error", "never", { "generators": "always" }] */
    ///
    /// const foo = bar(function *() {});
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `"never", { "generators": "always" }` options:
    /// ```js
    /// /* func-names: ["error", "never", { "generators": "always" }] */
    ///
    /// const foo = bar(function *baz() {});
    /// ```
    FuncNames,
    eslint,
    style,
    conditional_fix_suggestion
);

impl FuncNames {
    fn get_invalid_functions<'a>(
        &self,
        ctx: &'a LintContext<'_>,
    ) -> Vec<(&'a Function<'a>, &'a AstNode<'a>, &'a AstNode<'a>)> {
        let mut invalid_functions: Vec<(&Function, &AstNode, &AstNode)> = Vec::new();

        for node in ctx.nodes() {
            match node.kind() {
                // check function if it invalid, do not report it because maybe later the function is calling itself
                AstKind::Function(func) => {
                    let parent_node = ctx.nodes().parent_node(node.id());
                    let config =
                        if func.generator { self.config.generators } else { self.config.functions };

                    if is_invalid_function(config, func, parent_node) {
                        invalid_functions.push((func, node, parent_node));
                    }
                }

                // check if the calling function is inside its own body
                // then, remove it from invalid_functions because recursion are always named
                AstKind::CallExpression(expression) => {
                    retain_recursive_function_from_invalid_functions(
                        &mut invalid_functions,
                        expression,
                        node,
                        ctx,
                    );
                }
                _ => {}
            }
        }

        invalid_functions
    }
}

impl Rule for FuncNames {
    fn from_configuration(value: serde_json::Value) -> Self {
        let Some(functions_config) = value.get(0) else {
            return Self::default();
        };
        let generators_config =
            value.get(1).and_then(|v| v.get("generators")).unwrap_or(functions_config);

        Self {
            config: FuncNamesConfig {
                functions: FuncNamesConfigType::from(functions_config),
                generators: FuncNamesConfigType::from(generators_config),
            },
        }
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        for (func, node, parent_node) in self.get_invalid_functions(ctx) {
            diagnostic_invalid_function(func, node, parent_node, ctx);
        }
    }
}

fn retain_recursive_function_from_invalid_functions(
    invalid_functions: &mut Vec<(&Function, &AstNode, &AstNode)>,
    expression: &oxc_ast::ast::CallExpression,
    node: &AstNode,
    ctx: &LintContext,
) {
    let Expression::Identifier(identifier) = &expression.callee else {
        return;
    };
    // check at first if the callee calls an invalid function
    if !invalid_functions
        .iter()
        .filter_map(|(func, _, _)| func.name())
        .any(|func_name| func_name == identifier.name)
    {
        return;
    }

    // a function which is calling itself inside is always valid
    if let Some(span) = ctx.nodes().ancestors(node.id()).find_map(|p| {
        if let AstKind::Function(func) = p.kind() {
            func.name().filter(|n| *n == identifier.name).map(|_| func.span)
        } else {
            None
        }
    }) {
        invalid_functions.retain(|(func, _, _)| func.span != span);
    }
}

const INVALID_IDENTIFIER_NAMES: [&str; 9] =
    ["arguments", "async", "await", "constructor", "default", "eval", "null", "undefined", "yield"];

fn diagnostic_invalid_function(
    func: &Function,
    node: &AstNode,
    parent_node: &AstNode,
    ctx: &LintContext,
) {
    let func_name_complete = get_function_name_with_kind(node, parent_node);
    let report_span = Span::new(func.span.start, func.params.span.start);

    if let Some(id) = func.id.as_ref() {
        ctx.diagnostic_with_suggestion(
            named_diagnostic(&func_name_complete, report_span),
            |fixer| fixer.delete(id),
        );
        return;
    }

    let replace_span = Span::new(
        func.span.start,
        func.type_parameters.as_ref().map_or_else(|| func.params.span.start, |tp| tp.span.start),
    );

    let function_name = guess_function_name(ctx, node.id()).map(Cow::into_owned);

    let is_safe_fix =
        function_name.as_ref().is_some_and(|name| can_safely_apply_fix(func, name, ctx));

    let msg = unnamed_diagnostic(&func_name_complete, report_span);

    ctx.diagnostic_with_fix(msg, |fixer| {
        apply_rule_fix(&fixer, is_safe_fix, replace_span, function_name)
    });
}

fn is_valid_identifier_name(name: &str) -> bool {
    !INVALID_IDENTIFIER_NAMES.contains(&name) && is_identifier_name(name)
}

/// Determines whether the current FunctionExpression node is a get, set, or
/// shorthand method in an object literal or a class.
fn is_object_or_class_method(parent_node: &AstNode) -> bool {
    match parent_node.kind() {
        AstKind::MethodDefinition(_) => true,
        AstKind::ObjectProperty(property) => {
            property.method || matches!(property.kind, PropertyKind::Get | PropertyKind::Set)
        }
        _ => false,
    }
}

fn does_object_assignment_target_have_name(target: &ObjectAssignmentTarget) -> bool {
    target.properties.iter().any(|property| {
        matches!(
            property,
            AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(identifier)
                if matches!(
                    identifier.init,
                    Some(Expression::FunctionExpression(_))
                )
        )
    })
}

/// Determines whether the current FunctionExpression node has a name that would be
/// inferred from context in a conforming ES6 environment.
fn has_inferred_name<'a>(function: &Function<'a>, parent_node: &AstNode<'a>) -> bool {
    if is_object_or_class_method(parent_node) {
        return true;
    }

    match parent_node.kind() {
        AstKind::VariableDeclarator(declarator) => {
            matches!(declarator.id.kind, BindingPatternKind::BindingIdentifier(_))
                && declarator.init.as_ref().is_some_and(|init| is_same_function(init, function))
        }
        AstKind::ObjectProperty(property) => is_same_function(&property.value, function),
        AstKind::PropertyDefinition(definition) => {
            definition.value.as_ref().is_some_and(|value| is_same_function(value, function))
        }
        AstKind::AssignmentExpression(expression) => {
            matches!(expression.left, AssignmentTarget::AssignmentTargetIdentifier(_))
                && is_same_function(&expression.right, function)
        }
        AstKind::AssignmentTargetWithDefault(target) => {
            matches!(target.binding, AssignmentTarget::AssignmentTargetIdentifier(_))
                && is_same_function(&target.init, function)
        }
        AstKind::AssignmentPattern(pattern) => {
            matches!(pattern.left.kind, BindingPatternKind::BindingIdentifier(_))
                && is_same_function(&pattern.right, function)
        }
        AstKind::AssignmentTargetPropertyIdentifier(ident) => {
            ident.init.as_ref().is_some_and(|expr| is_same_function(expr, function))
        }
        AstKind::ObjectAssignmentTarget(target) => does_object_assignment_target_have_name(target),
        _ => false,
    }
}

fn is_same_function<'a>(fn1: &Expression<'a>, fn2: &Function<'a>) -> bool {
    matches!(fn1, Expression::FunctionExpression(function_expression)
        if get_function_identifier(function_expression) == get_function_identifier(fn2)
    )
}

/**
 * Gets the identifier for the function
 */
fn get_function_identifier<'a>(func: &'a Function<'a>) -> Option<&'a Span> {
    func.id.as_ref().map(|id| &id.span)
}

fn is_invalid_function(
    config_type: FuncNamesConfigType,
    func: &Function,
    parent_node: &AstNode<'_>,
) -> bool {
    let func_name = func.name();

    match config_type {
        FuncNamesConfigType::Never => {
            func_name.is_some() && func.r#type != FunctionType::FunctionDeclaration
        }
        FuncNamesConfigType::AsNeeded => {
            func_name.is_none() && !has_inferred_name(func, parent_node)
        }
        FuncNamesConfigType::Always => {
            func_name.is_none() && !is_object_or_class_method(parent_node)
        }
    }
}

/// Returns whether it's safe to insert a function name without breaking shadowing rules
fn can_safely_apply_fix(func: &Function, name: &str, ctx: &LintContext) -> bool {
    !ctx.scoping().find_binding(func.scope_id(), name).is_some_and(|shadowed_var| {
        ctx.semantic().symbol_references(shadowed_var).any(|reference| {
            func.span.contains_inclusive(ctx.nodes().get_node(reference.node_id()).kind().span())
        })
    })
}

fn apply_rule_fix<'a>(
    fixer: &RuleFixer<'_, 'a>,
    is_safe_fix: bool,
    replace_span: Span,
    function_name: Option<String>,
) -> RuleFix<'a> {
    if !is_safe_fix {
        return fixer.noop();
    }

    fixer.insert_text_after(&replace_span, format!(" {}", function_name.unwrap()))
}

fn guess_function_name<'a>(ctx: &LintContext<'a>, node_id: NodeId) -> Option<Cow<'a, str>> {
    ctx.nodes().ancestor_kinds(node_id).find_map(|parent_kind| match parent_kind {
        AstKind::AssignmentExpression(assign) => {
            assign.left.get_identifier_name().map(Cow::Borrowed)
        }
        AstKind::VariableDeclarator(decl) => {
            decl.id.get_identifier_name().as_ref().map(Atom::as_str).map(Cow::Borrowed)
        }
        AstKind::ObjectProperty(prop) => {
            prop.key.static_name().filter(|name| is_valid_identifier_name(name))
        }
        AstKind::PropertyDefinition(prop_def) => {
            prop_def.key.static_name().filter(|name| is_valid_identifier_name(name))
        }
        _ => None,
    })
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let always = Some(json!(["always"]));
    let as_needed = Some(json!(["as-needed"]));
    let never = Some(json!(["never"]));

    let pass = vec![
        ("Foo.prototype.bar = function bar(){};", None),
        ("Foo.prototype.bar = () => {}", None), // { "ecmaVersion": 6 },
        ("function foo(){}", None),
        ("function test(d, e, f) {}", None),
        ("new function bar(){}", None),
        ("exports = { get foo() { return 1; }, set bar(val) { return val; } };", None),
        ("({ foo() { return 1; } });", None), // { "ecmaVersion": 6 },
        ("class A { constructor(){} foo(){} get bar(){} set baz(value){} static qux(){}}", None), // { "ecmaVersion": 6 },
        ("function foo() {}", always.clone()),
        ("var a = function foo() {};", always.clone()),
        (
            "class A { constructor(){} foo(){} get bar(){} set baz(value){} static qux(){}}",
            as_needed.clone(),
        ), // { "ecmaVersion": 6 },
        ("({ foo() {} });", as_needed.clone()), // { "ecmaVersion": 6 },
        ("var foo = function(){};", as_needed.clone()),
        ("({foo: function(){}});", as_needed.clone()),
        ("(foo = function(){});", as_needed.clone()),
        ("({foo = function(){}} = {});", as_needed.clone()), // { "ecmaVersion": 6 },
        ("({key: foo = function(){}} = {});", as_needed.clone()), // { "ecmaVersion": 6 },
        ("[foo = function(){}] = [];", as_needed.clone()),   // { "ecmaVersion": 6 },
        ("function fn(foo = function(){}) {}", as_needed.clone()), // { "ecmaVersion": 6 },
        ("function foo() {}", never.clone()),
        ("var a = function() {};", never.clone()),
        ("var a = function foo() { foo(); };", never.clone()),
        ("var foo = {bar: function() {}};", never.clone()),
        ("$('#foo').click(function() {});", never.clone()),
        ("Foo.prototype.bar = function() {};", never.clone()),
        (
            "class A { constructor(){} foo(){} get bar(){} set baz(value){} static qux(){}}",
            never.clone(),
        ), // { "ecmaVersion": 6 },
        ("({ foo() {} });", never.clone()), // { "ecmaVersion": 6 },
        ("export default function foo() {}", always.clone()), // { "sourceType": "module", "ecmaVersion": 6 },
        ("export default function foo() {}", as_needed.clone()), // { "sourceType": "module", "ecmaVersion": 6 },
        ("export default function foo() {}", never.clone()), // { "sourceType": "module", "ecmaVersion": 6 },
        ("export default function() {}", never.clone()), // { "sourceType": "module", "ecmaVersion": 6 },
        ("var foo = bar(function *baz() {});", always.clone()), // { "ecmaVersion": 6 },
        ("var foo = bar(function *baz() {});", Some(json!(["always", { "generators": "always" }]))), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *baz() {});",
            Some(json!(["always", { "generators": "as-needed" }])),
        ), // { "ecmaVersion": 6 },
        ("var foo = function*() {};", Some(json!(["always", { "generators": "as-needed" }]))), // { "ecmaVersion": 6 },
        ("var foo = bar(function *baz() {});", as_needed.clone()), // { "ecmaVersion": 6 },
        ("var foo = function*() {};", as_needed.clone()),          // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *baz() {});",
            Some(json!(["as-needed", { "generators": "always" }])),
        ), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *baz() {});",
            Some(json!(["as-needed", { "generators": "as-needed" }])),
        ), // { "ecmaVersion": 6 },
        ("var foo = function*() {};", Some(json!(["as-needed", { "generators": "as-needed" }]))), // { "ecmaVersion": 6 },
        ("var foo = bar(function *baz() {});", Some(json!(["never", { "generators": "always" }]))), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *baz() {});",
            Some(json!(["never", { "generators": "as-needed" }])),
        ), // { "ecmaVersion": 6 },
        ("var foo = function*() {};", Some(json!(["never", { "generators": "as-needed" }]))), // { "ecmaVersion": 6 },
        ("var foo = bar(function *() {});", never.clone()), // { "ecmaVersion": 6 },
        ("var foo = function*() {};", never.clone()),       // { "ecmaVersion": 6 },
        ("(function*() {}())", never.clone()),              // { "ecmaVersion": 6 },
        ("var foo = bar(function *() {});", Some(json!(["never", { "generators": "never" }]))), // { "ecmaVersion": 6 },
        ("var foo = function*() {};", Some(json!(["never", { "generators": "never" }]))), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(json!(["never", { "generators": "never" }]))), // { "ecmaVersion": 6 },
        ("var foo = bar(function *() {});", Some(json!(["always", { "generators": "never" }]))), // { "ecmaVersion": 6 },
        ("var foo = function*() {};", Some(json!(["always", { "generators": "never" }]))), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(json!(["always", { "generators": "never" }]))), // { "ecmaVersion": 6 },
        ("var foo = bar(function *() {});", Some(json!(["as-needed", { "generators": "never" }]))), // { "ecmaVersion": 6 },
        ("var foo = function*() {};", Some(json!(["as-needed", { "generators": "never" }]))), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(json!(["as-needed", { "generators": "never" }]))), // { "ecmaVersion": 6 },
        ("class C { foo = function() {}; }", as_needed.clone()), // { "ecmaVersion": 2022 },
        ("class C { [foo] = function() {}; }", as_needed.clone()), // { "ecmaVersion": 2022 },
        ("class C { #foo = function() {}; }", as_needed.clone()), // { "ecmaVersion": 2022 }
    ];

    let fail = vec![
        ("Foo.prototype.bar = function() {};", None),
        ("(function(){}())", None),
        ("f(function(){})", None),
        ("var a = new Date(function() {});", None),
        ("var test = function(d, e, f) {};", None),
        ("new function() {}", None),
        ("Foo.prototype.bar = function() {};", as_needed.clone()),
        ("(function(){}())", as_needed.clone()),
        ("f(function(){})", as_needed.clone()),
        ("var a = new Date(function() {});", as_needed.clone()),
        ("new function() {}", as_needed.clone()),
        ("var {foo} = function(){};", as_needed.clone()), // { "ecmaVersion": 6 },
        ("({ a: obj.prop = function(){} } = foo);", as_needed.clone()), // { "ecmaVersion": 6 },
        ("[obj.prop = function(){}] = foo;", as_needed.clone()), // { "ecmaVersion": 6 },
        ("var { a: [b] = function(){} } = foo;", as_needed.clone()), // { "ecmaVersion": 6 },
        ("function foo({ a } = function(){}) {};", as_needed.clone()), // { "ecmaVersion": 6 },
        ("var x = function foo() {};", never.clone()),
        ("Foo.prototype.bar = function foo() {};", never.clone()),
        ("({foo: function foo() {}})", never.clone()),
        ("export default function() {}", always.clone()), // { "sourceType": "module", "ecmaVersion": 6 },
        ("export default function() {}", as_needed.clone()), // { "sourceType": "module", "ecmaVersion": 6 },
        ("export default (function(){});", as_needed.clone()), // { "sourceType": "module", "ecmaVersion": 6 },
        ("var foo = bar(function *() {});", always.clone()),   // { "ecmaVersion": 6 },
        ("var foo = function*() {};", always.clone()),         // { "ecmaVersion": 6 },
        ("(function*() {}())", always.clone()),                // { "ecmaVersion": 6 },
        ("var foo = bar(function *() {});", Some(json!(["always", { "generators": "always" }]))), // { "ecmaVersion": 6 },
        ("var foo = function*() {};", Some(json!(["always", { "generators": "always" }]))), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(json!(["always", { "generators": "always" }]))), // { "ecmaVersion": 6 },
        ("var foo = bar(function *() {});", Some(json!(["always", { "generators": "as-needed" }]))), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(json!(["always", { "generators": "as-needed" }]))), // { "ecmaVersion": 6 },
        ("var foo = bar(function *() {});", as_needed.clone()), // { "ecmaVersion": 6 },
        ("(function*() {}())", as_needed.clone()),              // { "ecmaVersion": 6 },
        ("var foo = bar(function *() {});", Some(json!(["as-needed", { "generators": "always" }]))), // { "ecmaVersion": 6 },
        ("var foo = function*() {};", Some(json!(["as-needed", { "generators": "always" }]))), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(json!(["as-needed", { "generators": "always" }]))), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *() {});",
            Some(json!(["as-needed", { "generators": "as-needed" }])),
        ), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(json!(["as-needed", { "generators": "as-needed" }]))), // { "ecmaVersion": 6 },
        ("var foo = bar(function *() {});", Some(json!(["never", { "generators": "always" }]))), // { "ecmaVersion": 6 },
        ("var foo = function*() {};", Some(json!(["never", { "generators": "always" }]))), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(json!(["never", { "generators": "always" }]))), // { "ecmaVersion": 6 },
        ("var foo = bar(function *() {});", Some(json!(["never", { "generators": "as-needed" }]))), // { "ecmaVersion": 6 },
        ("(function*() {}())", Some(json!(["never", { "generators": "as-needed" }]))), // { "ecmaVersion": 6 },
        ("var foo = bar(function *baz() {});", never.clone()), // { "ecmaVersion": 6 },
        ("var foo = bar(function *baz() {});", Some(json!(["never", { "generators": "never" }]))), // { "ecmaVersion": 6 },
        ("var foo = bar(function *baz() {});", Some(json!(["always", { "generators": "never" }]))), // { "ecmaVersion": 6 },
        (
            "var foo = bar(function *baz() {});",
            Some(json!(["as-needed", { "generators": "never" }])),
        ), // { "ecmaVersion": 6 },
        ("class C { foo = function() {} }", always.clone()), // { "ecmaVersion": 2022 },
        ("class C { public foo = function() {} }", always.clone()), // { "ecmaVersion": 2022 },
        ("class C { [foo] = function() {} }", always.clone()), // { "ecmaVersion": 2022 },
        ("class C { #foo = function() {} }", always.clone()), // { "ecmaVersion": 2022 },
        ("class C { foo = bar(function() {}) }", as_needed), // { "ecmaVersion": 2022 },
        ("class C { foo = function bar() {} }", never.clone()), // { "ecmaVersion": 2022 }
    ];

    let fix = vec![
        // lb
        ("const foo = function() {}", "const foo = function foo() {}", always.clone()),
        (
            "Foo.prototype.bar = function() {}",
            "Foo.prototype.bar = function bar() {}",
            always.clone(),
        ),
        ("let foo; foo = function() {}", "let foo; foo = function foo() {}", always.clone()),
        (
            "class C { public foo = function() {} }",
            "class C { public foo = function foo() {} }",
            always.clone(),
        ),
        (
            "class C { public ['foo'] = function() {} }",
            "class C { public ['foo'] = function foo() {} }",
            always.clone(),
        ),
        (
            "class C { public [`foo`] = function() {} }",
            "class C { public [`foo`] = function foo() {} }",
            always.clone(),
        ),
        (
            "class C { public ['invalid identifier name'] = function() {} }",
            "class C { public ['invalid identifier name'] = function() {} }",
            always.clone(),
        ),
        (
            "class C { public [foo] = function() {} }",
            "class C { public [foo] = function() {} }",
            always.clone(),
        ),
        (
            "class C { public [undefined] = function() {} }",
            "class C { public [undefined] = function() {} }",
            always.clone(),
        ),
        (
            "class C { public [null] = function() {} }",
            "class C { public [null] = function() {} }",
            always.clone(),
        ),
        (
            "class C { public ['undefined'] = function() {} }",
            "class C { public ['undefined'] = function() {} }",
            always.clone(),
        ),
        (
            "class C { public ['null'] = function() {} }",
            "class C { public ['null'] = function() {} }",
            always.clone(),
        ),
        (
            "const x = { foo: function() {} }",
            "const x = { foo: function foo() {} }",
            always.clone(),
        ),
        (
            "const x = { ['foo']: function() {} }",
            "const x = { ['foo']: function foo() {} }",
            always.clone(),
        ),
        // suggest removal when configured to "never"
        ("const foo = function foo() {}", "const foo = function () {}", never.clone()),
        (
            "Foo.prototype.bar = function bar() {}",
            "Foo.prototype.bar = function () {}",
            never.clone(),
        ),
        ("class C { foo = function foo() {} }", "class C { foo = function () {} }", never),
        (
            "const restoreGracefully = function <T>(entries: T[]) { }",
            "const restoreGracefully = function  restoreGracefully<T>(entries: T[]) { }",
            None,
        ),
        ("const foo = async function() {}", "const foo = async function foo() {}", always.clone()),
        (
            "const foo = async function            () {}",
            "const foo = async function             foo() {}",
            always.clone(),
        ),
        (
            "const foo =      async          function      <T>      ()           {}",
            "const foo =      async          function       foo<T>      ()           {}",
            always.clone(),
        ),
        (
            "const foo =      async          function      <T           >      ()           {}",
            "const foo =      async          function       foo<T           >      ()           {}",
            always.clone(),
        ),
        ("const foo = function* () {}", "const foo = function*  foo() {}", always.clone()),
        (
            "const foo = async function* (){}",
            "const foo = async function*  foo(){}",
            always.clone(),
        ),
        (
            "const foo = async function* <T extends foo>(){}",
            "const foo = async function*  foo<T extends foo>(){}",
            always.clone(),
        ),
        // we can't fix this case because adding a name would cause the
        (
            "const setState = Component.prototype.setState;
             Component.prototype.setState = function (update, callback) {
	             return setState.call(this, update, callback);
            };",
            "const setState = Component.prototype.setState;
             Component.prototype.setState = function (update, callback) {
	             return setState.call(this, update, callback);
            };",
            always,
        ),
    ];

    Tester::new(FuncNames::NAME, FuncNames::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
