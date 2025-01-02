use rustc_hash::FxHashSet;

use oxc_allocator::Vec as ArenaVec;
use oxc_ast::ast::*;
use oxc_syntax::{es_target::ESTarget, symbol::SymbolId};
use oxc_traverse::{traverse_mut_with_ctx, ReusableTraverseCtx, Traverse, TraverseCtx};

use crate::{CompressOptions, CompressorPass};

/// Remove Unused Code
///
/// <https://github.com/google/closure-compiler/blob/v20240609/src/com/google/javascript/jscomp/RemoveUnusedCode.java>
pub struct RemoveUnusedCode {
    options: CompressOptions,

    pub(crate) changed: bool,

    symbol_ids_to_remove: FxHashSet<SymbolId>,
}

impl<'a> CompressorPass<'a> for RemoveUnusedCode {
    fn build(&mut self, program: &mut Program<'a>, ctx: &mut ReusableTraverseCtx<'a>) {
        self.changed = false;
        traverse_mut_with_ctx(self, program, ctx);
    }
}

impl<'a> Traverse<'a> for RemoveUnusedCode {
    fn enter_program(&mut self, _node: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        let symbols = ctx.symbols();
        for symbol_id in symbols.symbol_ids() {
            if symbols.get_resolved_references(symbol_id).count() == 0 {
                self.symbol_ids_to_remove.insert(symbol_id);
            }
        }
    }

    fn exit_statements(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        if self.changed {
            stmts.retain(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));
        }
    }

    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Statement::VariableDeclaration(decl) = stmt {
            decl.declarations.retain(|d| {
                if let BindingPatternKind::BindingIdentifier(ident) = &d.id.kind {
                    if d.init.is_none() && self.symbol_ids_to_remove.contains(&ident.symbol_id()) {
                        return false;
                    }
                }
                true
            });
            if decl.declarations.is_empty() {
                self.changed = true;
                *stmt = ctx.ast.statement_empty(decl.span);
            }
        }
    }

    fn exit_catch_clause(&mut self, catch: &mut CatchClause<'a>, _ctx: &mut TraverseCtx<'a>) {
        self.compress_catch_clause(catch);
    }
}

impl RemoveUnusedCode {
    pub fn new(options: CompressOptions) -> Self {
        Self { options, changed: false, symbol_ids_to_remove: FxHashSet::default() }
    }

    fn compress_catch_clause(&mut self, catch: &mut CatchClause<'_>) {
        if self.options.target >= ESTarget::ES2019 {
            if let Some(param) = &catch.param {
                if let BindingPatternKind::BindingIdentifier(ident) = &param.pattern.kind {
                    if self.symbol_ids_to_remove.contains(&ident.symbol_id()) {
                        catch.param = None;
                        self.changed = true;
                    }
                }
            };
        }
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use crate::{tester, CompressOptions};

    fn test(source_text: &str, expected: &str) {
        let allocator = Allocator::default();
        let options = CompressOptions::default();
        let mut pass = super::RemoveUnusedCode::new(options);
        tester::test(&allocator, source_text, expected, &mut pass);
    }

    fn test_same(source_text: &str) {
        test(source_text, source_text);
    }

    #[test]
    fn optional_catch_binding() {
        test("try {} catch(e) {}", "try {} catch {}");
        test_same("try {} catch([e]) {}");
        test_same("try {} catch({e}) {}");

        let allocator = Allocator::default();
        let options = CompressOptions {
            target: oxc_syntax::es_target::ESTarget::ES2018,
            ..CompressOptions::default()
        };
        let mut pass = super::RemoveUnusedCode::new(options);
        let code = "try {} catch(e) {}";
        tester::test(&allocator, code, code, &mut pass);
    }

    #[test]
    fn simple() {
        test("var x", "");
        test_same("var x = 1");
    }
}
