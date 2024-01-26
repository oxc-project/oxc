use std::{ collections::HashMap };

use oxc_allocator::{Allocator, Vec};

use oxc_ast::AstKind;
#[allow(clippy::wildcard_imports)]
use oxc_ast::{ast::*, AstBuilder, VisitMut};
use oxc_semantic::{ReferenceId, Semantic, SemanticBuilder, SymbolId};
use oxc_span::SourceType;

pub struct Prepass<'a> {
    ast: AstBuilder<'a>,
    pub reference_symbols: HashMap<ReferenceId, SymbolId>,
    pub inlinable_symbols: HashMap<SymbolId, Option<Expression<'a>>>,
    semantic: Semantic<'a>
}

impl<'a> Prepass<'a> {
    pub fn new(allocator: &'a Allocator, source_text: &'a str, source_type: SourceType, program: &mut Program<'a>) -> Self {
        let semantic_builder = SemanticBuilder::new(source_text, source_type);
        let semantic_ret = semantic_builder.build(program);
        //let mut map: HashMap<SymbolId, Option<AstKind>> = HashMap::new();
        let mut pass = Self {
            ast: AstBuilder::new(allocator),
            reference_symbols: HashMap::new(),
            inlinable_symbols: HashMap::new(),
            semantic: semantic_ret.semantic
        };
        pass.build(program);
        pass
    }

    pub fn build(&mut self, program: &mut Program<'a>) {
        self.visit_program(program);
    }

    fn strip_parenthesized_expression(&self, expr: &mut Expression<'a>) {
        if let Expression::ParenthesizedExpression(paren_expr) = expr {
            *expr = self.ast.move_expression(&mut paren_expr.expression);
            self.strip_parenthesized_expression(expr);
        }
    }
}

impl<'a> VisitMut<'a> for Prepass<'a> {
    fn visit_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        stmts.retain(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));
        for stmt in stmts.iter_mut() {
            self.visit_statement(stmt);
        }
    }

    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        self.strip_parenthesized_expression(expr);
        self.visit_expression_match(expr);
    }

    // fn visit_identifier_name(&mut self, ident: &mut IdentifierName) {
    //     println!("{ident:#?}");
    // }

    // fn visit_expression_statement(&mut self, stmt: &mut ExpressionStatement<'a>) {
    //     println!("{stmt:#?}");
    // }

    fn visit_identifier_reference(&mut self, ident: &mut IdentifierReference) {
        if let Some(reference_id) = ident.reference_id.get() {
            let reference = self.semantic.symbols().get_reference(reference_id);
            if let Some(symbol_id) = &reference.symbol_id() {
                self.reference_symbols.insert(reference_id, *symbol_id);
                // already analyzed
                if self.inlinable_symbols.contains_key(symbol_id) {
                    return;
                }
                // println!("{ident:?}");
                for reference in self.semantic.symbol_references(*symbol_id) {
                    if reference.is_write() {
                        self.inlinable_symbols.insert(*symbol_id, None);
                        return;
                    }
                }
                let decl = self.semantic.symbol_declaration(*symbol_id);
                if let AstKind::VariableDeclarator(VariableDeclarator { span: _, kind: _, id: _, init: Some(exp), definite: _ }) = decl.kind() {
                    // TODO: simplify with a derived clone?
                    // TODO: cfg to detect if all the references are after the declaration init?
                    self.semantic.cfg().before_statement(decl.id(), StatementControlFlowType);
                    match exp {
                        Expression::BooleanLiteral(lit) => {
                            self.inlinable_symbols.insert(
                                *symbol_id, Some(
                                    self.ast.literal_boolean_expression(
                                        self.ast.boolean_literal(lit.span, lit.value)
                                    )
                                )
                            );
                            return
                        }
                        Expression::NumberLiteral(lit) => {
                            self.inlinable_symbols.insert(
                                *symbol_id, Some(
                                    self.ast.literal_number_expression(
                                        self.ast.number_literal(lit.span, lit.value, lit.raw, lit.base)
                                    )
                                )
                            );
                            return
                        }
                        _ => {}
                    }
                    return;
                }
                self.inlinable_symbols.insert(*symbol_id, None);
            } else {
                // TODO: arguments is being inferred as global incorrectly and not entangled with other bindings
                // println!("{:?} {:?}", ident.name, self.semantic.is_reference_to_global_variable(ident))
            }
        }
    }
}
