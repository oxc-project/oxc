#![no_main]

use libfuzzer_sys::fuzz_target;
use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::{ast::*, AstBuilder};
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::{SourceType, Span};
use oxc_syntax::{number::NumberBase, operator::BinaryOperator};

struct AstShrinker<'a> {
    ast: AstBuilder<'a>,
}

impl<'a> AstShrinker<'a> {
    fn new(ast: AstBuilder<'a>) -> Self {
        Self { ast }
    }

    /// Try to simplify a program by reducing complexity
    fn shrink_program(&mut self, program: &Program<'a>) -> Program<'a> {
        // Start with a minimal program and gradually add complexity
        let mut statements = self.ast.vec();
        
        // Add only the first statement if multiple exist
        if let Some(first_stmt) = program.body.first() {
            statements.push(self.shrink_statement(first_stmt));
        }

        self.ast.program(
            Span::default(),
            program.source_type,
            "",
            self.ast.vec(),
            None,
            self.ast.vec(),
            statements,
        )
    }

    fn shrink_statement(&mut self, stmt: &Statement<'a>) -> Statement<'a> {
        match stmt {
            Statement::ExpressionStatement(expr_stmt) => {
                let simplified_expr = self.shrink_expression(&expr_stmt.expression);
                self.ast.statement_expression(Span::default(), simplified_expr)
            },
            Statement::VariableDeclaration(var_decl) => {
                // Keep only the first declarator
                if let Some(first_decl) = var_decl.declarations.first() {
                    let simplified_id = first_decl.id.clone_in(&self.ast.allocator);
                    let simplified_init = first_decl.init.as_ref().map(|init| self.shrink_expression(init));
                    
                    let declarator = self.ast.variable_declarator(
                        Span::default(),
                        var_decl.kind,
                        simplified_id,
                        simplified_init,
                        false,
                    );
                    let declarations = self.ast.vec_from_iter([declarator]);
                    
                    Statement::VariableDeclaration(
                        self.ast.alloc_variable_declaration(Span::default(), var_decl.kind, declarations, false)
                    )
                } else {
                    // Fallback to empty statement
                    self.ast.statement_empty(Span::default())
                }
            },
            Statement::BlockStatement(block) => {
                // Keep only the first statement in the block
                let mut body = self.ast.vec();
                if let Some(first_stmt) = block.body.first() {
                    body.push(self.shrink_statement(first_stmt));
                }
                self.ast.statement_block(Span::default(), body)
            },
            Statement::IfStatement(if_stmt) => {
                // Simplify the test condition and remove alternate
                let simplified_test = self.shrink_expression(&if_stmt.test);
                let simplified_consequent = self.shrink_statement(&if_stmt.consequent);
                self.ast.statement_if(Span::default(), simplified_test, simplified_consequent, None)
            },
            Statement::ForStatement(for_stmt) => {
                // Convert to simpler expression statement
                if let Some(test) = &for_stmt.test {
                    let simplified_test = self.shrink_expression(test);
                    self.ast.statement_expression(Span::default(), simplified_test)
                } else {
                    self.ast.statement_empty(Span::default())
                }
            },
            _ => {
                // For other statements, convert to empty statement
                self.ast.statement_empty(Span::default())
            }
        }
    }

    fn shrink_expression(&mut self, expr: &Expression<'a>) -> Expression<'a> {
        match expr {
            Expression::BinaryExpression(bin_expr) => {
                // Try to reduce to just the left operand
                self.shrink_expression(&bin_expr.left)
            },
            Expression::UnaryExpression(unary_expr) => {
                // Try to reduce to just the operand
                self.shrink_expression(&unary_expr.argument)
            },
            Expression::ArrayExpression(arr_expr) => {
                // Keep only the first element, but convert to simple literal
                if !arr_expr.elements.is_empty() {
                    // Just return a simple literal instead of trying to preserve array elements
                    self.ast.expression_numeric_literal(Span::default(), 1.0, None, NumberBase::Decimal)
                } else {
                    // Empty array becomes null
                    self.ast.expression_null_literal(Span::default())
                }
            },
            Expression::CallExpression(call_expr) => {
                // Reduce to just the callee
                self.shrink_expression(&call_expr.callee)
            },
            Expression::StaticMemberExpression(member_expr) => {
                // Reduce to just the object
                self.shrink_expression(&member_expr.object)
            },
            Expression::TemplateLiteral(_) => {
                // Convert to simple string
                self.ast.expression_string_literal(Span::default(), self.ast.atom("template"), None)
            },
            // For literals and identifiers, create new copies (they're already minimal)
            _ => {
                // Create a simple identifier as fallback
                self.ast.expression_identifier(Span::default(), self.ast.atom("x"))
            },
        }
    }

    /// Generate a test case to verify the shrinking works
    fn generate_test_case(&mut self) -> Program<'a> {
        // Create a complex program that should be shrinkable
        let mut statements = self.ast.vec();
        
        // Complex binary expression: (a + b) * (c - d)
        let left_add = self.ast.expression_binary(
            Span::default(),
            self.ast.expression_identifier(Span::default(), self.ast.atom("a")),
            BinaryOperator::Addition,
            self.ast.expression_identifier(Span::default(), self.ast.atom("b")),
        );
        
        let right_sub = self.ast.expression_binary(
            Span::default(),
            self.ast.expression_identifier(Span::default(), self.ast.atom("c")),
            BinaryOperator::Subtraction,
            self.ast.expression_identifier(Span::default(), self.ast.atom("d")),
        );
        
        let complex_expr = self.ast.expression_binary(
            Span::default(),
            left_add,
            BinaryOperator::Multiplication,
            right_sub,
        );
        
        statements.push(self.ast.statement_expression(Span::default(), complex_expr));

        self.ast.program(
            Span::default(),
            SourceType::default(),
            "",
            self.ast.vec(),
            None,
            self.ast.vec(),
            statements,
        )
    }
}

fuzz_target!(|data: &[u8]| {
    // Skip empty inputs
    if data.is_empty() {
        return;
    }

    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);
    let mut shrinker = AstShrinker::new(ast);
    
    // Generate a test case to shrink
    let original_program = shrinker.generate_test_case();
    
    // Convert to code to see original
    let codegen_options1 = CodegenOptions::default();
    let original_code = Codegen::new().with_options(codegen_options1).build(&original_program);
    
    // Shrink the program
    let shrunk_program = shrinker.shrink_program(&original_program);
    
    // Convert shrunk program to code
    let codegen_options2 = CodegenOptions::default();
    let shrunk_code = Codegen::new().with_options(codegen_options2).build(&shrunk_program);
    
    // Verify both versions parse correctly
    let parser_allocator1 = Allocator::default();
    let parser_allocator2 = Allocator::default();
    let source_type = SourceType::default();
    let parser_options = ParseOptions::default();
    
    let original_result = Parser::new(&parser_allocator1, &original_code.code, source_type)
        .with_options(parser_options)
        .parse();
        
    let shrunk_result = Parser::new(&parser_allocator2, &shrunk_code.code, source_type)
        .with_options(parser_options)
        .parse();
    
    // Both should parse without errors
    if !original_result.errors.is_empty() {
        eprintln!("Original code parsing failed: {}", original_code.code);
        panic!("Original code generation created unparseable code");
    }
    
    if !shrunk_result.errors.is_empty() {
        eprintln!("Shrunk code parsing failed: {}", shrunk_code.code);
        panic!("Shrunk code generation created unparseable code");
    }
    
    // The shrunk version should be simpler (fewer characters is a simple heuristic)
    assert!(shrunk_code.code.len() <= original_code.code.len(), 
            "Shrunk code should not be longer than original.\nOriginal: {}\nShrunk: {}", 
            original_code.code, shrunk_code.code);
});