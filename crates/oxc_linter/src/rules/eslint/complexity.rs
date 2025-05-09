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

/// The perceived complexity rule. It reports a warning if a function’s
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
    /// Enforces a maximum perceived complexity allowed in a program.
    ///
    /// ### Why is this bad?
    ///
    /// High perceived complexity makes code harder to understand, maintain, and test.
    ///
    /// ### Examples
    ///
    /// For example, with the default `{ "max": 0 }` option:
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
    /// // Base complexity is 0; this function has complexity 3.
    /// ```
    ///
    /// ### Options
    ///
    /// #### max
    ///
    /// `{ type: number, default: 0 }`
    ///
    /// Maximum allowed perceived complexity.
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

/// Computes the perceived complexity for a given function node.
fn compute_complexity<'a>(
    function_node: &AstNode<'a>,
    nodes: &'a AstNodes<'a>,
    variant: ComplexityVariant,
) -> usize {
    let mut complexity = 0; // initial path
    for node in nodes.iter() {
        // Skip the function node itself.
        if node.id() == function_node.id() {
            continue;
        }
        // Count the node only if the first encountered function ancestor is `function_node`.
        if is_in_function(function_node, node, nodes) {
            // Add complexity for decision points
            if is_decision_point(node, variant) {
                complexity += 1;
            }
            
            // Special handling for if statements with else branches
            if let AstKind::IfStatement(if_stmt) = node.kind() {
                if let Some(alternate) = &if_stmt.alternate {
                    // Check if this is an "else if" construction
                    let is_else_if = matches!(alternate, oxc_ast::ast::Statement::IfStatement(_));
                    
                    if !is_else_if {
                        complexity += 1;
                    }
                }
            }

            // Special handling for switch statements with default cases
            if let AstKind::SwitchStatement(switch_stmt) = node.kind() {
                if switch_stmt.cases.iter().any(|case| case.test.is_none()) {
                    complexity += 1;
                }
            }

            // Special handling for logical expressions
            if let AstKind::LogicalExpression(_) = node.kind() {
                if !has_logical_expression_parent(node, nodes) {
                    let operator_sequences = count_logical_operator_sequences(node);
                    complexity += operator_sequences;
                }
            }
        }
    }
    complexity
}

/// Checks if the node has a logical expression as its parent
fn has_logical_expression_parent<'a>(
    node: &AstNode<'a>,
    nodes: &'a AstNodes<'a>,
) -> bool {
    if let Some(parent_id) = nodes.parent_id(node.id()) {
        let parent = nodes.get_node(parent_id);
        matches!(parent.kind(), AstKind::LogicalExpression(_))
    } else {
        false
    }
}

