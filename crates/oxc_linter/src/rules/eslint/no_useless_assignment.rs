use oxc_ast::{
    AstKind,
    ast::{Argument, AssignmentTarget, Expression},
};
use oxc_cfg::{
    BasicBlockId, ControlFlowGraph, EdgeType, ErrorEdgeKind, InstructionKind, graph::Direction,
    visit::neighbors_filtered_by_edge_weight,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{
    NodeId, SymbolId,
    dot::{DebugDot, DebugDotContext},
};
use oxc_span::{Atom, GetSpan, Span};

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
                AstKind::UpdateExpression(update) => {
                    let id_name = update.argument.get_identifier_name();
                    let Some(name) = id_name else {
                        continue;
                    };
                    let Some(symbol_id) = ctx.scoping().get_binding(node.scope_id(), &name) else {
                        continue;
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
    MaybeWrite,
    MaybeWriteRead,
    MaybeWriteWrittenInCatch,
    Missing,
}

const KEEP_WALKING_ON_THIS_PATH: bool = true;
const STOP_WALKING_ON_THIS_PATH: bool = false;

fn analyze(ctx: &LintContext, cfg: &ControlFlowGraph, start_node_id: NodeId, symbol_id: SymbolId) {
    let start_node = ctx.nodes().get_node(start_node_id);
    let start_node_bb_id = ctx.nodes().cfg_id(start_node_id);

    if pre_checks_skip(ctx, cfg, start_node_id, symbol_id) {
        return;
    }

    println!("-------------------------------");
    println!(
        "Analyzing assignment at span: {:?}",
        ctx.nodes().parent_node(start_node_id).span().source_text(ctx.source_text())
    );

    let found_usages = neighbors_filtered_by_edge_weight(
        cfg.graph(),
        start_node_bb_id,
        &|edge_type| match edge_type {
            EdgeType::Error(error) => match error {
                ErrorEdgeKind::Explicit => None,
                _ => Some(FoundAssignmentUsage::No),
            },
            EdgeType::Unreachable => Some(FoundAssignmentUsage::Missing),
            _ => None,
        },
        &mut |basic_block_id, state| {
            println!("Visiting basic block: {:?}", basic_block_id);

            // if the block has only one incoming edge which is an error edge, continue
            let mut incoming_edges =
                cfg.graph().edges_directed(*basic_block_id, Direction::Incoming);
            if incoming_edges.clone().count() > 0
                && incoming_edges
                    .all(|edge| matches!(edge.weight(), EdgeType::Error(ErrorEdgeKind::Explicit)))
            {
                println!("Block is only reachable via an error edge, continuing traversal");
                return (state, KEEP_WALKING_ON_THIS_PATH);
            }

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
                    AstKind::TryStatement(_) => continue,
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
                            if matches!(state, FoundAssignmentUsage::MaybeWrite) {
                                println!("State was MaybeWrite, returning MaybeWriteRead");
                                return (
                                    FoundAssignmentUsage::MaybeWriteRead,
                                    STOP_WALKING_ON_THIS_PATH,
                                );
                            } else {
                                return (FoundAssignmentUsage::Yes, STOP_WALKING_ON_THIS_PATH);
                            }
                        }
                        if reference.is_write() {
                            // check if the reference is part of a catch block and if state is maybewrite
                            if node_part_of_catch(ctx, &ref_node_id)
                                && matches!(state, FoundAssignmentUsage::MaybeWrite)
                            {
                                println!(
                                    "It's a write in a catch block with MaybeWrite state, returning MaybeWriteWrittenInCatch"
                                );
                                return (
                                    FoundAssignmentUsage::MaybeWriteWrittenInCatch,
                                    STOP_WALKING_ON_THIS_PATH,
                                );
                            }

                            // check if the write has a read to the same variable on the right side
                            // e.g., a = a + 1;
                            let parent_node = ctx.nodes().parent_node(ref_node_id);
                            if let AstKind::AssignmentExpression(assignment) = parent_node.kind() {
                                let rhs = &assignment.right;
                                if expr_uses_symbol(ctx, rhs, symbol_id) {
                                    println!(
                                        "It's a write that uses the variable on the right side, returning Yes"
                                    );
                                    return (FoundAssignmentUsage::Yes, STOP_WALKING_ON_THIS_PATH);
                                }
                            }

                            // check if the reference is in a try block, i.e. there is an explicit error
                            // edge from this block. If so, mark maybe used.
                            if write_part_of_error_block(ctx, &ref_node_id) {
                                println!("It's a write in a try/catch block, returning MaybeWrite");
                                return (
                                    FoundAssignmentUsage::MaybeWrite,
                                    KEEP_WALKING_ON_THIS_PATH,
                                );
                            }

                            println!("It's a write, returning No");
                            return (FoundAssignmentUsage::No, STOP_WALKING_ON_THIS_PATH);
                        }
                    }
                }
            }

            println!("No matching read/write found in this block, continuing traversal");
            (FoundAssignmentUsage::Missing, KEEP_WALKING_ON_THIS_PATH)
        },
    );

    println!("Found usages: {:?}", found_usages);

    if found_usages.iter().all(|usage| matches!(usage, FoundAssignmentUsage::Missing)) {
        return;
    }

    println!(
        "Done analyzing: {:?}",
        ctx.nodes().parent_node(start_node_id).span().source_text(ctx.source_text())
    );
    println!("-------------------------------");

    // Case: maybe written and read in catch, but not definitely read
    if found_usages.iter().any(|usage| matches!(usage, FoundAssignmentUsage::MaybeWriteRead))
        && !found_usages.iter().any(|usage| matches!(usage, FoundAssignmentUsage::Yes))
        && found_usages
            .iter()
            .any(|usage| matches!(usage, FoundAssignmentUsage::MaybeWriteWrittenInCatch))
    {
        ctx.diagnostic(no_useless_assignment_diagnostic(start_node.span()));
        return;
    }

    // Case: no definite reads found
    if !found_usages.iter().any(|usage| matches!(usage, FoundAssignmentUsage::Yes)) {
        ctx.diagnostic(no_useless_assignment_diagnostic(start_node.span()));
    }
}

