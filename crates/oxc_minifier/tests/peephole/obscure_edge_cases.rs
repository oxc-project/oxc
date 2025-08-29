use crate::{test, test_same};

/// Tests for edge cases that should reveal minification problems
/// Focus on cases where optimizations might be unsafe or incorrect

#[test]
fn test_dead_code_elimination_edge_cases() {
    // Test DCE with known constant conditions
    test("if (true) { foo(); } else { bar(); }", "foo();");
    test("if (false) { foo(); } else { bar(); }", "bar();");
    test("true ? foo() : bar()", "foo()");
    test("false ? foo() : bar()", "bar()");

    // Test logical operator short-circuiting
    test("true && foo()", "foo()");
    test("false && foo()", "");
    test("true || foo()", ""); // foo() eliminated since true || anything is always true, result gets eliminated as unused expression
    test("false || foo()", "foo()");

    // Test with complex expressions that have side effects
    test("sideEffect() && false", "sideEffect()"); // side effect preserved, false eliminated
    test("true || sideEffect()", ""); // sideEffect eliminated since true || anything is always true, result gets eliminated

    // Test numeric comparisons that are always true/false
    test("if (5 > 3) { definite(); }", "definite();");
    test("if (2 < 1) { never(); }", "");
    test("if (0 === 0) { always(); }", "always();");
    test("if (1 !== 1) { never(); }", "");
}

#[test]
fn test_string_concatenation_edge_cases() {
    // Test cases where string concatenation is optimized
    test("return 'hello ' + 'world'", "return 'hello world'"); // string literal concatenation
    test("return 'count: ' + 42", "return 'count: 42'"); // string + number concatenation
    test("return 42 + ' items'", "return '42 items'"); // number + string concatenation

    // Test with complex expressions - should not optimize if there are side effects
    test_same("return getValue() + 'suffix'");
    test_same("return 'prefix' + sideEffect()");

    // Test property access optimization (this is actually supported)
    test("return obj['property']", "return obj.property");
    test("return obj['validName123']", "return obj.validName123");
    test("return obj['$special']", "return obj.$special");

    // Test cases that should NOT be optimized
    test_same("return obj['123invalid']"); // starts with number
    test_same("return obj['key-with-dash']"); // contains dash
    test_same("return obj['key with space']"); // contains space
    test_same("return obj[dynamicKey]"); // dynamic key
    test_same("return obj['']"); // empty string
    test("return obj['class']", "return obj.class"); // reserved word but valid in property context
    test("return obj['function']", "return obj.function"); // reserved word but valid in property context

    // Dynamic concatenation is optimized when both parts are static
    test("return obj['prop' + 'name']", "return obj.propname");
}

#[test]
fn test_typeof_optimization_edge_cases() {
    // Test typeof with literals get optimized to string literals
    test("return typeof 42", "return 'number'"); // optimized to literal string
    test("return typeof 'string'", "return 'string'"); // optimized to literal string
    test("return typeof true", "return 'boolean'"); // optimized to literal string
    test("return typeof undefined", "return 'undefined'"); // optimized to literal string
    test("return typeof null", "return 'object'"); // optimized to literal string
    test("return typeof []", "return 'object'"); // optimized to literal string
    test("return typeof {}", "return 'object'"); // optimized to literal string
    test("return typeof function(){}", "return 'function'"); // optimized to literal string

    // Test typeof in conditionals that ARE optimized
    test("if (typeof 5 === 'number') { always(); }", "always();"); // optimized to always()
    test("if (typeof 'test' !== 'string') { never(); }", ""); // optimized to empty (never executes)

    // Test typeof with variables - these become dead code when unused
    test("typeof x === 'undefined'", ""); // eliminated as unused expression
    test("typeof unknownVar", ""); // eliminated as unused expression
}

