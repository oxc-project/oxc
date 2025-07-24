use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::ConstantEvaluation;
use oxc_span::GetSpan;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    pub fn init_symbol_value(&self, decl: &VariableDeclarator<'a>, ctx: &mut Ctx<'a, '_>) {
        let BindingPatternKind::BindingIdentifier(ident) = &decl.id.kind else { return };
        let Some(symbol_id) = ident.symbol_id.get() else { return };
        // Skip if not `const` variable.
        if !ctx.scoping().symbol_flags(symbol_id).is_const_variable() {
            return;
        }
        let Some(value) = decl.init.evaluate_value(ctx) else { return };
        ctx.init_value(symbol_id, value);
    }

    pub fn inline_identifier_reference(&self, expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::Identifier(ident) = expr else { return };
        let Some(reference_id) = ident.reference_id.get() else { return };
        let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id() else { return };
        let Some(symbol_value) = ctx.state.symbol_values.get_symbol_value(symbol_id) else {
            return;
        };
        // Only inline single reference (for now).
        if symbol_value.read_references_count > 1 {
            return;
        }
        // Skip if there are write references.
        if symbol_value.write_references_count > 0 {
            return;
        }
        *expr = ctx.value_to_expr(expr.span(), symbol_value.constant.clone());
        ctx.state.changed = true;
    }
}

#[cfg(test)]
mod test {
    use crate::{
        CompressOptions,
        tester::{test_options, test_same},
    };

    #[test]
    fn r#const() {
        let options = CompressOptions::smallest();
        test_options("const foo = 1; log(foo)", "log(1)", &options);
        test_options("export const foo = 1; log(foo)", "export const foo = 1; log(1)", &options);
        test_same("const foo = 1; log(foo), log(foo)");
    }
}
