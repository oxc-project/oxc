//! Test cases from vanilla eslint

use super::NoUnusedVars;
use crate::{tester::Tester, RuleMeta as _};

/// These are tests from ESLint that are not passing. If you make a change that
/// causes this test to fail, that's a good thing!
///
/// 1. Delete the offending test case from this function
/// 2. Find where it is commented out in [`test`] and un-comment it. Cases that are in `pass` in
///    [`fixme`] will be in [`fail`] in [`test`]
/// 3. Add it to your PR :)
#[test]
fn fixme() {
    let pass = vec![
        ("function foo(cb) { cb = function(a) { return cb(1 + a); }(); } foo();", None),
        ("function foo(cb) { cb = (0, function(a) { cb(1 + a); }); } foo();", None),
        (
            "let x = [];
        	x = x.concat(x);",
            None,
        ), // { "ecmaVersion": 2015 },
    ];
    let fail = vec![];
    Tester::new(NoUnusedVars::NAME, pass, fail).intentionally_allow_no_fix_tests().test();
}

#[test]
fn test() {
    let pass = vec![
        (
            "var foo = 5;
			
			label: while (true) {
			    console.log(foo);
			    break label;
			}",
            None,
        ),
        (
            "var foo = 5;
			
			while (true) {
			    console.log(foo);
			    break;
			}",
            None,
        ),
        (
            "for (let prop in box) { box[prop] = parseInt(box[prop]); }",
            None,
        ), // { "ecmaVersion": 6 },
        (
            "var box = {a: 2};
            for (var prop in box) {
                box[prop] = parseInt(box[prop]);
			}",
            None,
        ),
        ("f({ set foo(a) { return; } });", None),
        ("a; var a;", Some(serde_json::json!(["all"]))),
        ("var a=10; alert(a);", Some(serde_json::json!(["all"]))),
        ("var a=10; (function() { alert(a); })();", Some(serde_json::json!(["all"]))),
        (
            "var a=10; (function() { setTimeout(function() { alert(a); }, 0); })();",
            Some(serde_json::json!(["all"])),
        ),
        ("var a=10; d[a] = 0;", Some(serde_json::json!(["all"]))),
        ("(function() { var a=10; return a; })();", Some(serde_json::json!(["all"]))),
        ("(function g() {})()", Some(serde_json::json!(["all"]))),
        ("function f(a) {alert(a);}; f();", Some(serde_json::json!(["all"]))),
        (
            "var c = 0; function f(a){ var b = a; return b; }; f(c);",
            Some(serde_json::json!(["all"])),
        ),
        ("function a(x, y){ return y; }; a();", Some(serde_json::json!(["all"]))),
        (
            "var arr1 = [1, 2]; var arr2 = [3, 4]; for (var i in arr1) { arr1[i] = 5; } for (var i in arr2) { arr2[i] = 10; }",
            Some(serde_json::json!(["all"])),
        ),
        ("var a=10;", Some(serde_json::json!(["local"]))),
        (r#"var min = "min"; Math[min];"#, Some(serde_json::json!(["all"]))),
        ("Foo.bar = function(baz) { return baz; };", Some(serde_json::json!(["all"]))),
        ("myFunc(function foo() {}.bind(this))", None),
        ("myFunc(function foo(){}.toString())", None),
        (
            "function foo(first, second) {
			doStuff(function() {
			console.log(second);});}; foo()",
            None,
        ),
        ("(function() { var doSomething = function doSomething() {}; doSomething() }())", None),
        // ("/*global a */ a;", None),
        ("var a=10; (function() { alert(a); })();", Some(serde_json::json!([{ "vars": "all" }]))),
        (
            "function g(bar, baz) { return baz; }; g();",
            Some(serde_json::json!([{ "vars": "all" }])),
        ),
        (
            "function g(bar, baz) { return baz; }; g();",
            Some(serde_json::json!([{ "vars": "all", "args": "after-used" }])),
        ),
        (
            "function g(bar, baz) { return bar; }; g();",
            Some(serde_json::json!([{ "vars": "all", "args": "none" }])),
        ),
        (
            "function g(bar, baz) { return 2; }; g();",
            Some(serde_json::json!([{ "vars": "all", "args": "none" }])),
        ),
        (
            "function g(bar, baz) { return bar + baz; }; g();",
            Some(serde_json::json!([{ "vars": "local", "args": "all" }])),
        ),
        (
            "var g = function(bar, baz) { return 2; }; g();",
            Some(serde_json::json!([{ "vars": "all", "args": "none" }])),
        ),
        ("(function z() { z(); })();", None),
        (" ", None), // { "globals": { "a": true } },
        (
            r#"var who = "Paul";
			module.exports = `Hello ${who}!`;"#,
            None,
        ), // { "ecmaVersion": 6 },
        ("export var foo = 123;", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export function foo () {}", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("let toUpper = (partial) => partial.toUpperCase; export {toUpper}", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export class foo {}", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("class Foo{}; var x = new Foo(); x.foo()", None), // { "ecmaVersion": 6 },
        (
            r#"const foo = "hello!";function bar(foobar = foo) {  foobar.replace(/!$/, " world!");}
			bar();"#,
            None,
        ), // { "ecmaVersion": 6 },
        ("function Foo(){}; var x = new Foo(); x.foo()", None),
        ("function foo() {var foo = 1; return foo}; foo();", None),
        ("function foo(foo) {return foo}; foo(1);", None),
        ("function foo() {function foo() {return 1;}; return foo()}; foo();", None),
        ("function foo() {var foo = 1; return foo}; foo();", None), // { "ecmaVersion": 6 },
        ("function foo(foo) {return foo}; foo(1);", None),          // { "ecmaVersion": 6 },
        ("function foo() {function foo() {return 1;}; return foo()}; foo();", None), // { "ecmaVersion": 6 },
        ("const x = 1; const [y = x] = []; foo(y);", None), // { "ecmaVersion": 6 },
        ("const x = 1; const {y = x} = {}; foo(y);", None), // { "ecmaVersion": 6 },
        ("const x = 1; const {z: [y = x]} = {}; foo(y);", None), // { "ecmaVersion": 6 },
        ("const x = []; const {z: [y] = x} = {}; foo(y);", None), // { "ecmaVersion": 6 },
        ("const x = 1; let y; [y = x] = []; foo(y);", None), // { "ecmaVersion": 6 },
        ("const x = 1; let y; ({z: [y = x]} = {}); foo(y);", None), // { "ecmaVersion": 6 },
        ("const x = []; let y; ({z: [y] = x} = {}); foo(y);", None), // { "ecmaVersion": 6 },
        ("const x = 1; function foo(y = x) { bar(y); } foo();", None), // { "ecmaVersion": 6 },
        ("const x = 1; function foo({y = x} = {}) { bar(y); } foo();", None), // { "ecmaVersion": 6 },
        ("const x = 1; function foo(y = function(z = x) { bar(z); }) { y(); } foo();", None), // { "ecmaVersion": 6 },
        ("const x = 1; function foo(y = function() { bar(x); }) { y(); } foo();", None), // { "ecmaVersion": 6 },
        ("var x = 1; var [y = x] = []; foo(y);", None), // { "ecmaVersion": 6 },
        ("var x = 1; var {y = x} = {}; foo(y);", None), // { "ecmaVersion": 6 },
        ("var x = 1; var {z: [y = x]} = {}; foo(y);", None), // { "ecmaVersion": 6 },
        ("var x = []; var {z: [y] = x} = {}; foo(y);", None), // { "ecmaVersion": 6 },
        ("var x = 1, y; [y = x] = []; foo(y);", None),  // { "ecmaVersion": 6 },
        ("var x = 1, y; ({z: [y = x]} = {}); foo(y);", None), // { "ecmaVersion": 6 },
        ("var x = [], y; ({z: [y] = x} = {}); foo(y);", None), // { "ecmaVersion": 6 },
        ("var x = 1; function foo(y = x) { bar(y); } foo();", None), // { "ecmaVersion": 6 },
        ("var x = 1; function foo({y = x} = {}) { bar(y); } foo();", None), // { "ecmaVersion": 6 },
        ("var x = 1; function foo(y = function(z = x) { bar(z); }) { y(); } foo();", None), // { "ecmaVersion": 6 },
        ("var x = 1; function foo(y = function() { bar(x); }) { y(); } foo();", None), // { "ecmaVersion": 6 },
        // ("/*exported toaster*/ var toaster = 'great'", None),
        // ("/*exported toaster, poster*/ var toaster = 1; poster = 0;", None),
        // ("/*exported x*/ var { x } = y", None), // { "ecmaVersion": 6 },
        // ("/*exported x, y*/  var { x, y } = z", None), // { "ecmaVersion": 6 },
        // ("/*eslint custom/use-every-a:1*/ var a;", None),
        // ("/*eslint custom/use-every-a:1*/ !function(a) { return 1; }", None),
        // ("/*eslint custom/use-every-a:1*/ !function() { var a; return 1 }", None),
        ("var _a;", Some(serde_json::json!([{ "vars": "all", "varsIgnorePattern": "^_" }]))),
        (
            "var a; function foo() { var _b; } foo();",
            Some(serde_json::json!([{ "vars": "local", "varsIgnorePattern": "^_" }])),
        ),
        (
            "function foo(_a) { } foo();",
            Some(serde_json::json!([{ "args": "all", "argsIgnorePattern": "^_" }])),
        ),
        (
            "function foo(a, _b) { return a; } foo();",
            Some(serde_json::json!([{ "args": "after-used", "argsIgnorePattern": "^_" }])),
        ),
        (
            "var [ firstItemIgnored, secondItem ] = items;
			console.log(secondItem);",
            Some(serde_json::json!([{ "vars": "all", "varsIgnorePattern": "[iI]gnored" }])),
        ), // { "ecmaVersion": 6 },
        (
            "const [ a, _b, c ] = items;
			console.log(a+c);",
            Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
        ), // { "ecmaVersion": 6 },
        (
            "const [ [a, _b, c] ] = items;
			console.log(a+c);",
            Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
        ), // { "ecmaVersion": 6 },
        (
            "const { x: [_a, foo] } = bar;
			console.log(foo);",
            Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
        ), // { "ecmaVersion": 6 },
        (
            "function baz([_b, foo]) { foo; };
			baz()",
            Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
        ), // { "ecmaVersion": 6 },
        (
            "function baz({x: [_b, foo]}) {foo};
			baz()",
            Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
        ), // { "ecmaVersion": 6 },
        (
            "function baz([{x: [_b, foo]}]) {foo};
			baz()",
            Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
        ), // { "ecmaVersion": 6 },
        (
            "
            let _a, b;
            foo.forEach(item => {
                [_a, b] = item;
                doSomething(b);
            });
            ",
            Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
        ), // { "ecmaVersion": 6 },
        (
            "
            // doesn't report _x
            let _x, y;
            _x = 1;
            [_x, y] = foo;
            y;

            // doesn't report _a
            let _a, b;
            [_a, b] = foo;
            _a = 1;
            b;
            ",
            Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
        ), // { "ecmaVersion": 2018 },
        (
            "
            // doesn't report _x
            let _x, y;
            _x = 1;
            [_x, y] = foo;
            y;

            // doesn't report _a
            let _a, b;
            _a = 1;
            ({_a, ...b } = foo);
            b;
            ",
            Some(
                serde_json::json!([{ "destructuredArrayIgnorePattern": "^_", "ignoreRestSiblings": true }]),
            ),
        ), // { "ecmaVersion": 2018 },
        (
            "try {} catch ([firstError]) {}",
            Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "Error$" }])),
        ), // { "ecmaVersion": 2015 },
        ("(function(obj) { var name; for ( name in obj ) return; })({});", None),
        ("(function(obj) { var name; for ( name in obj ) { return; } })({});", None),
        ("(function(obj) { for ( var name in obj ) { return true } })({})", None),
        ("(function(obj) { for ( var name in obj ) return true })({})", None),
        ("(function(obj) { let name; for ( name in obj ) return; })({});", None), // { "ecmaVersion": 6 },
        ("(function(obj) { let name; for ( name in obj ) { return; } })({});", None), // { "ecmaVersion": 6 },
        ("(function(obj) { for ( let name in obj ) { return true } })({})", None), // { "ecmaVersion": 6 },
        ("(function(obj) { for ( let name in obj ) return true })({})", None), // { "ecmaVersion": 6 },
        ("(function(obj) { for ( const name in obj ) { return true } })({})", None), // { "ecmaVersion": 6 },
        ("(function(obj) { for ( const name in obj ) return true })({})", None), // { "ecmaVersion": 6 },
        ("(function(iter) { let name; for ( name of iter ) return; })({});", None), // { "ecmaVersion": 6 },
        ("(function(iter) { let name; for ( name of iter ) { return; } })({});", None), // { "ecmaVersion": 6 },
        ("(function(iter) { for ( let name of iter ) { return true } })({})", None), // { "ecmaVersion": 6 },
        ("(function(iter) { for ( let name of iter ) return true })({})", None), // { "ecmaVersion": 6 },
        ("(function(iter) { for ( const name of iter ) { return true } })({})", None), // { "ecmaVersion": 6 },
        ("(function(iter) { for ( const name of iter ) return true })({})", None), // { "ecmaVersion": 6 },
        ("let x = 0; foo = (0, x++);", None), // { "ecmaVersion": 6 },
        ("let x = 0; foo = (0, x += 1);", None), // { "ecmaVersion": 6 },
        ("let x = 0; foo = (0, x = x + 1);", None), // { "ecmaVersion": 6 },
        ("try{}catch(err){}", Some(serde_json::json!([{ "caughtErrors": "none" }]))),
        (
            "try{}catch(err){console.error(err);}",
            Some(serde_json::json!([{ "caughtErrors": "all" }])),
        ),
        (
            "try{}catch(ignoreErr){}",
            Some(serde_json::json!([{ "caughtErrorsIgnorePattern": "^ignore" }])),
        ),
        (
            "try{}catch(ignoreErr){}",
            Some(
                serde_json::json!([{ "caughtErrors": "all", "caughtErrorsIgnorePattern": "^ignore" }]),
            ),
        ),
        (
            "try {} catch ({ message, stack }) {}",
            Some(serde_json::json!([{ "caughtErrorsIgnorePattern": "message|stack" }])),
        ), // { "ecmaVersion": 2015 },
        (
            "try {} catch ({ errors: [firstError] }) {}",
            Some(serde_json::json!([{ "caughtErrorsIgnorePattern": "Error$" }])),
        ), // { "ecmaVersion": 2015 },
        (
            "try{}catch(err){}",
            Some(serde_json::json!([{ "caughtErrors": "none", "vars": "all", "args": "all" }])),
        ),
        (
            "const data = { type: 'coords', x: 1, y: 2 };
			const { type, ...coords } = data;
			 console.log(coords);",
            Some(serde_json::json!([{ "ignoreRestSiblings": true }])),
        ), // { "ecmaVersion": 2018 },
        (
            "try {} catch ({ foo, ...bar }) { console.log(bar); }",
            Some(serde_json::json!([{ "ignoreRestSiblings": true }])),
        ), // { "ecmaVersion": 2018 },
        ("var a = 0, b; b = a = a + 1; foo(b);", None),
        ("var a = 0, b; b = a += a + 1; foo(b);", None),
        ("var a = 0, b; b = a++; foo(b);", None),
        ("function foo(a) { var b = a = a + 1; bar(b) } foo();", None),
        ("function foo(a) { var b = a += a + 1; bar(b) } foo();", None),
        ("function foo(a) { var b = a++; bar(b) } foo();", None),
        (
            r#"var unregisterFooWatcher;
			// ...
			unregisterFooWatcher = $scope.$watch( "foo", function() {
			    // ...some code..
			    unregisterFooWatcher();
			});
			"#,
            None,
        ),
        (
            "var ref;
			ref = setInterval(
			    function(){
			        clearInterval(ref);
			    }, 10);
			",
            None,
        ),
        (
            "var _timer;
			function f() {
			    _timer = setTimeout(function () {}, _timer ? 100 : 0);
			}
			f();
			",
            None,
        ),
        (
            "function foo(cb) { cb = function() { function something(a) { cb(1 + a); } register(something); }(); } foo();",
            None,
        ),
        ("function* foo(cb) { cb = yield function(a) { cb(1 + a); }; } foo();", None), // { "ecmaVersion": 6 },
        ("function foo(cb) { cb = tag`hello${function(a) { cb(1 + a); }}`; } foo();", None), // { "ecmaVersion": 6 },
        ("function foo(cb) { var b; cb = b = function(a) { cb(1 + a); }; b(); } foo();", None),
        (
            "function someFunction() {
			    var a = 0, i;
			    for (i = 0; i < 2; i++) {
			        a = myFunction(a);
			    }
			}
			someFunction();
			",
            None,
        ),
        ("(function(a, b, {c, d}) { d })", Some(serde_json::json!([{ "argsIgnorePattern": "c" }]))), // { "ecmaVersion": 6 },
        ("(function(a, b, {c, d}) { c })", Some(serde_json::json!([{ "argsIgnorePattern": "d" }]))), // { "ecmaVersion": 6 },
        ("(function(a, b, c) { c })", Some(serde_json::json!([{ "argsIgnorePattern": "c" }]))),
        (
            "(function(a, b, {c, d}) { c })",
            Some(serde_json::json!([{ "argsIgnorePattern": "[cd]" }])),
        ), // { "ecmaVersion": 6 },
        ("(class { set foo(UNUSED) {} })", None), // { "ecmaVersion": 6 },
        ("class Foo { set bar(UNUSED) {} } console.log(Foo)", None), // { "ecmaVersion": 6 },
        (
            "(({a, ...rest}) => rest)",
            Some(serde_json::json!([{ "args": "all", "ignoreRestSiblings": true }])),
        ), // { "ecmaVersion": 2018 },
        (
            "let foo, rest;
			({ foo, ...rest } = something);
			console.log(rest);",
            Some(serde_json::json!([{ "ignoreRestSiblings": true }])),
        ), // { "ecmaVersion": 2020 },
        // ("/*eslint custom/use-every-a:1*/ !function(b, a) { return 1 }", None),
        ("var a = function () { a(); }; a();", None),
        ("var a = function(){ return function () { a(); } }; a();", None),
        ("const a = () => { a(); }; a();", None), // { "ecmaVersion": 2015 },
        ("const a = () => () => { a(); }; a();", None), // { "ecmaVersion": 2015 },
        (r#"export * as ns from "source""#, None), // { "ecmaVersion": 2020, "sourceType": "module" },
        ("import.meta", None), // { "ecmaVersion": 2020, "sourceType": "module" },
        // NOTE (@DonIsaac) ESLint thinks this counts as being used, I disagree
        // ("var a; a ||= 1;", None), // { "ecmaVersion": 2021 },
        // ("var a; a &&= 1;", None), // { "ecmaVersion": 2021 },
        // ("var a; a ??= 1;", None), // { "ecmaVersion": 2021 },
        (
            "class Foo { static {} }",
            Some(serde_json::json!([{ "ignoreClassWithStaticInitBlock": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class Foo { static {} }",
            Some(
                serde_json::json!([{ "ignoreClassWithStaticInitBlock": true, "varsIgnorePattern": "^_" }]),
            ),
        ), // { "ecmaVersion": 2022 },
        (
            "class Foo { static {} }",
            Some(
                serde_json::json!([{ "ignoreClassWithStaticInitBlock": false, "varsIgnorePattern": "^Foo" }]),
            ),
        ), // { "ecmaVersion": 2022 },
        (
            "const a = 5; const _c = a + 5;",
            Some(
                serde_json::json!([{ "args": "all", "varsIgnorePattern": "^_", "reportUsedIgnorePattern": true }]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "(function foo(a, _b) { return a + 5 })(5)",
            Some(
                serde_json::json!([{ "args": "all", "argsIgnorePattern": "^_", "reportUsedIgnorePattern": true }]),
            ),
        ),
        (
            "const [ a, _b, c ] = items;
			console.log(a+c);",
            Some(
                serde_json::json!([{ "destructuredArrayIgnorePattern": "^_", "reportUsedIgnorePattern": true }]),
            ),
        ), // { "ecmaVersion": 6 }
    ];

    let fail = vec![
        ("function foox() { return foox(); }", None),
        // ("(function() { function foox() { if (true) { return foox(); } } }())", None),
        ("var a=10", None),
        ("function f() { var a = 1; return function(){ f(a *= 2); }; }", None),
        ("function f() { var a = 1; return function(){ f(++a); }; }", None),
        // ("/*global a */", None),
        (
            "function foo(first, second) {
			doStuff(function() {
			console.log(second);});};",
            None,
        ),
        ("var a=10;", Some(serde_json::json!(["all"]))),
        ("var a=10; a=20;", Some(serde_json::json!(["all"]))),
        ("var a=10; (function() { var a = 1; alert(a); })();", Some(serde_json::json!(["all"]))),
        ("var a=10, b=0, c=null; alert(a+b)", Some(serde_json::json!(["all"]))),
        (
            "var a=10, b=0, c=null; setTimeout(function() { var b=2; alert(a+b+c); }, 0);",
            Some(serde_json::json!(["all"])),
        ),
        (
            "var a=10, b=0, c=null; setTimeout(function() { var b=2; var c=2; alert(a+b+c); }, 0);",
            Some(serde_json::json!(["all"])),
        ),
        ("function f(){var a=[];return a.map(function(){});}", Some(serde_json::json!(["all"]))),
        ("function f(){var a=[];return a.map(function g(){});}", Some(serde_json::json!(["all"]))),
        (
            "function foo() {function foo(x) {
			return x; }; return function() {return foo; }; }",
            None,
        ),
        (
            "function f(){var x;function a(){x=42;}function b(){alert(x);}}",
            Some(serde_json::json!(["all"])),
        ),
        ("function f(a) {}; f();", Some(serde_json::json!(["all"]))),
        ("function a(x, y, z){ return y; }; a();", Some(serde_json::json!(["all"]))),
        ("var min = Math.min", Some(serde_json::json!(["all"]))),
        ("var min = {min: 1}", Some(serde_json::json!(["all"]))),
        ("Foo.bar = function(baz) { return 1; };", Some(serde_json::json!(["all"]))),
        ("var min = {min: 1}", Some(serde_json::json!([{ "vars": "all" }]))),
        (
            "function gg(baz, bar) { return baz; }; gg();",
            Some(serde_json::json!([{ "vars": "all" }])),
        ),
        (
            "(function(foo, baz, bar) { return baz; })();",
            Some(serde_json::json!([{ "vars": "all", "args": "after-used" }])),
        ),
        (
            "(function(foo, baz, bar) { return baz; })();",
            Some(serde_json::json!([{ "vars": "all", "args": "all" }])),
        ),
        (
            "(function z(foo) { var bar = 33; })();",
            Some(serde_json::json!([{ "vars": "all", "args": "all" }])),
        ),
        ("(function z(foo) { z(); })();", Some(serde_json::json!([{}]))),
        (
            "function f() { var a = 1; return function(){ f(a = 2); }; }",
            Some(serde_json::json!([{}])),
        ),
        (r#"import x from "y";"#, None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export function fn2({ x, y }) { console.log(x); };", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export function fn2( x, y ) { console.log(x); };", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("/*exported max*/ var max = 1, min = {min: 1}", None),
        ("/*exported x*/ var { x, y } = z", None), // { "ecmaVersion": 6 },
        ("var _a; var b;", Some(serde_json::json!([{ "vars": "all", "varsIgnorePattern": "^_" }]))),
        (
            "var a; function foo() { var _b; var c_; } foo();",
            Some(serde_json::json!([{ "vars": "local", "varsIgnorePattern": "^_" }])),
        ),
        (
            "function foo(a, _b) { } foo();",
            Some(serde_json::json!([{ "args": "all", "argsIgnorePattern": "^_" }])),
        ),
        (
            "function foo(a, _b, c) { return a; } foo();",
            Some(serde_json::json!([{ "args": "after-used", "argsIgnorePattern": "^_" }])),
        ),
        (
            "function foo(_a) { } foo();",
            Some(serde_json::json!([{ "args": "all", "argsIgnorePattern": "[iI]gnored" }])),
        ),
        (
            "var [ firstItemIgnored, secondItem ] = items;",
            Some(serde_json::json!([{ "vars": "all", "varsIgnorePattern": "[iI]gnored" }])),
        ), // { "ecmaVersion": 6 },
        (
            "
            const array = ['a', 'b', 'c'];
            const [a, _b, c] = array;
            const newArray = [a, c];
            ",
            Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
        ), // { "ecmaVersion": 2020 },
        (
            "
            const array = ['a', 'b', 'c', 'd', 'e'];
            const [a, _b, c] = array;
            ",
            Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
        ), // { "ecmaVersion": 2020 },
        (
            "
            const array = ['a', 'b', 'c'];
            const [a, _b, c] = array;
            const fooArray = ['foo'];
            const barArray = ['bar'];
            const ignoreArray = ['ignore'];
            ",
            Some(
                serde_json::json!([{ "destructuredArrayIgnorePattern": "^_", "varsIgnorePattern": "ignore" }]),
            ),
        ), // { "ecmaVersion": 2020 },
        (
            "
            const array = [obj];
            const [{_a, foo}] = array;
            console.log(foo);
            ",
            Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
        ), // { "ecmaVersion": 2020 },
        (
            "
            function foo([{_a, bar}]) {
                bar;
            }
            foo();
            ",
            Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
        ), // { "ecmaVersion": 2020 },
        (
            "
            let _a, b;

            foo.forEach(item => {
                [a, b] = item;
            });
            ",
            Some(serde_json::json!([{ "destructuredArrayIgnorePattern": "^_" }])),
        ), // { "ecmaVersion": 2020 },
        ("(function(obj) { var name; for ( name in obj ) { i(); return; } })({});", None),
        ("(function(obj) { var name; for ( name in obj ) { } })({});", None),
        ("(function(obj) { for ( var name in obj ) { } })({});", None),
        ("(function(iter) { var name; for ( name of iter ) { i(); return; } })({});", None), // { "ecmaVersion": 6 },
        ("(function(iter) { var name; for ( name of iter ) { } })({});", None), // { "ecmaVersion": 6 },
        ("(function(iter) { for ( var name of iter ) { } })({});", None), // { "ecmaVersion": 6 },
        // (
        //     "
        // 	/* global foobar, foo, bar */
        // 	foobar;",
        //     None,
        // ),
        // (
        //     "
        // 	/* global foobar,
        // 	   foo,
        // 	   bar
        // 	 */
        // 	foobar;",
        //     None,
        // ),
        (
            "const data = { type: 'coords', x: 1, y: 2 };
			const { type, ...coords } = data;
			 console.log(coords);",
            None,
        ), // { "ecmaVersion": 2018 },
        (
            "const data = { type: 'coords', x: 2, y: 2 };
			const { type, ...coords } = data;
			 console.log(type)",
            Some(serde_json::json!([{ "ignoreRestSiblings": true }])),
        ), // { "ecmaVersion": 2018 },
        (
            "let type, coords;
			({ type, ...coords } = data);
			 console.log(type)",
            Some(serde_json::json!([{ "ignoreRestSiblings": true }])),
        ), // { "ecmaVersion": 2018 },
        (
            "const data = { type: 'coords', x: 3, y: 2 };
			const { type, ...coords } = data;
			 console.log(type)",
            None,
        ), // { "ecmaVersion": 2018 },
        (
            "const data = { vars: ['x','y'], x: 1, y: 2 };
			const { vars: [x], ...coords } = data;
			 console.log(coords)",
            None,
        ), // { "ecmaVersion": 2018 },
        (
            "const data = { defaults: { x: 0 }, x: 1, y: 2 };
			const { defaults: { x }, ...coords } = data;
			 console.log(coords)",
            None,
        ), // { "ecmaVersion": 2018 },
        (
            "(({a, ...rest}) => {})",
            Some(serde_json::json!([{ "args": "all", "ignoreRestSiblings": true }])),
        ), // { "ecmaVersion": 2018 },
        // (
        //     "/* global a$fooz,$foo */
        // 	a$fooz;",
        //     None,
        // ),
        // (
        //     "/* globals a$fooz, $ */
        // 	a$fooz;",
        //     None,
        // ),
        // ("/*globals $foo*/", None),
        // ("/* global global*/", None),
        // ("/*global foo:true*/", None),
        // (
        //     "/*global 変数, 数*/
        // 	変数;",
        //     None,
        // ),
        // (
        //     "/*global 𠮷𩸽, 𠮷*/
        // 	\\u{20BB7}\\u{29E3D};",
        //     None,
        // ), // { "ecmaVersion": 6 },
        ("export default function(a) {}", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export default function(a, b) { console.log(a); }", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export default (function(a) {});", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export default (function(a, b) { console.log(a); });", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export default (a) => {};", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("export default (a, b) => { console.log(a); };", None), // { "ecmaVersion": 6, "sourceType": "module" },
        ("try{}catch(err){};", None),
        ("try{}catch(err){};", Some(serde_json::json!([{ "caughtErrors": "all" }]))),
        (
            "try{}catch(err){};",
            Some(
                serde_json::json!([{ "caughtErrors": "all", "caughtErrorsIgnorePattern": "^ignore" }]),
            ),
        ),
        (
            "try{}catch(err){};",
            Some(serde_json::json!([{ "caughtErrors": "all", "varsIgnorePattern": "^err" }])),
        ),
        (
            "try{}catch(err){};",
            Some(serde_json::json!([{ "caughtErrors": "all", "varsIgnorePattern": "^." }])),
        ),
        (
            "try{}catch(ignoreErr){}try{}catch(err){};",
            Some(
                serde_json::json!([{ "caughtErrors": "all", "caughtErrorsIgnorePattern": "^ignore" }]),
            ),
        ),
        (
            "try{}catch(error){}try{}catch(err){};",
            Some(
                serde_json::json!([{ "caughtErrors": "all", "caughtErrorsIgnorePattern": "^ignore" }]),
            ),
        ),
        (
            "try{}catch(err){};",
            Some(serde_json::json!([{ "vars": "all", "args": "all", "caughtErrors": "all" }])),
        ),
        (
            "try{}catch(err){};",
            Some(
                serde_json::json!([{ "vars": "all", "args": "all", "caughtErrors": "all","argsIgnorePattern": "^er" }]),
            ),
        ),
        ("var a = 0; a = a + 1;", None),
        ("var a = 0; a = a + a;", None),
        ("var a = 0; a += a + 1;", None),
        ("var a = 0; a++;", None),
        ("function foo(a) { a = a + 1 } foo();", None),
        ("function foo(a) { a += a + 1 } foo();", None),
        ("function foo(a) { a++ } foo();", None),
        ("var a = 3; a = a * 5 + 6;", None),
        ("var a = 2, b = 4; a = a * 2 + b;", None),
        // https://github.com/oxc-project/oxc/issues/4436
        ("function foo(cb) { cb = function(a) { cb(1 + a); }; bar(not_cb); } foo();", None),
        ("function foo(cb) { cb = (function(a) { cb(1 + a); }, cb); } foo();", None),
        // ("function foo(cb) { cb = (0, function(a) { cb(1 + a); }); } foo();", None),
        (
            "while (a) {
			    function foo(b) {
			        b = b + 1;
			    }
			    foo()
			}",
            None,
        ),
        ("(function(a, b, c) {})", Some(serde_json::json!([{ "argsIgnorePattern": "c" }]))),
        ("(function(a, b, {c, d}) {})", Some(serde_json::json!([{ "argsIgnorePattern": "[cd]" }]))), // { "ecmaVersion": 6 },
        ("(function(a, b, {c, d}) {})", Some(serde_json::json!([{ "argsIgnorePattern": "c" }]))), // { "ecmaVersion": 6 },
        ("(function(a, b, {c, d}) {})", Some(serde_json::json!([{ "argsIgnorePattern": "d" }]))), // { "ecmaVersion": 6 },
        //         (
        //             "/*global
        // foo*/",
        //             None,
        //         ),
        ("(function ({ a }, b ) { return b; })();", None), // { "ecmaVersion": 2015 },
        ("(function ({ a }, { b, c } ) { return b; })();", None), // { "ecmaVersion": 2015 },
        (
            "let x = 0;
			            x++, x = 0;",
            None,
        ), // { "ecmaVersion": 2015 },
        (
            "let x = 0;
			            x++, x = 0;
			            x=3;",
            None,
        ), // { "ecmaVersion": 2015 },
        ("let x = 0; x++, 0;", None),                      // { "ecmaVersion": 2015 },
        ("let x = 0; 0, x++;", None),                      // { "ecmaVersion": 2015 },
        ("let x = 0; 0, (1, x++);", None),                 // { "ecmaVersion": 2015 },
        ("let x = 0; foo = (x++, 0);", None),              // { "ecmaVersion": 2015 },
        ("let x = 0; foo = ((0, x++), 0);", None),         // { "ecmaVersion": 2015 },
        ("let x = 0; x += 1, 0;", None),                   // { "ecmaVersion": 2015 },
        ("let x = 0; 0, x += 1;", None),                   // { "ecmaVersion": 2015 },
        ("let x = 0; 0, (1, x += 1);", None),              // { "ecmaVersion": 2015 },
        ("let x = 0; foo = (x += 1, 0);", None),           // { "ecmaVersion": 2015 },
        ("let x = 0; foo = ((0, x += 1), 0);", None),      // { "ecmaVersion": 2015 },
        (
            "let z = 0;
			            z = z + 1, z = 2;
			            ",
            None,
        ), // { "ecmaVersion": 2020 },
        (
            "let z = 0;
			            z = z+1, z = 2;
			            z = 3;",
            None,
        ), // { "ecmaVersion": 2020 },
        (
            "let z = 0;
			            z = z+1, z = 2;
			            z = z+3;
			            ",
            None,
        ), // { "ecmaVersion": 2020 },
        ("let x = 0; 0, x = x+1;", None),                  // { "ecmaVersion": 2020 },
        ("let x = 0; x = x+1, 0;", None),                  // { "ecmaVersion": 2020 },
        // https://github.com/oxc-project/oxc/issues/4437
        ("let x = 0; foo = ((0, x = x + 1), 0);", None), // { "ecmaVersion": 2020 },
        ("let x = 0; foo = (x = x+1, 0);", None),        // { "ecmaVersion": 2020 },
        ("let x = 0; 0, (1, x=x+1);", None),             // { "ecmaVersion": 2020 },
        ("(function ({ a, b }, { c } ) { return b; })();", None), // { "ecmaVersion": 2015 },
        ("(function ([ a ], b ) { return b; })();", None), // { "ecmaVersion": 2015 },
        ("(function ([ a ], [ b, c ] ) { return b; })();", None), // { "ecmaVersion": 2015 },
        ("(function ([ a, b ], [ c ] ) { return b; })();", None), // { "ecmaVersion": 2015 },
        (
            "(function(_a) {})();",
            Some(serde_json::json!([{ "args": "all", "varsIgnorePattern": "^_" }])),
        ),
        (
            "(function(_a) {})();",
            Some(serde_json::json!([{ "args": "all", "caughtErrorsIgnorePattern": "^_" }])),
        ),
        ("var a = function() { a(); };", None),
        ("var a = function(){ return function() { a(); } };", None),
        ("const a = () => () => { a(); };", None), // { "ecmaVersion": 2015 },
        (
            "let myArray = [1,2,3,4].filter((x) => x == 0);
			    myArray = myArray.filter((x) => x == 1);",
            None,
        ), // { "ecmaVersion": 2015 },
        ("const a = 1; a += 1;", None),            // { "ecmaVersion": 2015 },
        ("const a = () => { a(); };", None),       // { "ecmaVersion": 2015 },
        // TODO
        // (
        //     "let x = [];
        // 	x = x.concat(x);",
        //     None,
        // ), // { "ecmaVersion": 2015 },
        (
            "let a = 'a';
            a = 10;
            function foo(){
                a = 11;
                a = () => {
                    a = 13
                }
            }",
            None,
        ), // { "ecmaVersion": 2020 },
        (
            "let foo;
            init();
            foo = foo + 2;
            function init() {
                foo = 1;
            }",
            None,
        ), // { "ecmaVersion": 2020 },
        (
            "function foo(n) {
                if (n < 2) return 1;
                return n * foo(n - 1);
            }",
            None,
        ), // { "ecmaVersion": 2020 },
        (
            "let c = 'c'
			c = 10
			function foo1() {
			    c = 11
			    c = () => { c = 13 }
			}
			c = foo1",
            None,
        ), // { "ecmaVersion": 2020 },
        (
            "class Foo { static {} }",
            Some(serde_json::json!([{ "ignoreClassWithStaticInitBlock": false }])),
        ), // { "ecmaVersion": 2022 },
        ("class Foo { static {} }", None), // { "ecmaVersion": 2022 },
        (
            "class Foo { static { var bar; } }",
            Some(serde_json::json!([{ "ignoreClassWithStaticInitBlock": true }])),
        ), // { "ecmaVersion": 2022 },
        ("class Foo {}", Some(serde_json::json!([{ "ignoreClassWithStaticInitBlock": true }]))), // { "ecmaVersion": 2022 },
        (
            "class Foo { static bar; }",
            Some(serde_json::json!([{ "ignoreClassWithStaticInitBlock": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "class Foo { static bar() {} }",
            Some(serde_json::json!([{ "ignoreClassWithStaticInitBlock": true }])),
        ), // { "ecmaVersion": 2022 },
        (
            "const _a = 5;const _b = _a + 5",
            Some(
                serde_json::json!([{ "args": "all", "varsIgnorePattern": "^_", "reportUsedIgnorePattern": true }]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "const _a = 42; foo(() => _a);",
            Some(
                serde_json::json!([{ "args": "all", "varsIgnorePattern": "^_", "reportUsedIgnorePattern": true }]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "(function foo(_a) { return _a + 5 })(5)",
            Some(
                serde_json::json!([{ "args": "all", "argsIgnorePattern": "^_", "reportUsedIgnorePattern": true }]),
            ),
        ),
        // TODO
        (
            "const [ a, _b ] = items;
        	console.log(a+_b);",
            Some(
                serde_json::json!([{ "destructuredArrayIgnorePattern": "^_", "reportUsedIgnorePattern": true }]),
            ),
        ), // { "ecmaVersion": 6 },
        // (
        //     "let _x;
        // 	[_x] = arr;
        // 	foo(_x);",
        //     Some(
        //         serde_json::json!([{ "destructuredArrayIgnorePattern": "^_", "reportUsedIgnorePattern": true, "varsIgnorePattern": "[iI]gnored" }]),
        //     ),
        // ), // { "ecmaVersion": 6 },
        (
            "const [ignored] = arr;
			foo(ignored);",
            Some(
                serde_json::json!([{ "destructuredArrayIgnorePattern": "^_", "reportUsedIgnorePattern": true, "varsIgnorePattern": "[iI]gnored" }]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "try{}catch(_err){console.error(_err)}",
            Some(
                serde_json::json!([{ "caughtErrors": "all", "caughtErrorsIgnorePattern": "^_", "reportUsedIgnorePattern": true }]),
            ),
        ),
        (
            "try {} catch ({ message }) { console.error(message); }",
            Some(
                serde_json::json!([{ "caughtErrorsIgnorePattern": "message", "reportUsedIgnorePattern": true }]),
            ),
        ), // { "ecmaVersion": 2015 },
        (
            "try {} catch ([_a, _b]) { doSomething(_a, _b); }",
            Some(
                serde_json::json!([{ "caughtErrorsIgnorePattern": "^_", "reportUsedIgnorePattern": true }]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "try {} catch ([_a, _b]) { doSomething(_a, _b); }",
            Some(
                serde_json::json!([{ "destructuredArrayIgnorePattern": "^_", "reportUsedIgnorePattern": true }]),
            ),
        ), // { "ecmaVersion": 6 },
        (
            "
			try {
			} catch (_) {
			  _ = 'foo'
			}
			            ",
            Some(serde_json::json!([{ "caughtErrorsIgnorePattern": "foo" }])),
        ),
        (
            "
			try {
			} catch (_) {
			  _ = 'foo'
			}
			            ",
            Some(
                serde_json::json!([{ "caughtErrorsIgnorePattern": "ignored", "varsIgnorePattern": "_" }]),
            ),
        ),
        (
            "try {} catch ({ message, errors: [firstError] }) {}",
            Some(serde_json::json!([{ "caughtErrorsIgnorePattern": "foo" }])),
        ), // { "ecmaVersion": 2015 },
        (
            "try {} catch ({ stack: $ }) { $ = 'Something broke: ' + $; }",
            Some(serde_json::json!([{ "caughtErrorsIgnorePattern": "\\w" }])),
        ), // { "ecmaVersion": 2015 },
        (
            "
			_ => { _ = _ + 1 };
			            ",
            Some(serde_json::json!([{ "argsIgnorePattern": "ignored", "varsIgnorePattern": "_" }])),
        ), // { "ecmaVersion": 2015 }
    ];

    Tester::new(NoUnusedVars::NAME, pass, fail)
        .intentionally_allow_no_fix_tests()
        .with_snapshot_suffix("eslint")
        .test_and_snapshot();
}
