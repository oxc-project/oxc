use cow_utils::CowUtils;
use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, match_member_expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, ast_util, context::LintContext, rule::Rule};

fn unicorn_prefer_spread_diagnostic(span: Span, bad_method: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer the spread operator (`...`) over {bad_method}"))
        .with_help("The spread operator (`...`) is more concise and readable.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferSpread;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of [the spread operator (`...`)](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Spread_syntax) over outdated patterns.
    ///
    /// ### Why is this bad?
    ///
    /// Using the spread operator is more concise and readable.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = Array.from(set);
    /// const foo = Array.from(new Set([1, 2]));
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// [...set].map(() => {});
    /// Array.from(...argumentsArray);
    /// ```
    PreferSpread,
    unicorn,
    style,
    conditional_fix
);

impl Rule for PreferSpread {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        check_unicorn_prefer_spread(node, call_expr, ctx);
    }
}

fn check_unicorn_prefer_spread<'a>(
    node: &AstNode<'a>,
    call_expr: &CallExpression<'a>,
    ctx: &LintContext<'a>,
) {
    let Some(member_expr) = call_expr.callee.without_parentheses().as_member_expression() else {
        return;
    };

    let Some(static_property_name) = member_expr.static_property_name() else {
        return;
    };

    match static_property_name {
        // `Array.from()`
        "from" => {
            if call_expr.arguments.len() != 1 || member_expr.is_computed() {
                return;
            }

            let Some(expr) = call_expr.arguments[0].as_expression() else {
                return;
            };
            if matches!(expr.without_parentheses(), Expression::ObjectExpression(_)) {
                return;
            }

            let Expression::Identifier(ident) = member_expr.object().without_parentheses() else {
                return;
            };

            if ident.name != "Array" {
                return;
            }

            report_with_spread_fixer(node, ctx, call_expr.span, "Array.from()", expr);
        }
        // `array.concat()`
        "concat" => {
            if is_not_array(member_expr.object().without_parentheses(), ctx) {
                return;
            }

            ctx.diagnostic(unicorn_prefer_spread_diagnostic(call_expr.span, "array.concat()"));
        }
        // `array.slice()`
        "slice" => {
            if call_expr.arguments.len() > 1 {
                return;
            }

            let member_expr_obj = member_expr.object().without_parentheses();

            if matches!(
                member_expr_obj,
                Expression::ArrayExpression(_) | Expression::ThisExpression(_)
            ) {
                return;
            }

            if is_typed_array_or_buffer_construction(member_expr_obj) {
                return;
            }

            if let Expression::Identifier(ident) = member_expr_obj
                && IGNORED_SLICE_CALLEE.contains(&ident.name.as_str())
            {
                return;
            }

            if let Some(first_arg) = call_expr.arguments.first() {
                let Some(first_arg) = first_arg.as_expression() else {
                    return;
                };
                if let Expression::NumericLiteral(num_lit) = first_arg.without_parentheses() {
                    if num_lit.value != 0.0 {
                        return;
                    }
                } else {
                    return;
                }
            }

            report_with_spread_fixer(node, ctx, call_expr.span, "array.slice()", member_expr_obj);
        }
        // `array.toSpliced()`
        "toSpliced" => {
            if !call_expr.arguments.is_empty() {
                return;
            }

            if matches!(member_expr.object().without_parentheses(), Expression::ArrayExpression(_))
            {
                return;
            }

            report_with_spread_fixer(
                node,
                ctx,
                call_expr.span,
                "array.toSpliced()",
                member_expr.object(),
            );
        }
        // `string.split()`
        "split" => {
            if call_expr.arguments.len() != 1 {
                return;
            }

            let Some(expr) = call_expr.arguments[0].as_expression() else {
                return;
            };
            let Expression::StringLiteral(string_lit) = expr.without_parentheses() else {
                return;
            };

            if !string_lit.value.is_empty() {
                return;
            }

            ctx.diagnostic_with_fix(
                unicorn_prefer_spread_diagnostic(call_expr.span, "string.split()"),
                |fixer| {
                    let needs_semi = ast_util::could_be_asi_hazard(node, ctx);
                    let callee_obj = member_expr.object().without_parentheses();
                    let prefix = if needs_semi { ";" } else { "" };
                    fixer.replace(
                        call_expr.span,
                        format!(
                            "{prefix}[...{}]",
                            callee_obj.span().source_text(ctx.source_text())
                        ),
                    )
                },
            );
        }
        _ => {}
    }
}

