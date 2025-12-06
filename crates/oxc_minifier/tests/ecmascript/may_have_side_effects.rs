use javascript_globals::GLOBALS;

use rustc_hash::FxHashSet;

use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, IdentifierReference, Statement};
use oxc_ecmascript::{
    GlobalContext,
    side_effects::{
        MayHaveSideEffects, MayHaveSideEffectsContext, PropertyReadSideEffects, is_pure_function,
    },
};
use oxc_parser::Parser;
use oxc_span::SourceType;

struct Ctx {
    global_variable_names: FxHashSet<&'static str>,
    annotation: bool,
    pure_function_names: Vec<String>,
    property_read_side_effects: PropertyReadSideEffects,
    unknown_global_side_effects: bool,
}

impl Default for Ctx {
    fn default() -> Self {
        Self {
            global_variable_names: GLOBALS["builtin"]
                .keys()
                .copied()
                .chain(["arguments", "URL"])
                .collect::<FxHashSet<_>>(),
            annotation: true,
            pure_function_names: vec![],
            property_read_side_effects: PropertyReadSideEffects::All,
            unknown_global_side_effects: true,
        }
    }
}

impl<'a> GlobalContext<'a> for Ctx {
    fn is_global_reference(&self, ident: &IdentifierReference<'a>) -> bool {
        self.global_variable_names.contains(ident.name.as_str())
    }
}

impl MayHaveSideEffectsContext<'_> for Ctx {
    fn annotations(&self) -> bool {
        self.annotation
    }

    fn manual_pure_functions(&self, callee: &Expression) -> bool {
        is_pure_function(callee, &self.pure_function_names)
    }

    fn property_read_side_effects(&self) -> PropertyReadSideEffects {
        self.property_read_side_effects
    }

    fn unknown_global_side_effects(&self) -> bool {
        self.unknown_global_side_effects
    }
}

#[track_caller]
fn test(source_text: &str, expected: bool) {
    let ctx = Ctx::default();
    test_with_ctx(source_text, &ctx, expected);
}

#[track_caller]
fn test_with_global_variables(
    source_text: &str,
    global_variable_names: &[&'static str],
    expected: bool,
) {
    let ctx = Ctx {
        global_variable_names: global_variable_names.iter().copied().collect(),
        ..Default::default()
    };
    test_with_ctx(source_text, &ctx, expected);
}

#[track_caller]
fn test_with_ctx(source_text: &str, ctx: &Ctx, expected: bool) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, SourceType::mjs()).parse();
    assert!(!ret.panicked, "{source_text}");
    assert!(ret.errors.is_empty(), "{source_text}");

    let Some(Statement::ExpressionStatement(stmt)) = &ret.program.body.first() else {
        panic!("should have a expression statement body: {source_text}");
    };
    assert_eq!(stmt.expression.may_have_side_effects(ctx), expected, "{source_text}");
}

#[track_caller]
fn test_in_function(source_text: &str, expected: bool) {
    let ctx = Ctx::default();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, SourceType::mjs()).parse();
    assert!(!ret.panicked, "{source_text}");
    assert!(ret.errors.is_empty(), "{source_text}");

    let Some(Statement::FunctionDeclaration(stmt)) = &ret.program.body.first() else {
        panic!("should have a function declaration: {source_text}");
    };
    let Some(Statement::ExpressionStatement(stmt)) =
        &stmt.body.as_ref().expect("should have a body").statements.first()
    else {
        panic!("should have a expression statement body: {source_text}");
    };

    assert_eq!(stmt.expression.may_have_side_effects(&ctx), expected, "{source_text}");
}

#[track_caller]
fn test_assign_target(source_text: &str, expected: bool) {
    test_assign_target_with_global_variables(source_text, &[], expected);
}

#[track_caller]
fn test_assign_target_with_global_variables(
    source_text: &str,
    global_variable_names: &[&'static str],
    expected: bool,
) {
    let ctx = Ctx {
        global_variable_names: global_variable_names.iter().copied().collect(),
        ..Default::default()
    };
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, SourceType::mjs()).parse();
    assert!(!ret.panicked, "{source_text}");
    assert!(ret.errors.is_empty(), "{source_text}");

    let Some(Statement::ExpressionStatement(stmt)) = &ret.program.body.first() else {
        panic!("should have a expression statement body: {source_text}");
    };
    let Expression::AssignmentExpression(assign_expr) = &stmt.expression.without_parentheses()
    else {
        panic!("should have a assignment expression: {source_text}");
    };

    assert_eq!(assign_expr.left.may_have_side_effects(&ctx), expected, "{source_text}");
}

