use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

mod analysis;
mod cfg_segmenter;
mod rwu_table;

fn no_useless_assignment_diagnostic(span: Span, name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Value assigned to '{name}' is never read"))
        .with_help("Remove the assignment or use the value before reassigning")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessAssignment;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    NoUselessAssignment,
    eslint,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    pending, // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for NoUselessAssignment {
    fn run_once(&self, ctx: &LintContext) {
        let cfg = ctx.cfg();
        run_cfg_based_analysis(cfg, ctx);
    }
}

/// Run CFG-based liveness analysis on the entire program
fn run_cfg_based_analysis(cfg: &oxc_cfg::ControlFlowGraph, ctx: &LintContext<'_>) {
    // Segment the CFG by function
    let segments = cfg_segmenter::segment_cfg(cfg, ctx.semantic().scoping(), ctx.nodes());

    // Run liveness analysis on each segment
    let mut useless_writes = Vec::new();

    for (seg_idx, segment) in segments.iter().enumerate() {
        let mut liveness =
            analysis::Liveness::new(cfg, segment, ctx.semantic().scoping(), ctx.nodes());

        // Compute liveness
        liveness.compute();

        // Collect useless assignments from this segment
        let assignments = liveness.all_useless_assignments();
        #[cfg(debug_assertions)]
        eprintln!("Segment {}: found {} useless assignments", seg_idx, assignments.len());
        for (symbol_id, node_id) in assignments {
            useless_writes.push((symbol_id, node_id));
        }
    }

    let scoping = ctx.semantic().scoping();

    // Report diagnostics for useless assignments
    for (symbol_id, node_id) in useless_writes {
        let name = scoping.symbol_name(symbol_id);

        // Skip if name starts with underscore
        if name.starts_with('_') {
            continue;
        }

        let flags = scoping.symbol_flags(symbol_id);

        // Skip function declarations and class declarations
        if flags.is_function() || flags.is_class() {
            continue;
        }

        // Skip catch clause parameters
        if flags.is_catch_variable() {
            continue;
        }

        // Skip module-level variables (they might be used externally)
        // Module-level code is often used in ways the linter can't see (e.g., HTML script tags)
        let symbol_scope = scoping.symbol_scope_id(symbol_id);
        let is_module_level = symbol_scope == scoping.root_scope_id();
        if is_module_level {
            continue;
        }

        let node = ctx.nodes().get_node(node_id);
        ctx.diagnostic(no_useless_assignment_diagnostic(node.span(), name));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "let v = 'used';
			        console.log(v);
			        v = 'used-2'
			        console.log(v);",
        "function foo() {
			            let v = 'used';
			            console.log(v);
			            v = 'used-2';
			            console.log(v);
			        }",
        "function foo() {
			            let v = 'used';
			            if (condition) {
			                v = 'used-2';
			                console.log(v);
			                return
			            }
			            console.log(v);
			        }",
        "function foo() {
			            let v = 'used';
			            if (condition) {
			                console.log(v);
			            } else {
			                v = 'used-2';
			                console.log(v);
			            }
			        }",
        "function foo() {
			            let v = 'used';
			            if (condition) {
			                //
			            } else {
			                v = 'used-2';
			            }
			            console.log(v);
			        }",
        "var foo = function () {
			            let v = 'used';
			            console.log(v);
			            v = 'used-2'
			            console.log(v);
			        }",
        "var foo = () => {
			            let v = 'used';
			            console.log(v);
			            v = 'used-2'
			            console.log(v);
			        }",
        "class foo {
			            static {
			                let v = 'used';
			                console.log(v);
			                v = 'used-2'
			                console.log(v);
			            }
			        }",
        "function foo () {
			            let v = 'used';
			            for (let i = 0; i < 10; i++) {
			                console.log(v);
			                v = 'used in next iteration';
			            }
			        }",
        "function foo () {
			            let i = 0;
			            i++;
			            i++;
			            console.log(i);
			        }",
        "export let foo = 'used';
			        console.log(foo);
			        foo = 'unused like but exported';",
        "export function foo () {};
			        console.log(foo);
			        foo = 'unused like but exported';",
        "export class foo {};
			        console.log(foo);
			        foo = 'unused like but exported';",
        "export default function foo () {};
			        console.log(foo);
			        foo = 'unused like but exported';",
        "export default class foo {};
			        console.log(foo);
			        foo = 'unused like but exported';",
        "let foo = 'used';
			        export { foo };
			        console.log(foo);
			        foo = 'unused like but exported';",
        "function foo () {};
			        export { foo };
			        console.log(foo);
			        foo = 'unused like but exported';",
        "class foo {};
			        export { foo };
			        console.log(foo);
			        foo = 'unused like but exported';",
        "/* exported foo */
			            let foo = 'used';
			            console.log(foo);
			            foo = 'unused like but exported with directive';", // { "sourceType": "script" },
        "/*eslint test/use-a:1*/
			        let a = 'used';
			        console.log(a);
			        a = 'unused like but marked by markVariableAsUsed()';
			        ",
        "v = 'used';
			        console.log(v);
			        v = 'unused'",
        "let v = 'used variable';",
        "function foo() {
			            return;

			            const x = 1;
			            if (y) {
			                bar(x);
			            }
			        }",
        "function foo() {
			            const x = 1;
			            console.log(x);
			            return;

			            x = 'Foo'
			        }",
        "function foo() {
			            let a = 42;
			            console.log(a);
			            a++;
			            console.log(a);
			        }",
        "function foo() {
			            let a = 42;
			            console.log(a);
			            a--;
			            console.log(a);
			        }",
        "function foo() {
			            let a = 42;
			            console.log(a);
			            a = 10;
			            a = a + 1;
			            console.log(a);
			        }",
        "function foo() {
			            let a = 42;
			            console.log(a);
			            a = 10;
			            if (cond) {
			                a = a + 1;
			            } else {
			                a = 2 + a;
			            }
			            console.log(a);
			        }",
        "function foo() {
			            let a = 'used', b = 'used', c = 'used', d = 'used';
			            console.log(a, b, c, d);
			            ({ a, arr: [b, c, ...d] } = fn());
			            console.log(a, b, c, d);
			        }",
        "function foo() {
			            let a = 'used', b = 'used', c = 'used';
			            console.log(a, b, c);
			            ({ a = 'unused', foo: b, ...c } = fn());
			            console.log(a, b, c);
			        }",
        "function foo() {
			            let a = {};
			            console.log(a);
			            a.b = 'unused like, but maybe used in setter';
			        }",
        "function foo() {
			            let a = { b: 42 };
			            console.log(a);
			            a.b++;
			        }",
        "function foo () {
			            let v = 'used';
			            console.log(v);
			            function bar() {
			                v = 'used in outer scope';
			            }
			            bar();
			            console.log(v);
			        }",
        "function foo () {
			            let v = 'used';
			            console.log(v);
			            setTimeout(() => console.log(v), 1);
			            v = 'used in other scope';
			        }",
        "function foo () {
			            let v = 'used';
			            console.log(v);
			            for (let i = 0; i < 10; i++) {
			                if (condition) {
			                    v = 'maybe used';
			                    continue;
			                }
			                console.log(v);
			            }
			        }",
        "/* globals foo */
			        const bk = foo;
			        foo = 42;
			        try {
			            // process
			        } finally {
			            foo = bk;
			        }",
        "
			            const bk = console;
			            console = { log () {} };
			            try {
			                // process
			            } finally {
			                console = bk;
			            }", // {				"globals": { "console": false },			},
        "let message = 'init';
			        try {
			            const result = call();
			            message = result.message;
			        } catch (e) {
			            // ignore
			        }
			        console.log(message)",
        "let message = 'init';
			        try {
			            message = call().message;
			        } catch (e) {
			            // ignore
			        }
			        console.log(message)",
        "let v = 'init';
			        try {
			            v = callA();
			            try {
			                v = callB();
			            } catch (e) {
			                // ignore
			            }
			        } catch (e) {
			            // ignore
			        }
			        console.log(v)",
        "let v = 'init';
			        try {
			            try {
			                v = callA();
			            } catch (e) {
			                // ignore
			            }
			        } catch (e) {
			            // ignore
			        }
			        console.log(v)",
        "let a;
			        try {
			            foo();
			        } finally {
			            a = 5;
			        }
			        console.log(a);",
        "const obj = { a: 5 };
			        const { a, b = a } = obj;
			        console.log(b); // 5",
        "const arr = [6];
			        const [c, d = c] = arr;
			        console.log(d); // 6",
        "const obj = { a: 1 };
			        let {
			            a,
			            b = (a = 2)
			        } = obj;
			        console.log(a, b);",
        "let { a, b: {c = a} = {} } = obj;
			        console.log(c);",
        "function foo(){
			            let bar;
			            try {
			                bar = 2;
			                unsafeFn();
			                return { error: undefined };
			            } catch {
			                return { bar };
			            }
			        }
			        function unsafeFn() {
			            throw new Error();
			        }",
        "function foo(){
			            let bar, baz;
			            try {
			                bar = 2;
			                unsafeFn();
			                return { error: undefined };
			            } catch {
			               baz = bar;
			            }
			            return baz;
			        }
			        function unsafeFn() {
			            throw new Error();
			        }",
        "function foo(){
			            let bar;
			            try {
			                bar = 2;
			                unsafeFn();
			                bar = 4;
			            } catch {
			               // handle error
			            }
			            return bar;
			        }
			        function unsafeFn() {
			            throw new Error();
			        }",
        // r#"/*eslint test/unknown-ref:1*/
        // 	        let a = "used";
        // 			console.log(a);
        // 			a = "unused";"#,
        // r#"/*eslint test/unknown-ref:1*/
        // 			function foo() {
        // 				let a = "used";
        // 				console.log(a);
        // 				a = "unused";
        // 			}"#,
        // r#"/*eslint test/unknown-ref:1*/
        // 			function foo() {
        // 				let a = "used";
        // 				if (condition) {
        // 					a = "unused";
        // 					return
        // 				}
        // 				console.log(a);
        // 	        }"#,
        r#"
			                function App() {
			                    const A = "";
			                    return <A/>;
			                }
			            "#, // {				"parserOptions": {					"ecmaFeatures": {						"jsx": true,					},				},			},
        r#"
			                function App() {
			                    let A = "";
			                    foo(A);
			                    A = "A";
			                    return <A/>;
			                }
			            "#, // {				"parserOptions": {					"ecmaFeatures": {						"jsx": true,					},				},			},
        r#"
			                function App() {
								let A = "a";
			                    foo(A);
			                    return <A/>;
			                }
			            "#, // {				"parserOptions": {					"ecmaFeatures": {						"jsx": true,					},				},			},
        "function App() {
							let x = 0;
							foo(x);
							x = 1;
							return <A prop={x} />;
						}", // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			},
        r#"function App() {
							let x = "init";
							foo(x);
							x = "used";
							return <A>{x}</A>;
						}"#, // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			},
        "function App() {
							let props = { a: 1 };
							foo(props);
							props = { b: 2 };
							return <A {...props} />;
						}", // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			},
        "function App() {
							let NS = Lib;
							return <NS.Cmp />;
						}", // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			},
        "function App() {
							let a = 0;
							a++;
							return <A prop={a} />;
						}", // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			},
        "function App() {
							const obj = { a: 1 };
							const { a, b = a } = obj;
							return <A prop={b} />;
						}", // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			},
        // "function App() {
        // 					let { a, b: { c = a } = {} } = obj;
        // 					return <A prop={c} />;
        // 				}", // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			},
        r#"function App() {
							let x = "init";
							if (cond) {
								x = "used";
								return <A prop={x} />;
							}
							return <A prop={x} />;
						}"#, // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			},
        "function App() {
							let A;
							if (cond) {
							  A = Foo;
							} else {
							  A = Bar;
							}
							return <A />;
						}", // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			},
        "function App() {
							let m;
							try {
							  m = 2;
							  unsafeFn();
							  m = 4;
							} catch (e) {
							  // ignore
							}
							return <A prop={m} />;
						}", // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			},
        "function App() {
							const arr = [6];
							const [c, d = c] = arr;
							return <A prop={d} />;
						}", // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			},
        "function App() {
							const obj = { a: 1 };
							let {
							  a,
							  b = (a = 2)
							} = obj;
							return <A prop={a} />;
						}", // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			}
    ];

    let fail = vec![
        "let v = 'used';
			            console.log(v);
			            v = 'unused'",
        "function foo() {
			                let v = 'used';
			                console.log(v);
			                v = 'unused';
			            }",
        "function foo() {
			                let v = 'used';
			                if (condition) {
			                    v = 'unused';
			                    return
			                }
			                console.log(v);
			            }",
        "function foo() {
			                let v = 'used';
			                if (condition) {
			                    console.log(v);
			                } else {
			                    v = 'unused';
			                }
			            }",
        "var foo = function () {
			                let v = 'used';
			                console.log(v);
			                v = 'unused'
			            }",
        "var foo = () => {
			                let v = 'used';
			                console.log(v);
			                v = 'unused'
			            }",
        "class foo {
			                static {
			                    let v = 'used';
			                    console.log(v);
			                    v = 'unused'
			                }
			            }",
        "function foo() {
			                let v = 'unused';
			                if (condition) {
			                    v = 'used';
			                    console.log(v);
			                    return
			                }
			            }",
        "function foo() {
			                let v = 'used';
			                console.log(v);
			                v = 'unused';
			                v = 'unused';
			            }",
        "function foo() {
			                let v = 'used';
			                console.log(v);
			                v = 'unused';
			                v = 'used';
			                console.log(v);
			                v = 'used';
			                console.log(v);
			            }",
        "
			            let v;
			            v = 'unused';
			            if (foo) {
			                v = 'used';
			            } else {
			                v = 'used';
			            }
			            console.log(v);",
        "function foo() {
			                let v = 'used';
			                console.log(v);
			                v = 'unused';
			                v = 'unused';
			                v = 'used';
			                console.log(v);
			            }",
        "function foo() {
			                let v = 'unused';
			                if (condition) {
			                    if (condition2) {
			                        v = 'used-2';
			                    } else {
			                        v = 'used-3';
			                    }
			                } else {
			                    v = 'used-4';
			                }
			                console.log(v);
			            }",
        "function foo() {
			                let v;
			                if (condition) {
			                    v = 'unused';
			                } else {
			                    //
			                }
			                if (condition2) {
			                    v = 'used-1';
			                } else {
			                    v = 'used-2';
			                }
			                console.log(v);
			            }",
        "function foo() {
			                let v = 'used';
			                if (condition) {
			                    v = 'unused';
			                    v = 'unused';
			                    v = 'used';
			                }
			                console.log(v);
			            }",
        "function foo() {
			                let a = 42;
			                console.log(a);
			                a++;
			            }",
        "function foo() {
			                let a = 42;
			                console.log(a);
			                a--;
			            }",
        "function foo() {
			                let a = 'used', b = 'used', c = 'used', d = 'used';
			                console.log(a, b, c, d);
			                ({ a, arr: [b, c,, ...d] } = fn());
			                console.log(c);
			            }",
        "function foo() {
			                let a = 'used', b = 'used', c = 'used';
			                console.log(a, b, c);
			                ({ a = 'unused', foo: b, ...c } = fn());
			            }",
        "function foo () {
			                let v = 'used';
			                console.log(v);
			                setTimeout(() => v = 42, 1);
			                v = 'unused and variable is only updated in other scopes';
			            }",
        "function foo() {
			                let v = 'used';
			                if (condition) {
			                    let v = 'used';
			                    console.log(v);
			                    v = 'unused';
			                }
			                console.log(v);
			                v = 'unused';
			            }",
        "function foo() {
			                let v = 'used';
			                if (condition) {
			                    console.log(v);
			                    v = 'unused';
			                } else {
			                    v = 'unused';
			                }
			            }",
        "function foo () {
			                let v = 'used';
			                console.log(v);
			                v = 'unused';
			                return;
			                console.log(v);
			            }",
        "function foo () {
			                let v = 'used';
			                console.log(v);
			                v = 'unused';
			                throw new Error();
			                console.log(v);
			            }",
        "function foo () {
			                let v = 'used';
			                console.log(v);
			                for (let i = 0; i < 10; i++) {
			                    v = 'unused';
			                    continue;
			                    console.log(v);
			                }
			            }
			            function bar () {
			                let v = 'used';
			                console.log(v);
			                for (let i = 0; i < 10; i++) {
			                    v = 'unused';
			                    break;
			                    console.log(v);
			                }
			            }",
        "function foo () {
			                let v = 'used';
			                console.log(v);
			                for (let i = 0; i < 10; i++) {
			                    if (condition) {
			                        v = 'unused';
			                        break;
			                    }
			                    console.log(v);
			                }
			            }",
        "let message = 'unused';
			            try {
			                const result = call();
			                message = result.message;
			            } catch (e) {
			                message = 'used';
			            }
			            console.log(message)",
        "let message = 'unused';
			            try {
			                message = 'used';
			                console.log(message)
			            } catch (e) {
			            }",
        "let message = 'unused';
			            try {
			                message = call();
			            } catch (e) {
			                message = 'used';
			            }
			            console.log(message)",
        "let v = 'unused';
			            try {
			                v = callA();
			                try {
			                    v = callB();
			                } catch (e) {
			                    // ignore
			                }
			            } catch (e) {
			                v = 'used';
			            }
			            console.log(v)",
        "
			            var x = 1; // used
			            x = x + 1; // unused
			            x = 5; // used
			            f(x);",
        "
			            var x = 1; // used
			            x = // used
			                x++; // unused
			            f(x);",
        "const obj = { a: 1 };
			            let {
			                a,
			                b = (a = 2)
			            } = obj;
			            a = 3
			            console.log(a, b);",
        r#"function App() {
			            let A = "unused";
			            A = "used";
			            return <A/>;
			            }"#, // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			},
        r#"function App() {
			            let A = "unused";
			            A = "used";
			            return <A></A>;
			            }"#, // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			},
        r#"function App() {
			            let A = "unused";
			            A = "used";
			            return <A.B />;
			            }"#, // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			},
        r#"function App() {
			            let x = "used";
			            if (cond) {
			              return <A prop={x} />;
			            } else {
			              x = "unused";
			            }
			            }"#, // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			},
        r#"function App() {
			            let A;
			            A = "unused";
			            if (cond) {
			              A = "used1";
			            } else {
			              A = "used2";
			            }
			            return <A/>;
			            }"#, // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			},
        "function App() {
			            let message = 'unused';
			            try {
			              const result = call();
			              message = result.message;
			            } catch (e) {
			              message = 'used';
			            }
			            return <A prop={message} />;
			            }", // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			},
        "function App() {
			            let x = 1;
			            x = x + 1;
			            x = 5;
			            return <A prop={x} />;
			            }", // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			},
        "function App() {
			            let x = 1;
			            x = 2;
			            return <A>{x}</A>;
			            }", // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			},
        "function App() {
			            let x = 0;
			            x = 1;
			            x = 2;
			            return <A prop={x} />;
			            }", // {				"parserOptions": {					"ecmaFeatures": { "jsx": true },				},			}
    ];

    Tester::new(NoUselessAssignment::NAME, NoUselessAssignment::PLUGIN, pass, fail)
        .test_and_snapshot();
}