const IGNORED_SLICE_CALLEE: [&str; 5] = ["arrayBuffer", "blob", "buffer", "file", "this"];

/// Check if an expression is `new TypedArray(...)`, `new ArrayBuffer(...)`,
/// or `new SharedArrayBuffer(...)`. Spreading these either fails (ArrayBuffer
/// has no iterator) or changes the type (TypedArray → number[]).
fn is_typed_array_or_buffer_construction(expr: &Expression) -> bool {
    let Expression::NewExpression(new_expr) = expr else { return false };
    let Expression::Identifier(ident) = &new_expr.callee else { return false };
    matches!(
        ident.name.as_str(),
        "ArrayBuffer"
            | "SharedArrayBuffer"
            | "Int8Array"
            | "Uint8Array"
            | "Uint8ClampedArray"
            | "Int16Array"
            | "Uint16Array"
            | "Int32Array"
            | "Uint32Array"
            | "Float16Array"
            | "Float32Array"
            | "Float64Array"
            | "BigInt64Array"
            | "BigUint64Array"
    )
}

fn is_not_array(expr: &Expression, ctx: &LintContext) -> bool {
    if matches!(
        expr.without_parentheses(),
        Expression::TemplateLiteral(_) | Expression::BinaryExpression(_)
    ) {
        return true;
    }
    if expr.is_literal() {
        return true;
    }

    if let Expression::CallExpression(call_expr) = expr {
        if let Some(member_expr) = call_expr.callee.without_parentheses().as_member_expression() {
            if Some("join") == member_expr.static_property_name() && call_expr.arguments.len() < 2 {
                return true;
            }
            return false;
        }
        return false;
    }

    let ident = match expr.without_parentheses() {
        Expression::Identifier(ident) => {
            if let Some(symbol_id) = ast_util::get_symbol_id_of_variable(ident, ctx) {
                let symbol_table = ctx.scoping();
                let node = ctx.nodes().get_node(symbol_table.symbol_declaration(symbol_id));

                if let AstKind::VariableDeclarator(variable_declarator) = node.kind()
                    && let Some(ref_expr) = &variable_declarator.init
                {
                    return is_not_array(ref_expr, ctx);
                }
            }

            ident.name.as_str()
        }
        expr @ match_member_expression!(Expression) => {
            if let Some(v) = expr.to_member_expression().static_property_name() {
                v
            } else {
                return false;
            }
        }
        _ => return false,
    };

    if ident.starts_with(|c: char| c.is_ascii_uppercase())
        && ident.cow_to_ascii_uppercase() != ident
    {
        return true;
    }

    false
}

