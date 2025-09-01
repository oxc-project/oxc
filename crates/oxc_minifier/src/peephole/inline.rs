use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::{ConstantEvaluation, ConstantValue, IsLiteralValue};
use oxc_semantic::{ScopeFlags, ScopeId, SymbolId};
use oxc_span::GetSpan;
use oxc_traverse::Ancestor;
use rustc_hash::FxHashSet;

use crate::{
    ctx::Ctx,
    symbol_value::{SymbolInformation, SymbolValue},
};

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    pub fn init_symbol_information_map(program: &Program<'a>, ctx: &mut Ctx<'a, '_>) {
        let exported_values = if ctx.source_type().is_script() {
            FxHashSet::default()
        } else {
            program
                .body
                .iter()
                .flat_map(|stmt| -> Vec<SymbolId> {
                    match stmt {
                        Statement::ExportDefaultDeclaration(decl) => match &decl.declaration {
                            ExportDefaultDeclarationKind::FunctionDeclaration(func) => {
                                func.id.iter().map(BindingIdentifier::symbol_id).collect()
                            }
                            ExportDefaultDeclarationKind::ClassDeclaration(class) => {
                                class.id.iter().map(BindingIdentifier::symbol_id).collect()
                            }
                            _ => vec![],
                        },
                        Statement::ExportNamedDeclaration(decl) if decl.source.is_none() => {
                            if let Some(declaration) = &decl.declaration {
                                return match declaration {
                                    Declaration::ClassDeclaration(class) => {
                                        class.id.iter().map(BindingIdentifier::symbol_id).collect()
                                    }
                                    Declaration::FunctionDeclaration(func) => {
                                        func.id.iter().map(BindingIdentifier::symbol_id).collect()
                                    }
                                    Declaration::VariableDeclaration(var) => var
                                        .declarations
                                        .iter()
                                        .flat_map(|d| {
                                            d.id.get_binding_identifiers()
                                                .into_iter()
                                                .map(BindingIdentifier::symbol_id)
                                        })
                                        .collect(),
                                    _ => vec![],
                                };
                            }
                            decl.specifiers
                                .iter()
                                .filter_map(|spec| {
                                    if let ModuleExportName::IdentifierReference(id) = &spec.local {
                                        return ctx
                                            .scoping()
                                            .get_reference(id.reference_id())
                                            .symbol_id();
                                    }
                                    None
                                })
                                .collect()
                        }
                        _ => vec![],
                    }
                })
                .collect()
        };

        let symbol_ids = ctx.scoping().symbol_ids().collect::<Vec<_>>();
        for symbol_id in symbol_ids {
            let mut read_references_count = 0;
            let mut write_references_count = 0;
            for r in ctx.scoping().get_resolved_references(symbol_id) {
                if r.is_read() {
                    read_references_count += 1;
                }
                if r.is_write() {
                    write_references_count += 1;
                }
            }
            let value = SymbolInformation {
                value: SymbolValue::default(),
                exported: exported_values.contains(&symbol_id),
                read_references_count,
                write_references_count,
                scope_id: None,
            };
            ctx.state.symbol_values.init_value(symbol_id, value);
        }
    }

    pub fn init_symbol_value(decl: &VariableDeclarator<'a>, ctx: &mut Ctx<'a, '_>) {
        let BindingPatternKind::BindingIdentifier(ident) = &decl.id.kind else { return };
        let symbol_id = ident.symbol_id();
        // Set None for for statement initializers as the value of these are set by the for statement.
        if Self::is_for_statement_init(ctx) {
            return;
        }

        let init = decl.init.as_ref();
        let symbol_value = if let Some(value) =
            init.map_or(Some(ConstantValue::Undefined), |e| e.evaluate_value(ctx))
        {
            if decl.kind.is_var() {
                SymbolValue::ScopedPrimitive(value)
            } else {
                SymbolValue::Primitive(value)
            }
        } else if init.is_some_and(|init| init.is_literal_value(true, ctx)) {
            SymbolValue::ScopedLiteral
        } else {
            return;
        };
        let current_scope_id = ctx.current_scope_id();
        ctx.state.symbol_values.set_value(symbol_id, symbol_value, current_scope_id);
    }

    fn is_for_statement_init(ctx: &Ctx<'a, '_>) -> bool {
        ctx.ancestors().nth(1).is_some_and(Ancestor::is_parent_of_for_statement_left)
    }

    pub fn inline_identifier_reference(expr: &mut Expression<'a>, ctx: &mut Ctx<'a, '_>) {
        let Expression::Identifier(ident) = expr else { return };
        let reference_id = ident.reference_id();
        let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id() else { return };
        let Some(symbol_value) = ctx.state.symbol_values.get_symbol_value(symbol_id) else {
            return;
        };
        // Skip if there are write references.
        if symbol_value.write_references_count > 0 {
            return;
        }
        match &symbol_value.value {
            SymbolValue::Primitive(cv) => {
                if symbol_value.read_references_count == 1
                    || Self::can_inline_constant_multiple_times(cv)
                {
                    *expr = ctx.value_to_expr(expr.span(), cv.clone());
                    ctx.state.changed = true;
                }
            }
            SymbolValue::ScopedPrimitive(cv) => {
                if (symbol_value.read_references_count == 1
                    || Self::can_inline_constant_multiple_times(cv))
                    && symbol_value.scope_id.is_some_and(|declared_scope_id| {
                        Self::is_referenced_in_same_hoist_scope(declared_scope_id, ctx)
                    })
                {
                    *expr = ctx.value_to_expr(expr.span(), cv.clone());
                    ctx.state.changed = true;
                }
            }
            SymbolValue::ScopedLiteral => {
                if symbol_value.read_references_count == 1
                    && symbol_value.scope_id.is_some_and(|declared_scope_id| {
                        Self::is_referenced_in_same_non_control_scope(declared_scope_id, ctx)
                    })
                {
                    ctx.state.symbol_values.mark_symbol_inlineable(symbol_id);
                }
            }
            SymbolValue::Unknown => {}
        }
    }

    fn can_inline_constant_multiple_times(cv: &ConstantValue<'_>) -> bool {
        match cv {
            ConstantValue::Number(n) => n.fract() == 0.0 && *n >= -99.0 && *n <= 999.0,
            ConstantValue::BigInt(_) => false,
            ConstantValue::String(s) => s.len() <= 3,
            ConstantValue::Boolean(_) | ConstantValue::Undefined | ConstantValue::Null => true,
        }
    }

    fn is_referenced_in_same_hoist_scope(declared_scope_id: ScopeId, ctx: &Ctx<'a, '_>) -> bool {
        ctx.scoping()
            .scope_ancestors(ctx.current_scope_id())
            .find_map(|scope_id| {
                if declared_scope_id == scope_id {
                    return Some(true);
                }
                if ctx.scoping().scope_flags(scope_id).contains(ScopeFlags::Var) {
                    return Some(false);
                }
                None
            })
            .unwrap_or_default()
    }

    fn is_referenced_in_same_non_control_scope(declared_scope_id: ScopeId, ctx: &Ctx<'a, '_>) -> bool {
        #[expect(clippy::unnecessary_find_map)] // TODO
        ctx.scoping()
            .scope_ancestors(ctx.current_scope_id())
            .find_map(|scope_id| {
                if declared_scope_id == scope_id {
                    return Some(true);
                }
                // TODO: allow non-control scope
                Some(false)
            })
            .unwrap_or_default()
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
            r#"console.log('<p autocapitalize="words" contenteditable="false"/>');"#,
            &options,
        );
    }
}
