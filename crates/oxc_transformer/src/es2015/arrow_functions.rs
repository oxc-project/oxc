use std::rc::Rc;

use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstBuilder, AstKind, VisitMut};
use oxc_span::{Atom, SPAN};
use serde::Deserialize;

use crate::context::TransformerCtx;
use crate::options::TransformOptions;
use crate::TransformTarget;

/// ES2015 Arrow Functions
///
/// References:
/// * <https://babeljs.io/docs/babel-plugin-transform-arrow-functions>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-arrow-functions>
pub struct ArrowFunctions<'a> {
    ast: Rc<AstBuilder<'a>>,
    nodes: Vec<'a, AstKind<'a>>,
    uid: usize,
    has_this: bool,
    /// Insert a variable declaration at the top of the BlockStatement
    insert: bool,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ArrowFunctionsOptions {
    /// This option enables the following:
    /// * Wrap the generated function in .bind(this) and keeps uses of this inside the function as-is, instead of using a renamed this.
    /// * Add a runtime check to ensure the functions are not instantiated.
    /// * Add names to arrow functions.
    pub spec: bool,
}

impl<'a> VisitMut<'a> for ArrowFunctions<'a> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        self.nodes.push(kind);
    }

    fn leave_node(&mut self, _kind: AstKind<'a>) {
        self.nodes.pop();
    }

    fn visit_jsx_identifier(&mut self, ident: &mut JSXIdentifier) {
        let parent_kind = self.nodes.last().unwrap();
        let parent_parent_kind = self.nodes[self.nodes.len() - 2];
        if ident.name == "this"
            && (matches!(parent_kind, AstKind::JSXElementName(_))
                || matches!(parent_parent_kind, AstKind::JSXMemberExpression(_)))
        {
            if !self.has_this {
                self.has_this = true;
                self.uid += 1;
            }
            *ident = self.ast.jsx_identifier(SPAN, self.get_this_name());
        }
    }
}

impl<'a> ArrowFunctions<'a> {
    pub fn new(
        ast: Rc<AstBuilder<'a>>,
        _: TransformerCtx<'a>,
        options: &TransformOptions,
    ) -> Option<Self> {
        (options.target < TransformTarget::ES2015 || options.arrow_functions.is_some()).then(|| {
            let nodes = ast.new_vec();
            Self { ast, uid: 0, nodes, has_this: false, insert: false }
        })
    }

    fn get_this_name(&self) -> Atom {
        let uid = if self.uid == 1 { String::new() } else { self.uid.to_string() };
        format!("_this{uid}",).into()
    }

    pub fn transform_statements(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        if self.insert {
            let binding_pattern = self.ast.binding_pattern(
                self.ast
                    .binding_pattern_identifier(BindingIdentifier::new(SPAN, self.get_this_name())),
                None,
                false,
            );

            let variable_declarator = self.ast.variable_declarator(
                SPAN,
                VariableDeclarationKind::Var,
                binding_pattern,
                Some(self.ast.this_expression(SPAN)),
                false,
            );

            let stmt = self.ast.variable_declaration(
                SPAN,
                VariableDeclarationKind::Var,
                self.ast.new_vec_single(variable_declarator),
                Modifiers::empty(),
            );
            stmts.insert(0, Statement::Declaration(Declaration::VariableDeclaration(stmt)));
            self.insert = false;
        }

        // Insert to parent block
        if self.has_this {
            self.insert = true;
            self.has_this = false;
        }
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        if let Expression::ArrowExpression(arrow_expr) = expr {
            let mut body = self.ast.copy(&arrow_expr.body);

            if arrow_expr.expression {
                let first_stmt = body.statements.remove(0);
                if let Statement::ExpressionStatement(stmt) = first_stmt {
                    let return_statement =
                        self.ast.return_statement(SPAN, Some(self.ast.copy(&stmt.expression)));
                    body.statements.push(return_statement);
                }
            }

            self.visit_function_body(&mut body);

            let new_function = self.ast.function(
                FunctionType::FunctionExpression,
                SPAN,
                None,
                false,
                arrow_expr.r#async,
                None,
                self.ast.copy(&arrow_expr.params),
                Some(body),
                self.ast.copy(&arrow_expr.type_parameters),
                self.ast.copy(&arrow_expr.return_type),
                Modifiers::empty(),
            );

            *expr = Expression::FunctionExpression(new_function);
        }
    }
}
