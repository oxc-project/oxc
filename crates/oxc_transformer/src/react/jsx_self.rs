use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{Span, SPAN};
use oxc_traverse::{Ancestor, Traverse, TraverseCtx};

use crate::context::Ctx;

const SELF: &str = "__self";

/// [plugin-transform-react-jsx-self](https://babeljs.io/docs/babel-plugin-transform-react-jsx-self)
///
/// This plugin is included in `preset-react` and only enabled in development mode.
///
/// ## Example
///
/// In: `<sometag />`
/// Out: `<sometag __self={this} />`
pub struct ReactJsxSelf<'a> {
    ctx: Ctx<'a>,
}

impl<'a> ReactJsxSelf<'a> {
    pub fn new(ctx: Ctx<'a>) -> Self {
        Self { ctx }
    }
}

impl<'a> Traverse<'a> for ReactJsxSelf<'a> {
    fn enter_jsx_opening_element(
        &mut self,
        elem: &mut JSXOpeningElement<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        self.add_self_this_attribute(elem);
    }
}

impl<'a> ReactJsxSelf<'a> {
    pub fn report_error(&self, span: Span) {
        let error = OxcDiagnostic::warn("Duplicate __self prop found.").with_label(span);
        self.ctx.error(error);
    }

    #[allow(clippy::unused_self)]
    fn is_inside_constructor(&self, ctx: &TraverseCtx<'a>) -> bool {
        for scope_id in ctx.ancestor_scopes() {
            let flags = ctx.scopes().get_flags(scope_id);
            if flags.is_block() || flags.is_arrow() {
                continue;
            }
            return flags.is_constructor();
        }
        unreachable!(); // Always hit `Program` and exit before loop ends
    }

    fn has_no_super_class(ctx: &TraverseCtx<'a>) -> bool {
        for ancestor in ctx.ancestors() {
            if let Ancestor::ClassBody(class) = ancestor {
                return class.super_class().is_none();
            }
        }
        true
    }

    pub fn get_object_property_kind_for_jsx_plugin(&self) -> ObjectPropertyKind<'a> {
        let kind = PropertyKind::Init;
        let key = self.ctx.ast.property_key_identifier_name(SPAN, SELF);
        let value = self.ctx.ast.expression_this(SPAN);
        self.ctx
            .ast
            .object_property_kind_object_property(SPAN, kind, key, value, None, false, false, false)
    }

    pub fn can_add_self_attribute(&self, ctx: &TraverseCtx<'a>) -> bool {
        !self.is_inside_constructor(ctx) || Self::has_no_super_class(ctx)
    }

    /// `<div __self={this} />`
    ///       ^^^^^^^^^^^^^
    fn add_self_this_attribute(&self, elem: &mut JSXOpeningElement<'a>) {
        // Check if `__self` attribute already exists
        for item in &elem.attributes {
            if let JSXAttributeItem::Attribute(attribute) = item {
                if let JSXAttributeName::Identifier(ident) = &attribute.name {
                    if ident.name == SELF {
                        self.report_error(ident.span);
                        return;
                    }
                }
            }
        }

        let name = self.ctx.ast.jsx_attribute_name_jsx_identifier(SPAN, SELF);
        let value = {
            let jsx_expr = JSXExpression::from(self.ctx.ast.expression_this(SPAN));
            self.ctx.ast.jsx_attribute_value_jsx_expression_container(SPAN, jsx_expr)
        };
        let attribute = self.ctx.ast.jsx_attribute_item_jsx_attribute(SPAN, name, Some(value));
        elem.attributes.push(attribute);
    }
}
