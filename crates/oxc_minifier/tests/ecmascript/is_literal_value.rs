use std::iter;

use javascript_globals::GLOBALS;

use rustc_hash::FxHashSet;

use oxc_allocator::Allocator;
use oxc_ast::ast::{IdentifierReference, Statement};
use oxc_ecmascript::{GlobalContext, constant_evaluation::IsLiteralValue};
use oxc_parser::Parser;
use oxc_span::SourceType;

struct Ctx {
    global_variable_names: FxHashSet<&'static str>,
}

impl Default for Ctx {
    fn default() -> Self {
        Self {
            global_variable_names: GLOBALS["builtin"]
                .keys()
                .copied()
                .chain(iter::once("arguments"))
                .collect::<FxHashSet<_>>(),
        }
    }
}

impl<'a> GlobalContext<'a> for Ctx {
    fn is_global_reference(&self, ident: &IdentifierReference<'a>) -> bool {
        self.global_variable_names.contains(ident.name.as_str())
    }
}

#[track_caller]
fn test(source_text: &str, expected: bool) {
    let ctx = Ctx::default();
    test_with_ctx(source_text, &ctx, expected);
}

#[track_caller]
fn test_include_functions(source_text: &str, expected: bool) {
    let ctx = Ctx::default();
    test_with_ctx_and_functions_option(source_text, true, &ctx, expected);
}

#[track_caller]
fn test_with_global_variables(
    source_text: &str,
    global_variable_names: &[&'static str],
    expected: bool,
) {
    let ctx = Ctx { global_variable_names: global_variable_names.iter().copied().collect() };
    test_with_ctx(source_text, &ctx, expected);
}

#[track_caller]
fn test_with_ctx(source_text: &str, ctx: &Ctx, expected: bool) {
    test_with_ctx_and_functions_option(source_text, false, ctx, expected);
}

#[track_caller]
fn test_with_ctx_and_functions_option(
    source_text: &str,
    include_functions: bool,
    ctx: &Ctx,
    expected: bool,
) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, SourceType::mjs()).parse();
    assert!(!ret.panicked, "{source_text}");
    assert!(ret.errors.is_empty(), "{source_text}");

    let Some(Statement::ExpressionStatement(stmt)) = &ret.program.body.first() else {
        panic!("should have a expression statement body: {source_text}");
    };
    assert_eq!(stmt.expression.is_literal_value(include_functions, ctx), expected, "{source_text}");
}

#[test]
fn test_identifier_reference() {
    test("NaN", true);
    test("undefined", true);
    test("Infinity", true);
    test_with_global_variables("NaN", &[], false);
}

#[test]
fn test_simple_expressions() {
    test("true", true);
    test("null", true);
    test("0", true);
    test("/foo/", true);
    test("('foo')", true);
    test("0n", true);
    test("this", false);
    test("import.meta", false);
    test("(() => {})", false);
    test("(function(){})", false);
    test_include_functions("(() => {})", true);
    test_include_functions("(function(){})", true);
}

#[test]
fn test_template_literal() {
    test("``", true);
    test("`a`", true);
    test("`${1}`", false); // can improve
    test("`${[]}`", false); // Array::[Symbol.toPrimitive] might be overridden
    test("`${a}`", false);
}

#[test]
fn test_unary_expressions() {
    test("void 'foo'", true);
    test("void foo", false);
    test("!'foo'", true);
    test("!foo", false);
    test("typeof 'foo'", true);
    test("typeof foo", false);

    test("+0", true);
    test("+0n", false);
    test("+undefined", true); // NaN
    test("+Infinity", true);
    test("+NaN", true);
    test("+null", true); // 0
    test("+false", true); // 0
    test("+true", true); // 1
    test("+'foo'", true); // NaN
    test("+`foo`", true); // NaN
    test("+/foo/", false); // RegExp::[Symbol.toPrimitive] might be overridden
    test("+[]", false); // Array::[Symbol.toPrimitive] might be overridden

    test("+(() => {})", false);
    test("+(function () {})", false);
    test_include_functions("+(() => {})", true);
    test_include_functions("+(function () {})", true);
    test("+(void 1)", true);
    test("+(void a)", false);
    test("+(!1)", true);
    test("+(!1n)", true);
    test("+(!a)", false);
    test("+(typeof 1)", true);
    test("+(typeof a)", false);
    test("+(+1)", true);
    test("+(+1n)", false);
    test("+(+a)", false);
    test("+(-1)", true);
    test("+(-1n)", false);
    test("+(-a)", false);
    test("+(~1)", true);
    test("+(~1n)", false);
    test("+(~a)", false);
    test("+(delete a)", false);
    test("+('' + true)", true);
    test("+('' + 2)", true);
    test("+(1 + 2)", true);
    test("+(1n + 2n)", true);
    test("+('' + a)", false);
    test("+(1 - 2)", true);
    test("+(1 - a)", false);
    test("+(1 < 2)", false);
    test("+(1 || 2)", true);
    test("+(a || 2)", false);
    test("+(1 || b)", false);
    test("+(1n || 2)", false);
    test("+(1 ? 2 : 3)", true);
    test("+(1n ? 2 : 3)", true);
    test("+(1 ? 2n : 3)", false);
    test("+(1 ? 2 : 3n)", false);
    test("+(1)", true);
    test("+(a)", false);
    test("+(1, 2)", true);
    test("+(1n, 2)", true);
    test("+(1, 2n)", false);

    test("-0", true);
    test("-0n", true);
    test("-undefined", true); // NaN
    test("-Infinity", true);
    test("-NaN", true);
    test("-null", true); // -0
    test("-false", true); // -0
    test("-true", true); // -1
    test("-'foo'", true); // NaN
    test("-`foo`", true); // NaN
    test("-/foo/", false); // RegExp::[Symbol.toPrimitive] might be overridden
    test("-[]", false); // Array::[Symbol.toPrimitive] might be overridden

    test("~0", true);
    test("~'foo'", true);
    test("~foo", false);

    test("delete 0", false);
    test("delete foo", false);
}

