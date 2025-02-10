use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

use crate::{ast_util::IsConstant, context::LintContext, rule::Rule};

fn no_constant_condition_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected constant condition")
        .with_help("Constant expression as a test condition is not allowed")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub enum CheckLoops {
    All,
    None,
    #[default]
    AllExceptWhileTrue,
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
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// if (false) {
    ///    doSomethingUnfinished();
    /// }
    ///
    /// if (new Boolean(x)) {
    ///     doSomethingAlways();
    /// }
    /// if (x ||= true) {
    ///     doSomethingAlways();
    /// }
    ///
    /// do {
    ///     doSomethingForever();
    /// } while (x = -1);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// if (x === 0) {
    ///     doSomething();
    /// }
    ///
    /// while (typeof x === "undefined") {
    ///     doSomething();
    /// }
    /// ```
    NoConstantCondition,
    eslint,
    correctness
);

impl Rule for NoConstantCondition {
    fn from_configuration(value: serde_json::Value) -> Self {
        let raw_check_loops = value.get(0).and_then(|v| v.get("checkLoops"));

        let check_loops = raw_check_loops
            .and_then(|val| {
                serde_json::Value::as_bool(val)
                    .map(|val| if val { CheckLoops::All } else { CheckLoops::None })
                    .or_else(|| {
                        serde_json::Value::as_str(val).map(|val| match val {
                            "all" => CheckLoops::All,
                            "none" => CheckLoops::None,
                            _ => CheckLoops::AllExceptWhileTrue,
                        })
                    })
            })
            .unwrap_or_default();

        Self { check_loops }
    }

