use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_traverse::Traverse;

use crate::{context::TraverseCtx, state::TransformState};

mod export_namespace_from;
mod nullish_coalescing_operator;
mod optional_chaining;
mod options;
use export_namespace_from::ExportNamespaceFrom;
use nullish_coalescing_operator::NullishCoalescingOperator;
pub use optional_chaining::OptionalChaining;
pub use options::ES2020Options;

pub struct ES2020<'a> {
    options: ES2020Options,

    // Plugins
    export_namespace_from: ExportNamespaceFrom,
    nullish_coalescing_operator: NullishCoalescingOperator,
    optional_chaining: OptionalChaining<'a>,
}

impl ES2020<'_> {
    pub fn new(options: ES2020Options) -> Self {
        Self {
            options,
            export_namespace_from: ExportNamespaceFrom::new(),
            nullish_coalescing_operator: NullishCoalescingOperator::new(),
            optional_chaining: OptionalChaining::new(),
        }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for ES2020<'a> {
    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.export_namespace_from {
            self.export_namespace_from.exit_program(program, ctx);
        }
    }

    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.nullish_coalescing_operator {
            self.nullish_coalescing_operator.enter_expression(expr, ctx);
        }

        if self.options.optional_chaining {
            self.optional_chaining.enter_expression(expr, ctx);
        }
    }

    fn enter_formal_parameters(
        &mut self,
        node: &mut FormalParameters<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.optional_chaining {
            self.optional_chaining.enter_formal_parameters(node, ctx);
        }
    }

    fn exit_formal_parameters(
        &mut self,
        node: &mut FormalParameters<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.optional_chaining {
            self.optional_chaining.exit_formal_parameters(node, ctx);
        }
    }

    fn enter_big_int_literal(&mut self, node: &mut BigIntLiteral<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.big_int {
            let warning = OxcDiagnostic::warn(
                "Big integer literals are not available in the configured target environment.",
            )
            .with_label(node.span);
            ctx.state.error(warning);
        }
    }

    fn enter_import_specifier(
        &mut self,
        node: &mut ImportSpecifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.arbitrary_module_namespace_names
            && let ModuleExportName::StringLiteral(literal) = &node.imported
        {
            let warning = OxcDiagnostic::warn(
                "Arbitrary module namespace identifier names are not available in the configured target environment.",
            )
            .with_label(literal.span);
            ctx.state.error(warning);
        }
    }

    fn enter_export_specifier(
        &mut self,
        node: &mut ExportSpecifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.arbitrary_module_namespace_names {
            if let ModuleExportName::StringLiteral(literal) = &node.exported {
                let warning = OxcDiagnostic::warn(
                    "Arbitrary module namespace identifier names are not available in the configured target environment.",
                )
                .with_label(literal.span);
                ctx.state.error(warning);
            }
            if let ModuleExportName::StringLiteral(literal) = &node.local {
                let warning = OxcDiagnostic::warn(
                    "Arbitrary module namespace identifier names are not available in the configured target environment.",
                )
                .with_label(literal.span);
                ctx.state.error(warning);
            }
        }
    }

    fn enter_export_all_declaration(
        &mut self,
        node: &mut ExportAllDeclaration<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.arbitrary_module_namespace_names
            && let Some(ModuleExportName::StringLiteral(literal)) = &node.exported
        {
            let warning = OxcDiagnostic::warn(
                "Arbitrary module namespace identifier names are not available in the configured target environment.",
            )
            .with_label(literal.span);
            ctx.state.error(warning);
        }
    }
}
