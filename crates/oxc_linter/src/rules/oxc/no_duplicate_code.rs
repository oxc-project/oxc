use oxc_ast::ast::{Statement, BlockStatement, FunctionBody};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
};

fn no_duplicate_code_diagnostic(span: Span, original_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Duplicated code block detected")
        .with_help("Consider extracting the duplicated code into a reusable function")
        .with_labels([
            span.label("This code block is duplicated"),
            original_span.label("Original code block is here"),
        ])
}

#[derive(Debug, Clone)]
pub struct NoDuplicateCode {
    /// Minimum number of statements to consider as duplicated code
    min_statements: usize,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects duplicated code blocks that could be refactored into reusable functions.
    ///
    /// ### Why is this bad?
    ///
    /// Duplicated code is a common source of bugs and maintenance issues. When the same
    /// logic is repeated in multiple places, any changes or bug fixes need to be applied
    /// in multiple locations, increasing the risk of inconsistencies.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// function processUserA() {
    ///     if (user.isActive) {
    ///         user.lastSeen = new Date();
    ///         user.status = 'online';
    ///         saveUser(user);
    ///     }
    /// }
    ///
    /// function processUserB() {
    ///     if (user.isActive) {
    ///         user.lastSeen = new Date();
    ///         user.status = 'online';
    ///         saveUser(user);
    ///     }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// function updateActiveUser(user) {
    ///     user.lastSeen = new Date();
    ///     user.status = 'online';
    ///     saveUser(user);
    /// }
    ///
    /// function processUserA() {
    ///     if (user.isActive) {
    ///         updateActiveUser(user);
    ///     }
    /// }
    ///
    /// function processUserB() {
    ///     if (user.isActive) {
    ///         updateActiveUser(user);
    ///     }
    /// }
    /// ```
    ///
    /// ### Options
    ///
    /// This rule supports the following configuration options:
    ///
    /// - `minStatements` (number, default: 3): The minimum number of statements required
    ///   in a function body to be considered for duplication detection.
    ///
    /// Example configuration:
    /// ```json
    /// {
    ///   "oxc/no-duplicate-code": ["warn", { "minStatements": 5 }]
    /// }
    /// ```
    NoDuplicateCode,
    oxc,
    suspicious
);

impl Default for NoDuplicateCode {
    fn default() -> Self {
        Self {
            min_statements: 3, // Minimum 3 statements to consider as duplication
        }
    }
}

impl Rule for NoDuplicateCode {
    fn from_configuration(value: serde_json::Value) -> Self {
        let min_statements = value
            .get(0)
            .and_then(|v| v.get("minStatements"))
            .and_then(serde_json::Value::as_u64)
            .map(|v| v as usize)
            .unwrap_or(3);

        Self { min_statements }
    }

    fn run_once(&self, ctx: &LintContext) {
        let mut function_bodies: Vec<(&FunctionBody, Span)> = Vec::new();
        
        // Collect all function bodies
        for node in ctx.nodes().iter() {
            match node.kind() {
                oxc_ast::AstKind::Function(func) => {
                    if let Some(body) = &func.body {
                        function_bodies.push((body, func.span));
                    }
                }
                oxc_ast::AstKind::ArrowFunctionExpression(arrow) => {
                    function_bodies.push((&arrow.body, arrow.span));
                }
                _ => {}
            }
        }
        
        // Compare function bodies for duplicates
        for i in 0..function_bodies.len() {
            for j in (i + 1)..function_bodies.len() {
                let (body1, span1) = function_bodies[i];
                let (body2, span2) = function_bodies[j];
                
                if self.are_function_bodies_similar(body1, body2) {
                    ctx.diagnostic(no_duplicate_code_diagnostic(span2, span1));
                }
            }
        }
    }
}

impl NoDuplicateCode {
    fn are_function_bodies_similar(&self, body1: &FunctionBody, body2: &FunctionBody) -> bool {
        let statements1 = &body1.statements;
        let statements2 = &body2.statements;
        
        // Both bodies must have at least min_statements
        if statements1.len() < self.min_statements || statements2.len() < self.min_statements {
            return false;
        }
        
        // For now, require exact same number of statements
        if statements1.len() != statements2.len() {
            return false;
        }
        
        // Compare each statement pair
        for (stmt1, stmt2) in statements1.iter().zip(statements2.iter()) {
            if !self.are_statements_similar(stmt1, stmt2) {
                return false;
            }
        }
        
        true
    }
    
