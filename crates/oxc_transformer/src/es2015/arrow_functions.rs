use std::cell::Cell;

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_syntax::{scope::ScopeFlags, symbol::SymbolFlags};
use oxc_traverse::TraverseCtx;
use serde::Deserialize;

use crate::{context::Ctx, helpers::bindings::BoundIdentifier};

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
//
// TODO: We create `var _this = this;` in parent block, whereas we should create it in
// parent vars block like Babel:
// ```js
// // Input
// function foo() {
//   { let f = () => this; }
//   { let f2 = () => this; }
// }
//
// // Babel output
// function foo() {
//   var _this = this;
//   { let f = function () { return _this; } }
//   { let f2 = function () { return _this; } }
// }
//
// // Our output
// function foo() {
//   {
//     var _this = this;
//     let f = function () { return _this; }
//   }
//   {
//     var _this2 = this;
//     let f2 = function () { return _this2; }
//   }
// }
// ```
pub struct ArrowFunctions<'a> {
    ctx: Ctx<'a>,
    _options: ArrowFunctionsOptions,
    this_var: Option<BoundIdentifier<'a>>,
    /// Stack to keep track of whether we are inside an arrow function or not.
    stacks: std::vec::Vec<bool>,
    // var _this = this;
    this_statements: std::vec::Vec<Option<Statement<'a>>>,
}

impl<'a> ArrowFunctions<'a> {
    pub fn new(options: ArrowFunctionsOptions, ctx: Ctx<'a>) -> Self {
        Self { ctx, _options: options, this_var: None, stacks: vec![], this_statements: vec![] }
    }

    fn is_inside_arrow_function(&self) -> bool {
        self.stacks.last().copied().unwrap_or(false)
    }

    fn get_this_name(&mut self, ctx: &mut TraverseCtx<'a>) -> BoundIdentifier<'a> {
        if self.this_var.is_none() {
            self.this_var = Some(BoundIdentifier::new_uid(
                "this",
                ctx.current_scope_id(),
                SymbolFlags::FunctionScopedVariable,
                ctx,
            ));
        }
        self.this_var.as_ref().unwrap().clone()
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

        if let Some(id) = &self.this_var {
            let binding_pattern = self.ctx.ast.binding_pattern(
                self.ctx
                    .ast
                    .binding_pattern_kind_from_binding_identifier(id.create_binding_identifier()),
                Option::<TSTypeAnnotation>::None,
                false,
            );

            let variable_declarator = self.ctx.ast.variable_declarator(
                SPAN,
                VariableDeclarationKind::Var,
                binding_pattern,
                Some(self.ctx.ast.expression_this(SPAN)),
                false,
            );

            let stmt = self.ctx.ast.alloc_variable_declaration(
                SPAN,
                VariableDeclarationKind::Var,
                self.ctx.ast.new_vec_single(variable_declarator),
                false,
            );

            let stmt = Statement::VariableDeclaration(stmt);
            // store it, insert it in last statements
            self.this_statements.last_mut().unwrap().replace(stmt);

            // TODO: This isn't quite right. In this case, output is invalid:
            // ```js
            // function foo() {
            //   let f = () => this;
            //   let f2 = () => this;
            // }
            // ```
            self.this_var = None;
        }
    }

    /// Change <this></this> to <_this></_this>, and mark it as found
    pub fn transform_jsx_element_name(
        &mut self,
        name: &mut JSXElementName<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
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
            // We can't produce a proper identifier with a `ReferenceId` because `JSXIdentifier`
            // lacks that field. https://github.com/oxc-project/oxc/issues/3528
            // So generate a reference and just use its name.
            // If JSX transform is enabled, that transform runs before this and will have converted
            // this to a proper `ThisExpression`, and this visitor won't run.
            // So only a problem if JSX transform is disabled.
            let new_ident = self.get_this_name(ctx).create_read_reference(ctx);
            ident.name = new_ident.name;
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
                    .statement_return(stmt.span, Some(self.ctx.ast.copy(&stmt.expression)));
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
            declare: false,
            this_param: None,
            params: self.ctx.ast.copy(&arrow_function_expr.params),
            body: Some(body),
            type_parameters: self.ctx.ast.copy(&arrow_function_expr.type_parameters),
            return_type: self.ctx.ast.copy(&arrow_function_expr.return_type),
            scope_id: Cell::new(scope_id),
        };

        let expr = Expression::FunctionExpression(self.ctx.ast.alloc(new_function));
        // Avoid creating a function declaration.
        // `() => {};` => `(function () {});`
        self.ctx.ast.expression_parenthesized(SPAN, expr)
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

                let ident =
                    self.get_this_name(ctx).create_spanned_read_reference(this_expr.span, ctx);
                *expr = self.ctx.ast.expression_from_identifier_reference(ident);
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
