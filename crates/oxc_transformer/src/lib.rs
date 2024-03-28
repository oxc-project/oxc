#![allow(clippy::wildcard_imports, clippy::option_map_unit_fn)]

//! Transformer / Transpiler
//!
//! References:
//! * <https://www.typescriptlang.org/tsconfig#target>
//! * <https://babel.dev/docs/presets>
//! * <https://github.com/microsoft/TypeScript/blob/main/src/compiler/transformer.ts>

mod context;
mod es2015;
mod es2016;
mod es2019;
mod es2020;
mod es2021;
mod es2022;
mod es3;
mod options;
mod proposals;
mod react_jsx;
mod regexp;
#[cfg(test)]
mod tester;
mod typescript;
mod utils;

use std::{rc::Rc, sync::Arc};

use es2015::TemplateLiterals;
use oxc_allocator::Allocator;
use oxc_ast::{
    ast::*,
    visit::walk_mut::{
        walk_assignment_expression_mut, walk_catch_clause_mut, walk_class_body_mut,
        walk_declaration_mut, walk_expression_mut, walk_function_mut, walk_method_definition_mut,
        walk_object_expression_mut, walk_object_property_mut, walk_program_mut, walk_statement_mut,
        walk_statements_mut, walk_variable_declarator_mut,
    },
    AstBuilder, VisitMut,
};
use oxc_diagnostics::Error;
use oxc_semantic::{ScopeFlags, Semantic};
use oxc_span::SourceType;
use proposals::Decorators;

use crate::{
    context::TransformerCtx,
    es2015::*,
    es2016::ExponentiationOperator,
    es2019::{JsonStrings, OptionalCatchBinding},
    es2020::NullishCoalescingOperator,
    es2021::{LogicalAssignmentOperators, NumericSeparator},
    es2022::ClassStaticBlock,
    es3::PropertyLiteral,
    react_jsx::ReactJsx,
    regexp::RegexpFlags,
    typescript::TypeScript,
    utils::CreateVars,
};

pub use crate::{
    es2015::ArrowFunctionsOptions,
    es2020::NullishCoalescingOperatorOptions,
    options::{TransformOptions, TransformTarget},
    proposals::DecoratorsOptions,
    react_jsx::{ReactJsxOptions, ReactJsxRuntime, ReactJsxRuntimeOption},
    typescript::TypescriptOptions,
};

pub struct Transformer<'a> {
    ctx: TransformerCtx<'a>,
    decorators: Option<Decorators<'a>>,
    #[allow(unused)]
    typescript: Option<TypeScript<'a>>,
    react_jsx: Option<ReactJsx<'a>>,
    regexp_flags: Option<RegexpFlags<'a>>,
    // es2022
    es2022_class_static_block: Option<ClassStaticBlock<'a>>,
    // es2021
    es2021_logical_assignment_operators: Option<LogicalAssignmentOperators<'a>>,
    es2021_numeric_separator: Option<NumericSeparator<'a>>,
    // es2020
    es2020_nullish_coalescing_operators: Option<NullishCoalescingOperator<'a>>,
    // es2019
    es2019_json_strings: Option<JsonStrings<'a>>,
    es2019_optional_catch_binding: Option<OptionalCatchBinding<'a>>,
    // es2016
    es2016_exponentiation_operator: Option<ExponentiationOperator<'a>>,
    // es2015
    es2015_function_name: Option<FunctionName<'a>>,
    es2015_arrow_functions: Option<ArrowFunctions<'a>>,
    es2015_shorthand_properties: Option<ShorthandProperties<'a>>,
    es2015_template_literals: Option<TemplateLiterals<'a>>,
    es2015_duplicate_keys: Option<DuplicateKeys<'a>>,
    es2015_instanceof: Option<Instanceof<'a>>,
    es2015_literals: Option<Literals<'a>>,
    es2015_new_target: Option<NewTarget<'a>>,
    es3_property_literal: Option<PropertyLiteral<'a>>,
}