#[test]
fn test_numeric_comparison_edge_cases() {
    // Test numeric comparisons that get optimized to constants
    test("return 5 > 3", "return !0"); // optimized to true constant
    test("return 10 <= 5", "return !1"); // optimized to false constant
    test("return 7 === 7", "return !0"); // optimized to true constant
    test("return 3 !== 5", "return !0"); // optimized to true constant

    // Test string comparisons (in return context)
    test("return 'a' < 'b'", "return !0"); // optimized to true constant
    test("return 'hello' === 'hello'", "return !0"); // optimized to true constant
    test("return 'abc' !== 'def'", "return !0"); // optimized to true constant

    // Test special value comparisons get optimized
    test("return null == undefined", "return !0"); // optimized to true constant
    test("return null === undefined", "return !1"); // optimized to false constant
    test("return null == null", "return !0"); // optimized to true constant
    test("return undefined === undefined", "return !0"); // optimized to true constant

    // Test type coercion comparisons get optimized
    test("return 0 == false", "return !0"); // optimized to true constant
    test("return '' == false", "return !0"); // optimized to true constant
    test("return '0' == false", "return !0"); // optimized to true constant
    test("return 0 === false", "return !1"); // optimized to false constant

    // Test NaN comparisons get optimized
    test("return NaN === NaN", "return !1"); // optimized to false constant
    test("return NaN == NaN", "return !1"); // optimized to false constant
    test("return NaN !== NaN", "return !0"); // optimized to true constant
    test("return NaN != NaN", "return !0"); // optimized to true constant
}

#[test]
fn test_mathematical_expression_edge_cases() {
    // Test operations with special numeric values get optimized
    test("return 1 / 0", "return Infinity"); // optimized to Infinity
    test("return -1 / 0", "return -Infinity"); // optimized to -Infinity
    test("return 0 / 0", "return NaN"); // optimized to NaN

    // Test simple arithmetic - these ARE optimized by oxc
    test("return 2 + 3", "return 5");
    test("return 10 - 4", "return 6");
    test("return 3 * 7", "return 21");
    test_same("return 15 / 3"); // division might not be optimized consistently

    // Test cases that are eliminated as dead code (unused expressions)
    test("NaN + 1", ""); // eliminated as unused expression
    test("NaN * 0", ""); // eliminated as unused expression
    test("NaN / NaN", ""); // eliminated as unused expression
    test("Infinity + 1", ""); // eliminated as unused expression
    test("Infinity - Infinity", ""); // eliminated as unused expression
    test("Infinity / Infinity", ""); // eliminated as unused expression
    test_same("Math.PI * 2"); // runtime value
    test_same("Math.E + 1"); // runtime value
    test("-0 + 0", ""); // eliminated as unused expression
    test("-0 * 1", ""); // eliminated as unused expression
    test("1 / -0", ""); // eliminated as unused expression
}

#[test]
fn test_function_call_optimization_edge_cases() {
    // Test constructor calls that get optimized
    test("return String(42)", "return '42'"); // string constructor optimization
    test("return Number('123')", "return 123"); // number constructor optimization
    test("return Boolean(1)", "return !0"); // boolean constructor optimization
    test("return Boolean(0)", "return !1"); // boolean constructor optimization

    // Test cases that should NOT be optimized due to side effects
    test_same("console.log('test')");
    test_same("Object.keys(obj)"); // depends on obj

    test("Math.random()", "");
    test("Date.now()", "");
    test("Object(null)", "");

    // Test method calls on literals that get optimized
    test("return 'hello'.length", "return 5"); // string length optimization
    test("return ''.length", "return 0"); // empty string length
    test("return [1, 2, 3].length", "return 3"); // array length optimization
    test("return [].length", "return 0"); // empty array length
}