fn count_logical_operator_sequences<'a>(
    node: &AstNode<'a>,
) -> usize {
    use oxc_syntax::operator::LogicalOperator;
    
    fn count_sequences(
        expr: &oxc_ast::ast::LogicalExpression,
        current_op: Option<LogicalOperator>,
        count: &mut usize,
    ) {
        if current_op.is_none() || current_op != Some(expr.operator) {
            *count += 1;
        }
        
        if let oxc_ast::ast::Expression::LogicalExpression(left_expr) = &expr.left {
            count_sequences(left_expr, Some(expr.operator), count);
        }
        
        if let oxc_ast::ast::Expression::LogicalExpression(right_expr) = &expr.right {
            count_sequences(right_expr, Some(expr.operator), count);
        }
    }
    
    let mut sequence_count = 0;

    if let AstKind::LogicalExpression(expr) = node.kind() {
        count_sequences(expr, None, &mut sequence_count);
    }
    
    sequence_count
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

fn is_decision_point(node: &AstNode, _variant: ComplexityVariant) -> bool {
    match node.kind() {
        AstKind::IfStatement(_) => true,
        AstKind::ForStatement(_) => true,
        AstKind::ForInStatement(_) => true,
        AstKind::ForOfStatement(_) => true,
        AstKind::WhileStatement(_) => true,
        AstKind::DoWhileStatement(_) => true,
        AstKind::ConditionalExpression(_) => true,
        AstKind::CatchClause(_) => true,
        AstKind::AssignmentPattern(_) => true,
        AstKind::SwitchStatement(_) => true,
        AstKind::AssignmentExpression(assign_expr) => {
            // If the assignment operator has short-circuiting behavior,
            // count it as a decision point.
            is_logical_assignment_operator(assign_expr.operator)
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

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::SourceType;

    #[test]
    fn test_compute_complexity_directly() {
        // Test cases with expected complexity values
        let test_cases = vec![
            // Basic function
            ("function simple() { return true; }", 0), 
            
            // optional chaining on member expressions (obj?.prop)
            ("function optChain(obj) { return obj?.prop; }", 0),

            // optional chaining on call expressions (func?.())
            ("function optionalCall(func) { return func?.(); }", 0),
            
            // Function with one if
            ("function oneIf(a) { if (a) { return true; } return false; }", 1),

            // Function with one if-else
            ("function oneIf(a) { if (a) { return true; } else { return false; } }", 2), 
            
            // Function with if-else if
            ("function ifElseIf(a, b) { if (a) { return 1; } else if (b) { return 2; } return 0; }", 2),
        
            // Function with ternary
            ("function ternary(val) { return val ? 'yes' : 'no'; }", 1),

            // Function with nullish coalescing
            ("function nullishCoalescing(val) { return val ?? 'default'; }", 1),

            // Function with nullish coalescing assignment
            ("function nullishAssign(obj) { obj.val ??= 'default'; return obj; }", 1),
            
            // Function with try-catch
            ("function tryCatch() { try { return true; } catch(e) { return false; } }", 1),

            // switch statement
            ("function switchTest(val) { switch(val) { case 1: return 'one'; case 2: return 'two'; } }", 1),

            // switch statement with default vase
            ("function switchTest(val) { switch(val) { case 1: return 'one'; case 2: return 'two'; default: return 'other'; } }", 2),                
            
            // Function with logical assignment
            ("function logAssign(obj) { obj.val ||= 'default'; return obj; }", 1),

            // Function with same logical && expression sequence: +1 for if; +1 for all &&
            ("function logicalTest(a, b, c){ if(a && b && c) {}}", 2),

            // Function with same logical || expression sequence: +1 for if; +1 for all ||
            ("function logicalTest(a, b, c){ if(a || b || c) {}}", 2),

            // Function with two continuous logical sequences: +1 for if; +1 for all &&; +1 for all ||
            ("function logicalTest(a, b, c, d, e){ if(a && b && c || d || e) {}}", 3),

            // Function with two mixed logical sequences: +1 for if; +1 for &&; +1 for ||; +1 for && (new)
            ("function logicalTest(a, b, c, d, e){ if(a && b || c && d) {}}", 4),            
        ];
        
        for (js, expected) in &test_cases {
            let complexity = get_test_complexity(js, ComplexityVariant::Classic);
            assert_eq!(complexity, *expected, "Classic variant failed for: {}", js);
        }
        
    }
    
    // Helper function to prepare AST and call compute_complexity
    fn get_test_complexity(js: &str, variant: ComplexityVariant) -> usize {
        let allocator = Allocator::default();
        let source_type = SourceType::default();
        
        // Parse the source code
        let ret = Parser::new(&allocator, js, source_type).parse();
        let program = ret.program;
        
        // Build semantic model
        let semantic_builder = SemanticBuilder::new();
        let semantic = semantic_builder.build(&program);
        let nodes = semantic.semantic.nodes();
        
        // Find the first function node in the program
        let function_node = nodes.iter()
            .find(|node| is_function_node(node))
            .expect("No function node found in test code");
            
        // Call the function we're testing
        compute_complexity(function_node, nodes, variant)
    }
}