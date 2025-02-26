/// Module runner transform
///
/// ## Implementation
///
/// Based on https://github.com/vitejs/vite/blob/00deea4ff88e30e299cb40a801b5dc0205ac913d/packages/vite/src/node/ssr/ssrTransform.ts
///
///
/// ## Complicated parts if we want to support this in main transformer:
/// 1. In Vite, it will collect import deps and dynamic import deps during the transform process, and return them
/// at the end of function. We can do this, but how to return it?
/// 2. In case other plugins will insert imports/exports, we must transform them in exit_program, but it will pose
/// another problem: how to transform identifiers which refers to imports? We must collect some information from imports,
/// but it already at the end of the visitor. To solve this, we may introduce a new visitor to transform identifier,
/// dynamic import and meta property.
use compact_str::ToCompactString;
use oxc_syntax::identifier::is_identifier_name;
use rustc_hash::FxHashMap;

use oxc_allocator::{String as ArenaString, Vec as ArenaVec};
use oxc_ast::{NONE, ast::*};
use oxc_semantic::{ReferenceFlags, SymbolFlags, SymbolId};
use oxc_span::SPAN;
use oxc_traverse::{BoundIdentifier, Traverse, TraverseCtx};

use crate::utils::ast_builder::{create_compute_property_access, create_property_access};

pub struct ModuleRunnerTransform<'a> {
    import_uid: u32,
    import_bindings: FxHashMap<SymbolId, (BoundIdentifier<'a>, Option<Atom<'a>>)>,
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
    fn transform_identifier(&self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::Identifier(ident) = expr else {
            unreachable!();
        };

        let Some(reference_id) = ident.reference_id.get() else {
            return;
        };
        let Some(symbol_id) = ctx.symbols().get_reference(reference_id).symbol_id() else {
            return;
        };
        if let Some((binding, property)) = self.import_bindings.get(&symbol_id) {
            let object = binding.create_read_expression(ctx);
            if let Some(property) = property {
                // TODO(improvement): It looks like here always return a computed member expression,
                //                    so that we don't need to check if it's a identifier name
                if is_identifier_name(property) {
                    *expr = create_property_access(ident.span, object, property, ctx);
                } else {
                    *expr = create_compute_property_access(ident.span, object, property, ctx);
                }
            } else {
                *expr = object;
            }
        }
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
        let string = Atom::from(string);

        // `await __vite_ssr_import__('vue', {importedNames: ['foo']});`
        let callee =
            ctx.create_unbound_ident_expr(SPAN, Atom::from(SSR_IMPORT_KEY), ReferenceFlags::Read);
        let mut arguments = ctx.ast.vec_with_capacity(1 + usize::from(specifiers.is_some()));
        arguments.push(Argument::from(Expression::StringLiteral(ctx.ast.alloc(source))));

        let pattern = if let Some(mut specifiers) = specifiers {
            // `import * as vue from 'vue';` -> `const __vite_ssr_import_1__ = await __vite_ssr_import__('vue');`
            if matches!(
                specifiers.last(),
                Some(ImportDeclarationSpecifier::ImportNamespaceSpecifier(_))
            ) {
                let Some(ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier)) =
                    specifiers.pop()
                else {
                    unreachable!()
                };

                // Reuse the `vue` binding identifier by renaming it to `__vite_ssr_import_1__`
                let mut local = specifier.unbox().local;
                local.name = string;
                let binding = BoundIdentifier::from_binding_ident(&local);
                ctx.symbols_mut().set_name(binding.symbol_id, &binding.name);
                self.import_bindings.insert(binding.symbol_id, (binding, None));

                let binding_pattern_kind = BindingPatternKind::BindingIdentifier(ctx.alloc(local));
                ctx.ast.binding_pattern(binding_pattern_kind, NONE, false)
            } else {
                let binding =
                    ctx.generate_binding_in_current_scope(string, SymbolFlags::BlockScopedVariable);
                arguments.push(self.transform_import_metadata(&binding, specifiers, ctx));
                binding.create_binding_pattern(ctx)
            }
        } else {
            ctx.generate_binding_in_current_scope(string, SymbolFlags::BlockScopedVariable)
                .create_binding_pattern(ctx)
        };

        let call = ctx.ast.expression_call(SPAN, callee, NONE, arguments, false);
        let init = ctx.ast.expression_await(SPAN, call);

        // `const __vite_ssr_import_1__ = await __vite_ssr_import__('vue', { importedNames: ['foo'] });`
        let kind = VariableDeclarationKind::Const;
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
        let elements_iter =
            ctx.ast.vec_from_iter(specifiers.into_iter().map(|specifier| match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                    let ImportSpecifier { span, local, imported, .. } = specifier.unbox();
                    self.insert_import_binding(span, binding, local, imported.name(), ctx)
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    let ImportDefaultSpecifier { span, local } = specifier.unbox();
                    self.insert_import_binding(span, binding, local, Atom::from("default"), ctx)
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                    unreachable!()
                }
            }));
        let elements = ctx.ast.vec_from_iter(elements_iter);
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

    /// Insert an import binding into the import bindings map and then return an imported name.
    fn insert_import_binding(
        &mut self,
        span: Span,
        binding: &BoundIdentifier<'a>,
        ident: BindingIdentifier<'a>,
        key: Atom<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> ArrayExpressionElement<'a> {
        let BindingIdentifier { name, symbol_id, .. } = ident;

        let scopes = ctx.scopes_mut();
        scopes.remove_binding(scopes.root_scope_id(), &name);

        let symbol_id = symbol_id.get().unwrap();
        if !ctx.symbols().get_resolved_reference_ids(symbol_id).is_empty() {
            self.import_bindings.insert(symbol_id, (binding.clone(), Some(key)));
        }

        ArrayExpressionElement::from(ctx.ast.expression_string_literal(span, key, None))
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
            "import foo from 'vue';console.log(foo.bar)",
            "const __vite_ssr_import_1__ = await __vite_ssr_import__('vue', { importedNames: ['default'] });\nconsole.log(__vite_ssr_import_1__.default.bar);\n",
        );
    }

    #[test]
    fn named_import() {
        test_same(
            "import { ref } from 'vue';function foo() { return ref(0) }",
            "const __vite_ssr_import_1__ = await __vite_ssr_import__('vue', { importedNames: ['ref'] });\nfunction foo() {\n\treturn __vite_ssr_import_1__.ref(0);\n}\n",
        );
    }

    #[test]
    fn named_import_arbitrary_module_namespace() {
        test_same(
            "import { \"some thing\" as ref } from 'vue';function foo() { return ref(0) }",
            "const __vite_ssr_import_1__ = await __vite_ssr_import__('vue', { importedNames: ['some thing'] });\nfunction foo() {\n\treturn __vite_ssr_import_1__['some thing'](0);\n}\n",
        );
    }

    #[test]
    fn namespace_import() {
        test_same(
            "import * as vue from 'vue';function foo() { return vue.ref(0) }",
            "const __vite_ssr_import_1__ = await __vite_ssr_import__('vue');\nfunction foo() {\n\treturn __vite_ssr_import_1__.ref(0);\n}\n",
        );
    }
}
