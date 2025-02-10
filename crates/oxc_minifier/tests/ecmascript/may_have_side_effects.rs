use oxc_allocator::Allocator;
use oxc_ast::ast::{IdentifierReference, Statement};
use oxc_ecmascript::side_effects::MayHaveSideEffects;
use oxc_parser::Parser;
use oxc_span::SourceType;

struct SideEffectChecker {
    global_variable_names: Vec<String>,
}
impl MayHaveSideEffects for SideEffectChecker {
    fn is_global_reference(&self, ident: &IdentifierReference<'_>) -> bool {
        self.global_variable_names.iter().any(|name| name == ident.name.as_str())
    }
}

fn test(source_text: &str, expected: bool) {
    test_with_global_variables(source_text, vec![], expected);
}

fn test_with_global_variables(
    source_text: &str,
    global_variable_names: Vec<String>,
    expected: bool,
) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, SourceType::mjs()).parse();
    assert!(!ret.panicked, "{source_text}");
    assert!(ret.errors.is_empty(), "{source_text}");

    let side_effect_checker = SideEffectChecker { global_variable_names };

    let Some(Statement::ExpressionStatement(stmt)) = &ret.program.body.first() else {
        panic!("should have a expression statement body: {source_text}");
    };
    assert_eq!(
        side_effect_checker.expression_may_have_side_effects(&stmt.expression),
        expected,
        "{source_text}"
    );
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
    test("(class { [computedName()]() {} })", false); // computedName is called when constructed
    test("(class { [computedName]() {} })", false);
    test("(class Foo extends Bar { })", false);
    test("(class extends foo() { })", false); // foo() is called when constructed
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
    test("`template${name}`", false);
    test("`${name}template`", false);
    test("`${naming()}template`", true);
    test("templateFunction`template`", true);
    test("st = `${name}template`", true);
    test("tempFunc = templateFunction`template`", true);
    // test("new RegExp('foobar', 'i')", false);
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
    test_with_global_variables("-Infinity", vec!["Infinity".to_string()], false);
    test_with_global_variables("Infinity", vec!["Infinity".to_string()], false);
    test_with_global_variables("NaN", vec!["NaN".to_string()], false);
    // test("({}||[]).foo = 2;", false);
    // test("(true ? {} : []).foo = 2;", false);
    // test("({},[]).foo = 2;", false);
    test("delete a.b", true);
    // test("Math.random();", false);
    test("Math.random(seed);", true);
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
    test("[...`templatelit ${safe}`]", false);
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
    // test("Math.sin(...`templatelit ${safe}`)", false);
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
    // test("new Object(...`templatelit ${safe}`)", false);
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
    test("(class C { [a()]() {} })", false); // a is called when constructed

    // computed property getters and setters are modeled as COMPUTED_PROP with an
    // annotation to indicate getter or setter.
    test("(class C { get [a]() {} })", false);
    test("(class C { get [a()]() {} })", false); // a is called when constructed
    test("(class C { set [a](x) {} })", false);
    test("(class C { set [a()](x) {} })", false); // a is called when constructed

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
    // test("(class C { static { [1]; } })", false);
    test("(class C { static { let x; } })", true);
    test("(class C { static { const x =1 ; } })", true);
    test("(class C { static { var x; } })", true);
    test("(class C { static { this.x = 1; } })", true);
    test("(class C { static { function f() { } } })", true);
    // test("(class C { static { (function () {} )} })", false);
    // test("(class C { static { ()=>{} } })", false);

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
    test_with_global_variables("a", vec!["a".to_string()], true);
    // accessing known globals are side-effect free
    test_with_global_variables("NaN", vec!["NaN".to_string()], false);
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
fn test_unary_expressions() {
    test("delete 'foo'", true);
    test("delete foo()", true);

    test("void 'foo'", false);
    test("void foo()", true);
    test("!'foo'", false);
    test("!foo()", true);

    test("typeof 'foo'", false);
    test_with_global_variables("typeof a", vec!["a".to_string()], false);
    test("typeof foo()", true);

    test("+0", false);
    test("+0n", true);
    test("+null", false); // 0
    test("+true", false); // 1
    test("+'foo'", false); // NaN
    test("+`foo`", false); // NaN
    test("+/foo/", false); // NaN
    test_with_global_variables("+Infinity", vec!["Infinity".to_string()], false);
    test_with_global_variables("+NaN", vec!["NaN".to_string()], false);
    test_with_global_variables("+undefined", vec!["undefined".to_string()], false); // NaN
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
    test_with_global_variables("-Infinity", vec!["Infinity".to_string()], false);
    test_with_global_variables("-NaN", vec!["NaN".to_string()], false);
    test_with_global_variables("-undefined", vec!["undefined".to_string()], false); // NaN
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
    test_with_global_variables("'' + Infinity", vec!["Infinity".to_string()], false);
    test_with_global_variables("'' + NaN", vec!["NaN".to_string()], false);
    test_with_global_variables("'' + undefined", vec!["undefined".to_string()], false);
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
    test_with_global_variables("0 - Infinity", vec!["Infinity".to_string()], false); // -Infinity
    test_with_global_variables("0 - NaN", vec!["NaN".to_string()], false); // NaN
    test_with_global_variables("0 - undefined", vec!["undefined".to_string()], false); // NaN
    test_with_global_variables("null - Infinity", vec!["Infinity".to_string()], false); // -Infinity
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

    // b maybe not a object
    // b maybe a proxy that has a side effectful "has" trap
    test("a in b", true);
    // b maybe not a function
    // b[Symbol.hasInstance] may have a side effect
    // a maybe a proxy that has a side effectful "getPrototypeOf" trap
    test("a instanceof b", true);
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
    test("({[foo()]: 1 })", true);
    test("({a: foo()})", true);
    test("({...a})", true);
    test("({...[]})", false);
    test("({...[...a]})", true);
    test("({...'foo'})", false);
    test("({...`foo`})", false);
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
    test("[...`foo${foo()}`]", true);
    test("[...foo()]", true);
}

#[test]
fn test_class_expression() {
    test("(class {})", false);
    test("(class extends a {})", false);
    test("(class extends foo() {})", false); // foo() is called when constructed
    test("(class { static {} })", false);
    test("(class { static { foo(); } })", true);
    test("(class { a; })", false);
    test("(class { 1; })", false);
    test("(class { [1]; })", false);
    test("(class { [1n]; })", false);
    test("(class { #a; })", false);
    test("(class { [foo()] = 1 })", false); // foo() is called when constructed
    test("(class { a = foo() })", false); // foo() is called when constructed
    test("(class { static a; })", false);
    test("(class { static 1; })", false);
    test("(class { static [1]; })", false);
    test("(class { static [1n]; })", false);
    test("(class { static #a; })", false);
    test("(class { static [foo()] = 1 })", true);
    test("(class { static a = foo() })", true);
    test("(class { accessor [foo()]; })", false);
    test("(class { static accessor [foo()]; })", true);
}

#[test]
fn test_side_effectful_expressions() {
    test("a.b", true);
    test("a[0]", true);
    test("a?.b", true);
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
    test("+{ ...`foo${foo}` }", false);
    test("+{ ...{ toString() { return Symbol() } } }", true);
    test("+{ ...{ valueOf() { return Symbol() } } }", true);
    test("+{ ...{ [Symbol.toPrimitive]() { return Symbol() } } }", true);
}
