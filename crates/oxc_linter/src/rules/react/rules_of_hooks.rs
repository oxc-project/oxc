use std::borrow::Cow;

use oxc_ast::{
    ast::{ArrowFunctionExpression, Function},
    AstKind,
};
use oxc_cfg::{
    graph::{algo, visit::Control},
    ControlFlowGraph, EdgeType, ErrorEdgeKind, InstructionKind,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNodes, NodeId};
use oxc_syntax::operator::AssignmentOperator;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{is_react_component_or_hook_name, is_react_function_call, is_react_hook},
    AstNode,
};

mod diagnostics {
    use oxc_diagnostics::OxcDiagnostic;
    use oxc_span::Span;
    const SCOPE: &str = "eslint-plugin-react-hooks";

    pub(super) fn function_error(span: Span, hook_name: &str, func_name: &str) -> OxcDiagnostic {
        OxcDiagnostic::warn(format!(
            "React Hook {hook_name:?} is called in function {func_name:?} that is neither \
            a React function component nor a custom React Hook function. \
            React component names must start with an uppercase letter. \
            React Hook names must start with the word \"use\".",
        ))
        .with_label(span)
        .with_error_code_scope(SCOPE)
    }

    pub(super) fn conditional_hook(span: Span, hook_name: &str) -> OxcDiagnostic {
        OxcDiagnostic::warn(format!(
            "React Hook {hook_name:?} is called conditionally. React Hooks must be \
            called in the exact same order in every component render."
        ))
        .with_label(span)
        .with_error_code_scope(SCOPE)
    }

    pub(super) fn loop_hook(span: Span, hook_name: &str) -> OxcDiagnostic {
        OxcDiagnostic::warn(format!(
            "React Hook {hook_name:?} may be executed more than once. Possibly \
            because it is called in a loop. React Hooks must be called in the \
            exact same order in every component render."
        ))
        .with_label(span)
        .with_error_code_scope(SCOPE)
    }

    pub(super) fn top_level_hook(span: Span, hook_name: &str) -> OxcDiagnostic {
        OxcDiagnostic::warn(format!(
            "React Hook {hook_name:?} cannot be called at the top level. React Hooks \
            must be called in a React function component or a custom React \
            Hook function."
        ))
        .with_label(span)
        .with_error_code_scope(SCOPE)
    }

    pub(super) fn async_component(span: Span, func_name: &str) -> OxcDiagnostic {
        OxcDiagnostic::warn(format!(
            "message: `React Hook {func_name:?} cannot be called in an async function. "
        ))
        .with_label(span)
        .with_error_code_scope(SCOPE)
    }

    pub(super) fn class_component(span: Span, hook_name: &str) -> OxcDiagnostic {
        OxcDiagnostic::warn(format!(
            "React Hook {hook_name:?} cannot be called in a class component. React Hooks \
            must be called in a React function component or a custom React \
            Hook function."
        ))
        .with_label(span)
        .with_error_code_scope(SCOPE)
    }

    pub(super) fn generic_error(span: Span, hook_name: &str) -> OxcDiagnostic {
        OxcDiagnostic::warn(format!(
            "React Hook {hook_name:?} cannot be called inside a callback. React Hooks \
            must be called in a React function component or a custom React \
            Hook function."
        ))
        .with_label(span)
        .with_error_code_scope(SCOPE)
    }
}

#[derive(Debug, Default, Clone)]
pub struct RulesOfHooks;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This enforces the Rules of Hooks
    ///
    /// <https://reactjs.org/docs/hooks-rules.html>
    ///
    RulesOfHooks,
    react,
    pedantic
);

