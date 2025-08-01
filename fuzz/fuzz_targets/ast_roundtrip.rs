#![no_main]

use libfuzzer_sys::fuzz_target;
use oxc_allocator::Allocator;
use oxc_ast::{ast::*, AstBuilder};
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::{SourceType, Span};
use oxc_syntax::{number::NumberBase, operator::{BinaryOperator, UnaryOperator}};

struct AstFuzzer<'a> {
    ast: AstBuilder<'a>,
    data: &'a [u8],
    position: usize,
    depth: u8,
    max_depth: u8,
}

impl<'a> AstFuzzer<'a> {
    fn new(ast: AstBuilder<'a>, data: &'a [u8]) -> Self {
        Self {
            ast,
            data,
            position: 0,
            depth: 0,
            max_depth: 8,
        }
    }

    fn next_byte(&mut self) -> u8 {
        if self.position >= self.data.len() {
            0
        } else {
            let byte = self.data[self.position];
            self.position += 1;
            byte
        }
    }

    fn next_u32(&mut self) -> u32 {
        let mut result = 0u32;
        for _ in 0..4 {
            result = (result << 8) | u32::from(self.next_byte());
        }
        result
    }

    fn should_recurse(&self) -> bool {
        self.depth < self.max_depth
    }

    fn with_depth<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        self.depth += 1;
        let result = f(self);
        self.depth -= 1;
        result
    }

    fn generate_identifier(&mut self, prefix: &str) -> Expression<'a> {
        let id = format!("{}_{}", prefix, self.next_byte());
        self.ast.expression_identifier(Span::default(), self.ast.atom(&id))
    }

    fn generate_literal(&mut self) -> Expression<'a> {
        match self.next_byte() % 5 {
            0 => self.ast.expression_boolean_literal(Span::default(), self.next_byte() % 2 == 0),
            1 => self.ast.expression_null_literal(Span::default()),
            2 => {
                let value = (self.next_u32() % 1000) as f64;
                self.ast.expression_numeric_literal(Span::default(), value, None, NumberBase::Decimal)
            }
            3 => {
                let text = format!("str{}", self.next_byte());
                self.ast.expression_string_literal(Span::default(), self.ast.atom(&text), None)
            }
            _ => {
                // Template literal with simple text
                let text = format!("template{}", self.next_byte());
                let value = TemplateElementValue {
                    raw: self.ast.atom(&text),
                    cooked: Some(self.ast.atom(&text)),
                };
                let quasi = self.ast.template_element(Span::default(), value, true);
                let quasis = self.ast.vec_from_iter([quasi]);
                let expressions = self.ast.vec();
                self.ast.expression_template_literal(Span::default(), quasis, expressions)
            }
        }
    }

    fn generate_binary_expression(&mut self) -> Expression<'a> {
        if !self.should_recurse() {
            return self.generate_literal();
        }

        let operators = [
            BinaryOperator::Addition,
            BinaryOperator::Subtraction,
            BinaryOperator::Multiplication,
            BinaryOperator::Division,
            BinaryOperator::Equality,
            BinaryOperator::Inequality,
            BinaryOperator::LessThan,
            BinaryOperator::GreaterThan,
        ];
        
        let op = operators[self.next_byte() as usize % operators.len()];
        let left = self.with_depth(|g| g.generate_expression());
        let right = self.with_depth(|g| g.generate_expression());
        
        self.ast.expression_binary(Span::default(), left, op, right)
    }

    fn generate_unary_expression(&mut self) -> Expression<'a> {
        if !self.should_recurse() {
            return self.generate_literal();
        }

        let operators = [
            UnaryOperator::UnaryNegation,
            UnaryOperator::UnaryPlus,
            UnaryOperator::LogicalNot,
            UnaryOperator::BitwiseNot,
            UnaryOperator::Typeof,
        ];
        
        let op = operators[self.next_byte() as usize % operators.len()];
        let argument = self.with_depth(|g| g.generate_expression());
        
        self.ast.expression_unary(Span::default(), op, argument)
    }

    fn generate_array_expression(&mut self) -> Expression<'a> {
        if !self.should_recurse() {
            return self.generate_literal();
        }

        let count = (self.next_byte() % 5) as usize;
        let mut elements = self.ast.vec();
        
        for _ in 0..count {
            if self.next_byte() % 10 == 0 {
                // Occasional hole in array - use spread
                let spread_expr = self.with_depth(|g| g.generate_expression());
                elements.push(ArrayExpressionElement::SpreadElement(
                    self.ast.alloc_spread_element(Span::default(), spread_expr)
                ));
            } else {
                let expr = self.with_depth(|g| g.generate_expression());
                elements.push(ArrayExpressionElement::from(expr));
            }
        }
        
        self.ast.expression_array(Span::default(), elements)
    }

    fn generate_call_expression(&mut self) -> Expression<'a> {
        if !self.should_recurse() {
            return self.generate_literal();
        }

        let callee = self.with_depth(|g| g.generate_expression());
        let arg_count = (self.next_byte() % 4) as usize;
        let mut arguments = self.ast.vec();
        
        for _ in 0..arg_count {
            let arg = self.with_depth(|g| g.generate_expression());
            arguments.push(Argument::from(arg));
        }
        
        self.ast.expression_call(
            Span::default(), 
            callee, 
            Option::<oxc_ast::ast::TSTypeParameterInstantiation>::None, 
            arguments, 
            false
        )
    }

    fn generate_expression(&mut self) -> Expression<'a> {
        if !self.should_recurse() {
            return self.generate_literal();
        }

        match self.next_byte() % 10 {
            0..=2 => self.generate_literal(),
            3 => self.generate_identifier("var"),
            4..=5 => self.generate_binary_expression(),
            6 => self.generate_unary_expression(),
            7 => self.generate_array_expression(),
            8 => self.generate_call_expression(),
            _ => {
                // Member expression - use StaticMemberExpression directly
                let object = self.with_depth(|g| g.generate_expression());
                let property = self.ast.identifier_name(Span::default(), "prop");
                Expression::StaticMemberExpression(
                    self.ast.alloc_static_member_expression(Span::default(), object, property, false)
                )
            }
        }
    }

    fn generate_variable_declaration(&mut self) -> Statement<'a> {
        let kind = match self.next_byte() % 3 {
            0 => VariableDeclarationKind::Var,
            1 => VariableDeclarationKind::Let,
            _ => VariableDeclarationKind::Const,
        };

        let var_name = self.ast.atom(&format!("var{}", self.next_byte()));
        let binding_id = self.ast.binding_identifier(Span::default(), var_name);
        let id = BindingPattern {
            kind: BindingPatternKind::BindingIdentifier(self.ast.alloc(binding_id)),
            type_annotation: None,
            optional: false,
        };

        let init = if self.next_byte() % 2 == 0 {
            Some(self.with_depth(|g| g.generate_expression()))
        } else {
            None
        };

        let declarator = self.ast.variable_declarator(Span::default(), kind, id, init, false);
        let declarations = self.ast.vec_from_iter([declarator]);
        
        Statement::VariableDeclaration(
            self.ast.alloc_variable_declaration(Span::default(), kind, declarations, false)
        )
    }

    fn generate_if_statement(&mut self) -> Statement<'a> {
        if !self.should_recurse() {
            return self.generate_expression_statement();
        }

        let test = self.with_depth(|g| g.generate_expression());
        let consequent = self.with_depth(|g| g.generate_statement());
        let alternate = if self.next_byte() % 3 == 0 {
            Some(self.with_depth(|g| g.generate_statement()))
        } else {
            None
        };

        self.ast.statement_if(Span::default(), test, consequent, alternate)
    }

    fn generate_for_statement(&mut self) -> Statement<'a> {
        if !self.should_recurse() {
            return self.generate_expression_statement();
        }

        let init = if self.next_byte() % 2 == 0 {
            let binding_id = self.ast.binding_identifier(Span::default(), "i");
            let binding_pattern = BindingPattern {
                kind: BindingPatternKind::BindingIdentifier(self.ast.alloc(binding_id)),
                type_annotation: None,
                optional: false,
            };
            Some(ForStatementInit::VariableDeclaration(
                self.ast.alloc_variable_declaration(
                    Span::default(),
                    VariableDeclarationKind::Let,
                    self.ast.vec_from_iter([self.ast.variable_declarator(
                        Span::default(),
                        VariableDeclarationKind::Let,
                        binding_pattern,
                        Some(self.ast.expression_numeric_literal(Span::default(), 0.0, None, NumberBase::Decimal)),
                        false,
                    )]),
                    false,
                )
            ))
        } else {
            None
        };

        let test = if self.next_byte() % 2 == 0 {
            Some(self.with_depth(|g| g.generate_expression()))
        } else {
            None
        };

        let update = if self.next_byte() % 2 == 0 {
            Some(self.with_depth(|g| g.generate_expression()))
        } else {
            None
        };

        let body = self.with_depth(|g| g.generate_statement());

        self.ast.statement_for(Span::default(), init, test, update, body)
    }

    fn generate_expression_statement(&mut self) -> Statement<'a> {
        let expr = self.generate_expression();
        self.ast.statement_expression(Span::default(), expr)
    }

    fn generate_statement(&mut self) -> Statement<'a> {
        if !self.should_recurse() {
            return self.generate_expression_statement();
        }

        match self.next_byte() % 8 {
            0..=2 => self.generate_expression_statement(),
            3 => self.generate_variable_declaration(),
            4 => self.generate_if_statement(),
            5 => self.generate_for_statement(),
            6 => {
                // Block statement
                let count = (self.next_byte() % 4) as usize;
                let mut body = self.ast.vec();
                for _ in 0..count {
                    body.push(self.with_depth(|g| g.generate_statement()));
                }
                self.ast.statement_block(Span::default(), body)
            }
            _ => {
                // Return statement
                let argument = if self.next_byte() % 2 == 0 {
                    Some(self.with_depth(|g| g.generate_expression()))
                } else {
                    None
                };
                self.ast.statement_return(Span::default(), argument)
            }
        }
    }

    fn generate_program(&mut self) -> Program<'a> {
        let statement_count = std::cmp::max(1, (self.next_byte() % 8) as usize);
        let mut statements = self.ast.vec();
        
        for _ in 0..statement_count {
            statements.push(self.generate_statement());
        }

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
    // Skip empty or very large inputs
    if data.is_empty() || data.len() > 1024 {
        return;
    }

    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);
    
    // Generate sophisticated AST from fuzz input
    let mut fuzzer = AstFuzzer::new(ast, data);
    let original_ast = fuzzer.generate_program();
    
    // Convert AST to code
    let codegen_options = CodegenOptions::default();
    let generated_code = Codegen::new().with_options(codegen_options).build(&original_ast);
    let code = generated_code.code;
    
    // Skip if generated code is too long (likely indicates infinite recursion or similar issue)
    if code.len() > 50_000 {
        return;
    }
    
    // Parse the generated code back to AST
    let parser_allocator = Allocator::default();
    let source_type = SourceType::default();
    let parser_options = ParseOptions::default();
    
    let ret = Parser::new(&parser_allocator, &code, source_type)
        .with_options(parser_options)
        .parse();
    
    // Basic checks to ensure the round-trip is valid
    if !ret.errors.is_empty() {
        // If there are parse errors, this indicates a bug in codegen
        // or our AST generation, which is valuable to find
        eprintln!("Generated code: {}", code);
        for error in &ret.errors {
            eprintln!("Parse error: {:?}", error);
        }
        panic!("Code generation created unparseable code");
    }
    
    // If we get here, the round-trip succeeded
});
