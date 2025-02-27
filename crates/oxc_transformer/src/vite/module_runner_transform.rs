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
use rustc_hash::FxHashMap;
use std::iter;

use oxc_allocator::{Box as ArenaBox, String as ArenaString, Vec as ArenaVec};
use oxc_ast::{NONE, ast::*};
use oxc_ecmascript::BoundNames;
use oxc_semantic::{ReferenceFlags, ScopeFlags, SymbolFlags, SymbolId};
use oxc_span::SPAN;
use oxc_syntax::identifier::is_identifier_name;
use oxc_traverse::{BoundIdentifier, Traverse, TraverseCtx};

use crate::utils::ast_builder::{
    create_compute_property_access, create_member_callee, create_property_access,
};

pub struct ModuleRunnerTransform<'a> {
    import_uid: u32,
    import_bindings: FxHashMap<SymbolId, (BoundIdentifier<'a>, Option<Atom<'a>>)>,
}

impl ModuleRunnerTransform<'_> {
    pub fn new() -> Self {
        Self { import_uid: 0, import_bindings: FxHashMap::default() }
    }
}

const SSR_MODULE_EXPORTS_KEY: Atom<'static> = Atom::new_const("__vite_ssr_exports__");
const SSR_IMPORT_KEY: Atom<'static> = Atom::new_const("__vite_ssr_import__");
const SSR_DYNAMIC_IMPORT_KEY: Atom<'static> = Atom::new_const("__vite_ssr_dynamic_import__");
const SSR_EXPORT_ALL_KEY: Atom<'static> = Atom::new_const("__vite_ssr_exportAll__");
const SSR_IMPORT_META_KEY: Atom<'static> = Atom::new_const("__vite_ssr_import_meta__");

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
    fn transform_imports_and_exports(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let should_transform = program.body.iter().any(Self::should_transform_statement);
        if !should_transform {
            return;
        }
        // Reserve enough space for new statements
        let mut new_stmts: ArenaVec<'a, Statement<'a>> =
            ctx.ast.vec_with_capacity(program.body.len() * 2);

        let mut hoist_index = None;

        for stmt in program.body.drain(..) {
            match stmt {
                Statement::ImportDeclaration(import) => {
                    let ImportDeclaration { span, source, specifiers, .. } = import.unbox();
                    let import_statement = self.transform_import(span, source, specifiers, ctx);
                    // Need to hoist import statements to the above of the other statements
                    if let Some(index) = hoist_index {
                        new_stmts.insert(index, import_statement);
                        hoist_index.replace(index + 1);
                    } else {
                        new_stmts.push(import_statement);
                    }
                    continue;
                }
                Statement::ExportNamedDeclaration(export) => {
                    self.transform_export_named_declaration(&mut new_stmts, export, ctx);
                }
                Statement::ExportAllDeclaration(export) => {
                    self.transform_export_all_declaration(&mut new_stmts, export, ctx);
                }
                Statement::ExportDefaultDeclaration(export) => {
                    self.transform_export_default_declaration(&mut new_stmts, export, ctx);
                }
                _ => {
                    new_stmts.push(stmt);
                }
            }
            if hoist_index.is_none() {
                hoist_index.replace(new_stmts.len() - 1);
            }
        }

        program.body = new_stmts;
    }

    #[inline]
    fn transform_import_expression(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let Expression::ImportExpression(import_expr) = ctx.ast.move_expression(expr) else {
            unreachable!();
        };

        let ImportExpression { span, source, arguments, .. } = import_expr.unbox();
        let flags = ReferenceFlags::Read;
        let callee = ctx.create_unbound_ident_expr(SPAN, SSR_DYNAMIC_IMPORT_KEY, flags);
        let arguments = arguments.into_iter().map(Argument::from);
        let arguments = ctx.ast.vec_from_iter(iter::once(Argument::from(source)).chain(arguments));
        *expr = ctx.ast.expression_call(span, callee, NONE, arguments, false);
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
                // TODO(improvement): It looks like here could always return a computed member expression,
                //                    so that we don't need to check if it's an identifier name.
                *expr = if is_identifier_name(property) {
                    create_property_access(ident.span, object, property, ctx)
                } else {
                    create_compute_property_access(ident.span, object, property, ctx)
                };
            } else {
                *expr = object;
            }
        }
    }

    #[inline]
    fn transform_meta_property(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::MetaProperty(meta) = expr else {
            unreachable!();
        };

        *expr = ctx.create_unbound_ident_expr(meta.span, SSR_IMPORT_META_KEY, ReferenceFlags::Read);
    }

    fn transform_import(
        &mut self,
        span: Span,
        source: StringLiteral<'a>,
        specifiers: Option<ArenaVec<'a, ImportDeclarationSpecifier<'a>>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        // ['vue', { importedNames: ['foo'] }]`
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
                local.name = self.generate_import_binding_name(ctx);
                let binding = BoundIdentifier::from_binding_ident(&local);
                ctx.symbols_mut().set_name(binding.symbol_id, &binding.name);
                self.import_bindings.insert(binding.symbol_id, (binding, None));

                let binding_pattern_kind = BindingPatternKind::BindingIdentifier(ctx.alloc(local));
                ctx.ast.binding_pattern(binding_pattern_kind, NONE, false)
            } else {
                let binding = self.generate_import_binding(ctx);
                arguments.push(self.transform_import_metadata(&binding, specifiers, ctx));
                binding.create_binding_pattern(ctx)
            }
        } else {
            let binding = self.generate_import_binding(ctx);
            binding.create_binding_pattern(ctx)
        };

        Self::create_import(span, pattern, arguments, ctx)
    }

    fn transform_export_named_declaration(
        &mut self,
        new_stmts: &mut ArenaVec<'a, Statement<'a>>,
        export: ArenaBox<'a, ExportNamedDeclaration<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let ExportNamedDeclaration { span, source, specifiers, declaration, .. } = export.unbox();

        if let Some(declaration) = declaration {
            let export_expression = match &declaration {
                Declaration::VariableDeclaration(variable) => {
                    let new_stmts_index = new_stmts.len();
                    variable.bound_names(&mut |ident| {
                        let binding = BoundIdentifier::from_binding_ident(ident);
                        let ident = binding.create_read_expression(ctx);
                        new_stmts.push(Self::create_export(span, ident, binding.name, ctx));
                    });
                    // Should be inserted before the exports
                    new_stmts.insert(new_stmts_index, Statement::from(declaration));
                    return;
                }
                Declaration::FunctionDeclaration(func) => {
                    let binding = BoundIdentifier::from_binding_ident(func.id.as_ref().unwrap());
                    let ident = binding.create_read_expression(ctx);
                    Self::create_export(span, ident, binding.name, ctx)
                }
                Declaration::ClassDeclaration(class) => {
                    let binding = BoundIdentifier::from_binding_ident(class.id.as_ref().unwrap());
                    let ident = binding.create_read_expression(ctx);
                    Self::create_export(span, ident, binding.name, ctx)
                }
                _ => {
                    unreachable!(
                        "Unsupported for transforming typescript declaration in named export"
                    );
                }
            };
            new_stmts.extend([Statement::from(declaration), export_expression]);
        } else {
            // ```js
            // export { foo, bar } from 'vue';
            // // to
            // const __vite_ssr_import_1__ = await __vite_ssr_import__('vue', { importedNames: ['foo', 'bar'] });
            // Object.defineProperty(__vite_ssr_exports__, 'foo', { enumerable: true, configurable: true, get(){ return __vite_ssr_import_1__.foo } });
            // Object.defineProperty(__vite_ssr_exports__, 'bar', { enumerable: true, configurable: true, get(){ return __vite_ssr_import_1__.bar } });
            // ```
            let import_binding = source.map(|source| {
                let binding = self.generate_import_binding(ctx);
                let pattern = binding.create_binding_pattern(ctx);
                let imported_names = ctx.ast.vec_from_iter(specifiers.iter().map(|specifier| {
                    let local_name = specifier.local.name();
                    let local_name_expr = ctx.ast.expression_string_literal(SPAN, local_name, None);
                    ArrayExpressionElement::from(local_name_expr)
                }));
                let arguments = ctx.ast.vec_from_array([
                    Argument::from(Expression::StringLiteral(ctx.ast.alloc(source))),
                    Self::create_imported_names_object(imported_names, ctx),
                ]);
                new_stmts.push(Self::create_import(SPAN, pattern, arguments, ctx));
                binding
            });

            new_stmts.extend(specifiers.into_iter().map(|specifier| {
                let ExportSpecifier { span, exported, local, .. } = specifier;
                let expr = if let Some(import_binding) = &import_binding {
                    let object = import_binding.create_read_expression(ctx);
                    let property = local.name();
                    // TODO(improvement): It looks like here could always return a computed member expression,
                    //                    so that we don't need to check if it's an identifier name.
                    if is_identifier_name(&property) {
                        create_property_access(SPAN, object, &property, ctx)
                    } else {
                        create_compute_property_access(SPAN, object, &property, ctx)
                    }
                } else {
                    let ModuleExportName::IdentifierReference(ident) = local else {
                        unreachable!()
                    };
                    Expression::Identifier(ctx.ast.alloc(ident))
                };
                Self::create_export(span, expr, exported.name(), ctx)
            }));
        }
    }

    fn transform_export_all_declaration(
        &mut self,
        new_stmts: &mut ArenaVec<'a, Statement<'a>>,
        export: ArenaBox<'a, ExportAllDeclaration<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let ExportAllDeclaration { span, source, exported, .. } = export.unbox();
        let binding = self.generate_import_binding(ctx);
        let pattern = binding.create_binding_pattern(ctx);
        let arguments =
            ctx.ast.vec1(Argument::from(Expression::StringLiteral(ctx.ast.alloc(source))));
        new_stmts.push(Self::create_import(span, pattern, arguments, ctx));

        let ident = binding.create_read_expression(ctx);

        let export = if let Some(exported) = exported {
            // `export * as foo from 'vue'` ->
            // `defineProperty(__vite_ssr_exports__, 'foo', { enumerable: true, configurable: true, get(){ return __vite_ssr_import_1__ } });`
            Self::create_export(span, ident, exported.name(), ctx)
        } else {
            let callee = ctx.ast.expression_identifier(SPAN, SSR_EXPORT_ALL_KEY);
            let arguments = ctx.ast.vec1(Argument::from(ident));
            // `export * from 'vue'` -> `__vite_ssr_exportAll__(__vite_ssr_import_1__);`
            let call = ctx.ast.expression_call(SPAN, callee, NONE, arguments, false);
            ctx.ast.statement_expression(span, call)
        };
        new_stmts.push(export);
    }

    fn transform_export_default_declaration(
        &self,
        new_stmts: &mut ArenaVec<'a, Statement<'a>>,
        export: ArenaBox<'a, ExportDefaultDeclaration<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let default = Atom::new_const("default");
        let ExportDefaultDeclaration { span, declaration, .. } = export.unbox();
        let expr = match declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(mut func) => {
                // `export default function foo() {}` ->
                // `function foo() {}; Object.defineProperty(__vite_ssr_exports__, 'default', { enumerable: true, configurable: true, get(){ return foo } });`
                if let Some(id) = &func.id {
                    let ident = BoundIdentifier::from_binding_ident(id).create_read_expression(ctx);
                    new_stmts.extend([
                        Statement::FunctionDeclaration(func),
                        Self::create_export(span, ident, default, ctx),
                    ]);
                    return;
                }

                func.r#type = FunctionType::FunctionExpression;
                Expression::FunctionExpression(func)
            }
            ExportDefaultDeclarationKind::ClassDeclaration(mut class) => {
                // `export default class Foo {}` ->
                // `class Foo {}; Object.defineProperty(__vite_ssr_exports__, 'default', { enumerable: true, configurable: true, get(){ return Foo } });`
                if let Some(id) = &class.id {
                    let ident = BoundIdentifier::from_binding_ident(id).create_read_expression(ctx);
                    new_stmts.extend([
                        Statement::ClassDeclaration(class),
                        Self::create_export(span, ident, default, ctx),
                    ]);
                    return;
                }

                class.r#type = ClassType::ClassExpression;
                Expression::ClassExpression(class)
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(_) => {
                // Do nothing for `interface Foo {}`
                return;
            }
            expr @ match_expression!(ExportDefaultDeclarationKind) => expr.into_expression(),
        };
        // `export default expr` -> `Object.defineProperty(__vite_ssr_exports__, 'default', { enumerable: true, configurable: true, get(){ return expr } });`
        new_stmts.push(Self::create_export(span, expr, default, ctx));
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
        Self::create_imported_names_object(elements, ctx)
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

    /// Generate a unique import binding name like `__vite_ssr_import_{uid}__`.
    fn generate_import_binding_name(&mut self, ctx: &TraverseCtx<'a>) -> Atom<'a> {
        self.import_uid += 1;
        let uid = self.import_uid.to_compact_string();
        let capacity = 20 + uid.len();
        let mut string = ArenaString::with_capacity_in(capacity, ctx.ast.allocator);
        string.push_str("__vite_ssr_import_");
        string.push_str(&uid);
        string.push_str("__");
        Atom::from(string)
    }

    /// Generate a unique import binding whose name is like `__vite_ssr_import_{uid}__`.
    #[inline]
    fn generate_import_binding(&mut self, ctx: &mut TraverseCtx<'a>) -> BoundIdentifier<'a> {
        let name = self.generate_import_binding_name(ctx);
        ctx.generate_binding_in_current_scope(name, SymbolFlags::BlockScopedVariable)
    }

    // { importedNames: ['foo', 'bar'] }
    fn create_imported_names_object(
        elements: ArenaVec<'a, ArrayExpressionElement<'a>>,
        ctx: &TraverseCtx<'a>,
    ) -> Argument<'a> {
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
        Argument::from(ctx.ast.expression_object(SPAN, ctx.ast.vec1(imported_names), None))
    }

    // `const __vite_ssr_import_1__ = await __vite_ssr_import__('vue', { importedNames: ['foo'] });`
    fn create_import(
        span: Span,
        pattern: BindingPattern<'a>,
        arguments: ArenaVec<'a, Argument<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let callee = ctx.create_unbound_ident_expr(SPAN, SSR_IMPORT_KEY, ReferenceFlags::Read);
        let call = ctx.ast.expression_call(SPAN, callee, NONE, arguments, false);
        let init = ctx.ast.expression_await(SPAN, call);

        let kind = VariableDeclarationKind::Const;
        let declarator = ctx.ast.variable_declarator(SPAN, kind, pattern, Some(init), false);
        let declaration = ctx.ast.declaration_variable(span, kind, ctx.ast.vec1(declarator), false);
        Statement::from(declaration)
    }

    // `Object.defineProperty(...arguments)`
    fn create_define_property(
        arguments: ArenaVec<'a, Argument<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let object =
            ctx.create_unbound_ident_expr(SPAN, Atom::from("Object"), ReferenceFlags::Read);
        let member = create_member_callee(object, "defineProperty", ctx);
        ctx.ast.expression_call(SPAN, member, NONE, arguments, false)
    }

    // `key: value` or `key() {}`
    fn create_object_property(
        key: &str,
        value: Option<Expression<'a>>,
        ctx: &TraverseCtx<'a>,
    ) -> ObjectPropertyKind<'a> {
        let is_method = value.is_some();
        ctx.ast.object_property_kind_object_property(
            SPAN,
            PropertyKind::Init,
            ctx.ast.property_key_static_identifier(SPAN, key),
            value.unwrap_or_else(|| ctx.ast.expression_boolean_literal(SPAN, true)),
            is_method,
            false,
            false,
        )
    }

    /// `{ enumerable: true, configurable: true, get(){ return expr } }`
    fn create_function_with_return_statement(
        expr: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let kind = FormalParameterKind::FormalParameter;
        let params = ctx.ast.formal_parameters(SPAN, kind, ctx.ast.vec(), NONE);
        let statement = ctx.ast.statement_return(SPAN, Some(expr));
        let body = ctx.ast.function_body(SPAN, ctx.ast.vec(), ctx.ast.vec1(statement));
        let r#type = FunctionType::FunctionExpression;
        let scope_id = ctx.create_child_scope(ctx.scopes().root_scope_id(), ScopeFlags::Function);
        let function = ctx.ast.alloc_function_with_scope_id(
            SPAN,
            r#type,
            None,
            false,
            false,
            false,
            NONE,
            NONE,
            params,
            NONE,
            Some(body),
            scope_id,
        );
        Expression::FunctionExpression(function)
    }

    // `Object.defineProperty(__vite_ssr_exports__, 'foo', {enumerable: true, configurable: true, get(){ return foo }});`
    fn create_export(
        span: Span,
        expr: Expression<'a>,
        exported_name: Atom<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let getter = Self::create_function_with_return_statement(expr, ctx);
        let object = ctx.ast.expression_object(
            SPAN,
            ctx.ast.vec_from_array([
                Self::create_object_property("enumerable", None, ctx),
                Self::create_object_property("configurable", None, ctx),
                Self::create_object_property("get", Some(getter), ctx),
            ]),
            None,
        );

        let arguments = ctx.ast.vec_from_array([
            Argument::from(ctx.create_unbound_ident_expr(
                SPAN,
                SSR_MODULE_EXPORTS_KEY,
                ReferenceFlags::Read,
            )),
            Argument::from(ctx.ast.expression_string_literal(SPAN, exported_name, None)),
            Argument::from(object),
        ]);

        ctx.ast.statement_expression(span, Self::create_define_property(arguments, ctx))
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

    #[test]
    fn export_function_declaration() {
        test_same(
            "export function foo() {}",
            "function foo() {}\nObject.defineProperty(__vite_ssr_exports__, 'foo', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn foo;\n\t}\n});\n",
        );
    }

    #[test]
    fn export_class_declaration() {
        test_same(
            "export class foo {}",
            "class foo {}\nObject.defineProperty(__vite_ssr_exports__, 'foo', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn foo;\n\t}\n});\n",
        );
    }

    #[test]
    fn export_var_declaration() {
        test_same(
            "export const a = 1, b = 2",
            "const a = 1, b = 2;\nObject.defineProperty(__vite_ssr_exports__, 'a', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn a;\n\t}\n});\nObject.defineProperty(__vite_ssr_exports__, 'b', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn b;\n\t}\n});\n",
        );
    }

    #[test]
    fn export_named() {
        test_same(
            "const a = 1, b = 2; export { a, b as c }",
            "const a = 1, b = 2;\nObject.defineProperty(__vite_ssr_exports__, 'a', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn a;\n\t}\n});\nObject.defineProperty(__vite_ssr_exports__, 'c', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn b;\n\t}\n});\n",
        );
    }

    #[test]
    fn export_named_from() {
        test_same(
            "export { ref, computed as c } from 'vue'",
            "const __vite_ssr_import_1__ = await __vite_ssr_import__('vue', { importedNames: ['ref', 'computed'] });\nObject.defineProperty(__vite_ssr_exports__, 'ref', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn __vite_ssr_import_1__.ref;\n\t}\n});\nObject.defineProperty(__vite_ssr_exports__, 'c', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn __vite_ssr_import_1__.computed;\n\t}\n});\n",
        );
    }

    #[test]
    fn export_star_from() {
        test_same(
            "export * from 'vue'\nexport * from 'react'",
            "const __vite_ssr_import_1__ = await __vite_ssr_import__('vue');\n__vite_ssr_exportAll__(__vite_ssr_import_1__);\nconst __vite_ssr_import_2__ = await __vite_ssr_import__('react');\n__vite_ssr_exportAll__(__vite_ssr_import_2__);\n",
        );
    }

    #[test]
    fn export_star_as_from() {
        test_same(
            "export * as foo from 'vue'",
            "const __vite_ssr_import_1__ = await __vite_ssr_import__('vue');\nObject.defineProperty(__vite_ssr_exports__, 'foo', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn __vite_ssr_import_1__;\n\t}\n});\n",
        );
    }

    #[test]
    fn re_export_by_imported_name() {
        test_same(
            "import * as foo from 'foo'; export * as foo from 'foo'",
            "const __vite_ssr_import_1__ = await __vite_ssr_import__('foo');\nconst __vite_ssr_import_2__ = await __vite_ssr_import__('foo');\nObject.defineProperty(__vite_ssr_exports__, 'foo', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn __vite_ssr_import_2__;\n\t}\n});\n",
        );

        test_same(
            "import { foo } from 'foo'; export { foo } from 'foo'",
            "const __vite_ssr_import_1__ = await __vite_ssr_import__('foo', { importedNames: ['foo'] });\nconst __vite_ssr_import_2__ = await __vite_ssr_import__('foo', { importedNames: ['foo'] });\nObject.defineProperty(__vite_ssr_exports__, 'foo', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn __vite_ssr_import_2__.foo;\n\t}\n});\n",
        );
    }

    #[test]
    fn export_arbitrary_namespace() {
        test_same(
            "export * as \"arbitrary string\" from 'vue'",
            "const __vite_ssr_import_1__ = await __vite_ssr_import__('vue');\nObject.defineProperty(__vite_ssr_exports__, 'arbitrary string', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn __vite_ssr_import_1__;\n\t}\n});\n",
        );

        test_same(
            "const something = \"Something\"; export { something as \"arbitrary string\" }",
            "const something = 'Something';\nObject.defineProperty(__vite_ssr_exports__, 'arbitrary string', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn something;\n\t}\n});\n",
        );

        test_same(
            "export { \"arbitrary string2\" as \"arbitrary string\" } from 'vue'",
            "const __vite_ssr_import_1__ = await __vite_ssr_import__('vue', { importedNames: ['arbitrary string2'] });\nObject.defineProperty(__vite_ssr_exports__, 'arbitrary string', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn __vite_ssr_import_1__['arbitrary string2'];\n\t}\n});\n",
        );
    }

    #[test]
    fn export_default() {
        test_same(
            "export default {}",
            "Object.defineProperty(__vite_ssr_exports__, 'default', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn {};\n\t}\n});\n",
        );
    }

    #[test]
    fn export_then_import_minified() {
        test_same(
            "export * from 'vue';import {createApp} from 'vue'",
            "const __vite_ssr_import_1__ = await __vite_ssr_import__('vue');\n__vite_ssr_exportAll__(__vite_ssr_import_1__);\nconst __vite_ssr_import_2__ = await __vite_ssr_import__('vue', { importedNames: ['createApp'] });\n",
        );
    }

    #[test]
    fn hoist_import_to_top() {
        test_same(
            "path.resolve('server.js');import path from 'node:path';",
            "const __vite_ssr_import_1__ = await __vite_ssr_import__('node:path', { importedNames: ['default'] });\n__vite_ssr_import_1__.default.resolve('server.js');\n",
        );
    }

    #[test]
    fn import_meta() {
        test_same("console.log(import.meta.url)", "console.log(__vite_ssr_import_meta__.url);\n");
    }

    #[test]
    fn dynamic_import() {
        test_same(
            "export const i = () => import('./foo')",
            "const i = () => __vite_ssr_dynamic_import__('./foo');\nObject.defineProperty(__vite_ssr_exports__, 'i', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn i;\n\t}\n});\n",
        );
    }

    /// <https://github.com/vitejs/vite/issues/4049>
    #[test]
    fn handle_default_export_variants() {
        // default anonymous functions
        test_same(
            "export default function() {}",
            "Object.defineProperty(__vite_ssr_exports__, 'default', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn function() {};\n\t}\n});\n",
        );

        // default anonymous class
        test_same(
            "export default class {}",
            "Object.defineProperty(__vite_ssr_exports__, 'default', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn class {};\n\t}\n});\n",
        );

        // default named functions
        test_same(
            "export default function foo() {}\nfoo.prototype = Object.prototype;",
            // "function foo() {}\nfoo.prototype = Object.prototype;\nObject.defineProperty(__vite_ssr_exports__, 'default', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn foo;\n\t}\n});\n",
            "function foo() {}\nObject.defineProperty(__vite_ssr_exports__, 'default', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn foo;\n\t}\n});\nfoo.prototype = Object.prototype;\n",
        );

        // default named classes
        test_same(
            "export default class A {}\nexport class B extends A {}",
            // "class A {}\nclass B extends A {}\nObject.defineProperty(__vite_ssr_exports__, 'B', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn B;\n\t}\n});\nObject.defineProperty(__vite_ssr_exports__, 'default', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn A;\n\t}\n});\n",
            "class A {}\nObject.defineProperty(__vite_ssr_exports__, 'default', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn A;\n\t}\n});\nclass B extends A {}\nObject.defineProperty(__vite_ssr_exports__, 'B', {\n\tenumerable: true,\n\tconfigurable: true,\n\tget() {\n\t\treturn B;\n\t}\n});\n",
        );
    }
}
