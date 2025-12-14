use oxc_ast::{AstKind, ast::BindingIdentifier};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{NodeId, Reference};
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

use crate::{context::LintContext, rule::Rule};

fn no_useless_assignment_diagnostic(name: &str, first_write: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Useless assignment to variable {name}"))
        .with_labels([first_write.label(format!("{name} is written to here but never read"))])
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessAssignment;

declare_oxc_lint!(
  /// # What it does
  ///
  /// Disallow variable assignments when the value is not used
  ///
  /// ## Why is this bad?
  ///
  /// “Dead stores” waste processing and memory, so it is better to remove unnecessary assignments to variables.
  /// Also, if the author intended the variable to be used, there is likely a mistake around the dead store.
  NoUselessAssignment,
  eslint,
  correctness
);

impl Rule for NoUselessAssignment {
    fn run_once(&self, ctx: &LintContext) {
        let cfg = ctx.cfg();
        let nodes = ctx.nodes();

        for node in nodes {
            match node.kind() {
                AstKind::VariableDeclarator(declarator) => {
                    if let Some(ident) = declarator.id.get_binding_identifier() {
                        let cfg_block_id = nodes.cfg_id(node.id());
                        println!(
                            "Found initialization of variable '{}' in CFG block {:?}",
                            ident, cfg_block_id
                        );
                        let symbol_id = ident.symbol_id();
                        let references: Vec<_> =
                            ctx.semantic().symbol_references(symbol_id).collect();
                        let cfg_nodes = references
                            .iter()
                            .map(|r| nodes.cfg_id(r.node_id()))
                            .collect::<FxHashSet<_>>();

                        let has_multi_cfg_references = cfg_nodes.len() > 1
                            || if let Some(_) = &declarator.init {
                                let init_cfg_id = nodes.cfg_id(node.id());
                                !cfg_nodes.contains(&init_cfg_id)
                            } else {
                                false
                            };

                        println!(
                            "Variable '{}' has references in CFG blocks: {:?}",
                            ident, has_multi_cfg_references
                        );

                        let mut last_write: Option<NodeId> = None;
                        // if has initialization
                        if let Some(_) = &declarator.init {
                            last_write = Some(node.id());
                        }

                        println!(
                            "Processing references for variable '{}', has_multi_cfg_references: {}",
                            ident, has_multi_cfg_references
                        );

                        if has_multi_cfg_references {
                            handle_multi_block_references(&ident, &references, ctx, last_write);
                        } else {
                            handle_single_block_references(&ident, &references, ctx);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn handle_multi_block_references(
    ident: &BindingIdentifier,
    references: &[&Reference],
    ctx: &LintContext,
    last_write: Option<NodeId>,
) {
    // function fn2() {
    //     let v = 'used';
    //     if (condition) {
    //         v = 'unused';
    //         return
    //     }
    //     doSomething(v);
    // }
    let nodes = ctx.nodes();
    let cfg = match ctx.semantic().cfg() {
        Some(cfg) => cfg,
        None => return,
    };
    let mut local_last_write = last_write;
    println!("Handling multi-block references for variable '{}'", ident);

    // logic:
    /*
    - split references by cfg blocks
    - start with first block, keep track of last write
    - for this block:
        - for each successor block:
            - call handle_single_block_references with references in that block
            - if any successor block returns a last write, break
        - if any successor block returns a last write, report local last write
        - else, continue to next block
     */

    // let mut block_references: FxHashMap<NodeIndex, Vec<&Reference>> = FxHashMap::default();
    // for r in references {
    //     block_references.entry(nodes.cfg_id(r.node_id()).clone()).or_default().push(r);
    // }

    // let first_block = if let Some(first_ref) = references.first() {
    //     nodes.cfg_id(first_ref.node_id())
    // } else {
    //     return;
    // };

    // cfg.graph.neighbors(first_block).map(|n| {
    //     if let Some(block_refs) = block_references.get(&n) {
    //         handle_single_block_references(ident, block_refs, ctx, local_last_write.clone());
    //     }
    // });

    // logic:
    /*
       - start with the first write, keep track of last write
       - do recursively:
       - if cfg block changes, for each path from last write block to current block
           - if any path has a read before next write, break
           - if any path ends without read, report last write
       - at the end, for each path from last write block to end of function
           - if any path has a read before next write, break
           - if any path ends without read, report last write
    */

    for (i, r) in references.iter().enumerate() {
        println!(
            "Reference to variable '{}' at node {:?} is a {}",
            ident,
            nodes.get_node(r.node_id()).span().start,
            if r.is_write() { "write" } else { "read" }
        );
        println!(
            "Handling write reference to variable '{}' at node {:?}",
            ident,
            nodes.get_node(r.node_id()).span().start
        );
        if let Some(local_last_write) = local_last_write {
            let last_write_node = nodes.get_node(local_last_write);
            let r_node = nodes.get_node(r.node_id());
            // print src line for last_write_node and r_node
            println!(
                "Last write node src: '{:?}'",
                ctx.source_text()
                    .get(
                        (last_write_node.span().start as usize) - 2
                            ..last_write_node.span().end as usize + 2
                    )
                    .unwrap()
            );
            println!(
                "Current reference node src: '{:?}'",
                ctx.source_text()
                    .get((r_node.span().start as usize) - 2..r_node.span().end as usize + 2)
                    .unwrap()
            );
            if r.is_write() {
                ctx.diagnostic(no_useless_assignment_diagnostic(
                    &ident.name,
                    last_write_node.span(),
                ));
            }
            if nodes.cfg_id(local_last_write) != nodes.cfg_id(r.node_id()) {
                let is_reachable =
                    cfg.is_reachable(nodes.cfg_id(local_last_write), nodes.cfg_id(r.node_id()));
                if is_reachable {
                    handle_multi_block_references(
                        ident,
                        &references[i + 1..],
                        ctx,
                        Some(local_last_write),
                    );
                    return;
                }
            }
        }

        local_last_write = if r.is_write() { Some(r.node_id()) } else { None };
    }

    if let Some(local_last_write) = local_last_write {
        let last_write_node = nodes.get_node(local_last_write);
        ctx.diagnostic(no_useless_assignment_diagnostic(&ident.name, last_write_node.span()));
    }
}

fn handle_single_block_references(
    ident: &BindingIdentifier,
    references: &[&Reference],
    ctx: &LintContext,
    // last_write: Option<NodeId>,
) {
    // function fn1() {
    //     let v = 'used';
    //     doSomething(v);
    //     v = 'unused';
    // }
    let nodes = ctx.nodes();
    let mut last_reference: Option<&Reference> = None;
    for r in references {
        if let Some(last_ref) = last_reference {
            let last_ref_node = nodes.get_node(last_ref.node_id());
            if r.is_write() && last_ref.is_write() {
                let diagnostic =
                    no_useless_assignment_diagnostic(&ident.name, last_ref_node.span());
                ctx.diagnostic(diagnostic);
            }
        }
        last_reference = Some(r);
    }

    if let Some(last_ref) = last_reference {
        let last_ref_node = nodes.get_node(last_ref.node_id());
        if last_ref.is_write() {
            let diagnostic = no_useless_assignment_diagnostic(&ident.name, last_ref_node.span());
            ctx.diagnostic(diagnostic);
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("let x = 5; console.log(x);", None),
        (
            "function fn1() {
        let v = 'used';
        doSomething(v);
        v = 'used-2';
        doSomething(v);
    }",
            None,
        ),
        (
            "function fn2() {
        let v = 'used';
        if (condition) {
            v = 'used-2';
            doSomething(v);
            return
        }
        doSomething(v);
    }",
            None,
        ),
        (
            "function fn3() {
        let v = 'used';
        if (condition) {
            doSomething(v);
        } else {
            v = 'used-2';
            doSomething(v);
        }
    }",
            None,
        ),
        (
            "function fn4() {
        let v = 'used';
        for (let i = 0; i < 10; i++) {
            doSomething(v);
            v = 'used in next iteration';
        }
    }",
            None,
        ),
    ];
    let fail = vec![
        ("let x = 5; x = 7;", None),
        (
            "function fn1() {
            let v = 'used';
            doSomething(v);
            v = 'unused';
        }",
            None,
        ),
        (
            "function fn2() {
            let v = 'used';
            if (condition) {
                v = 'unused';
                return
            }
            doSomething(v);
        }",
            None,
        ),
        (
            "function fn3() {
            let v = 'used';
            if (condition) {
                doSomething(v);
            } else {
                v = 'unused';
            }
        }",
            None,
        ),
        (
            "function fn4() {
            let v = 'unused';
            if (condition) {
                v = 'used';
                doSomething(v);
                return
            }
        }",
            None,
        ),
        (
            "function fn5() {
            let v = 'used';
            if (condition) {
                let v = 'used';
                console.log(v);
                v = 'unused';
            }
            console.log(v);
        }",
            None,
        ),
    ];

    Tester::new(NoUselessAssignment::NAME, NoUselessAssignment::PLUGIN, pass, fail)
        .test_and_snapshot();
}