impl<'a> Transformer<'a> {
    #[rustfmt::skip]
    pub fn new(
        allocator: &'a Allocator,
        source_type: SourceType,
        semantic: Semantic<'a>,
        options: TransformOptions,
    ) -> Self {
        let ast = Rc::new(AstBuilder::new(allocator));
        let ctx = TransformerCtx::new(
            ast,
            semantic,
            options,
        );

        Self {
            decorators: Decorators::new(ctx.clone()),
            typescript: source_type.is_typescript().then(|| TypeScript::new(ctx.clone())),
            regexp_flags: RegexpFlags::new(ctx.clone()),
            // es2022
            es2022_class_static_block: es2022::ClassStaticBlock::new(ctx.clone()),
            // es2021
            es2021_logical_assignment_operators: LogicalAssignmentOperators::new(ctx.clone()),
            es2021_numeric_separator: NumericSeparator::new(ctx.clone()),
            // es2020
            es2020_nullish_coalescing_operators: NullishCoalescingOperator::new(ctx.clone()),
            // es2019
            es2019_json_strings: JsonStrings::new(ctx.clone()),
            es2019_optional_catch_binding: OptionalCatchBinding::new(ctx.clone()),
            // es2016
            es2016_exponentiation_operator: ExponentiationOperator::new(ctx.clone()),
            // es2015
            es2015_function_name: FunctionName::new(ctx.clone()),
            es2015_arrow_functions: ArrowFunctions::new(ctx.clone()),
            es2015_shorthand_properties: ShorthandProperties::new(ctx.clone()),
            es2015_template_literals: TemplateLiterals::new(ctx.clone()),
            es2015_duplicate_keys: DuplicateKeys::new(ctx.clone()),
            es2015_instanceof: Instanceof::new(ctx.clone()),
            es2015_literals: Literals::new(ctx.clone()),
            es2015_new_target: NewTarget::new(ctx.clone()),
            // other
            es3_property_literal: PropertyLiteral::new(ctx.clone()),
            react_jsx: ReactJsx::new(ctx.clone()),
            // original context
            ctx,
        }
    }

