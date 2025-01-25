use std::cell::Cell;

use oxc_allocator::CloneIn;
use oxc_ast::{
    ast::{
        ArrowFunctionExpression, BindingIdentifier, Expression, Function, FunctionBody,
        ReturnStatement, TSType, TSTypeAliasDeclaration, TSTypeName, TSTypeQueryExprName,
    },
    AstBuilder, Visit,
};
use oxc_span::{Atom, GetSpan, SPAN};
use oxc_syntax::scope::{ScopeFlags, ScopeId};

use crate::{diagnostics::type_containing_private_name, IsolatedDeclarations};

/// Infer return type from return statement.
/// ```ts
/// function foo() {
///    return 1;
/// }
/// // inferred type is number
///
/// function bar() {
///   if (true) {
///    return;
///   }
///   return 1;
/// }
/// // inferred type is number | undefined
///
/// function baz() {
///  if (true) {
///   return null;
///  }
///  return 1;
/// }
/// // We can't infer return type if there are multiple return statements with different types
/// ```
#[allow(clippy::option_option)]
pub struct FunctionReturnType<'a> {
    ast: AstBuilder<'a>,
    return_expression: Option<Option<Expression<'a>>>,
    value_bindings: Vec<Atom<'a>>,
    type_bindings: Vec<Atom<'a>>,
    return_statement_count: u8,
    scope_depth: u32,
}

impl<'a> FunctionReturnType<'a> {
    pub fn infer(
        transformer: &IsolatedDeclarations<'a>,
        body: &FunctionBody<'a>,
    ) -> Option<TSType<'a>> {
        let mut visitor = FunctionReturnType {
            ast: transformer.ast,
            return_expression: None,
            return_statement_count: 0,
            scope_depth: 0,
            value_bindings: Vec::default(),
            type_bindings: Vec::default(),
        };

        visitor.visit_function_body(body);

        let expr = visitor.return_expression??;
        let Some(mut expr_type) = transformer.infer_type_from_expression(&expr) else {
            // Avoid report error in parent function
            return if expr.is_function() {
                Some(transformer.ast.ts_type_unknown_keyword(SPAN))
            } else {
                None
            };
        };

        if let Some((reference_name, is_value)) = match &expr_type {
            TSType::TSTypeReference(type_reference) => {
                if let TSTypeName::IdentifierReference(ident) = &type_reference.type_name {
                    Some((ident.name, false))
                } else {
                    None
                }
            }
            TSType::TSTypeQuery(query) => {
                if let TSTypeQueryExprName::IdentifierReference(ident) = &query.expr_name {
                    Some((ident.name, true))
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
                transformer.error(type_containing_private_name(
                    &reference_name,
                    expr_type
                        .get_identifier_reference()
                        .map_or_else(|| expr_type.span(), |ident| ident.span),
                ));
            }
        }

        // If there are multiple return statements, which means there must be a union with `undefined`
        if visitor.return_statement_count > 1 {
            // Here is a union type, if the return type is a function type, we need to wrap it in parentheses
            if matches!(expr_type, TSType::TSFunctionType(_)) {
                expr_type = transformer.ast.ts_type_parenthesized_type(SPAN, expr_type);
            }

            let types = transformer
                .ast
                .vec_from_array([expr_type, transformer.ast.ts_type_undefined_keyword(SPAN)]);
            expr_type = transformer.ast.ts_type_union_type(SPAN, types);
        }
        Some(expr_type)
    }
}

impl<'a> Visit<'a> for FunctionReturnType<'a> {
    fn enter_scope(&mut self, _flags: ScopeFlags, _: &Cell<Option<ScopeId>>) {
        self.scope_depth += 1;
    }

    fn leave_scope(&mut self) {
        self.scope_depth -= 1;
    }

    fn visit_binding_identifier(&mut self, ident: &BindingIdentifier<'a>) {
        if self.scope_depth == 0 {
            self.value_bindings.push(ident.name);
        }
    }

    fn visit_ts_type_alias_declaration(&mut self, decl: &TSTypeAliasDeclaration<'a>) {
        if self.scope_depth == 0 {
            self.type_bindings.push(decl.id.name);
        }
    }

    fn visit_function(&mut self, _func: &Function<'a>, _flags: ScopeFlags) {
        // We don't care about nested functions
    }

    fn visit_arrow_function_expression(&mut self, _expr: &ArrowFunctionExpression<'a>) {
        // We don't care about nested functions
    }

    fn visit_return_statement(&mut self, stmt: &ReturnStatement<'a>) {
        self.return_statement_count += 1;
        if self.return_statement_count > 1 {
            if let Some(expr) = &self.return_expression {
                // if last return statement is not empty, we can't infer return type
                if expr.is_some() {
                    self.return_expression = None;
                    return;
                }
            } else {
                return;
            }
        }

        self.return_expression = Some(stmt.argument.clone_in(self.ast.allocator));
    }
}
