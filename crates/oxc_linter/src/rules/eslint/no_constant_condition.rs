use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use serde_json::Value;

use crate::{AstNode, ast_util::IsConstant, context::LintContext, rule::Rule};

fn no_constant_condition_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected constant condition")
        .with_help("Constant expression as a test condition is not allowed")
        .with_label(span)
}

#[derive(Debug, Default, Clone, PartialEq)]
enum CheckLoops {
    All,
    #[default]
    AllExceptWhileTrue,
    None,
}

impl CheckLoops {
    fn from(value: &Value) -> Option<Self> {
        match value {
            Value::String(str) => match str.as_str() {
                "all" => Some(Self::All),
                "allExceptWhileTrue" => Some(Self::AllExceptWhileTrue),
                "none" => Some(Self::None),
                _ => None,
            },
            Value::Bool(bool) => {
                if *bool {
                    Some(Self::All)
                } else {
                    Some(Self::None)
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct NoConstantCondition {
    check_loops: CheckLoops,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow constant expressions in conditions
    ///
    /// ### Why is this bad?
    ///
    /// A constant expression (for example, a literal) as a test condition might
    /// be a typo or development trigger for a specific behavior.
    ///
    /// This rule disallows constant expressions in the test condition of:
    ///
    /// - `if`, `for`, `while`, or `do...while` statement
    /// - `?`: ternary expression
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// if (false) {
    ///   doSomethingUnfinished();
    /// }
    ///
    /// if (new Boolean(x)) {
    ///   doSomethingAlways();
    /// }
    /// if (x ||= true) {
    ///   doSomethingAlways();
    /// }
    ///
    /// do {
    ///   doSomethingForever();
    /// } while (x = -1);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// if (x === 0) {
    ///   doSomething();
    /// }
    ///
    /// while (typeof x === "undefined") {
    ///   doSomething();
    /// }
    /// ```
    ///
    /// ### Options
    ///
    /// #### checkLoops
    ///
    /// `{ type: "all" | "allExceptWhileTrue" | "none" | boolean, default: "allExceptWhileTrue" }`
    ///
    /// - `"all"` or `true` disallows constant expressions in loops
    /// - `"allExceptWhileTrue"` disallows constant expressions in loops except while loops with expression `true`
    /// - `"none"` or `false` allows constant expressions in loops
    NoConstantCondition,
    eslint,
    correctness
);

impl Rule for NoConstantCondition {
    fn from_configuration(value: Value) -> Self {
        let obj = value.get(0);

        Self {
            check_loops: obj
                .and_then(|v| v.get("checkLoops"))
                .and_then(CheckLoops::from)
                .unwrap_or_default(),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::IfStatement(if_stmt) => check(ctx, &if_stmt.test),
            AstKind::ConditionalExpression(condition_expr) => check(ctx, &condition_expr.test),
            AstKind::WhileStatement(while_stmt) => self.check_loop(ctx, &while_stmt.test, true),
            AstKind::DoWhileStatement(do_while_stmt) => {
                self.check_loop(ctx, &do_while_stmt.test, false);
            }
            AstKind::ForStatement(for_stmt) => {
                let Some(test) = &for_stmt.test else {
                    return;
                };

                self.check_loop(ctx, test, false);
            }
            _ => {}
        }
    }
}

impl NoConstantCondition {
    fn check_loop<'a>(&self, ctx: &LintContext<'a>, test: &'a Expression<'_>, is_while: bool) {
        match self.check_loops {
            CheckLoops::None => return,
            CheckLoops::AllExceptWhileTrue if is_while => match test {
                Expression::BooleanLiteral(bool) if bool.value => return,
                _ => {}
            },
            _ => {}
        }

        check(ctx, test);
    }
}

fn check<'a>(ctx: &LintContext<'a>, test: &'a Expression<'_>) {
    if test.is_constant(true, ctx) {
        ctx.diagnostic(no_constant_condition_diagnostic(test.span()));
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        ("if(a);", None),
        ("if(a == 0);", None),
        ("if(a = f());", None),
        ("if(a += 1);", None),
        ("if(a |= 1);", None),
        ("if(a |= true);", None),
        ("if(a |= false);", None),
        ("if(a &= 1);", None),
        ("if(a &= true);", None),
        ("if(a &= false);", None),
        ("if(a >>= 1);", None),
        ("if(a >>= true);", None),
        ("if(a >>= false);", None),
        ("if(a >>>= 1);", None),
        ("if(a ??= 1);", None),
        ("if(a ??= true);", None),
        ("if(a ??= false);", None),
        ("if(a ||= b);", None),
        ("if(a ||= false);", None),
        ("if(a ||= 0);", None),
        ("if(a ||= void 0);", None),
        ("if(+(a ||= 1));", None),
        ("if(f(a ||= true));", None),
        ("if((a ||= 1) + 2);", None),
        ("if(1 + (a ||= true));", None),
        ("if(a ||= '' || false);", None),
        ("if(a ||= void 0 || null);", None),
        ("if((a ||= false) || b);", None),
        ("if(a || (b ||= false));", None),
        ("if((a ||= true) && b);", None),
        ("if(a && (b ||= true));", None),
        ("if(a &&= b);", None),
        ("if(a &&= true);", None),
        ("if(a &&= 1);", None),
        ("if(a &&= 'foo');", None),
        ("if((a &&= '') + false);", None),
        ("if('' + (a &&= null));", None),
        ("if(a &&= 1 && 2);", None),
        ("if((a &&= true) && b);", None),
        ("if(a && (b &&= true));", None),
        ("if((a &&= false) || b);", None),
        ("if(a || (b &&= false));", None),
        ("if(a ||= b ||= false);", None),
        ("if(a &&= b &&= true);", None),
        ("if(a ||= b &&= false);", None),
        ("if(a ||= b &&= true);", None),
        ("if(a &&= b ||= false);", None),
        ("if(a &&= b ||= true);", None),
        ("if(1, a);", None),
        ("if ('every' in []);", None),
        ("if (`\\\n${a}`) {}", None),
        ("if (`${a}`);", None),
        ("if (`${foo()}`);", None),
        ("if (`${a === 'b' && b==='a'}`);", None),
        ("if (`foo${a}` === 'fooa');", None),
        ("if (tag`a`);", None),
        ("if (tag`${a}`);", None),
        ("if (+(a || true));", None),
        ("if (-(a || true));", None),
        ("if (~(a || 1));", None),
        ("if (+(a && 0) === +(b && 0));", None),
        ("if(typeof x === 'undefined'){}", None),
        ("if(`${typeof x}` === 'undefined'){}", None),
        ("if(a === 'str' && typeof b){}", None),
        ("typeof a == typeof b", None),
        ("typeof 'a' === 'string'|| typeof b === 'string'", None),
        ("`${typeof 'a'}` === 'string'|| `${typeof b}` === 'string'", None),
        ("if (void a || a);", None),
        ("if (a || void a);", None),
        ("if(xyz === 'str1' && abc==='str2'){}", None),
        ("if(xyz === 'str1' || abc==='str2'){}", None),
        ("if(xyz === 'str1' || abc==='str2' && pqr === 5){}", None),
        ("if(typeof abc === 'string' && abc==='str2'){}", None),
        ("if(false || abc==='str'){}", None),
        ("if(true && abc==='str'){}", None),
        ("if(typeof 'str' && abc==='str'){}", None),
        ("if(abc==='str' || false || def ==='str'){}", None),
        ("if(true && abc==='str' || def ==='str'){}", None),
        ("if(true && typeof abc==='string'){}", None),
        ("if('str1' && a){}", None),
        ("if(a && 'str'){}", None),
        ("if ((foo || true) === 'baz') {}", None),
        ("if ((foo || 'bar') === 'baz') {}", None),
        ("if ((foo || 'bar') !== 'baz') {}", None),
        ("if ((foo || 'bar') == 'baz') {}", None),
        ("if ((foo || 'bar') != 'baz') {}", None),
        ("if ((foo || 233) > 666) {}", None),
        ("if ((foo || 233) < 666) {}", None),
        ("if ((foo || 233) >= 666) {}", None),
        ("if ((foo || 233) <= 666) {}", None),
        ("if ((key || 'k') in obj) {}", None),
        ("if ((foo || {}) instanceof obj) {}", None),
        ("if ((foo || 'bar' || 'bar') === 'bar');", None),
        ("if ((foo || 1n) === 'baz') {}", None),
        ("if (a && 0n || b);", None),
        ("if(1n && a){};", None),
        ("if ('' + [y] === '' + [ty]) {}", None),
        ("if ('a' === '' + [ty]) {}", None),
        ("if ('' + [y, m, d] === 'a') {}", None),
        ("if ('' + [y, 'm'] === '' + [ty, 'tm']) {}", None),
        ("if ('' + [y, 'm'] === '' + ['ty']) {}", None),
        ("if ([,] in\n\n($2))\n ;\nelse\n ;", None),
        ("if ([...x]+'' === 'y'){}", None),
        ("if (new Number(x) + 1 === 2) {}", None),
        ("if([a]==[b]) {}", None),
        ("if (+[...a]) {}", None),
        ("if (+[...[...a]]) {}", None),
        ("if (`${[...a]}`) {}", None),
        ("if (`${[a]}`) {}", None),
        ("if (+[a]) {}", None),
        ("if (0 - [a]) {}", None),
        ("if (1 * [a]) {}", None),
        ("if (Boolean(a)) {}", None),
        ("if (Boolean(...args)) {}", None),
        ("if (foo.Boolean(1)) {}", None),
        // TODO
        // ("const undefined = 'lol'; if (undefined) {}", None),
        // ("function foo(Boolean) { if (Boolean(1)) {} }", None),
        // ("const Boolean = () => {}; if (Boolean(1)) {}", None),
        // ("if (Boolean()) {}", None),
        // ("if (undefined) {}", None),
        ("q > 0 ? 1 : 2;", None),
        ("`${a}` === a ? 1 : 2", None),
        ("`foo${a}` === a ? 1 : 2", None),
        ("tag`a` === a ? 1 : 2", None),
        ("tag`${a}` === a ? 1 : 2", None),
        ("while(~!a);", None),
        ("while(a = b);", None),
        ("while(`${a}`);", None),
        ("for(;x < 10;);", None),
        ("for(;;);", None),
        ("for(;`${a}`;);", None),
        ("do{ }while(x)", None),
        ("while(x += 3) {}", None),
        ("while(tag`a`) {}", None),
        ("while(tag`${a}`) {}", None),
        ("while(`\\\n${a}`) {}", None),
        ("while(true);", Some(json!([{ "checkLoops": false }]))),
        ("for(;true;);", Some(json!([{ "checkLoops": false }]))),
        ("do{}while(true)", Some(json!([{ "checkLoops": "none" }]))),
        ("while(true);", Some(json!([{ "checkLoops": "none" }]))),
        ("for(;true;);", Some(json!([{ "checkLoops": "none" }]))),
        ("do{ }while(x);", Some(json!([{ "checkLoops": "all" }]))),
        ("while(true);", Some(json!([{ "checkLoops": "allExceptWhileTrue" }]))),
        ("while(true);", None),
        ("while(a == b);", Some(json!([{ "checkLoops": "all" }]))),
        ("for (let x = 0; x <= 10; x++) {};", Some(json!([{ "checkLoops": "all" }]))),
        ("do{}while(true)", Some(json!([{ "checkLoops": false }]))),
        // TODO
        // ("function* foo(){while(true){yield 'foo';}}", None),
        // ("function* foo(){for(;true;){yield 'foo';}}", None),
        // ("function* foo(){do{yield 'foo';}while(true)}", None),
        // ("function* foo(){while (true) { while(true) {yield;}}}", None),
        // ("function* foo() {for (; yield; ) {}}", None),
        // ("function* foo() {for (; ; yield) {}}", None),
        // ("function* foo() {while (true) {function* foo() {yield;}yield;}}", None),
        // ("function* foo() { for (let x = yield; x < 10; x++) {yield;}yield;}", None),
        // ("function* foo() { for (let x = yield; ; x++) { yield; }}", None),
    ];

    let fail = vec![
        ("if(-2);", None),
        ("if(true);", None),
        ("if(1);", None),
        ("if({});", None),
        ("if(0 < 1);", None),
        ("if(0 || 1);", None),
        ("if(a, 1);", None),
        ("if(`foo`);", None),
        ("if(``);", None),
        ("if(`\\\n`);", None),
        ("if(`${'bar'}`);", None),
        ("if(`${'bar' + `foo`}`);", None),
        ("if(`foo${false || true}`);", None),
        ("if(`foo${0 || 1}`);", None),
        ("if(`foo${bar}`);", None),
        ("if(`${bar}foo`);", None),
        ("if(!(true || a));", None),
        ("if(!(a && void b && c));", None),
        ("if(0 || !(a && null));", None),
        ("if(1 + !(a || true));", None),
        ("if(!(null && a) > 1);", None),
        ("if(+(!(a && 0)));", None),
        ("if(!typeof a === 'string');", None),
        ("if(-('foo' || a));", None),
        ("if(+(void a && b) === ~(1 || c));", None),
        ("if(a ||= true);", None),
        ("if(a ||= 5);", None),
        ("if(a ||= 'foo' || b);", None),
        ("if(a ||= b || /regex/);", None),
        ("if(a ||= b ||= true);", None),
        ("if(a ||= b ||= c || 1);", None),
        ("if(!(a ||= true));", None),
        ("if(!(a ||= 'foo') === true);", None),
        ("if(!(a ||= 'foo') === false);", None),
        ("if(a || (b ||= true));", None),
        ("if((a ||= 1) || b);", None),
        ("if((a ||= true) && true);", None),
        ("if(true && (a ||= true));", None),
        ("if(a &&= false);", None),
        ("if(a &&= null);", None),
        ("if(a &&= void b);", None),
        ("if(a &&= 0 && b);", None),
        ("if(a &&= b && '');", None),
        ("if(a &&= b &&= false);", None),
        ("if(a &&= b &&= c && false);", None),
        ("if(!(a &&= false));", None),
        ("if(!(a &&= 0) + 1);", None),
        ("if(a && (b &&= false));", None),
        ("if((a &&= null) && b);", None),
        ("if(false || (a &&= false));", None),
        ("if((a &&= false) || false);", None),
        ("while(x = 1);", None),
        ("if(typeof x){}", None),
        ("if(typeof 'abc' === 'string'){}", None),
        ("if(a = typeof b){}", None),
        ("if(a, typeof b){}", None),
        ("if(typeof 'a' == 'string' || typeof 'b' == 'string'){}", None),
        ("while(typeof x){}", None),
        ("if(1 || void x);", None),
        ("if(void x);", None),
        ("if(y = void x);", None),
        ("if(x, void x);", None),
        ("if(void x === void y);", None),
        ("if(void x && a);", None),
        ("if(a && void x);", None),
        ("if(false && abc==='str'){}", None),
        ("if(true || abc==='str'){}", None),
        ("if(1 || abc==='str'){}", None),
        ("if(abc==='str' || true){}", None),
        ("if(abc==='str' || true || def ==='str'){}", None),
        ("if(false || true){}", None),
        ("if(typeof abc==='str' || true){}", None),
        ("if('str' || a){}", None),
        ("if('str' || abc==='str'){}", None),
        ("if('str1' || 'str2'){}", None),
        ("if('str1' && 'str2'){}", None),
        ("if(abc==='str' || 'str'){}", None),
        ("if(a || 'str'){}", None),
        ("if([a]) {}", None),
        ("if([]) {}", None),
        ("if(''+['a']) {}", None),
        ("if(''+[]) {}", None),
        ("if(+1) {}", None),
        ("if ([,] + ''){}", None),
        ("if(/foo/ui);", None),
        ("if(0n);", None),
        ("if(0b0n);", None),
        ("if(0o0n);", None),
        ("if(0x0n);", None),
        ("if(0b1n);", None),
        ("if(0o1n);", None),
        ("if(0x1n);", None),
        ("if(0x1n || foo);", None),
        // Classes and instances are always truthy
        ("if(class {}) {}", None),
        ("if(new Foo()) {}", None),
        // Boxed primitives are always truthy
        ("if(new Boolean(foo)) {}", None),
        ("if(new String(foo)) {}", None),
        ("if(new Number(foo)) {}", None),
        // Spreading a constant array
        ("if(`${[...['a']]}`) {}", None),
        // undefined is always falsy (except in old browsers that let you
        // re-assign, but that's an obscure enough edge case to not worry about)
        ("if (undefined) {}", None),
        // Coercion to boolean via Boolean function
        ("if (Boolean(1)) {}", None),
        ("if (Boolean()) {}", None),
        ("if (Boolean([a])) {}", None),
        ("if (Boolean(1)) { function Boolean() {}}", None),
        ("true ? 1 : 2;", None),
        ("1 ? 1 : 2;", None),
        ("q = 0 ? 1 : 2;", None),
        ("(q = 0) ? 1 : 2;", None),
        ("`` ? 1 : 2;", None),
        ("`foo` ? 1 : 2;", None),
        ("`foo${bar}` ? 1 : 2;", None),
        ("for(;true;);", None),
        ("for(;``;);", None),
        ("for(;`foo`;);", None),
        ("for(;`foo${bar}`;);", None),
        ("do{}while(true)", None),
        ("do{}while('1')", None),
        ("do{}while(0)", None),
        ("do{}while(t = -2)", None),
        ("do{}while(``)", None),
        ("do{}while(`foo`)", None),
        ("do{}while(`foo${bar}`)", None),
        ("while([]);", None),
        ("while(~!0);", None),
        ("while(x = 1);", Some(json!([{ "checkLoops": "all" }]))),
        ("while(function(){});", None),
        ("while(true);", Some(json!([{ "checkLoops": "all" }]))),
        ("while(1);", None),
        ("while(() => {});", None),
        ("while(`foo`);", None),
        ("while(``);", None),
        ("while(`${'foo'}`);", None),
        ("while(`${'foo' + 'bar'}`);", None),
        ("do{ }while(x = 1)", Some(json!([{ "checkLoops": "all" }]))),
        ("for (;true;) {};", Some(json!([{ "checkLoops": "all" }]))),
        // TODO
        // ("function* foo(){while(true){} yield 'foo';}", Some(json!([{ "checkLoops": "all" }]))),
        // ("function* foo(){while(true){} yield 'foo';}", Some(json!([{ "checkLoops": true }]))),
        // ("function* foo(){while(true){if (true) {yield 'foo';}}}",Some(json!([{ "checkLoops": "all" }])),),
        // ("function* foo(){while(true){if (true) {yield 'foo';}}}",Some(json!([{ "checkLoops": true }])),),
        // ("function* foo(){while(true){yield 'foo';} while(true) {}}", Some(json!([{ "checkLoops": "all" }])),),
        // ("function* foo(){while(true){yield 'foo';} while(true) {}}",Some(json!([{ "checkLoops": true }])),),
        // ("var a = function* foo(){while(true){} yield 'foo';}",Some(json!([{ "checkLoops": "all" }])),),
        // ("var a = function* foo(){while(true){} yield 'foo';}",Some(json!([{ "checkLoops": true }])),),
        // ("while (true) { function* foo() {yield;}}", Some(json!([{ "checkLoops": "all" }]))),
        // ("while (true) { function* foo() {yield;}}", Some(json!([{ "checkLoops": true }]))),
        // ("function* foo(){if (true) {yield 'foo';}}", None),
        // ("function* foo() {for (let foo = yield; true;) {}}", None),
        // ("function* foo() {for (foo = yield; true;) {}}", None),
        // ("function foo() {while (true) {function* bar() {while (true) {yield;}}}}",Some(json!([{ "checkLoops": "all" }])),),
        // ("function foo() {while (true) {const bar = function*() {while (true) {yield;}}}}",Some(json!([{ "checkLoops": "all" }])),),
        // ("function* foo() { for (let foo = 1 + 2 + 3 + (yield); true; baz) {}}", None),
    ];

    Tester::new(NoConstantCondition::NAME, NoConstantCondition::PLUGIN, pass, fail)
        .test_and_snapshot();
}