    /// # Errors
    /// Returns `Vec<Error>` if any errors were collected during the transformation.
    pub fn build(mut self, program: &mut Program<'a>) -> Result<(), Vec<Error>> {
        self.visit_program(program);
        let errors: Vec<_> = self
            .ctx
            .errors()
            .into_iter()
            .map(|e| e.with_source_code(Arc::new(self.ctx.semantic().source_text().to_string())))
            .collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl<'a> VisitMut<'a> for Transformer<'a> {
    fn visit_program(&mut self, program: &mut Program<'a>) {
        walk_program_mut(self, program);
        self.typescript.as_mut().map(|t| t.transform_program(program));
        self.react_jsx.as_mut().map(|t| t.add_react_jsx_runtime_imports(program));
        self.decorators.as_mut().map(|t| t.transform_program(program));
    }

    fn visit_assignment_expression(&mut self, expr: &mut AssignmentExpression<'a>) {
        self.es2015_function_name.as_mut().map(|t| t.transform_assignment_expression(expr));
        walk_assignment_expression_mut(self, expr);
    }

    fn visit_statements(&mut self, stmts: &mut oxc_allocator::Vec<'a, Statement<'a>>) {
        self.typescript.as_mut().map(|t| t.transform_statements(stmts));

        walk_statements_mut(self, stmts);

        // TODO: we need scope id to insert the vars into the correct statements
        self.es2021_logical_assignment_operators.as_mut().map(|t| t.add_vars_to_statements(stmts));
        self.es2020_nullish_coalescing_operators.as_mut().map(|t| t.add_vars_to_statements(stmts));
        self.es2016_exponentiation_operator.as_mut().map(|t| t.add_vars_to_statements(stmts));
        self.es2015_arrow_functions.as_mut().map(|t| t.transform_statements(stmts));
    }

    fn visit_statement(&mut self, stmt: &mut Statement<'a>) {
        self.typescript.as_mut().map(|t| t.transform_statement(stmt));
        self.decorators.as_mut().map(|t| t.transform_statement(stmt));
        walk_statement_mut(self, stmt);
    }

    fn visit_declaration(&mut self, decl: &mut Declaration<'a>) {
        walk_declaration_mut(self, decl);
        self.typescript.as_mut().map(|t| t.transform_declaration(decl));
        self.decorators.as_mut().map(|t| t.transform_declaration(decl));
    }

    fn visit_expression(&mut self, expr: &mut Expression<'a>) {
        // self.typescript.as_mut().map(|t| t.transform_expression(expr));
        self.react_jsx.as_mut().map(|t| t.transform_expression(expr));
        self.regexp_flags.as_mut().map(|t| t.transform_expression(expr));

        self.es2021_logical_assignment_operators.as_mut().map(|t| t.transform_expression(expr));
        self.es2020_nullish_coalescing_operators.as_mut().map(|t| t.transform_expression(expr));
        self.es2015_arrow_functions.as_mut().map(|t| t.transform_expression(expr));
        self.es2015_instanceof.as_mut().map(|t| t.transform_expression(expr));
        self.es2016_exponentiation_operator.as_mut().map(|t| t.transform_expression(expr));
        self.es2015_template_literals.as_mut().map(|t| t.transform_expression(expr));
        self.es2015_new_target.as_mut().map(|t| t.transform_expression(expr));

        walk_expression_mut(self, expr);
    }

    fn visit_catch_clause(&mut self, clause: &mut CatchClause<'a>) {
        self.es2019_optional_catch_binding.as_mut().map(|t| t.transform_catch_clause(clause));
        walk_catch_clause_mut(self, clause);
    }

    fn visit_object_expression(&mut self, expr: &mut ObjectExpression<'a>) {
        self.es2015_function_name.as_mut().map(|t| t.transform_object_expression(expr));
        self.es2015_duplicate_keys.as_mut().map(|t| t.transform_object_expression(expr));
        walk_object_expression_mut(self, expr);
    }

    fn visit_object_property(&mut self, prop: &mut ObjectProperty<'a>) {
        self.es2015_new_target.as_mut().map(|t| t.enter_object_property(prop));
        self.es2015_shorthand_properties.as_mut().map(|t| t.transform_object_property(prop));
        self.es3_property_literal.as_mut().map(|t| t.transform_object_property(prop));

        walk_object_property_mut(self, prop);

        self.es2015_new_target.as_mut().map(|t| t.leave_object_property(prop));
    }

    fn visit_class_body(&mut self, body: &mut ClassBody<'a>) {
        self.es2022_class_static_block.as_mut().map(|t| t.transform_class_body(body));

        walk_class_body_mut(self, body);
    }

    fn visit_variable_declarator(&mut self, declarator: &mut VariableDeclarator<'a>) {
        self.es2015_function_name.as_mut().map(|t| t.transform_variable_declarator(declarator));
        walk_variable_declarator_mut(self, declarator);
    }

    fn visit_directive(&mut self, directive: &mut Directive<'a>) {
        self.es2019_json_strings.as_mut().map(|t| t.transform_directive(directive));
    }

    fn visit_number_literal(&mut self, lit: &mut NumericLiteral<'a>) {
        self.es2021_numeric_separator.as_mut().map(|t| t.transform_number_literal(lit));
        self.es2015_literals.as_mut().map(|t| t.transform_number_literal(lit));
    }

    fn visit_bigint_literal(&mut self, lit: &mut BigIntLiteral<'a>) {
        self.es2021_numeric_separator.as_mut().map(|t| t.transform_bigint_literal(lit));
    }

    fn visit_string_literal(&mut self, lit: &mut StringLiteral<'a>) {
        self.es2019_json_strings.as_mut().map(|t| t.transform_string_literal(lit));
        self.es2015_literals.as_mut().map(|t| t.transform_string_literal(lit));
    }

    fn visit_method_definition(&mut self, def: &mut MethodDefinition<'a>) {
        self.es2015_new_target.as_mut().map(|t| t.enter_method_definition(def));
        self.typescript.as_mut().map(|t| t.transform_method_definition(def));

        walk_method_definition_mut(self, def);

        self.es2015_new_target.as_mut().map(|t| t.leave_method_definition(def));
    }

    fn visit_function(&mut self, func: &mut Function<'a>, flags: Option<ScopeFlags>) {
        self.es2015_new_target.as_mut().map(|t| t.enter_function(func));
        walk_function_mut(self, func, flags);
        self.es2015_new_target.as_mut().map(|t| t.leave_function(func));
    }
}
