use oxc_ast::{AstKind, ast::AssignmentTarget};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_semantic::{NodeId, Reference, ReferenceFlags, SymbolId};
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
                        if is_dead_store(symbol_id, ctx) {
                            ctx.diagnostic(no_useless_assignment_diagnostic(assign.span));
                        }
                    }
                }
            }
        }
    }
}

fn is_dead_store(symbol_id: SymbolId, ctx: &LintContext<'_>) -> bool {
    // Get all references to this symbol ordered by span position
    let references = ctx.semantic().symbol_references(symbol_id).collect::<Vec<_>>();
    // get SymbolId's declaration node
    let declaration_node_id = ctx.semantic().scoping().symbol_declaration(symbol_id);
    let has_if_else_loop_after_declaration = check_has_if_else_loop_after_declaration(&declaration_node_id, &references, ctx);

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


    // create a reference for the declaration node
    let dec_control_parent = find_control_parent(declaration_node_id, ctx);
    let dec_ref = Reference::new(declaration_node_id, ReferenceFlags::None);
    if let Some(parent_id) = dec_control_parent {
        reference_group.entry(parent_id).or_insert_with(Vec::new).push(&dec_ref);
    } else {
        reference_group_without_parent.push(&dec_ref);
    }

    // combine reference_group values and reference_group_without_parent to one vector with vector item
    let mut reference_group_values = vec![];
    for references in reference_group.values() {
        reference_group_values.push(references);
    }

    if !reference_group_without_parent.is_empty() {
        reference_group_values.push(&reference_group_without_parent);
    }

    let mut has_read = false;
    let mut is_multiple_assignments = false;
    for reference_group_value in reference_group_values {
        let mut read_count_total = 0;
        let mut write_count_total = 0;
        let mut write_count = 0;
        let mut has_declaration = false;
        let mut declaration_node_idx = 0;

        let mut sorted_references = reference_group_value.clone();
        sorted_references.sort_by_key(|reference| reference.node_id());
        for (idx, reference) in sorted_references.iter().enumerate() {
            if has_declaration && idx - 1 == declaration_node_idx {
                // can't assign immediately after declaration
                if reference.flags().is_write() {
                    is_multiple_assignments = true;
                }
            }

            // check if reference is declaration
            if reference.node_id() == declaration_node_id {
                has_declaration = true;
                declaration_node_idx = idx;
            }

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


        // if there is only one reference and no if/else/loop statement after it, it's a dead store
        if has_declaration &&
            reference_group_value.len() == 1 &&
            !has_if_else_loop_after_declaration
            {
            return true;
        }

        if write_count_total > read_count_total {
            is_multiple_assignments = true;
        }

        if write_count > 0 && !in_loop_context(reference_group_value[0].node_id(), ctx) && read_count_total > 0 {
            return true;
        }
    }

    // bypass no-unused-vars
    if is_multiple_assignments && has_read {
        return true;
    }

    false
}


fn check_has_if_else_loop_after_declaration(dec_node_id: &NodeId, references: &Vec<&Reference>, ctx: &LintContext<'_>) -> bool {
    for reference in references {
        // Find the containing control structure (if statement, for loop, etc.)
        if reference.node_id() > *dec_node_id {
            let control_parent = find_control_parent(reference.node_id(), ctx);
            // check if control_parent is IfStatement
            if let Some(parent_node_id) = control_parent {
                if has_if_else_context(parent_node_id, ctx) {
                    return true;
                }
                if has_loop_context(parent_node_id, ctx) {
                    return true;
                }
            }
        }
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
                AstKind::DoWhileStatement(_)
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
            if has_loop_context(parent_id, ctx) {
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

fn has_loop_context(node_id: NodeId, ctx: &LintContext<'_>) -> bool {
    matches!(
        ctx.semantic().nodes().get_node(node_id).kind(),
        AstKind::ForStatement(_) |
        AstKind::ForInStatement(_) |
        AstKind::ForOfStatement(_) |
        AstKind::WhileStatement(_) |
        AstKind::DoWhileStatement(_)
    )
}

fn has_if_else_context(node_id: NodeId, ctx: &LintContext<'_>) -> bool {
    if let AstKind::IfStatement(statement) = ctx.semantic().nodes().get_node(node_id).kind() {
        statement.alternate.is_some()
    } else {
        false
    }
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
        ,
        "
            function fn6() {
                let v = 'used1';
                if (condition) {
                    let v = 'used2';
                    v = 'unused1';
                    console.log(v);
                }
                console.log(v);
            }
        ",
        "
            function fn7() {
                let v = 'used1';
                if (condition) {
                    let v = 'used2';
                    v = 'unused1';
                    console.log(v);
                }
                console.log(v);
            }
        ",
    ];

    Tester::new(NoUselessAssignment::NAME, NoUselessAssignment::PLUGIN, pass, fail)
        .test_and_snapshot();
}