/// <https://github.com/google/closure-compiler/blob/v20240609/test/com/google/javascript/jscomp/AstAnalyzerTest.java#L362>
#[test]
fn closure_compiler_tests() {
    test("[1]", false);
    test("[1, 2]", false);
    test("i++", true);
    test("[b, [a, i++]]", true);
    test("i=3", true);
    test("[0, i=3]", true);
    test("b()", true);
    test("[1, b()]", true);
    test("b.b=4", true);
    test("b.b--", true);
    test("i--", true);
    test("a[0][i=4]", true);
    test("a += 3", true);
    test("a, b, z += 4", true);
    test("a ? c : d++", true);
    test("a ?? b++", true);
    test("a + c++", true);
    test("a + c - d()", true);
    test("a + c - d()", true);
    // test("function foo() {}", true);
    // test("class Foo {}", true);
    // test("while(true);", true);
    // test("if(true){a()}", true);
    // test("if(true){a}", false);
    test("(function() { })", false);
    test("(function() { i++ })", false);
    test("[function a(){}]", false);
    test("(class { })", false);
    test("(class { method() { i++ } })", false);
    test("(class { [computedName()]() {} })", true);
    test("(class { [computedName]() {} })", false);
    test("(class Foo extends Bar { })", false);
    test("(class extends foo() { })", true);
    test("a", false);
    test("a.b", true);
    test("a.b.c", true);
    test("[b, c, [d, [e]]]", false);
    test("({a: x, b: y, c: z})", false);
    test("({a, b, c})", false);
    test("/abc/gi", false);
    test("('a')", false); // wrapped with parentheses to avoid treated as a directive
    test("0", false);
    test("a + c", true);
    test("'c' + a[0]", true);
    test("a[0][1]", true);
    test("'a' + c", true);
    test("'a' + a.name", true);
    test("1, 2, 3", false);
    test("a, b, 3", false);
    test("(function(a, b) {  })", false);
    test("a ? c : d", false);
    test("a ?? b", false);
    // test("'1' + navigator.userAgent", false);
    test("`template`", false);
    test("`template${name}`", true);
    test("`${name}template`", true);
    test("`${naming()}template`", true);
    test("templateFunction`template`", true);
    test("st = `${name}template`", true);
    test("tempFunc = templateFunction`template`", true);
    test("new RegExp('foobar', 'i')", false);
    test("new RegExp('foobar', 2)", true);
    test("new RegExp(SomethingWacky(), 'i')", true);
    // test("new Array()", false);
    // test("new Array", false);
    // test("new Array(4)", false);
    // test("new Array('a', 'b', 'c')", false);
    test("new SomeClassINeverHeardOf()", true);
    test("new SomeClassINeverHeardOf()", true);
    // test("({}).foo = 4", false);
    // test("([]).foo = 4", false);
    // test("(function() {}).foo = 4", false);
    test("this.foo = 4", true);
    test("a.foo = 4", true);
    test("(function() { return n; })().foo = 4", true);
    test("([]).foo = bar()", true);
    test("undefined", false);
    test("void 0", false);
    test("void foo()", true);
    test("-Infinity", false);
    test("Infinity", false);
    test("NaN", false);
    // test("({}||[]).foo = 2;", false);
    // test("(true ? {} : []).foo = 2;", false);
    // test("({},[]).foo = 2;", false);
    test("delete a.b", true);
    test("Math.random();", false);
    test("Math.random(Math);", true);
    // test("[1, 1].foo;", false);
    // test("export var x = 0;", true);
    // test("export let x = 0;", true);
    // test("export const x = 0;", true);
    // test("export class X {};", true);
    // test("export function x() {};", true);
    // test("export {x};", true);

    // ARRAYLIT-ITER_SPREAD
    test("[...[]]", false);
    test("[...[1]]", false);
    test("[...[i++]]", true);
    test("[...'string']", false);
    test("[...`templatelit`]", false);
    test("[...`templatelit ${safe}`]", true);
    test("[...`templatelit ${unsafe()}`]", true);
    test("[...f()]", true);
    test("[...5]", true);
    test("[...null]", true);
    test("[...true]", true);

    // CALL-ITER_SPREAD
    // test("Math.sin(...[])", false);
    // test("Math.sin(...[1])", false);
    test("Math.sin(...[i++])", true);
    // test("Math.sin(...'string')", false);
    // test("Math.sin(...`templatelit`)", false);
    // test("Math.sin(...`templatelit ${safe}`)", true);
    test("Math.sin(...`templatelit ${unsafe()}`)", true);
    test("Math.sin(...f())", true);
    test("Math.sin(...5)", true);
    test("Math.sin(...null)", true);
    test("Math.sin(...true)", true);

    // NEW-ITER_SPREAD
    // test("new Object(...[])", false);
    // test("new Object(...[1])", false);
    test("new Object(...[i++])", true);
    // test("new Object(...'string')", false);
    // test("new Object(...`templatelit`)", false);
    // test("new Object(...`templatelit ${safe}`)", true);
    test("new Object(...`templatelit ${unsafe()}`)", true);
    test("new Object(...f())", true);
    test("new Object(...5)", true);
    test("new Object(...null)", true);
    test("new Object(...true)", true);

    // OBJECT_SPREAD
    // These could all invoke getters.
    test("({...x})", true);
    test("({...{}})", true);
    test("({...{a:1}})", true);
    test("({...{a:i++}})", true);
    test("({...{a:f()}})", true);
    test("({...f()})", true);

    // OBJECT_REST
    // This could invoke getters.
    test("({...x} = something)", true);
    // the presence of `a` affects what goes into `x`
    test("({a, ...x} = something)", true);

    // ITER_REST
    // We currently assume all iterable-rests are side-effectful.
    test("([...x] = 'safe')", true);
    test("(function(...x) { })", false);

    // Context switch
    // test("async function f() { await 0; }", true);
    // test("(async()=>{ for await (let x of []) {} })", true);
    // test("function f() { throw 'something'; }", true);
    // test("function* f() { yield 'something'; }", true);
    // test("function* f() { yield* 'something'; }", true);

    // Enhanced for loop
    // These edge cases are actually side-effect free. We include them to confirm we just give
    // up on enhanced for loops.
    // test("for (const x in []) { }", true);
    // test("for (const x of []) { }", true);

    // COMPUTED_PROP - OBJECTLIT
    test("({[a]: x})", false);
    test("({[a()]: x})", true);
    test("({[a]: x()})", true);

    // computed property getters and setters are modeled as COMPUTED_PROP with an
    // annotation to indicate getter or setter.
    test("({ get [a]() {} })", false);
    test("({ get [a()]() {} })", true);
    test("({ set [a](x) {} })", false);
    test("({ set [a()](x) {} })", true);

    // COMPUTED_PROP - CLASS
    test("(class C { [a]() {} })", false);
    test("(class C { [a()]() {} })", true);

    // computed property getters and setters are modeled as COMPUTED_PROP with an
    // annotation to indicate getter or setter.
    test("(class C { get [a]() {} })", false);
    test("(class C { get [a()]() {} })", true);
    test("(class C { set [a](x) {} })", false);
    test("(class C { set [a()](x) {} })", true);

    // GETTER_DEF
    test("({ get a() {} })", false);
    test("(class C { get a() {} })", false);

    // Getter use
    test("x.normal;", true);
    test("x?.normal;", true);
    test("({normal} = foo());", true);

    // SETTER_DEF
    test("({ set a(x) {} })", false);
    test("(class C { set a(x) {} })", false);

    // SETTER_USE
    test("x.normal = 0;", true);

    // MEMBER_FUNCTION_DEF
    test("({ a(x) {} })", false);
    test("(class C { a(x) {} })", false);

    // MEMBER_FIELD_DEF
    test("(class C { x=2; })", false);
    test("(class C { x; })", false);
    test("(class C { x })", false);
    test("(class C { x \n y })", false);
    test("(class C { static x=2; })", false);
    test("(class C { static x; })", false);
    test("(class C { static x })", false);
    test("(class C { static x \n static y })", false);
    test("(class C { x = alert(1); })", false);
    test("(class C { static x = alert(1); })", true);

    // COMPUTED_FIELD_DEF
    test("(class C { [x]; })", false);
    test("(class C { ['x']=2; })", false);
    test("(class C { 'x'=2; })", false);
    test("(class C { 1=2; })", false);
    test("(class C { static [x]; })", false);
    test("(class C { static ['x']=2; })", false);
    test("(class C { static 'x'=2; })", false);
    test("(class C { static 1=2; })", false);
    test("(class C { ['x'] = alert(1); })", false);
    test("(class C { static ['x'] = alert(1); })", true);
    test("(class C { static [alert(1)] = 2; })", true);

    // CLASS_STATIC_BLOCK
    test("(class C { static {} })", false);
    test("(class C { static { [1]; } })", false);
    test("(class C { static { let x; } })", false);
    test("(class C { static { const x =1 ; } })", false);
    test("(class C { static { var x; } })", false);
    test("(class C { static { this.x = 1; } })", true);
    test("(class C { static { function f() { } } })", false);
    test("(class C { static { (function () {} )} })", false);
    test("(class C { static { ()=>{} } })", false);

    // SUPER calls
    test("super()", true);
    test("super.foo()", true);

    // A RegExp Object by itself doesn't have any side-effects
    test("/abc/gi", false);

    // RegExp instance methods have global side-effects, so whether they are
    // considered side-effect free depends on whether the global properties
    // are referenced.
    test("(/abc/gi).test('')", true);
    test("(/abc/gi).test(a)", true);
    test("(/abc/gi).exec('')", true);

    // Some RegExp object method that may have side-effects.
    test("(/abc/gi).foo('')", true);

    // Try the string RegExp ops.
    test("''.match('a')", true);
    test("''.match(/(a)/)", true);
    test("''.replace('a')", true);
    test("''.search('a')", true);
    test("''.split('a')", true);

    // Some non-RegExp string op that may have side-effects.
    test("''.foo('a')", true);

    // 'a' might be a RegExp object with the 'g' flag, in which case
    // the state might change by running any of the string ops.
    // Specifically, using these methods resets the "lastIndex" if used
    // in combination with a RegExp instance "exec" method.
    test("''.match(a)", true);

    // Dynamic import changes global state
    test("import('./module.js')", true);
}