impl Rule for RulesOfHooks {
    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        // disable this rule in vue/nuxt and svelte(kit) files
        // react hook can be build in only `.ts` files,
        // but `useX` functions are popular and can be false positive in other frameworks
        !ctx.file_path().extension().is_some_and(|ext| ext == "vue" || ext == "svelte")
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call) = node.kind() else { return };

        if !is_react_hook(&call.callee) {
            return;
        }

        let cfg = ctx.cfg();

        let span = call.span;
        let hook_name =
            call.callee_name().expect("We identify hooks using their names so it should be named.");

        let semantic = ctx.semantic();
        let nodes = semantic.nodes();

        let is_use = is_react_function_call(call, "use");

        let Some(parent_func) = parent_func(nodes, node) else {
            return ctx.diagnostic(diagnostics::top_level_hook(span, hook_name));
        };

        // Check if our parent function is part of a class.
        if matches!(
            nodes.parent_kind(parent_func.id()),
            Some(
                AstKind::MethodDefinition(_)
                    | AstKind::StaticBlock(_)
                    | AstKind::PropertyDefinition(_)
            )
        ) {
            return ctx.diagnostic(diagnostics::class_component(span, hook_name));
        }

        match parent_func.kind() {
            // We are in a named function that isn't a hook or component, which is illegal
            AstKind::Function(Function { id: Some(id), .. })
                if !is_react_component_or_hook_name(&id.name) =>
            {
                return ctx.diagnostic(diagnostics::function_error(
                    id.span,
                    hook_name,
                    id.name.as_str(),
                ));
            }
            // Hooks are allowed inside of unnamed functions used as arguments. As long as they are
            // not used as a callback inside of components or hooks.
            AstKind::Function(Function { id: None, .. }) | AstKind::ArrowFunctionExpression(_)
                if is_non_react_func_arg(nodes, parent_func.id()) =>
            {
                // This rule doesn't apply to `use(...)`.
                if !is_use && is_somewhere_inside_component_or_hook(nodes, parent_func.id()) {
                    ctx.diagnostic(diagnostics::generic_error(span, hook_name));
                }
                return;
            }
            AstKind::Function(Function { span, id: None, .. })
            | AstKind::ArrowFunctionExpression(ArrowFunctionExpression {
                span,
                r#async: false,
                ..
            }) => {
                let ident = get_declaration_identifier(nodes, parent_func.id());

                // Hooks cannot be used in a function declaration outside of a react component or hook.
                // For example these are invalid:
                // const notAComponent = () => {
                //    return () => {
                //         useState();
                //    }
                // }
                // --------------
                // export default () => {
                //     if (isVal) {
                //         useState(0);
                //     }
                // }
                // --------------
                // export default function() {
                //     if (isVal) {
                //         useState(0);
                //     }
                // }
                if ident.is_some_and(|name| !is_react_component_or_hook_name(&name)) {
                    return ctx.diagnostic(diagnostics::function_error(
                        *span,
                        hook_name,
                        "Anonymous",
                    ));
                }
            }
            // Hooks can't be called from async function.
            AstKind::Function(Function { id: Some(id), r#async: true, .. }) => {
                return ctx.diagnostic(diagnostics::async_component(id.span, id.name.as_str()));
            }
            // Hooks can't be called from async arrow function.
            AstKind::ArrowFunctionExpression(ArrowFunctionExpression {
                span,
                r#async: true,
                ..
            }) => {
                return ctx.diagnostic(diagnostics::async_component(*span, "Anonymous"));
            }
            _ => {}
        }

        // `use(...)` can be called conditionally, And,
        // `use(...)` can be called within a loop.
        // So we don't need the following checks.
        if is_use {
            return;
        }

        let node_cfg_id = node.cfg_id();
        let func_cfg_id = parent_func.cfg_id();

        // there is no branch between us and our parent function
        if node_cfg_id == func_cfg_id {
            return;
        }

        if !cfg.is_reachable(func_cfg_id, node_cfg_id) {
            // There should always be a control flow path between a parent and child node.
            // If there is none it means we always do an early exit before reaching our hook call.
            // In some cases it might mean that we are operating on an invalid `cfg` but in either
            // case, It is somebody else's problem so we just return.
            return;
        }

        // Is this node cyclic?
        if cfg.is_cyclic(node_cfg_id) {
            return ctx.diagnostic(diagnostics::loop_hook(span, hook_name));
        }

        if has_conditional_path_accept_throw(cfg, parent_func, node) {
            #[allow(clippy::needless_return)]
            return ctx.diagnostic(diagnostics::conditional_hook(span, hook_name));
        }
    }
}

fn has_conditional_path_accept_throw(
    cfg: &ControlFlowGraph,
    from: &AstNode<'_>,
    to: &AstNode<'_>,
) -> bool {
    let from_graph_id = from.cfg_id();
    let to_graph_id = to.cfg_id();
    let graph = cfg.graph();
    if graph
        .edges(to_graph_id)
        .any(|it| matches!(it.weight(), EdgeType::Error(ErrorEdgeKind::Explicit)))
    {
        // TODO: We are simplifying here, There is a real need for a trait like `MayThrow` that
        // would provide a method `may_throw`, since not everything may throw and break the control flow.
        return true;
        // let paths = algo::all_simple_paths::<Vec<_>, _>(graph, from_graph_id, to_graph_id, 0, None);
        // if paths
        //     .flatten()
        //     .flat_map(|id| cfg.basic_block(id).instructions())
        //     .filter_map(|it| match it {
        //         Instruction { kind: InstructionKind::Statement, node_id: Some(node_id) } => {
        //             let r = Some(nodes.get_node(*node_id));
        //             dbg!(&r);
        //             r
        //         }
        //         _ => None,
        //     })
        //     .filter(|it| it.id() != to.id())
        //     .any(|it| {
        //         // TODO: it.may_throw()
        //         matches!(
        //             it.kind(),
        //             AstKind::ExpressionStatement(ExpressionStatement {
        //                 expression: Expression::CallExpression(_),
        //                 ..
        //             })
        //         )
        //     })
        // {
        //     // return true;
        // }
    }
    // All nodes should be able to reach the hook node, Otherwise we have a conditional/branching flow.
    algo::dijkstra(graph, from_graph_id, Some(to_graph_id), |e| match e.weight() {
        EdgeType::NewFunction | EdgeType::Error(ErrorEdgeKind::Implicit) => 1,
        EdgeType::Error(ErrorEdgeKind::Explicit)
        | EdgeType::Join
        | EdgeType::Finalize
        | EdgeType::Jump
        | EdgeType::Unreachable
        | EdgeType::Backedge
        | EdgeType::Normal => 0,
    })
    .into_iter()
    .filter(|(_, val)| *val == 0)
    .any(|(f, _)| {
        !cfg.is_reachable_filtered(f, to_graph_id, |it| {
            if cfg
                .basic_block(it)
                .instructions()
                .iter()
                .any(|i| matches!(i.kind, InstructionKind::Throw))
            {
                Control::Break(true)
            } else {
                Control::Continue
            }
        })
    })
}