#[test]
fn test_object_literal_edge_cases() {
    // Test property name optimization that might be supported in future
    test("const obj = { 'key': 1 }", "const obj = {key: 1};"); // could be optimized to { key: value }
    test("const obj = { 'validName': 1 }", "const obj = {validName: 1};"); // could be optimized to { validName: value }

    // Test cases that should NOT be optimized
    test_same("const obj = { '123invalid': 1 }"); // starts with number
    test_same("const obj = { 'key-with-dash': 1 }"); // contains dash
    test_same("const obj = { 'key with space': 1 }"); // contains space
    test_same("const obj = { '': 1 }"); // empty string
    test("const obj = { 'class': 1 }", "const obj = { class: 1 };"); // reserved word (context dependent)

    // Test computed property names
    test("const obj = { ['key']: 1 }", "const obj = {key: 1};"); // could be optimized to { key: value }
    test_same("const obj = { [dynamicKey]: 1 }"); // should not optimize
    test("const obj = { ['prop' + 'name']: 1 }", "const obj = {propname: 1};"); // might be optimized to { propname: value }
}

#[test]
fn test_regex_literal_edge_cases() {
    // Test regex operations - these are complex and usually should not be optimized
    // for safety and correctness
    test_same("/abc/.test('abc')"); // could potentially be optimized to true
    test_same("/abc/.test('def')"); // could potentially be optimized to false
    test_same("/\\d+/.test('123')"); // could potentially be optimized to true
    test_same("/\\d+/.test('abc')"); // could potentially be optimized to false

    // Test cases that should NOT be optimized
    test_same("/complex(?:pattern)+/.test(input)"); // too complex
    test_same("new RegExp(pattern).test(input)"); // dynamic pattern
    test_same("regex.test(input)"); // dynamic regex

    // Test regex flags
    test_same("/abc/i.test('ABC')"); // could potentially be optimized to true
    test_same("/abc/i.test('def')"); // could potentially be optimized to false
}

#[test]
fn test_array_method_edge_cases() {
    // Test array methods on literals - these are NOT currently optimized by oxc
    test_same("return [1, 2, 3].indexOf(2)");
    test_same("return [1, 2, 3].indexOf(5)");
    test_same("return ['a', 'b', 'c'].includes('b')");
    test_same("return ['a', 'b', 'c'].includes('d')");
    test_same("return [1, 2, 3].slice(1)");
    test("return [1, 2].concat([3, 4])", "return [\n\t1,\n\t2,\n\t3,\n\t4\n];");

    // Test methods that should NOT be optimized due to side effects
    test_same("[1, 2, 3].forEach(fn)");
    test_same("arr.push(item)");
    test_same("arr.pop()");
    test_same("arr.map(fn)");
}

#[test]
fn test_assignment_optimization_edge_cases() {
    // Test assignment expressions that can be optimized
    test("x = x + 1", "x += 1"); // compound assignment optimization
    test("x = x - 1", "--x"); // optimized to prefix decrement
    test("x = x * 2", "x *= 2"); // compound assignment optimization
    test("x = x / 2", "x /= 2"); // compound assignment optimization

    // Test cases that demonstrate current conservative behavior with property/array access
    test_same("obj.prop = obj.prop + 1"); // conservative with property access (getters/setters)
    test_same("arr[i] = arr[i] + 1"); // conservative with array access
    test("this.prop = this.prop + 1", "this.prop += 1"); // this context gets optimized
}

#[test]
fn test_side_effect_analysis_edge_cases() {
    // Test expressions that are pure but not eliminated in all contexts
    test("42;", ""); // literal expressions get eliminated
    test("'hello';", "'hello';"); // string literal statement not eliminated
    test("true;", ""); // boolean literals get eliminated
    test("1 + 2 + 3;", ""); // pure arithmetic gets eliminated

    // Test expressions that have side effects and should be preserved
    test_same("console.log('side effect');");
    test_same("obj.method();");
    test_same("globalVar = 5;");
    test_same("delete obj.prop;");
    test_same("++counter;");

    // Test mixed pure and impure - pure expressions get eliminated
    test("1 + 2; console.log('keep'); 3 + 4;", "console.log('keep');");

    // Test property access that might have getters - these are NOT eliminated
    test_same("obj.prop;");
    test("obj['computed'];", "obj.computed;");
}

