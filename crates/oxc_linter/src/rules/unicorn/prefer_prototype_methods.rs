use oxc_ast::{
    AstKind,
    ast::{Expression, IdentifierReference, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::JSStr;

use crate::{
    AstNode,
    context::LintContext,
    globals::GLOBAL_OBJECT_NAMES,
    rule::Rule,
    utils::{pad_fix_with_token_boundary, static_string_value},
};

const OBJECT_PROTOTYPE_METHODS: [&str; 6] = [
    "hasOwnProperty",
    "isPrototypeOf",
    "propertyIsEnumerable",
    "toLocaleString",
    "toString",
    "valueOf",
];

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
    /// “Borrowing” a method from an instance of `Array`, `Object`, or the global object is less clear than getting it from the corresponding prototype.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const array = [].slice.apply(bar);
    /// const type = {}.toString.call(foo);
    /// const globalType = globalThis.toString.call(foo);
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
    recommended,
    conditional_fix,
    version = "0.0.21",
    short_description =
        "This rule prefers borrowing methods from the prototype instead of the instance.",
);

impl Rule for PreferPrototypeMethods {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.optional {
            return;
        }
        let Expression::StaticMemberExpression(callee_member) =
            call_expr.callee.without_parentheses()
        else {
            return;
        };
        if callee_member.optional {
            return;
        }

        let method_expr = match callee_member.property.name.as_str() {
            // `Reflect.apply([].foo, …)`
            // `Reflect.apply({}.foo, …)`
            "apply"
                if !call_expr.arguments.is_empty()
                    && matches!(
                        callee_member.object.without_parentheses(),
                        Expression::Identifier(ident) if ident.name == "Reflect"
                    ) =>
            {
                call_expr.arguments[0].as_expression().map(Expression::without_parentheses)
            }
            // `[].foo.{apply,bind,call}(…)`
            // `({}).foo.{apply,bind,call}(…)`
            "apply" | "bind" | "call" => Some(callee_member.object.without_parentheses()),
            _ => return,
        };

        let Some(method_expr) = method_expr else {
            return;
        };
        let Some(method_expr) = method_expr.as_member_expression() else {
            return;
        };
        let object_expr = method_expr.object().without_parentheses();

        let (constructor_name, method_name, global_method_name) = match object_expr {
            Expression::Identifier(ident) => {
                let Some(method_name) = global_object_prototype_method(ident, method_expr, ctx)
                else {
                    return;
                };
                ("Object", Some(method_name), Some(method_name))
            }
            _ if method_expr.optional() => return,
            Expression::ArrayExpression(array) if array.elements.is_empty() => {
                ("Array", method_expr.static_property_name().and_then(JSStr::as_str), None)
            }
            Expression::ObjectExpression(object) if object.properties.is_empty() => {
                ("Object", method_expr.static_property_name().and_then(JSStr::as_str), None)
            }
            _ => return,
        };

        let diagnostic = method_name.map_or_else(
            || unknown_method(method_expr.span(), constructor_name),
            |method_name| known_method(method_expr.span(), constructor_name, method_name),
        );
        let replace_span =
            if global_method_name.is_some() { method_expr.span() } else { object_expr.span() };

        if ctx.has_comments_between(replace_span) {
            ctx.diagnostic(diagnostic);
            return;
        }

        ctx.diagnostic_with_fix(diagnostic, |fixer| {
            let mut replacement = match global_method_name {
                Some(method_name) => format!("{constructor_name}.prototype.{method_name}"),
                None => format!("{constructor_name}.prototype"),
            };
            pad_fix_with_token_boundary(ctx.source_text(), replace_span, &mut replacement);
            fixer.replace(replace_span, replacement)
        });
    }
}

fn is_unmodified_global_variable(ident: &IdentifierReference<'_>, ctx: &LintContext<'_>) -> bool {
    ctx.is_reference_to_global_variable(ident)
        && !ctx.scoping().root_unresolved_references().get(ident.name.as_str()).is_some_and(
            |refs| refs.iter().any(|&ref_id| ctx.scoping().get_reference(ref_id).is_write()),
        )
}

fn is_global_object_reference(ident: &IdentifierReference<'_>, ctx: &LintContext<'_>) -> bool {
    let mut ident = ident;
    for _ in 0..=8 {
        if GLOBAL_OBJECT_NAMES.contains(&ident.name.as_str())
            && is_unmodified_global_variable(ident, ctx)
        {
            return true;
        }
        let scoping = ctx.scoping();
        let Some(symbol_id) = scoping.get_reference(ident.reference_id()).symbol_id() else {
            return false;
        };
        let AstKind::VariableDeclarator(declarator) =
            ctx.nodes().kind(scoping.symbol_declaration(symbol_id))
        else {
            return false;
        };
        if !declarator.id.is_binding_identifier()
            || scoping.symbol_flags(symbol_id).is_function_scoped_declaration()
            || scoping.symbol_is_mutated(symbol_id)
            || !scoping.symbol_redeclarations(symbol_id).is_empty()
        {
            return false;
        }
        let Some(Expression::Identifier(init)) =
            declarator.init.as_ref().map(Expression::without_parentheses)
        else {
            return false;
        };
        ident = init;
    }
    false
}