fn parent_func<'a>(nodes: &'a AstNodes<'a>, node: &AstNode) -> Option<&'a AstNode<'a>> {
    nodes
        .ancestor_ids(node.id())
        .map(|id| nodes.get_node(id))
        .find(|it| it.kind().is_function_like())
}

/// Checks if the `node_id` is a callback argument,
/// And that function isn't a `React.memo` or `React.forwardRef`.
/// Returns `true` if this node is a function argument and that isn't a React special function.
/// Otherwise it would return `false`.
fn is_non_react_func_arg(nodes: &AstNodes, node_id: NodeId) -> bool {
    let argument = match nodes.parent_node(node_id) {
        Some(parent) if matches!(parent.kind(), AstKind::Argument(_)) => parent,
        _ => return false,
    };

    let Some(AstKind::CallExpression(call)) = nodes.parent_kind(argument.id()) else {
        return false;
    };

    !(is_react_function_call(call, "forwardRef") || is_react_function_call(call, "memo"))
}

fn is_somewhere_inside_component_or_hook(nodes: &AstNodes, node_id: NodeId) -> bool {
    nodes
        .ancestor_ids(node_id)
        .map(|id| nodes.get_node(id))
        .filter(|node| node.kind().is_function_like())
        .map(|node| {
            (
                node.id(),
                match node.kind() {
                    AstKind::Function(func) => func.name().map(Cow::from),
                    AstKind::ArrowFunctionExpression(_) => {
                        get_declaration_identifier(nodes, node.id())
                    }
                    _ => unreachable!(),
                },
            )
        })
        .any(|(id, ident)| {
            ident.is_some_and(|name| is_react_component_or_hook_name(&name))
                || is_memo_or_forward_ref_callback(nodes, id)
        })
}

fn get_declaration_identifier<'a>(
    nodes: &'a AstNodes<'a>,
    node_id: NodeId,
) -> Option<Cow<'a, str>> {
    let node = nodes.get_node(node_id);

    match node.kind() {
        AstKind::Function(Function { id: Some(id), .. }) => {
            // function useHook() {}
            // const whatever = function useHook() {};
            //
            // Function declaration or function expression names win over any
            // assignment statements or other renames.
            Some(Cow::Borrowed(id.name.as_str()))
        }
        AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => {
            let parent =
                nodes.ancestor_ids(node_id).skip(1).map(|node| nodes.get_node(node)).next()?;

            match parent.kind() {
                AstKind::VariableDeclarator(decl) => {
                    decl.id.get_identifier_name().map(|id| Cow::Borrowed(id.as_str()))
                }
                // useHook = () => {};
                AstKind::AssignmentExpression(expr)
                    if matches!(expr.operator, AssignmentOperator::Assign) =>
                {
                    expr.left.get_identifier_name().map(std::convert::Into::into)
                }
                // const {useHook = () => {}} = {};
                // ({useHook = () => {}} = {});
                AstKind::AssignmentPattern(patt) => {
                    patt.left.get_identifier_name().map(|id| Cow::Borrowed(id.as_str()))
                }
                // { useHook: () => {} }
                // { useHook() {} }
                AstKind::ObjectProperty(prop) => prop.key.name(),
                _ => None,
            }
        }
        _ => None,
    }
}

/// # Panics
/// `node_id` should always point to a valid `Function`.
fn is_memo_or_forward_ref_callback(nodes: &AstNodes, node_id: NodeId) -> bool {
    nodes.ancestor_ids(node_id).map(|id| nodes.get_node(id)).any(|node| {
        if let AstKind::CallExpression(call) = node.kind() {
            call.callee_name().is_some_and(|name| matches!(name, "forwardRef" | "memo"))
        } else {
            false
        }
    })
}

