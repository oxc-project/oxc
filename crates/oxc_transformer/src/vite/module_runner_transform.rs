/// ### Complicated parts if we want to support this in main transformer:
/// 1. In Vite, it will collect import deps and dynamic import deps during the transform process, and return them
/// at the end of function. We can do this, but how to return it?
/// 2. In case other plugins will insert imports/exports, we must transform them in exit_program, but it will pose
/// another problem: how to transform identifiers which refers to imports? We must collect some information from imports,
/// but it already at the end of the visitor. To solve this, we may introduce a new visitor to transform identifier,
/// dynamic import and meta property.
use compact_str::ToCompactString;
use rustc_hash::FxHashMap;

use oxc_allocator::{String as ArenaString, Vec as ArenaVec};
use oxc_ast::{NONE, ast::*};
use oxc_semantic::{ReferenceFlags, SymbolFlags, SymbolId};
use oxc_span::SPAN;
use oxc_traverse::{BoundIdentifier, Traverse, TraverseCtx};

pub struct ModuleRunnerTransform<'a> {
    import_uid: u32,
    import_bindings: FxHashMap<SymbolId, BoundIdentifier<'a>>,
}

impl ModuleRunnerTransform<'_> {
    pub fn new() -> Self {
        Self { import_uid: 0, import_bindings: FxHashMap::default() }
    }
}

const SSR_MODULE_EXPORTS_KEY: &str = "__vite_ssr_exports__";
const SSR_IMPORT_KEY: &str = "__vite_ssr_import__";
const SSR_DYNAMIC_IMPORT_KEY: &str = "__vite_ssr_dynamic_import__";
const SSR_EXPORT_ALL_KEY: &str = "__vite_ssr_exportAll__";
const SSR_IMPORT_META_KEY: &str = "__vite_ssr_import_meta__";

impl<'a> Traverse<'a> for ModuleRunnerTransform<'a> {
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

impl<'a> ModuleRunnerTransform<'a> {
    #[inline]
    fn transform_imports_and_exports(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        for stmt in &mut program.body {
            if Self::should_transform_statement(stmt) {
                match ctx.ast.move_statement(stmt) {
                    Statement::ImportDeclaration(import) => {
                        let ImportDeclaration { span, source, specifiers, .. } = import.unbox();
                        *stmt = self.transform_import(span, source, specifiers, ctx);
                    }
                    Statement::ExportAllDeclaration(export) => {}
                    Statement::ExportNamedDeclaration(export) => {}
                    Statement::ExportDefaultDeclaration(export) => {}
                    _ => {}
                }
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

    fn transform_import(
        &mut self,
        span: Span,
        source: StringLiteral<'a>,
        specifiers: Option<ArenaVec<'a, ImportDeclarationSpecifier<'a>>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        self.import_uid += 1;
        let uid = self.import_uid.to_compact_string();
        let capacity = 20 + uid.len();
        let mut string = ArenaString::with_capacity_in(capacity, ctx.ast.allocator);
        string.push_str("__vite_ssr_import_");
        string.push_str(&uid);
        string.push_str("__");

        let binding = ctx.generate_binding_in_current_scope(
            Atom::from(string),
            SymbolFlags::BlockScopedVariable,
        );

        // `await __vite_ssr_import__('vue', {importedNames: ['foo']});`
        let callee =
            ctx.create_unbound_ident_expr(SPAN, Atom::from(SSR_IMPORT_KEY), ReferenceFlags::Read);
        let mut arguments = ctx.ast.vec_with_capacity(1 + usize::from(specifiers.is_some()));
        arguments.push(Argument::from(Expression::StringLiteral(ctx.ast.alloc(source))));
        if let Some(specifiers) = specifiers {
            arguments.push(self.transform_import_metadata(&binding, specifiers, ctx));
        }
        let call = ctx.ast.expression_call(SPAN, callee, NONE, arguments, false);
        let init = ctx.ast.expression_await(SPAN, call);

        // `const __vite_ssr_import_1__ = await __vite_ssr_import__('vue', {importedNames: ['foo']});`
        let kind = VariableDeclarationKind::Const;
        let pattern = binding.create_binding_pattern(ctx);
        let declarator = ctx.ast.variable_declarator(SPAN, kind, pattern, Some(init), false);
        let declaration = ctx.ast.declaration_variable(span, kind, ctx.ast.vec1(declarator), false);
        Statement::from(declaration)
    }

    fn transform_import_metadata(
        &mut self,
        binding: &BoundIdentifier<'a>,
        specifiers: ArenaVec<'a, ImportDeclarationSpecifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Argument<'a> {
        let elements =
            ctx.ast.vec_from_iter(specifiers.into_iter().map(|specifier| match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                    self.transform_import_binding(binding, specifier.unbox().local, ctx)
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    self.transform_import_binding(binding, specifier.unbox().local, ctx)
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    self.transform_import_binding(binding, specifier.unbox().local, ctx)
                }
            }));
        let value = ctx.ast.expression_array(SPAN, elements, None);
        let key = ctx.ast.property_key_static_identifier(SPAN, Atom::from("importedNames"));
        let imported_names = ctx.ast.object_property_kind_object_property(
            SPAN,
            PropertyKind::Init,
            key,
            value,
            false,
            false,
            false,
        );
        // { importedNames: ['foo', 'bar'] }
        Argument::from(ctx.ast.expression_object(SPAN, ctx.ast.vec1(imported_names), None))
    }

    fn transform_import_binding(
        &mut self,
        binding: &BoundIdentifier<'a>,
        ident: BindingIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> ArrayExpressionElement<'a> {
        let BindingIdentifier { span, name, symbol_id } = ident;
        let scopes = ctx.scopes_mut();
        scopes.remove_binding(scopes.root_scope_id(), &name);

        let symbol_id = symbol_id.get().unwrap();
        if !ctx.symbols().get_resolved_reference_ids(symbol_id).is_empty() {
            self.import_bindings.insert(symbol_id, binding.clone());
        }

        ArrayExpressionElement::from(ctx.ast.expression_string_literal(span, name, None))
    }

    #[inline]
    fn should_transform_statement(statement: &Statement<'a>) -> bool {
        matches!(
            statement,
            Statement::ImportDeclaration(_)
                | Statement::ExportAllDeclaration(_)
                | Statement::ExportNamedDeclaration(_)
                | Statement::ExportDefaultDeclaration(_)
        )
    }
}

#[cfg(test)]
mod test {
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
        test_same(
            "import { foo } from 'vue';console.log(foo.bar)",
            "const __vite_ssr_import_1__ = await __vite_ssr_import__('vue', { importedNames: ['foo'] });\nconsole.log(foo.bar);\n",
        );
    }
}
