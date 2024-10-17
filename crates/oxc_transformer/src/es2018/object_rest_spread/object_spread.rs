//! ES2018 object spread transformation.
//!
//! This plugin transforms object spread properties (`{ ...x }`) to a series of `_objectSpread` calls.
//!
//! > This plugin is included in `preset-env`, in ES2018
//!
//! ## Example
//!
//! Input:
//! ```js
//! var x = { a: 1, b: 2 };
//! var y = { ...x, c: 3 };
//! ```
//!
//! Output:
//! ```js
//! var x = { a: 1, b: 2 };
//! var y = _objectSpread({}, x, { c: 3 });
//! ```
//!
//! ## Implementation
//!
//! Implementation based on [@babel/plugin-transform-object-rest-spread](https://babeljs.io/docs/babel-plugin-transform-object-rest-spread).
//!
//! ## References:
//! * Babel plugin implementation: <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-object-rest-spread>
//! * Object rest/spread TC39 proposal: <https://github.com/tc39/proposal-object-rest-spread>

use oxc_ast::{ast::*, NONE};
use oxc_semantic::{ReferenceFlags, SymbolId};
use oxc_span::SPAN;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::{common::helper_loader::Helper, TransformCtx};

use super::ObjectRestSpreadOptions;

pub struct ObjectSpread<'a, 'ctx> {
    options: ObjectRestSpreadOptions,
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> ObjectSpread<'a, 'ctx> {
    pub fn new(options: ObjectRestSpreadOptions, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { options, ctx }
    }
}
impl<'a, 'ctx> Traverse<'a> for ObjectSpread<'a, 'ctx> {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::ObjectExpression(obj_expr) = expr else {
            return;
        };

        if obj_expr
            .properties
            .iter()
            .all(|prop| matches!(prop, ObjectPropertyKind::ObjectProperty(..)))
        {
            return;
        }

        // collect `y` and `z` from `{ ...x, y, z }`
        let mut obj_prop_list = ctx.ast.vec();
        while obj_expr
            .properties
            .last()
            .map_or(false, |prop| matches!(prop, ObjectPropertyKind::ObjectProperty(..)))
        {
            let prop = obj_expr.properties.pop().unwrap();
            obj_prop_list.push(prop);
        }

        let Some(ObjectPropertyKind::SpreadProperty(mut spread_prop)) = obj_expr.properties.pop()
        else {
            unreachable!();
        };

        let mut arguments = ctx.ast.vec();
        arguments.push(Argument::from(ctx.ast.move_expression(expr)));
        arguments.push(Argument::from(ctx.ast.move_expression(&mut spread_prop.argument)));

        let object_id = self.get_object_symbol_id(ctx);
        let callee = self.get_extend_object_callee(object_id, ctx);

        // ({ ...x }) => _objectSpread({}, x)
        *expr = ctx.ast.expression_call(SPAN, callee, NONE, arguments, false);

        // ({ ...x, y, z }) => _objectSpread(_objectSpread({}, x), { y, z });
        if !obj_prop_list.is_empty() {
            obj_prop_list.reverse();
            let mut arguments = ctx.ast.vec();
            arguments.push(Argument::from(ctx.ast.move_expression(expr)));
            arguments.push(Argument::from(ctx.ast.expression_object(SPAN, obj_prop_list, None)));

            let callee = self.get_extend_object_callee(object_id, ctx);

            *expr = ctx.ast.expression_call(SPAN, callee, NONE, arguments, false);
        }
    }
}

impl<'a, 'ctx> ObjectSpread<'a, 'ctx> {
    #[expect(clippy::option_option)]
    fn get_object_symbol_id(&self, ctx: &mut TraverseCtx<'a>) -> Option<Option<SymbolId>> {
        if self.options.set_spread_properties {
            Some(ctx.scopes().find_binding(ctx.current_scope_id(), "Object"))
        } else {
            None
        }
    }

    #[expect(clippy::option_option)]
    fn get_extend_object_callee(
        &mut self,
        object_id: Option<Option<SymbolId>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        if let Some(object_id) = object_id {
            Self::object_assign(object_id, ctx)
        } else {
            self.babel_external_helper(ctx)
        }
    }

    fn object_assign(symbol_id: Option<SymbolId>, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        let ident =
            ctx.create_reference_id(SPAN, Atom::from("Object"), symbol_id, ReferenceFlags::Read);
        let object = ctx.ast.expression_from_identifier_reference(ident);
        let property = ctx.ast.identifier_name(SPAN, Atom::from("assign"));

        Expression::from(ctx.ast.member_expression_static(SPAN, object, property, false))
    }

    fn babel_external_helper(&self, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        self.ctx.helper_load(Helper::ObjectSpread2, ctx)
    }
}