#[test]
fn test_variable_elimination_edge_cases() {
    // Test unused variable elimination with boolean optimization
    test_same("var unused = 5;"); // could be eliminated if truly unused
    test_same("let unused = 'hello';"); // could be eliminated if truly unused
    test("const unused = true", "const unused = !0"); // boolean gets optimized

    // Test used variables that should not be eliminated - but inlining happens
    test("var used = 5; console.log(used);", "console.log(5);");
    test("let used = 'hello'; return used;", "let used = 'hello';\nreturn 'hello';"); // variable gets inlined
    test("const used = true; if (used) foo();", "const used = !0;\nfoo();"); // constant inlined and optimized

    // Test variable inlining
    test("const y = 'hello'; return y;", "const y = 'hello';\nreturn 'hello';"); // check if const also gets inlined

    // Test cases where inlining should NOT happen
    test_same("var x = sideEffect(); console.log(x);"); // side effect
    test("var x = 5; x = 10; console.log(x);", "var x = 5;\nx = 10, console.log(x);"); // reassigned
    test(
        "let x = 5; if (condition) x = 10; console.log(x);",
        "let x = 5;\ncondition && (x = 10), console.log(x);",
    ); // conditionally reassigned
}

#[test]
fn test_loop_optimization_edge_cases() {
    // Test loops with constant conditions that can be eliminated
    test("while (false) { neverExecuted(); }", ""); // eliminated since condition is false
    test("for (; false;) { neverExecuted(); }", ""); // eliminated since condition is false
    // Test while loops get optimized
    test("while (true) { infiniteLoop(); }", "for (;;) infiniteLoop();"); // optimized loop form

    // Test do-while loops - false becomes !1, braces may be removed, true becomes !0
    test("do { executedOnce(); } while (false);", "do\n\texecutedOnce();\nwhile (!1);");
    test("do { body(); } while (true);", "do\n\tbody();\nwhile (!0);");

    // Test for loops with analyzable bounds
    test(
        "for (var i = 0; i < 0; i++) { neverExecuted(); }",
        "for (var i = 0; i < 0; i++) neverExecuted();",
    ); // could be eliminated
    test(
        "for (var i = 5; i < 3; i++) { neverExecuted(); }",
        "for (var i = 5; i < 3; i++) neverExecuted();",
    ); // could be eliminated
    test("for (var i = 0; i < 3; i++) { executed(); }", "for (var i = 0; i < 3; i++) executed();"); // should preserve
}

#[test]
fn test_switch_statement_edge_cases() {
    // Test switch with constant discriminant - might be optimized in future
    test_same("switch (2) { case 1: a(); break; case 2: b(); break; case 3: c(); break; }");
    // Could be optimized to just: b();

    test_same("switch ('test') { case 'foo': a(); break; case 'test': b(); break; default: c(); }");
    // Could be optimized to just: b();

    // Test switch with no matching case
    test_same("switch (5) { case 1: a(); break; case 2: b(); break; }");
    // Could be optimized to empty

    // Test switch with default
    test_same("switch (5) { case 1: a(); break; default: b(); break; }");
    // Could be optimized to just: b();

    // Test switch with fall-through - more complex, keep as same for safety
    test_same("switch (1) { case 1: a(); case 2: b(); break; case 3: c(); }");
    // Should preserve fall-through behavior
}

#[test]
fn test_try_catch_optimization_edge_cases() {
    // Test try-catch where code cannot throw
    test_same("try { safeCode(); } catch (e) { handleError(e); }");
    // Could be optimized to just: safeCode();

    // Test try-finally
    test_same("try { code(); } finally { cleanup(); }");
    // Should preserve both parts

    // Test try-catch-finally
    test_same("try { code(); } catch (e) { handle(e); } finally { cleanup(); }");
    // Complex case - should be careful about optimization

    // Test cases that should NOT be optimized
    test_same("try { riskyCode(); } catch (e) { handleError(e); }");
    test_same("try { eval(code); } catch (e) { handleError(e); }");
}

