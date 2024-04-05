use std::collections::HashSet;

use oxc_allocator::Box;
use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_ast::walk_mut::walk_ts_module_declaration_mut;
use oxc_ast::Visit;
use oxc_ast::VisitMut;
use oxc_ast::VisitResult;
use oxc_span::Span;

use crate::context::TransformerContext;
use crate::visit_utils::TransformResult;

pub struct Namespace<'a> {
    ctx: TransformerContext<'a>,
}

impl<'a> VisitMut<'a> for Namespace<'a> {
    type Result = TransformResult<'a>;

    fn visit_ts_module_declaration(&mut self, decl: &mut TSModuleDeclaration<'a>) -> Self::Result {
        walk_ts_module_declaration_mut(self, decl);

        match handle_nested(decl) {
            Some(value) => {
                // TODO: if scope has own binding
                if false {
                    TransformResult::replace().with_statement(value)
                }
                // register declaration
                else {
                    TransformResult::replace().with_many_statements(vec![
                        Statement::Declaration(Declaration::VariableDeclaration(
                            self.ctx.ast.variable_declaration(
                                Span::new(0, 0),
                                VariableDeclarationKind::Let,
                                {
                                    let mut list = self.ctx.ast.new_vec();
                                    list.push(self.ctx.ast.variable_declarator(
                                        Span::new(0, 0),
                                        VariableDeclarationKind::Let,
                                        BindingPattern::new_with_kind(
                                            BindingPatternKind::BindingIdentifier(
                                                self.ctx.ast.alloc(BindingIdentifier::new(
                                                    Span::new(0, 0),
                                                    decl.id.name().to_owned(),
                                                )),
                                            ),
                                        ),
                                        None,
                                        false,
                                    ));
                                    list
                                },
                                Modifiers::empty(),
                            ),
                        )),
                        value,
                    ])
                }
            }
            None => {
                // Type only, so remove entirely
                TransformResult::remove()
            }
        }
    }
}

fn handle_nested<'a>(decl: &TSModuleDeclaration<'a>) -> Option<Statement<'a>> {
    let mut names = HashSet::new();
    let mut is_empty = true;

    match decl.body.as_ref() {
        Some(TSModuleDeclarationBody::TSModuleBlock(b)) => {
            for child in &b.body {
                // First pass
                match child {
                    Statement::Declaration(d) => {
                        match d {
                            Declaration::TSModuleDeclaration(i) => {}
                            Declaration::TSEnumDeclaration(i) => {
                                is_empty = false;
                                names.insert(i.id.name.clone());
                                continue;
                            }
                            Declaration::FunctionDeclaration(i) => {
                                is_empty = false;
                                names.insert(i.id.as_ref().unwrap().name.clone());
                                continue;
                            }
                            Declaration::ClassDeclaration(i) => {
                                is_empty = false;
                                names.insert(i.id.as_ref().unwrap().name.clone());
                                continue;
                            }
                            Declaration::VariableDeclaration(i) => {
                                is_empty = false;

                                for var in &i.declarations {
                                    // names.insert();
                                }
                                continue;
                            }
                            _ => {}
                        };
                    }
                    _ => {}
                };
            }
        }
        Some(TSModuleDeclarationBody::TSModuleDeclaration(d)) => {}
        _ => {}
    };

    None
}
