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

use std::{cell::RefCell, rc::Rc, sync::Arc};

use es2015::TemplateLiterals;
use oxc_allocator::Allocator;
use oxc_ast::{ast::*, AstBuilder, AstKind, VisitMut};
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
    es2021::LogicalAssignmentOperators,
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
            Rc::clone(&ast),
            Rc::new(RefCell::new(semantic)),
        );

        Self {
            ctx: ctx.clone(),
            decorators: Decorators::new(Rc::clone(&ast), ctx.clone(), &options),
            // TODO: pass verbatim_module_syntax from user config
            typescript: source_type.is_typescript().then(|| TypeScript::new(Rc::clone(&ast), ctx.clone(), false, &options)),
            regexp_flags: RegexpFlags::new(Rc::clone(&ast), &options),
            // es2022
            es2022_class_static_block: es2022::ClassStaticBlock::new(Rc::clone(&ast), &options),
            // es2021
            es2021_logical_assignment_operators: LogicalAssignmentOperators::new(Rc::clone(&ast), ctx.clone(), &options),
            // es2020
            es2020_nullish_coalescing_operators: NullishCoalescingOperator::new(Rc::clone(&ast), ctx.clone(), &options),
            // es2019
            es2019_json_strings: JsonStrings::new(Rc::clone(&ast), &options),
            es2019_optional_catch_binding: OptionalCatchBinding::new(Rc::clone(&ast), &options),
            // es2016
            es2016_exponentiation_operator: ExponentiationOperator::new(Rc::clone(&ast), ctx.clone(), &options),
            // es2015
            es2015_function_name: FunctionName::new(Rc::clone(&ast), ctx.clone(), &options),
            es2015_arrow_functions: ArrowFunctions::new(Rc::clone(&ast), ctx.clone(), &options),
            es2015_shorthand_properties: ShorthandProperties::new(Rc::clone(&ast), &options),
            es2015_template_literals: TemplateLiterals::new(Rc::clone(&ast), &options),
            es2015_duplicate_keys: DuplicateKeys::new(Rc::clone(&ast), &options),
            es2015_instanceof: Instanceof::new(Rc::clone(&ast), ctx.clone(), &options),
            es2015_new_target: NewTarget::new(Rc::clone(&ast),ctx.clone(), &options),
            // other
            es3_property_literal: PropertyLiteral::new(Rc::clone(&ast), &options),
            react_jsx: ReactJsx::new(Rc::clone(&ast), ctx.clone(), options)
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
    fn enter_node(&mut self, kind: oxc_ast::AstKind<'a>) {
        self.es2015_new_target.as_mut().map(|t| t.enter_node(kind));
    }

    fn leave_node(&mut self, kind: oxc_ast::AstKind<'a>) {
        self.es2015_new_target.as_mut().map(|t| t.leave_node(kind));
    }

    fn visit_program(&mut self, program: &mut Program<'a>) {
        let kind = AstKind::Program(self.alloc(program));
        self.enter_scope({
            let mut flags = ScopeFlags::Top;
            if program.is_strict() {
                flags |= ScopeFlags::StrictMode;
            }
            flags
        });
        self.enter_node(kind);

        for directive in program.directives.iter_mut() {
            self.visit_directive(directive);
        }

        self.typescript.as_mut().map(|t| t.transform_program(program));
        self.visit_statements(&mut program.body);

        self.react_jsx.as_mut().map(|t| t.add_react_jsx_runtime_imports(program));
        self.decorators.as_mut().map(|t| t.transform_program(program));
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_assignment_expression(&mut self, expr: &mut AssignmentExpression<'a>) {
        let kind = AstKind::AssignmentExpression(self.alloc(expr));
        self.enter_node(kind);

        self.es2015_function_name.as_mut().map(|t| t.transform_assignment_expression(expr));

        self.visit_assignment_target(&mut expr.left);
        self.visit_expression(&mut expr.right);

        self.leave_node(kind);
    }

    fn visit_statements(&mut self, stmts: &mut oxc_allocator::Vec<'a, Statement<'a>>) {
        self.typescript.as_mut().map(|t| t.transform_statements(stmts));

        for stmt in stmts.iter_mut() {
            self.visit_statement(stmt);
        }
        // TODO: we need scope id to insert the vars into the correct statements
        self.es2021_logical_assignment_operators.as_mut().map(|t| t.add_vars_to_statements(stmts));
        self.es2020_nullish_coalescing_operators.as_mut().map(|t| t.add_vars_to_statements(stmts));
        self.es2016_exponentiation_operator.as_mut().map(|t| t.add_vars_to_statements(stmts));
        self.es2015_arrow_functions.as_mut().map(|t| t.transform_statements(stmts));
    }

    fn visit_statement(&mut self, stmt: &mut Statement<'a>) {
        self.typescript.as_mut().map(|t| t.transform_statement(stmt));
        self.decorators.as_mut().map(|t| t.transform_statement(stmt));
        self.visit_statement_match(stmt);
    }

    fn visit_declaration(&mut self, decl: &mut Declaration<'a>) {
        self.visit_declaration_match(decl);
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

        self.visit_expression_match(expr);
    }

    fn visit_catch_clause(&mut self, clause: &mut CatchClause<'a>) {
        let kind = AstKind::CatchClause(self.alloc(clause));
        self.enter_scope(ScopeFlags::empty());
        self.enter_node(kind);

        self.es2019_optional_catch_binding.as_mut().map(|t| t.transform_catch_clause(clause));

        if let Some(param) = &mut clause.param {
            self.visit_binding_pattern(param);
        }
        self.visit_statements(&mut clause.body.body);
        self.leave_node(kind);
        self.leave_scope();
    }

    fn visit_object_expression(&mut self, expr: &mut ObjectExpression<'a>) {
        let kind = AstKind::ObjectExpression(self.alloc(expr));
        self.enter_node(kind);
        self.es2015_function_name.as_mut().map(|t| t.transform_object_expression(expr));
        self.es2015_duplicate_keys.as_mut().map(|t| t.transform_object_expression(expr));

        for property in expr.properties.iter_mut() {
            self.visit_object_property_kind(property);
        }
        self.leave_node(kind);
    }

    fn visit_object_property(&mut self, prop: &mut ObjectProperty<'a>) {
        let kind = AstKind::ObjectProperty(self.alloc(prop));
        self.enter_node(kind);

        self.es2015_shorthand_properties.as_mut().map(|t| t.transform_object_property(prop));
        self.es3_property_literal.as_mut().map(|t| t.transform_object_property(prop));

        self.visit_property_key(&mut prop.key);
        self.visit_expression(&mut prop.value);
        if let Some(init) = &mut prop.init {
            self.visit_expression(init);
        }
        self.leave_node(kind);
    }

    fn visit_class_body(&mut self, class_body: &mut ClassBody<'a>) {
        self.es2022_class_static_block.as_mut().map(|t| t.transform_class_body(class_body));

        class_body.body.iter_mut().for_each(|class_element| {
            self.visit_class_element(class_element);
        });
    }

    fn visit_variable_declarator(&mut self, declarator: &mut VariableDeclarator<'a>) {
        let kind = AstKind::VariableDeclarator(self.alloc(declarator));
        self.enter_node(kind);

        self.es2015_function_name.as_mut().map(|t| t.transform_variable_declarator(declarator));

        self.visit_binding_pattern(&mut declarator.id);

        if let Some(init) = &mut declarator.init {
            self.visit_expression(init);
        }

        self.leave_node(kind);
    }

    fn visit_directive(&mut self, directive: &mut Directive<'a>) {
        self.es2019_json_strings
            .as_mut()
            .map(|t: &mut JsonStrings| t.transform_directive(directive));
    }

    fn visit_string_literal(&mut self, lit: &mut StringLiteral) {
        self.es2019_json_strings
            .as_mut()
            .map(|t: &mut JsonStrings| t.transform_string_literal(lit));
    }
}