#[test]
fn test_minifier_safety_boundaries() {
    // Test with statement minification (braces removal)
    test("with (obj) { prop = value; }", "with(obj) prop = value;"); // braces removed for single statement
    // Test sequence expressions that get optimized
    test("(1, eval)('code')", "(0, eval)('code')"); // sequence gets optimized to preserve indirect eval
    test_same("arguments[0] = 'modified';"); // arguments object

    // Test getter/setter preservation
    test_same("obj = { get prop() { return this._prop; } }");
    test_same("obj = { set prop(v) { this._prop = v; } }");

    // Test proxy/reflect that can't be statically analyzed
    test_same("new Proxy(obj, handler)");
    test_same("Reflect.get(obj, prop)");

    // Test dynamic property access
    test_same("obj[computedKey]");
    test_same("obj[key()]");
}

#[test]
fn test_esm_minification_edge_cases() {
    // Static import statements (should be preserved as-is)
    test_same("import { a, b } from 'module';");
    test_same("import { a as x, b as y } from 'module';");
    test_same("import * as ns from 'module';");
    test_same("import defaultExport from 'module';");
    test_same("import defaultExport, { a, b } from 'module';");
    test_same("import defaultExport, * as ns from 'module';");
    test_same("import 'side-effect-module';");

    // Static export statements (should be preserved as-is)
    test_same("export { a, b };");
    test_same("export { a as x, b as y };");
    test_same("export * from 'module';");
    test_same("export * as ns from 'module';");
    test_same("export { a, b } from 'module';");
    test_same("export { a as x, b as y } from 'module';");
    test_same("export default value;");
    test_same("export default function() {}");
    test_same("export default class {}");

    // Export declarations with optimizations
    test("export var a = 1 + 2;", "export var a = 3;");
    test("export let b = 'hello' + ' world';", "export let b = 'hello world';");
    test("export const c = true ? 'yes' : 'no';", "export const c = 'yes';");
    test("export function f() { return 2 + 3; }", "export function f() { return 5; }");
    test(
        "export class C { method() { return 'a' + 'b'; } }",
        "export class C { method() { return 'ab'; } }",
    );

    // Dynamic imports (expression optimization)
    test_same("import('./module.js')");
    test_same("import(`./modules/${name}.js`)");
    test("import('prefix' + 'suffix')", "import('prefixsuffix')");
    test("import(`module-${1 + 2}`)", "import('module-3')");

    // Dynamic imports with options
    test_same("import('./module.js', { assert: { type: 'json' } })");
    test_same("import('./module.js', { with: { type: 'json' } })");
    test(
        "import('./module.js', { assert: { type: 'js' + 'on' } })",
        "import('./module.js', { assert: { type: 'json' } })",
    );

    // Import.meta (should be preserved mostly)
    test_same("import.meta.url");
    test_same("import.meta.resolve('./module.js')");
    test_same("import.meta.hot");
    test_same("import.meta.env");
    test("import.meta.resolve('prefix' + 'suffix')", "import.meta.resolve('prefixsuffix')");

    // Top-level await (should be preserved)
    test_same("await import('./dynamic.js')");
    test_same("const data = await import('./data.json', { assert: { type: 'json' } });");
    test("const result = await (1 + 2);", "const result = await 3;");

    // Complex ESM patterns
    test_same("export { default as myDefault } from './other.js';");
    test_same("import('./polyfill.js').then(() => import('./main.js'));");
    test("export const computed = obj['prop' + 'erty'];", "export const computed = obj.property;");

    // Import/export with computed property names in object patterns
    test(
        "const { ['computed' + 'Name']: value } = module;",
        "const { computedName: value } = module;",
    );
    test("import('./modules/' + (true ? 'prod' : 'dev') + '.js')", "import('./modules/prod.js')");

    // Module namespace access
    test_same("ns.exported");
    test("ns['exported']", "ns.exported");
    test("ns['prop' + 'name']", "ns.propname");
}

