use cow_utils::CowUtils;
use oxc_ast::{ast::MemberExpression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, NodeId, ReferenceId};
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashMap;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{collect_possible_jest_call_node, parse_jest_fn_call, PossibleJestNode},
};

fn no_global_set_timeout_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`jest.setTimeout` should be call in `global` scope").with_label(span)
}

fn no_multiple_set_timeouts_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not call `jest.setTimeout` multiple times")
        .with_help("Only the last call to `jest.setTimeout` will have an effect.")
        .with_label(span)
}

fn no_unorder_set_timeout_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`jest.setTimeout` should be placed before any other jest methods")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoConfusingSetTimeout;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow confusing usages of jest.setTimeout
    ///
    /// ### Why is this bad?
    ///
    /// - being called anywhere other than in global scope
    /// - being called multiple times
    /// - being called after other Jest functions like hooks, `describe`, `test`, or `it`
    ///
    ///
    /// ### Example
    ///
    /// All of these are invalid case:
    /// ```javascript
    /// escribe('test foo', () => {
    ///   jest.setTimeout(1000);
    ///   it('test-description', () => {
    ///     // test logic;
    ///   });
    /// });
    ///
    /// describe('test bar', () => {
    ///   it('test-description', () => {
    ///     jest.setTimeout(1000);
    ///     // test logic;
    ///   });
    /// });
    ///
    /// test('foo-bar', () => {
    ///   jest.setTimeout(1000);
    /// });
    ///
    /// describe('unit test', () => {
    ///   beforeEach(() => {
    ///     jest.setTimeout(1000);
    ///   });
    /// });
    /// ```
    NoConfusingSetTimeout,
    jest,
    style
);

impl Rule for NoConfusingSetTimeout {
    fn run_once(&self, ctx: &LintContext) {
        let scopes = ctx.scopes();
        let symbol_table = ctx.symbols();
        let possible_nodes = collect_possible_jest_call_node(ctx);
        let id_to_jest_node_map =
            possible_nodes.iter().fold(FxHashMap::default(), |mut acc, cur| {
                acc.insert(cur.node.id(), cur);
                acc
            });

        let mut jest_reference_id_list: Vec<(ReferenceId, Span)> = vec![];
        let mut seen_jest_set_timeout = false;

        for reference_ids in scopes.root_unresolved_references_ids() {
            collect_jest_reference_id(reference_ids, &mut jest_reference_id_list, ctx);
        }

        for reference_ids in symbol_table.resolved_references() {
            collect_jest_reference_id(
                reference_ids.iter().copied(),
                &mut jest_reference_id_list,
                ctx,
            );
        }

        for reference_id_list in scopes.root_unresolved_references_ids() {
            handle_jest_set_time_out(
                ctx,
                reference_id_list,
                &jest_reference_id_list,
                &mut seen_jest_set_timeout,
                &id_to_jest_node_map,
            );
        }

        for reference_id_list in symbol_table.resolved_references() {
            handle_jest_set_time_out(
                ctx,
                reference_id_list.iter().copied(),
                &jest_reference_id_list,
                &mut seen_jest_set_timeout,
                &id_to_jest_node_map,
            );
        }
    }
}

fn collect_jest_reference_id(
    reference_id_list: impl Iterator<Item = ReferenceId>,
    jest_reference_list: &mut Vec<(ReferenceId, Span)>,
    ctx: &LintContext,
) {
    let symbol_table = ctx.symbols();
    let nodes = ctx.nodes();

    for reference_id in reference_id_list {
        let reference = symbol_table.get_reference(reference_id);

        if !is_jest_call(ctx.semantic().reference_name(reference)) {
            continue;
        }
        let Some(parent_node) = nodes.parent_node(reference.node_id()) else {
            continue;
        };
        let AstKind::MemberExpression(member_expr) = parent_node.kind() else {
            continue;
        };
        jest_reference_list.push((reference_id, member_expr.span()));
    }
}

fn handle_jest_set_time_out<'a>(
    ctx: &LintContext<'a>,
    reference_id_list: impl Iterator<Item = ReferenceId>,
    jest_reference_id_list: &Vec<(ReferenceId, Span)>,
    seen_jest_set_timeout: &mut bool,
    id_to_jest_node_map: &FxHashMap<NodeId, &PossibleJestNode<'a, '_>>,
) {
    let nodes = ctx.nodes();
    let scopes = ctx.scopes();
    let symbol_table = ctx.symbols();

    for reference_id in reference_id_list {
        let reference = symbol_table.get_reference(reference_id);

        let Some(parent_node) = nodes.parent_node(reference.node_id()) else {
            continue;
        };

        if !is_jest_call(ctx.semantic().reference_name(reference)) {
            if is_jest_fn_call(parent_node, id_to_jest_node_map, ctx) {
                for (jest_reference_id, span) in jest_reference_id_list {
                    if jest_reference_id > &reference_id {
                        ctx.diagnostic(no_unorder_set_timeout_diagnostic(*span));
                    }
                }
            }
            continue;
        }

        let AstKind::MemberExpression(member_expr) = parent_node.kind() else {
            continue;
        };

        let MemberExpression::StaticMemberExpression(expr) = member_expr else {
            continue;
        };

        if expr.property.name == "setTimeout" {
            if !scopes.get_flags(parent_node.scope_id()).is_top() {
                ctx.diagnostic(no_global_set_timeout_diagnostic(member_expr.span()));
            }

            if *seen_jest_set_timeout {
                ctx.diagnostic(no_multiple_set_timeouts_diagnostic(member_expr.span()));
            } else {
                *seen_jest_set_timeout = true;
            }
        }
    }
}

