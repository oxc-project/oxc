use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNodes;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::AssignmentOperator;
use serde_json::Value;

// Update the import here to get AstNode from oxc_syntax::node
use crate::{
    ast_util::is_function_node,
    context::LintContext,
    rule::Rule,
    AstNode
};

/// Variant to control which decision points are counted.
/// In the "classic" variant (the default), switch statements do not
/// add a decision point for the entire switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ComplexityVariant {
    Classic,
    Modified,
}

impl Default for ComplexityVariant {
    fn default() -> Self {
        ComplexityVariant::Classic
    }
}

/// The cyclomatic complexity rule. It reports a warning if a function’s
/// complexity (i.e. number of independent paths) exceeds the allowed maximum.
#[derive(Debug, Default, Clone)]
pub struct Complexity {
    max: usize,
    variant: ComplexityVariant,
}

/// Creates a diagnostic message for excessive complexity.
fn complexity_diagnostic(name: &str, complexity: usize, max: usize, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "{} has a complexity of {}. Maximum allowed is {}.",
        name, complexity, max
    ))
    .with_help("Consider refactoring your code to reduce its complexity.")
    .with_label(span)
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a maximum cyclomatic complexity allowed in a program.
    ///
    /// ### Why is this bad?
    ///
    /// High cyclomatic complexity makes code harder to understand, maintain, and test.
    ///
    /// ### Examples
    ///
    /// For example, with the default `{ "max": 20 }` option:
    /// ```js
    /// function foo(a) {
    ///     if (a) {            // +1
    ///         while(b) {      // +1
    ///             do {       // +1
    ///                 // ...
    ///             } while(c);
    ///         }
    ///     }
    /// }
    /// // Base complexity is 1; this function has complexity 4.
    /// ```
    ///
    /// ### Options
    ///
    /// #### max
    ///
    /// `{ type: number, default: 20 }`
    ///
    /// Maximum allowed cyclomatic complexity.
    Complexity,
    eslint,
    pedantic
);

impl Rule for Complexity {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Only process function-like nodes.
        if !is_function_node(node) {
            return;
        }

        let nodes = ctx.semantic().nodes();
        let complexity = compute_complexity(node, nodes, self.variant);
        if complexity > self.max {
            let name = get_function_name(node);
            ctx.diagnostic(complexity_diagnostic(&name, complexity, self.max, node.span()));
        }
    }

    fn from_configuration(value: Value) -> Self {
        // The configuration can be either a number or an object with a "max" property.
        let config = value.get(0);
        let max = if let Some(num) = config.and_then(Value::as_u64) {
            num as usize
        } else {
            config
                .and_then(|config| config.get("max"))
                .and_then(Value::as_u64)
                .map_or(0, |v| v as usize)
        };

        // Optionally, a "variant" can be provided ("classic" [default] or "modified").
        let variant = config
            .and_then(|config| config.get("variant"))
            .and_then(Value::as_str)
            .map(|s| {
                if s == "modified" {
                    ComplexityVariant::Modified
                } else {
                    ComplexityVariant::Classic
                }
            })
            .unwrap_or(ComplexityVariant::Classic);

        Self { max, variant }
    }
}

/// Computes the cyclomatic complexity for a given function node.
/// It starts at 1 and adds 1 for every decision point encountered
/// in nodes that are directly within the function (excluding nested functions).
fn compute_complexity<'a>(
    function_node: &AstNode<'a>,
    nodes: &'a AstNodes<'a>,
    variant: ComplexityVariant,
) -> usize {
    let mut complexity = 1; // initial path
    for node in nodes.iter() {
        // Skip the function node itself.
        if node.id() == function_node.id() {
            continue;
        }
        // Count the node only if the first encountered function ancestor is `function_node`.
        if is_in_function(function_node, node, nodes) && is_decision_point(node, variant) {
            complexity += 1;
        }
    }
    complexity
}

/// Returns true if the given `node` is directly inside the specified function.
/// That is, among its ancestors, the first function node encountered must be `function_node`.
fn is_in_function<'a>(
    function_node: &AstNode<'a>,
    node: &AstNode<'a>,
    nodes: &AstNodes<'a>,
) -> bool {
    for ancestor in nodes.ancestors(node.id()) {
        if is_function_node(ancestor) {
            return ancestor.id() == function_node.id();
        }
    }
    false
}

/// Returns true if the given node is a decision point that increases cyclomatic complexity.
///
/// The following AST node kinds (if present) contribute to the complexity:
/// - IfStatement, ForStatement, ForInStatement, ForOfStatement, WhileStatement, DoWhileStatement
/// - ConditionalExpression, LogicalExpression, CatchClause, AssignmentPattern
/// - SwitchCase (if a test is present)
/// - SwitchStatement (only if the variant is "modified")
/// - In an AssignmentExpression, if the operator is a logical assignment operator
/// - In a MemberExpression or CallExpression, if the optional chaining flag is set
fn is_decision_point(node: &AstNode, variant: ComplexityVariant) -> bool {
    match node.kind() {
        AstKind::IfStatement(_) => true,
        AstKind::ForStatement(_) => true,
        AstKind::ForInStatement(_) => true,
        AstKind::ForOfStatement(_) => true,
        AstKind::WhileStatement(_) => true,
        AstKind::DoWhileStatement(_) => true,
        AstKind::ConditionalExpression(_) => true,
        AstKind::LogicalExpression(_) => true,
        AstKind::CatchClause(_) => true,
        AstKind::AssignmentPattern(_) => true,
        AstKind::SwitchCase(switch_case) => {
            // Count only non-default cases (i.e. if there is a test expression).
            switch_case.test.is_some()
        }
        AstKind::SwitchStatement(_) => {
            // In the modified variant, also count the switch statement itself.
            variant == ComplexityVariant::Modified
        }
        AstKind::AssignmentExpression(assign_expr) => {
            // If the assignment operator has short-circuiting behavior,
            // count it as a decision point.
            is_logical_assignment_operator(assign_expr.operator)
        }
        AstKind::MemberExpression(member_expr) => {
            // Count optional chaining.
            member_expr.optional()
        }
        AstKind::CallExpression(call_expr) => {
            // Count optional chaining on calls.
            call_expr.optional
        }
        _ => false,
    }
}

/// Returns true if the given assignment operator is one of the logical assignment operators.
fn is_logical_assignment_operator(op: AssignmentOperator) -> bool {
    matches!(op, AssignmentOperator::LogicalAnd | AssignmentOperator::LogicalOr | AssignmentOperator::LogicalNullish)
}

/// Attempts to get a name for a function node. If no name is available,
/// returns a default string.
fn get_function_name(node: &AstNode) -> String {
    match node.kind() {
        AstKind::Function(func) => {
            // Assume that function nodes provide an optional name.
            if let Some(name) = func.name() {
                name.to_string()
            } else {
                "anonymous function".to_string()
            }
        }
        _ => "function".to_string(),
    }
}
