//! AST visitor which walks AST and delegates token processing to a [`Context`].
//!
//! See [`Visitor`].

use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk};
use oxc_span::GetSpan;

use crate::{context::Context, jsx_state::JSXState, token_type::TokenType};

/// Visitor that walks the AST and delegates token processing to a [`Context`].
///
/// AST visitation is in source order, matching the order of tokens in the iterator.
///
/// Tokens are consumed from `tokens` iterator in source order. When visitor method encounters
/// an AST node that requires a token type override, all preceding tokens are emitted
/// with their default types, then the overridden token is emitted with its corrected type.
/// After the AST walk, any remaining tokens are emitted with default types.
///
/// This wrapper is needed because Rust's orphan rules prevent implementing the foreign [`Visit`] trait
/// directly on [`Context`] implementors (which are generic over `O: ESTreeTokenConfig`).
/// `Visitor` is a local type, so it can implement [`Visit`].
#[repr(transparent)]
pub struct Visitor<C: Context> {
    ctx: C,
}

impl<C: Context> Visitor<C> {
    /// Create new [`Visitor`] with the given [`Context`].
    pub fn new(ctx: C) -> Self {
        Self { ctx }
    }

    /// Get mutable reference to the inner [`Context`].
    pub fn ctx_mut(&mut self) -> &mut C {
        &mut self.ctx
    }

    /// Consume [`Visitor`] and return the inner [`Context`].
    pub fn into_ctx(self) -> C {
        self.ctx
    }
}

impl<'a, C: Context> Visit<'a> for Visitor<C> {
    fn visit_ts_type_name(&mut self, type_name: &TSTypeName<'a>) {
        // `this` is emitted as `Identifier` token instead of `Keyword`
        match type_name {
            TSTypeName::ThisExpression(this_expr) => {
                self.ctx.emit_this_identifier_at(this_expr.span.start);
            }
            TSTypeName::IdentifierReference(ident) => {
                self.visit_identifier_reference(ident);
            }
            TSTypeName::QualifiedName(qualified_name) => {
                self.visit_ts_qualified_name(qualified_name);
            }
        }
    }

    fn visit_ts_import_type(&mut self, import_type: &TSImportType<'a>) {
        // Manual walk.
        // * `source` is a `StringLiteral` — visit to ensure it's emitted with JSON encoding
        //   (string values are not JSON-safe). No-op in update mode.
        // * `options` is an `ObjectExpression`. Manually walk each property, but don't visit the key if it's `with`,
        //   as it needs to remain a `Keyword` token, not get converted to `Identifier`.
        // * `qualifier` and `type_arguments` are visited as usual.
        self.visit_string_literal(&import_type.source);

        if let Some(options) = &import_type.options {
            for property in &options.properties {
                match property {
                    ObjectPropertyKind::ObjectProperty(property) => {
                        let is_with_key = matches!(
                            &property.key,
                            PropertyKey::StaticIdentifier(id) if id.name == "with"
                        );
                        if !is_with_key {
                            self.visit_property_key(&property.key);
                        }
                        self.visit_expression(&property.value);
                    }
                    ObjectPropertyKind::SpreadProperty(spread) => {
                        self.visit_spread_element(spread);
                    }
                }
            }
        }

        if let Some(qualifier) = &import_type.qualifier {
            self.visit_ts_import_type_qualifier(qualifier);
        }

        if let Some(type_arguments) = &import_type.type_arguments {
            self.visit_ts_type_parameter_instantiation(type_arguments);
        }
    }

    fn visit_identifier_name(&mut self, identifier: &IdentifierName<'a>) {
        if self.ctx.is_ts() && self.ctx.jsx_state().should_emit_jsx_identifier() {
            self.ctx.emit_jsx_identifier_at(identifier.span.start, &identifier.name);
        } else {
            self.ctx.emit_identifier_at(identifier.span.start, &identifier.name);
        }
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        if self.ctx.is_ts() && self.ctx.jsx_state().should_emit_jsx_identifier() {
            self.ctx.emit_jsx_identifier_at(identifier.span.start, &identifier.name);
        } else {
            self.ctx.emit_identifier_at(identifier.span.start, &identifier.name);
        }
    }

    fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
        self.ctx.emit_identifier_at(identifier.span.start, &identifier.name);
    }

    fn visit_label_identifier(&mut self, identifier: &LabelIdentifier<'a>) {
        self.ctx.emit_identifier_at(identifier.span.start, &identifier.name);
    }

    fn visit_private_identifier(&mut self, identifier: &PrivateIdentifier<'a>) {
        self.ctx.emit_private_identifier_at(identifier.span.start, &identifier.name);
    }

    fn visit_reg_exp_literal(&mut self, regexp: &RegExpLiteral<'a>) {
        self.ctx.emit_regexp(regexp);
    }

    fn visit_ts_this_parameter(&mut self, parameter: &TSThisParameter<'a>) {
        self.ctx.emit_this_identifier_at(parameter.this_span.start);
        walk::walk_ts_this_parameter(self, parameter);
    }

    fn visit_meta_property(&mut self, _meta_property: &MetaProperty<'a>) {
        // Don't walk.
        // * `meta` (either `import` or `new`) has a `Keyword` token already, which is correct.
        // * `property` (either `meta` or `target`) has an `Identifier` token, which is correct.
    }

    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        // For shorthand `{ x }`, key and value share the same span.
        // Skip the key to avoid emitting the same token twice.
        if !property.shorthand {
            self.visit_property_key(&property.key);
        }
        self.visit_expression(&property.value);
    }

    fn visit_binding_property(&mut self, property: &BindingProperty<'a>) {
        // For shorthand `{ x }`, key and value share the same span.
        // Skip the key to avoid emitting the same token twice.
        if !property.shorthand {
            self.visit_property_key(&property.key);
        }
        self.visit_binding_pattern(&property.value);
    }

    fn visit_import_specifier(&mut self, specifier: &ImportSpecifier<'a>) {
        // For `import { x }`, `imported` and `local` share the same span.
        // Only visit `imported` when it differs from `local`, to avoid emitting the same token twice.
        if specifier.imported.span() != specifier.local.span {
            self.visit_module_export_name(&specifier.imported);
        }
        self.visit_binding_identifier(&specifier.local);
    }

    fn visit_export_specifier(&mut self, specifier: &ExportSpecifier<'a>) {
        // For `export { x }`, `local` and `exported` share the same span.
        // Only visit `exported` when it differs from `local`, to avoid emitting the same token twice.
        self.visit_module_export_name(&specifier.local);
        if specifier.exported.span() != specifier.local.span() {
            self.visit_module_export_name(&specifier.exported);
        }
    }

    fn visit_jsx_identifier(&mut self, identifier: &JSXIdentifier<'a>) {
        self.ctx.emit_jsx_identifier_at(identifier.span.start, &identifier.name);
    }

    fn visit_jsx_element_name(&mut self, name: &JSXElementName<'a>) {
        if let JSXElementName::IdentifierReference(identifier) = name {
            self.ctx.emit_jsx_identifier_at(identifier.span.start, &identifier.name);
        } else {
            walk::walk_jsx_element_name(self, name);
        }
    }

    fn visit_jsx_member_expression_object(&mut self, object: &JSXMemberExpressionObject<'a>) {
        if let JSXMemberExpressionObject::IdentifierReference(identifier) = object {
            self.ctx.emit_jsx_identifier_at(identifier.span.start, &identifier.name);
        } else {
            walk::walk_jsx_member_expression_object(self, object);
        }
    }

    fn visit_jsx_namespaced_name(&mut self, name: &JSXNamespacedName<'a>) {
        if self.ctx.is_js() {
            self.ctx.emit_jsx_identifier_at(name.namespace.span.start, &name.namespace.name);
            self.ctx.emit_jsx_identifier_at(name.name.span.start, &name.name.name);
        } else {
            // In TS mode, these tokens retain their default type (`Identifier`)
        }
    }

    fn visit_jsx_expression_container(&mut self, container: &JSXExpressionContainer<'a>) {
        self.ctx.jsx_state_mut().enter_jsx_expression();
        walk::walk_jsx_expression_container(self, container);
        self.ctx.jsx_state_mut().exit_jsx_expression();
    }

    fn visit_member_expression(&mut self, member_expr: &MemberExpression<'a>) {
        self.ctx.jsx_state_mut().enter_member_expression(member_expr);
        walk::walk_member_expression(self, member_expr);
        self.ctx.jsx_state_mut().exit_member_expression(member_expr);
    }

    fn visit_jsx_spread_attribute(&mut self, attribute: &JSXSpreadAttribute<'a>) {
        self.ctx.jsx_state_mut().enter_jsx_expression();
        walk::walk_jsx_spread_attribute(self, attribute);
        self.ctx.jsx_state_mut().exit_jsx_expression();
    }

    fn visit_jsx_spread_child(&mut self, spread_child: &JSXSpreadChild<'a>) {
        self.ctx.jsx_state_mut().enter_jsx_expression();
        walk::walk_jsx_spread_child(self, spread_child);
        self.ctx.jsx_state_mut().exit_jsx_expression();
    }

    fn visit_string_literal(&mut self, literal: &StringLiteral<'a>) {
        // No-op in update mode - token's `Kind` is already `String`
        self.ctx.emit_unsafe_token_at(literal.span.start, TokenType::new("String"));
    }

    fn visit_jsx_text(&mut self, text: &JSXText<'a>) {
        // Use `emit_unsafe_token_at` not `emit_jsx_text_at`, as the token's `Kind` is already `JSXText`,
        // so no-op in update mode
        self.ctx.emit_unsafe_token_at(text.span.start, TokenType::new("JSXText"));
    }

    fn visit_jsx_attribute(&mut self, attribute: &JSXAttribute<'a>) {
        // Manual walk.
        // * `name`: Visit normally.
        // * `value`: Set `JSXText` token type if it's a `StringLiteral`.
        self.visit_jsx_attribute_name(&attribute.name);
        match &attribute.value {
            Some(JSXAttributeValue::StringLiteral(string_literal)) => {
                // Use `emit_jsx_text_at` not `emit_unsafe_token_at`, as the token `Kind`
                // needs to be updated to `JSXText` in update mode
                self.ctx.emit_jsx_text_at(string_literal.span.start);
            }
            Some(value) => self.visit_jsx_attribute_value(value),
            None => {}
        }
    }

    fn visit_template_literal(&mut self, literal: &TemplateLiteral<'a>) {
        C::walk_template_quasis_interleaved(
            self,
            &literal.quasis,
            Visit::visit_expression,
            &literal.expressions,
        );
    }

    fn visit_ts_template_literal_type(&mut self, literal: &TSTemplateLiteralType<'a>) {
        C::walk_template_quasis_interleaved(
            self,
            &literal.quasis,
            Visit::visit_ts_type,
            &literal.types,
        );
    }
}
