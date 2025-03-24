use oxc_ast::{AstKind, ast::AssignmentTarget};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_semantic::{SymbolId, NodeId, ReferenceId, ScopeId, Reference};
use rustc_hash::FxHashMap;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_useless_assignment_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("This assigned value is not used in subsequent statements.")
        .with_label(span)
        .with_help("Remove the assignment.")
}


#[derive(Debug, Default, Clone)]
pub struct NoUselessAssignment;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow variable assignments when the value is not used
    ///
    /// ### Why is this bad?
    ///
    /// Unused assignments can make the code harder to read and understand.
    ///
    /// ### Examples
    ///
    /// ```javascript
    /// let id = "x1234";    // this is a "dead store" - this value ("x1234") is never read
    ///
    /// id = generateId();
    ///
    /// doSomethingWith(id);
    /// ```
    NoUselessAssignment,
    eslint,
    correctness,
);

impl Rule for NoUselessAssignment {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::AssignmentExpression(assign) = node.kind() {
            if let AssignmentTarget::AssignmentTargetIdentifier(id) = &assign.left {
                if let Some(reference_id) = id.reference_id.get() {
                    let reference = ctx.semantic().scoping().get_reference(reference_id);
                    if let Some(symbol_id) = reference.symbol_id() {
                        if is_dead_store(symbol_id, node.id(), ctx) {
                            ctx.diagnostic(no_useless_assignment_diagnostic(assign.span));
                        }
                    }
                }
            }
        }
    }
}

// Check if a symbol is a "dead store" - not read after this point
fn is_dead_store(symbol_id: SymbolId, _node_id: NodeId, ctx: &LintContext<'_>) -> bool {
    // Check if any assignment happens before using it
    has_multiple_assignments_before_read(symbol_id, ctx)
}


fn has_multiple_assignments_before_read(symbol_id: SymbolId, ctx: &LintContext<'_>) -> bool {
    // Get all references to this symbol ordered by span position
    let references = ctx.semantic().symbol_references(symbol_id).collect::<Vec<_>>();

    // Create a map to group references by control structure
    let mut reference_group = FxHashMap::default();
    let mut reference_group_without_parent = vec![];

    for reference in references {
        // Find the containing control structure (if statement, for loop, etc.)
        let control_parent = find_control_parent(reference.node_id(), ctx);
        if let Some(parent_id) = control_parent {
            reference_group.entry(parent_id).or_insert_with(Vec::new).push(reference);
        } else {
            reference_group_without_parent.push(reference);
        }
    }

    // combine reference_group values and reference_group_without_parent to one vector with vector item
    let mut reference_group_values = vec![];
    for references in reference_group.values() {
        reference_group_values.push(references);
    }

    if !reference_group_without_parent.is_empty() {
        reference_group_values.push(&reference_group_without_parent);
    }

    println!("reference_group_values: {:?}", reference_group_values);

    let mut has_read = false;
    let mut is_multiple_assignments_without_read = false;
    for reference_group_value in reference_group_values {
        let mut read_count_total = 0;
        let mut write_count_total = 0;
        let mut write_count = 0;
        for reference in reference_group_value.iter() {
            if reference.flags().is_write() {
               // Count this as an assignment
                write_count += 1;
                write_count_total += 1;
            }

            if write_count > 1 {
                return true;
            }

            if reference.flags().is_read() {
                has_read = true;
                read_count_total += 1;
                write_count = 0;
            }
        }

        if write_count_total > read_count_total {
            is_multiple_assignments_without_read = true;
        }

        if write_count > 0 && !in_loop_context(reference_group_value[0].node_id(), ctx) && read_count_total > 0 {
            return true;
        }
    }

    if is_multiple_assignments_without_read && has_read {
        return true;
    }

    false
}

// Helper to find the parent control structure (if/for/while statements)
fn find_control_parent(node_id: NodeId, ctx: &LintContext<'_>) -> Option<NodeId> {
    let mut current = Some(node_id);

    while let Some(current_id) = current {
        let parent_id = ctx.semantic().nodes().parent_id(current_id);

        if let Some(parent_id) = parent_id {
            let parent = ctx.semantic().nodes().get_node(parent_id);

            if matches!(
                parent.kind(),
                AstKind::IfStatement(_) |
                AstKind::ForStatement(_) |
                AstKind::ForInStatement(_) |
                AstKind::ForOfStatement(_) |
                AstKind::WhileStatement(_) |
                AstKind::DoWhileStatement(_) |
                AstKind::BlockStatement(_)
            ) {
                return Some(parent_id);
            }

            // Continue with the parent
            current = Some(parent_id);
        } else {
            break;
        }
    }

    None
}

// Helper to check if a node is within a loop context
fn in_loop_context(node_id: NodeId, ctx: &LintContext<'_>) -> bool {
    let mut current = Some(node_id);

    while let Some(current_id) = current {
        let parent_id = ctx.semantic().nodes().parent_id(current_id);

        if let Some(parent_id) = parent_id {
            let parent = ctx.semantic().nodes().get_node(parent_id);

            if matches!(
                parent.kind(),
                AstKind::ForStatement(_) |
                AstKind::ForInStatement(_) |
                AstKind::ForOfStatement(_) |
                AstKind::WhileStatement(_) |
                AstKind::DoWhileStatement(_)
            ) {
                return true;
            }

            // Continue with the parent
            current = Some(parent_id);
        } else {
            break;
        }
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "
            function fn1() {
                let v = 'used';
                doSomething(v);
                v = 'used-2';
                doSomething(v);
            }
        ",
        "
            function fn2() {
                let v = 'used';
                if (condition) {
                    v = 'used-2';
                    doSomething(v);
                    return
                }
                doSomething(v);
            }
        ",
        "
            function fn3() {
                let v = 'used';
                if (condition) {
                    doSomething(v);
                } else {
                    v = 'used-2';
                    doSomething(v);
                }
            }
        ",
        "
            function fn4() {
                let v = 'used';
                for (let i = 0; i < 10; i++) {
                    doSomething(v);
                    v = 'used in next iteration';
                }
            }
        ",
        "
            function fn5() {
                let v = 'unused';
                v = 'unused-2'
                doSomething();
            }
        "
    ];

    let fail = vec![
        "
            function fn1() {
                let v = 'used';
                doSomething(v);
                v = 'unused';
            }
        ",

        "
            function fn2() {
            let v = 'used';
            if (condition) {
                v = 'unused';
                return
            }
            doSomething(v);
        }",

        "
        function fn3() {
            let v = 'used';
            if (condition) {
                doSomething(v);
            } else {
                v = 'unused';
            }
        }
        ",
        "
        function fn4() {
            let v = 'unused';
            if (condition) {
                v = 'used';
                doSomething(v);
                return
            }
        }",
        "
        function fn5() {
            let v = 'used';
            if (condition) {
                let v = 'used';
                console.log(v);
                v = 'unused';
            }
            console.log(v);
        }"
    ];

    Tester::new(NoUselessAssignment::NAME, NoUselessAssignment::PLUGIN, pass, fail)
        .test_and_snapshot();
}