#[test]
fn test_advanced_esm_patterns() {
    // Re-exports with optimization potential
    test_same("export { a } from './module.js';");
    test_same("export { a as b } from './module.js';");
    test_same("export * from './module.js';");
    test_same("export * as namespace from './module.js';");

    // Dynamic import with conditional expressions
    test("import(condition ? './a.js' : './b.js')", "import(condition ? './a.js' : './b.js')");
    test("import(true ? './production.js' : './development.js')", "import('./production.js')");
    test("import(false ? './production.js' : './development.js')", "import('./development.js')");

    // Import assertions/attributes with string optimization
    test(
        "import('./data.json', { assert: { type: 'js' + 'on' } })",
        "import('./data.json', { assert: { type: 'json' } })",
    );
    test(
        "import('./styles.css', { with: { type: 'cs' + 's' } })",
        "import('./styles.css', { with: { type: 'css' } })",
    );

    // Complex import.meta patterns
    test(
        "const url = import.meta.url + '/relative';",
        "const url = import.meta.url + '/relative';",
    );
    test("import.meta.resolve('./' + 'module' + '.js')", "import.meta.resolve('./module.js')");
    test("import.meta.resolve('module-' + (2 + 3))", "import.meta.resolve('module-5')");

    // Top-level await with expressions that can be optimized
    test(
        "const mod = await import('./' + 'dynamic' + '.js');",
        "const mod = await import('./dynamic.js');",
    );
    test("await (async () => 1 + 2)()", "await (async () => 3)()");

    // ESM in conditionals and loops
    test("if (true) { import('./conditional.js'); }", "import('./conditional.js');");
    test("if (false) { import('./never.js'); }", "");
    test(
        "for (const module of modules) { await import(module); }",
        "for (let module of modules) await import(module);",
    );

    // Module caching patterns
    test(
        "const cache = new Map(); const getModule = (name) => cache.get(name) ?? import(name);",
        "const cache = /* @__PURE__ */ new Map(), getModule = (name) => cache.get(name) ?? import(name);",
    );

    // ESM with destructuring and optimization
    test(
        "const { ['prop' + 'name']: value } = await import('./module.js');",
        "const { propname: value } = await import('./module.js');",
    );

    // Side-effect imports with conditions
    test("if (true) import('./polyfill.js');", "import('./polyfill.js');");
    test("if (false) import('./polyfill.js');", "");

    // ESM factory patterns
    test_same("const createModule = () => import('./factory.js');");
    test(
        "const createModule = () => import('prefix' + 'Factory' + '.js');",
        "const createModule = () => import('prefixFactory.js');",
    );
}

