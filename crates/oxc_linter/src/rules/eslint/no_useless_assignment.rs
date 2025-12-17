use oxc_ast::{AstKind, ast::BindingIdentifier};
use oxc_cfg::BlockNodeId;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{NodeId, Reference};
use oxc_span::{GetSpan, Span};
use rustc_hash::{FxHashMap, FxHashSet};

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

// LastWrite can be either a Reference or NodeId of the initialization or None

#[derive(Debug, Clone)]
enum LastWrite {
    Reference(Reference),
    Initialization(NodeId),
    None,
}

impl Rule for NoUselessAssignment {
    fn run_once(&self, ctx: &LintContext) {
        let nodes = ctx.nodes();

        for node in nodes {
            match node.kind() {
                AstKind::VariableDeclarator(declarator) => {
                    if let Some(ident) = declarator.id.get_binding_identifier() {
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

                        let last_write: LastWrite = if let Some(_) = &declarator.init {
                            LastWrite::Initialization(node.id())
                        } else {
                            LastWrite::None
                        };

                        if has_multi_cfg_references {
                            handle_multi_block_references(
                                &ident,
                                &references,
                                ctx,
                                last_write,
                                &mut FxHashSet::default(),
                            );
                        } else {
                            let write = handle_single_block_references(&references, last_write);

                            match &write {
                                LastWrite::Reference(r) => {
                                    let diagnostic = no_useless_assignment_diagnostic(
                                        &ident.name,
                                        nodes.get_node(r.node_id()).span(),
                                    );
                                    ctx.diagnostic(diagnostic);
                                }
                                LastWrite::Initialization(node_id) => {
                                    let diagnostic = no_useless_assignment_diagnostic(
                                        &ident.name,
                                        nodes.get_node(*node_id).span(),
                                    );
                                    ctx.diagnostic(diagnostic);
                                }
                                LastWrite::None => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn handle_single_block_references<'a>(
    references: &Vec<&'a Reference>,
    last_write: LastWrite,
) -> LastWrite {
    let mut local_last_write: LastWrite = last_write;

    for reference in references {
        if reference.is_write() {
            match &local_last_write {
                LastWrite::Reference(_) => return local_last_write,
                LastWrite::Initialization(_) => return local_last_write,
                LastWrite::None => {}
            }
            local_last_write = LastWrite::Reference((*reference).clone());
        } else if reference.is_read() {
            local_last_write = LastWrite::None;
        }
    }

    local_last_write
}

fn handle_multi_block_references<'a>(
    ident: &BindingIdentifier,
    references: &Vec<&'a Reference>,
    ctx: &LintContext,
    last_write: LastWrite,
    visited_blocks: &mut FxHashSet<BlockNodeId>,
) {
    let nodes = ctx.nodes();
    let cfg = ctx.cfg();

    // Group references by their CFG block
    let block_references: FxHashMap<BlockNodeId, Vec<&Reference>> =
        references.iter().fold(FxHashMap::default(), |mut acc, r| {
            let block_id = nodes.cfg_id(r.node_id());
            acc.entry(block_id).or_default().push(*r);
            acc
        });

    let mut local_last_write: LastWrite = last_write;
    for (block_id, refs) in &block_references {
        if visited_blocks.contains(&block_id) {
            continue;
        }
        visited_blocks.insert(*block_id);

        local_last_write = handle_single_block_references(&refs, local_last_write.clone());
        // Process neighbor blocks
        let last_write_in_neighbors = cfg
            .graph
            .neighbors(*block_id)
            .map(|n| {
                if let Some(neighbor_refs) = block_references.get(&n) {
                    visited_blocks.insert(n);
                    handle_single_block_references(neighbor_refs, local_last_write.clone())
                } else {
                    local_last_write.clone()
                }
            })
            .any(|w| match &w {
                LastWrite::Reference(_) => true,
                LastWrite::Initialization(_) => true,
                LastWrite::None => false,
            });

        if last_write_in_neighbors {
            match &local_last_write {
                LastWrite::Reference(r) => {
                    let diagnostic = no_useless_assignment_diagnostic(
                        &ident.name,
                        nodes.get_node(r.node_id()).span(),
                    );
                    ctx.diagnostic(diagnostic);
                }
                LastWrite::Initialization(node_id) => {
                    let diagnostic = no_useless_assignment_diagnostic(
                        &ident.name,
                        nodes.get_node(*node_id).span(),
                    );
                    ctx.diagnostic(diagnostic);
                }
                LastWrite::None => {}
            }
        }

        local_last_write = LastWrite::None;
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
        //     (
        //         "function fn4() {
        //     let v = 'used';
        //     for (let i = 0; i < 10; i++) {
        //         doSomething(v);
        //         v = 'used in next iteration';
        //     }
        // }",
        //         None,
        //     ),
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
