//! Visitor to count nodes, scopes, symbols and references in AST.
//! These counts can be used to pre-allocate sufficient capacity in `AstNodes`,
//! `ScopeTree`, and `SymbolTable` to store info for all these items.

use std::{cell::Cell, ops::Deref};

use oxc_ast::{
    ast::{
        BindingIdentifier, IdentifierReference, JSXElementName, JSXMemberExpressionObject,
        TSEnumMemberName, TSModuleDeclarationName,
    },
    visit::walk::{walk_ts_enum_member_name, walk_ts_module_declaration_name},
    AstKind, Statistics, Visit,
};
use oxc_syntax::scope::{ScopeFlags, ScopeId};

#[allow(clippy::struct_field_names)]
#[derive(Default, Debug)]
pub(crate) struct Counter {
    statistics: Statistics,
}
impl Deref for Counter {
    type Target = Statistics;
    fn deref(&self) -> &Self::Target {
        &self.statistics
    }
}
impl Counter {
    pub fn into_inner(self) -> Statistics {
        self.statistics
    }
}

impl<'a> Visit<'a> for Counter {
    #[inline]
    fn enter_node(&mut self, _: AstKind<'a>) {
        self.observe_node();
    }
    #[inline]
    fn enter_scope(&mut self, _: ScopeFlags, _: &Cell<Option<ScopeId>>) {
        // self.scopes_count += 1;
        self.observe_scope();
    }

    #[inline]
    fn visit_binding_identifier(&mut self, _: &BindingIdentifier<'a>) {
        // self.nodes_count += 1;
        // self.symbols_count += 1;
        self.observe_symbol();
        self.observe_node();
    }

    #[inline]
    fn visit_identifier_reference(&mut self, _: &IdentifierReference<'a>) {
        // self.nodes_count += 1;
        // self.references_count += 1;
        self.observe_node();
        self.observe_reference();
    }

    #[inline]
    fn visit_jsx_member_expression_object(&mut self, it: &JSXMemberExpressionObject<'a>) {
        // self.nodes_count += 1;
        self.observe_node();
        match it {
            JSXMemberExpressionObject::MemberExpression(expr) => {
                self.visit_jsx_member_expression(expr);
            }
            JSXMemberExpressionObject::Identifier(_) => {
                self.observe_node();
                self.observe_reference();
            }
        }
    }

    #[inline]
    fn visit_jsx_element_name(&mut self, it: &JSXElementName<'a>) {
        self.observe_node();
        match it {
            JSXElementName::Identifier(ident) => {
                self.observe_node();
                if ident.name.chars().next().is_some_and(char::is_uppercase) {
                    self.observe_reference();
                }
            }
            JSXElementName::NamespacedName(name) => self.visit_jsx_namespaced_name(name),
            JSXElementName::MemberExpression(expr) => self.visit_jsx_member_expression(expr),
        }
    }

    #[inline]
    fn visit_ts_enum_member_name(&mut self, it: &TSEnumMemberName<'a>) {
        if !it.is_expression() {
            self.observe_symbol();
        }
        walk_ts_enum_member_name(self, it);
    }

    #[inline]
    fn visit_ts_module_declaration_name(&mut self, it: &TSModuleDeclarationName<'a>) {
        self.observe_symbol();
        walk_ts_module_declaration_name(self, it);
    }
}
