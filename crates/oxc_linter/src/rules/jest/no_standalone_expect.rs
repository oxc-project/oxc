use oxc_ast::{
    ast::{CallExpression, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    jest_ast_util::{
        parse_expect_jest_fn_call, parse_general_jest_fn_call, JestFnKind, ParsedExpectFnCall, JestGeneralFnKind,
    },
    rule::Rule,
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("")]
#[diagnostic(severity(warning), help(""))]
struct NoStandaloneExpectDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoStandaloneExpect;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoStandaloneExpect,
    correctness
);

impl Rule for NoStandaloneExpect {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, node, ctx) else {
            return;
        };
        let ParsedExpectFnCall { head, members, .. } = jest_fn_call;

        // only report `expect.hasAssertions` & `expect.assertions` member calls
        if members.len() == 1
            && members[0].is_name_unequal("assertions")
            && members[0].is_name_unequal("hasAssertions")
        {
            if let Some(Expression::MemberExpression(_)) = head.parent {
                return;
            }
        }

        if let Some(_) = find_up(node, ctx) {
            ctx.diagnostic(NoStandaloneExpectDiagnostic(head.span));
        }
    }
}

fn find_up<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> Option<()> {
    let parent = ctx.nodes().parent_node(node.id())?;

    match parent.kind() {
        AstKind::BlockStatement(_)
        | AstKind::FunctionBody(_) => return find_up(node, ctx),
        AstKind::ArrowExpression(_) | AstKind::Function(_) => {
            let Some(grandparent) = ctx.nodes().parent_node(parent.id()) else {
                return None;
            };

            if matches!(grandparent.kind(), AstKind::CallExpression(_)) {
                return find_up(grandparent, ctx);
            };
        }
        AstKind::CallExpression(call_expr) => {
            let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, node, ctx) else {
                return None;
            };
            let JestFnKind::General(fn_kind) = jest_fn_call.kind else {
                return None;
            };
            if matches!(fn_kind, JestGeneralFnKind::Describe) {
                return Some(());
            }
            return None 
        }
        _ => {}
    }

    None
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("expect.any(String)", None),
        ("expect.extend({})", None),
        ("describe('a test', () => { it('an it', () => {expect(1).toBe(1); }); });", None),
        ("describe('a test', () => { it('an it', () => { const func = () => { expect(1).toBe(1); }; }); });", None),
        ("describe('a test', () => { const func = () => { expect(1).toBe(1); }; });", None),
        ("describe('a test', () => { function func() { expect(1).toBe(1); }; });", None),
        ("describe('a test', () => { const func = function(){ expect(1).toBe(1); }; });", None),
        ("it('an it', () => expect(1).toBe(1))", None),
        ("const func = function(){ expect(1).toBe(1); };", None),
        ("const func = () => expect(1).toBe(1);", None),
        ("{}", None),
        ("it.each([1, true])('trues', value => { expect(value).toBe(true); });", None),
        ("it.each([1, true])('trues', value => { expect(value).toBe(true); }); it('an it', () => { expect(1).toBe(1) });", None),
        (
            "
                it.each`
                    num   | value
                    ${1} | ${true}
                `('trues', ({ value }) => {
                    expect(value).toBe(true);
                });
            ", 
            None
        ),
        ("it.only('an only', value => { expect(value).toBe(true); });", None),
        ("it.concurrent('an concurrent', value => { expect(value).toBe(true); });", None),
        ("describe.each([1, true])('trues', value => { it('an it', () => expect(value).toBe(true) ); });", None),
        ("
            describe('scenario', () => {
                const t = Math.random() ? it.only : it;
                t('testing', () => expect(true));
            });
        ", Some(serde_json::json!([{ "additionalTestBlockFunctions": ['t'] }]))),
        (
            r#"
                each([
                [1, 1, 2],
                [1, 2, 3],
                [2, 1, 3],
                ]).test('returns the result of adding %d to %d', (a, b, expected) => {
                    expect(a + b).toBe(expected);
                });
            "#, Some(serde_json::json!([{ "additionalTestBlockFunctions": ["each.test"] }])))
        ];

    let fail = vec![
        ("(() => {})('testing', () => expect(true).toBe(false))", None),
        ("expect.hasAssertions()", None),
        ("expect().hasAssertions()", None),
        (
            "
                describe('scenario', () => {
                    const t = Math.random() ? it.only : it;
                    t('testing', () => expect(true).toBe(false));
                });
            ",
            None
        ),
        (
            "
                describe('scenario', () => {
                    const t = Math.random() ? it.only : it;
                    t('testing', () => expect(true).toBe(false));
                });
            ",
            Some(serde_json::json!([{ "additionalTestBlockFunctions": "undefined" }]))
        ),
        (
            "
                each([
                    [1, 1, 2],
                    [1, 2, 3],
                    [2, 1, 3],
                ]).test('returns the result of adding %d to %d', (a, b, expected) => {
                    expect(a + b).toBe(expected);
                });
            ", None),
        (
            "
                each([
                    [1, 1, 2],
                    [1, 2, 3],
                    [2, 1, 3],
                ]).test('returns the result of adding %d to %d', (a, b, expected) => {
                    expect(a + b).toBe(expected);
                });
            ", 
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ["each"] }]))
        ),
        (
            "
                each([
                    [1, 1, 2],
                    [1, 2, 3],
                    [2, 1, 3],
                ]).test('returns the result of adding %d to %d', (a, b, expected) => {
                    expect(a + b).toBe(expected);
                });
            ", 
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ["test"] }]))
        ),
        ("describe('a test', () => { expect(1).toBe(1); });", None),
        ("describe('a test', () => expect(1).toBe(1));", None),
        ("describe('a test', () => { const func = () => { expect(1).toBe(1); }; expect(1).toBe(1); });", None),
        ("describe('a test', () => {  it(() => { expect(1).toBe(1); }); expect(1).toBe(1); });", None),
        ("expect(1).toBe(1);", None),
        ("{expect(1).toBe(1)}", None),
        ("it.each([1, true])('trues', value => { expect(value).toBe(true); }); expect(1).toBe(1);", None),
        ("describe.each([1, true])('trues', value => { expect(value).toBe(true); });", None),
        (
            "
                import { expect as pleaseExpect } from '@jest/globals';
                describe('a test', () => { pleaseExpect(1).toBe(1); });
            ", 
            None
        ),
        (
            "
                import { expect as pleaseExpect } from '@jest/globals';
                beforeEach(() => pleaseExpect.hasAssertions());
            ",
            None
        )
    ];

    Tester::new(NoStandaloneExpect::NAME, pass, fail).test_and_snapshot();
}