#[test]
fn test_identifier_reference() {
    // accessing global variables may have a side effect
    test_with_global_variables("a", &["a"], true);
    // accessing known globals are side-effect free
    test("NaN", false);
}

#[test]
fn test_simple_expressions() {
    test("1n", false);
    test("true", false);
    test("this", false);
    test("import.meta", false);
    test("(() => {})", false);
}

#[test]
fn test_template_literal() {
    test("``", false);
    test("`a`", false);
    test("`${1}`", false);
    test("`${[]}`", false);
    test("`${Symbol()}`", true);
    test("`${{ toString() { console.log('sideeffect') } }}`", true);
    test("`${{ valueOf() { console.log('sideeffect') } }}`", true);
    test("`${{ [s]() { console.log('sideeffect') } }}`", true); // assuming s is Symbol.toPrimitive
    test("`${a}`", true); // a maybe a symbol
    test("`${a()}`", true);
    test("`${a() === b}`", true);
}

#[test]
fn test_unary_expressions() {
    test("delete 'foo'", true);
    test("delete foo()", true);

    test("void 'foo'", false);
    test("void foo()", true);
    test("!'foo'", false);
    test("!foo()", true);

    test("typeof 'foo'", false);
    test_with_global_variables("typeof a", &["a"], false);
    test_with_global_variables("typeof (0, a)", &["a"], true);
    test("typeof foo()", true);

    test("+0", false);
    test("+0n", true);
    test("+null", false); // 0
    test("+true", false); // 1
    test("+'foo'", false); // NaN
    test("+`foo`", false); // NaN
    test("+/foo/", false); // NaN
    test("+Infinity", false);
    test("+NaN", false);
    test("+undefined", false); // NaN
    test("+[]", false); // 0
    test("+[foo()]", true);
    test("+foo()", true);
    test("+foo", true); // foo can be Symbol or BigInt
    test("+Symbol()", true);
    test("+{}", false); // NaN
    test("+{ valueOf() { return Symbol() } }", true);

    test("-0", false);
    test("-0n", false);
    test("-null", false); // -0
    test("-true", false); // -1
    test("-'foo'", false); // -NaN
    test("-`foo`", false); // NaN
    test("-/foo/", false); // NaN
    test("-Infinity", false);
    test("-NaN", false);
    test("-undefined", false); // NaN
    test("-[]", false); // -0
    test("-[foo()]", true);
    test("-foo()", true);
    test("-foo", true); // foo can be Symbol
    test("-Symbol()", true);
    test("-{}", false); // NaN
    test("-{ valueOf() { return Symbol() } }", true);

    test("~0", false);
    test("~'foo'", false);
    test("~foo()", true);
    test("~foo", true);
}

