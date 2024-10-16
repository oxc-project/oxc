use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{CompactStr, Span};
use rustc_hash::FxHashMap;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, get_node_name, parse_expect_jest_fn_call,
        parse_general_jest_fn_call, JestFnKind, JestGeneralFnKind, KnownMemberExpressionParentKind,
        ParsedExpectFnCall, PossibleJestNode,
    },
    AstNode,
};

fn no_standalone_expect_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expect must be inside of a test block.")
        .with_help("Did you forget to wrap `expect` in a `test` or `it` block?")
        .with_label(span)
}

/// <https://github.com/jest-community/eslint-plugin-jest/blob/main/docs/rules/no-standalone-expect.md>
#[derive(Debug, Default, Clone)]
pub struct NoStandaloneExpect(Box<NoStandaloneExpectConfig>);

#[derive(Debug, Default, Clone)]
pub struct NoStandaloneExpectConfig {
    additional_test_block_functions: Vec<CompactStr>,
}

impl std::ops::Deref for NoStandaloneExpect {
    type Target = NoStandaloneExpectConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents `expect` statements outside of a `test` or `it` block. An `expect`
    /// within a helper function (but outside of a `test` or `it` block) will not
    /// trigger this rule.
    ///
    /// Statements like `expect.hasAssertions()` will NOT trigger this rule since these
    /// calls will execute if they are not in a test block.
    ///
    /// ### Example
    /// ```javascript
    /// describe('a test', () => {
    ///     expect(1).toBe(1);
    /// });
    /// ```
    NoStandaloneExpect,
    correctness
);

impl Rule for NoStandaloneExpect {
    fn from_configuration(value: serde_json::Value) -> Self {
        let additional_test_block_functions = value
            .get(0)
            .and_then(|v| v.get("additionalTestBlockFunctions"))
            .and_then(serde_json::Value::as_array)
            .map(|v| v.iter().filter_map(serde_json::Value::as_str).map(CompactStr::from).collect())
            .unwrap_or_default();

        Self(Box::new(NoStandaloneExpectConfig { additional_test_block_functions }))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let possible_jest_nodes = collect_possible_jest_call_node(ctx);
        let id_nodes_mapping =
            possible_jest_nodes.iter().fold(FxHashMap::default(), |mut acc, cur| {
                acc.entry(cur.node.id()).or_insert(cur);
                acc
            });

        for possible_jest_node in &possible_jest_nodes {
            self.run(possible_jest_node, &id_nodes_mapping, ctx);
        }
    }
}

impl NoStandaloneExpect {
    fn run<'a>(
        &self,
        possible_jest_node: &PossibleJestNode<'a, '_>,
        id_nodes_mapping: &FxHashMap<NodeId, &PossibleJestNode<'a, '_>>,
        ctx: &LintContext<'a>,
    ) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };
        let ParsedExpectFnCall { head, members, .. } = jest_fn_call;

        // only report `expect.hasAssertions` & `expect.assertions` member calls
        if members.len() == 1
            && members[0].is_name_unequal("assertions")
            && members[0].is_name_unequal("hasAssertions")
            && matches!(head.parent_kind, Some(KnownMemberExpressionParentKind::Member))
        {
            return;
        }

        if is_correct_place_to_call_expect(
            node,
            &self.additional_test_block_functions,
            id_nodes_mapping,
            ctx,
        )
        .is_none()
        {
            ctx.diagnostic(no_standalone_expect_diagnostic(head.span));
        }
    }
}

fn is_correct_place_to_call_expect<'a>(
    node: &AstNode<'a>,
    additional_test_block_functions: &[CompactStr],
    id_nodes_mapping: &FxHashMap<NodeId, &PossibleJestNode<'a, '_>>,
    ctx: &LintContext<'a>,
) -> Option<()> {
    let mut parent = ctx.nodes().parent_node(node.id())?;

    // loop until find the closest function body
    loop {
        match parent.kind() {
            AstKind::FunctionBody(_) => {
                break;
            }
            _ => {
                parent = ctx.nodes().parent_node(parent.id())?;
            }
        }
    }

    let node = parent;
    let parent = ctx.nodes().parent_node(node.id())?;

    match parent.kind() {
        AstKind::Function(function) => {
            // `function foo() { expect(1).toBe(1); }`
            if function.is_function_declaration() {
                return Some(());
            }

            if function.is_expression() {
                let grandparent = ctx.nodes().parent_node(parent.id())?;

                // `test('foo', function () { expect(1).toBe(1) })`
                // `const foo = function() {expect(1).toBe(1)}`
                return if is_var_declarator_or_test_block(
                    grandparent,
                    additional_test_block_functions,
                    id_nodes_mapping,
                    ctx,
                ) {
                    Some(())
                } else {
                    None
                };
            }
        }
        AstKind::ArrowFunctionExpression(_) => {
            let grandparent = ctx.nodes().parent_node(parent.id())?;
            // `test('foo', () => expect(1).toBe(1))`
            // `const foo = () => expect(1).toBe(1)`
            return if is_var_declarator_or_test_block(
                grandparent,
                additional_test_block_functions,
                id_nodes_mapping,
                ctx,
            ) {
                Some(())
            } else {
                None
            };
        }
        _ => {}
    }

    None
}