#[test]
fn test_logical_expressions() {
    test("a || b", false);
    test("a || 0", false);
    test("1 || b", false);
    test("1 || 0", true);
    test("a && b", false);
    test("a && 0", false);
    test("1 && b", false);
    test("1 && 0", true);
    test("a ?? b", false);
    test("a ?? 0", false);
    test("1 ?? b", false);
    test("1 ?? 0", true);
}

#[test]
fn test_other_expressions() {
    test("(1)", true);
    test("(foo)", false);

    test("1 ? 2 : 3", true);
    test("1 ? 2 : c", false);
    test("a ? 2 : 3", false);
    test("1 ? b : 3", false);

    test("1, 2", true);
    test("a, 2", false);
    test("1, b", false);
}

#[test]
fn test_array_expression() {
    test("[]", true);
    test("[1]", true);
    test("[foo]", false);
    test("[,]", true);
    test("[...a]", false);
    test("[...[]]", false); // Array::[Symbol.iterator] might be overridden
    test("[...'foo']", false); // String::[Symbol.iterator] might be overridden
}

#[test]
fn test_object_expression() {
    // wrapped with parentheses to avoid treated as a block statement
    test("({})", true);
    test("({a: 1})", true);
    test("({a: foo})", false);
    test("({1: 1})", true);
    test("({[1]: 1})", true);
    test("({[1n]: 1})", true);
    test("({['1']: 1})", true);
    test("({[a]: 1})", false);
    test("({[[]]: 1})", false); // Array::[Symbol.toPrimitive] might be overridden

    test("({[()=>{}]: 1})", false);
    test("({[function(){}]: 1})", false);
    test_include_functions("({[()=>{}]: 1})", true);
    test_include_functions("({[function(){}]: 1})", true);
    test("({[void 0]: 1})", true);
    test("({[void a]: 1})", false);
    test("({[!0]: 1})", true);
    test("({[!a]: 1})", false);
    test("({[typeof 0]: 1})", true);
    test("({[typeof a]: 1})", false);
    test("({[+0]: 1})", true);
    test("({[-0]: 1})", true);
    test("({[~0]: 1})", true);
    test("({[delete a]: 1})", false);
    test("({['' + 2]: 1})", true);
    test("({[1 + 2]: 1})", true);
    test("({[1n + 2n]: 1})", true);
    test("({['' + a]: 1})", false);
    test("({[1 - 2]: 1})", true);
    test("({[1 - a]: 1})", false);
    test("({[1 < 2]: 1})", false);
    test("({['foo' || 'bar']: 1})", true);
    test("({[a || 'bar']: 1})", false);
    test("({['foo' || a]: 1})", false);
    test("({[1 ? 'foo' : 'bar']: 1})", true);
    test("({[1 ? a : 'bar']: 1})", false);
    test("({[1 ? 'foo' : b]: 1})", false);
    test("({[('foo')]: 1})", true);
    test("({[(a)]: 1})", false);
    test("({[(1, 'foo')]: 1})", true);
    test("({[(1, b)]: 1})", false);

    test("({...a})", false);
    test("({...[]})", true);
    test("({...[0]})", true);
    test("({...[a]})", false);
    test("({...'foo'})", true);
    test("({...`foo`})", true);
    test("({...`foo${a}`})", false);
    test("({...{}})", true);
    test("({...{a:1}})", true);
    test("({...{a:a}})", false);
}

#[test]
fn test_binary_expressions() {
    test("1 === 2", true);
    test("a === 2", false);
    test("1 === b", false);
    test("a === b", false);
    test("1 !== 2", true);
    test("a !== b", false);

    test("'' + ''", true);
    test("'' + ``", true);
    test("'' + null", true);
    test("'' + 0", true);
    test("'' + a", false);
    test("null + ''", true);
    test("0 + ''", true);
    test("a + ''", false);
    test("1 + 2", true);
    test("1n + 2n", true);

    test("0n - 1n", true);
    test("0n - 0", false);
    test("0n - a", false);
    test("a - 0n", false);
    test("0 - 1", true);
    test("0 - a", false);
    test("a - 0", false);
    test("0 - ''", true); // 0
    test("0 - ``", true); // 0
    test("0 - true", true); // -1
    test("0 - []", false); // Array::[Symbol.toPrimitive] might be overridden

    test("0 * 1", true);
    test("0 * a", false);
    test("0 / 1", true);
    test("0 / a", false);
    test("0 % 1", true);
    test("0 % a", false);
    test("0 ** 1", true);
    test("0 ** a", false);
    test("0 << 1", true);
    test("0 << a", false);
    test("0 >> 1", true);
    test("0 >> a", false);
    test("0 >>> 1", true);
    test("0 >>> a", false);
    test("0 | 1", true);
    test("0 | a", false);
    test("0 ^ 1", true);
    test("0 ^ a", false);
    test("0 & 1", true);
    test("0 & a", false);

    test("1n ** (-1n)", false); // `**` throws an error when the right operand is negative
    test("1n / 0n", false); // `/` throws an error when the right operand is zero
    test("1n % 0n", false); // `%` throws an error when the right operand is zero
    test("0n >>> 1n", false); // `>>>` throws an error even when both operands are bigint

    test("1 == 2", false);
    test("1 != 2", false);
    test("1 < 2", false);
    test("1 <= 2", false);
    test("1 > 2", false);
    test("1 >= 2", false);
    test("1 in 2", false);
    test("1 instanceof 2", false);
}
