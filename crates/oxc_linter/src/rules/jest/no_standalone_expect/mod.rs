use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{CompactStr, Span};
use rustc_hash::FxHashMap;

use crate::{
    AstNode,
    context::LintContext,
    rule::Rule,
    utils::{
        JestFnKind, JestGeneralFnKind, KnownMemberExpressionParentKind, ParsedExpectFnCall,
        PossibleJestNode, collect_possible_jest_call_node, get_node_name,
        parse_expect_jest_fn_call, parse_general_jest_fn_call,
    },
};

fn no_standalone_expect_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expect must be inside of a test block.")
        .with_help("Did you forget to wrap `expect` in a `test` or `it` block?")
        .with_label(span)
}

/// <https://github.com/jest-community/eslint-plugin-jest/blob/v28.9.0/docs/rules/no-standalone-expect.md>
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
    /// ### Why is this bad?
    ///
    /// `expect` statements outside of test blocks will not be executed by the Jest
    /// test runner, which means they won't actually test anything. This can lead to
    /// false confidence in test coverage and may hide bugs that would otherwise be
    /// caught by proper testing.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// describe('a test', () => {
    ///     expect(1).toBe(1);
    /// });
    /// ```
    NoStandaloneExpect,
    jest,
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
    let mut parent = ctx.nodes().parent_node(node.id());

    // loop until find the closest function body
    while !matches!(parent.kind(), AstKind::FunctionBody(_) | AstKind::Program(_)) {
        parent = ctx.nodes().parent_node(parent.id());
    }

    let parent = ctx.nodes().parent_node(parent.id());

    match parent.kind() {
        AstKind::Function(function) => {
            // `function foo() { expect(1).toBe(1); }`
            if function.is_function_declaration() {
                return Some(());
            }

            if function.is_expression() {
                let grandparent = ctx.nodes().parent_node(parent.id());

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
            let grandparent = ctx.nodes().parent_node(parent.id());
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
            if additional_test_block_functions.contains(&node_name) {
                return true;
            }
        }
        AstKind::ArrayExpression(_) | AstKind::ObjectExpression(_) => {
            let mut current = node;
            loop {
                let parent = ctx.nodes().parent_node(current.id());
                match parent.kind() {
                    AstKind::CallExpression(_) | AstKind::VariableDeclarator(_) => {
                        return is_var_declarator_or_test_block(
                            parent,
                            additional_test_block_functions,
                            id_nodes_mapping,
                            ctx,
                        );
                    }
                    AstKind::ArrayExpression(_) | AstKind::ObjectExpression(_) => {
                        current = parent;
                    }
                    _ => break,
                }
            }
        }
        _ => {}
    }

    false
}
