use std::cell::Cell;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::{Atom, SPAN};
use oxc_syntax::scope::ScopeFlags;
use oxc_traverse::TraverseCtx;
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
//
// TODO: The `spec` option is not currently supported. Add support for it.
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
    pub fn new(options: ArrowFunctionsOptions, ctx: Ctx<'a>) -> Self {
        Self {
            ctx,
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
    pub fn transform_jsx_element_name(&mut self, name: &mut JSXElementName<'a>) {
        if !self.is_inside_arrow_function() {
            return;
        }

        let ident = match name {
            JSXElementName::Identifier(ident) => ident,
            JSXElementName::MemberExpression(member_expr) => {
                member_expr.get_object_identifier_mut()
            }
            JSXElementName::NamespacedName(_) => return,
        };
        if ident.name == "this" {
            self.mark_this_as_found();
            ident.name = self.get_this_name();
        }
    }

    fn transform_arrow_function_expression(
        &mut self,
        arrow_function_expr: &mut ArrowFunctionExpression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let mut body = self.ctx.ast.copy(&arrow_function_expr.body);

        if arrow_function_expr.expression {
            let first_stmt = body.statements.remove(0);
            if let Statement::ExpressionStatement(stmt) = first_stmt {
                let return_statement = self
                    .ctx
                    .ast
                    .return_statement(stmt.span, Some(self.ctx.ast.copy(&stmt.expression)));
                body.statements.push(return_statement);
            }
        }

        // There shouldn't need to be a conditional here. Every arrow function should have a scope ID.
        // But at present TS transforms don't seem to set `scope_id` in some cases, so this test case
        // fails if just unwrap `scope_id`:
        // `typescript/tests/cases/compiler/classFieldSuperAccessible.ts`.
        // ```ts
        // class D {
        //   accessor b = () => {}
        // }
        // ```
        // TODO: Change to `arrow_function_expr.scope_id.get().unwrap()` once scopes are correct
        // in TS transforms.
        let scope_id = arrow_function_expr.scope_id.get();
        if let Some(scope_id) = scope_id {
            let flags = ctx.scopes_mut().get_flags_mut(scope_id);
            *flags &= !ScopeFlags::Arrow;
        }

        let new_function = Function {
            r#type: FunctionType::FunctionExpression,
            span: arrow_function_expr.span,
            id: None,
            generator: false,
            r#async: arrow_function_expr.r#async,
            this_param: None,
            params: self.ctx.ast.copy(&arrow_function_expr.params),
            body: Some(body),
            type_parameters: self.ctx.ast.copy(&arrow_function_expr.type_parameters),
            return_type: self.ctx.ast.copy(&arrow_function_expr.return_type),
            modifiers: Modifiers::empty(),
            scope_id: Cell::new(scope_id),
        };

        Expression::FunctionExpression(self.ctx.ast.alloc(new_function))
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

    pub fn transform_expression_on_exit(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        match expr {
            Expression::ThisExpression(this_expr) => {
                if !self.is_inside_arrow_function() {
                    return;
                }

                self.mark_this_as_found();
                *expr = self.ctx.ast.identifier_reference_expression(IdentifierReference::new(
                    this_expr.span,
                    self.get_this_name(),
                ));
            }
            Expression::ArrowFunctionExpression(arrow_function_expr) => {
                *expr = self.transform_arrow_function_expression(arrow_function_expr, ctx);
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
