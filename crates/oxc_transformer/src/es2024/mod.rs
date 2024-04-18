#![allow(dead_code, unused_imports)]

use std::rc::Rc;

use oxc_ast::ast::{RegExpFlags, RegExpLiteral};

use crate::context::Ctx;

pub struct Es2024<'a> {
    ctx: Ctx<'a>,
}

impl<'a> Es2024<'a> {
    pub fn new(ctx: &Ctx<'a>) -> Self {
        Self { ctx: Rc::clone(ctx) }
    }

    pub fn transform_reg_expr_literal(&mut self, _lit: &mut RegExpLiteral<'a>) {
        // if !lit.regex.flags.contains(RegExpFlags::V) {
        //     return;
        // }

        // lit.regex.flags.remove(RegExpFlags::V);
        // lit.regex.flags.insert(RegExpFlags::U);
    }
}