#[test]
fn test_logical_expressions() {
    test("a || b", false);
    test("a() || b", true);
    test("a && b", false);
    test("a() && b", true);
    test("a ?? b", false);
    test("a() ?? b", true);
}

#[test]
fn test_other_expressions() {
    test("(foo)", false);
    test("(foo())", true);

    test("a ? b : c", false);
    test("a() ? b : c", true);

    test("a, b", false);
    test("a(), b", true);
    test("a, b()", true);
}

#[test]
fn test_binary_expressions() {
    test("a === b", false);
    test("a() === b", true);
    test("a !== b", false);
    test("a() !== b", true);

    test("a == b", false);
    test("a() == b", true);
    // These actually have a side effect, but this treated as side-effect free.
    test("'' == { toString() { console.log('sideeffect') } }", false);
    test("'' == { valueOf() { console.log('sideeffect') } }", false);
    test("'' == { [s]() { console.log('sideeffect') } }", false); // assuming s is Symbol.toPrimitive
    test("a != b", false);
    test("a() != b", true);

    test("a < b", false);
    test("a() < b", true);
    // These actually have a side effect, but this treated as side-effect free.
    test("'' < { toString() { console.log('sideeffect') } }", false);
    test("'' < { valueOf() { console.log('sideeffect') } }", false);
    test("'' < { [s]() { console.log('sideeffect') } }", false); // assuming s is Symbol.toPrimitive
    test("a > b", false);
    test("a() > b", true);
    test("a >= b", false);
    test("a() >= b", true);
    test("a <= b", false);
    test("a() <= b", true);

    test("'' + ''", false);
    test("'' + ``", false);
    test("'' + `${foo()}`", true);
    test("'' + null", false);
    test("'' + 0", false);
    test("'' + 0n", false);
    test("'' + true", false);
    test("'' + /a/", false);
    test("'' + []", false);
    test("'' + [foo()]", true);
    test("'' + Symbol()", true);
    test("'' + Infinity", false);
    test("'' + NaN", false);
    test("'' + undefined", false);
    test("'' + s", true); // assuming s is Symbol
    test("Symbol() + ''", true);
    test("'' + {}", false);
    test("'' + { toString() { return Symbol() } }", true);
    test("'' + { valueOf() { return Symbol() } }", true);
    test("'' + { [s]() { return Symbol() } }", true); // assuming s is Symbol.toPrimitive
    test("/a/ + 1", false); // /a/1
    test("[] + 1", false); // 1
    test("({} + 1)", false); // [object Object]1
    test("0 + 1", false);
    test("0 + null", false); // 0
    test("0 + true", false); // 1
    test("0 + a", true); // a can be BigInt
    test("0n + 1n", false);
    test("0n + a", true); // a can be Number
    test("a + b", true);

    test("0n - 1n", false);
    test("0n - 0", true);
    test("0n - a", true); // a can be Number
    test("a - 0n", true); // a can be Number
    test("0n - a()", true);
    test("0 - 1", false);
    test("0 - a", true); // a can be BigInt
    test("0 - ''", false); // 0
    test("0 - ``", false); // 0
    test("0 - true", false); // -1
    test("0 - /a/", false); // NaN
    test("0 - []", false); // 0
    test("0 - [foo()]", true);
    test("0 - Infinity", false); // -Infinity
    test("0 - NaN", false); // NaN
    test("0 - undefined", false); // NaN
    test("null - Infinity", false); // -Infinity
    test("0 - {}", false); // NaN
    test("'' - { toString() { return Symbol() } }", true);
    test("'' - { valueOf() { return Symbol() } }", true);
    test("'' - { [s]() { return Symbol() } }", true); // assuming s is Symbol.toPrimitive
    test("a - b", true);
    test("0 * 1", false);
    test("0 * a", true);
    test("0 / 1", false);
    test("0 / a", true);
    test("0 % 1", false);
    test("0 % a", true);
    test("0 << 1", false);
    test("0 << a", true);
    test("0 | 1", false);
    test("0 | a", true);
    test("0 >> 1", false);
    test("0 >> a", true);
    test("0 ^ 1", false);
    test("0 ^ a", true);
    test("0 & 1", false);
    test("0 & a", true);
    test("0 ** 1", false);
    test("0 ** a", true);
    test("1n ** (-1n)", true); // `**` throws an error when the right operand is negative
    test("1n / 0n", true); // `/` throws an error when the right operand is zero
    test("1n % 0n", true); // `%` throws an error when the right operand is zero
    test("0n >>> 1n", true); // `>>>` throws an error even when both operands are bigint

    test("[] instanceof 1", true); // throws an error
    test("[] instanceof { [Symbol.hasInstance]() { throw 'foo' } }", true);
    test("[] instanceof Object", false);
    test("a instanceof Object", true); // a maybe a proxy that has a side effectful "getPrototypeOf" trap

    // b maybe not a object
    // b maybe a proxy that has a side effectful "has" trap
    test("a in b", true);
}

