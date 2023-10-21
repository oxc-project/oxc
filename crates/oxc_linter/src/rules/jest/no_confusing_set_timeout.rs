use oxc_ast::{
    ast::{CallExpression, Expression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, ReferenceId};
use oxc_span::{Atom, GetSpan, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{parse_jest_fn_call, JestFnKind},
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(no-confusing-set-timeout)")]
#[diagnostic(severity(warning), help("`jest.setTimeout` should be call in `global` scope"))]
struct NoGlobalSetTimeoutDiagnostic(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(no-confusing-set-timeout)")]
#[diagnostic(
    severity(warning),
    help(
        "Do not call `jest.setTimeout` multiple times, as only the last call will have an effect"
    )
)]
struct NoMultipleSetTimeoutsDiagnostic(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(no-confusing-set-timeout)")]
#[diagnostic(
    severity(warning),
    help("`jest.setTimeout` should be placed before any other jest methods")
)]
struct NoUnorderSetTimeoutDiagnostic(#[label] pub Span);

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
    restriction
);

impl Rule for NoConfusingSetTimeout {
    fn run_once(&self, ctx: &LintContext) {
        let scopes = ctx.scopes();
        let symbol_table = ctx.symbols();

        let mut jest_reference_id_list: Vec<(ReferenceId, Span)> = vec![];
        let mut seen_jest_set_timeout = false;

        for reference_ids in scopes.root_unresolved_references().values() {
            collect_jest_reference_id(reference_ids, &mut jest_reference_id_list, ctx);
        }

        for reference_ids in &symbol_table.resolved_references {
            collect_jest_reference_id(reference_ids, &mut jest_reference_id_list, ctx);
        }

        for reference_id_list in scopes.root_unresolved_references().values() {
            handle_jest_set_time_out(
                ctx,
                reference_id_list,
                &jest_reference_id_list,
                &mut seen_jest_set_timeout,
            );
        }

        for reference_id_list in &symbol_table.resolved_references {
            handle_jest_set_time_out(
                ctx,
                reference_id_list,
                &jest_reference_id_list,
                &mut seen_jest_set_timeout,
            );
        }
    }
}

fn collect_jest_reference_id(
    reference_id_list: &Vec<ReferenceId>,
    jest_reference_list: &mut Vec<(ReferenceId, Span)>,
    ctx: &LintContext,
) {
    let symbol_table = ctx.symbols();
    let nodes = ctx.nodes();

    for reference_id in reference_id_list {
        let reference = symbol_table.get_reference(*reference_id);
        if !is_jest_call(reference.name()) {
            continue;
        }
        let Some(parent_node) = nodes.parent_node(reference.node_id()) else {
            continue;
        };
        let AstKind::MemberExpression(member_expr) = parent_node.kind() else {
            continue;
        };
        jest_reference_list.push((*reference_id, member_expr.span()));
    }
}

fn handle_jest_set_time_out(
    ctx: &LintContext,
    reference_id_list: &Vec<ReferenceId>,
    jest_reference_id_list: &Vec<(ReferenceId, Span)>,
    seen_jest_set_timeout: &mut bool,
) {
    let nodes = ctx.nodes();
    let scopes = ctx.scopes();
    let symbol_table = ctx.symbols();

    for &reference_id in reference_id_list {
        let reference = symbol_table.get_reference(reference_id);

        let Some(parent_node) = nodes.parent_node(reference.node_id()) else {
            continue;
        };

        if !is_jest_call(reference.name()) {
            if is_jest_fn_call(parent_node, ctx) {
                for (jest_reference_id, span) in jest_reference_id_list {
                    if jest_reference_id > &reference_id {
                        ctx.diagnostic(NoUnorderSetTimeoutDiagnostic(*span));
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
                ctx.diagnostic(NoGlobalSetTimeoutDiagnostic(member_expr.span()));
            }

            if *seen_jest_set_timeout {
                ctx.diagnostic(NoMultipleSetTimeoutsDiagnostic(member_expr.span()));
            } else {
                *seen_jest_set_timeout = true;
            }
        }
    }
}

fn is_jest_fn_call<'a>(parent_node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let AstKind::CallExpression(call_expr) = parent_node.kind() else {
        return false;
    };
    if let Expression::Identifier(ident) = &call_expr.callee {
        if ident.name == "expect" {
            let Some(grand_node) = ctx.nodes().parent_node(parent_node.id()) else {
                return false;
            };
            let Some(grand_grand_node) = ctx.nodes().parent_node(grand_node.id()) else {
                return false;
            };
            return match_jest_fn_call(call_expr, grand_grand_node, ctx);
        }
    };

    match_jest_fn_call(call_expr, parent_node, ctx)
}

fn match_jest_fn_call<'a>(
    expr: &'a CallExpression<'a>,
    node: &AstNode<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    let Some(jest_fn_call) = parse_jest_fn_call(expr, node, ctx) else {
        return false;
    };
    match jest_fn_call.kind() {
        JestFnKind::Expect | JestFnKind::General(_) => true,
        JestFnKind::Unknown => false,
    }
}

fn is_jest_call(name: &Atom) -> bool {
    // handle "jest" | "Jest" | "JEST" | "JEst" to "jest", For example:
    //
    // import { jest as Jest } from "@jest/globals";
    // Jest.setTimeout
    name.to_ascii_lowercase().eq_ignore_ascii_case("jest")
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
            None
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

    Tester::new(NoConfusingSetTimeout::NAME, pass, fail).test_and_snapshot();
}
