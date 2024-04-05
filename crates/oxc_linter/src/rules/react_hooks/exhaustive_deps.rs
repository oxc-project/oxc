use miette::diagnostic;
use oxc_allocator::Box as OBox;
use oxc_semantic::ScopeId;
use std::collections::HashSet;

use oxc_ast::{
    ast::{
        Argument, ArrayExpressionElement, AssignmentTarget, BindingPatternKind, BlockStatement,
        CallExpression, ChainElement, Declaration, Expression, IdentifierReference,
        MemberExpression, SimpleAssignmentTarget, Statement, VariableDeclarationKind,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, CompactStr, Span};
use phf::phf_set;

use crate::{ast_util::get_declaration_of_variable, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("react-hooks(exhaustive-deps): React Hook {0} has a missing dependency: {1}")]
#[diagnostic(severity(warning), help("Either include it or remove the dependency array."))]
struct MissingDependencyDiagnostic(CompactStr, CompactStr, #[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("react-hooks(exhaustive-deps): React Hook {0} has unnecessary dependency: {1}")]
#[diagnostic(severity(warning), help("Either exclude it or remove the dependency array."))]
struct UnnecessaryDependencyDiagnostic(CompactStr, CompactStr, #[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error(
    "react-hooks(exhaustive-deps): React Hook {0} does nothing when called with only one argument."
)]
#[diagnostic(severity(warning), help("Did you forget to pass an array of dependencies?"))]
struct DependencyArrayRequiredDiagnostic(CompactStr, #[label] pub Span);

// `React Hook ${reactiveHookName} has a missing dependency: '${callback.name}'. ` +
// `Either include it or remove the dependency array.`,

#[derive(Debug, Default, Clone)]
pub struct ExhaustiveDeps;

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
    ExhaustiveDeps,
    correctness
);

const HOOKS: phf::Set<&'static str> =
    phf_set!("useEffect", "useLayoutEffect", "useCallback", "useMemo");

const HOOKS_USELESS_WITHOUT_DEPENDENCIES: phf::Set<&'static str> =
    phf_set!("useCallback", "useMemo");

// struct ScanOptions {
//     component_scope_id: ScopeId,
//     hook_scope_id: ScopeId,
// }

impl Rule for ExhaustiveDeps {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };
        let Some(callback) = func_call_without_react_namespace(call_expr) else { return };

        if HOOKS.contains(&callback) {
            let second_arg = call_expr.arguments.get(1);

            if second_arg.is_none() && HOOKS_USELESS_WITHOUT_DEPENDENCIES.contains(&callback) {
                ctx.diagnostic(DependencyArrayRequiredDiagnostic(
                    CompactStr::from(callback.to_string()),
                    call_expr.span,
                ));
                return;
            }

            let Some(Argument::Expression(arg0_expr)) = call_expr.arguments.get(0) else { return };
            let Expression::ArrowFunctionExpression(body_expr) = arg0_expr else { return };

            let Some(component_scope_id) = ctx.semantic().scopes().get_parent_id(node.scope_id())
            else {
                return;
            };

            let declared_deps = if let Some(arg) = second_arg {
                collect_dependencies(arg, ctx)
            } else {
                HashSet::new()
            };

            let body_expr = &body_expr.body;
            let mut found_deps: DependencyList = HashSet::new();

            // println!("lint {callback}");
            for stmt in &body_expr.statements {
                check_statement(stmt, ctx, &mut found_deps, component_scope_id);
            }

            dbg!(&declared_deps);
            dbg!(&found_deps);
            let undeclared_deps: Vec<_> = found_deps.difference(&declared_deps).collect();
            for dep in undeclared_deps {
                if declared_deps.iter().any(|decl_dep| dep.contains(decl_dep)) {
                    continue;
                }

                if !is_identifier_a_dependency(dep.iref, ctx, component_scope_id) {
                    continue;
                };

                ctx.diagnostic(MissingDependencyDiagnostic(
                    CompactStr::from(callback.to_string()),
                    CompactStr::from(dep.to_string()),
                    dep.iref.span,
                ));
                return;
            }

            let unnecessary_deps: Vec<_> = declared_deps.difference(&found_deps).collect();
            dbg!(&unnecessary_deps);

            for dep in unnecessary_deps {
                if found_deps.iter().any(|found_dep| found_dep.contains(dep)) {
                    continue;
                }

                ctx.diagnostic(UnnecessaryDependencyDiagnostic(
                    CompactStr::from(callback.to_string()),
                    CompactStr::from(dep.to_string()),
                    dep.iref.span,
                ));
            }
        }
    }
}

// TODO: i don't like this, but don't know of a better way yet.
fn chain_contains(a: &Vec<String>, b: &Vec<String>) -> bool {
    for (index, part) in b.iter().enumerate() {
        let Some(other) = a.get(index) else { return false };
        if other != part {
            return false;
        };
    }

    return true;
}

#[derive(Hash, Debug)]
struct Dependency<'a> {
    iref: &'a OBox<'a, IdentifierReference<'a>>,
    chain: Vec<String>,
}

impl PartialEq for Dependency<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.iref.name == other.iref.name && self.chain == other.chain
    }
}

impl Eq for Dependency<'_> {}

impl Dependency<'_> {
    fn to_string(&self) -> String {
        [vec![self.iref.name.to_string()], self.chain.clone()].concat().join(".")
    }

    fn contains(&self, other: &Self) -> bool {
        self.iref.name == other.iref.name && chain_contains(&self.chain, &other.chain)
    }
}

type DependencyList<'a> = HashSet<Dependency<'a>>;

fn collect_dependencies<'a>(deps: &'a Argument<'a>, _ctx: &LintContext) -> DependencyList<'a> {
    let Argument::Expression(arg1_expr) = deps else { return HashSet::new() };

    let Expression::ArrayExpression(array_expr) = arg1_expr else {
        return HashSet::new();
    };

    let mut result: DependencyList = HashSet::new();

    for elem in &array_expr.elements {
        match elem {
            ArrayExpressionElement::Expression(expr) => {
                if let Some(dependency) = analyze_property_chain(expr) {
                    result.insert(dependency);
                }
            }
            _ => {
                println!("TODO(connect_dependencies) {:?}", elem);
            }
        }
    }

    return result;
}

// https://github.com/facebook/react/blob/fee786a057774ab687aff765345dd86fce534ab2/packages/eslint-plugin-react-hooks/src/ExhaustiveDeps.js#L1705
fn analyze_property_chain<'a>(expr: &'a Expression<'a>) -> Option<Dependency> {
    match expr {
        Expression::Identifier(ident) => return Some(Dependency { iref: ident, chain: vec![] }),
        Expression::MemberExpression(member_expr) => concat_members(member_expr),
        Expression::ChainExpression(chain_expr) => match &chain_expr.expression {
            ChainElement::MemberExpression(member_expr) => concat_members(member_expr),
            _ => {
                println!("TODO(analyze_property_chain) {:?}", expr);
                return None;
            }
        },
        _ => {
            println!("TODO(analyze_property_chain) {:?}", expr);
            return None;
        }
    }
}

fn concat_members<'a>(member_expr: &'a OBox<'_, MemberExpression<'a>>) -> Option<Dependency<'a>> {
    let Some(source) = analyze_property_chain(member_expr.object()) else { return None };

    if let Some(prop_name) = member_expr.static_property_name() {
        let new_chain = Vec::from([prop_name.to_string()]);
        return Some(Dependency { iref: source.iref, chain: [source.chain, new_chain].concat() });
    } else {
        return Some(source);
    };
}

fn check_statement<'a>(
    statement: &'a Statement<'a>,
    ctx: &LintContext,
    deps: &mut DependencyList<'a>,
    component_scope_id: ScopeId,
) {
    match statement {
        Statement::ExpressionStatement(expr) => {
            check_expression(&expr.expression, ctx, deps, component_scope_id);
        }
        Statement::IfStatement(if_statement) => {
            check_statement(&if_statement.consequent, ctx, deps, component_scope_id);

            if let Some(alt) = &if_statement.alternate {
                check_statement(&alt, ctx, deps, component_scope_id);
            }
        }
        Statement::BlockStatement(block) => {
            for entry in &block.body {
                check_statement(entry, ctx, deps, component_scope_id);
            }
        }
        Statement::Declaration(decl) => {
            check_declaration(decl, ctx, deps, component_scope_id);
        }
        Statement::TryStatement(try_statement) => {
            check_block_statement(&try_statement.block, ctx, deps, component_scope_id);

            if let Some(handler) = &try_statement.handler {
                check_block_statement(&handler.body, ctx, deps, component_scope_id);
            }

            if let Some(finally) = &try_statement.finalizer {
                check_block_statement(&finally, ctx, deps, component_scope_id);
            }
        }
        Statement::ReturnStatement(ret_statement) => {
            if let Some(arg) = &ret_statement.argument {
                check_expression(&arg, ctx, deps, component_scope_id);
            }
        }
        _ => {
            println!("TODO(check_statement) {:?}", statement);
        }
    }
}

fn check_block_statement<'a>(
    block: &'a BlockStatement<'a>,
    ctx: &LintContext,
    deps: &mut DependencyList<'a>,
    component_scope_id: ScopeId,
) {
    for entry in &block.body {
        check_statement(entry, ctx, deps, component_scope_id);
    }
}

