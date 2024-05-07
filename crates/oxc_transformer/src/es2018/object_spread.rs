use std::rc::Rc;

use oxc_ast::{ast::*, AstBuilder};
use oxc_span::{CompactStr, SPAN};
use serde::Deserialize;

use crate::{context::Ctx, helpers::module_imports::NamedImport};

#[derive(Debug, Default, Clone, Copy, Deserialize)]
pub struct ObjectSpreadOptions {
    pub set_spread_properties: bool,
    pub pure_getters: bool,
}

pub struct ObjectSpread<'a> {
    ctx: Ctx<'a>,
    options: ObjectSpreadOptions,
}

impl<'a> ObjectSpread<'a> {
    pub fn new(options: ObjectSpreadOptions, ctx: &Ctx<'a>) -> Self {
        Self { ctx: Rc::clone(ctx), options }
    }

    fn ast(&self) -> &AstBuilder<'a> {
        &self.ctx.ast
    }

    fn get_static_member_expression(
        &self,
        object_ident_name: &str,
        property_name: &str,
    ) -> Expression<'a> {
        let property = IdentifierName::new(SPAN, self.ast().new_atom(property_name));
        let ident = IdentifierReference::new(SPAN, self.ast().new_atom(object_ident_name));
        let object = self.ast().identifier_reference_expression(ident);
        self.ast().static_member_expression(SPAN, object, property, false)
    }

    fn get_extend_object_callee(&mut self) -> Expression<'a> {
        if self.options.set_spread_properties {
            self.get_static_member_expression("Object", "assign")
        } else {
            if self.ctx.source_type.is_module() {
                self.add_import_statement(
                    "default",
                    "_objectSpread",
                    "@babel/helpers/lib/helpers/objectSpread2.js".into(),
                );
            } else {
                self.add_require_statement(
                    "_objectSpread",
                    "@babel/helpers/lib/helpers/objectSpread2.js".into(),
                    true,
                );
            }

            let ident = IdentifierReference::new(SPAN, self.ast().new_atom("_objectSpread"));
            self.ast().identifier_reference_expression(ident)
        }
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
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

        let mut obj_prop_list = self.ast().new_vec();
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

        self.transform_expression(expr);
        let mut args = self.ast().new_vec();
        args.push(Argument::from(self.ast().move_expression(expr)));
        args.push(Argument::from(self.ast().move_expression(&mut spread_prop.argument)));

        let callee = self.get_extend_object_callee();

        *expr = self.ast().call_expression(SPAN, callee, args, false, None);

        if !obj_prop_list.is_empty() {
            obj_prop_list.reverse();
            let mut args = self.ast().new_vec();
            args.push(Argument::from(self.ast().move_expression(expr)));
            args.push(Argument::from(self.ast().object_expression(SPAN, obj_prop_list, None)));

            let callee = self.get_extend_object_callee();

            *expr = self.ast().call_expression(SPAN, callee, args, false, None);
        }
    }

    pub fn transform_program_on_exit(&mut self, program: &mut Program<'a>) {
        let imports = self.ctx.module_imports.get_import_statements();
        let index = program
            .body
            .iter()
            .rposition(|stmt| matches!(stmt, Statement::ImportDeclaration(_)))
            .map_or(0, |i| i + 1);
        program.body.splice(index..index, imports);
    }

    fn add_import_statement(&mut self, imported: &str, local: &str, source: CompactStr) {
        let import = NamedImport::new(imported.into(), Some(local.into()));
        self.ctx.module_imports.add_import(source, import);
    }

    fn add_require_statement(&mut self, variable_name: &str, source: CompactStr, front: bool) {
        let import = NamedImport::new(variable_name.into(), None);
        self.ctx.module_imports.add_require(source, import, front);
    }
}
