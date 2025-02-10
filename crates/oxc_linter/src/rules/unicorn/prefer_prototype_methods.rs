use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    ast_util::is_method_call,
    context::LintContext,
    rule::Rule,
    utils::{is_empty_array_expression, is_empty_object_expression},
    AstNode,
};

fn known_method(span: Span, obj_name: &str, method_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer using `{obj_name}.prototype.{method_name}`."))
        .with_label(span)
}

fn unknown_method(span: Span, obj_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer using method from `{obj_name}.prototype`."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferPrototypeMethods;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule prefers borrowing methods from the prototype instead of the instance.
    ///
    /// ### Why is this bad?
    ///
    /// “Borrowing” a method from an instance of `Array` or `Object` is less clear than getting it from the corresponding prototype.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const array = [].slice.apply(bar);
    /// const type = {}.toString.call(foo);
    /// Reflect.apply([].forEach, arrayLike, [callback]);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const array = Array.prototype.slice.apply(bar);
    /// const type = Object.prototype.toString.call(foo);
    /// Reflect.apply(Array.prototype.forEach, arrayLike, [callback]);
    /// const maxValue = Math.max.apply(Math, numbers);
    /// ```
    PreferPrototypeMethods,
    unicorn,
    pedantic,
    fix
);

impl Rule for PreferPrototypeMethods {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.optional {
            return;
        }
        match call_expr.callee.without_parentheses() {
            Expression::StaticMemberExpression(member_expr) if !member_expr.optional => {}
            Expression::PrivateFieldExpression(member_expr) if !member_expr.optional => {}
            _ => return,
        };

        let mut method_expr: Option<&Expression> = None;

        // `Reflect.apply([].foo, …)`
        // `Reflect.apply({}.foo, …)`
        if is_method_call(call_expr, Some(&["Reflect"]), Some(&["apply"]), Some(1), None) {
            if let Some(argument_expr) = call_expr.arguments[0].as_expression() {
                method_expr = Some(argument_expr.without_parentheses());
            }
        }
        // `[].foo.{apply,bind,call}(…)`
        // `({}).foo.{apply,bind,call}(…)`
        else if is_method_call(call_expr, None, Some(&["apply", "bind", "call"]), None, None) {
            method_expr = call_expr
                .callee
                .get_member_expr()
                .map(|member_expr| member_expr.object().without_parentheses());
        }

        let Some(method_expr) = method_expr else {
            return;
        };
        let Some(method_expr) = method_expr.as_member_expression() else {
            return;
        };
        let object_expr = method_expr.object().without_parentheses();

        if !is_empty_array_expression(object_expr) && !is_empty_object_expression(object_expr) {
            return;
        }

        let constructor_name = match object_expr {
            Expression::ArrayExpression(_) => "Array",
            Expression::ObjectExpression(_) => "Object",
            _ => unreachable!(),
        };
        // TODO: Replace `static_property_name` with a function similar to `getPropertyName`
        // in @eslint-community/eslint-utils to generate better error messages for some cases.
        let method_name = method_expr.static_property_name();