fn check_declaration<'a>(
    decl: &'a oxc_ast::ast::Declaration<'a>,
    ctx: &LintContext<'_>,
    deps: &mut DependencyList<'a>,
    component_scope_id: ScopeId,
) {
    match decl {
        Declaration::VariableDeclaration(var) => {
            for entry in &var.declarations {
                let Some(init) = &entry.init else {
                    continue;
                };
                check_expression(&init, ctx, deps, component_scope_id);
            }
        }
        Declaration::FunctionDeclaration(func_dec) => {
            let Some(body) = &func_dec.body else {
                return;
            };

            for stmt in &body.statements {
                check_statement(stmt, ctx, deps, component_scope_id);
            }
        }
        _ => {
            println!("TODO(check_declaration) {:?}", decl);
        }
    }
}

fn check_expression<'a>(
    expression: &'a Expression<'a>,
    ctx: &LintContext,
    deps: &mut DependencyList<'a>,
    component_scope_id: ScopeId,
) {
    // dbg!(expression);

    match expression {
        Expression::CallExpression(call_expr) => {
            check_call_expression(call_expr, ctx, deps, component_scope_id);
        }
        // TODO: avoid checking the same identifier multiple times in multiple references?
        Expression::Identifier(ident) => {
            deps.insert(Dependency { iref: ident, chain: vec![] });
        }
        Expression::MemberExpression(member_expr) => {
            check_member_expression(member_expr, ctx, deps, component_scope_id)
        }
        Expression::ArrowFunctionExpression(arrow_fn) => {
            for entry in &arrow_fn.body.statements {
                check_statement(entry, ctx, deps, component_scope_id);
            }
        }
        Expression::ChainExpression(chain_expr) => match &chain_expr.expression {
            ChainElement::CallExpression(call_expr) => {
                check_call_expression(call_expr, ctx, deps, component_scope_id)
            }
            ChainElement::MemberExpression(member_expr) => {
                check_member_expression(member_expr, ctx, deps, component_scope_id)
            }
        },
        Expression::LogicalExpression(logical_expr) => {
            check_expression(&logical_expr.left, ctx, deps, component_scope_id);
            check_expression(&logical_expr.right, ctx, deps, component_scope_id);
        }
        Expression::AssignmentExpression(ass) => {
            if let AssignmentTarget::SimpleAssignmentTarget(target) = &ass.left {
                match target {
                    SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                        deps.insert(Dependency { iref: &ident, chain: vec![] });
                    }
                    SimpleAssignmentTarget::MemberAssignmentTarget(member_expr) => {
                        check_member_expression(member_expr, ctx, deps, component_scope_id);
                    }
                    _ => {
                        println!("TODO(check_expression) {:?}", target)
                    }
                }
            }

            check_expression(&ass.right, ctx, deps, component_scope_id);
        }

        Expression::ArrayExpression(ary_expr) => {
            for elem in &ary_expr.elements {
                match elem {
                    ArrayExpressionElement::Expression(expr) => {
                        check_expression(expr, ctx, deps, component_scope_id);
                    }
                    _ => {
                        println!("TODO(check_expression) {:?}", elem);
                    }
                }
            }
        }
        _ => {
            println!("TODO(check_expression) {:?}", expression);
        }
    }
}

fn check_call_expression<'a>(
    call_expr: &'a OBox<'_, CallExpression<'a>>,
    ctx: &LintContext<'_>,
    deps: &mut DependencyList<'a>,
    component_scope_id: ScopeId,
) {
    println!("check_call_expression {:?}", call_expr);
    check_expression(&call_expr.callee, ctx, deps, component_scope_id);

    for arg in &call_expr.arguments {
        match arg {
            Argument::Expression(expr) => check_expression(&expr, ctx, deps, component_scope_id),
            _ => {
                println!("TODO(check_expression) {:?}", arg);
            }
        }
    }
}

fn check_member_expression<'a>(
    member_expr: &'a OBox<'_, MemberExpression<'a>>,
    ctx: &LintContext,
    deps: &mut DependencyList<'a>,
    component_scope_id: ScopeId,
) {
    let mut object = member_expr.object();

    dbg!(&object);

    while let Expression::MemberExpression(expr) = object {
        object = expr.object();
        dbg!(&object);
    }

    if let Expression::CallExpression(call_expr) = object {
        check_call_expression(&call_expr, ctx, deps, component_scope_id);
    }

    // TODO: check arguments
    // TODO: final span to cover all the expression.
    // check_expression(object, ctx, deps, component_scope_id);

    if let Some(dependency) = concat_members(member_expr) {
        deps.insert(dependency);
    };
}

fn is_identifier_a_dependency(
    ident: &OBox<'_, IdentifierReference<'_>>,
    ctx: &LintContext,
    component_scope_id: ScopeId,
) -> bool {
    if ctx.semantic().is_reference_to_global_variable(ident) {
        return false;
    }

    let Some(declaration) = get_declaration_of_variable(ident, ctx) else {
        return false;
    };

    // TODO: if identifier is assigned, check whether the source is a dependency instead.

    let semantic = ctx.semantic();
    let scopes = semantic.scopes();

    // Variable was declared outside the component scope
    if scopes.ancestors(component_scope_id).any(|parent| parent == declaration.scope_id()) {
        return false;
    }

    if is_stable_value(declaration, &ident.name) {
        return false;
    }

    let Some(reference_id) = ident.reference_id.get() else {
        return false;
    };

    let reference = semantic.symbols().get_reference(reference_id);
    let node = semantic.nodes().get_node(reference.node_id());

    if declaration.scope_id() == node.scope_id()
        || scopes.descendants(node.scope_id()).any(|id| id == declaration.scope_id())
    {
        return false;
    } else {
        println!(
            "name {:?} decl scope {:?}, node scope {:?}",
            ident.name,
            declaration.scope_id(),
            node.scope_id()
        );
        // dbg!(scopes.descendants(node.scope_id()).collect());
    }

    return true;
}

// https://github.com/facebook/react/blob/fee786a057774ab687aff765345dd86fce534ab2/packages/eslint-plugin-react-hooks/src/ExhaustiveDeps.js#L164
fn is_stable_value(node: &AstNode, name: &Atom) -> bool {
    // println!("is_stable_value");
    // dbg!(node);
    match node.kind() {
        AstKind::VariableDeclaration(declaration) => {
            if declaration.kind == VariableDeclarationKind::Const {
                return true;
            }

            println!("TODO(is_stable_value) {:?}", declaration);
            return false;
        }
        // AstKind::Function(_) => true,
        AstKind::VariableDeclarator(declaration) => {
            let Some(init) = &declaration.init else {
                return false;
            };

            if matches!(init, Expression::ArrowFunctionExpression(_))
                || matches!(init, Expression::FunctionExpression(_))
            {
                return true;
            };

            // dbg!(declaration);
            if declaration.kind == VariableDeclarationKind::Const
                && (init.is_literal() || matches!(init, Expression::ObjectExpression(_)))
            {
                return true;
            };

            let Expression::CallExpression(init_expr) = &init else {
                return false;
            };

            let Some(init_name) = func_call_without_react_namespace(&init_expr) else {
                return false;
            };

            if init_name == "useRef" {
                return true;
            }

            let BindingPatternKind::ArrayPattern(array_pat) = &declaration.id.kind else {
                return false;
            };

            let Some(Some(second_arg)) = array_pat.elements.get(1) else {
                return false;
            };

            let BindingPatternKind::BindingIdentifier(binding_ident) = &second_arg.kind else {
                return false;
            };

            if (init_name == "useState"
                || init_name == "useReducer"
                || init_name == "useTransition")
                && binding_ident.name == name
            {
                return true;
            }

            return false;
        }
        AstKind::FormalParameter(_) => return false,
        AstKind::Function(_) => return true,
        _ => {
            println!("TODO(is_stable_value) {:?}", node);
            return false;
        }
    }
}

// TODO: return atom instead of string?
// https://github.com/facebook/react/blob/fee786a057774ab687aff765345dd86fce534ab2/packages/eslint-plugin-react-hooks/src/ExhaustiveDeps.js#L1742
fn func_call_without_react_namespace(call_expr: &CallExpression) -> Option<String> {
    let inner_exp = call_expr.callee.get_inner_expression();

    if let Expression::Identifier(ident) = inner_exp {
        return Some(ident.name.to_string());
    }

    let Expression::MemberExpression(member_expr) = inner_exp else {
        return None;
    };

    let MemberExpression::StaticMemberExpression(member) = &member_expr.0 else {
        return None;
    };

    let Some(reference) = &member.object.get_identifier_reference() else { return None };

    if reference.name == "React" {
        return Some(member.property.name.to_string());
    }

    None
}