#[test]
fn test_object_expression() {
    // wrapped with parentheses to avoid treated as a block statement
    test("({})", false);
    test("({a: 1})", false);
    test("({a: foo()})", true);
    test("({1: 1})", false);
    test("({[1]: 1})", false);
    test("({[1n]: 1})", false);
    test("({['1']: 1})", false);
    // These actually have a side effect, but this treated as side-effect free.
    test("({[{ toString() { console.log('sideeffect') } }]: 1})", false);
    test("({[{ valueOf() { console.log('sideeffect') } }]: 1})", false);
    test("({[{ [s]() { console.log('sideeffect') } }]: 1})", false); // assuming s is Symbol.toPrimitive
    test("({[foo]: 1})", false);
    test("({[foo()]: 1 })", true);
    test("({a: foo()})", true);
    test("({...a})", true);
    test("({...[]})", false);
    test("({...[...a]})", true);
    test("({...'foo'})", false);
    test("({...`foo`})", false);
    test("({...`foo${1}`})", false);
    test("({...`foo${foo}`})", true);
    test("({...`foo${foo()}`})", true);
    test("({...foo()})", true);
}

#[test]
fn test_array_expression() {
    test("[]", false);
    test("[1]", false);
    test("[foo()]", true);
    test("[,]", false);
    test("[...a]", true);
    test("[...[]]", false);
    test("[...[...a]]", true);
    test("[...'foo']", false);
    test("[...`foo`]", false);
    test("[...`foo${1}`]", false);
    test("[...`foo${foo}`]", true);
    test("[...`foo${foo()}`]", true);
    test("[...foo()]", true);
    // test_in_function("[...arguments]", true);
    test_in_function("function foo() { [...arguments] }", false);
}

#[test]
fn test_class_expression() {
    test("(class {})", false);
    test("(@foo class {})", true);
    test("(class extends a {})", false); // this may have a side effect, but ignored by the assumption
    test("(class extends foo() {})", true);
    test("(class extends (() => {}) {})", true);
    test("(class { static {} })", false);
    test("(class { static { 1; } })", false);
    test("(class { static { foo(); } })", true);
    test("(class { a() {} })", false);
    test("(class { [1]() {} })", false);
    test("(class { [1n]() {} })", false);
    test("(class { #a() {} })", false);
    test("(class { [foo()]() {} })", true);
    test("(class { @foo a() {} })", true);
    test("(class { a; })", false);
    test("(class { 1; })", false);
    test("(class { [1]; })", false);
    test("(class { [1n]; })", false);
    test("(class { #a; })", false);
    test("(class { @foo a; })", true);
    test("(class { [foo()] = 1 })", true);
    test("(class { a = foo() })", false); // foo() is called when constructed
    test("(class { static a; })", false);
    test("(class { static 1; })", false);
    test("(class { static [1]; })", false);
    test("(class { static [1n]; })", false);
    test("(class { static #a; })", false);
    test("(class { static [foo()] = 1 })", true);
    test("(class { static a = foo() })", true);
    test("(class { accessor [foo()]; })", true);
    test("(class { static accessor [foo()]; })", true);
}

#[test]
fn test_property_access() {
    test("a.length", true);
    test("a?.length", true);
    test("'a'.length", false);
    test("'a'?.length", false);
    test("[].length", false);
    test("[]['length']", false);
    test("[][`length`]", false);
    test("(foo() + '').length", true);

    test("a[0]", true);
    test("''[-1]", true); // String.prototype[-1] may be overridden
    test("''[0.3]", true); // String.prototype[0.3] may be overridden
    test("''[0]", true); // String.prototype[0] may be overridden
    test("'a'[0]", false);
    test("'a'[0n]", false);
    test("'a'[1]", true); // String.prototype[1] may be overridden
    test("'あ'[0]", false);
    test("'あ'[1]", true); // the length of "あ" is 1
    test("'😀'[0]", false);
    test("'😀'[1]", false);
    test("'😀'[2]", true); // the length of "😀" is 2

    test("[][-1]", true); // Array.prototype[-1] may be overridden
    test("[][0.3]", true); // Array.prototype[0.3] may be overridden
    test("[][0]", true); // Array.prototype[0] may be overridden
    test("[1][0]", false);
    test("[1][0n]", false);
    test("[1][1]", true); // Array.prototype[1] may be overridden
    test("[,][0]", false);
    test("[...[], 1][0]", false);
    test("[...[1]][0]", false);
    test("[...'a'][0]", false);
    test("[...'a'][1]", true);
    test("[...'😀'][0]", false);
    test("[...'😀'][1]", true);
    test("[...a, 1][0]", true); // "...a" may have a sideeffect
}

