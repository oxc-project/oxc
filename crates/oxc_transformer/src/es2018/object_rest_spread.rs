use oxc_ast::ast::*;
use serde::Deserialize;

use crate::{context::Ctx, CompilerAssumptions};

use super::object_spread::{ObjectSpread, ObjectSpreadOptions};

#[derive(Debug, Default, Clone, Copy, Deserialize)]
pub struct ObjectRestSpreadOptions {
    #[serde(rename = "loose")]
    _loose: bool,

    #[serde(rename = "useBuiltIns")]
    _use_built_ins: bool,
}

/// [plugin-transform-object-rest-spread](https://babeljs.io/docs/babel-plugin-transform-object-rest-spread)
///
/// This plugin transforms rest properties for object destructuring assignment and spread properties for object literals.
///
/// This plugin is included in `preset-env`
///
/// References:
///
/// * <https://babeljs.io/docs/babel-plugin-transform-object-rest-spread>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-object-rest-spread>
pub struct ObjectRestSpread<'a> {
    object_spread: ObjectSpread<'a>,
}

impl<'a> ObjectRestSpread<'a> {
    pub fn new(assumptions: CompilerAssumptions, ctx: &Ctx<'a>) -> Self {
        Self {
            object_spread: ObjectSpread::new(
                ObjectSpreadOptions {
                    set_spread_properties: assumptions.set_spread_properties,
                    pure_getters: assumptions.pure_getters,
                },
                ctx,
            ),
        }
    }

    pub fn transform_expression(&mut self, expr: &mut Expression<'a>) {
        self.object_spread.transform_expression(expr);
    }

    pub fn transform_program_on_exit(&mut self, program: &mut Program<'a>) {
        self.object_spread.transform_program_on_exit(program);
    }
}
