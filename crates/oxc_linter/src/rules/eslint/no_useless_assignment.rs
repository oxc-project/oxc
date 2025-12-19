use oxc_ast::{AstKind, ast::BindingIdentifier};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::Reference;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_useless_assignment_diagnostic(name: &str, first_write: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Useless assignment to variable {name}"))
        .with_labels([first_write.label(format!("{name} is written to here but never read"))])
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
  /// “Dead stores” waste processing and memory, so it is better to remove unnecessary assignments to variables.
  /// Also, if the author intended the variable to be used, there is likely a mistake around the dead store.
  ///
  /// ### Examples
  ///
  /// Examples of **incorrect** code for this rule:
  /// ```js
  /// let x = 5;
  /// x = 7; // 'x' is assigned a value but never used
  ///
  /// // --------------------------------------------------
  ///
  /// function fn1() {
  ///   let v = 'used';
  ///   doSomething(v);
  ///   v = 'unused'; // 'v' is assigned a value but never used
  /// }
  /// ```
  ///
  /// Additional examples of **incorrect** code for this rule:
  /// ```js
  /// function fn2() {
  ///   let v = 'used';
  ///   if (condition) {
  ///     v = 'unused'; // 'v' is assigned a value but never used
  ///     return
  ///   }
  ///   doSomething(v);
  /// }
  ///
  /// // --------------------------------------------------
  ///
  /// function fn3() {
  ///   let v = 'used';
  ///   if (condition) {
  ///     doSomething(v);
  ///   } else {
  ///     v = 'unused'; // 'v' is assigned a value but never used
  ///   }
  /// }
  /// ```
  ///
  /// Examples of **correct** code for this rule:
  /// ```js
  /// let x = 5;
  /// console.log(x); // 'x' is used here
  ///
  /// // --------------------------------------------------
  ///
  /// function fn1() {
  ///   let v = 'used';
  ///   doSomething(v);
  ///   v = 'used-2';
  ///   doSomething(v); // 'v' is used here
  /// }
  ///
  /// // --------------------------------------------------
  ///
  /// function fn2() {
  ///   let v = 'used';
  ///   if (condition) {
  ///     v = 'used-2';
  ///     doSomething(v); // 'v' is used here
  ///     return
  ///   }
  ///   doSomething(v); // 'v' is used here
  /// }
  /// ```
  NoUselessAssignment,
  eslint,
  correctness
);

impl Rule for NoUselessAssignment {
    fn run_once(&self, ctx: &LintContext) {
        let cfg = match ctx.semantic().cfg() {
            Some(cfg) => cfg,
            None => return,
        };

        let nodes = ctx.nodes();
        let scoping = ctx.scoping();

        for node in nodes {
            match node.kind() {
                AstKind::VariableDeclarator(_) => {
                    let declarator = node.kind().as_variable_declarator().unwrap();
                    let Some(ident) = declarator.id.get_binding_identifier() else {
                        continue;
                    };

                    let symbol_id = ident.symbol_id();
                    let references: Vec<&Reference> =
                        scoping.get_resolved_references(symbol_id).collect();

                    if references.is_empty() {
                        continue;
                    }

                    analyze_variable_references(ident, &references, node, ctx, cfg);
                }
                _ => continue,
            }
        }
    }
}

fn analyze_variable_references(
    ident: &BindingIdentifier,
    references: &[&Reference],
    decl_node: &AstNode,
    ctx: &LintContext,
    cfg: &oxc_cfg::ControlFlowGraph,
) {
    let nodes = ctx.nodes();

    let has_init =
        decl_node.kind().as_variable_declarator().and_then(|d| d.init.as_ref()).is_some();

    if has_init {
        let init_cfg_id = nodes.cfg_id(decl_node.id());

        /*
         * Initialization is used if:
         *   - There exists a read reachable from init, AND
         *   - That read is in a different CFG block than any writes between init and read
         *     OR there are no writes reachable from init
         */

        let mut init_value_is_used = false;

        let writes_after_init: Vec<_> = references
            .iter()
            .filter(|r| r.is_write() && cfg.is_reachable(init_cfg_id, nodes.cfg_id(r.node_id())))
            .collect();

        for r in references.iter() {
            if r.is_write() {
                continue;
            }

            let read_cfg_id = nodes.cfg_id(r.node_id());
            let read_span = nodes.get_node(r.node_id()).span();

            if !cfg.is_reachable(init_cfg_id, read_cfg_id) {
                continue;
            }

            let mut init_is_used_for_read = true;
            for w in &writes_after_init {
                let write_cfg_id = nodes.cfg_id(w.node_id());
                let write_span = nodes.get_node(w.node_id()).span();

                if cfg.is_reachable(write_cfg_id, read_cfg_id) && write_span.start < read_span.start
                {
                    init_is_used_for_read = false;
                    break;
                }
            }

            if init_is_used_for_read {
                init_value_is_used = true;
                break;
            }
        }

        if !init_value_is_used {
            ctx.diagnostic(no_useless_assignment_diagnostic(&ident.name, decl_node.span()));
        }
    }

    /*
     * Non-initial writes are used if:
     *   - There exists a read reachable from the write
     */
    for (i, reference) in references.iter().enumerate() {
        if !reference.is_write() {
            continue;
        }

        let write_cfg_id = nodes.cfg_id(reference.node_id());
        let write_span = nodes.get_node(reference.node_id()).span();

        let mut has_read_after = false;
        for r in references[i + 1..].iter() {
            if r.is_write() {
                break;
            }
            if cfg.is_reachable(write_cfg_id, nodes.cfg_id(r.node_id())) {
                has_read_after = true;
                break;
            }
        }

        if !has_read_after {
            if cfg.is_cyclic(write_cfg_id) {
                println!("Node is cyclic {:?}", nodes.get_node(reference.node_id()));
                for r in references[..i].iter() {
                    if r.is_write() {
                        continue;
                    }
                    if cfg.is_reachable(write_cfg_id, nodes.cfg_id(r.node_id())) {
                        has_read_after = true;
                        break;
                    }
                }
            }
        }

        if !has_read_after {
            ctx.diagnostic(no_useless_assignment_diagnostic(&ident.name, write_span));
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