// `[ValueProperties]: PURE` in <https://github.com/rollup/rollup/blob/master/src/ast/nodes/shared/knownGlobals.ts>
#[test]
fn test_new_expressions() {
    test("new AggregateError", true);
    test("new DataView", true);
    test("new Set", false);
    test("new Map", false);
    test("new WeakSet", false);
    test("new WeakMap", false);
    test("new ArrayBuffer", false);
    test("new Date", false);
    test("new Boolean", false);
    test("new Error", false);
    test("new EvalError", false);
    test("new RangeError", false);
    test("new ReferenceError", false);
    test("new RegExp", false);
    test("new SyntaxError", false);
    test("new TypeError", false);
    test("new URIError", false);
    test("new Number", false);
    test("new Object", false);
    test("new String", false);
    test("new Symbol", false);
}

// `PF` in <https://github.com/rollup/rollup/blob/master/src/ast/nodes/shared/knownGlobals.ts>
#[test]
fn test_call_expressions() {
    test("AggregateError()", true);
    test("DataView()", true);
    test("Set()", true);
    test("Map()", true);
    test("WeakSet()", true);
    test("WeakMap()", true);
    test("ArrayBuffer()", true);
    test("Date()", false);
    test("Boolean()", false);
    test("Error()", false);
    test("EvalError()", false);
    test("RangeError()", false);
    test("ReferenceError()", false);
    test("RegExp()", false);
    test("SyntaxError()", false);
    test("TypeError()", false);
    test("URIError()", false);
    test("Number()", false);
    test("Object()", false);
    test("String()", false);
    test("Symbol()", false);

    test("decodeURI()", false);
    test("decodeURIComponent()", false);
    test("encodeURI()", false);
    test("encodeURIComponent()", false);
    test("escape()", false);
    test("isFinite()", false);
    test("isNaN()", false);
    test("parseFloat()", false);
    test("parseInt()", false);

    test("Array.isArray()", false);
    test("Array.of()", false);

    test("ArrayBuffer.isView()", false);

    test("Date.now()", false);
    test("Date.parse()", false);
    test("Date.UTC()", false);

    test("Math.abs()", false);
    test("Math.acos()", false);
    test("Math.acosh()", false);
    test("Math.asin()", false);
    test("Math.asinh()", false);
    test("Math.atan()", false);
    test("Math.atan2()", false);
    test("Math.atanh()", false);
    test("Math.cbrt()", false);
    test("Math.ceil()", false);
    test("Math.clz32()", false);
    test("Math.cos()", false);
    test("Math.cosh()", false);
    test("Math.exp()", false);
    test("Math.expm1()", false);
    test("Math.floor()", false);
    test("Math.fround()", false);
    test("Math.hypot()", false);
    test("Math.imul()", false);
    test("Math.log()", false);
    test("Math.log10()", false);
    test("Math.log1p()", false);
    test("Math.log2()", false);
    test("Math.max()", false);
    test("Math.min()", false);
    test("Math.pow()", false);
    test("Math.random()", false);
    test("Math.round()", false);
    test("Math.sign()", false);
    test("Math.sin()", false);
    test("Math.sinh()", false);
    test("Math.sqrt()", false);
    test("Math.tan()", false);
    test("Math.tanh()", false);
    test("Math.trunc()", false);

    test("Number.isFinite()", false);
    test("Number.isInteger()", false);
    test("Number.isNaN()", false);
    test("Number.isSafeInteger()", false);
    test("Number.parseFloat()", false);
    test("Number.parseInt()", false);

    test("Object.create()", false);
    test("Object.getOwnPropertyDescriptor()", false);
    test("Object.getOwnPropertyDescriptors()", false);
    test("Object.getOwnPropertyNames()", false);
    test("Object.getOwnPropertySymbols()", false);
    test("Object.getPrototypeOf()", false);
    test("Object.hasOwn()", false);
    test("Object.is()", false);
    test("Object.isExtensible()", false);
    test("Object.isFrozen()", false);
    test("Object.isSealed()", false);
    test("Object.keys()", false);

    test("String.fromCharCode()", false);
    test("String.fromCodePoint()", false);
    test("String.raw()", false);

    test("Symbol.for()", false);
    test("Symbol.keyFor()", false);

    test("URL.canParse()", false);

    test("Float32Array.of()", false);
    test("Float64Array.of()", false);
    test("Int16Array.of()", false);
    test("Int32Array.of()", false);
    test("Int8Array.of()", false);
    test("Uint16Array.of()", false);
    test("Uint32Array.of()", false);
    test("Uint8Array.of()", false);
    test("Uint8ClampedArray.of()", false);

    // may have side effects if shadowed
    test_with_global_variables("Date()", &[], true);
    test_with_global_variables("Object.create()", &[], true);
}

