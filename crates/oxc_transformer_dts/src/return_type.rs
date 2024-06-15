use std::rc::Rc;

use oxc_ast::{
    ast::{
        BindingIdentifier, Expression, Function, FunctionBody, ReturnStatement, TSType,
        TSTypeAliasDeclaration, TSTypeName, TSTypeQueryExprName,
    },
    Visit,
};
use oxc_span::{Atom, GetSpan};
use oxc_syntax::scope::ScopeFlags;

use crate::{context::Ctx, diagnostics::type_containing_private_name, TransformerDts};

/// Infer return type from return statement. Does not support multiple return statements.
pub struct FunctionReturnType<'a> {
    ctx: Ctx<'a>,
    return_expression: Option<Expression<'a>>,
    value_bindings: Vec<Atom<'a>>,
    type_bindings: Vec<Atom<'a>>,
    return_statement_count: u8,
    scope_depth: u32,
}

impl<'a> FunctionReturnType<'a> {
    pub fn infer(transformer: &TransformerDts<'a>, body: &FunctionBody<'a>) -> Option<TSType<'a>> {
        let mut visitor = FunctionReturnType {
            ctx: Rc::clone(&transformer.ctx),
            return_expression: None,
            return_statement_count: 0,
            scope_depth: 0,
            value_bindings: Vec::default(),
            type_bindings: Vec::default(),
        };

        visitor.visit_function_body(body);

        if visitor.return_statement_count > 1 {
            return None;
        }

        visitor.return_expression.and_then(|expr| {
            let expr_type = transformer.infer_type_from_expression(&expr)?;

            if let Some((reference_name, is_value)) = match &expr_type {
                TSType::TSTypeReference(type_reference) => {
                    if let TSTypeName::IdentifierReference(ident) = &type_reference.type_name {
                        Some((ident.name.clone(), false))
                    } else {
                        None
                    }
                }
                TSType::TSTypeQuery(query) => {
                    if let TSTypeQueryExprName::IdentifierReference(ident) = &query.expr_name {
                        Some((ident.name.clone(), true))
                    } else {
                        None
                    }
                }
                _ => None,
            } {
                let is_defined_in_current_scope = if is_value {
                    visitor.value_bindings.contains(&reference_name)
                } else {
                    visitor.type_bindings.contains(&reference_name)
                };

                if is_defined_in_current_scope {
                    transformer.ctx.error(type_containing_private_name(
                        &reference_name,
                        expr_type
                            .get_identifier_reference()
                            .map_or_else(|| expr_type.span(), |ident| ident.span),
                    ));
                }
            }

            Some(expr_type)
        })
    }
}

impl<'a> Visit<'a> for FunctionReturnType<'a> {
    fn enter_scope(&mut self, _flags: ScopeFlags) {
        self.scope_depth += 1;
    }
    fn leave_scope(&mut self) {
        self.scope_depth -= 1;
    }
    fn visit_binding_identifier(&mut self, ident: &BindingIdentifier<'a>) {
        if self.scope_depth == 0 {
            self.value_bindings.push(ident.name.clone());
        }
    }
    fn visit_ts_type_alias_declaration(&mut self, decl: &TSTypeAliasDeclaration<'a>) {
        if self.scope_depth == 0 {
            self.type_bindings.push(decl.id.name.clone());
        }
    }
    fn visit_function(&mut self, _func: &Function<'a>, _flags: Option<ScopeFlags>) {
        // We don't care about nested functions
    }
    fn visit_return_statement(&mut self, stmt: &ReturnStatement<'a>) {
        self.return_statement_count += 1;
        if self.return_statement_count > 1 {
            return;
        }
        self.return_expression = self.ctx.ast.copy(&stmt.argument);
    }
}
