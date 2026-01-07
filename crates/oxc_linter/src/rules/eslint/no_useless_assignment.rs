use oxc_ast::{AstKind, ast::AssignmentTarget};
use oxc_cfg::{
    ControlFlowGraph, EdgeType, ErrorEdgeKind, InstructionKind,
    visit::neighbors_filtered_by_edge_weight,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{
    NodeId, SymbolId,
    dot::{DebugDot, DebugDotContext},
};
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule};

fn no_useless_assignment_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow variable assignments when the value is not used")
        .with_label(span.label("This assigned value is not used in subsequent statements."))
        .with_help("Remove this assignment.")
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
    correctness, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    pending, // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for NoUselessAssignment {
    fn run_once(&self, ctx: &LintContext) {
        let cfg = ctx.cfg();
        println!("{}", cfg.debug_dot(DebugDotContext::new(ctx.nodes(), true)));
        let nodes = ctx.nodes();
        for node in nodes.iter() {
            match node.kind() {
                AstKind::VariableDeclarator(declarator) => {
                    let Some(_) = &declarator.init else {
                        continue;
                    };
                    let Some(identifier) = declarator.id.get_binding_identifier() else {
                        continue;
                    };
                    let symbol_id = identifier.symbol_id();
                    analyze(ctx, cfg, node.id(), symbol_id);
                }
                AstKind::AssignmentExpression(assignment) => {
                    let symbol_id = match &assignment.left {
                        AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                            let reference = ident.reference_id();
                            match ctx.scoping().get_reference(reference).symbol_id() {
                                Some(symbol_id) => symbol_id,
                                None => continue,
                            }
                        }
                        // AssignmentTarget::ObjectAssignmentTarget(target) => {
                        // 	target.
                        // }
                        _ => continue,
                    };
                    analyze(ctx, cfg, node.id(), symbol_id);
                }
                _ => continue,
            }
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
enum FoundAssignmentUsage {
    #[default]
    Yes,
    No,
}

const KEEP_WALKING_ON_THIS_PATH: bool = true;
const STOP_WALKING_ON_THIS_PATH: bool = false;

fn analyze(ctx: &LintContext, cfg: &ControlFlowGraph, start_node_id: NodeId, symbol_id: SymbolId) {
    let start_node = ctx.nodes().get_node(start_node_id);
    let start_node_bb_id = ctx.nodes().cfg_id(start_node_id);

    println!("-------------------------------");
    println!(
        "Analyzing assignment at span: {:?}",
        ctx.nodes().parent_node(start_node_id).span().source_text(ctx.source_text())
    );

    let found_usage = neighbors_filtered_by_edge_weight(
        cfg.graph(),
        start_node_bb_id,
        &|edge_type| match edge_type {
            EdgeType::Error(error) => match error {
                ErrorEdgeKind::Explicit => None,
                _ => Some(FoundAssignmentUsage::No),
            },
            EdgeType::Unreachable => Some(FoundAssignmentUsage::No),
            _ => None,
        },
        &mut |basic_block_id, _| {
            println!("Visiting basic block: {:?}", basic_block_id);
            let basic_block = cfg.basic_block(*basic_block_id);
            let is_start_block = *basic_block_id == start_node_bb_id;
            let mut found_assignment = false;

            for instruction in &basic_block.instructions {
                match instruction.kind {
                    InstructionKind::Statement
                    | InstructionKind::Condition
                    | InstructionKind::Return(_) => {}
                    _ => continue,
                }
                let Some(node_id) = instruction.node_id else {
                    continue;
                };
                let instr_node = ctx.nodes().get_node(node_id);

                match instr_node.kind() {
                    AstKind::IfStatement(_) => continue,
                    _ => {}
                }

                let instr_span = instr_node.span();

                if is_start_block {
                    if instr_span.contains_inclusive(start_node.span()) {
                        found_assignment = true;
                        continue;
                    }
                    if !found_assignment {
                        continue;
                    }
                }

                let references = ctx.scoping().get_resolved_references(symbol_id);
                for reference in references {
                    let ref_node_id = reference.node_id();
                    let ref_span = ctx.nodes().get_node(ref_node_id).span();

                    if instr_span.contains_inclusive(ref_span) {
                        println!(
                            "Found reference at span: {:?}",
                            ref_span.source_text(ctx.source_text())
                        );
                        println!(
                            "{:?}",
                            ctx.nodes()
                                .parent_node(ref_node_id)
                                .span()
                                .source_text(ctx.source_text())
                        );
                        if reference.is_read() {
                            println!("It's a read, returning Yes");
                            return (FoundAssignmentUsage::Yes, STOP_WALKING_ON_THIS_PATH);
                        }
                        if reference.is_write() {
                            println!("It's a write, returning No");
                            return (FoundAssignmentUsage::No, STOP_WALKING_ON_THIS_PATH);
                        }
                    }
                }
            }

            println!("No matching read/write found in this block, continuing traversal");
            (FoundAssignmentUsage::No, KEEP_WALKING_ON_THIS_PATH)
        },
    )
    .iter()
    .any(|result| matches!(result, FoundAssignmentUsage::Yes));

    println!("Found usage: {}", found_usage);
    println!("{:?}", ctx.nodes().parent_node(start_node_id).span().source_text(ctx.source_text()));
    println!("-------------------------------");
    if !found_usage {
        ctx.diagnostic(no_useless_assignment_diagnostic(start_node.span()));
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
        // "/* globals foo */
        //       const bk = foo;
        //       foo = 42;
        //       try {
        //           // process
        //       } finally {
        //           foo = bk;
        //       }",
        "
			            const bk = console;
			            console = { log () {} };
			            try {
			                // process
			            } finally {
			                console = bk;
			            }", // {  "globals": { "console": false },  },
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
        //       let a = "used";
        // 	console.log(a);
        // 	a = "unused";"#,
        // r#"/*eslint test/unknown-ref:1*/
        // 	function foo() {
        // 		let a = "used";
        // 		console.log(a);
        // 		a = "unused";
        // 	}"#,
        // r#"/*eslint test/unknown-ref:1*/
        // 	function foo() {
        // 		let a = "used";
        // 		if (condition) {
        // 			a = "unused";
        // 			return
        // 		}
        // 		console.log(a);
        //       }"#,
        r#"
			                function App() {
			                    const A = "";
			                    return <A/>;
			                }
			            "#, // {  "parserOptions": {  "ecmaFeatures": {  "jsx": true,  },  },  },
        r#"
			                function App() {
			                    let A = "";
			                    foo(A);
			                    A = "A";
			                    return <A/>;
			                }
			            "#, // {  "parserOptions": {  "ecmaFeatures": {  "jsx": true,  },  },  },
        r#"
			                function App() {
								let A = "a";
			                    foo(A);
			                    return <A/>;
			                }
			            "#, // {  "parserOptions": {  "ecmaFeatures": {  "jsx": true,  },  },  },
        "function App() {
							let x = 0;
							foo(x);
							x = 1;
							return <A prop={x} />;
						}", // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  },
        r#"function App() {
							let x = "init";
							foo(x);
							x = "used";
							return <A>{x}</A>;
						}"#, // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  },
        "function App() {
							let props = { a: 1 };
							foo(props);
							props = { b: 2 };
							return <A {...props} />;
						}", // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  },
        "function App() {
							let NS = Lib;
							return <NS.Cmp />;
						}", // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  },
        "function App() {
							let a = 0;
							a++;
							return <A prop={a} />;
						}", // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  },
        "function App() {
							const obj = { a: 1 };
							const { a, b = a } = obj;
							return <A prop={b} />;
						}", // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  },
        "function App() {
							let { a, b: { c = a } = {} } = obj;
							return <A prop={c} />;
						}", // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  },
        r#"function App() {
							let x = "init";
							if (cond) {
								x = "used";
								return <A prop={x} />;
							}
							return <A prop={x} />;
						}"#, // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  },
        "function App() {
							let A;
							if (cond) {
							  A = Foo;
							} else {
							  A = Bar;
							}
							return <A />;
						}", // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  },
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
						}", // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  },
        "function App() {
							const arr = [6];
							const [c, d = c] = arr;
							return <A prop={d} />;
						}", // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  },
        "function App() {
							const obj = { a: 1 };
							let {
							  a,
							  b = (a = 2)
							} = obj;
							return <A prop={a} />;
						}", // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  }
    ];

    let _pass = vec![
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
			            }"#, // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  },
        r#"function App() {
			            let A = "unused";
			            A = "used";
			            return <A></A>;
			            }"#, // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  },
        r#"function App() {
			            let A = "unused";
			            A = "used";
			            return <A.B />;
			            }"#, // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  },
        r#"function App() {
			            let x = "used";
			            if (cond) {
			              return <A prop={x} />;
			            } else {
			              x = "unused";
			            }
			            }"#, // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  },
        r#"function App() {
			            let A;
			            A = "unused";
			            if (cond) {
			              A = "used1";
			            } else {
			              A = "used2";
			            }
			            return <A/>;
			            }"#, // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  },
        "function App() {
			            let message = 'unused';
			            try {
			              const result = call();
			              message = result.message;
			            } catch (e) {
			              message = 'used';
			            }
			            return <A prop={message} />;
			            }", // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  },
        "function App() {
			            let x = 1;
			            x = x + 1;
			            x = 5;
			            return <A prop={x} />;
			            }", // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  },
        "function App() {
			            let x = 1;
			            x = 2;
			            return <A>{x}</A>;
			            }", // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  },
        "function App() {
			            let x = 0;
			            x = 1;
			            x = 2;
			            return <A prop={x} />;
			            }", // {  "parserOptions": {  "ecmaFeatures": { "jsx": true },  },  }
    ];

    let _fail = vec![
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
    ];

    Tester::new(NoUselessAssignment::NAME, NoUselessAssignment::PLUGIN, pass, fail)
        .test_and_snapshot();
}