fn report_with_spread_fixer(
    node: &AstNode,
    ctx: &LintContext,
    span: Span,
    bad_method: &str,
    expr_to_spread: &Expression,
) {
    ctx.diagnostic_with_fix(unicorn_prefer_spread_diagnostic(span, bad_method), |fixer| {
        let needs_semi = ast_util::could_be_asi_hazard(node, ctx);
        let mut codegen = fixer.codegen();
        if needs_semi {
            codegen.print_str(";");
        }
        codegen.print_str("[...");
        codegen.print_expression(expr_to_spread);
        codegen.print_str("]");
        fixer.replace(span, codegen.into_source_text())
    });
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "[...set].map(() => {});",
        "Int8Array.from(set);",
        "Uint8Array.from(set);",
        "Uint8ClampedArray.from(set);",
        "Int16Array.from(set);",
        "Uint16Array.from(set);",
        "Int32Array.from(set);",
        "Uint32Array.from(set);",
        "Float16Array.from(set);",
        "Float32Array.from(set);",
        "Float64Array.from(set);",
        "BigInt64Array.from(set);",
        "BigUint64Array.from(set);",
        "new Array.from(foo);",
        "from(foo);",
        r#"Array["from"](foo);"#,
        "Array[from](foo);",
        "Array.foo(foo);",
        "foo.from(foo);",
        "lib.Array.from(foo);",
        "Array.from();",
        "Array.from(foo, mapFn, thisArg, extra);",
        "Array.from(...argumentsArray);",
        "Array.from(set, mapFn).reduce(() => {});",
        "Array.from(set, mapFn, thisArg).reduce(() => {});",
        "Array.from(set, () => {}, thisArg).reduce(() => {});",
        "Array.from({length: 10});",
        "new Array.concat(1)",
        "concat(1)",
        "array[concat](1)",
        r#""foo".concat("bar")"#,
        r#"`${foo}`.concat("bar")"#,
        r#"const string = 'foo';
            foo = string.concat("bar");"#,
        "const bufA = Buffer.concat([buf1, buf2, buf3], totalLength);",
        "Foo.concat(1)",
        "FooBar.concat(1)",
        "global.Buffer.concat([])",
        r#"["1", "2"].join(",").concat("...")"#,
        r#"foo.join(",").concat("...")"#,
        "foo.join().concat(bar)",
        // "(a + b).concat(c)",
        "new Array.slice()",
        "slice()",
        "array[slice]()",
        "array.slice",
        "array.slice(1)",
        "array.slice(...[])",
        "array.slice(...[0])",
        "array.slice(0 + 0)",
        r#"array.slice("")"#,
        "array.slice(null)",
        "const ZERO = 0;array.slice(ZERO)",
        "array.slice(0, array.length)",
        "array.slice(0, 0)",
        "array.notSlice()",
        "[...foo].slice()",
        "[foo].slice()",
        "arrayBuffer.slice()",
        "blob.slice()",
        "buffer.slice()",
        "file.slice()",
        "class A {foo() {this.slice()}}",
        // TODO: Handle optional chaining case.
        // "scopeManager?.scopes.slice()",
        // TypedArray/ArrayBuffer constructors - spreading doesn't work or changes type
        "new ArrayBuffer(10).slice()",
        "new ArrayBuffer(10).slice(0)",
        "new SharedArrayBuffer(10).slice()",
        "new SharedArrayBuffer(10).slice(0)",
        "new Int8Array([1, 2, 3]).slice()",
        "new Int8Array([1, 2, 3]).slice(0)",
        "new Uint8Array([10, 20, 30, 40, 50]).slice()",
        "new Uint8Array([10, 20, 30, 40, 50]).slice(0)",
        "new Uint8ClampedArray([1, 2, 3]).slice()",
        "new Int16Array([1, 2, 3]).slice()",
        "new Uint16Array([1, 2, 3]).slice()",
        "new Int32Array([1, 2, 3]).slice()",
        "new Uint32Array([1, 2, 3]).slice()",
        "new Float32Array([1, 2, 3]).slice()",
        "new Float64Array([1, 2, 3]).slice()",
        "new BigInt64Array([1n, 2n, 3n]).slice()",
        "new BigUint64Array([1n, 2n, 3n]).slice()",
        "new Array.toSpliced()",
        "toSpliced()",
        "array[toSpliced]()",
        "array.toSpliced",
        "array.toSpliced(0)",
        "array.toSpliced(...[])",
        "array.toSpliced(...[0])",
        "array.toSpliced(0 + 0)",
        r#"array.toSpliced("")"#,
        "array.toSpliced(null)",
        "const ZERO = 0;array.toSpliced(0, ZERO)",
        "array.toSpliced(0, array.length)",
        "array.toSpliced(0, 0)",
        "array.notToSpliced()",
        "[...foo].toSpliced()",
        "[foo].toSpliced()",
        "array.toSpliced(100, 0)",
        "array.toSpliced(-1, 0)",
        r#"new foo.split("")"#,
        r#"split("")"#,
        r#"string[split]("")"#,
        "string.split",
        "string.split(1)",
        r#"string.split(..."")"#,
        r#"string.split(...[""])"#,
        r#"string.split("" + "")"#,
        "string.split(0)",
        "string.split(false)",
        "string.split(undefined)",
        "string.split(0n)",
        "string.split(null)",
        r#"string.split(/""/)"#,
        "string.split(``)",
        r#"const EMPTY_STRING = ""; string.split(EMPTY_STRING)"#,
        r#"string.split("", limit)"#,
        r#""".split(string)"#,
        "string.split()",
        r#"string.notSplit("")"#,
        // r#"const notString = 0; notString.split("")"#,
    ];

    let fail = vec![
        "const x = Array.from(set);",
        "Array.from(set).map(() => {});",
        "Array.from(new Set([1, 2])).map(() => {});",
        r#"Array.from(document.querySelectorAll("*")).map(() => {});"#,
        "const foo = []
            Array.from(arrayLike).forEach(doSomething)",
        r#"const foo = "1"
            Array.from(arrayLike).forEach(doSomething)"#,
        "const foo = null
            Array.from(arrayLike).forEach(doSomething)",
        "const foo = true
            Array.from(arrayLike).forEach(doSomething)",
        "const foo = 1
            Array.from(arrayLike).forEach(doSomething)",
        "const foo = /./
            Array.from(arrayLike).forEach(doSomething)",
        "const foo = /./g
            Array.from(arrayLike).forEach(doSomething)",
        "const foo = bar
            Array.from(arrayLike).forEach(doSomething)",
        "const foo = bar.baz
            Array.from(arrayLike).forEach(doSomething)",
        "function* foo() {
                yield Array.from(arrayLike).forEach(doSomething)
            }",
        r"const foo = \\`bar\\`
            Array.from(arrayLike).forEach(doSomething)",
        "const foo = [];
            Array.from(arrayLike).forEach(doSomething)",
        "for (const key of Array.from(arrayLike)) {
            }",
        "for (const key in Array.from(arrayLike)) {
            }",
        "const foo = `${Array.from(arrayLike)}`",
        "async function foo(){
                return await Array.from(arrayLike)
            }",
        "foo()
            Array.from(arrayLike).forEach(doSomething)",
        "const foo = {}
            Array.from(arrayLike).forEach(doSomething)",
        "(Array).from(foo)",
        "(Array.from)(foo)",
        "((Array).from)(foo)",
        "(Array).from((0, foo))",
        "(Array.from)((0, foo))",
        "((Array).from)((0, foo))",
        "Array.from(a ? b : c)",
        "Array.from((0, a))",
        "Array.from([...a, ...b], )",
        "Array.from([1])",
        "Array.from([...a, ...b])",
        "/* 1 */ Array /* 2 */ .from /* 3 */ ( /* 4 */ a /* 5 */,)",
        "[1].concat(2)",
        "[1].concat([2, 3])",
        "[1].concat(2,)",
        "[1].concat([2, ...bar],)",
        "[1,].concat(2)",
        "[1,].concat([2, 3])",
        "[1,].concat(2,)",
        "[1,].concat([2, 3],)",
        "(( (( (( [1,] )).concat ))( (([2, 3])) ,) ))",
        "(( (( (( [1,] )).concat ))( (([2, 3])) , bar ) ))",
        "foo.concat(2)",
        "foo.concat([2, 3])",
        "foo.concat(2,)",
        "foo.concat([2, 3],)",
        "(( (( ((foo)).concat ))( (([2, 3])) ,) ))",
        "(( (( ((foo)).concat ))( (([2, 3])) , bar ) ))",
        "bar()
            foo.concat(2)",
        "const foo = foo.concat(2)",
        "const foo = () => foo.concat(2)",
        "const five = 2 + 3;
            foo.concat(five);",
        "const array = [2 + 3];
            foo.concat(array);",
        "foo.concat([bar])",
        "foo.concat(bar)",
        "Array.from(set).concat([2, 3])",
        "foo.concat([2, 3]).concat(4)",
        r#"string.concat("bar")"#,
        "foo.concat(2, 3)",
        "foo.concat(2, bar)",
        "[...foo, 2].concat(bar)",
        "let sortedScores = scores.concat().sort((a, b) => b[0] - a[0]);",
        "foo.concat(bar, 2, 3)",
        "foo.concat(bar, 2, 3, baz)",
        "async function a() {return [].concat(await bar)}",
        "async function a() {return [].concat((0, bar))}",
        "async function a() {return [].concat(((await bar)))}",
        "foo.concat((0, 1))",
        "async function a() {return (await bar).concat(1)}",
        "[].concat(...bar)",
        "[].concat([,], [])",
        "[,].concat([,], [,])",
        "[,].concat([,,], [,])",
        "[,].concat([,], [,,])",
        "[1].concat([2,], [3,])",
        "[1].concat([2,,], [3,,])",
        "[1,].concat([2,], [3,])",
        "[1,].concat([2,,], [3,,])",
        "[].concat([], [])",
        r#"const EMPTY_STRING = ""
            const EMPTY_STRING_IN_ARRAY = ""
            const EMPTY_STRING_IN_ARRAY_OF_ARRAY = ""
            const array = [].concat(
                undefined,
                null,
                EMPTY_STRING,
                false,
                0,
                [EMPTY_STRING_IN_ARRAY],
                [[EMPTY_STRING_IN_ARRAY_OF_ARRAY]]
            )"#,
        "[].concat((a.b.c), 2)",
        "[].concat(a.b(), 2)",
        "foo.concat(bar, 2, [3, 4], baz, 5, [6, 7])",
        "foo.concat(bar, 2, 3, ...baz)",
        "notClass.concat(1)",
        "_A.concat(1)",
        "FOO.concat(1)",
        "A.concat(1)",
        "Foo.x.concat(1)",
        "if (test) foo.concat(1)",
        "if (test) {} else foo.concat(1)",
        "if (test) {} else foo.concat(1)",
        "for (;;) foo.concat(1)",
        "for (a in b) foo.concat(1)",
        "for (a in b) foo.concat(1)",
        "for (const a of b) foo.concat(1)",
        "while (test) foo.concat(1)",
        "do foo.concat(1); while (test)",
        "with (foo) foo.concat(1)", // {"parserOptions": {"sourceType": "script"}},
        "const baz = [2];
            call(foo, ...[bar].concat(baz));",
        r#"foo.join(foo, bar).concat("...")"#,
        "array.slice()",
        "array.slice().slice()",
        "array.slice(1).slice()",
        "array.slice().slice(1)",
        "const copy = array.slice()",
        "(( (( (( array )).slice ))() ))",
        "(scopeManager?.scopes).slice()",
        "bar()
            foo.slice()",
        r#""".slice()"#,
        "array.slice(0)",
        "array.slice(0b0)",
        "array.slice(0.00)",
        "array.slice(0.00, )",
        "array.toSpliced()",
        "array.toSpliced().toSpliced()",
        "const copy = array.toSpliced()",
        "(( (( (( array )).toSpliced ))() ))",
        "bar()
            foo.toSpliced()",
        r#""".toSpliced()"#,
        "new Uint8Array([10, 20, 30, 40, 50]).toSpliced()",
        r#""string".split("")"#,
        r#""string".split('')"#,
        r#"unknown.split("")"#,
        r#"const characters = "string".split("")"#,
        r#"(( (( (( "string" )).split ))( (("")) ) ))"#,
        r#"bar()
            foo.split("")"#,
        r#"unknown.split("")"#,
        r#""🦄".split("")"#,
        r#"const {length} = "🦄".split("")"#,
    ];

    let fix = vec![
        // `Array.from()`
        ("const x = Array.from(set);", "const x = [...set];"),
        ("Array.from(new Set([1, 2])).map(() => {});", "[...new Set([1, 2])].map(() => {});"),
        // `Array.from()` - ASI hazard cases (need semicolon prefix)
        (
            "const foo = bar\nArray.from(set).map(() => {})",
            "const foo = bar\n;[...set].map(() => {})",
        ),
        ("foo()\nArray.from(set).forEach(doSomething)", "foo()\n;[...set].forEach(doSomething)"),
        // `Array.from()` - No ASI hazard (semicolon already present)
        (
            "const foo = bar;\nArray.from(set).map(() => {})",
            "const foo = bar;\n[...set].map(() => {})",
        ),
        // `Array.from()` - ASI hazard with comments before
        (
            "foo() /* comment */\nArray.from(set).map(() => {})",
            "foo() /* comment */\n;[...set].map(() => {})",
        ),
        (
            "foo() // comment\nArray.from(set).map(() => {})",
            "foo() // comment\n;[...set].map(() => {})",
        ),
        // `array.slice()`
        ("array.slice()", "[...array]"),
        ("array.slice(1).slice()", "[...array.slice(1)]"),
        // `array.slice()` - ASI hazard cases
        ("foo()\narray.slice()", "foo()\n;[...array]"),
        // `array.toSpliced()`
        ("array.toSpliced()", "[...array]"),
        ("const copy = array.toSpliced()", "const copy = [...array]"),
        // `array.toSpliced()` - ASI hazard cases
        ("foo()\narray.toSpliced()", "foo()\n;[...array]"),
        // `string.split()`
        (r#""🦄".split("")"#, r#"[..."🦄"]"#),
        (r#""foo bar baz".split("")"#, r#"[..."foo bar baz"]"#),
        // `string.split()` - ASI hazard cases
        ("foo()\nstr.split(\"\")", "foo()\n;[...str]"),
        (
            r"Array.from(path.matchAll(/\{([^{}?]+\??)\}/g))",
            "[...path.matchAll(/\\{([^{}?]+\\??)\\}/g)]",
        ),
        // Cases where NO semicolon should be added (not an ExpressionStatement)
        ("return Array.from(set)", "return [...set]"),
        ("const x = Array.from(set)", "const x = [...set]"),
        ("foo(Array.from(set))", "foo([...set])"),
        ("if (Array.from(set).length) {}", "if ([...set].length) {}"),
        // `Array.from()` - ASI hazard with multi-byte Unicode identifiers
        ("日本語\nArray.from(set).map(() => {})", "日本語\n;[...set].map(() => {})"),
        (
            "const foo = 日本語\nArray.from(set).map(() => {})",
            "const foo = 日本語\n;[...set].map(() => {})",
        ),
        ("/**/Array.from(set).map(() => {})", "/**/[...set].map(() => {})"),
        ("/regex/\nArray.from(set).map(() => {})", "/regex/\n;[...set].map(() => {})"),
        ("/regex/g\nArray.from(set).map(() => {})", "/regex/g\n;[...set].map(() => {})"),
        ("0.\nArray.from(set).map(() => {})", "0.\n;[...set].map(() => {})"),
        ("foo()\u{00A0}\nArray.from(set).map(() => {})", "foo()\u{00A0}\n;[...set].map(() => {})"),
        ("foo()\u{FEFF}\nArray.from(set).map(() => {})", "foo()\u{FEFF}\n;[...set].map(() => {})"),
        ("foo() /* a */ /* b */\nArray.from(set)", "foo() /* a */ /* b */\n;[...set]"),
        ("x++\narray.slice()", "x++\n;[...array]"),
        ("x--\narray.slice()", "x--\n;[...array]"),
        ("arr[0]\narray.slice()", "arr[0]\n;[...array]"),
        ("obj.prop\narray.slice()", "obj.prop\n;[...array]"),
        ("while (array.slice().length) {}", "while ([...array].length) {}"),
        ("do {} while (array.slice().length)", "do {} while ([...array].length)"),
        ("for (array.slice();;) {}", "for ([...array];;) {}"),
        ("switch (array.slice()[0]) {}", "switch ([...array][0]) {}"),
        ("`template`\narray.toSpliced()", "`template`\n;[...array]"),
        (
            r#"'string'
str.split("")"#,
            "'string'\n;[...str]",
        ),
        (
            r#""string"
str.split("")"#,
            r#""string"
;[...str]"#,
        ),
        (
            "foo()\nArray.from(set).map(x => x).filter(Boolean).length",
            "foo()\n;[...set].map(x => x).filter(Boolean).length",
        ),
        ("const fn = () => Array.from(set)", "const fn = () => [...set]"),
        ("foo ? Array.from(a) : b", "foo ? [...a] : b"),
        ("foo || Array.from(set)", "foo || [...set]"),
        ("foo && Array.from(set)", "foo && [...set]"),
        ("foo + Array.from(set).length", "foo + [...set].length"),
        ("x = Array.from(set)", "x = [...set]"),
        ("const obj = { arr: Array.from(set) }", "const obj = { arr: [...set] }"),
        ("(foo, Array.from(set))", "(foo, [...set])"),
        ("[Array.from(set)]", "[[...set]]"),
        ("async () => await Array.from(set)", "async () => await [...set]"),
    ];

    Tester::new(PreferSpread::NAME, PreferSpread::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