    fn run_once(&self, ctx: &LintContext) {
        let mut current_scope = Span::default();
        let mut scopes = Vec::<(Span, FxHashSet<Span>)>::new();
        for node in ctx.semantic().nodes() {
            let is_in_scope = node.span().start < current_scope.end;

            // Exit previous generator function scope and emit diagnostics
            if !is_in_scope {
                if let Some((prev_scope, spans)) = scopes.pop() {
                    current_scope = prev_scope;
                    for span in spans {
                        ctx.diagnostic(no_constant_condition_diagnostic(span));
                    }
                }
            }

            match node.kind() {
                AstKind::IfStatement(if_stmt) => {
                    if if_stmt.test.is_constant(true, ctx) {
                        ctx.diagnostic(no_constant_condition_diagnostic(if_stmt.test.span()));
                    }
                }
                AstKind::ConditionalExpression(condition_expr) => {
                    if condition_expr.test.is_constant(true, ctx) {
                        ctx.diagnostic(no_constant_condition_diagnostic(
                            condition_expr.test.span(),
                        ));
                    }
                }
                AstKind::WhileStatement(while_stmt) => {
                    if let CheckLoops::AllExceptWhileTrue = self.check_loops {
                        if let Expression::BooleanLiteral(val) = &while_stmt.test {
                            if val.value {
                                return;
                            }
                        }
                    }
                    self.check_loop(ctx, &while_stmt.test, &mut scopes, is_in_scope);
                }
                AstKind::DoWhileStatement(do_while_stmt) => {
                    self.check_loop(ctx, &do_while_stmt.test, &mut scopes, is_in_scope);
                }
                AstKind::ForStatement(for_stmt) => {
                    if let Some(expr) = &for_stmt.test {
                        self.check_loop(ctx, expr, &mut scopes, is_in_scope);
                    }
                }
                AstKind::Function(func) => {
                    if func.generator {
                        scopes.push((current_scope, FxHashSet::default()));
                        current_scope = func.span;
                    }
                }
                AstKind::YieldExpression(yield_expr) => {
                    if let Some((_, spans)) = scopes.last_mut() {
                        for node in ctx.nodes().ancestors(node.id()).skip(1) {
                            match node.kind() {
                                AstKind::Function(func) => {
                                    if func.generator {
                                        break;
                                    }
                                }
                                AstKind::WhileStatement(while_stmt) => {
                                    spans.remove(&while_stmt.test.span());
                                }
                                AstKind::DoWhileStatement(do_while_stmt) => {
                                    spans.remove(&do_while_stmt.test.span());
                                }
                                AstKind::ForStatement(for_stmt) => {
                                    if let Some(expr) = &for_stmt.test {
                                        let span = expr.span();
                                        if yield_expr.span().start > span.start {
                                            spans.remove(&span);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Emit remaining diagnostics
        for (_, spans) in scopes {
            for span in spans {
                ctx.diagnostic(no_constant_condition_diagnostic(span));
            }
        }
    }
}

impl NoConstantCondition {
    fn check_loop<'a>(
        &self,
        ctx: &LintContext<'a>,
        expr: &Expression<'a>,
        scopes: &mut [(Span, FxHashSet<Span>)],
        is_in_scope: bool,
    ) {
        if matches!(self.check_loops, CheckLoops::All | CheckLoops::AllExceptWhileTrue)
            && expr.is_constant(true, ctx)
        {
            if is_in_scope {
                if let Some((_, spans)) = scopes.last_mut() {
                    spans.insert(expr.span());
                    return;
                }
            }
            ctx.diagnostic(no_constant_condition_diagnostic(expr.span()));
        }
    }
}

#[test]
fn test() {
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
        ("while(~!a);", None),
        ("while(a = b);", None),
        ("while(`${a}`);", None),
        ("for(;x < 10;);", None),
        ("for(;;);", None),
        ("for(;`${a}`;);", None),
        ("do{ }while(x)", None),
        ("q > 0 ? 1 : 2;", None),
        ("`${a}` === a ? 1 : 2", None),
        ("`foo${a}` === a ? 1 : 2", None),
        ("tag`a` === a ? 1 : 2", None),
        ("tag`${a}` === a ? 1 : 2", None),
        ("while(x += 3) {}", None),
        ("while(tag`a`) {}", None),
        ("while(tag`${a}`) {}", None),
        ("while(`\\\n${a}`) {}", None),
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
        ("while(true);", Some(serde_json::json!([{ "checkLoops": false }]))),
        ("for(;true;);", Some(serde_json::json!([{ "checkLoops": false }]))),
        ("do{}while(true)", Some(serde_json::json!([{ "checkLoops": false }]))),
        ("while(true);", Some(serde_json::json!([{ "checkLoops": "none" }]))),
        ("for(;true;);", Some(serde_json::json!([{ "checkLoops": "none" }]))),
        ("do{}while(true)", Some(serde_json::json!([{ "checkLoops": "none" }]))),
        ("while(true);", Some(serde_json::json!([{ "checkLoops": "allExceptWhileTrue" }]))),
        ("while(true);", None),
        ("while(a == b);", Some(serde_json::json!([{ "checkLoops": "all" }]))),
        ("do{ }while(x);", Some(serde_json::json!([{ "checkLoops": "all" }]))),
        ("for (let x = 0; x <= 10; x++) {};", Some(serde_json::json!([{ "checkLoops": "all" }]))),
        ("function* foo(){while(true){yield 'foo';}}", None),
        ("function* foo(){for(;true;){yield 'foo';}}", None),
        ("function* foo(){do{yield 'foo';}while(true)}", None),
        ("function* foo(){while (true) { while(true) {yield;}}}", None),
        ("function* foo() {for (; yield; ) {}}", None),
        ("function* foo() {for (; ; yield) {}}", None),
        ("function* foo() {while (true) {function* foo() {yield;}yield;}}", None),
        ("function* foo() { for (let x = yield; x < 10; x++) {yield;}yield;}", None),
        ("function* foo() { for (let x = yield; ; x++) { yield; }}", None),
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
        ("function foo(Boolean) { if (Boolean(1)) {} }", None),
        ("const Boolean = () => {}; if (Boolean(1)) {}", None),
        ("const undefined = 'lol'; if (undefined) {}", None),
    ];

    let fail = vec![
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
        ("true ? 1 : 2;", None),
        ("1 ? 1 : 2;", None),
        ("q = 0 ? 1 : 2;", None),
        ("(q = 0) ? 1 : 2;", None),
        ("`` ? 1 : 2;", None),
        ("`foo` ? 1 : 2;", None),
        ("`foo${bar}` ? 1 : 2;", None),
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
        ("while([]);", None),
        ("while(~!0);", None),
        ("while(x = 1);", None),
        ("while(function(){});", None),
        ("while(true);", Some(serde_json::json!([{ "checkLoops": "all" }]))),
        ("while(1);", None),
        ("while(() => {});", None),
        ("while(`foo`);", None),
        ("while(``);", None),
        ("while(`${'foo'}`);", None),
        ("while(`${'foo' + 'bar'}`);", None),
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
        ("while(x = 1);", Some(serde_json::json!([{ "checkLoops": "all" }]))),
        ("do{ }while(x = 1)", Some(serde_json::json!([{ "checkLoops": "all" }]))),
        ("for (;true;) {};", Some(serde_json::json!([{ "checkLoops": "all" }]))),
        (
            "function* foo(){while(true){} yield 'foo';}",
            Some(serde_json::json!([{ "checkLoops": "all" }])),
        ),
        (
            "function* foo(){while(true){} yield 'foo';}",
            Some(serde_json::json!([{ "checkLoops": true }])),
        ),
        (
            "function* foo(){while(true){if (true) {yield 'foo';}}}",
            Some(serde_json::json!([{ "checkLoops": "all" }])),
        ),
        (
            "function* foo(){while(true){if (true) {yield 'foo';}}}",
            Some(serde_json::json!([{ "checkLoops": true }])),
        ),
        (
            "function* foo(){while(true){yield 'foo';} while(true) {}}",
            Some(serde_json::json!([{ "checkLoops": "all" }])),
        ),
        (
            "function* foo(){while(true){yield 'foo';} while(true) {}}",
            Some(serde_json::json!([{ "checkLoops": true }])),
        ),
        (
            "var a = function* foo(){while(true){} yield 'foo';}",
            Some(serde_json::json!([{ "checkLoops": "all" }])),
        ),
        (
            "var a = function* foo(){while(true){} yield 'foo';}",
            Some(serde_json::json!([{ "checkLoops": true }])),
        ),
        (
            "while (true) { function* foo() {yield;}}",
            Some(serde_json::json!([{ "checkLoops": "all" }])),
        ),
        (
            "while (true) { function* foo() {yield;}}",
            Some(serde_json::json!([{ "checkLoops": true }])),
        ),
        ("function* foo(){if (true) {yield 'foo';}}", None),
        ("function* foo() {for (let foo = yield; true;) {}}", None),
        ("function* foo() {for (foo = yield; true;) {}}", None),
        (
            "function foo() {while (true) {function* bar() {while (true) {yield;}}}}",
            Some(serde_json::json!([{ "checkLoops": "all" }])),
        ),
        (
            "function foo() {while (true) {const bar = function*() {while (true) {yield;}}}}",
            Some(serde_json::json!([{ "checkLoops": "all" }])),
        ),
        ("function* foo() { for (let foo = 1 + 2 + 3 + (yield); true; baz) {}}", None),
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
        ("if(class {}) {}", None),
        ("if(new Foo()) {}", None),
        ("if(new Boolean(foo)) {}", None),
        ("if(new String(foo)) {}", None),
        ("if(new Number(foo)) {}", None),
        ("if(`${[...['a']]}`) {}", None),
        ("if (undefined) {}", None),
        ("if (Boolean(1)) {}", None),
        ("if (Boolean()) {}", None),
        ("if (Boolean([a])) {}", None),
        ("if (Boolean(1)) { function Boolean() {}}", None),
    ];

    Tester::new(NoConstantCondition::NAME, NoConstantCondition::PLUGIN, pass, fail)
        .test_and_snapshot();

    let pass = vec![
        ("if (Boolean()) {}", None, Some(serde_json::json!({ "globals": { "Boolean": "off" } }))),
        ("if (undefined) {}", None, Some(serde_json::json!({ "globals": { "undefined": "off" } }))),
    ];
    let fail = vec![];

    Tester::new(NoConstantCondition::NAME, NoConstantCondition::PLUGIN, pass, fail).test();
}
