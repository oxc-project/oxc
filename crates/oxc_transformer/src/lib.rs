#![allow(clippy::wildcard_imports, clippy::option_map_unit_fn)]

//! Transformer / Transpiler
//!
//! References:
//! * <https://www.typescriptlang.org/tsconfig#target>
//! * <https://babel.dev/docs/presets>
//! * <https://github.com/microsoft/TypeScript/blob/main/src/compiler/transformer.ts>

mod es2016;
mod es2019;
mod es2021;
mod options;
mod react_jsx;
mod typescript;

use oxc_allocator::Allocator;
use oxc_ast::{ast::*, AstBuilder, VisitMut};
use oxc_span::SourceType;
use std::rc::Rc;

use es2016::ExponentiationOperator;
use es2019::OptionalCatchBinding;
use es2021::LogicalAssignmentOperators;
use react_jsx::ReactJsx;
use typescript::TypeScript;

pub use crate::options::{
    TransformOptions, TransformReactOptions, TransformReactRuntime, TransformTarget,
};

#[derive(Default)]
pub struct Transformer<'a> {
    typescript: Option<TypeScript<'a>>,
    react_jsx: Option<ReactJsx<'a>>,
    // es2021
    es2021_logical_assignment_operators: Option<LogicalAssignmentOperators<'a>>,
    // es2019
    es2019_optional_catch_binding: Option<OptionalCatchBinding<'a>>,
    // es2016
    es2016_exponentiation_operator: Option<ExponentiationOperator<'a>>,
}

impl<'a> Transformer<'a> {
    pub fn new(
        allocator: &'a Allocator,
        source_type: SourceType,
        options: TransformOptions,
    ) -> Self {
        let ast = Rc::new(AstBuilder::new(allocator));

        let mut t = Self::default();
        if source_type.is_typescript() {
            t.typescript.replace(TypeScript::new(Rc::clone(&ast)));
        }
        if let Some(react_options) = options.react {
            t.react_jsx.replace(ReactJsx::new(Rc::clone(&ast), react_options));
        }
        if options.target < TransformTarget::ES2021 {
            t.es2021_logical_assignment_operators
                .replace(LogicalAssignmentOperators::new(Rc::clone(&ast)));
        }
        if options.target < TransformTarget::ES2019 {
            t.es2019_optional_catch_binding.replace(OptionalCatchBinding::new(Rc::clone(&ast)));
        }
        if options.target < TransformTarget::ES2016 {
            t.es2016_exponentiation_operator.replace(ExponentiationOperator::new(Rc::clone(&ast)));
        }
        t
    }

    pub fn build<'b>(mut self, program: &'b mut Program<'a>) {
        self.visit_program(program);
    }
}

impl<'a, 'b> VisitMut<'a, 'b> for Transformer<'a> {
    fn visit_expression(&mut self, expr: &'b mut Expression<'a>) {
        // self.typescript.as_mut().map(|t| t.transform_expression(expr));
        // self.react_jsx.as_mut().map(|t| t.transform_expression(expr));
        self.es2021_logical_assignment_operators.as_mut().map(|t| t.transform_expression(expr));
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