    fn are_statements_similar(&self, stmt1: &Statement, stmt2: &Statement) -> bool {
        // Compare statements structurally with stricter requirements
        match (stmt1, stmt2) {
            (Statement::ExpressionStatement(expr1), Statement::ExpressionStatement(expr2)) => {
                self.are_expressions_similar(&expr1.expression, &expr2.expression)
            }
            (Statement::VariableDeclaration(var1), Statement::VariableDeclaration(var2)) => {
                // For variable declarations, require same kind and number of declarations
                // but be lenient on variable names and values to catch structural duplication
                var1.kind == var2.kind && var1.declarations.len() == var2.declarations.len()
            }
            (Statement::ReturnStatement(ret1), Statement::ReturnStatement(ret2)) => {
                match (&ret1.argument, &ret2.argument) {
                    (Some(arg1), Some(arg2)) => self.are_expressions_similar(arg1, arg2),
                    (None, None) => true,
                    _ => false,
                }
            }
            (Statement::IfStatement(if1), Statement::IfStatement(if2)) => {
                // For if statements, we need to compare the test condition more carefully
                self.are_expressions_similar(&if1.test, &if2.test)
            }
            (Statement::BlockStatement(block1), Statement::BlockStatement(block2)) => {
                self.are_block_statements_similar(block1, block2)
            }
            _ => {
                // For other statement types, use the type-based comparison
                self.hash_statement_type(stmt1) == self.hash_statement_type(stmt2)
            }
        }
    }
    
    fn are_block_statements_similar(&self, block1: &BlockStatement, block2: &BlockStatement) -> bool {
        let statements1 = &block1.body;
        let statements2 = &block2.body;
        
        if statements1.len() != statements2.len() {
            return false;
        }
        
        for (stmt1, stmt2) in statements1.iter().zip(statements2.iter()) {
            if !self.are_statements_similar(stmt1, stmt2) {
                return false;
            }
        }
        
        true
    }
    
    fn are_expressions_similar(&self, expr1: &oxc_ast::ast::Expression, expr2: &oxc_ast::ast::Expression) -> bool {
        // Basic structural comparison for expressions - requiring more exact matches
        match (expr1, expr2) {
            (oxc_ast::ast::Expression::Identifier(_id1), oxc_ast::ast::Expression::Identifier(_id2)) => {
                // For better duplicate detection, we'll require structural similarity but be lenient on names
                true
            }
            (oxc_ast::ast::Expression::CallExpression(call1), oxc_ast::ast::Expression::CallExpression(call2)) => {
                // Compare function calls structurally - same number of arguments and similar callee
                call1.arguments.len() == call2.arguments.len() &&
                self.are_expressions_similar(&call1.callee, &call2.callee)
            }
            (oxc_ast::ast::Expression::StaticMemberExpression(mem1), oxc_ast::ast::Expression::StaticMemberExpression(mem2)) => {
                // Require same property name for member expressions
                mem1.property.name == mem2.property.name &&
                self.are_expressions_similar(&mem1.object, &mem2.object)
            }
            (oxc_ast::ast::Expression::AssignmentExpression(assign1), oxc_ast::ast::Expression::AssignmentExpression(assign2)) => {
                assign1.operator == assign2.operator &&
                self.are_expressions_similar(&assign1.right, &assign2.right)
                // Skip left side comparison for now as it's complex with AssignmentTarget
            }
            (oxc_ast::ast::Expression::BinaryExpression(bin1), oxc_ast::ast::Expression::BinaryExpression(bin2)) => {
                // Binary expressions must have same operator to be considered similar
                bin1.operator == bin2.operator &&
                self.are_expressions_similar(&bin1.left, &bin2.left) &&
                self.are_expressions_similar(&bin1.right, &bin2.right)
            }
            (oxc_ast::ast::Expression::NewExpression(new1), oxc_ast::ast::Expression::NewExpression(new2)) => {
                // For 'new Date()' calls, require same constructor and same number of arguments
                new1.arguments.len() == new2.arguments.len() &&
                self.are_expressions_similar(&new1.callee, &new2.callee)
            }
            (oxc_ast::ast::Expression::NumericLiteral(_num1), oxc_ast::ast::Expression::NumericLiteral(_num2)) => {
                // Different numbers should not be considered similar for duplication
                false // Require exact match for numbers
            }
            (oxc_ast::ast::Expression::StringLiteral(str1), oxc_ast::ast::Expression::StringLiteral(str2)) => {
                // Different strings should not be considered similar
                str1.value == str2.value
            }
            _ => {
                // For other expression types, use the type-based comparison
                self.hash_expression_kind(expr1) == self.hash_expression_kind(expr2)
            }
        }
    }