fn pre_checks_skip(
    ctx: &LintContext,
    _cfg: &ControlFlowGraph,
    start_node_id: NodeId,
    _symbol_id: SymbolId,
) -> bool {
    let start_node = ctx.nodes().get_node(start_node_id);

    let (name, span) = match start_node.kind() {
        AstKind::VariableDeclarator(declarator) => {
            let Some(identifier) = declarator.id.get_binding_identifier() else {
                return false;
            };
            (identifier.name, identifier.span)
        }
        AstKind::AssignmentExpression(assignment) => {
            let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assignment.left else {
                return false;
            };
            (ident.name, ident.span)
        }
        _ => return false,
    };

    if is_exported(ctx, name, span) {
        return true;
    }

    if is_var_function_decl(ctx, start_node_id) {
        return true;
    }

    if is_in_unreachable_block(ctx, start_node_id) {
        return true;
    }

    return false;
}

fn is_exported(ctx: &LintContext, name: Atom, span: Span) -> bool {
    let modules = ctx.module_record();

    modules.exported_bindings.contains_key(name.as_str())
        || modules.export_default.is_some_and(|default| default == span)
        || modules
            .local_export_entries
            .iter()
            .any(|entry| entry.local_name.name().is_some_and(|n| n == name))
}

fn is_var_function_decl(ctx: &LintContext, node_id: NodeId) -> bool {
    let node = ctx.nodes().get_node(node_id);
    match node.kind() {
        AstKind::VariableDeclarator(declarator) => {
            let Some(init) = &declarator.init else {
                return false;
            };
            declarator.kind.is_var()
                && matches!(
                    init.without_parentheses(),
                    Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
                )
        }
        _ => false,
    }
}

fn is_in_unreachable_block(ctx: &LintContext, node_id: NodeId) -> bool {
    let cfg_id = ctx.nodes().cfg_id(node_id);
    let cfg = ctx.cfg();

    if cfg.graph().neighbors_directed(cfg_id, Direction::Incoming).count() == 1 {
        let incoming_edge = cfg.graph().edges_directed(cfg_id, Direction::Incoming).next().unwrap();
        if let EdgeType::Unreachable = incoming_edge.weight() {
            return true;
        }
    }

    false
}

