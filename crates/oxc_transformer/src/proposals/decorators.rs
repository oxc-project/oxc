use std::rc::Rc;

use oxc_allocator::Box;
use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{Atom, SPAN};
use oxc_syntax::operator::{AssignmentOperator, LogicalOperator};
use serde::Deserialize;

use crate::{context::TransformerCtx, options::TransformOptions};

/// Proposal: Decorators
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-proposal-decorators>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-proposal-decorators>
/// * <https://github.com/tc39/proposal-decorators>
pub struct Decorators<'a> {
    ast: Rc<AstBuilder<'a>>,
    _ctx: TransformerCtx<'a>,
    options: DecoratorsOptions,
    // Insert to the top of the program
    top_statements: Vec<Statement<'a>>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecoratorsOptions {
    version: Version,
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Version {
    Legacy,
    #[serde(rename = "2023-05")]
    #[default]
    Year2023May,
}
impl Version {
    fn is_legacy(self) -> bool {
        matches!(self, Self::Legacy)
    }
}

impl<'a> Decorators<'a> {
    pub fn new(
        ast: Rc<AstBuilder<'a>>,
        ctx: TransformerCtx<'a>,
        options: &TransformOptions,
    ) -> Option<Self> {
        options.decorators.map(|options| Self {
            ast,
            _ctx: ctx,
            options,
            top_statements: Vec::new(),
        })
    }

    pub fn transform_program(&mut self, program: &mut Program<'a>) {
        program.body.splice(0..0, self.top_statements.drain(..));
    }

    pub fn transform_declaration(&mut self, decl: &mut Declaration<'a>) {
        let new_decl = match decl {
            Declaration::ClassDeclaration(class) => {
                if self.options.version.is_legacy() {
                    Some(self.transform_class_legacy(class))
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(new_decl) = new_decl {
            *decl = new_decl;
        }
    }

    pub fn transform_class_legacy(
        &mut self,
        class: &mut Box<'a, oxc_ast::ast::Class<'a>>,
    ) -> Declaration<'a> {
        let class_identifier_name: Atom = "_class".into();
        let class_identifier = IdentifierReference::new(SPAN, class_identifier_name.clone());

        let decl = self.ast.variable_declaration(
            SPAN,
            VariableDeclarationKind::Var,
            self.ast.new_vec_single(self.ast.variable_declarator(
                SPAN,
                VariableDeclarationKind::Var,
                BindingPattern::new_with_kind(self.ast.binding_pattern_identifier(
                    BindingIdentifier::new(SPAN, class_identifier_name),
                )),
                None,
                false,
            )),
            Modifiers::empty(),
        );
        self.top_statements.push(Statement::Declaration(Declaration::VariableDeclaration(decl)));

        let left = AssignmentTarget::SimpleAssignmentTarget(
            self.ast.simple_assignment_target_identifier(class_identifier.clone()),
        );
        let right = self.ast.class_expression(self.ast.copy(class));
        let new_expr =
            self.ast.assignment_expression(SPAN, AssignmentOperator::Assign, left, right);

        let new_expr = class.decorators.drain(..).fold(new_expr, |new_expr, decorator| {
            match &decorator.expression {
                Expression::Identifier(identifier) => self.ast.call_expression(
                    SPAN,
                    self.ast.identifier_reference_expression(IdentifierReference::new(
                        SPAN,
                        identifier.name.clone(),
                    )),
                    self.ast.new_vec_single(Argument::Expression(self.ast.copy(&new_expr))),
                    false,
                    None,
                ),
                _ => new_expr,
            }
        });

        let init = self.ast.logical_expression(
            SPAN,
            new_expr,
            LogicalOperator::Or,
            self.ast.identifier_reference_expression(class_identifier),
        );

        let declarator = self.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Let,
            BindingPattern::new_with_kind(
                self.ast.binding_pattern_identifier(self.ast.copy(&class.id.clone().unwrap())),
            ),
            Some(init),
            false,
        );

        Declaration::VariableDeclaration(self.ast.variable_declaration(
            SPAN,
            VariableDeclarationKind::Let,
            self.ast.new_vec_single(declarator),
            Modifiers::empty(),
        ))
    }
}
