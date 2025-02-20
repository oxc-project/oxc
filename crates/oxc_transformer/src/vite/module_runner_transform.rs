/// ### Complicated parts if we want to support this in main transformer:
/// 1. In Vite, it will collect import deps and dynamic import deps during the transform process, and return them
/// at the end of function. We can do this, but how to return it?
/// 2. In case other plugins will insert imports/exports, we must transform them in exit_program, but it will pose
/// another problem: how to transform identifiers which refers to imports? We must collect some information from imports,
/// but it already at the end of the visitor. To solve this, we may introduce a new visitor to transform identifier,
/// dynamic import and meta property.
use oxc_allocator::Vec as ArenaVec;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

pub struct ModuleRunnerTransform {
    import_uid: u32,
}

impl ModuleRunnerTransform {
    pub fn new() -> Self {
        Self { import_uid: 0 }
    }
}

impl<'a> Traverse<'a> for ModuleRunnerTransform {
    #[inline]
    fn enter_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.transform_imports_and_exports(program, ctx);
    }

    #[inline]
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        match expr {
            Expression::Identifier(_) => self.transform_identifier(expr, ctx),
            Expression::MetaProperty(_) => self.transform_meta_property(expr, ctx),
            Expression::ImportExpression(_) => self.transform_import_expression(expr, ctx),
            _ => {}
        }
    }
}

impl<'a> ModuleRunnerTransform {
    #[inline]
    fn transform_imports_and_exports(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        for statement in &mut program.body {
            match statement {
                Statement::ImportDeclaration(import) => {
                    self.define_import(import, ctx);
                }
                Statement::ExportAllDeclaration(export) => {}
                Statement::ExportNamedDeclaration(export) => {}
                Statement::ExportDefaultDeclaration(export) => {}
                _ => {}
            }
        }
    }

    #[inline]
    fn transform_import_expression(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::ImportExpression(import_expr) = expr else {
            unreachable!();
        };
    }

    #[inline]
    fn transform_identifier(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::Identifier(import_expr) = expr else {
            unreachable!();
        };
    }

    #[inline]
    fn transform_meta_property(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::MetaProperty(import_expr) = expr else {
            unreachable!();
        };
    }

    fn define_import(
        &mut self,
        source: &str,
        specifiers: Option<&mut ArenaVec<'a, ImportSpecifier<'a>>>,
        ctx: &mut TraverseCtx<'a>,
    ) {


        `__vite_ssr_import_${uid++}__`
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use oxc_allocator::Allocator;
    use oxc_codegen::{CodeGenerator, CodegenOptions};
    use oxc_diagnostics::OxcDiagnostic;
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::SourceType;
    use oxc_traverse::traverse_mut;

    use super::ModuleRunnerTransform;

    fn test(source_text: &str) -> Result<String, Vec<OxcDiagnostic>> {
        let source_type = SourceType::default();
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        let mut program = ret.program;
        let (symbols, scopes) =
            SemanticBuilder::new().build(&program).semantic.into_symbol_table_and_scope_tree();

        let mut module_runner_transform = ModuleRunnerTransform::new();
        traverse_mut(&mut module_runner_transform, &allocator, &mut program, symbols, scopes);

        if !ret.errors.is_empty() {
            return Err(ret.errors);
        }
        let code = CodeGenerator::new()
            .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
            .build(&program)
            .code;
        Ok(code)
    }

    fn test_same(source_text: &str, expected: &str) {
        debug_assert_eq!(test(source_text).ok(), Some(expected.to_string()));
    }

    #[test]
    fn default_import() {
        test_same("import { foo } from 'vue';console.log(foo.bar)",  "const __vite_ssr_import_0__ = await __vite_ssr_import__(\"vue\", {\"importedNames\":[\"foo\"]});console.log(__vite_ssr_import_0__.foo.bar)");
    }
}