#[test]
fn test_esm_module_patterns() {
    // Barrel exports
    test_same("export * from './components/Button.js';");
    test_same("export * from './components/Input.js';");
    test_same("export { default as Button } from './components/Button.js';");

    // Conditional exports
    test(
        "export const config = env === 'prod' ? prodConfig : devConfig;",
        "export const config = env === 'prod' ? prodConfig : devConfig;",
    );
    test(
        "export const isProduction = 'production' === 'production';",
        "export const isProduction = !0;",
    );

    // Dynamic re-exports with computed properties
    test_same("const obj = { [computedName]: value }; export { obj };");
    test(
        "const obj = { ['static' + 'Name']: value }; export { obj };",
        "const obj = { staticName: value }; export { obj };",
    );

    // ESM with IIFE patterns
    test("export default (() => { return 1 + 2; })();", "export default (() => 3)();");
    test(
        "export default (async () => { return await fetchData(); })();",
        "export default (async () => await fetchData())();",
    );

    // Import/export hoisting behavior (should preserve order)
    test_same("console.log('before'); import './side-effect.js'; console.log('after');");

    // Module-level variables
    test(
        "let moduleVar = 5 + 3; export { moduleVar };",
        "let moduleVar = 8; export { moduleVar };",
    );
    test(
        "const moduleConst = 'hello' + ' world'; export default moduleConst;",
        "const moduleConst = 'hello world'; export default 'hello world';",
    );

    // Import maps and specifier resolution (preserve as-is)
    test_same("import React from 'react';");
    test_same("import { Component } from '@org/package';");
    test_same("import utils from '#internal/utils';");

    // Worker and shared worker imports
    test_same("new Worker(new URL('./worker.js', import.meta.url));");
    test(
        "new Worker(new URL('./' + 'worker' + '.js', import.meta.url));",
        "new Worker(new URL('./worker.js', import.meta.url));",
    );

    // Preload patterns
    test_same(
        "document.head.appendChild(Object.assign(document.createElement('link'), { rel: 'modulepreload', href: './module.js' }));",
    );

    // Module federation patterns
    test_same("const container = await window.__webpack_init_sharing__('default');");
    test_same("const factory = await container.get('./Component');");

    // ESM with string template optimization
    test("import(`./modules/${1 + 2}.js`)", "import('./modules/3.js')");
    test("export const path = `./dist/${'app' + '.js'}`;", "export const path = './dist/app.js';");

    // Complex conditional imports
    test(
        "const loader = condition ? () => import('./a.js') : () => import('./b.js');",
        "const loader = condition ? () => import('./a.js') : () => import('./b.js');",
    );
    test(
        "const loader = true ? () => import('./production.js') : () => import('./dev.js');",
        "const loader = () => import('./production.js');",
    );

    // Import with computed expressions
    test("import('module-' + (version || 'latest'))", "import('module-' + (version || 'latest'))");
    test("import('module-' + (2 + 3))", "import('module-5')");
}

#[test]
fn test_bigint_edge_cases() {
    // Test BigInt operations - these ARE optimized by oxc
    test("return 1n + 2n", "return 3n");
    test_same("return 10n * 5n");
    test_same("return 100n / 4n");
    test("return 123n > 456n", "return !1");
    test("return 456n > 123n", "return !0");

    // Test mixed BigInt and Number (should NOT optimize - runtime error)
    test_same("1n + 2"); // TypeError at runtime
    test_same("BigInt(5) + 3"); // should not optimize
}

#[test]
fn test_unicode_string_edge_cases() {
    // Test Unicode in strings - these ARE optimized by oxc
    test("return '🚀'.length", "return 2");
    test("return '𝒽𝑒𝓁𝓁𝑜'.length", "return 10");

    // Test Unicode escape sequences
    test_same("return '\\u0048\\u0065\\u006C\\u006C\\u006F'");
    test_same("return '\\u{1F680}'");

    // Test normalization issues (in return context)
    test("return 'é' === '\\u0065\\u0301'", "return !1"); // composed vs decomposed - could be optimized
}

#[test]
fn test_performance_regression_patterns() {
    // Test patterns that might cause exponential behavior in the minifier
    test_same("a ? b ? c ? d ? e ? f ? g ? h ? i : j : k : l : m : n : o : p : q");

    // Test patterns with many similar subexpressions
    test_same("obj.prop + obj.prop + obj.prop + obj.prop + obj.prop");

    // Test deeply nested expressions
    test_same("((((((((((a))))))))))");

    // Test very long identifier names
    test_same("veryLongVariableNameThatMightCauseProblemsInTheMinifier = 1;");

    // Test large object literals
    test_same(
        "const obj = {a:1,b:2,c:3,d:4,e:5,f:6,g:7,h:8,i:9,j:10,k:11,l:12,m:13,n:14,o:15,p:16,q:17,r:18,s:19,t:20}",
    );
}
