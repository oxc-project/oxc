use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::{Span, SPAN};
use oxc_traverse::{Ancestor, FinderRet, TraverseCtx};

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

    pub fn transform_jsx_opening_element(&self, elem: &mut JSXOpeningElement<'a>) {
        self.add_self_this_attribute(elem);
    }

    pub fn get_object_property_kind_for_jsx_plugin(&self) -> ObjectPropertyKind<'a> {
        let kind = PropertyKind::Init;
        let key = self.ctx.ast.property_key_identifier_name(SPAN, SELF);
        let value = self.ctx.ast.expression_this(SPAN);
        self.ctx
            .ast
            .object_property_kind_object_property(SPAN, kind, key, value, None, false, false, false)
    }

    pub fn report_error(&self, span: Span) {
        let error = OxcDiagnostic::warn("Duplicate __self prop found.").with_label(span);
        self.ctx.error(error);
    }

    #[allow(clippy::unused_self)]
    fn is_inside_constructor(&self, ctx: &TraverseCtx<'a>) -> bool {
        ctx.find_scope_by_flags(|flags| {
            if flags.is_block() || flags.is_arrow() {
                return FinderRet::Continue;
            }
            FinderRet::Found(flags.is_constructor())
        })
        .unwrap_or(false)
    }

    fn has_no_super_class(ctx: &TraverseCtx<'a>) -> bool {
        ctx.find_ancestor(|ancestor| match ancestor {
            Ancestor::ClassBody(class) => FinderRet::Found(class.super_class().is_none()),
            _ => FinderRet::Continue,
        })
        .unwrap_or(true)
    }

    pub fn can_add_self_attribute(&self, ctx: &TraverseCtx<'a>) -> bool {
        !self.is_inside_constructor(ctx) || Self::has_no_super_class(ctx)
    }
}

impl<'a> ReactJsxSelf<'a> {
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