fn expr_uses_symbol(ctx: &LintContext, expr: &Expression, symbol_id: SymbolId) -> bool {
    println!("Checking expr: {:?} {}", expr, expr.span().source_text(ctx.source_text()));
    match expr {
        Expression::Identifier(identifier) => {
            let reference = identifier.reference_id();
            if let Some(id_symbol) = ctx.scoping().get_reference(reference).symbol_id() {
                return id_symbol == symbol_id;
            }
            false
        }
        Expression::BinaryExpression(binary_expr) => {
            expr_uses_symbol(ctx, &binary_expr.left, symbol_id)
                || expr_uses_symbol(ctx, &binary_expr.right, symbol_id)
        }
        Expression::CallExpression(call_expr) => {
            if expr_uses_symbol(ctx, &call_expr.callee, symbol_id) {
                return true;
            }
            for arg in &call_expr.arguments {
                match arg {
                    Argument::SpreadElement(spread) => {
                        if expr_uses_symbol(ctx, &spread.argument, symbol_id) {
                            return true;
                        }
                    }
                    _ => {
                        if arg.is_expression()
                            && expr_uses_symbol(ctx, arg.to_expression(), symbol_id)
                        {
                            return true;
                        }
                    }
                }
            }
            false
        }
        Expression::UnaryExpression(unary_expr) => {
            expr_uses_symbol(ctx, &unary_expr.argument, symbol_id)
        }
        _ => false,
    }
}

fn write_part_of_error_block(ctx: &LintContext, node_id: &NodeId) -> bool {
    let basic_block_id = ctx.nodes().cfg_id(*node_id);
    let cfg = ctx.cfg();

    println!("write_part_of_error_block, edges from block {:?}:", basic_block_id);
    println!(
        "{:?}",
        cfg.graph().edges_directed(basic_block_id, Direction::Outgoing).collect::<Vec<_>>()
    );

    cfg.graph().edges_directed(basic_block_id, Direction::Outgoing).any(|edge| {
        if matches!(edge.weight(), EdgeType::Error(ErrorEdgeKind::Explicit) | EdgeType::Finalize) {
            true
        } else {
            false
        }
    })
}

fn node_part_of_catch(ctx: &LintContext, node_id: &NodeId) -> bool {
    for kind in ctx.nodes().ancestor_kinds(*node_id) {
        if let AstKind::CatchClause(_) = kind {
            return true;
        }
        if let AstKind::TryStatement(_) = kind {
            return false;
        }
    }
    false
}

fn _node_part_of_finally(ctx: &LintContext, node_id: &NodeId) -> bool {
    let basic_block_id = ctx.nodes().cfg_id(*node_id);
    let cfg = ctx.cfg();

    println!("node_part_of_finally, edges from block {:?}:", basic_block_id);
    println!(
        "{:?}",
        cfg.graph().edges_directed(basic_block_id, Direction::Incoming).collect::<Vec<_>>()
    );

    cfg.graph()
        .edges_directed(basic_block_id, Direction::Incoming)
        .any(|edge| if matches!(edge.weight(), EdgeType::Finalize) { true } else { false })
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
        // "/* exported foo */
        //           let foo = 'used';
        //           console.log(foo);
        //           foo = 'unused like but exported with directive';", // { "sourceType": "script" },
        // "/*eslint test/use-a:1*/
        //       let a = 'used';
        //       console.log(a);
        //       a = 'unused like but marked by markVariableAsUsed()';
        //       ",
        "v = 'used';
			        console.log(v);
			        v = 'unused'",
        "let v = 'used variable';", // Incorrect test case?
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

    // let _pass = vec![
    //     // "let message = 'init';
    //           //       try {
    //           //           const result = call();
    //           //           message = result.message;
    //           //       } catch (e) {
    //           //           // ignore
    //           //       }
    //           //       console.log(message)",
    // ];

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
        "let message = 'unused';
			            try {
			                const result = call();
			                message = result.message;
			            } catch (e) {
			                message = 'used';
			            }
			            console.log(message)",
        // "let message = 'unused';
        //           try {
        //               message = 'used';
        //               console.log(message)
        //           } catch (e) {
        //           }",
    ];

    Tester::new(NoUselessAssignment::NAME, NoUselessAssignment::PLUGIN, pass, fail)
        .test_and_snapshot();
}