fn global_object_prototype_method<'a>(
    ident: &IdentifierReference<'a>,
    member_expr: &MemberExpression<'a>,
    ctx: &LintContext<'a>,
) -> Option<&'a str> {
    let method_name = match member_expr {
        MemberExpression::StaticMemberExpression(static_expr) => static_expr.property.name.as_str(),
        MemberExpression::ComputedMemberExpression(computed_expr) => {
            static_string_value(&computed_expr.expression, ctx.allocator())?.as_str()?
        }
        MemberExpression::PrivateFieldExpression(_) => return None,
    };
    (OBJECT_PROTOTYPE_METHODS.contains(&method_name) && is_global_object_reference(ident, ctx))
        .then_some(method_name)
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
        "const foo = [].push.notApply(bar, elements);",
        "const push = [].push.notBind(foo)",
        "[].forEach.notCall(foo, () => {})",
        // "/* globals foo: readonly */ foo.call(bar)",
        "const toString = () => {}; toString.call(bar)",
        // "/* globals toString: off */ toString.call(bar)",
        "const _hasOwnProperty = globalThis.hasOwnProperty; _hasOwnProperty.call(bar)",
        "const _globalThis = globalThis; globalThis[hasOwnProperty].call(bar)",
        r#"const _ = globalThis, TO_STRING = "toString"; _[TO_STRING].call(bar)"#,
        "const _ = [globalThis.toString]; _[0].call(bar)",
        "globalThis.setTimeout.call(bar)",
        "const globalThis = {}; globalThis.toString.call(bar)",
        "const window = {}; window.toString.call(bar)",
        "self = customObj; self.toString.call(bar)",
        "globalThis = customObj; globalThis.toString.call(bar)",
        "global = customObj; const _ = global; _.toString.call(bar)",
        "let _ = globalThis; _ = customObj; _.toString.call(bar)",
        "var _ = globalThis; var _ = customObj; _.toString.call(bar)",
        "var a = b; var b = globalThis; a.toString.call(bar)",
        "if (cond) { var _ = globalThis; } _.toString.call(bar)",
        "_.toString.call(bar); var _ = globalThis;",
        "const { foo } = globalThis; foo.toString.call(bar)",
        "const a = b; const b = a; a.toString.call(bar)",
        "[]?.slice.call(bar)",
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
        // "/* globals hasOwnProperty: readonly */ hasOwnProperty.call(bar)",
        // "/* globals toString: readonly */ toString.apply(bar, [])",
        // "/* globals toString: readonly */ Reflect.apply(toString, baz, [])",
        "globalThis.toString.call(bar)",
        "const _ = globalThis; _.hasOwnProperty.call(bar)",
        r#"const _ = globalThis; _["hasOwnProperty"].call(bar)"#,
        r#"const _ = globalThis; _["hasOwn" + "Property"].call(bar)"#,
        "Reflect.apply(globalThis.toString, baz, [])",
        "Reflect.apply(window.toString, baz, [])",
        "Reflect.apply(global.toString, baz, [])",
        r#"Reflect.apply(globalThis["toString"], baz, [])"#,
        "globalThis /* comment */ .toString.call(bar)",
        r#"Reflect.apply(globalThis[/* comment */ "toString"], baz, [])"#,
        "[] /* comment */ .slice.call(bar)",
        "[/* comment */].slice.call(bar)",
        "({/* keep */}).toString.call(o);",
        "function foo(){return(globalThis).toString.call(bar)}",
        "const a = globalThis; const b = a; b.isPrototypeOf.call(bar)",
        "Reflect.apply(self.toString, baz, [])",
        "globalThis[`toString`].call(bar)",
        "globalThis?.toString.call(bar)",
    ];

    let fix = vec![
        (
            "const foo = [].push.apply(bar, elements);",
            "const foo = Array.prototype.push.apply(bar, elements);",
        ),
        ("const foo = [].slice.call(bar);", "const foo = Array.prototype.slice.call(bar);"),
        ("const foo = {}.toString.call(bar);", "const foo = Object.prototype.toString.call(bar);"),
        (
            r#"const foo = {}.hasOwnProperty.call(bar, "property");"#,
            r#"const foo = Object.prototype.hasOwnProperty.call(bar, "property");"#,
        ),
        (
            r#"const foo = {}.propertyIsEnumerable.call(bar, "property");"#,
            r#"const foo = Object.prototype.propertyIsEnumerable.call(bar, "property");"#,
        ),
        ("[].forEach.call(foo, () => {})", "Array.prototype.forEach.call(foo, () => {})"),
        ("const push = [].push.bind(foo)", "const push = Array.prototype.push.bind(foo)"),
        ("const foo = [][method].call(foo)", "const foo = Array.prototype[method].call(foo)"),
        (
            r#"const method = "realMethodName";const foo = [][method].call(foo)"#,
            r#"const method = "realMethodName";const foo = Array.prototype[method].call(foo)"#,
        ),
        (
            "const array = Reflect.apply([].slice, foo, [])",
            "const array = Reflect.apply(Array.prototype.slice, foo, [])",
        ),
        ("Reflect.apply([].bar, baz, [])", "Reflect.apply(Array.prototype.bar, baz, [])"),
        (
            "const foo = ({}).toString.call(bar);",
            "const foo = (Object.prototype).toString.call(bar);",
        ),
        (
            "const foo = ({}.toString).call(bar);",
            "const foo = (Object.prototype.toString).call(bar);",
        ),
        (
            "const foo = ({}.toString.call)(bar);",
            "const foo = (Object.prototype.toString.call)(bar);",
        ),
        (
            "function foo(){return[].slice.call(bar);}",
            "function foo(){return Array.prototype.slice.call(bar);}",
        ),
        (
            "function foo(){return{}.toString.call(bar)}",
            "function foo(){return Object.prototype.toString.call(bar)}",
        ),
        (
            "Reflect.apply({}[Symbol()], baz, [])",
            "Reflect.apply(Object.prototype[Symbol()], baz, [])",
        ),
        (
            r#"Reflect.apply({}[Symbol("symbol description")], baz, [])"#,
            r#"Reflect.apply(Object.prototype[Symbol("symbol description")], baz, [])"#,
        ),
        (
            "Reflect.apply([][Symbol()], baz, [])",
            "Reflect.apply(Array.prototype[Symbol()], baz, [])",
        ),
        (
            r#"Reflect.apply({}[Symbol("symbol description")], baz, [])"#,
            r#"Reflect.apply(Object.prototype[Symbol("symbol description")], baz, [])"#,
        ),
        ("[][Symbol.iterator].call(foo)", "Array.prototype[Symbol.iterator].call(foo)"),
        ("const foo = [].at.call(bar)", "const foo = Array.prototype.at.call(bar)"),
        ("const foo = [].findLast.call(bar)", "const foo = Array.prototype.findLast.call(bar)"),
        ("globalThis.toString.call(bar)", "Object.prototype.toString.call(bar)"),
        (
            "const _ = globalThis; _.hasOwnProperty.call(bar)",
            "const _ = globalThis; Object.prototype.hasOwnProperty.call(bar)",
        ),
        (
            r#"const _ = globalThis; _["hasOwnProperty"].call(bar)"#,
            "const _ = globalThis; Object.prototype.hasOwnProperty.call(bar)",
        ),
        (
            r#"const _ = globalThis; _["hasOwn" + "Property"].call(bar)"#,
            "const _ = globalThis; Object.prototype.hasOwnProperty.call(bar)",
        ),
        (
            "const a = globalThis; const b = a; b.isPrototypeOf.call(bar)",
            "const a = globalThis; const b = a; Object.prototype.isPrototypeOf.call(bar)",
        ),
        (
            "Reflect.apply(globalThis.toString, baz, [])",
            "Reflect.apply(Object.prototype.toString, baz, [])",
        ),
        (
            "Reflect.apply(window.toString, baz, [])",
            "Reflect.apply(Object.prototype.toString, baz, [])",
        ),
        (
            "Reflect.apply(global.toString, baz, [])",
            "Reflect.apply(Object.prototype.toString, baz, [])",
        ),
        (
            "Reflect.apply(self.toString, baz, [])",
            "Reflect.apply(Object.prototype.toString, baz, [])",
        ),
        (
            r#"Reflect.apply(globalThis["toString"], baz, [])"#,
            "Reflect.apply(Object.prototype.toString, baz, [])",
        ),
        ("globalThis[`toString`].call(bar)", "Object.prototype.toString.call(bar)"),
        (
            "globalThis /* comment */ .toString.call(bar)",
            "globalThis /* comment */ .toString.call(bar)",
        ),
        (
            r#"Reflect.apply(globalThis[/* comment */ "toString"], baz, [])"#,
            r#"Reflect.apply(globalThis[/* comment */ "toString"], baz, [])"#,
        ),
        ("[] /* comment */ .slice.call(bar)", "Array.prototype /* comment */ .slice.call(bar)"),
        ("[/* comment */].slice.call(bar)", "[/* comment */].slice.call(bar)"),
        ("({/* keep */}).toString.call(o);", "({/* keep */}).toString.call(o);"),
        (
            "function foo(){return(globalThis).toString.call(bar)}",
            "function foo(){return Object.prototype.toString.call(bar)}",
        ),
        ("globalThis?.toString.call(bar)", "Object.prototype.toString.call(bar)"),
    ];

    Tester::new(PreferPrototypeMethods::NAME, PreferPrototypeMethods::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
