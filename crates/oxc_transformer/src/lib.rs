#![allow(clippy::wildcard_imports, clippy::option_map_unit_fn)]

//! Transformer / Transpiler
//!
//! References:
//! * <https://www.typescriptlang.org/tsconfig#target>
//! * <https://babel.dev/docs/presets>
//! * <https://github.com/microsoft/TypeScript/blob/main/src/compiler/transformer.ts>

mod es2016;
mod es2019;

use oxc_allocator::Allocator;
use oxc_ast::{ast::*, AstBuilder, VisitMut};
use std::rc::Rc;

use es2016::ExponentiationOperator;
use es2019::OptionalCatchBinding;

#[derive(Debug, Default, Clone)]
pub struct TransformOptions {
    pub target: TransformTarget,
}

/// See <https://www.typescriptlang.org/tsconfig#target>
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum TransformTarget {
    ES2015,
    ES2016,
    ES2019,
    #[default]
    ESNext,
}

#[derive(Default)]
pub struct Transformer<'a> {
    // es2016
    es2016_exponentiation_operator: Option<ExponentiationOperator<'a>>,
    // es2019
    es2019_optional_catch_binding: Option<OptionalCatchBinding<'a>>,
}

impl<'a> Transformer<'a> {
    pub fn new(allocator: &'a Allocator, options: &TransformOptions) -> Self {
        let ast = Rc::new(AstBuilder::new(allocator));

        let mut t = Self::default();
        if options.target < TransformTarget::ES2016 {
            t.es2016_exponentiation_operator.replace(ExponentiationOperator::new(Rc::clone(&ast)));
        }
        if options.target < TransformTarget::ES2019 {
            t.es2019_optional_catch_binding.replace(OptionalCatchBinding::new(Rc::clone(&ast)));
        }
        t
    }

    pub fn build<'b>(mut self, program: &'b mut Program<'a>) {
        self.visit_program(program);
    }
}

impl<'a, 'b> VisitMut<'a, 'b> for Transformer<'a> {
    fn visit_expression(&mut self, expr: &'b mut Expression<'a>) {
        self.es2016_exponentiation_operator.as_mut().map(|t| t.transform_expression(expr));

        self.visit_expression_match(expr);
    }

    fn visit_catch_clause(&mut self, clause: &'b mut CatchClause<'a>) {
        self.es2019_optional_catch_binding.as_mut().map(|t| t.transform_catch_clause(clause));

        if let Some(param) = &mut clause.param {
            self.visit_binding_pattern(param);
        }
        self.visit_statements(&mut clause.body.body);
    }
}