#[test]
fn test_call_like_expressions() {
    test("foo()", true);
    test("/* #__PURE__ */ foo()", false);
    test("/* #__PURE__ */ foo(1)", false);
    test("/* #__PURE__ */ foo(bar())", true);
    test("/* #__PURE__ */ foo(...[])", false);
    test("/* #__PURE__ */ foo(...[1])", false);
    test("/* #__PURE__ */ foo(...[bar()])", true);
    test("/* #__PURE__ */ foo(...bar)", true);
    test("/* #__PURE__ */ foo(...`foo`)", false);
    test("/* #__PURE__ */ foo(...`${1}`)", false);
    test("/* #__PURE__ */ foo(...`${bar}`)", true);
    test("/* #__PURE__ */ foo(...`${bar()}`)", true);
    test("/* #__PURE__ */ (() => { foo() })()", false);
    test("foo?.()", true);
    test("/* #__PURE__ */ foo?.()", false);

    test("new Foo()", true);
    test("/* #__PURE__ */ new Foo()", false);
    test("/* #__PURE__ */ new Foo(1)", false);
    test("/* #__PURE__ */ new Foo(bar())", true);
    test("/* #__PURE__ */ new Foo(...[])", false);
    test("/* #__PURE__ */ new Foo(...[1])", false);
    test("/* #__PURE__ */ new Foo(...[bar()])", true);
    test("/* #__PURE__ */ new Foo(...bar)", true);
    test("/* #__PURE__ */ new Foo(...`foo`)", false);
    test("/* #__PURE__ */ new Foo(...`${1}`)", false);
    test("/* #__PURE__ */ new Foo(...`${bar}`)", true);
    test("/* #__PURE__ */ new Foo(...`${bar()}`)", true);
    test("/* #__PURE__ */ new class { constructor() { foo() } }()", false);

    let ctx = Ctx { annotation: false, ..Default::default() };
    test_with_ctx("/* #__PURE__ */ foo()", &ctx, true);
    test_with_ctx("/* #__PURE__ */ new Foo()", &ctx, true);
}

#[test]
fn test_is_pure_call_support() {
    let ctx = Ctx {
        pure_function_names: vec!["foo".to_string(), "Foo".to_string()],
        ..Default::default()
    };
    test_with_ctx("foo()", &ctx, false);
    test_with_ctx("foo(1)", &ctx, false);
    test_with_ctx("foo(bar())", &ctx, true);
    test_with_ctx("bar()", &ctx, true);
    test_with_ctx("new Foo()", &ctx, false);
    test_with_ctx("new Foo(1)", &ctx, false);
    test_with_ctx("new Foo(bar())", &ctx, true);
    test_with_ctx("new Bar()", &ctx, true);
    test_with_ctx("foo``", &ctx, false);
    test_with_ctx("foo`1`", &ctx, false);
    test_with_ctx("foo`${bar()}`", &ctx, true);
    test_with_ctx("bar``", &ctx, true);
}

#[test]
fn test_manual_pure_functions_with_dotted_names() {
    let ctx = Ctx { pure_function_names: vec!["console".to_string()], ..Default::default() };
    test_with_ctx("console()", &ctx, false);
    test_with_ctx("console.log()", &ctx, false);
    test_with_ctx("console.log(bar())", &ctx, true);
    test_with_ctx("other.log()", &ctx, true);
    let ctx = Ctx { pure_function_names: vec!["console.log".to_string()], ..Default::default() };
    test_with_ctx("console.log()", &ctx, false);
    test_with_ctx("console.warn()", &ctx, true);
    test_with_ctx("console.log.foo()", &ctx, false);
}

#[test]
fn test_property_read_side_effects_support() {
    let all_ctx =
        Ctx { property_read_side_effects: PropertyReadSideEffects::All, ..Default::default() };
    let none_ctx =
        Ctx { property_read_side_effects: PropertyReadSideEffects::None, ..Default::default() };

    test_with_ctx("foo.bar", &all_ctx, true);
    test_with_ctx("foo.bar", &none_ctx, false);
    test_with_ctx("foo[0]", &none_ctx, false);
    test_with_ctx("foo[0n]", &none_ctx, false);
    test_with_ctx("foo[bar()]", &none_ctx, true);
    test_with_ctx("foo.#bar", &all_ctx, true);
    test_with_ctx("foo.#bar", &none_ctx, false);
    test_with_ctx("({ bar } = foo)", &all_ctx, true);
    // test_with_ctx("({ bar } = foo)", &none_ctx, false);
}

#[test]
fn test_unknown_global_side_effects_support() {
    let true_ctx = Ctx {
        unknown_global_side_effects: true,
        global_variable_names: FxHashSet::from_iter(["foo"]),
        ..Default::default()
    };
    let false_ctx = Ctx {
        unknown_global_side_effects: false,
        global_variable_names: FxHashSet::from_iter(["foo"]),
        ..Default::default()
    };
    test_with_ctx("foo", &true_ctx, true);
    test_with_ctx("foo", &false_ctx, false);
}

