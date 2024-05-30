use std::rc::Rc;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::{Atom, SPAN};
use serde::Deserialize;

use crate::context::Ctx;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ArrowFunctionsOptions {
    /// This option enables the following:
    /// * Wrap the generated function in .bind(this) and keeps uses of this inside the function as-is, instead of using a renamed this.
    /// * Add a runtime check to ensure the functions are not instantiated.
    /// * Add names to arrow functions.
    #[serde(default)]
    pub spec: bool,
}

/// [plugin-transform-arrow-functions](https://babel.dev/docs/babel-plugin-transform-arrow-functions)
///
/// This plugin transforms arrow functions to function expressions.
///
/// This plugin is included in `preset-env`
///
/// References:
///
/// * <https://babeljs.io/docs/babel-plugin-transform-arrow-functions>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-arrow-functions>
pub struct ArrowFunctions<'a> {
    ctx: Ctx<'a>,
    _options: ArrowFunctionsOptions,
    uid: usize,
    has_this: bool,
    /// Stack to keep track of whether we are inside an arrow function or not.
    stacks: std::vec::Vec<bool>,
    // var _this = this;
    this_statements: std::vec::Vec<Option<Statement<'a>>>,
}

impl<'a> ArrowFunctions<'a> {
    pub fn new(options: ArrowFunctionsOptions, ctx: &Ctx<'a>) -> Self {
        Self {
            ctx: Rc::clone(ctx),
            _options: options,
            uid: 0,
            has_this: false,
            stacks: vec![],
            this_statements: vec![],
        }
    }

    fn is_inside_arrow_function(&self) -> bool {
        self.stacks.last().copied().unwrap_or(false)
    }

    fn get_this_name(&self) -> Atom<'a> {
        let uid = if self.uid == 1 { String::new() } else { self.uid.to_string() };
        self.ctx.ast.new_atom(&format!("_this{uid}"))
    }

    fn mark_this_as_found(&mut self) {
        if !self.has_this {
            self.has_this = true;
            self.uid += 1;
        }
    }

    pub fn transform_statements(&mut self, _stmts: &mut Vec<'a, Statement<'a>>) {
        self.this_statements.push(None);
    }

    /// ```ts
    /// function a(){
    ///    () => console.log(this);
    /// }
    /// // to
    /// function a(){
    ///   var _this = this;
    ///  (function() { return console.log(_this); });
    /// }
    /// ```
    /// Insert the var _this = this; statement outside the arrow function
    pub fn transform_statements_on_exit(&mut self, stmts: &mut Vec<'a, Statement<'a>>) {
        // Insert the var _this = this;
        if let Some(Some(stmt)) = self.this_statements.pop() {
            stmts.insert(0, stmt);
        }

        if self.has_this {
            let binding_pattern = self.ctx.ast.binding_pattern(
                self.ctx
                    .ast
                    .binding_pattern_identifier(BindingIdentifier::new(SPAN, self.get_this_name())),
                None,
                false,
            );

            let variable_declarator = self.ctx.ast.variable_declarator(
                SPAN,
                VariableDeclarationKind::Var,
                binding_pattern,
                Some(self.ctx.ast.this_expression(SPAN)),
                false,
            );

            let stmt = self.ctx.ast.variable_declaration(
                SPAN,
                VariableDeclarationKind::Var,
                self.ctx.ast.new_vec_single(variable_declarator),
                Modifiers::empty(),
            );

            let stmt = Statement::VariableDeclaration(stmt);
            // store it, insert it in last statements
            self.this_statements.last_mut().unwrap().replace(stmt);
            self.has_this = false;
        }
    }

    /// Change <this></this> to <_this></_this>, and mark it as found
    pub fn transform_jsx_opening_element(&mut self, elem: &mut JSXOpeningElement<'a>) {
        if !self.is_inside_arrow_function() {
            return;
        }

        let mut change_ident = |ident: &mut JSXIdentifier<'a>| {
            if ident.name == "this" {
                self.mark_this_as_found();
                ident.name = self.get_this_name();
            }
        };
        match &mut elem.name {
            JSXElementName::Identifier(ref mut ident) => change_ident(ident),
            JSXElementName::MemberExpression(ref mut member) => {
                let mut member_expr = member;
                loop {
                    match &mut member_expr.object {
                        JSXMemberExpressionObject::Identifier(ident) => {
                            change_ident(ident);
                            break;
                        }
                        JSXMemberExpressionObject::MemberExpression(expr) => {
                            member_expr = expr;
                        }
                    }
                }
            }
            JSXElementName::NamespacedName(_) => {}
        }
    }

    fn transform_arrow_function_expression(
        &mut self,
        arrow_function_expr: &mut ArrowFunctionExpression<'a>,
    ) -> Expression<'a> {
        let mut body = self.ctx.ast.copy(&arrow_function_expr.body);

        if arrow_function_expr.expression {
            let first_stmt = body.statements.remove(0);
            if let Statement::ExpressionStatement(stmt) = first_stmt {
                let return_statement =
                    self.ctx.ast.return_statement(SPAN, Some(self.ctx.ast.copy(&stmt.expression)));
                body.statements.push(return_statement);
            }
        }

        let new_function = self.ctx.ast.function(
            FunctionType::FunctionExpression,
            SPAN,
            None,
            false,
            arrow_function_expr.r#async,
            None,
            self.ctx.ast.copy(&arrow_function_expr.params),
            Some(body),
            self.ctx.ast.copy(&arrow_function_expr.type_parameters),
            self.ctx.ast.copy(&arrow_function_expr.return_type),
            Modifiers::empty(),
        );

        Expression::FunctionExpression(new_function)
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        match expr {
            Expression::ArrowFunctionExpression(_) => {
                self.stacks.push(true);
            }
            Expression::FunctionExpression(_) => self.stacks.push(false),
            _ => {}
        }
    }

    pub fn transform_expression_on_exit(&mut self, expr: &mut Expression<'a>) {
        match expr {
            Expression::ThisExpression(_) => {
                if !self.is_inside_arrow_function() {
                    return;
                }

                self.mark_this_as_found();
                *expr = self.ctx.ast.identifier_reference_expression(IdentifierReference::new(
                    SPAN,
                    self.get_this_name(),
                ));
            }
            Expression::ArrowFunctionExpression(arrow_function_expr) => {
                *expr = self.transform_arrow_function_expression(arrow_function_expr);
                self.stacks.pop();
            }
            Expression::FunctionExpression(_) => {
                self.stacks.pop();
            }
            _ => {}
        }
    }

    pub fn transform_declaration(&mut self, decl: &mut Declaration<'a>) {
        if let Declaration::FunctionDeclaration(_) = decl {
            self.stacks.push(false);
        }
    }

    pub fn transform_declaration_on_exit(&mut self, decl: &mut Declaration<'a>) {
        if let Declaration::FunctionDeclaration(_) = decl {
            self.stacks.pop();
        }
    }

    pub fn transform_class(&mut self, _class: &mut Class<'a>) {
        self.stacks.push(false);
    }

    pub fn transform_class_on_exit(&mut self, _class: &mut Class<'a>) {
        self.stacks.pop();
    }
}