        ctx.diagnostic_with_fix(
            method_name.map_or_else(
                || unknown_method(method_expr.span(), constructor_name),
                |method_name| known_method(method_expr.span(), constructor_name, method_name),
            ),
            |fixer| {
                let span = object_expr.span();
                let need_padding = span.start >= 1
                    && ctx.source_text().as_bytes()[span.start as usize - 1].is_ascii_alphabetic();
                fixer.replace(
                    span,
                    format!(
                        "{}{}.prototype",
                        if need_padding { " " } else { "" },
                        constructor_name
                    ),
                )
            },
        );
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const foo = Array.prototype.push.apply(bar, elements);",
        "const foo = Array.prototype.slice.call(bar);",
        "const foo = Object.prototype.toString.call(bar);",
        r#"const foo = Object.prototype.hasOwnProperty.call(bar, "property");"#,
        r#"const foo = Object.prototype.propertyIsEnumerable.call(bar, "property");"#,
        r#"const foo = "".charCodeAt.call(bar, 0);"#,
        "const foo = [notEmpty].push.apply(bar, elements);",
        "const foo = {notEmpty}.toString.call(bar);",
        "Array.prototype.forEach.call(foo, () => {})",
        "const push = Array.prototype.push.bind(foo)",
        "const push = [].push",
        "const {push} = []",
        "Math.max.apply(null, numbers)",
        "foo.apply(null, args)",
        "Reflect.apply(...[].slice)",
        "Reflect.apply(foo, [].slice)",
        "Reflect.apply(Math.max, Math, numbers)",
        "Reflect.apply()",
        r#"Reflect["apply"]([].slice, foo, [])"#,
        "NotReflect.apply([].slice, foo, [])",
        "Reflect.notApply([].slice, foo, [])",
        "Reflect.apply([]?.slice, foo, [])",
        "Reflect.apply(foo, [].bar, [].bar)",
        "class Foo {bar() { this.baz.bind(this) }}",
        "const a = {bar() { this.baz.bind(this) }}",
        "foo.bar.bind(foo)",
        "foo.bar.bind(bar)",
        "foo[{}].call(bar)",
        "Object.hasOwn(bar)",
    ];

    let fail = vec![
        "const foo = [].push.apply(bar, elements);",
        "const foo = [].slice.call(bar);",
        "const foo = {}.toString.call(bar);",
        r#"const foo = {}.hasOwnProperty.call(bar, "property");"#,
        r#"const foo = {}.propertyIsEnumerable.call(bar, "property");"#,
        "[].forEach.call(foo, () => {})",
        "const push = [].push.bind(foo)",
        "const foo = [][method].call(foo)",
        r#"const method = "realMethodName";const foo = [][method].call(foo)"#, // TODO: Improve error message for this case.
        "const array = Reflect.apply([].slice, foo, [])",
        "Reflect.apply([].bar, baz, [])",
        "const foo = ({}).toString.call(bar);",
        "const foo = ({}.toString).call(bar);",
        "const foo = ({}.toString.call)(bar);",
        "function foo(){return[].slice.call(bar);}",
        "function foo(){return{}.toString.call(bar)}",
        "Reflect.apply({}[Symbol()], baz, [])",
        r#"Reflect.apply({}[Symbol("symbol description")], baz, [])"#,
        "Reflect.apply([][Symbol()], baz, [])",
        r#"Reflect.apply({}[Symbol("symbol description")], baz, [])"#,
        "[][Symbol.iterator].call(foo)", // TODO: Improve error message for this case.
        "const foo = [].at.call(bar)",
        "const foo = [].findLast.call(bar)",
    ];

    let fix = vec![
        (
            "const foo = [].push.apply(bar, elements);",
            "const foo = Array.prototype.push.apply(bar, elements);",
            None,
        ),
        ("const foo = [].slice.call(bar);", "const foo = Array.prototype.slice.call(bar);", None),
        (
            "const foo = {}.toString.call(bar);",
            "const foo = Object.prototype.toString.call(bar);",
            None,
        ),
        (
            r#"const foo = {}.hasOwnProperty.call(bar, "property");"#,
            r#"const foo = Object.prototype.hasOwnProperty.call(bar, "property");"#,
            None,
        ),
        (
            r#"const foo = {}.propertyIsEnumerable.call(bar, "property");"#,
            r#"const foo = Object.prototype.propertyIsEnumerable.call(bar, "property");"#,
            None,
        ),
        ("[].forEach.call(foo, () => {})", "Array.prototype.forEach.call(foo, () => {})", None),
        ("const push = [].push.bind(foo)", "const push = Array.prototype.push.bind(foo)", None),
        ("const foo = [][method].call(foo)", "const foo = Array.prototype[method].call(foo)", None),
        (
            r#"const method = "realMethodName";const foo = [][method].call(foo)"#,
            r#"const method = "realMethodName";const foo = Array.prototype[method].call(foo)"#,
            None,
        ),
        (
            "const array = Reflect.apply([].slice, foo, [])",
            "const array = Reflect.apply(Array.prototype.slice, foo, [])",
            None,
        ),
        ("Reflect.apply([].bar, baz, [])", "Reflect.apply(Array.prototype.bar, baz, [])", None),
        (
            "const foo = ({}).toString.call(bar);",
            "const foo = (Object.prototype).toString.call(bar);",
            None,
        ),
        (
            "const foo = ({}.toString).call(bar);",
            "const foo = (Object.prototype.toString).call(bar);",
            None,
        ),
        (
            "const foo = ({}.toString.call)(bar);",
            "const foo = (Object.prototype.toString.call)(bar);",
            None,
        ),
        (
            "function foo(){return[].slice.call(bar);}",
            "function foo(){return Array.prototype.slice.call(bar);}",
            None,
        ),
        (
            "function foo(){return{}.toString.call(bar)}",
            "function foo(){return Object.prototype.toString.call(bar)}",
            None,
        ),
        (
            "Reflect.apply({}[Symbol()], baz, [])",
            "Reflect.apply(Object.prototype[Symbol()], baz, [])",
            None,
        ),
        (
            r#"Reflect.apply({}[Symbol("symbol description")], baz, [])"#,
            r#"Reflect.apply(Object.prototype[Symbol("symbol description")], baz, [])"#,
            None,
        ),
        (
            "Reflect.apply([][Symbol()], baz, [])",
            "Reflect.apply(Array.prototype[Symbol()], baz, [])",
            None,
        ),
        (
            r#"Reflect.apply({}[Symbol("symbol description")], baz, [])"#,
            r#"Reflect.apply(Object.prototype[Symbol("symbol description")], baz, [])"#,
            None,
        ),
        ("[][Symbol.iterator].call(foo)", "Array.prototype[Symbol.iterator].call(foo)", None),
        ("const foo = [].at.call(bar)", "const foo = Array.prototype.at.call(bar)", None),
        (
            "const foo = [].findLast.call(bar)",
            "const foo = Array.prototype.findLast.call(bar)",
            None,
        ),
    ];

    Tester::new(PreferPrototypeMethods::NAME, PreferPrototypeMethods::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