    fn hash_statement_type(&self, statement: &Statement) -> String {
        // Create a simplified representation of the statement type for fallback comparison
        match statement {
            Statement::BlockStatement(_) => "block".to_string(),
            Statement::BreakStatement(_) => "break".to_string(),
            Statement::ContinueStatement(_) => "continue".to_string(),
            Statement::ExpressionStatement(_) => "expr".to_string(),
            Statement::IfStatement(_) => "if".to_string(),
            Statement::ReturnStatement(_) => "return".to_string(),
            Statement::VariableDeclaration(var_decl) => {
                format!("var:{}", var_decl.kind)
            }
            Statement::WhileStatement(_) => "while".to_string(),
            Statement::ForStatement(_) => "for".to_string(),
            Statement::ForInStatement(_) => "for-in".to_string(),
            Statement::ForOfStatement(_) => "for-of".to_string(),
            Statement::DoWhileStatement(_) => "do-while".to_string(),
            Statement::SwitchStatement(_) => "switch".to_string(),
            Statement::TryStatement(_) => "try".to_string(),
            Statement::ThrowStatement(_) => "throw".to_string(),
            _ => "other".to_string(),
        }
    }

    fn hash_expression_kind(&self, expr: &oxc_ast::ast::Expression) -> String {
        // Very basic expression hashing for structural similarity
        match expr {
            oxc_ast::ast::Expression::Identifier(_) => "id".to_string(),
            oxc_ast::ast::Expression::CallExpression(_) => "call".to_string(),
            oxc_ast::ast::Expression::StaticMemberExpression(_) | 
            oxc_ast::ast::Expression::ComputedMemberExpression(_) | 
            oxc_ast::ast::Expression::PrivateFieldExpression(_) => "member".to_string(),
            oxc_ast::ast::Expression::AssignmentExpression(_) => "assign".to_string(),
            oxc_ast::ast::Expression::BinaryExpression(_) => "binary".to_string(),
            oxc_ast::ast::Expression::UnaryExpression(_) => "unary".to_string(),
            oxc_ast::ast::Expression::UpdateExpression(_) => "update".to_string(),
            oxc_ast::ast::Expression::ConditionalExpression(_) => "conditional".to_string(),
            oxc_ast::ast::Expression::ArrayExpression(_) => "array".to_string(),
            oxc_ast::ast::Expression::ObjectExpression(_) => "object".to_string(),
            _ => "expr".to_string(),
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Single statement blocks - not enough to be considered duplication
        r#"
        function a() {
            console.log("hello");
        }
        function b() {
            console.log("hello");
        }
        "#,
        
        // Different code blocks with different operations
        r#"
        function a() {
            let x = 1;
            let y = 2;
            return x + y;
        }
        function b() {
            let a = 3;
            let b = 4;
            return a * b;
        }
        "#,
        
        // Similar structure but different logic - should pass
        r#"
        function processUser(user) {
            if (user.isActive) {
                user.lastSeen = new Date();
                user.status = 'online';
            }
        }
        function processAdmin(admin) {
            if (admin.isActive) {
                admin.lastLogin = new Date();
                admin.role = 'administrator';
            }
        }
        "#,
    ];

    let fail = vec![
        // Exact duplicate code blocks with same expressions
        r#"
        function processUserA() {
            user.lastSeen = new Date();
            user.status = 'online';
            saveUser(user);
        }
        function processUserB() {
            user.lastSeen = new Date();
            user.status = 'online';
            saveUser(user);
        }
        "#,
        
        // Duplicate blocks with same function calls
        r#"
        function handleSuccess() {
            logger.info('Operation successful');
            metrics.increment('success');
            notify.send('completed');
        }
        function handleComplete() {
            logger.info('Operation successful');
            metrics.increment('success');
            notify.send('completed');
        }
        "#,
        
        // Test minStatements configuration - should pass with minStatements=5
        (
            r#"
            function smallA() {
                let x = 1;
                let y = 2;
                return x + y;
            }
            function smallB() {
                let x = 1;
                let y = 2;
                return x + y;
            }
            "#,
            Some(serde_json::json!([{ "minStatements": 2 }])),
        ),
    ];

    Tester::new(NoDuplicateCode::NAME, NoDuplicateCode::PLUGIN, pass, fail).test_and_snapshot();
}
