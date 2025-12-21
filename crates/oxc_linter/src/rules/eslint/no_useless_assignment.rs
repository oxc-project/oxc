use oxc_ast::{AstKind, ast::BindingIdentifier};
use oxc_cfg::{ControlFlowGraph, graph::graph::NodeIndex};
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

        check_write_usage(
            ident,
            references,
            ReferenceIndex::Init,
            init_cfg_id,
            decl_node.span(),
            ctx,
            cfg,
        );
    }

    for (i, reference) in references.iter().enumerate() {
        if !reference.is_write() {
            continue;
        }

        let write_cfg_id = nodes.cfg_id(reference.node_id());
        let assignment_node = nodes.parent_node(reference.node_id());
        let write_span = assignment_node.span();

        check_write_usage(
            ident,
            references,
            ReferenceIndex::NonInit(i),
            write_cfg_id,
            write_span,
            ctx,
            cfg,
        );
    }
}

enum ReferenceIndex {
    NonInit(usize),
    Init,
}

fn check_write_usage(
    ident: &BindingIdentifier,
    references: &[&Reference],
    idx: ReferenceIndex,
    write_cfg_id: NodeIndex,
    write_span: Span,
    ctx: &LintContext,
    cfg: &ControlFlowGraph,
) {
    let nodes = ctx.nodes();
    let references_after = match idx {
        ReferenceIndex::NonInit(i) => &references[i + 1..],
        ReferenceIndex::Init => references,
    };

    let mut assignment_used = false;
    let mut last_reachable_write: Option<&Reference> = None;
    for r in references_after.iter() {
        if r.is_write() && cfg.is_reachable(write_cfg_id, nodes.cfg_id(r.node_id())) {
            last_reachable_write = Some(r);
            continue;
        }
        if r.is_read() && cfg.is_reachable(write_cfg_id, nodes.cfg_id(r.node_id())) {
            if last_reachable_write.is_none() {
                assignment_used = true;
                break;
            }

            if let Some(last_reachable_write) = last_reachable_write {
                if !cfg.is_reachable(
                    nodes.cfg_id(last_reachable_write.node_id()),
                    nodes.cfg_id(r.node_id()),
                ) {
                    assignment_used = true;
                    break;
                }
            }
        }
    }

    if !assignment_used {
        let references_before = match idx {
            ReferenceIndex::NonInit(i) => Some(&references[..i]),
            ReferenceIndex::Init => None,
        };

        if let Some(references_before) = references_before {
            if cfg.is_cyclic(write_cfg_id) {
                for r in references_before.iter() {
                    if r.is_write() {
                        continue;
                    }
                    if cfg.is_reachable(write_cfg_id, nodes.cfg_id(r.node_id())) {
                        assignment_used = true;
                        break;
                    }
                }
            }
        }
    }

    if !assignment_used {
        ctx.diagnostic(no_useless_assignment_diagnostic(&ident.name, write_span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "let x = 5; console.log(x);",
        "function fn1() {
            let v = 'used';
            doSomething(v);
            v = 'used-2';
            doSomething(v);
        }",
        "function fn2() {
            let v = 'used';
            if (condition) {
                v = 'used-2';
                doSomething(v);
                return
            }
            doSomething(v);
        }",
        "function fn3() {
            let v = 'used';
            if (condition) {
                doSomething(v);
            } else {
                v = 'used-2';
                doSomething(v);
            }
        }",
        "function fn4() {
            let v = 'used';
            for (let i = 0; i < 10; i++) {
                doSomething(v);
                v = 'used in next iteration';
            }
        }",
        "function fn5() {
            let x;
            let result = (x = 5) > 3 ? 'yes' : 'no';
            console.log(x);
        }",
        "function fn6() {
            let result = 'initial';
            try {
                doSomething(result);
                result = 'success';
                doSomething(result);
            } catch (e) {
                // empty
            }
        }",
        "function fn7() {
            let v = 'initial';
            switch (x) {
                case 1:
                    console.log(v);
                    break;
                case 2:
                    console.log(v);
                    break;
                default:
                    v = 'default';
                    console.log(v);
            }
        }",
        "function fn8() {
            let i = 0;
            while (i < 10) {
                console.log(i);
                i = i + 1;
            }
        }",
        "function fn9() {
            let x = 1;
            let result = x || 5;
            console.log(result);
        }",
    ];
    let fail = vec![
        "let x = 5; x = 7;",
        "function fn1() {
            let v = 'used';
            doSomething(v);
            v = 'unused';
        }",
        "function fn2() {
            let v = 'used';
            if (condition) {
                v = 'unused';
                return
            }
            doSomething(v);
        }",
        "function fn3() {
            let v = 'used';
            if (condition) {
                doSomething(v);
            } else {
                v = 'unused';
            }
        }",
        "function fn4() {
            let v = 'unused';
            if (condition) {
                v = 'used';
                doSomething(v);
                return
            }
        }",
        "function fn5() {
            let v = 'used';
            if (condition) {
                let v = 'used';
                console.log(v);
                v = 'unused';
            }
            console.log(v);
        }",
        "function fn6() {
            let x = 1;
            x = 2;
            x = 3;
        }",
        "function fn7() {
            let result = 'unused';
            try {
                result = 'error';
            } catch (e) {
                // empty
            }
        }",
        "function fn8() {
            let v = 'initial';
            switch (x) {
                case 1:
                    console.log(v);
                    break;
                case 2:
                    v = 'unused';
                    break;
            }
        }",
        "function fn9() {
            let x = 1;
            function inner() {
                x = 2;
                console.log(x);
            }
            inner();
        }",
        "function fn10() {
            let result = 'initial';
            doSomething(result);
            if (condition) {
                result = 'unused';
                return;
            }
        }",
    ];

    Tester::new(NoUselessAssignment::NAME, NoUselessAssignment::PLUGIN, pass, fail)
        .test_and_snapshot();
}
