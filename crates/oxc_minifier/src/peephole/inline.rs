use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::{ConstantEvaluation, ConstantValue};
use oxc_span::GetSpan;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

// List of common global identifiers that should not be inlined when shadowed
fn is_global_identifier_name(name: &str) -> bool {
    matches!(
        name,
        "undefined" | "Infinity" | "NaN" | "window" | "global" | "globalThis" | 
        "console" | "JSON" | "Math" | "Date" | "RegExp" | "Array" | "Object" | 
        "String" | "Number" | "Boolean" | "Symbol" | "BigInt" | "Error" |
        "eval" | "isNaN" | "isFinite" | "parseInt" | "parseFloat" | "encodeURI" | 
        "encodeURIComponent" | "decodeURI" | "decodeURIComponent"
    )
}

impl<'a> PeepholeOptimizations {
    pub fn init_symbol_value(&self, decl: &VariableDeclarator<'a>, ctx: &mut Ctx<'a, '_>) {
        let BindingPatternKind::BindingIdentifier(ident) = &decl.id.kind else { return };
        let Some(symbol_id) = ident.symbol_id.get() else { return };
        
        // For var declarations, be conservative about inlining to avoid scope/semantics issues
        if decl.kind.is_var() {
            // Don't inline uninitialized vars (they default to undefined but have scope semantics)
            if decl.init.is_none() {
                return;
            }
            // Don't inline vars that shadow global identifiers (like 'undefined', 'Infinity', etc.)  
            if is_global_identifier_name(&ident.name) {
                return;
            }
        }
        
        let value =
            decl.init.as_ref().map_or(Some(ConstantValue::Undefined), |e| e.evaluate_value(ctx));
        ctx.init_value(symbol_id, value);
    }

    pub fn inline_identifier_reference(&self, expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::Identifier(ident) = expr else { return };
        let Some(reference_id) = ident.reference_id.get() else { return };
        let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id() else { return };
        let Some(symbol_value) = ctx.state.symbol_values.get_symbol_value(symbol_id) else {
            return;
        };
        // Skip if there are write references.
        if symbol_value.write_references_count > 0 {
            return;
        }
        if symbol_value.for_statement_init {
            return;
        }
        let Some(cv) = &symbol_value.initialized_constant else { return };
        if symbol_value.read_references_count == 1
            || match cv {
                ConstantValue::Number(n) => n.fract() == 0.0 && *n >= -99.0 && *n <= 999.0,
                ConstantValue::BigInt(_) => false,
                ConstantValue::String(s) => s.len() <= 3,
                ConstantValue::Boolean(_) | ConstantValue::Undefined | ConstantValue::Null => true,
            }
        {
            *expr = ctx.value_to_expr(expr.span(), cv.clone());
            ctx.state.changed = true;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        CompressOptions,
        tester::{test_options, test_same_options},
    };

    #[test]
    fn r#const() {
        let options = CompressOptions::smallest();
        test_options("const foo = 1; log(foo)", "log(1)", &options);
        test_options("export const foo = 1; log(foo)", "export const foo = 1; log(1)", &options);

        test_options("let foo = 1; log(foo)", "log(1)", &options);
        test_options("export let foo = 1; log(foo)", "export let foo = 1; log(1)", &options);
    }

    #[test]
    fn var_constants() {
        let options = CompressOptions::smallest();
        
        // Basic var constant inlining - declarations should be removed when fully inlined
        test_options("var foo = 1; log(foo)", "log(1)", &options);
        test_options("var foo = 'hello'; log(foo)", "log('hello')", &options);
        test_options("var foo = true; log(foo)", "log(!0)", &options);
        test_options("var foo = false; log(foo)", "log(!1)", &options);
        test_options("var foo = null; log(foo)", "log(null)", &options);
        test_options("var foo = undefined; log(foo)", "log(void 0)", &options);
        
        // Multiple references should inline (output format is comma-separated)
        test_options("var foo = 1; log(foo), log(foo)", "log(1), log(1)", &options);
        
        // Multiple var declarations with inlining
        test_options("var x = 1, y = 'hello', z = true; console.log(x, y, z)", "console.log(1, 'hello', !0)", &options);
        
        // Exported vars should keep declaration but inline usage
        test_options("export var foo = 1; log(foo)", "export var foo = 1; log(1)", &options);
        
        // Var with reassignment should not inline
        test_options("var foo = 1; foo = 2; log(foo)", "var foo = 1; foo = 2, log(foo)", &options);
        test_options("var foo = 1; foo++; log(foo)", "var foo = 1; foo++, log(foo)", &options);
        test_options("var foo = 1; ++foo; log(foo)", "var foo = 1; ++foo, log(foo)", &options);
        
        // Complex expressions should not inline if they're not constants
        test_same_options("var foo = Math.random(); log(foo)", &options);
        test_same_options("var foo = bar(); log(foo)", &options);
    }

    #[test]
    fn var_hoisting_edge_cases() {
        let options = CompressOptions::smallest();
        
        // Reference before declaration should not affect inlining after declaration
        // var declaration is kept due to hoisting effect on typeof check
        test_options("console.log(typeof x); var x = 1; console.log(x)", "console.log(typeof x); var x = 1; console.log(1)", &options);
    }

    #[test]
    fn small_value() {
        let options = CompressOptions::smallest();
        test_options("const foo = 999; log(foo), log(foo)", "log(999), log(999)", &options);
        test_options("const foo = -99; log(foo), log(foo)", "log(-99), log(-99)", &options);
        test_same_options("const foo = 1000; log(foo), log(foo)", &options);
        test_same_options("const foo = -100; log(foo), log(foo)", &options);

        test_same_options("const foo = 0n; log(foo), log(foo)", &options);

        test_options("const foo = 'aaa'; log(foo), log(foo)", "log('aaa'), log('aaa')", &options);
        test_same_options("const foo = 'aaaa'; log(foo), log(foo)", &options);

        test_options("const foo = true; log(foo), log(foo)", "log(!0), log(!0)", &options);
        test_options("const foo = false; log(foo), log(foo)", "log(!1), log(!1)", &options);
        test_options(
            "const foo = undefined; log(foo), log(foo)",
            "log(void 0), log(void 0)",
            &options,
        );
        test_options("const foo = null; log(foo), log(foo)", "log(null), log(null)", &options);

        test_options(
            r#"
            const o = 'o';
            const d = 'd';
            const boolean = false;
            var frag = `<p autocapitalize="${`w${o}r${d}s`}" contenteditable="${boolean}"/>`;
            console.log(frag);
            "#,
            r#"console.log('<p autocapitalize="words" contenteditable="false"/>')"#,
            &options,
        );
    }
}