#[test]
fn test_object_with_to_primitive_related_properties_overridden() {
    test("+{}", false);
    test("+{ foo: 0 }", false);
    test("+{ toString() { return Symbol() } }", true);
    test("+{ valueOf() { return Symbol() } }", true);
    test("+{ 'toString'() { return Symbol() } }", true);
    test("+{ 'valueOf'() { return Symbol() } }", true);
    test("+{ ['toString']() { return Symbol() } }", true);
    test("+{ ['valueOf']() { return Symbol() } }", true);
    test("+{ [`toString`]() { return Symbol() } }", true);
    test("+{ [`valueOf`]() { return Symbol() } }", true);
    test("+{ [Symbol.toPrimitive]() { return Symbol() } }", true);
    test("+{ ...foo }", true); // foo can include toString / valueOf / Symbol.toPrimitive
    test("+{ ...[] }", false);
    test("+{ ...'foo' }", false);
    test("+{ ...`foo` }", false);
    test("+{ ...`foo${1}` }", false);
    test("+{ ...`foo${foo}` }", true);
    test("+{ ...`foo${foo()}` }", true);
    test("+{ ...{ toString() { return Symbol() } } }", true);
    test("+{ ...{ valueOf() { return Symbol() } } }", true);
    test("+{ ...{ [Symbol.toPrimitive]() { return Symbol() } } }", true);
}

#[test]
fn test_typeof_guard_patterns() {
    test_with_global_variables("typeof x !== 'undefined' && x", &["x"], false);
    test_with_global_variables("typeof x != 'undefined' && x", &["x"], false);
    test_with_global_variables("'undefined' !== typeof x && x", &["x"], false);
    test_with_global_variables("'undefined' != typeof x && x", &["x"], false);
    test_with_global_variables("typeof x === 'undefined' || x", &["x"], false);
    test_with_global_variables("typeof x == 'undefined' || x", &["x"], false);
    test_with_global_variables("'undefined' === typeof x || x", &["x"], false);
    test_with_global_variables("'undefined' == typeof x || x", &["x"], false);
    test_with_global_variables("typeof x < 'u' && x", &["x"], false);
    test_with_global_variables("typeof x <= 'u' && x", &["x"], false);
    test_with_global_variables("'u' > typeof x && x", &["x"], false);
    test_with_global_variables("'u' >= typeof x && x", &["x"], false);
    test_with_global_variables("typeof x > 'u' || x", &["x"], false);
    test_with_global_variables("typeof x >= 'u' || x", &["x"], false);
    test_with_global_variables("'u' < typeof x || x", &["x"], false);
    test_with_global_variables("'u' <= typeof x || x", &["x"], false);

    test_with_global_variables("typeof x === 'undefined' ? 0 : x", &["x"], false);
    test_with_global_variables("typeof x == 'undefined' ? 0 : x", &["x"], false);
    test_with_global_variables("'undefined' === typeof x ? 0 : x", &["x"], false);
    test_with_global_variables("'undefined' == typeof x ? 0 : x", &["x"], false);
    test_with_global_variables("typeof x !== 'undefined' ? x : 0", &["x"], false);
    test_with_global_variables("typeof x != 'undefined' ? x : 0", &["x"], false);
    test_with_global_variables("'undefined' !== typeof x ? x : 0", &["x"], false);
    test_with_global_variables("'undefined' != typeof x ? x : 0", &["x"], false);

    test_with_global_variables("typeof x !== 'undefined' && (x + foo())", &["x"], true);
    test_with_global_variables("typeof x === 'undefined' || (x + foo())", &["x"], true);
    test_with_global_variables("typeof x === 'undefined' ? foo() : x", &["x"], true);
    test_with_global_variables("typeof x !== 'undefined' ? x : foo()", &["x"], true);
    test_with_global_variables("typeof foo() !== 'undefined' && x", &["x"], true);
    test_with_global_variables("typeof foo() === 'undefined' || x", &["x"], true);
    test_with_global_variables("typeof foo() === 'undefined' ? 0 : x", &["x"], true);
    test_with_global_variables("typeof y !== 'undefined' && x", &["x", "y"], true);
    test_with_global_variables("typeof y === 'undefined' || x", &["x", "y"], true);
    test_with_global_variables("typeof y === 'undefined' ? 0 : x", &["x", "y"], true);

    test("typeof localVar !== 'undefined' && localVar", false);
    test("typeof localVar === 'undefined' || localVar", false);
    test("typeof localVar === 'undefined' ? 0 : localVar", false);

    test_with_global_variables(
        "typeof x !== 'undefined' && typeof y !== 'undefined' && x && y",
        &["x", "y"],
        true, // This can be improved
    );
}

#[test]
fn test_assignment_targets() {
    test_assign_target("a = 1", false);
    test_assign_target("String = 1", false);
    test_assign_target("({ a } = 1)", true); // this can be improved
    test_assign_target("([a] = 1)", true); // this can be improved
    test_assign_target("a.b = 1", false); // the side effect of the setter of `a.b` happens in `PutValue`
    test_assign_target_with_global_variables("a.b = 1", &["a"], true); // `a` might not be declared and cause ReferenceError in strict mode
    test_assign_target("(foo(), a).b = 1", true); // `foo()` may have sideeffect
    test_assign_target("a['b'] = 1", false);
    test_assign_target("a[foo()] = 1", true); // `foo()` may have sideeffect
    test_assign_target_with_global_variables("a['b'] = 1", &["a"], true); // `a` might not be declared and cause ReferenceError in strict mode
    test_assign_target("a.#b = 1", false);
    test_assign_target_with_global_variables("a.#b = 1", &["a"], true); // `a` might not be declared and cause ReferenceError in strict mode
    test_assign_target("(foo(), a).#b = 1", true); // `foo()` may have sideeffect
}