fn is_var_declarator_or_test_block<'a>(
    node: &AstNode<'a>,
    additional_test_block_functions: &[CompactStr],
    id_nodes_mapping: &FxHashMap<NodeId, &PossibleJestNode<'a, '_>>,
    ctx: &LintContext<'a>,
) -> bool {
    match node.kind() {
        AstKind::VariableDeclarator(_) => return true,
        AstKind::CallExpression(call_expr) => {
            if let Some(jest_node) = id_nodes_mapping.get(&node.id()) {
                if let Some(jest_fn_call) = parse_general_jest_fn_call(call_expr, jest_node, ctx) {
                    return matches!(
                        jest_fn_call.kind,
                        JestFnKind::General(JestGeneralFnKind::Test)
                    );
                }
            }

            let node_name = get_node_name(&call_expr.callee);
            if additional_test_block_functions.iter().any(|fn_name| node_name == fn_name) {
                return true;
            }
        }
        AstKind::Argument(_) => {
            if let Some(parent) = ctx.nodes().parent_node(node.id()) {
                return is_var_declarator_or_test_block(
                    parent,
                    additional_test_block_functions,
                    id_nodes_mapping,
                    ctx,
                );
            }
        }
        _ => {}
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("expect.any(String)", None),
        ("expect.extend({})", None),
        ("describe('a test', () => { it('an it', () => {expect(1).toBe(1); }); });", None),
        (
            "describe('a test', () => { it('an it', () => { const func = () => { expect(1).toBe(1); }; }); });",
            None,
        ),
        ("describe('a test', () => { const func = () => { expect(1).toBe(1); }; });", None),
        ("describe('a test', () => { function func() { expect(1).toBe(1); }; });", None),
        ("describe('a test', () => { const func = function(){ expect(1).toBe(1); }; });", None),
        ("it('an it', () => expect(1).toBe(1))", None),
        ("const func = function(){ expect(1).toBe(1); };", None),
        ("const func = () => expect(1).toBe(1);", None),
        ("{}", None),
        ("it.each([1, true])('trues', value => { expect(value).toBe(true); });", None),
        (
            "it.each([1, true])('trues', value => { expect(value).toBe(true); }); it('an it', () => { expect(1).toBe(1) });",
            None,
        ),
        (
            "
                it.each`
                    num   | value
                    ${1} | ${true}
                `('trues', ({ value }) => {
                    expect(value).toBe(true);
                });
            ",
            None,
        ),
        ("it.only('an only', value => { expect(value).toBe(true); });", None),
        ("it.concurrent('an concurrent', value => { expect(value).toBe(true); });", None),
        (
            "describe.each([1, true])('trues', value => { it('an it', () => expect(value).toBe(true) ); });",
            None,
        ),
        (
            "
            describe('scenario', () => {
                const t = Math.random() ? it.only : it;
                t('testing', () => expect(true));
            });
        ",
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ['t'] }])),
        ),
        (
            r"
                each([
                [1, 1, 2],
                [1, 2, 3],
                [2, 1, 3],
                ]).test('returns the result of adding %d to %d', (a, b, expected) => {
                    expect(a + b).toBe(expected);
                });
            ",
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ["each.test"] }])),
        ),
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
            None,
        ),
        (
            "
                describe('scenario', () => {
                    const t = Math.random() ? it.only : it;
                    t('testing', () => expect(true).toBe(false));
                });
            ",
            None,
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
            None,
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
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ["each"] }])),
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
            Some(serde_json::json!([{ "additionalTestBlockFunctions": ["test"] }])),
        ),
        ("describe('a test', () => { expect(1).toBe(1); });", None),
        ("describe('a test', () => expect(1).toBe(1));", None),
        (
            "describe('a test', () => { const func = () => { expect(1).toBe(1); }; expect(1).toBe(1); });",
            None,
        ),
        (
            "describe('a test', () => {  it(() => { expect(1).toBe(1); }); expect(1).toBe(1); });",
            None,
        ),
        ("expect(1).toBe(1);", None),
        ("{expect(1).toBe(1)}", None),
        (
            "it.each([1, true])('trues', value => { expect(value).toBe(true); }); expect(1).toBe(1);",
            None,
        ),
        ("describe.each([1, true])('trues', value => { expect(value).toBe(true); });", None),
        (
            "
                import { expect as pleaseExpect } from '@jest/globals';
                describe('a test', () => { pleaseExpect(1).toBe(1); });
            ",
            None,
        ),
        (
            "
                import { expect as pleaseExpect } from '@jest/globals';
                beforeEach(() => pleaseExpect.hasAssertions());
            ",
            None,
        ),
    ];

    Tester::new(NoStandaloneExpect::NAME, pass, fail).with_jest_plugin(true).test_and_snapshot();
}