#[test]
fn test() {
    use crate::tester::Tester;

    // let pass_temp = vec![
    //     r"function MyComponent() {
    //   const local1 = someFunc();
    //   function MyNestedComponent() {
    //     const local2 = someFunc();
    //     useCallback(() => {
    //       console.log(local1);
    //       console.log(local2);
    //     }, [local2]);
    //   }
    // }",
    // ];
    // let fail_temp = vec![];

    // Tester::new(ExhaustiveDeps::NAME, pass_temp, fail_temp).test_and_snapshot();
    // return;

    let pass = vec![
        r"function MyComponent() {
          const local = {};
          useEffect(() => {
            console.log(local);
          });
        }",
        r"function MyComponent() {
          useEffect(() => {
            const local = {};
            console.log(local);
          }, []);
        }",
        r"function MyComponent() {
          const local = someFunc();
          useEffect(() => {
            console.log(local);
          }, [local]);
        }",
        r"function MyComponent() {
          useEffect(() => {
            console.log(props.foo);
          }, []);
        }",
        r"function MyComponent() {
          const local1 = {};
          {
            const local2 = {};
            useEffect(() => {
              console.log(local1);
              console.log(local2);
            });
          }
        }",
        r"function MyComponent() {
          const local1 = someFunc();
          {
            const local2 = someFunc();
            useCallback(() => {
              console.log(local1);
              console.log(local2);
            }, [local1, local2]);
          }
        }",
        r"function MyComponent() {
          const local1 = someFunc();
          function MyNestedComponent() {
            const local2 = someFunc();
            useCallback(() => {
              console.log(local1);
              console.log(local2);
            }, [local2]);
          }
        }",
        r"function MyComponent() {
          const local = someFunc();
          useEffect(() => {
            console.log(local);
            console.log(local);
          }, [local]);
        }",
        r"function MyComponent() {
          useEffect(() => {
            console.log(unresolved);
          }, []);
        }",
        r"function MyComponent() {
          const local = someFunc();
          useEffect(() => {
            console.log(local);
          }, [,,,local,,,]);
        }",
        r"function MyComponent({ foo }) {
          useEffect(() => {
            console.log(foo.length);
          }, [foo]);
        }",
        r"function MyComponent({ foo }) {
          useEffect(() => {
            console.log(foo.length);
            console.log(foo.slice(0));
          }, [foo]);
        }",
        r"function MyComponent({ history }) {
          useEffect(() => {
            return history.listen();
          }, [history]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {});
          useLayoutEffect(() => {});
          useImperativeHandle(props.innerRef, () => {});
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo);
          }, [props.foo]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo);
            console.log(props.bar);
          }, [props.bar, props.foo]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo);
            console.log(props.bar);
          }, [props.foo, props.bar]);
        }",
        r"function MyComponent(props) {
          const local = someFunc();
          useEffect(() => {
            console.log(props.foo);
            console.log(props.bar);
            console.log(local);
          }, [props.foo, props.bar, local]);
        }",
        r"function MyComponent(props) {
          const local = {};
          useEffect(() => {
            console.log(props.foo);
            console.log(props.bar);
          }, [props, props.foo]);

          let color = someFunc();
          useEffect(() => {
            console.log(props.foo.bar.baz);
            console.log(color);
          }, [props.foo, props.foo.bar.baz, color]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo?.bar?.baz ?? null);
          }, [props.foo]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo?.bar);
          }, [props.foo?.bar]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo?.bar);
          }, [props.foo.bar]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo.bar);
          }, [props.foo?.bar]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo.bar);
            console.log(props.foo?.bar);
          }, [props.foo?.bar]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo.bar);
            console.log(props.foo?.bar);
          }, [props.foo.bar]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo);
            console.log(props.foo?.bar);
          }, [props.foo]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo?.toString());
          }, [props.foo]);
        }",
        r"function MyComponent(props) {
          useMemo(() => {
            console.log(props.foo?.toString());
          }, [props.foo]);
        }",
        r"function MyComponent(props) {
          useCallback(() => {
            console.log(props.foo?.toString());
          }, [props.foo]);
        }",
        r"function MyComponent(props) {
          useCallback(() => {
            console.log(props.foo.bar?.toString());
          }, [props.foo.bar]);
        }",
        r"function MyComponent(props) {
          useCallback(() => {
            console.log(props.foo?.bar?.toString());
          }, [props.foo.bar]);
        }",
        r"function MyComponent(props) {
          useCallback(() => {
            console.log(props.foo.bar.toString());
          }, [props?.foo?.bar]);
        }",
        r"function MyComponent(props) {
          useCallback(() => {
            console.log(props.foo?.bar?.baz);
          }, [props?.foo.bar?.baz]);
        }",
        r"function MyComponent() {
          const myEffect = () => {
            // Doesn't use anything
          };
          useEffect(myEffect, []);
        }",
        r"const local = {};
        function MyComponent() {
          const myEffect = () => {
            console.log(local);
          };
          useEffect(myEffect, []);
        }",
        r"const local = {};
        function MyComponent() {
          function myEffect() {
            console.log(local);
          }
          useEffect(myEffect, []);
        }",
        r"function MyComponent() {
          const local = someFunc();
          function myEffect() {
            console.log(local);
          }
          useEffect(myEffect, [local]);
        }",
        r"function MyComponent() {
          function myEffect() {
            console.log(global);
          }
          useEffect(myEffect, []);
        }",
        r"const local = {};
        function MyComponent() {
          const myEffect = () => {
            otherThing()
          }
          const otherThing = () => {
            console.log(local);
          }
          useEffect(myEffect, []);
        }",
        r"function MyComponent({delay}) {
          const local = {};
          const myEffect = debounce(() => {
            console.log(local);
          }, delay);
          useEffect(myEffect, [myEffect]);
        }",
        r"function MyComponent({myEffect}) {
          useEffect(myEffect, [,myEffect]);
        }",
        r"function MyComponent({myEffect}) {
          useEffect(myEffect, [,myEffect,,]);
        }",
        r"let local = {};
        function myEffect() {
          console.log(local);
        }
        function MyComponent() {
          useEffect(myEffect, []);
        }",
        r"function MyComponent({myEffect}) {
          useEffect(myEffect, [myEffect]);
        }",
        r"function MyComponent({myEffect}) {
          useEffect(myEffect);
        }",
        r"function MyComponent(props) {
          useCustomEffect(() => {
            console.log(props.foo);
          });
        }",
        r"function MyComponent(props) {
          useCustomEffect(() => {
            console.log(props.foo);
          }, [props.foo]);
        }",
        r"function MyComponent(props) {
          useCustomEffect(() => {
            console.log(props.foo);
          }, []);
        }",
        r"function MyComponent(props) {
          useWithoutEffectSuffix(() => {
            console.log(props.foo);
          }, []);
        }",
        r"function MyComponent(props) {
          return renderHelperConfusedWithEffect(() => {
            console.log(props.foo);
          }, []);
        }",
        r"const local = {};
        useEffect(() => {
          console.log(local);
        }, []);",
        r"const local1 = {};
        {
          const local2 = {};
          useEffect(() => {
            console.log(local1);
            console.log(local2);
          }, []);
        }",
        r"function MyComponent() {
          const ref = useRef();
          useEffect(() => {
            console.log(ref.current);
          }, [ref]);
        }",
        r"function MyComponent() {
          const ref = useRef();
          useEffect(() => {
            console.log(ref.current);
          }, []);
        }",
        r"function MyComponent({ maybeRef2, foo }) {
          const definitelyRef1 = useRef();
          const definitelyRef2 = useRef();
          const maybeRef1 = useSomeOtherRefyThing();
          const [state1, setState1] = useState();
          const [state2, setState2] = React.useState();
          const [state3, dispatch1] = useReducer();
          const [state4, dispatch2] = React.useReducer();
          const [state5, maybeSetState] = useFunnyState();
          const [state6, maybeDispatch] = useFunnyReducer();
          const [isPending1] = useTransition();
          const [isPending2, startTransition2] = useTransition();
          const [isPending3] = React.useTransition();
          const [isPending4, startTransition4] = React.useTransition();
          const mySetState = useCallback(() => {}, []);
          let myDispatch = useCallback(() => {}, []);

          useEffect(() => {
            // Known to be static
            console.log(definitelyRef1.current);
            console.log(definitelyRef2.current);
            console.log(maybeRef1.current);
            console.log(maybeRef2.current);
            setState1();
            setState2();
            dispatch1();
            dispatch2();
            startTransition1();
            startTransition2();
            startTransition3();
            startTransition4();

            // Dynamic
            console.log(state1);
            console.log(state2);
            console.log(state3);
            console.log(state4);
            console.log(state5);
            console.log(state6);
            console.log(isPending2);
            console.log(isPending4);
            mySetState();
            myDispatch();

            // Not sure; assume dynamic
            maybeSetState();
            maybeDispatch();
          }, [
            // Dynamic
            state1, state2, state3, state4, state5, state6,
            maybeRef1, maybeRef2,
            isPending2, isPending4,

            // Not sure; assume dynamic
            mySetState, myDispatch,
            maybeSetState, maybeDispatch

            // In this test, we don't specify static deps.
            // That should be okay.
          ]);
        }",
        r"function MyComponent({ maybeRef2 }) {
          const definitelyRef1 = useRef();
          const definitelyRef2 = useRef();
          const maybeRef1 = useSomeOtherRefyThing();

          const [state1, setState1] = useState();
          const [state2, setState2] = React.useState();
          const [state3, dispatch1] = useReducer();
          const [state4, dispatch2] = React.useReducer();

          const [state5, maybeSetState] = useFunnyState();
          const [state6, maybeDispatch] = useFunnyReducer();

          const mySetState = useCallback(() => {}, []);
          let myDispatch = useCallback(() => {}, []);

          useEffect(() => {
            // Known to be static
            console.log(definitelyRef1.current);
            console.log(definitelyRef2.current);
            console.log(maybeRef1.current);
            console.log(maybeRef2.current);
            setState1();
            setState2();
            dispatch1();
            dispatch2();

            // Dynamic
            console.log(state1);
            console.log(state2);
            console.log(state3);
            console.log(state4);
            console.log(state5);
            console.log(state6);
            mySetState();
            myDispatch();

            // Not sure; assume dynamic
            maybeSetState();
            maybeDispatch();
          }, [
            // Dynamic
            state1, state2, state3, state4, state5, state6,
            maybeRef1, maybeRef2,

            // Not sure; assume dynamic
            mySetState, myDispatch,
            maybeSetState, maybeDispatch,

            // In this test, we specify static deps.
            // That should be okay too!
            definitelyRef1, definitelyRef2, setState1, setState2, dispatch1, dispatch2
          ]);
        }",
        r"const MyComponent = forwardRef((props, ref) => {
          useImperativeHandle(ref, () => ({
            focus() {
              alert(props.hello);
            }
          }))
        });",
        r"const MyComponent = forwardRef((props, ref) => {
          useImperativeHandle(ref, () => ({
            focus() {
              alert(props.hello);
            }
          }), [props.hello])
        });",
        r"function MyComponent(props) {
          let obj = someFunc();
          useEffect(() => {
            obj.foo = true;
          }, [obj]);
        }",
        r"function MyComponent(props) {
          let foo = {}
          useEffect(() => {
            foo.bar.baz = 43;
          }, [foo.bar]);
        }",
        r"function MyComponent() {
          const myRef = useRef();
          useEffect(() => {
            const handleMove = () => {};
            myRef.current = {};
            return () => {
              console.log(myRef.current.toString())
            };
          }, []);
          return <div />;
        }",
        r"function MyComponent() {
          const myRef = useRef();
          useEffect(() => {
            const handleMove = () => {};
            myRef.current = {};
            return () => {
              console.log(myRef?.current?.toString())
            };
          }, []);
          return <div />;
        }",
        r"function useMyThing(myRef) {
          useEffect(() => {
            const handleMove = () => {};
            myRef.current = {};
            return () => {
              console.log(myRef.current.toString())
            };
          }, [myRef]);
        }",
        // r"function MyComponent() {
        //   const myRef = useRef();
        //   useEffect(() => {
        //     const handleMove = () => {};
        //     const node = myRef.current;
        //     node.addEventListener('mousemove', handleMove);
        //     return () => node.removeEventListener('mousemove', handleMove);
        //   }, []);
        //   return <div ref={myRef} />;
        // }",
        // r"function useMyThing(myRef) {
        //   useEffect(() => {
        //     const handleMove = () => {};
        //     const node = myRef.current;
        //     node.addEventListener('mousemove', handleMove);
        //     return () => node.removeEventListener('mousemove', handleMove);
        //   }, [myRef]);
        //   return <div ref={myRef} />;
        // }",
        r"function useMyThing(myRef) {
          useCallback(() => {
            const handleMouse = () => {};
            myRef.current.addEventListener('mousemove', handleMouse);
            myRef.current.addEventListener('mousein', handleMouse);
            return function() {
              setTimeout(() => {
                myRef.current.removeEventListener('mousemove', handleMouse);
                myRef.current.removeEventListener('mousein', handleMouse);
              });
            }
          }, [myRef]);
        }",
        r"function useMyThing() {
          const myRef = useRef();
          useEffect(() => {
            const handleMove = () => {
              console.log(myRef.current)
            };
            window.addEventListener('mousemove', handleMove);
            return () => window.removeEventListener('mousemove', handleMove);
          }, []);
          return <div ref={myRef} />;
        }",
        r"function useMyThing() {
          const myRef = useRef();
          useEffect(() => {
            const handleMove = () => {
              return () => window.removeEventListener('mousemove', handleMove);
            };
            window.addEventListener('mousemove', handleMove);
            return () => {};
          }, []);
          return <div ref={myRef} />;
        }",
        r"function MyComponent() {
          const local1 = 42;
          const local2 = '42';
          const local3 = null;
          useEffect(() => {
            console.log(local1);
            console.log(local2);
            console.log(local3);
          }, []);
        }",
        r"function MyComponent() {
          const local1 = 42;
          const local2 = '42';
          const local3 = null;
          useEffect(() => {
            console.log(local1);
            console.log(local2);
            console.log(local3);
          }, [local1, local2, local3]);
        }",
        // r"function MyComponent(props) {
        //   const local = props.local;
        //   useEffect(() => {}, [local]);
        // }",
        // r"function Foo({ activeTab }) {
        //   useEffect(() => {
        //     window.scrollTo(0, 0);
        //   }, [activeTab]);
        // }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo.bar.baz);
          }, [props]);
          useEffect(() => {
            console.log(props.foo.bar.baz);
          }, [props.foo]);
          useEffect(() => {
            console.log(props.foo.bar.baz);
          }, [props.foo.bar]);
          useEffect(() => {
            console.log(props.foo.bar.baz);
          }, [props.foo.bar.baz]);
        }",
        r"function MyComponent(props) {
          const fn = useCallback(() => {
            console.log(props.foo.bar.baz);
          }, [props]);
          const fn2 = useCallback(() => {
            console.log(props.foo.bar.baz);
          }, [props.foo]);
          const fn3 = useMemo(() => {
            console.log(props.foo.bar.baz);
          }, [props.foo.bar]);
          const fn4 = useMemo(() => {
            console.log(props.foo.bar.baz);
          }, [props.foo.bar.baz]);
        }",
        r"function MyComponent(props) {
          function handleNext1() {
            console.log('hello');
          }
          const handleNext2 = () => {
            console.log('hello');
          };
          let handleNext3 = function() {
            console.log('hello');
          };
          useEffect(() => {
            return Store.subscribe(handleNext1);
          }, []);
          useLayoutEffect(() => {
            return Store.subscribe(handleNext2);
          }, []);
          useMemo(() => {
            return Store.subscribe(handleNext3);
          }, []);
        }",
        r"function MyComponent(props) {
          function handleNext() {
            console.log('hello');
          }
          useEffect(() => {
            return Store.subscribe(handleNext);
          }, []);
          useLayoutEffect(() => {
            return Store.subscribe(handleNext);
          }, []);
          useMemo(() => {
            return Store.subscribe(handleNext);
          }, []);
        }",
        r"function MyComponent(props) {
          let [, setState] = useState();
          let [, dispatch] = React.useReducer();

          function handleNext1(value) {
            let value2 = value * 100;
            setState(value2);
            console.log('hello');
          }
          const handleNext2 = (value) => {
            setState(foo(value));
            console.log('hello');
          };
          let handleNext3 = function(value) {
            console.log(value);
            dispatch({ type: 'x', value });
          };
          useEffect(() => {
            return Store.subscribe(handleNext1);
          }, []);
          useLayoutEffect(() => {
            return Store.subscribe(handleNext2);
          }, []);
          useMemo(() => {
            return Store.subscribe(handleNext3);
          }, []);
        }",
        // r"function useInterval(callback, delay) {
        //   const savedCallback = useRef();
        //   useEffect(() => {
        //     savedCallback.current = callback;
        //   });
        //   useEffect(() => {
        //     function tick() {
        //       savedCallback.current();
        //     }
        //     if (delay !== null) {
        //       let id = setInterval(tick, delay);
        //       return () => clearInterval(id);
        //     }
        //   }, [delay]);
        // }",
        // r"function Counter() {
        //   const [count, setCount] = useState(0);

        //   useEffect(() => {
        //     let id = setInterval(() => {
        //       setCount(c => c + 1);
        //     }, 1000);
        //     return () => clearInterval(id);
        //   }, []);

        //   return <h1>{count}</h1>;
        // }",
        // r"function Counter(unstableProp) {
        //   let [count, setCount] = useState(0);
        //   setCount = unstableProp
        //   useEffect(() => {
        //     let id = setInterval(() => {
        //       setCount(c => c + 1);
        //     }, 1000);
        //     return () => clearInterval(id);
        //   }, [setCount]);

        //   return <h1>{count}</h1>;
        // }",
        // r"function Counter() {
        //   const [count, setCount] = useState(0);

        //   function tick() {
        //     setCount(c => c + 1);
        //   }

        //   useEffect(() => {
        //     let id = setInterval(() => {
        //       tick();
        //     }, 1000);
        //     return () => clearInterval(id);
        //   }, []);

        //   return <h1>{count}</h1>;
        // }",
        // r"function Counter() {
        //   const [count, dispatch] = useReducer((state, action) => {
        //     if (action === 'inc') {
        //       return state + 1;
        //     }
        //   }, 0);

        //   useEffect(() => {
        //     let id = setInterval(() => {
        //       dispatch('inc');
        //     }, 1000);
        //     return () => clearInterval(id);
        //   }, []);

        //   return <h1>{count}</h1>;
        // }",
        // r"function Counter() {
        //   const [count, dispatch] = useReducer((state, action) => {
        //     if (action === 'inc') {
        //       return state + 1;
        //     }
        //   }, 0);

        //   const tick = () => {
        //     dispatch('inc');
        //   };

        //   useEffect(() => {
        //     let id = setInterval(tick, 1000);
        //     return () => clearInterval(id);
        //   }, []);

        //   return <h1>{count}</h1>;
        // }",
        r"function Podcasts() {
          useEffect(() => {
            setPodcasts([]);
          }, []);
          let [podcasts, setPodcasts] = useState(null);
        }",
        r"function withFetch(fetchPodcasts) {
          return function Podcasts({ id }) {
            let [podcasts, setPodcasts] = useState(null);
            useEffect(() => {
              fetchPodcasts(id).then(setPodcasts);
            }, [id]);
          }
        }",
        r"function Podcasts({ id }) {
          let [podcasts, setPodcasts] = useState(null);
          useEffect(() => {
            function doFetch({ fetchPodcasts }) {
              fetchPodcasts(id).then(setPodcasts);
            }
            doFetch({ fetchPodcasts: API.fetchPodcasts });
          }, [id]);
        }",
        r"function Counter() {
          let [count, setCount] = useState(0);

          function increment(x) {
            return x + 1;
          }

          useEffect(() => {
            let id = setInterval(() => {
              setCount(increment);
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function Counter() {
          let [count, setCount] = useState(0);

          function increment(x) {
            return x + 1;
          }

          useEffect(() => {
            let id = setInterval(() => {
              setCount(count => increment(count));
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"import increment from './increment';
        function Counter() {
          let [count, setCount] = useState(0);

          useEffect(() => {
            let id = setInterval(() => {
              setCount(count => count + increment);
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function withStuff(increment) {
          return function Counter() {
            let [count, setCount] = useState(0);

            useEffect(() => {
              let id = setInterval(() => {
                setCount(count => count + increment);
              }, 1000);
              return () => clearInterval(id);
            }, []);

            return <h1>{count}</h1>;
          }
        }",
        r"function App() {
          const [query, setQuery] = useState('react');
          const [state, setState] = useState(null);
          useEffect(() => {
            let ignore = false;
            fetchSomething();
            async function fetchSomething() {
              const result = await (await fetch('http://hn.algolia.com/api/v1/search?query=' + query)).json();
              if (!ignore) setState(result);
            }
            return () => { ignore = true; };
          }, [query]);
          return (
            <>
              <input value={query} onChange={e => setQuery(e.target.value)} />
              {JSON.stringify(state)}
            </>
          );
        }",
        // r"function Example() {
        //   const foo = useCallback(() => {
        //     foo();
        //   }, []);
        // }",
        // r"function Example({ prop }) {
        //   const foo = useCallback(() => {
        //     if (prop) {
        //       foo();
        //     }
        //   }, [prop]);
        // }",
        r"function Hello() {
          const [state, setState] = useState(0);
          useEffect(() => {
            const handleResize = () => setState(window.innerWidth);
            window.addEventListener('resize', handleResize);
            return () => window.removeEventListener('resize', handleResize);
          });
        }",
        r"function Example() {
          useEffect(() => {
            arguments
          }, [])
        }",
        r"function Example() {
          useEffect(() => {
            const bar = () => {
              arguments;
            };
            bar();
          }, [])
        }",
        r"function Example(props) {
          useEffect(() => {
            let topHeight = 0;
            topHeight = props.upperViewHeight;
          }, [props.upperViewHeight]);
        }",
        r"function Example(props) {
          useEffect(() => {
            let topHeight = 0;
            topHeight = props?.upperViewHeight;
          }, [props?.upperViewHeight]);
        }",
        r"function Example(props) {
          useEffect(() => {
            let topHeight = 0;
            topHeight = props?.upperViewHeight;
          }, [props]);
        }",
        r"function useFoo(foo){
          return useMemo(() => foo, [foo]);
        }",
        r"function useFoo(){
          const foo = 'hi!';
          return useMemo(() => foo, [foo]);
        }",
        r"function useFoo(){
          let {foo} = {foo: 1};
          return useMemo(() => foo, [foo]);
        }",
        r"function useFoo(){
          let [foo] = [1];
          return useMemo(() => foo, [foo]);
        }",
        r"function useFoo() {
          const foo = 'fine';
          if (true) {
            // Shadowed variable with constant construction in a nested scope is fine.
            const foo = {};
          }
          return useMemo(() => foo, [foo]);
        }",
        r"function MyComponent({foo}) {
          return useMemo(() => foo, [foo])
        }",
        r"function MyComponent() {
          const foo = true ? 'fine' : 'also fine';
          return useMemo(() => foo, [foo]);
        }",
        r"function MyComponent() {
          useEffect(() => {
            console.log('banana banana banana');
          }, undefined);
        }",
    ];

    let fail = vec![
        r"function MyComponent(props) {
          useCallback(() => {
            console.log(props.foo?.toString());
          }, []);
        }",
        r"function MyComponent(props) {
          useCallback(() => {
            console.log(props.foo?.bar.baz);
          }, []);
        }",
        r"function MyComponent(props) {
          useCallback(() => {
            console.log(props.foo?.bar?.baz);
          }, []);
        }",
        r"function MyComponent(props) {
          useCallback(() => {
            console.log(props.foo?.bar.toString());
          }, []);
        }",
        // r"function MyComponent() {
        //   const local = someFunc();
        //   useEffect(() => {
        //     console.log(local);
        //   }, []);
        // }",
        // r"function Counter(unstableProp) {
        //   let [count, setCount] = useState(0);
        //   setCount = unstableProp
        //   useEffect(() => {
        //     let id = setInterval(() => {
        //       setCount(c => c + 1);
        //     }, 1000);
        //     return () => clearInterval(id);
        //   }, []);

        //   return <h1>{count}</h1>;
        // }",
        // r"function MyComponent() {
        //   let local = 42;
        //   useEffect(() => {
        //     console.log(local);
        //   }, []);
        // }",
        // r"function MyComponent() {
        //   const local = /foo/;
        //   useEffect(() => {
        //     console.log(local);
        //   }, []);
        // }",
        r"function MyComponent(props) {
          const value = useMemo(() => { return 2*2; });
          const fn = useCallback(() => { alert('foo'); });
        }",
        r"function MyComponent({ fn1, fn2 }) {
          const value = useMemo(fn1);
          const fn = useCallback(fn2);
        }",
        r"function MyComponent() {
          useEffect()
          useLayoutEffect()
          useCallback()
          useMemo()
        }",
        r"function MyComponent() {
          const local = someFunc();
          useEffect(() => {
            if (true) {
              console.log(local);
            }
          }, []);
        }",
        // r"function MyComponent() {
        //   const local = {};
        //   useEffect(() => {
        //     try {
        //       console.log(local);
        //     } finally {}
        //   }, []);
        // }",
        // r"function MyComponent() {
        //   const local = {};
        //   useEffect(() => {
        //     function inner() {
        //       console.log(local);
        //     }
        //     inner();
        //   }, []);
        // }",
        r"function MyComponent() {
          const local1 = someFunc();
          {
            const local2 = someFunc();
            useEffect(() => {
              console.log(local1);
              console.log(local2);
            }, []);
          }
        }",
        // r"function MyComponent() {
        //   const local1 = {};
        //   const local2 = {};
        //   useEffect(() => {
        //     console.log(local1);
        //     console.log(local2);
        //   }, [local1]);
        // }",
        // r"function MyComponent() {
        //   const local1 = {};
        //   const local2 = {};
        //   useMemo(() => {
        //     console.log(local1);
        //   }, [local1, local2]);
        // }",
        // r"function MyComponent() {
        //   const local1 = someFunc();
        //   function MyNestedComponent() {
        //     const local2 = {};
        //     useCallback(() => {
        //       console.log(local1);
        //       console.log(local2);
        //     }, [local1]);
        //   }
        // }",
        // r"function MyComponent() {
        //   const local = {};
        //   useEffect(() => {
        //     console.log(local);
        //     console.log(local);
        //   }, []);
        // }",
        // r"function MyComponent() {
        //   const local = {};
        //   useEffect(() => {
        //     console.log(local);
        //     console.log(local);
        //   }, [local, local]);
        // }",
        r"function MyComponent() {
          useCallback(() => {}, [window]);
        }",
        r"function MyComponent(props) {
          let local = props.foo;
          useCallback(() => {}, [local]);
        }",
        r"function MyComponent({ history }) {
          useEffect(() => {
            return history.listen();
          }, []);
        }",
        r"function MyComponent({ history }) {
          useEffect(() => {
            return [
              history.foo.bar[2].dobedo.listen(),
              history.foo.bar().dobedo.listen[2]
            ];
          }, []);
        }",
        r"function MyComponent({ history }) {
          useEffect(() => {
            return [
              history?.foo
            ];
          }, []);
        }",
        r"function MyComponent() {
          useEffect(() => {}, ['foo']);
        }",
        r"function MyComponent({ foo, bar, baz }) {
          useEffect(() => {
            console.log(foo, bar, baz);
          }, ['foo', 'bar']);
        }",
        r"function MyComponent({ foo, bar, baz }) {
          useEffect(() => {
            console.log(foo, bar, baz);
          }, [42, false, null]);
        }",
        r"function MyComponent() {
          const dependencies = [];
          useEffect(() => {}, dependencies);
        }",
        r"function MyComponent() {
          const local = {};
          const dependencies = [local];
          useEffect(() => {
            console.log(local);
          }, dependencies);
        }",
        r"function MyComponent() {
          const local = {};
          const dependencies = [local];
          useEffect(() => {
            console.log(local);
          }, [...dependencies]);
        }",
        r"function MyComponent() {
          const local = someFunc();
          useEffect(() => {
            console.log(local);
          }, [local, ...dependencies]);
        }",
        r"function MyComponent() {
          const local = {};
          useEffect(() => {
            console.log(local);
          }, [computeCacheKey(local)]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.items[0]);
          }, [props.items[0]]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.items[0]);
          }, [props.items, props.items[0]]);
        }",
        r"function MyComponent({ items }) {
          useEffect(() => {
            console.log(items[0]);
          }, [items[0]]);
        }",
        r"function MyComponent({ items }) {
          useEffect(() => {
            console.log(items[0]);
          }, [items, items[0]]);
        }",
        r"function MyComponent(props) {
          const local = {};
          useCallback(() => {
            console.log(props.foo);
            console.log(props.bar);
          }, [props, props.foo]);
        }",
        r"function MyComponent(props) {
          const local = {};
          useCallback(() => {
            console.log(props.foo);
            console.log(props.bar);
          }, []);
        }",
        r"function MyComponent() {
          const local = {id: 42};
          useEffect(() => {
            console.log(local);
          }, [local.id]);
        }",
        r"function MyComponent() {
          const local = {id: 42};
          const fn = useCallback(() => {
            console.log(local);
          }, [local.id]);
        }",
        r"function MyComponent() {
          const local = {id: 42};
          const fn = useCallback(() => {
            console.log(local);
          }, [local.id, local]);
        }",
        r"function MyComponent(props) {
          const fn = useCallback(() => {
            console.log(props.foo.bar.baz);
          }, []);
        }",
        r"function MyComponent(props) {
          let color = {}
          const fn = useCallback(() => {
            console.log(props.foo.bar.baz);
            console.log(color);
          }, [props.foo, props.foo.bar.baz]);
        }",
        r"function MyComponent(props) {
          const fn = useCallback(() => {
            console.log(props.foo.bar.baz);
          }, [props.foo.bar.baz, props.foo]);
        }",
        r"function MyComponent(props) {
          const fn = useCallback(() => {
            console.log(props.foo.bar.baz);
            console.log(props.foo.fizz.bizz);
          }, []);
        }",
        r"function MyComponent(props) {
          const fn = useCallback(() => {
            console.log(props.foo.bar);
          }, [props.foo.bar.baz]);
        }",
        r"function MyComponent(props) {
          const fn = useCallback(() => {
            console.log(props);
            console.log(props.hello);
          }, [props.foo.bar.baz]);
        }",
        r"function MyComponent() {
          const local = {};
          useEffect(() => {
            console.log(local);
          }, [local, local]);
        }",
        r"function MyComponent() {
          const local1 = {};
          useCallback(() => {
            const local1 = {};
            console.log(local1);
          }, [local1]);
        }",
        r"function MyComponent() {
          const local1 = {};
          useCallback(() => {}, [local1]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo);
          }, []);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo);
            console.log(props.bar);
          }, []);
        }",
        r"function MyComponent(props) {
          let a, b, c, d, e, f, g;
          useEffect(() => {
            console.log(b, e, d, c, a, g, f);
          }, [c, a, g]);
        }",
        r"function MyComponent(props) {
          let a, b, c, d, e, f, g;
          useEffect(() => {
            console.log(b, e, d, c, a, g, f);
          }, [a, c, g]);
        }",
        r"function MyComponent(props) {
          let a, b, c, d, e, f, g;
          useEffect(() => {
            console.log(b, e, d, c, a, g, f);
          }, []);
        }",
        r"function MyComponent(props) {
          const local = {};
          useEffect(() => {
            console.log(props.foo);
            console.log(props.bar);
            console.log(local);
          }, []);
        }",
        r"function MyComponent(props) {
          const local = {};
          useEffect(() => {
            console.log(props.foo);
            console.log(props.bar);
            console.log(local);
          }, [props]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            console.log(props.foo);
          }, []);
          useCallback(() => {
            console.log(props.foo);
          }, []);
          useMemo(() => {
            console.log(props.foo);
          }, []);
          React.useEffect(() => {
            console.log(props.foo);
          }, []);
          React.useCallback(() => {
            console.log(props.foo);
          }, []);
          React.useMemo(() => {
            console.log(props.foo);
          }, []);
          React.notReactiveHook(() => {
            console.log(props.foo);
          }, []);
        }",
        r"function MyComponent(props) {
          useCustomEffect(() => {
            console.log(props.foo);
          }, []);
          useEffect(() => {
            console.log(props.foo);
          }, []);
          React.useEffect(() => {
            console.log(props.foo);
          }, []);
          React.useCustomEffect(() => {
            console.log(props.foo);
          }, []);
        }",
        r"function MyComponent() {
          const local = {};
          useEffect(() => {
            console.log(local);
          }, [a ? local : b]);
        }",
        r"function MyComponent() {
          const local = {};
          useEffect(() => {
            console.log(local);
          }, [a && local]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {}, [props?.attribute.method()]);
        }",
        r"function MyComponent(props) {
          useEffect(() => {}, [props.method()]);
        }",
        r"function MyComponent() {
          const ref = useRef();
          const [state, setState] = useState();
          useEffect(() => {
            ref.current = {};
            setState(state + 1);
          }, []);
        }",
        r"function MyComponent() {
          const ref = useRef();
          const [state, setState] = useState();
          useEffect(() => {
            ref.current = {};
            setState(state + 1);
          }, [ref]);
        }",
        r"function MyComponent(props) {
          const ref1 = useRef();
          const ref2 = useRef();
          useEffect(() => {
            ref1.current.focus();
            console.log(ref2.current.textContent);
            alert(props.someOtherRefs.current.innerHTML);
            fetch(props.color);
          }, []);
        }",
        r"function MyComponent(props) {
          const ref1 = useRef();
          const ref2 = useRef();
          useEffect(() => {
            ref1.current.focus();
            console.log(ref2.current.textContent);
            alert(props.someOtherRefs.current.innerHTML);
            fetch(props.color);
          }, [ref1.current, ref2.current, props.someOtherRefs, props.color]);
        }",
        r"function MyComponent(props) {
          const ref1 = useRef();
          const ref2 = useRef();
          useEffect(() => {
            ref1?.current?.focus();
            console.log(ref2?.current?.textContent);
            alert(props.someOtherRefs.current.innerHTML);
            fetch(props.color);
          }, [ref1?.current, ref2?.current, props.someOtherRefs, props.color]);
        }",
        r"function MyComponent() {
          const ref = useRef();
          useEffect(() => {
            console.log(ref.current);
          }, [ref.current]);
        }",
        r"function MyComponent({ activeTab }) {
          const ref1 = useRef();
          const ref2 = useRef();
          useEffect(() => {
            ref1.current.scrollTop = 0;
            ref2.current.scrollTop = 0;
          }, [ref1.current, ref2.current, activeTab]);
        }",
        r"function MyComponent({ activeTab, initY }) {
          const ref1 = useRef();
          const ref2 = useRef();
          const fn = useCallback(() => {
            ref1.current.scrollTop = initY;
            ref2.current.scrollTop = initY;
          }, [ref1.current, ref2.current, activeTab, initY]);
        }",
        r"function MyComponent() {
          const ref = useRef();
          useEffect(() => {
            console.log(ref.current);
          }, [ref.current, ref]);
        }",
        r"const MyComponent = forwardRef((props, ref) => {
          useImperativeHandle(ref, () => ({
            focus() {
              alert(props.hello);
            }
          }), [])
        });",
        r"function MyComponent(props) {
          useEffect(() => {
            if (props.onChange) {
              props.onChange();
            }
          }, []);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            if (props?.onChange) {
              props?.onChange();
            }
          }, []);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            function play() {
              props.onPlay();
            }
            function pause() {
              props.onPause();
            }
          }, []);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            if (props.foo.onChange) {
              props.foo.onChange();
            }
          }, []);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            props.onChange();
            if (props.foo.onChange) {
              props.foo.onChange();
            }
          }, []);
        }",
        r"function MyComponent(props) {
          const [skillsCount] = useState();
          useEffect(() => {
            if (skillsCount === 0 && !props.isEditMode) {
              props.toggleEditMode();
            }
          }, [skillsCount, props.isEditMode, props.toggleEditMode]);
        }",
        r"function MyComponent(props) {
          const [skillsCount] = useState();
          useEffect(() => {
            if (skillsCount === 0 && !props.isEditMode) {
              props.toggleEditMode();
            }
          }, []);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            externalCall(props);
            props.onChange();
          }, []);
        }",
        r"function MyComponent(props) {
          useEffect(() => {
            props.onChange();
            externalCall(props);
          }, []);
        }",
        r"function MyComponent(props) {
          let value;
          let value2;
          let value3;
          let value4;
          let asyncValue;
          useEffect(() => {
            if (value4) {
              value = {};
            }
            value2 = 100;
            value = 43;
            value4 = true;
            console.log(value2);
            console.log(value3);
            setTimeout(() => {
              asyncValue = 100;
            });
          }, []);
        }",
        r"function MyComponent(props) {
          let value;
          let value2;
          let value3;
          let asyncValue;
          useEffect(() => {
            value = {};
            value2 = 100;
            value = 43;
            console.log(value2);
            console.log(value3);
            setTimeout(() => {
              asyncValue = 100;
            });
          }, [value, value2, value3]);
        }",
        r"function MyComponent() {
          const myRef = useRef();
          useEffect(() => {
            const handleMove = () => {};
            myRef.current.addEventListener('mousemove', handleMove);
            return () => myRef.current.removeEventListener('mousemove', handleMove);
          }, []);
          return <div ref={myRef} />;
        }",
        r"function MyComponent() {
          const myRef = useRef();
          useEffect(() => {
            const handleMove = () => {};
            myRef?.current?.addEventListener('mousemove', handleMove);
            return () => myRef?.current?.removeEventListener('mousemove', handleMove);
          }, []);
          return <div ref={myRef} />;
        }",
        r"function MyComponent() {
          const myRef = useRef();
          useEffect(() => {
            const handleMove = () => {};
            myRef.current.addEventListener('mousemove', handleMove);
            return () => myRef.current.removeEventListener('mousemove', handleMove);
          });
          return <div ref={myRef} />;
        }",
        r"function useMyThing(myRef) {
          useEffect(() => {
            const handleMove = () => {};
            myRef.current.addEventListener('mousemove', handleMove);
            return () => myRef.current.removeEventListener('mousemove', handleMove);
          }, [myRef]);
        }",
        r"function useMyThing(myRef) {
          useEffect(() => {
            const handleMouse = () => {};
            myRef.current.addEventListener('mousemove', handleMouse);
            myRef.current.addEventListener('mousein', handleMouse);
            return function() {
              setTimeout(() => {
                myRef.current.removeEventListener('mousemove', handleMouse);
                myRef.current.removeEventListener('mousein', handleMouse);
              });
            }
          }, [myRef]);
        }",
        r"function useMyThing(myRef, active) {
          useEffect(() => {
            const handleMove = () => {};
            if (active) {
              myRef.current.addEventListener('mousemove', handleMove);
              return function() {
                setTimeout(() => {
                  myRef.current.removeEventListener('mousemove', handleMove);
                });
              }
            }
          }, [myRef, active]);
        }",
        r"function MyComponent() {
                  const myRef = useRef();
                  useLayoutEffect_SAFE_FOR_SSR(() => {
                    const handleMove = () => {};
                    myRef.current.addEventListener('mousemove', handleMove);
                    return () => myRef.current.removeEventListener('mousemove', handleMove);
                  });
                  return <div ref={myRef} />;
                }",
        r"function MyComponent() {
          const local1 = 42;
          const local2 = '42';
          const local3 = null;
          const local4 = {};
          useEffect(() => {
            console.log(local1);
            console.log(local2);
            console.log(local3);
            console.log(local4);
          }, [local1, local3]);
        }",
        r"function MyComponent() {
          useEffect(() => {
            window.scrollTo(0, 0);
          }, [window]);
        }",
        r"import MutableStore from 'store';

        function MyComponent() {
          useEffect(() => {
            console.log(MutableStore.hello);
          }, [MutableStore.hello]);
        }",
        r"import MutableStore from 'store';
        let z = {};

        function MyComponent(props) {
          let x = props.foo;
          {
            let y = props.bar;
            useEffect(() => {
              console.log(MutableStore.hello.world, props.foo, x, y, z, global.stuff);
            }, [MutableStore.hello.world, props.foo, x, y, z, global.stuff]);
          }
        }",
        r"import MutableStore from 'store';
        let z = {};

        function MyComponent(props) {
          let x = props.foo;
          {
            let y = props.bar;
            useEffect(() => {
              // nothing
            }, [MutableStore.hello.world, props.foo, x, y, z, global.stuff]);
          }
        }",
        r"import MutableStore from 'store';
        let z = {};

        function MyComponent(props) {
          let x = props.foo;
          {
            let y = props.bar;
            const fn = useCallback(() => {
              // nothing
            }, [MutableStore.hello.world, props.foo, x, y, z, global.stuff]);
          }
        }",
        r"import MutableStore from 'store';
        let z = {};

        function MyComponent(props) {
          let x = props.foo;
          {
            let y = props.bar;
            const fn = useCallback(() => {
              // nothing
            }, [MutableStore?.hello?.world, props.foo, x, y, z, global?.stuff]);
          }
        }",
        r"function MyComponent(props) {
          let [, setState] = useState();
          let [, dispatch] = React.useReducer();
          let taint = props.foo;

          function handleNext1(value) {
            let value2 = value * taint;
            setState(value2);
            console.log('hello');
          }
          const handleNext2 = (value) => {
            setState(taint(value));
            console.log('hello');
          };
          let handleNext3 = function(value) {
            setTimeout(() => console.log(taint));
            dispatch({ type: 'x', value });
          };
          useEffect(() => {
            return Store.subscribe(handleNext1);
          }, []);
          useLayoutEffect(() => {
            return Store.subscribe(handleNext2);
          }, []);
          useMemo(() => {
            return Store.subscribe(handleNext3);
          }, []);
        }",
        r"function MyComponent(props) {
          let [, setState] = useState();
          let [, dispatch] = React.useReducer();
          let taint = props.foo;

          // Shouldn't affect anything
          function handleChange() {}

          function handleNext1(value) {
            let value2 = value * taint;
            setState(value2);
            console.log('hello');
          }
          const handleNext2 = (value) => {
            setState(taint(value));
            console.log('hello');
          };
          let handleNext3 = function(value) {
            console.log(taint);
            dispatch({ type: 'x', value });
          };
          useEffect(() => {
            return Store.subscribe(handleNext1);
          }, []);
          useLayoutEffect(() => {
            return Store.subscribe(handleNext2);
          }, []);
          useMemo(() => {
            return Store.subscribe(handleNext3);
          }, []);
        }",
        r"function MyComponent(props) {
          let [, setState] = useState();
          let [, dispatch] = React.useReducer();
          let taint = props.foo;

          // Shouldn't affect anything
          const handleChange = () => {};

          function handleNext1(value) {
            let value2 = value * taint;
            setState(value2);
            console.log('hello');
          }
          const handleNext2 = (value) => {
            setState(taint(value));
            console.log('hello');
          };
          let handleNext3 = function(value) {
            console.log(taint);
            dispatch({ type: 'x', value });
          };
          useEffect(() => {
            return Store.subscribe(handleNext1);
          }, []);
          useLayoutEffect(() => {
            return Store.subscribe(handleNext2);
          }, []);
          useMemo(() => {
            return Store.subscribe(handleNext3);
          }, []);
        }",
        r"function MyComponent(props) {
          let [, setState] = useState();

          function handleNext(value) {
            setState(value);
          }

          useEffect(() => {
            return Store.subscribe(handleNext);
          }, [handleNext]);
        }",
        r"function MyComponent(props) {
          let [, setState] = useState();

          const handleNext = (value) => {
            setState(value);
          };

          useEffect(() => {
            return Store.subscribe(handleNext);
          }, [handleNext]);
        }",
        r"function MyComponent(props) {
          let [, setState] = useState();

          const handleNext = (value) => {
            setState(value);
          };

          useEffect(() => {
            return Store.subscribe(handleNext);
          }, [handleNext]);

          return <div onClick={handleNext} />;
        }",
        r"function MyComponent(props) {
          function handleNext1() {
            console.log('hello');
          }
          const handleNext2 = () => {
            console.log('hello');
          };
          let handleNext3 = function() {
            console.log('hello');
          };
          useEffect(() => {
            return Store.subscribe(handleNext1);
          }, [handleNext1]);
          useLayoutEffect(() => {
            return Store.subscribe(handleNext2);
          }, [handleNext2]);
          useMemo(() => {
            return Store.subscribe(handleNext3);
          }, [handleNext3]);
        }",
        r"function MyComponent(props) {
          function handleNext1() {
            console.log('hello');
          }
          const handleNext2 = () => {
            console.log('hello');
          };
          let handleNext3 = function() {
            console.log('hello');
          };
          useEffect(() => {
            handleNext1();
            return Store.subscribe(() => handleNext1());
          }, [handleNext1]);
          useLayoutEffect(() => {
            handleNext2();
            return Store.subscribe(() => handleNext2());
          }, [handleNext2]);
          useMemo(() => {
            handleNext3();
            return Store.subscribe(() => handleNext3());
          }, [handleNext3]);
        }",
        r"function MyComponent(props) {
          function handleNext1() {
            console.log('hello');
          }
          const handleNext2 = () => {
            console.log('hello');
          };
          let handleNext3 = function() {
            console.log('hello');
          };
          useEffect(() => {
            handleNext1();
            return Store.subscribe(() => handleNext1());
          }, [handleNext1]);
          useLayoutEffect(() => {
            handleNext2();
            return Store.subscribe(() => handleNext2());
          }, [handleNext2]);
          useMemo(() => {
            handleNext3();
            return Store.subscribe(() => handleNext3());
          }, [handleNext3]);
          return (
            <div
              onClick={() => {
                handleNext1();
                setTimeout(handleNext2);
                setTimeout(() => {
                  handleNext3();
                });
              }}
            />
          );
        }",
        r"function MyComponent(props) {
          const handleNext1 = () => {
            console.log('hello');
          };
          function handleNext2() {
            console.log('hello');
          }
          useEffect(() => {
            return Store.subscribe(handleNext1);
            return Store.subscribe(handleNext2);
          }, [handleNext1, handleNext2]);
          useEffect(() => {
            return Store.subscribe(handleNext1);
            return Store.subscribe(handleNext2);
          }, [handleNext1, handleNext2]);
        }",
        r"function MyComponent(props) {
          let handleNext = () => {
            console.log('hello');
          };
          if (props.foo) {
            handleNext = () => {
              console.log('hello');
            };
          }
          useEffect(() => {
            return Store.subscribe(handleNext);
          }, [handleNext]);
        }",
        r"function MyComponent(props) {
          let [, setState] = useState();
          let taint = props.foo;

          function handleNext(value) {
            let value2 = value * taint;
            setState(value2);
            console.log('hello');
          }

          useEffect(() => {
            return Store.subscribe(handleNext);
          }, [handleNext]);
        }",
        r"function Counter() {
          let [count, setCount] = useState(0);

          useEffect(() => {
            let id = setInterval(() => {
              setCount(count + 1);
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function Counter() {
          let [count, setCount] = useState(0);
          let [increment, setIncrement] = useState(0);

          useEffect(() => {
            let id = setInterval(() => {
              setCount(count + increment);
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function Counter() {
          let [count, setCount] = useState(0);
          let [increment, setIncrement] = useState(0);

          useEffect(() => {
            let id = setInterval(() => {
              setCount(count => count + increment);
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function Counter() {
          let [count, setCount] = useState(0);
          let increment = useCustomHook();

          useEffect(() => {
            let id = setInterval(() => {
              setCount(count => count + increment);
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function Counter({ step }) {
          let [count, setCount] = useState(0);

          function increment(x) {
            return x + step;
          }

          useEffect(() => {
            let id = setInterval(() => {
              setCount(count => increment(count));
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function Counter({ step }) {
          let [count, setCount] = useState(0);

          function increment(x) {
            return x + step;
          }

          useEffect(() => {
            let id = setInterval(() => {
              setCount(count => increment(count));
            }, 1000);
            return () => clearInterval(id);
          }, [increment]);

          return <h1>{count}</h1>;
        }",
        r"function Counter({ increment }) {
          let [count, setCount] = useState(0);

          useEffect(() => {
            let id = setInterval(() => {
              setCount(count => count + increment);
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function Counter() {
          const [count, setCount] = useState(0);

          function tick() {
            setCount(count + 1);
          }

          useEffect(() => {
            let id = setInterval(() => {
              tick();
            }, 1000);
            return () => clearInterval(id);
          }, []);

          return <h1>{count}</h1>;
        }",
        r"function Podcasts() {
          useEffect(() => {
            alert(podcasts);
          }, []);
          let [podcasts, setPodcasts] = useState(null);
        }",
        r"function Podcasts({ fetchPodcasts, id }) {
          let [podcasts, setPodcasts] = useState(null);
          useEffect(() => {
            fetchPodcasts(id).then(setPodcasts);
          }, [id]);
        }",
        r"function Podcasts({ api: { fetchPodcasts }, id }) {
          let [podcasts, setPodcasts] = useState(null);
          useEffect(() => {
            fetchPodcasts(id).then(setPodcasts);
          }, [id]);
        }",
        r"function Podcasts({ fetchPodcasts, fetchPodcasts2, id }) {
          let [podcasts, setPodcasts] = useState(null);
          useEffect(() => {
            setTimeout(() => {
              console.log(id);
              fetchPodcasts(id).then(setPodcasts);
              fetchPodcasts2(id).then(setPodcasts);
            });
          }, [id]);
        }",
        r"function Podcasts({ fetchPodcasts, id }) {
          let [podcasts, setPodcasts] = useState(null);
          useEffect(() => {
            console.log(fetchPodcasts);
            fetchPodcasts(id).then(setPodcasts);
          }, [id]);
        }",
        r"function Podcasts({ fetchPodcasts, id }) {
          let [podcasts, setPodcasts] = useState(null);
          useEffect(() => {
            console.log(fetchPodcasts);
            fetchPodcasts?.(id).then(setPodcasts);
          }, [id]);
        }",
        r"function Thing() {
          useEffect(() => {
            const fetchData = async () => {};
            fetchData();
          }, [fetchData]);
        }",
        r"function Hello() {
          const [state, setState] = useState(0);
          useEffect(() => {
            setState({});
          });
        }",
        r"function Hello() {
          const [data, setData] = useState(0);
          useEffect(() => {
            fetchData.then(setData);
          });
        }",
        r"function Hello({ country }) {
          const [data, setData] = useState(0);
          useEffect(() => {
            fetchData(country).then(setData);
          });
        }",
        r"function Hello({ prop1, prop2 }) {
          const [state, setState] = useState(0);
          useEffect(() => {
            if (prop1) {
              setState(prop2);
            }
          });
        }",
        r"function Thing() {
          useEffect(async () => {}, []);
        }",
        r"function Thing() {
          useEffect(async () => {});
        }",
        r"function Example() {
          const foo = useCallback(() => {
            foo();
          }, [foo]);
        }",
        r"function Example({ prop }) {
          const foo = useCallback(() => {
            prop.hello(foo);
          }, [foo]);
          const bar = useCallback(() => {
            foo();
          }, [foo]);
        }",
        r"function MyComponent() {
          const local = {};
          function myEffect() {
            console.log(local);
          }
          useEffect(myEffect, []);
        }",
        r"function MyComponent() {
          const local = {};
          const myEffect = () => {
            console.log(local);
          };
          useEffect(myEffect, []);
        }",
        r"function MyComponent() {
          const local = {};
          const myEffect = function() {
            console.log(local);
          };
          useEffect(myEffect, []);
        }",
        r"function MyComponent() {
          const local = {};
          const myEffect = () => {
            otherThing();
          };
          const otherThing = () => {
            console.log(local);
          };
          useEffect(myEffect, []);
        }",
        r"function MyComponent() {
          const local = {};
          const myEffect = debounce(() => {
            console.log(local);
          }, delay);
          useEffect(myEffect, []);
        }",
        r"function MyComponent() {
          const local = {};
          const myEffect = debounce(() => {
            console.log(local);
          }, delay);
          useEffect(myEffect, [local]);
        }",
        r"function MyComponent({myEffect}) {
          useEffect(myEffect, []);
        }",
        r"function MyComponent() {
          const local = {};
          useEffect(debounce(() => {
            console.log(local);
          }, delay), []);
        }",
        r"function MyComponent() {
          const local = {};
          useEffect(() => {
            console.log(local);
          }, []);
        }",
        r"function MyComponent(props) {
          let foo = {}
          useEffect(() => {
            foo.bar.baz = 43;
            props.foo.bar.baz = 1;
          }, []);
        }",
        r"function Component() {
          const foo = {};
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = [];
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = () => {};
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = function bar(){};
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = class {};
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = true ? {} : 'fine';
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = bar || {};
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = bar ?? {};
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = bar && {};
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = bar ? baz ? {} : null : null;
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          let foo = {};
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          var foo = {};
          useMemo(() => foo, [foo]);
        }",
        r"function Component() {
          const foo = {};
          useCallback(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Component() {
          const foo = {};
          useEffect(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Component() {
          const foo = {};
          useLayoutEffect(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Component() {
          const foo = {};
          useImperativeHandle(
            ref,
            () => {
               console.log(foo);
            },
            [foo]
          );
        }",
        r"function Foo(section) {
          const foo = section.section_components?.edges ?? [];
          useEffect(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Foo(section) {
          const foo = {};
          console.log(foo);
          useMemo(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Foo() {
          const foo = <>Hi!</>;
          useMemo(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Foo() {
          const foo = <div>Hi!</div>;
          useMemo(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Foo() {
          const foo = bar = {};
          useMemo(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Foo() {
          const foo = new String('foo'); // Note 'foo' will be boxed, and thus an object and thus compared by reference.
          useMemo(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Foo() {
          const foo = new Map([]);
          useMemo(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Foo() {
          const foo = /reg/;
          useMemo(() => {
            console.log(foo);
          }, [foo]);
        }",
        r"function Foo() {
          class Bar {};
          useMemo(() => {
            console.log(new Bar());
          }, [Bar]);
        }",
        r"function Foo() {
          const foo = {};
          useLayoutEffect(() => {
            console.log(foo);
          }, [foo]);
          useEffect(() => {
            console.log(foo);
          }, [foo]);
        }",
    ];

    Tester::new(ExhaustiveDeps::NAME, pass, fail).test_and_snapshot();
}