#[test]
fn test() {
    ///  Copyright (c) Meta Platforms, Inc. and affiliates.
    /// Most of these tests are sourced from the original react `eslint-plugin-react-hooks` package.
    /// https://github.com/facebook/react/blob/5b903cdaa94c78e8fabb985d8daca5bd7d266323/packages/eslint-plugin-react-hooks/__tests__/ESLintRulesOfHooks-test.js
    use crate::tester::Tester;

    let pass = vec![
        // Valid because components can use hooks.
        "
            function ComponentWithHook() {
              useHook();
            }
        ",
        // Valid because components can use hooks.
        "
            function createComponentWithHook() {
              return function ComponentWithHook() {
                useHook();
              };
            }
        ",
        // Valid because hooks can use hooks.
        "
            function useHookWithHook() {
              useHook();
            }
        ",
        // Valid because hooks can use hooks.
        "
            function createHook() {
              return function useHookWithHook() {
                useHook();
              }
            }
        ",
        // Valid because components can call functions.
        "
            function ComponentWithNormalFunction() {
              doSomething();
            }
        ",
        // Valid because functions can call functions.
        "
            function normalFunctionWithNormalFunction() {
              doSomething();
            }
        ",
        // Valid because functions can call functions.
        "
            function normalFunctionWithConditionalFunction() {
              if (cond) {
                doSomething();
              }
            }
        ",
        // Valid because functions can call functions.
        "
            function functionThatStartsWithUseButIsntAHook() {
              if (cond) {
                userFetch();
              }
            }
        ",
        // Valid although unconditional return doesn't make sense and would fail other rules.
        // We could make it invalid but it doesn't matter.
        "
            function useUnreachable() {
              return;
              useHook();
            }
        ",
        // Valid because hooks can call hooks.
        "
            function useHook() { useState(); }
            const whatever = function useHook() { useState(); };
            const useHook1 = () => { useState(); };
            let useHook2 = () => useState();
            useHook2 = () => { useState(); };
            ({useHook: () => { useState(); }});
            ({useHook() { useState(); }});
            const {useHook3 = () => { useState(); }} = {};
            ({useHook = () => { useState(); }} = {});
            Namespace.useHook = () => { useState(); };
        ",
        // Valid because hooks can call hooks.
        "
            function useHook() {
              useHook1();
              useHook2();
            }
        ",
        // Valid because hooks can call hooks.
        "
            function createHook() {
              return function useHook() {
                useHook1();
                useHook2();
              };
            }
        ",
        // Valid because hooks can call hooks.
        "
            function useHook() {
              useState() && a;
            }
        ",
        // Valid because hooks can call hooks.
        "
            function useHook() {
              return useHook1() + useHook2();
            }
        ",
        // Valid because hooks can call hooks.
        "
            function useHook() {
              return useHook1(useHook2());
            }
        ",
        // Valid because hooks can be used in anonymous arrow-function arguments
        // to forwardRef.
        "
            const FancyButton = React.forwardRef((props, ref) => {
              useHook();
              return <button {...props} ref={ref} />
            });
        ",
        // Valid because hooks can be used in anonymous function arguments to
        // forwardRef.
        "
            const FancyButton = React.forwardRef(function (props, ref) {
              useHook();
              return <button {...props} ref={ref} />
            });
        ",
        // Valid because hooks can be used in anonymous function arguments to
        // forwardRef.
        "
            const FancyButton = forwardRef(function (props, ref) {
              useHook();
              return <button {...props} ref={ref} />
            });
        ",
        // Valid because hooks can be used in anonymous function arguments to
        // React.memo.
        "
            const MemoizedFunction = React.memo(props => {
              useHook();
              return <button {...props} />
            });
        ",
        // Valid because hooks can be used in anonymous function arguments to
        // memo.
        "
            const MemoizedFunction = memo(function (props) {
              useHook();
              return <button {...props} />
            });
        ",
        // Valid because classes can call functions.
        // We don't consider these to be hooks.
        "
            class C {
              m() {
                this.useHook();
                super.useHook();
              }
            }
        ",
        // Valid -- this is a regression test.
        "
            jest.useFakeTimers();
            beforeEach(() => {
              jest.useRealTimers();
            })
        ",
        // Valid because they're not matching use[A-Z].
        "
            fooState();
            _use();
            _useState();
            use_hook();
            // also valid because it's not matching the PascalCase namespace
            jest.useFakeTimer()
        ",
        // Regression test for some internal code.
        // This shows how the "callback rule" is more relaxed,
        // and doesn't kick in unless we're confident we're in
        // a component or a hook.
        "
            function makeListener(instance) {
              each(pixelsWithInferredEvents, pixel => {
                if (useExtendedSelector(pixel.id) && extendedButton) {
                  foo();
                }
              });
            }
        ",
        // This is valid because "use"-prefixed functions called in
        // unnamed function arguments are not assumed to be hooks.
        "
            React.unknownFunction((foo, bar) => {
              if (foo) {
                useNotAHook(bar)
              }
            });
        ",
        // This is valid because "use"-prefixed functions called in
        // unnamed function arguments are not assumed to be hooks.
        "
            unknownFunction(function(foo, bar) {
              if (foo) {
                useNotAHook(bar)
              }
            });
        ",
        // Regression test for incorrectly flagged valid code.
        "
            function RegressionTest() {
              const foo = cond ? a : b;
              useState();
            }
        ",
        // Valid because exceptions abort rendering
        "
            function RegressionTest() {
              if (page == null) {
                throw new Error('oh no!');
              }
              useState();
            }
        ",
        // Valid because the loop doesn't change the order of hooks calls.
        "
            function RegressionTest(test) {
              while (test) {
                test = update(test);
              }
              React.useLayoutEffect(() => {});
            }
        ",
        // Valid because the loop doesn't change the order of hooks calls.
        "
            function RegressionTest() {
              const res = [];
              const additionalCond = true;
              for (let i = 0; i !== 10 && additionalCond; ++i ) {
                res.push(i);
              }
              React.useLayoutEffect(() => {});
            }
        ",
        // Is valid but hard to compute by brute-forcing
        "
            function MyComponent() {
              // 40 conditions
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}
              if (c) {} else {}

              // 10 hooks
              useHook();
              useHook();
              useHook();
              useHook();
              useHook();
              useHook();
              useHook();
              useHook();
              useHook();
              useHook();
            }
        ",
        // Valid because the neither the conditions before or after the hook affect the hook call
        // Failed prior to implementing BigInt because pathsFromStartToEnd and allPathsFromStartToEnd were too big and had rounding errors
        "
            const useSomeHook = () => {};

            const SomeName = () => {
              const filler = FILLER ?? FILLER ?? FILLER;
              const filler2 = FILLER ?? FILLER ?? FILLER;
              const filler3 = FILLER ?? FILLER ?? FILLER;
              const filler4 = FILLER ?? FILLER ?? FILLER;
              const filler5 = FILLER ?? FILLER ?? FILLER;
              const filler6 = FILLER ?? FILLER ?? FILLER;
              const filler7 = FILLER ?? FILLER ?? FILLER;
              const filler8 = FILLER ?? FILLER ?? FILLER;

              useSomeHook();

              if (anyConditionCanEvenBeFalse) {
                return null;
              }

              return (
                <React.Fragment>
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                  {FILLER ? FILLER : FILLER}
                </React.Fragment>
              );
            };
            ",
        // Valid because the neither the condition nor the loop affect the hook call.
        "
            function App(props) {
              const someObject = {propA: true};
              for (const propName in someObject) {
                if (propName === true) {
                } else {
                }
              }
              const [myState, setMyState] = useState(null);
            }
        ",
        "
            function App() {
              const text = use(Promise.resolve('A'));
              return <Text text={text} />
            }
        ",
        "
            import * as React from 'react';
            function App() {
              if (shouldShowText) {
                const text = use(query);
                const data = React.use(thing);
                const data2 = react.use(thing2);
                return <Text text={text} />
              }
              return <Text text={shouldFetchBackupText ? use(backupQuery) : \"Nothing to see here\"} />
            }
        ",
        "
            function App() {
              let data = [];
              for (const query of queries) {
                const text = use(item);
                data.push(text);
              }
              return <Child data={data} />
            }
        ",
        "
            function App() {
              const data = someCallback((x) => use(x));
              return <Child data={data} />
            }
        ",
        "
            function useLabeledBlock() {
                label: {
                    useHook();
                    if (a) break label;
                }
            }
        ",
        "
            export const FalsePositive = ({ editor, anchorElem, isLink, linkNodeUrl, close }: Props) => {
              // This custom hook invocation seems to trigger false positives below
              const [state, setState] = useCustomHook<State>({
                inputLinkUrl: linkNodeUrl ?? '',
                editable: !isLink,
                lastLinkUrl: '',
                lastSelection: null
              });

              const [someThing, setSomeThing] = useState(true);

              const onEdit = useCallback(() => setSomeThing(false), [inputLinkUrl, setSomeThing]);

              const updateLinkEditor = useCallback(() => {
                const rootElement = editor.getRootElement();

                if (nativeSelection.anchorNode === rootElement) {
                  let inner = rootElement;
                  while (inner.firstElementChild !== null) {
                    inner = inner.firstElementChild as HTMLElement;
                  }
                }
              }, [anchorElem, editor, setSomeThing]);

              return <div>test</div>;
            };
        ",
        "
            function useLabeledBlock() {
                let x = () => {
                    if (some) {
                        noop();
                    }
                };
                useHook();
            }
        ",
        "

            export const Component = () => {
                return {
                    Target: () => {
                        useEffect(() => {
                            return () => {
                                something.value = true;
                            };
                        }, []);
                        return <div></div>;
                    },
                    useTargetModule: (m) => {
                        useModule(m);
                    },
                };
            };
        ",
        "
            test.beforeEach(async () => {
                timer = Sinon.useFakeTimers({
                    toFake: ['setInterval'],
                });
            });
    ",
    "export default function App() {
       const [state, setState] = useState(0);

       useEffect(() => {
         console.log('Effect called');
       }, []);

       return <div>{state}</div>;
    }
    // https://github.com/toeverything/AFFiNE/blob/0ec1995addbb09fb5d4af765d84cc914b2905150/packages/frontend/core/src/hooks/use-query.ts#L46
    ",
    "const createUseQuery =
    (immutable: boolean): useQueryFn =>
    (options, config) => {
        const configWithSuspense: SWRConfiguration = useMemo(
            () => ({
                suspense: true,
                ...config,
            }),
            [config],
        );

        const useSWRFn = immutable ? useSWRImutable : useSWR;
        return useSWRFn(options ? () => ['cloud', options.query.id, options.variables] : null, options ? () => fetcher(options) : null, configWithSuspense);
    };",
    // https://github.com/oxc-project/oxc/issues/6651
    r"const MyComponent = makeComponent(() => { useHook(); });",
    r"const MyComponent2 = makeComponent(function () { useHook(); });",
    r"const MyComponent4 = makeComponent(function InnerComponent() { useHook(); });"
    ];

    let fail = vec![
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [conditionalError('useConditionalHook')],
        "
        function ComponentWithConditionalHook() {
               if (cond) {
                 useConditionalHook();
               }
             }
        ",
        // Invalid because hooks can only be called inside of a component.
        // errors: [
        //     topLevelError('Hook.useState'),
        //     topLevelError('Hook.use42'),
        //     topLevelError('Hook.useHook'),
        // ],
        "
            Hook.useState();
            Hook._useState();
            Hook.use42();
            Hook.useHook();
            Hook.use_hook();
        ",
        // errors: [classError('This.useHook'), classError('Super.useHook')],
        "
            class C {
                 m() {
                     This.useHook();
                     Super.useHook();
                 }
            }
        ",
        // This is a false positive (it's valid) that unfortunately
        // we cannot avoid. Prefer to rename it to not start with "use"
        // errors: [classError('FooStore.useFeatureFlag')],
        "
            class Foo extends Component {
                render() {
                    if (cond) {
                        FooStore.useFeatureFlag();
                    }
                }
            }
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [conditionalError('Namespace.useConditionalHook')],
        "
            function ComponentWithConditionalHook() {
                if (cond) {
                    Namespace.useConditionalHook();
                }
            }
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [conditionalError('useConditionalHook')],
        "
                function createComponent() {
                    return function ComponentWithConditionalHook() {
                        if (cond) {
                            useConditionalHook();
                        }
                    }
                }
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [conditionalError('useConditionalHook')],
        "
                function useHookWithConditionalHook() {
                    if (cond) {
                        useConditionalHook();
                    }
                }
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [conditionalError('useConditionalHook')],
        "
                function createHook() {
                    return function useHookWithConditionalHook() {
                        if (cond) {
                            useConditionalHook();
                        }
                    }
                }
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [conditionalError('useTernaryHook')],
        "
                function ComponentWithTernaryHook() {
                    cond ? useTernaryHook() : null;
                }
        ",
        // Invalid because it's a common misunderstanding.
        // We *could* make it valid but the runtime error could be confusing.
        // errors: [genericError('useHookInsideCallback')],
        "
                function ComponentWithHookInsideCallback() {
                    useEffect(() => {
                        useHookInsideCallback();
                    });
                }
        ",
        // Invalid because it's a common misunderstanding.
        // We *could* make it valid but the runtime error could be confusing.
        // errors: [genericError('useHookInsideCallback')],
        "
                function createComponent() {
                    return function ComponentWithHookInsideCallback() {
                        useEffect(() => {
                            useHookInsideCallback();
                        });
                    }
                }
        ",
        // Invalid because it's a common misunderstanding.
        // We *could* make it valid but the runtime error could be confusing.
        // errors: [genericError('useHookInsideCallback')],
        "
                const ComponentWithHookInsideCallback = React.forwardRef((props, ref) => {
                    useEffect(() => {
                        useHookInsideCallback();
                    });
                    return <button {...props} ref={ref} />
                });
        ",
        // Invalid because it's a common misunderstanding.
        // We *could* make it valid but the runtime error could be confusing.
        // errors: [genericError('useHookInsideCallback')],
        "
                const ComponentWithHookInsideCallback = React.memo(props => {
                    useEffect(() => {
                        useHookInsideCallback();
                    });
                    return <button {...props} />
                });
        ",
        // Invalid because it's a common misunderstanding.
        // We *could* make it valid but the runtime error could be confusing.
        // errors: [functionError('useState', 'handleClick')],
        "
                function ComponentWithHookInsideCallback() {
                    function handleClick() {
                        useState();
                    }
                }
        ",
        // Invalid because it's a common misunderstanding.
        // We *could* make it valid but the runtime error could be confusing.
        // errors: [functionError('useState', 'handleClick')],
        "
                function createComponent() {
                    return function ComponentWithHookInsideCallback() {
                        function handleClick() {
                            useState();
                        }
                    }
                }
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [loopError('useHookInsideLoop')],
        "
                function ComponentWithHookInsideLoop() {
                    while (cond) {
                        useHookInsideLoop();
                    }
                }
        ",
        "
            function ComponentWithHookInsideLoop() {
              do {
                useHookInsideLoop();
              } while (cond);
            }
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        "
            function ComponentWithHookInsideLoop() {
              do {
                foo();
              } while (useHookInsideLoop());
            }
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [functionError('useState', 'renderItem')],
        "
                function renderItem() {
                    useState();
                }

                function List(props) {
                    return props.items.map(renderItem);
                }
        ",
        // Currently invalid because it violates the convention and removes the "taint"
        // from a hook. We *could* make it valid to avoid some false positives but let's
        // ensure that we don't break the "renderItem" and "normalFunctionWithConditionalHook"
        // cases which must remain invalid.
        // errors: [functionError('useHookInsideNormalFunction', 'normalFunctionWithHook'), ],
        "
                function normalFunctionWithHook() {
                    useHookInsideNormalFunction();
                }
        ",
        // These are neither functions nor hooks.
        // errors: [
        //     functionError('useHookInsideNormalFunction', '_normalFunctionWithHook'),
        //     functionError('useHookInsideNormalFunction', '_useNotAHook'),
        // ],
        "
                function _normalFunctionWithHook() {
                    useHookInsideNormalFunction();
                }
                function _useNotAHook() {
                    useHookInsideNormalFunction();
                }
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [
        //   functionError(
        //     'useHookInsideNormalFunction',
        //     'normalFunctionWithConditionalHook'
        //   ),
        // ],
        "
                function normalFunctionWithConditionalHook() {
                    if (cond) {
                        useHookInsideNormalFunction();
                    }
                }
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [
        //     loopError('useHook1'),
        //     loopError('useHook2'),
        //     loopError('useHook3'),
        //     loopError('useHook4'),
        // ]
        "
                function useHookInLoops() {
                    while (a) {
                        useHook1();
                        if (b) return;
                        useHook2();
                    }
                    while (c) {
                        useHook3();
                        if (d) return;
                        useHook4();
                    }
                }
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [loopError('useHook1'), loopError('useHook2', true)],
        "
            function useHookInLoops() {
                while (a) {
                    useHook1();
                    if (b) continue;
                    useHook2();
                }
            }
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        r"
       function useHookInLoops() {
         do {
           useHook1();
           if (a) return;
           useHook2();
         } while (b);

         do {
           useHook3();
           if (c) return;
           useHook4();
         } while (d)
       }
       ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        r"
        function useHookInLoops() {
          do {
            useHook1();
            if (a) continue;
            useHook2();
          } while (b);
        }
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [conditionalError('useHook')],
        "
                function useLabeledBlock() {
                    label: {
                        if (a) break label;
                        useHook();
                    }
                }
        ",
        // Currently invalid.
        // These are variations capturing the current heuristic--
        // we only allow hooks in PascalCase or useFoo functions.
        // We *could* make some of these valid. But before doing it,
        // consider specific cases documented above that contain reasoning.
        // errors: [
        //     functionError('useState', 'a'),
        //     functionError('useState', 'b'),
        //     functionError('useState', 'c'),
        //     functionError('useState', 'd'),
        //     functionError('useState', 'e'),
        //     functionError('useState', 'f'),
        //     functionError('useState', 'g'),
        //     functionError('useState', 'j'),
        //     functionError('useState', 'k'),
        // ]
        "
            function a() { useState(); }
            const whatever = function b() { useState(); };
            const c = () => { useState(); };
            let d = () => useState();
            e = () => { useState(); };
            ({f: () => { useState(); }});
            ({g() { useState(); }});
            const {j = () => { useState(); }} = {};
            ({k = () => { useState(); }} = {});
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [conditionalError('useState', true)],
        "
                function useHook() {
                    if (a) return;
                    useState();
                }
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [conditionalError('useState', true)],
        "
                function useHook() {
                    if (a) return;
                    if (b) {
                        console.log('true');
                    } else {
                        console.log('false');
                    }
                    useState();
                }
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [conditionalError('useState', true)],
        "
                function useHook() {
                    if (b) {
                        console.log('true');
                    } else {
                        console.log('false');
                    }
                    if (a) return;
                    useState();
                }
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [conditionalError('useHook1'), conditionalError('useHook2')],
        "
                function useHook() {
                    a && useHook1();
                    b && useHook2();
                }
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [
        //     // NOTE: This is an error since `f()` could possibly throw.
        //     conditionalError('useState'),
        // ],
        "
                function useHook() {
                    try {
                        f();
                        useState();
                    } catch {}
                }
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [
        //     conditionalError('useState'),
        //     conditionalError('useState'),
        //     conditionalError('useState'),
        // ],
        "
                function useHook({ bar }) {
                    let foo1 = bar && useState();
                    let foo2 = bar || useState();
                    let foo3 = bar ?? useState();
                }
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [conditionalError('useCustomHook')],
        "
                const FancyButton = React.forwardRef((props, ref) => {
                    if (props.fancy) {
                        useCustomHook();
                    }
                    return <button ref={ref}>{props.children}</button>;
                });
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [conditionalError('useCustomHook')],
        "
                const FancyButton = forwardRef(function(props, ref) {
                    if (props.fancy) {
                        useCustomHook();
                    }
                    return <button ref={ref}>{props.children}</button>;
                });
        ",
        // Invalid because it's dangerous and might not warn otherwise.
        // This *must* be invalid.
        // errors: [conditionalError('useCustomHook')],
        "
                const MemoizedButton = memo(function(props) {
                    if (props.fancy) {
                        useCustomHook();
                    }
                    return <button>{props.children}</button>;
                });
        ",
        // This is invalid because "use"-prefixed functions used in named
        // functions are assumed to be hooks.
        // errors: [functionError('useProbablyAHook', 'notAComponent')],
        "
                React.unknownFunction(function notAComponent(foo, bar) {
                    useProbablyAHook(bar)
                });
        ",
        // Invalid because it's dangerous.
        // Normally, this would crash, but not if you use inline requires.
        // This *must* be invalid.
        // It's expected to have some false positives, but arguably
        // they are confusing anyway due to the use*() convention
        // already being associated with Hooks.
        // errors: [
        //     topLevelError('useState'),
        //     topLevelError('React.useCallback'),
        //     topLevelError('useCustomHook'),
        // ],
        "
            useState();
            if (foo) {
                const foo = React.useCallback(() => {});
            }
            useCustomHook();
        ",
        // Technically this is a false positive.
        // We *could* make it valid (and it used to be).
        //
        // However, top-level Hook-like calls can be very dangerous
        // in environments with inline requires because they can mask
        // the runtime error by accident.
        // So we prefer to disallow it despite the false positive.
        // errors: [topLevelError('useBasename')],
        "
            const {createHistory, useBasename} = require('history-2.1.2');
            const browserHistory = useBasename(createHistory)({
                basename: '/',
            });
        ",
        // errors: [classError('useFeatureFlag')],
        "
                class ClassComponentWithFeatureFlag extends React.Component {
                    render() {
                        if (foo) {
                            useFeatureFlag();
                        }
                    }
                }
        ",
        // errors: [classError('React.useState')],
        "
                class ClassComponentWithHook extends React.Component {
                    render() {
                        React.useState();
                    }
                }
        ",
        // errors: [classError('useState')],
        "(class {useHook = () => { useState(); }});",
        // errors: [classError('useState')],
        "(class {useHook() { useState(); }});",
        // errors: [classError('useState')],
        "(class {h = () => { useState(); }});",
        // errors: [classError('useState')],
        "(class {i() { useState(); }});",
        // errors: [asyncComponentHookError('useState')],
        "
                async function AsyncComponent() {
                    useState();
                }
        ",
        "
                const AsyncComponent = async () => {
                    useState();
                }
        ",
        r"
                async function Page() {
                  useId();
                  React.useId();
                }
        ",
        // errors: [asyncComponentHookError('useState')],
        "
                async function useAsyncHook() {
                    useState();
                }
        ",
        r"
                async function notAHook() {
                  useId();
                }
        ",
        // errors: [
        //     topLevelError('Hook.use'),
        //     topLevelError('Hook.useState'),
        //     topLevelError('Hook.use42'),
        //     topLevelError('Hook.useHook'),
        // ],
        "
            Hook.use();
            Hook._use();
            Hook.useState();
            Hook._useState();
            Hook.use42();
            Hook.useHook();
            Hook.use_hook();
        ",
        // errors: [functionError('use', 'notAComponent')],
        "
                function notAComponent() {
                    use(promise);
                }
        ",
        // errors: [topLevelError('use')],
        "
            const text = use(promise);
            function App() {
                return <Text text={text} />
            }
        ",
        // errors: [classError('use')],
        "
            class C {
                m() {
                    use(promise);
                }
            }
        ",
        // errors: [asyncComponentHookError('use')],
        "
            async function AsyncComponent() {
                    use();
            }
        ",
        // errors: [functionError('use', 'notAComponent')],
        // React doesn't report on this https://github.com/facebook/react/blob/9daabc0bf97805be23f6131be4d84d063a3ff446/packages/eslint-plugin-react-hooks/__tests__/ESLintRulesOfHooks-test.js#L520-L530
        // Even so, i think this is valid
        // e.g:
        // ```
        // const useMyHook = notAComponent();
        // function Foo () {
        //    useMyHook();
        // }
        // ```
        // "
        //     export const notAComponent = () => {
        //         return () => {
        //             useState();
        //       }
        //     }
        // ",
        // errors: [functionError('use', 'notAComponent')],
        "
            const notAComponent = () => {
                useState();
            }
        ",
        // errors: [genericError('useState')],
        "
            export default () => {
                if (isVal) {
                    useState(0);
                }
            }
        ",
        // errors: [genericError('useState')],
        "
            export default function() {
                if (isVal) {
                    useState(0);
                }
            }
        ",
        // TODO: This should error but doesn't.
        // Original rule also fails to raise this error.
        // errors: [genericError('useState')],
        // "
        //     function notAComponent() {
        //         return new Promise.then(() => {
        //             useState();
        //         });
        //     }
        // " ,
        // https://github.com/oxc-project/oxc/issues/6651
        r"const MyComponent3 = makeComponent(function foo () { useHook(); });",
    ];

    Tester::new(RulesOfHooks::NAME, RulesOfHooks::PLUGIN, pass, fail).test_and_snapshot();
}