fn is_jest_fn_call<'a>(
    parent_node: &AstNode<'a>,
    id_to_jest_node_map: &FxHashMap<NodeId, &PossibleJestNode<'a, '_>>,
    ctx: &LintContext<'a>,
) -> bool {
    let mut id = parent_node.id();
    loop {
        let parent = ctx.nodes().parent_node(id);
        if let Some(parent) = parent {
            let parent_kind = parent.kind();
            if matches!(
                parent_kind,
                AstKind::CallExpression(_)
                    | AstKind::MemberExpression(_)
                    | AstKind::TaggedTemplateExpression(_)
            ) {
                id = parent.id();
            } else {
                break;
            }
        } else {
            break;
        }
    }

    let Some(possible_jest_node) = id_to_jest_node_map.get(&id) else {
        return false;
    };
    let AstKind::CallExpression(call_expr) = parent_node.kind() else {
        return false;
    };
    parse_jest_fn_call(call_expr, possible_jest_node, ctx).is_some()
}

fn is_jest_call(name: &str) -> bool {
    // handle "jest" | "Jest" | "JEST" | "JEst" to "jest", For example:
    //
    // import { jest as Jest } from "@jest/globals";
    // Jest.setTimeout
    name.cow_to_ascii_lowercase().eq_ignore_ascii_case("jest")
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
                import { jest as Jest } from '@jest/globals';
                import ReactDom from 'react-dom';
                Jest.setTimeout(800);
                test('test', () => {
                    expect(1 + 2).toEqual(3);
                });
                setTimeout(800);
            ",
            None,
        ),
        (
            "
                jest.setTimeout(600);
                setTimeout(900);
            ",
            None,
        ),
        ("jest.setTimeout(1001);", None),
        (
            "
                jest.setTimeout(1002);
                describe('A', () => {
                    beforeEach(async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                    it('A.1', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                    it('A.2', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                });
            ",
            None,
        ),
        (
            "
                import { jest as Jest } from '@jest/globals';
                Jest.setTimeout(800);
                setTimeout(800);
            ",
            None,
        ),
        (
            "
                jest.setTimeout(1003);
                window.setTimeout(6000)
                describe('A', () => {
                    beforeEach(async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                    it('test foo', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                });
            ",
            None,
        ),
        (
            "
                import { handler } from 'dep/mod';
                jest.setTimeout(801);
                describe('A', () => {
                    beforeEach(async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                    it('A.1', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                    it('A.2', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                });
            ",
            None,
        ),
        (
            "
                function handler() {}
                jest.setTimeout(802);
                describe('A', () => {
                    beforeEach(async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                    it('A.1', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                    it('A.2', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                });
            ",
            None,
        ),
        (
            "
                const { handler } = require('dep/mod');
                jest.setTimeout(803);
                describe('A', () => {
                    beforeEach(async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                    it('A.1', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                    it('A.2', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                });
            ",
            None,
        ),
        (
            "
                jest.setTimeout(1004);
                window.setTimeout(60000);
            ",
            None,
        ),
        ("window.setTimeout(60000);", None),
        ("setTimeout(1005);", None),
        (
            "
                jest.setTimeout(1006);
                test('test case', () => {
                    setTimeout(() => {
                    Promise.resolv();
                    }, 5000);
                });
            ",
            None,
        ),
        (
            "
                test('test case', () => {
                    setTimeout(() => {
                        Promise.resolv();
                    }, 5000);
                });
            ",
            None,
        ),
    ];

    let fail = vec![
        (
            "
                jest.setTimeout(1000);
                setTimeout(1000);
                window.setTimeout(1000);
                describe('A', () => {
                    beforeEach(async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                    it('A.1', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                    it('A.2', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                });
                jest.setTimeout(800);
            ",
            None,
        ),
        (
            "
                describe('A', () => {
                    jest.setTimeout(800);
                    beforeEach(async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                    it('A.1', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                    it('A.2', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                });
            ",
            None,
        ),
        (
            "
                describe('B', () => {
                    it('B.1', async () => {
                        await new Promise((resolve) => {
                        jest.setTimeout(1000);
                        setTimeout(resolve, 10000).unref();
                        });
                    });
                    it('B.2', async () => {
                        await new Promise((resolve) => { setTimeout(resolve, 10000).unref(); });
                    });
                });
            ",
            None,
        ),
        (
            "
                test('test-suite', () => {
                    jest.setTimeout(1000);
                });
            ",
            None,
        ),
        (
            "
                describe('A', () => {
                    beforeEach(async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                    it('A.1', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                    it('A.2', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                });
                jest.setTimeout(1000);
            ",
            None,
        ),
        (
            "
                import { jest } from '@jest/globals';
                {
                   jest.setTimeout(800);
                }
                describe('A', () => {
                    beforeEach(async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                    it('A.1', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                    it('A.2', async () => { await new Promise(resolve => { setTimeout(resolve, 10000).unref(); });});
                });
            ",
            None,
        ),
        (
            "
                jest.setTimeout(800);
                jest.setTimeout(900);
            ",
            None,
        ),
        (
            "
                import { jest as Jest } from '@jest/globals';
                {
                    Jest.setTimeout(800);
                }
            ",
            None,
        ),
        (
            "
                expect(1 + 2).toEqual(3);
                jest.setTimeout(1000);
            ",
            None,
        ),
        (
            "
                import { jest as Jest } from '@jest/globals';
                import ReactDom from 'react-dom';

                test('test', () => {
                    expect(1 + 2).toEqual(3);
                });
                Jest.setTimeout(800);
                setTimeout(800);
            ",
            None,
        ),
        (
            "
                    test('test-suite', () => {
                        expect(1 + 2).toEqual(3);
                    });
                    jest.setTimeout(1000);
                ",
            None,
        ),
    ];

    Tester::new(NoConfusingSetTimeout::NAME, NoConfusingSetTimeout::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
