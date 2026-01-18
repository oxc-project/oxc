//! Module runner transform
//!
//! This plugin is used to transform import statement to import by `__vite_ssr_import__`
//! and export statement to export by `__vite_ssr_exports__`, these functions will be
//! injected by Vite node.
//!
//! ## Example
//!
//! Input:
//! ```js
//! import { foo } from 'vue';
//! import vue from 'vue';
//! import * as vue from 'vue';
//!
//! foo();
//! console.log(vue.version);
//! console.log(vue.zoo());
//! ```
//!
//! Output:
//! ```js
//! const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['foo'] });
//! const __vite_ssr_import_1__ = await __vite_ssr_import__('vue', { importedNames: ['default'] });
//! const __vite_ssr_import_2__ = await __vite_ssr_import__('vue');
//! (0, __vite_ssr_import_0__.foo)();
//! console.log(__vite_ssr_import_2__.version);
//! console.log(__vite_ssr_import_2__.zoo());
//! ```
//!
//! ## Implementation
//!
//! Based on [Vite](https://github.com/vitejs/vite/blob/00deea4ff88e30e299cb40a801b5dc0205ac913d/packages/vite/src/node/ssr/ssrTransform.ts)'s ssrTransform.
//!
//! All tests are copy-pasted from [ssrTransform.spec.ts](https://github.com/vitejs/vite/blob/00deea4ff88e30e299cb40a801b5dc0205ac913d/packages/vite/src/node/ssr/__tests__/ssrTransform.spec.ts).
//!
//! ## Integrate into main `Transformer` in future
//!
//! There are few problems to integrate this transform into the main transformer:
//!
//! 1. In Vite, it will collect import deps and dynamic import deps during the transform process, and return them
//!    at the end of function. We can do this, but how to pass them into the js side?
//!
//! 2. In case other plugins will insert imports/exports, we must transform them in `exit_program`, but it will pose
//!    another problem: how to transform identifiers which refer to imports? We must collect some information from imports,
//!    but it is already at the end of the visitor. To solve this, we may introduce a new visitor to transform identifiers,
//!    dynamic imports, and import meta.

use std::iter;

use itoa::Buffer as ItoaBuffer;
use rustc_hash::{FxHashMap, FxHashSet};

use oxc_allocator::{Allocator, Box as ArenaBox, TakeIn, Vec as ArenaVec};
use oxc_ast::{NONE, ast::*};
use oxc_ecmascript::BoundNames;
use oxc_semantic::{ReferenceFlags, ScopeFlags, Scoping, SymbolFlags, SymbolId};
use oxc_span::SPAN;
use oxc_syntax::identifier::is_identifier_name;
use oxc_traverse::{Ancestor, BoundIdentifier, Traverse, traverse_mut};

use crate::TraverseCtx;

#[derive(Debug, Default)]
pub struct ModuleRunnerTransform<'a> {
    /// Uid for generating import binding names.
    import_uid: u32,
    /// Import bindings used to determine which identifiers should be transformed.
    /// The key is a symbol id that belongs to the import binding.
    /// The value is a tuple of (Binding, Property).
    import_bindings: FxHashMap<SymbolId, (BoundIdentifier<'a>, Option<Atom<'a>>)>,

    // Collect deps and dynamic deps for Vite
    deps: FxHashSet<String>,
    dynamic_deps: FxHashSet<String>,
}

impl<'a> ModuleRunnerTransform<'a> {
    pub fn new() -> Self {
        Self {
            import_uid: 0,
            import_bindings: FxHashMap::default(),
            deps: FxHashSet::default(),
            dynamic_deps: FxHashSet::default(),
        }
    }

    /// Standalone transform
    pub fn transform(
        mut self,
        allocator: &'a Allocator,
        program: &mut Program<'a>,
        scoping: Scoping,
    ) -> (FxHashSet<String>, FxHashSet<String>) {
        traverse_mut(&mut self, allocator, program, scoping, ());
        (self.deps, self.dynamic_deps)
    }
}

const SSR_MODULE_EXPORTS_KEY: Atom<'static> = Atom::new_const("__vite_ssr_exports__");
const SSR_EXPORT_DEFAULT_KEY: Atom<'static> = Atom::new_const("__vite_ssr_export_default__");
const SSR_IMPORT_KEY: Atom<'static> = Atom::new_const("__vite_ssr_import__");
const SSR_DYNAMIC_IMPORT_KEY: Atom<'static> = Atom::new_const("__vite_ssr_dynamic_import__");
const SSR_EXPORT_ALL_KEY: Atom<'static> = Atom::new_const("__vite_ssr_exportAll__");
const SSR_IMPORT_META_KEY: Atom<'static> = Atom::new_const("__vite_ssr_import_meta__");
const DEFAULT: Atom<'static> = Atom::new_const("default");

impl<'a> Traverse<'a, ()> for ModuleRunnerTransform<'a> {
    #[inline]
    fn enter_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        self.transform_imports_and_exports(program, ctx);
    }

    #[inline]
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        match expr {
            Expression::Identifier(_) => self.transform_identifier(expr, ctx),
            Expression::MetaProperty(_) => Self::transform_meta_property(expr, ctx),
            Expression::ImportExpression(_) => self.transform_dynamic_import(expr, ctx),
            _ => {}
        }
    }
}

impl<'a> ModuleRunnerTransform<'a> {
    /// Transform import and export declarations.
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

        let mut hoist_imports = Vec::with_capacity(program.body.len());
        let mut hoist_exports = Vec::with_capacity(program.body.len());

        for stmt in program.body.drain(..) {
            match stmt {
                Statement::ImportDeclaration(import) => {
                    let ImportDeclaration { span, source, specifiers, .. } = import.unbox();
                    let import_statement = self.transform_import(span, source, specifiers, ctx);
                    hoist_imports.push(import_statement);
                }
                Statement::ExportAllDeclaration(export) => {
                    self.transform_export_all_declaration(
                        &mut hoist_imports,
                        &mut hoist_exports,
                        export,
                        ctx,
                    );
                }
                Statement::ExportNamedDeclaration(export) => {
                    self.transform_export_named_declaration(
                        &mut new_stmts,
                        &mut hoist_imports,
                        &mut hoist_exports,
                        export,
                        ctx,
                    );
                }
                Statement::ExportDefaultDeclaration(export) => {
                    Self::transform_export_default_declaration(
                        &mut new_stmts,
                        &mut hoist_exports,
                        export,
                        ctx,
                    );
                }
                _ => {
                    new_stmts.push(stmt);
                }
            }
        }

        new_stmts.splice(0..0, hoist_exports.into_iter().chain(hoist_imports));

        program.body = new_stmts;
    }

    /// Transform `identifier` to point correctly imported binding.
    ///
    /// - Import without renaming
    /// ```js
    /// import { foo } from 'vue';
    /// foo;
    /// // to
    /// __vite_ssr_import_0__.foo;
    /// ```
    ///
    /// - Import with renaming
    /// ```js
    /// import { "arbitrary string" as bar } from 'vue';
    /// bar;
    /// // to
    /// __vite_ssr_import_0__["arbitrary string"];
    /// ```
    ///
    /// - The identifier is a callee of a call expression
    /// ```js
    /// import { foo } from 'vue';
    /// foo();
    /// // to
    /// (0, __vite_ssr_import_0__.foo)();
    /// ```
    fn transform_identifier(&self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::Identifier(ident) = expr else {
            unreachable!();
        };

        let Some((binding, property)) = ident
            .reference_id
            .get()
            .and_then(|id| ctx.scoping().get_reference(id).symbol_id())
            .and_then(|id| self.import_bindings.get(&id))
        else {
            return;
        };

        let object = binding.create_read_expression(ctx);
        *expr = if let Some(property) = property {
            // TODO(improvement): It looks like here could always return a computed member expression,
            //                    so that we don't need to check if it's an identifier name.
            // __vite_ssr_import_0__.foo
            let expr = if is_identifier_name(property) {
                create_property_access(ident.span, object, property, ctx)
            } else {
                // __vite_ssr_import_0__['arbitrary string']
                create_compute_property_access(ident.span, object, property, ctx)
            };

            if matches!(ctx.parent(), Ancestor::CallExpressionCallee(_)) {
                // wrap with (0, ...) to avoid method binding `this`
                // <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Property_accessors#method_binding>
                let zero =
                    ctx.ast.expression_numeric_literal(SPAN, 0f64, None, NumberBase::Decimal);
                let expressions = ctx.ast.vec_from_array([zero, expr]);
                ctx.ast.expression_sequence(ident.span, expressions)
            } else {
                expr
            }
        } else {
            object
        };
    }

    /// Transform `import(source, ...arguments)` to `__vite_ssr_dynamic_import__(source, ...arguments)`.
    #[inline]
    fn transform_dynamic_import(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::ImportExpression(import_expr) = expr.take_in(ctx.ast) else {
            unreachable!();
        };

        let ImportExpression { span, source, options, .. } = import_expr.unbox();

        if let Expression::StringLiteral(source) = &source {
            self.dynamic_deps.insert(source.value.to_string());
        }

        let flags = ReferenceFlags::Read;
        let callee = ctx.create_unbound_ident_expr(SPAN, SSR_DYNAMIC_IMPORT_KEY, flags);
        let arguments = options.into_iter().map(Argument::from);
        let arguments = ctx.ast.vec_from_iter(iter::once(Argument::from(source)).chain(arguments));
        *expr = ctx.ast.expression_call(span, callee, NONE, arguments, false);
    }

    /// Transform `import.meta` to `__vite_ssr_import_meta__`.
    #[inline]
    fn transform_meta_property(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::MetaProperty(meta) = expr else {
            unreachable!();
        };

        *expr = ctx.create_unbound_ident_expr(meta.span, SSR_IMPORT_META_KEY, ReferenceFlags::Read);
    }

    /// Transform import declaration (`import { foo } from 'vue'`).
    ///
    /// - Import specifier
    /// ```js
    /// import { foo, bar } from 'vue';
    /// // to
    /// const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['foo', 'bar'] });
    /// ```
    ///
    /// - Import default specifier
    /// ```js
    /// import vue from 'vue';
    /// // to
    /// const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['default'] });
    /// ```
    ///
    /// - Import namespace specifier
    /// ```js
    /// import * as vue from 'vue';
    /// // to
    /// const __vite_ssr_import_0__ = await __vite_ssr_import__('vue');
    /// ```
    fn transform_import(
        &mut self,
        span: Span,
        source: StringLiteral<'a>,
        specifiers: Option<ArenaVec<'a, ImportDeclarationSpecifier<'a>>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        self.deps.insert(source.value.to_string());

        // ['vue', { importedNames: ['foo'] }]`
        let mut arguments = ctx.ast.vec_with_capacity(1 + usize::from(specifiers.is_some()));
        arguments.push(Argument::from(Expression::StringLiteral(ctx.ast.alloc(source))));
        let pattern = if let Some(mut specifiers) = specifiers {
            // `import * as vue from 'vue';` -> `const __vite_ssr_import_0__ = await __vite_ssr_import__('vue');`
            if matches!(
                specifiers.last(),
                Some(ImportDeclarationSpecifier::ImportNamespaceSpecifier(_))
            ) {
                let Some(ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier)) =
                    specifiers.pop()
                else {
                    unreachable!()
                };

                // Reuse the `vue` binding identifier by renaming it to `__vite_ssr_import_0__`
                let mut local = specifier.unbox().local;
                local.name = self.generate_import_binding_name(ctx);
                let binding = BoundIdentifier::from_binding_ident(&local);
                ctx.scoping_mut().set_symbol_name(binding.symbol_id, &binding.name);
                self.import_bindings.insert(binding.symbol_id, (binding, None));

                BindingPattern::BindingIdentifier(ctx.alloc(local))
            } else {
                let binding = self.generate_import_binding(ctx);
                arguments.push(self.transform_import_specifiers(&binding, specifiers, ctx));
                binding.create_binding_pattern(ctx)
            }
        } else {
            let binding = self.generate_import_binding(ctx);
            binding.create_binding_pattern(ctx)
        };

        Self::create_import(span, pattern, arguments, ctx)
    }

    /// Transform named export declaration (`export function foo() {}`).
    ///
    /// - Export a declaration
    /// ```js
    /// export function foo() {}
    /// // to
    /// Object.defineProperty(__vite_ssr_exports__, 'foo', { enumerable: true, configurable: true, get() { return foo; }});
    /// function foo() {}
    /// ```
    ///
    /// - Export specifiers
    /// ```js
    /// export { foo, bar };
    /// // to
    /// Object.defineProperty(__vite_ssr_exports__, 'foo', { enumerable: true, configurable: true, get() { return foo; }});
    /// Object.defineProperty(__vite_ssr_exports__, 'bar', { enumerable: true, configurable: true, get() { return bar; }});
    /// ```
    ///
    /// - Export specifiers from module
    /// ```js
    /// export { foo, bar } from 'vue';
    /// // to
    /// Object.defineProperty(__vite_ssr_exports__, 'foo', { enumerable: true, configurable: true, get() { return __vite_ssr_import_0__.foo; }});
    /// Object.defineProperty(__vite_ssr_exports__, 'bar', { enumerable: true, configurable: true, get() { return __vite_ssr_import_0__.bar; }});
    /// const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['foo', 'bar'] });
    /// ```
    ///
    /// - Export specifiers with renaming
    /// ```js
    /// export { foo as bar };
    /// // to
    /// Object.defineProperty(__vite_ssr_exports__, 'bar', { enumerable: true, configurable: true, get() { return foo; }});
    /// ```
    fn transform_export_named_declaration(
        &mut self,
        new_stmts: &mut ArenaVec<'a, Statement<'a>>,
        hoist_imports: &mut Vec<Statement<'a>>,
        hoist_exports: &mut Vec<Statement<'a>>,
        export: ArenaBox<'a, ExportNamedDeclaration<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let ExportNamedDeclaration { span, source, specifiers, declaration, .. } = export.unbox();

        if let Some(declaration) = declaration {
            let export_expression = match &declaration {
                // `export const [foo, bar] = [1, 2];`
                Declaration::VariableDeclaration(variable) => {
                    let new_stmts_index = new_stmts.len();
                    variable.bound_names(&mut |ident| {
                        let binding = BoundIdentifier::from_binding_ident(ident);
                        let ident = binding.create_read_expression(ctx);
                        hoist_exports.push(Self::create_export(span, ident, binding.name, ctx));
                    });
                    // Should be inserted before the exports
                    new_stmts.insert(new_stmts_index, Statement::from(declaration));
                    return;
                }
                // `export function foo() {}`
                Declaration::FunctionDeclaration(func) => {
                    let binding = BoundIdentifier::from_binding_ident(func.id.as_ref().unwrap());
                    let ident = binding.create_read_expression(ctx);
                    Self::create_export(span, ident, binding.name, ctx)
                }
                // `export class Foo {}`
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
            new_stmts.push(Statement::from(declaration));
            hoist_exports.push(export_expression);
        } else {
            // If the source is Some, then we need to import the module first and then export them.
            let import_binding = source.map(|source| {
                self.deps.insert(source.value.to_string());
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
                hoist_imports.push(Self::create_import(SPAN, pattern, arguments, ctx));
                binding
            });

            hoist_exports.extend(specifiers.into_iter().map(|specifier| {
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

    /// Transform export all declaration (`export * from 'vue'`).
    ///
    /// - Without renamed export:
    /// ```js
    /// export * from 'vue';
    /// // to
    /// const __vite_ssr_import_0__ = await __vite_ssr_import__('vue');
    /// Object.defineProperty(__vite_ssr_exports__, 'default', { enumerable: true, configurable: true, get(){ return __vite_ssr_import_0__ } });
    /// ```
    ///
    /// - Renamed export:
    /// ```js
    /// export * as foo from 'vue';
    /// // to
    /// Object.defineProperty(__vite_ssr_exports__, 'foo', { enumerable: true, configurable: true, get(){ return __vite_ssr_import_0__ } });
    /// const __vite_ssr_import_0__ = await __vite_ssr_import__('vue');
    /// ```
    fn transform_export_all_declaration(
        &mut self,
        hoist_imports: &mut Vec<Statement<'a>>,
        hoist_exports: &mut Vec<Statement<'a>>,
        export: ArenaBox<'a, ExportAllDeclaration<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let ExportAllDeclaration { span, source, exported, .. } = export.unbox();
        self.deps.insert(source.value.to_string());
        let binding = self.generate_import_binding(ctx);
        let pattern = binding.create_binding_pattern(ctx);
        let arguments =
            ctx.ast.vec1(Argument::from(Expression::StringLiteral(ctx.ast.alloc(source))));
        let import = Self::create_import(span, pattern, arguments, ctx);

        let ident = binding.create_read_expression(ctx);

        if let Some(exported) = exported {
            // `export * as foo from 'vue'` ->
            // `Object.defineProperty(__vite_ssr_exports__, 'foo', { enumerable: true, configurable: true, get(){ return __vite_ssr_import_0__ } });`
            let export = Self::create_export(span, ident, exported.name(), ctx);
            hoist_imports.push(import);
            hoist_exports.push(export);
        } else {
            let callee = ctx.ast.expression_identifier(SPAN, SSR_EXPORT_ALL_KEY);
            let arguments = ctx.ast.vec1(Argument::from(ident));
            // `export * from 'vue'` -> `__vite_ssr_exportAll__(__vite_ssr_import_0__);`
            let call = ctx.ast.expression_call(SPAN, callee, NONE, arguments, false);
            let export = ctx.ast.statement_expression(span, call);
            // names from `export *` cannot be known, so add it right after the import.
            hoist_imports.extend([import, export]);
        }
    }

    /// Transform export default declaration (`export default function foo() {}`).
    ///
    /// - Named function declaration
    /// ```js
    /// export default function foo() {}
    /// // to
    /// function foo() {}
    /// Object.defineProperty(__vite_ssr_exports__, 'default', { enumerable: true, configurable: true, get(){ return foo } });
    /// ```
    ///
    /// - Named class declaration
    /// ```js
    /// export default class Foo {}
    /// // to
    /// class Foo {}
    /// Object.defineProperty(__vite_ssr_exports__, 'default', { enumerable: true, configurable: true, get(){ return Foo } });
    /// ```
    ///
    /// - Without named declaration and expression
    /// ```js
    /// export default function () {}
    /// export default {}
    /// // to
    /// Object.defineProperty(__vite_ssr_exports__, 'default', { enumerable: true, configurable: true, get(){ return __vite_ssr_export_default__ } });
    /// const __vite_ssr_export_default__ = function () {}
    /// const __vite_ssr_export_default__ = {}
    /// ```
    fn transform_export_default_declaration(
        new_stmts: &mut ArenaVec<'a, Statement<'a>>,
        hoist_exports: &mut Vec<Statement<'a>>,
        export: ArenaBox<'a, ExportDefaultDeclaration<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        let ExportDefaultDeclaration { span, declaration, .. } = export.unbox();
        let expr = match declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(mut func) => {
                if let Some(id) = &func.id {
                    let ident = BoundIdentifier::from_binding_ident(id).create_read_expression(ctx);
                    new_stmts.push(Statement::FunctionDeclaration(func));
                    hoist_exports.push(Self::create_export(span, ident, DEFAULT, ctx));
                } else {
                    func.r#type = FunctionType::FunctionExpression;
                    let right = Expression::FunctionExpression(func);
                    new_stmts.push(Self::create_export_default_assignment(span, right, ctx));
                    hoist_exports.push(Self::create_export_default(span, ctx));
                }
                return;
            }
            ExportDefaultDeclarationKind::ClassDeclaration(mut class) => {
                if let Some(id) = &class.id {
                    let ident = BoundIdentifier::from_binding_ident(id).create_read_expression(ctx);
                    new_stmts.push(Statement::ClassDeclaration(class));
                    hoist_exports.push(Self::create_export(span, ident, DEFAULT, ctx));
                } else {
                    class.r#type = ClassType::ClassExpression;
                    let right = Expression::ClassExpression(class);
                    new_stmts.push(Self::create_export_default_assignment(span, right, ctx));
                    hoist_exports.push(Self::create_export_default(span, ctx));
                }
                return;
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(_) => {
                // Do nothing for `export default interface Foo {}`
                return;
            }
            expr @ match_expression!(ExportDefaultDeclarationKind) => expr.into_expression(),
        };

        new_stmts.push(Self::create_export_default_assignment(span, expr, ctx));
        hoist_exports.push(Self::create_export_default(span, ctx));
    }

    /// Transform import specifiers, and return an imported names object.
    fn transform_import_specifiers(
        &mut self,
        binding: &BoundIdentifier<'a>,
        specifiers: ArenaVec<'a, ImportDeclarationSpecifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Argument<'a> {
        let elements =
            ctx.ast.vec_from_iter(specifiers.into_iter().map(|specifier| match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                    let ImportSpecifier { span, local, imported, .. } = specifier.unbox();
                    self.insert_import_binding(span, binding, local, imported.name(), ctx)
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    let ImportDefaultSpecifier { span, local, .. } = specifier.unbox();
                    self.insert_import_binding(span, binding, local, DEFAULT, ctx)
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
                    unreachable!()
                }
            }));
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

        let scopes = ctx.scoping_mut();
        scopes.remove_binding(scopes.root_scope_id(), &name);

        let symbol_id = symbol_id.get().unwrap();
        // Do not need to insert if there no identifiers that point to this symbol
        if !ctx.scoping().symbol_is_unused(symbol_id) {
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
        let mut buffer = ItoaBuffer::new();
        let uid_str = buffer.format(self.import_uid);
        self.import_uid += 1;
        ctx.ast.atom_from_strs_array(["__vite_ssr_import_", uid_str, "__"])
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
        let value = ctx.ast.expression_array(SPAN, elements);
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
        Argument::from(ctx.ast.expression_object(SPAN, ctx.ast.vec1(imported_names)))
    }

    // `const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['foo'] });`
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
        let declarator = ctx.ast.variable_declarator(SPAN, kind, pattern, NONE, Some(init), false);
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
        key: &'static str,
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
        let scope_id = ctx.create_child_scope(ctx.scoping().root_scope_id(), ScopeFlags::Function);
        ctx.ast.expression_function_with_scope_id_and_pure_and_pife(
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
            false,
            false,
        )
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

    fn create_export_default(span: Span, ctx: &mut TraverseCtx<'a>) -> Statement<'a> {
        Self::create_export(
            span,
            ctx.create_unbound_ident_expr(SPAN, SSR_EXPORT_DEFAULT_KEY, ReferenceFlags::Read),
            DEFAULT,
            ctx,
        )
    }

    // const __vite_ssr_export_default__ = right;
    fn create_export_default_assignment(
        span: Span,
        right: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let binding = ctx.generate_binding_in_current_scope(
            SSR_EXPORT_DEFAULT_KEY,
            SymbolFlags::BlockScopedVariable,
        );
        let pattern = binding.create_binding_pattern(ctx);
        let kind = VariableDeclarationKind::Const;
        let declarator = ctx.ast.variable_declarator(SPAN, kind, pattern, NONE, Some(right), false);
        let declaration = ctx.ast.declaration_variable(span, kind, ctx.ast.vec1(declarator), false);
        Statement::from(declaration)
    }
}

/// `object` -> `object['a']`.
fn create_compute_property_access<'a>(
    span: Span,
    object: Expression<'a>,
    property: &str,
    ctx: &TraverseCtx<'a>,
) -> Expression<'a> {
    let expression = ctx.ast.expression_string_literal(SPAN, ctx.ast.atom(property), None);
    Expression::from(ctx.ast.member_expression_computed(span, object, expression, false))
}

/// `object` -> `object.call`.
pub fn create_member_callee<'a>(
    object: Expression<'a>,
    property: &'static str,
    ctx: &TraverseCtx<'a>,
) -> Expression<'a> {
    let property = ctx.ast.identifier_name(SPAN, Atom::from(property));
    Expression::from(ctx.ast.member_expression_static(SPAN, object, property, false))
}

/// `object` -> `object.a`.
pub fn create_property_access<'a>(
    span: Span,
    object: Expression<'a>,
    property: &str,
    ctx: &TraverseCtx<'a>,
) -> Expression<'a> {
    let property = ctx.ast.identifier_name(SPAN, ctx.ast.atom(property));
    Expression::from(ctx.ast.member_expression_static(span, object, property, false))
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use rustc_hash::FxHashSet;
    use similar::TextDiff;

    use oxc_allocator::Allocator;
    use oxc_codegen::{Codegen, CodegenOptions, CommentOptions};
    use oxc_diagnostics::OxcDiagnostic;
    use oxc_parser::Parser;
    use oxc_semantic::SemanticBuilder;
    use oxc_span::SourceType;
    use oxc_tasks_common::print_diff_in_terminal;
    use oxc_transformer::{JsxRuntime, TransformOptions, Transformer};

    use super::ModuleRunnerTransform;

    struct TransformReturn {
        code: String,
        deps: FxHashSet<String>,
        dynamic_deps: FxHashSet<String>,
    }

    fn transform(source_text: &str, is_jsx: bool) -> Result<TransformReturn, Vec<OxcDiagnostic>> {
        let source_type = SourceType::default().with_jsx(is_jsx);
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        let mut program = ret.program;
        let mut scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
        if is_jsx {
            let mut options = TransformOptions::default();
            options.jsx.runtime = JsxRuntime::Classic;
            let ret = Transformer::new(&allocator, Path::new(""), &options)
                .build_with_scoping(scoping, &mut program);
            scoping = ret.scoping;
        }
        let (deps, dynamic_deps) =
            ModuleRunnerTransform::new().transform(&allocator, &mut program, scoping);

        if !ret.errors.is_empty() {
            return Err(ret.errors);
        }
        let code = Codegen::new()
            .with_options(CodegenOptions {
                comments: CommentOptions::disabled(),
                single_quote: true,
                ..CodegenOptions::default()
            })
            .build(&program)
            .code;

        Ok(TransformReturn { code, deps, dynamic_deps })
    }

    fn format_expected_code(source_text: &str) -> String {
        let source_type = SourceType::default();
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        assert!(ret.errors.is_empty());

        Codegen::new()
            .with_options(CodegenOptions {
                comments: CommentOptions::disabled(),
                single_quote: true,
                ..CodegenOptions::default()
            })
            .build(&ret.program)
            .code
    }

    #[track_caller]
    fn test_same(source_text: &str, expected: &str) {
        let expected = format_expected_code(expected);
        let result = transform(source_text, false).unwrap().code;
        if result != expected {
            let diff = TextDiff::from_lines(&expected, &result);
            print_diff_in_terminal(&diff);
            panic!("Expected code does not match the result");
        }
    }

    #[track_caller]
    fn test_same_jsx(source_text: &str, expected: &str) {
        let expected = format_expected_code(expected);
        let result = transform(source_text, true).unwrap().code;
        if result != expected {
            let diff = TextDiff::from_lines(&expected, &result);
            print_diff_in_terminal(&diff);
            panic!("Expected code does not match the result");
        }
    }

    #[track_caller]
    fn test_same_and_deps(source_text: &str, expected: &str, deps: &[&str], dynamic_deps: &[&str]) {
        let expected = format_expected_code(expected);
        let TransformReturn { code, deps: result_deps, dynamic_deps: result_dynamic_deps } =
            transform(source_text, false).unwrap();
        if code != expected {
            let diff = TextDiff::from_lines(&expected, &code);
            print_diff_in_terminal(&diff);
            panic!("Expected code does not match the result");
        }
        for dep in deps {
            assert!(result_deps.contains(*dep));
        }
        for dep in dynamic_deps {
            assert!(result_dynamic_deps.contains(*dep));
        }
    }

    #[test]
    fn default_import() {
        test_same(
            "import foo from 'vue';console.log(foo.bar)",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['default'] });
console.log(__vite_ssr_import_0__.default.bar);",
        );
    }

    #[test]
    fn named_import() {
        test_same(
            "import { ref } from 'vue';function foo() { return ref(0) }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['ref'] });
function foo() {
  return (0, __vite_ssr_import_0__.ref)(0);
}",
        );
    }

    #[test]
    fn named_import_arbitrary_module_namespace() {
        test_same(
            "import { \"some thing\" as ref } from 'vue';function foo() { return ref(0) }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['some thing'] });
function foo() {
  return (0, __vite_ssr_import_0__['some thing'])(0);
}",
        );
    }

    #[test]
    fn namespace_import() {
        test_same(
            "import * as vue from 'vue';function foo() { return vue.ref(0) }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('vue');
function foo() {
  return __vite_ssr_import_0__.ref(0);
}",
        );
    }

    #[test]
    fn export_function_declaration() {
        test_same(
            "export function foo() {}",
            "
Object.defineProperty(__vite_ssr_exports__, 'foo', {
  enumerable: true,
  configurable: true,
  get() {
    return foo;
  }
});
function foo() {}
",
        );
    }

    #[test]
    fn export_class_declaration() {
        test_same(
            "export class foo {}",
            "
Object.defineProperty(__vite_ssr_exports__, 'foo', {
  enumerable: true,
  configurable: true,
  get() {
    return foo;
  }
});
class foo {}
",
        );
    }

    #[test]
    fn export_var_declaration() {
        test_same(
            "export const a = 1, b = 2",
            "
Object.defineProperty(__vite_ssr_exports__, 'a', {
  enumerable: true,
  configurable: true,
  get() {
    return a;
  }
});
Object.defineProperty(__vite_ssr_exports__, 'b', {
  enumerable: true,
  configurable: true,
  get() {
    return b;
  }
});
const a = 1, b = 2;
",
        );
    }

    #[test]
    fn export_named() {
        test_same(
            "const a = 1, b = 2; export { a, b as c }",
            "
Object.defineProperty(__vite_ssr_exports__, 'a', {
  enumerable: true,
  configurable: true,
  get() {
    return a;
  }
});
Object.defineProperty(__vite_ssr_exports__, 'c', {
  enumerable: true,
  configurable: true,
  get() {
    return b;
  }
});
const a = 1, b = 2;
",
        );
    }

    #[test]
    fn export_named_from() {
        test_same(
            "export { ref, computed as c } from 'vue'",
            "
Object.defineProperty(__vite_ssr_exports__, 'ref', {
  enumerable: true,
  configurable: true,
  get() {
    return __vite_ssr_import_0__.ref;
  }
});
Object.defineProperty(__vite_ssr_exports__, 'c', {
  enumerable: true,
  configurable: true,
  get() {
    return __vite_ssr_import_0__.computed;
  }
});
const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['ref', 'computed'] });
",
        );
    }

    #[test]
    fn export_named_imported_binding() {
        test_same(
            "
            import { createApp } from 'vue';
            export { createApp }
            ",
            "
            Object.defineProperty(__vite_ssr_exports__, 'createApp', {
              enumerable: true,
              configurable: true,
              get() {
                return __vite_ssr_import_0__.createApp;
              }
            });
            const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['createApp'] });
            ",
        );
    }

    #[test]
    fn export_star_from() {
        test_same(
            "export * from 'vue'\nexport * from 'react'",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('vue');
__vite_ssr_exportAll__(__vite_ssr_import_0__);
const __vite_ssr_import_1__ = await __vite_ssr_import__('react');
__vite_ssr_exportAll__(__vite_ssr_import_1__);",
        );
    }

    #[test]
    fn export_star_as_from() {
        test_same(
            "export * as foo from 'vue'",
            "
Object.defineProperty(__vite_ssr_exports__, 'foo', {
  enumerable: true,
  configurable: true,
  get() {
    return __vite_ssr_import_0__;
  }
});
const __vite_ssr_import_0__ = await __vite_ssr_import__('vue');
",
        );
    }

    #[test]
    fn re_export_by_imported_name() {
        test_same(
            "import * as foo from 'foo'; export * as foo from 'foo'",
            "
Object.defineProperty(__vite_ssr_exports__, 'foo', {
  enumerable: true,
  configurable: true,
  get() {
    return __vite_ssr_import_1__;
  }
});
const __vite_ssr_import_0__ = await __vite_ssr_import__('foo');
const __vite_ssr_import_1__ = await __vite_ssr_import__('foo');
",
        );

        test_same(
            "import { foo } from 'foo'; export { foo } from 'foo'",
            "
Object.defineProperty(__vite_ssr_exports__, 'foo', {
  enumerable: true,
  configurable: true,
  get() {
    return __vite_ssr_import_1__.foo;
  }
});
const __vite_ssr_import_0__ = await __vite_ssr_import__('foo', { importedNames: ['foo'] });
const __vite_ssr_import_1__ = await __vite_ssr_import__('foo', { importedNames: ['foo'] });
",
        );
    }

    #[test]
    fn export_arbitrary_namespace() {
        test_same(
            "export * as \"arbitrary string\" from 'vue'",
            "
Object.defineProperty(__vite_ssr_exports__, 'arbitrary string', {
  enumerable: true,
  configurable: true,
  get() {
    return __vite_ssr_import_0__;
  }
});
const __vite_ssr_import_0__ = await __vite_ssr_import__('vue');
",
        );

        test_same(
            "const something = \"Something\"; export { something as \"arbitrary string\" }",
            "
Object.defineProperty(__vite_ssr_exports__, 'arbitrary string', {
  enumerable: true,
  configurable: true,
  get() {
    return something;
  }
});
const something = 'Something';
",
        );

        test_same(
            "export { \"arbitrary string2\" as \"arbitrary string\" } from 'vue'",
            "
Object.defineProperty(__vite_ssr_exports__, 'arbitrary string', {
  enumerable: true,
  configurable: true,
  get() {
    return __vite_ssr_import_0__['arbitrary string2'];
  }
});
const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['arbitrary string2'] });
",
        );
    }

    #[test]
    fn export_default() {
        test_same(
            "export default {}",
            "
Object.defineProperty(__vite_ssr_exports__, 'default', {
       enumerable: true,
       configurable: true,
       get() {
               return __vite_ssr_export_default__;
       }
});
const __vite_ssr_export_default__ = {};
",
        );
    }

    #[test]
    fn export_then_import_minified() {
        test_same(
            "export * from 'vue';import {createApp} from 'vue'",
            "
const __vite_ssr_import_0__ = await __vite_ssr_import__('vue');
__vite_ssr_exportAll__(__vite_ssr_import_0__);
const __vite_ssr_import_1__ = await __vite_ssr_import__('vue', { importedNames: ['createApp'] });
",
        );
    }

    #[test]
    fn hoist_import_to_top() {
        test_same(
            "path.resolve('server.js');import path from 'node:path';",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('node:path', { importedNames: ['default'] });
__vite_ssr_import_0__.default.resolve('server.js');",
        );
    }

    #[test]
    fn import_meta() {
        test_same("console.log(import.meta.url)", "console.log(__vite_ssr_import_meta__.url);");
    }

    #[test]
    fn dynamic_import() {
        test_same_and_deps(
            "export const i = () => import('./foo')",
            "
Object.defineProperty(__vite_ssr_exports__, 'i', {
  enumerable: true,
  configurable: true,
  get() {
    return i;
  }
});
const i = () => __vite_ssr_dynamic_import__('./foo');
",
            &[],
            &["./foo"],
        );
    }

    #[test]
    fn do_not_rewrite_method_definition() {
        test_same_and_deps(
            "import { fn } from 'vue';class A { fn() { fn() } }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['fn'] });
            class A {
              fn() {
                (0, __vite_ssr_import_0__.fn)();
              }
            }",
            &["vue"],
            &[],
        );
    }

    #[test]
    fn do_not_rewrite_when_variable_in_scope() {
        test_same_and_deps(
            "import { fn } from 'vue';function A(){ const fn = () => {}; return { fn }; }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['fn'] });
            function A() {
              const fn = () => {};
              return { fn };
            }",
            &["vue"],
            &[]
        );
    }

    // <https://github.com/vitejs/vite/issues/5472>
    #[test]
    fn do_not_rewrite_destructuring_object() {
        test_same_and_deps(
            "import { fn } from 'vue';function A(){ let {fn, test} = {fn: 'foo', test: 'bar'}; return { fn }; }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['fn'] });
            function A() {
              let { fn, test } = {
                fn: 'foo',
                test: 'bar'
              };
              return { fn };
            }",
            &["vue"],
            &[]
        );
    }

    // <https://github.com/vitejs/vite/issues/5472>
    #[test]
    fn do_not_rewrite_destructuring_array() {
        test_same_and_deps(
            "import { fn } from 'vue';function A(){ let [fn, test] = ['foo', 'bar']; return { fn }; }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['fn'] });
            function A() {
              let [fn, test] = ['foo', 'bar'];
              return { fn };
            }",
            &["vue"],
            &[]
        );
    }

    // <https://github.com/vitejs/vite/issues/5727>
    #[test]
    fn rewrite_vars_in_string_interpolation_in_function_args() {
        test_same_and_deps(
            "import { fn } from 'vue';function A({foo = `test${fn}`} = {}){ return {}; }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['fn'] });
            function A({ foo = `test${__vite_ssr_import_0__.fn}` } = {}) {
              return {};
            }",
            &["vue"],
            &[]
        );
    }

    // <https://github.com/vitejs/vite/issues/6520>
    #[test]
    fn rewrite_vars_in_default_value_of_destructure_params() {
        test_same_and_deps(
            "import { fn } from 'vue';function A({foo = fn}){ return {}; }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['fn'] });
            function A({ foo = __vite_ssr_import_0__.fn }) {
              return {};
            }",
            &["vue"],
            &[]
        );
    }

    #[test]
    fn do_not_rewrite_when_function_declaration_is_in_scope() {
        test_same_and_deps(
          "import { fn } from 'vue';function A(){ function fn() {}; return { fn }; }",
          "const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['fn'] });
          function A() {
            function fn() {};
            return { fn };
          }",
          &["vue"],
          &[]
       );
    }

    // <https://github.com/vitejs/vite/issues/16552>
    #[test]
    fn do_not_rewrite_when_function_expression_in_scope() {
        test_same(
            "import {fn} from './vue';var a = function() { return function fn() { console.log(fn) } }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('./vue', { importedNames: ['fn'] });
            var a = function() {
              return function fn() {
            console.log(fn);
              };
            };",
        );
    }

    // <https://github.com/vitejs/vite/issues/16452>
    #[test]
    fn do_not_rewrite_when_function_expression_in_global_scope() {
        test_same(
            "import {fn} from './vue';foo(function fn(a = fn) { console.log(fn) })",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('./vue', { importedNames: ['fn'] });
            foo(function fn(a = fn) {
              console.log(fn);
            });",
        );
    }

    #[test]
    fn do_not_rewrite_when_class_declaration_is_in_scope() {
        test_same_and_deps(
            "import { cls } from 'vue';function A(){ class cls {} return { cls }; }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['cls'] });
            function A() {
              class cls {}
              return { cls };
            }",
            &["vue"],
            &[]
        );
    }

    #[test]
    fn do_not_rewrite_when_class_expression_in_scope() {
        test_same(
            "import { cls } from './vue';var a = function() { return class cls { constructor() { console.log(cls) } } }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('./vue', { importedNames: ['cls'] });
            var a = function() {
              return class cls {
            constructor() {
              console.log(cls);
            }
              };
            };",
        );
    }

    #[test]
    fn do_not_rewrite_when_class_expression_in_global_scope() {
        test_same(
            "import { cls } from './vue';foo(class cls { constructor() { console.log(cls) } })",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('./vue', { importedNames: ['cls'] });
            foo(class cls {
              constructor() {
            console.log(cls);
              }
            });",
        );
    }

    #[test]
    fn do_not_rewrite_catch_clause() {
        test_same_and_deps(
            "import {error} from './dependency';try {} catch(error) {}",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('./dependency', { importedNames: ['error'] });
try {} catch (error) {}",
            &["./dependency"],
            &[]
        );
    }

    #[test]
    fn should_declare_imported_super_class() {
        test_same(
            "import { Foo } from './dependency'; class A extends Foo {}",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('./dependency', { importedNames: ['Foo'] });
class A extends __vite_ssr_import_0__.Foo {}",
        );
    }

    #[test]
    fn should_declare_exported_super_class() {
        test_same(
            "
            import { Foo } from './dependency';
            export default class A extends Foo {}
            export class B extends Foo {}
            ",
            "
            Object.defineProperty(__vite_ssr_exports__, 'default', {
              enumerable: true,
              configurable: true,
              get() {
                return A;
              }
            });
            Object.defineProperty(__vite_ssr_exports__, 'B', {
              enumerable: true,
              configurable: true,
              get() {
                return B;
              }
            });
            const __vite_ssr_import_0__ = await __vite_ssr_import__('./dependency', { importedNames: ['Foo'] });
            class A extends __vite_ssr_import_0__.Foo {}
            class B extends __vite_ssr_import_0__.Foo {}
            ",
        );
    }

    /// <https://github.com/vitejs/vite/issues/4049>
    #[test]
    fn should_handle_default_export_variants() {
        // default anonymous functions
        test_same(
            "export default function() {}",
            "
Object.defineProperty(__vite_ssr_exports__, 'default', {
       enumerable: true,
       configurable: true,
       get() {
               return __vite_ssr_export_default__;
       }
});
const __vite_ssr_export_default__ = function() {};
",
        );

        // default anonymous class
        test_same(
            "export default class {}",
            "
Object.defineProperty(__vite_ssr_exports__, 'default', {
       enumerable: true,
       configurable: true,
       get() {
               return __vite_ssr_export_default__;
       }
});
const __vite_ssr_export_default__ = class {};
",
        );

        // default named functions
        test_same(
            "
export default function foo() {}
foo.prototype = Object.prototype;
",
            "
Object.defineProperty(__vite_ssr_exports__, 'default', {
enumerable: true,
configurable: true,
get() {
return foo;
}
});
function foo() {}
foo.prototype = Object.prototype;
",
        );

        // default named classes
        test_same(
            "export default class A {}\nexport class B extends A {}",
            "
Object.defineProperty(__vite_ssr_exports__, 'default', {
  enumerable: true,
  configurable: true,
  get() {
    return A;
  }
});
Object.defineProperty(__vite_ssr_exports__, 'B', {
  enumerable: true,
  configurable: true,
  get() {
    return B;
  }
});
class A {}
class B extends A {}
",
        );
    }

    #[test]
    fn overwrite_bindings() {
        test_same(
            "import { inject } from 'vue';
            const a = { inject }
            const b = { test: inject }
            function c() { const { test: inject } = { test: true }; console.log(inject) }
            const d = inject
            function f() { console.log(inject) }
            function e() { const { inject } = { inject: true } }
            function g() { const f = () => { const inject = true }; console.log(inject) }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['inject'] });
const a = { inject: __vite_ssr_import_0__.inject };
const b = { test: __vite_ssr_import_0__.inject };
function c() {
  const { test: inject } = { test: true };
  console.log(inject);
}
const d = __vite_ssr_import_0__.inject;
function f() {
  console.log(__vite_ssr_import_0__.inject);
}
function e() {
  const { inject } = { inject: true };
}
function g() {
  const f = () => {
    const inject = true;
  };
  console.log(__vite_ssr_import_0__.inject);
}",
        );
    }

    #[test]
    fn empty_array_pattern() {
        test_same("const [, LHS, RHS] = inMatch;", "const [, LHS, RHS] = inMatch;");
    }

    #[test]
    fn function_argument_destructure() {
        test_same(
            "
    import { foo, bar } from 'foo'
    const a = ({ _ = foo() }) => {}
    function b({ _ = bar() }) {}
    function c({ _ = bar() + foo() }) {}",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('foo', { importedNames: ['foo', 'bar'] });
const a = ({ _ = (0, __vite_ssr_import_0__.foo)() }) => {};
function b({ _ = (0, __vite_ssr_import_0__.bar)() }) {}
function c({ _ = (0, __vite_ssr_import_0__.bar)() + (0, __vite_ssr_import_0__.foo)() }) {}",
        );
    }

    #[test]
    fn object_destructure_alias() {
        test_same(
            "
    import { n } from 'foo'
    const a = () => {
        const { type: n = 'bar' } = {}
        console.log(n)
    }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('foo', { importedNames: ['n'] });
const a = () => {
  const { type: n = 'bar' } = {};
  console.log(n);
};",
        );

        // https://github.com/vitejs/vite/issues/9585
        test_same(
            "
    import { n, m } from 'foo'
    const foo = {}
    {
      const { [n]: m } = foo
    }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('foo', { importedNames: ['n', 'm'] });
const foo = {};
{
  const { [__vite_ssr_import_0__.n]: m } = foo;
}",
        );
    }

    #[test]
    fn nested_object_destructure_alias() {
        test_same(
            "import { remove, add, get, set, rest, objRest } from 'vue'

    function a() {
        const {
            o: { remove },
            a: { b: { c: [ add ] }},
            d: [{ get }, set, ...rest],
            ...objRest
        } = foo

        remove()
        add()
        get()
        set()
        rest()
        objRest()
    }

    remove()
    add()
    get()
    set()
    rest()
    objRest()",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['remove', 'add', 'get', 'set', 'rest', 'objRest'] });

function a() {
  const {
    o: { remove },
    a: { b: { c: [ add ] }},
    d: [{ get }, set, ...rest],
    ...objRest
  } = foo;

  remove();
  add();
  get();
  set();
  rest();
  objRest();
}

(0, __vite_ssr_import_0__.remove)();
(0, __vite_ssr_import_0__.add)();
(0, __vite_ssr_import_0__.get)();
(0, __vite_ssr_import_0__.set)();
(0, __vite_ssr_import_0__.rest)();
(0, __vite_ssr_import_0__.objRest)();
",
        );
    }

    #[test]
    fn object_props_and_methods() {
        test_same(
            "import foo from 'foo'

    const bar = 'bar'

    const obj = {
        foo() {},
        [foo]() {},
        [bar]() {},
        foo: () => {},
        [foo]: () => {},
        [bar]: () => {},
        bar(foo) {}
    }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('foo', { importedNames: ['default'] });

const bar = 'bar';

const obj = {
  foo() {},
  [__vite_ssr_import_0__.default]() {},
  [bar]() {},
  foo: () => {},
  [__vite_ssr_import_0__.default]: () => {},
  [bar]: () => {},
  bar(foo) {}
};",
        );
    }

    #[test]
    fn class_props() {
        test_same(
            "import { remove, add } from 'vue'
            class A {
                remove = 1
                add = null
            }",
            "
            const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['remove', 'add'] });
            class A {
              remove = 1;
              add = null;
            }
            ",
        );
    }

    #[test]
    fn class_methods() {
        test_same(
            "import foo from 'foo'

    const bar = 'bar'

    class A {
        foo() {}
        [foo]() {}
        [bar]() {}
        #foo() {}
        bar(foo) {}
    }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('foo', { importedNames: ['default'] });

const bar = 'bar';

class A {
  foo() {}
  [__vite_ssr_import_0__.default]() {}
  [bar]() {}
  #foo() {}
  bar(foo) {}
}",
        );
    }

    #[test]
    fn declare_scope() {
        test_same(
            "import { aaa, bbb, ccc, ddd } from 'vue'

    function foobar() {
        ddd()

        const aaa = () => {
            bbb(ccc)
            ddd()
        }
        const bbb = () => {
            console.log('hi')
        }
        const ccc = 1
        function ddd() {}

        aaa()
        bbb()
        ccc()
    }

    aaa()
    bbb()",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('vue', { importedNames: ['aaa', 'bbb', 'ccc', 'ddd'] });

function foobar() {
  ddd();

  const aaa = () => {
    bbb(ccc);
    ddd();
  };
  const bbb = () => {
    console.log('hi');
  };
  const ccc = 1;
  function ddd() {}

  aaa();
  bbb();
  ccc();
}

(0, __vite_ssr_import_0__.aaa)();
(0, __vite_ssr_import_0__.bbb)();
",
        );
    }

    #[test]
    fn jsx() {
        test_same_jsx(
            "import React from 'react'
    import { Foo, Slot } from 'foo'

    function Bar({ Slot = <Foo /> }) {
      return (
        <>
          <Slot />
        </>
      )
    }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('react', { importedNames: ['default'] });
const __vite_ssr_import_1__ = await __vite_ssr_import__('foo', { importedNames: ['Foo', 'Slot'] });
function Bar({ Slot = /* @__PURE__ */ __vite_ssr_import_0__.default.createElement(__vite_ssr_import_1__.Foo, null) }) {
  return /* @__PURE__ */ __vite_ssr_import_0__.default.createElement(__vite_ssr_import_0__.default.Fragment, null, /* @__PURE__ */ __vite_ssr_import_0__.default.createElement(Slot, null));
}",
        );
    }

    #[test]
    fn continuous_exports() {
        test_same(
            "export function fn1() {
    }export function fn2() {
    }",
            "
Object.defineProperty(__vite_ssr_exports__, 'fn1', {
  enumerable: true,
  configurable: true,
  get() {
    return fn1;
  }
});
Object.defineProperty(__vite_ssr_exports__, 'fn2', {
  enumerable: true,
  configurable: true,
  get() {
    return fn2;
  }
});
function fn1() {}
function fn2() {}
",
        );
    }

    // https://github.com/vitest-dev/vitest/issues/1141
    #[test]
    fn export_default_expression() {
        // esbuild transform result of following TS code
        // export default <MyFn> function getRandom() {
        //   return Math.random()
        // }
        test_same(
            "
export default (function getRandom() {
  return Math.random();
});
",
            "
Object.defineProperty(__vite_ssr_exports__, 'default', {
       enumerable: true,
       configurable: true,
       get() {
               return __vite_ssr_export_default__;
       }
});
const __vite_ssr_export_default__ = (function getRandom() {
  return Math.random();
});
",
        );

        test_same(
            "export default (class A {});",
            "
Object.defineProperty(__vite_ssr_exports__, 'default', {
       enumerable: true,
       configurable: true,
       get() {
               return __vite_ssr_export_default__;
       }
});
const __vite_ssr_export_default__ = class A {};",
        );
    }

    // https://github.com/vitejs/vite/issues/8002
    #[test]
    fn with_hashbang() {
        test_same(
            "#!/usr/bin/env node
    console.log(\"it can parse the hashbang\")",
            "#!/usr/bin/env node
console.log(\"it can parse the hashbang\");",
        );
    }

    #[test]
    fn import_hoisted_after_hashbang() {
        test_same(
            "#!/usr/bin/env node
    console.log(foo);
    import foo from 'foo'",
            "#!/usr/bin/env node
            const __vite_ssr_import_0__ = await __vite_ssr_import__('foo', { importedNames: ['default'] });
            console.log(__vite_ssr_import_0__.default);
            ",
        );
    }

    #[test]
    fn identity_function_helper_injected_after_hashbang() {
        test_same(
            "#!/usr/bin/env node
            import { foo } from 'foo';
            foo();",
            "#!/usr/bin/env node
            const __vite_ssr_import_0__ = await __vite_ssr_import__('foo', { importedNames: ['foo'] });
            (0, __vite_ssr_import_0__.foo)();
            ",
        );
    }

    // https://github.com/vitejs/vite/issues/10289
    #[test]
    fn track_scope_by_class_function_condition_blocks() {
        test_same(
            "import { foo, bar } from 'foobar'
    if (false) {
      const foo = 'foo'
      console.log(foo)
    } else if (false) {
      const [bar] = ['bar']
      console.log(bar)
    } else {
      console.log(foo)
      console.log(bar)
    }
    export class Test {
      constructor() {
        if (false) {
          const foo = 'foo'
          console.log(foo)
        } else if (false) {
          const [bar] = ['bar']
          console.log(bar)
        } else {
          console.log(foo)
          console.log(bar)
        }
      }
    }",
            "
Object.defineProperty(__vite_ssr_exports__, 'Test', {
  enumerable: true,
  configurable: true,
  get() {
    return Test;
  }
});
const __vite_ssr_import_0__ = await __vite_ssr_import__('foobar', { importedNames: ['foo', 'bar'] });
if (false) {
  const foo = 'foo';
  console.log(foo);
} else if (false) {
  const [bar] = ['bar'];
  console.log(bar);
} else {
  console.log(__vite_ssr_import_0__.foo);
  console.log(__vite_ssr_import_0__.bar);
}
class Test {
  constructor() {
    if (false) {
      const foo = 'foo';
      console.log(foo);
    } else if (false) {
      const [bar] = ['bar'];
      console.log(bar);
    } else {
      console.log(__vite_ssr_import_0__.foo);
      console.log(__vite_ssr_import_0__.bar);
    }
  }
}
",
        );
    }

    // https://github.com/vitejs/vite/issues/10386
    #[test]
    fn track_var_scope_by_function() {
        test_same(
            "import { foo, bar } from 'foobar'
    function test() {
      if (true) {
        var foo = () => { var why = 'would' }, bar = 'someone'
      }
      return [foo, bar]
    }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('foobar', { importedNames: ['foo', 'bar'] });
function test() {
  if (true) {
    var foo = () => { var why = 'would'; }, bar = 'someone';
  }
  return [foo, bar];
}",
        );
    }

    // https://github.com/vitejs/vite/issues/11806
    #[test]
    fn track_scope_by_blocks() {
        test_same(
            "import { foo, bar, baz } from 'foobar'
    function test() {
      [foo];
      {
        let foo = 10;
        let bar = 10;
      }
      try {} catch (baz){ baz }
      return bar;
    }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('foobar', { importedNames: ['foo', 'bar', 'baz'] });
function test() {
  [__vite_ssr_import_0__.foo];
  {
    let foo = 10;
    let bar = 10;
  }
  try {} catch (baz) { baz; }
  return __vite_ssr_import_0__.bar;
}",
        );
    }

    #[test]
    fn track_scope_in_for_loops() {
        test_same(
            "import { test } from './test.js'

    for (const test of tests) {
      console.log(test)
    }

    for (let test = 0; test < 10; test++) {
      console.log(test)
    }

    for (const test in tests) {
      console.log(test)
    }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('./test.js', { importedNames: ['test'] });
for (const test of tests) {
  console.log(test);
}
for (let test = 0; test < 10; test++) {
  console.log(test);
}
for (const test in tests) {
  console.log(test);
}",
        );
    }

    #[test]
    fn avoid_binding_class_expression() {
        test_same(
            "import Foo, { Bar } from './foo';

    console.log(Foo, Bar);
    const obj = {
      foo: class Foo {},
      bar: class Bar {}
    }
    const Baz = class extends Foo {}",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('./foo', { importedNames: ['default', 'Bar'] });
console.log(__vite_ssr_import_0__.default, __vite_ssr_import_0__.Bar);
const obj = {
  foo: class Foo {},
  bar: class Bar {}
};
const Baz = class extends __vite_ssr_import_0__.default {};",
        );
    }

    #[test]
    fn import_assertion_attribute() {
        test_same(
            "import * as foo from './foo.json' with { type: 'json' };
    import('./bar.json', { with: { type: 'json' } });",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('./foo.json');
__vite_ssr_dynamic_import__('./bar.json', { with: { type: 'json' } });",
        );
    }

    #[test]
    fn import_and_export_ordering() {
        // Given all imported modules logs `mod ${mod}` on execution,
        // and `foo` is `bar`, the logging order should be:
        // "mod a", "mod foo", "mod b", "bar1", "bar2"
        test_same(
            "
    console.log(foo + 1)
    export * from './a'
    import { foo } from './foo'
    export * from './b'
    console.log(foo + 2)",
            "
const __vite_ssr_import_0__ = await __vite_ssr_import__('./a');
__vite_ssr_exportAll__(__vite_ssr_import_0__);
const __vite_ssr_import_1__ = await __vite_ssr_import__('./foo', { importedNames: ['foo']});
const __vite_ssr_import_2__ = await __vite_ssr_import__('./b');
__vite_ssr_exportAll__(__vite_ssr_import_2__);
console.log(__vite_ssr_import_1__.foo + 1);
console.log(__vite_ssr_import_1__.foo + 2);",
        );
    }

    #[test]
    fn identity_function_is_declared_before_used() {
        test_same(
            "
    import { foo } from './foo'
    export default foo()
    export * as bar from './bar'
    console.log(bar)
",
            "
Object.defineProperty(__vite_ssr_exports__, 'default', {
       enumerable: true,
       configurable: true,
       get() {
               return __vite_ssr_export_default__;
       }
});
Object.defineProperty(__vite_ssr_exports__, 'bar', {
  enumerable: true,
  configurable: true,
  get() {
    return __vite_ssr_import_1__;
  }
});
const __vite_ssr_import_0__ = await __vite_ssr_import__('./foo', { importedNames: ['foo'] });
const __vite_ssr_import_1__ = await __vite_ssr_import__('./bar');
const __vite_ssr_export_default__ = (0, __vite_ssr_import_0__.foo)();
console.log(bar);
",
        );
    }

    #[test]
    fn inject_semicolon_call_wrapper() {
        let code = "import { f } from './f'

    let x = 0;

    x
    f()

    if (1)
      x
    f()

    if (1)
      x
    else
      x
    f()


    let y = x
    f()

    x /*;;*/ /*;;*/
    f()

    function z() {
      x
      f()

      if (1) {
        x
        f()
      }
    }

    let a = {}
    f()

    let b = () => {}
    f()

    function c() {
    }
    f()

    class D {
    }
    f()

    {
      x
    }
    f()

    switch (1) {
      case 1:
        x
        f()
        break
    }

    if(0){}f()

    if(0){}else{}f()

    switch(1){}f()

    {}f(1)";

        let expected = "

    const __vite_ssr_import_0__ = await __vite_ssr_import__('./f', { importedNames: ['f'] });

    let x = 0;

    x;
    (0,__vite_ssr_import_0__.f)();

    if (1)
      x;
    (0,__vite_ssr_import_0__.f)();

    if (1)
      x
    else
      x;
    (0,__vite_ssr_import_0__.f)();


    let y = x;
    (0,__vite_ssr_import_0__.f)();

    x; /*;;*/ /*;;*/
    (0,__vite_ssr_import_0__.f)();

    function z() {
      x;
      (0,__vite_ssr_import_0__.f)();

      if (1) {
        x;
        (0,__vite_ssr_import_0__.f)()
      }
    }

    let a = {};
    (0,__vite_ssr_import_0__.f)();

    let b = () => {};
    (0,__vite_ssr_import_0__.f)();

    function c() {
    }
    (0,__vite_ssr_import_0__.f)();

    class D {
    }
    (0,__vite_ssr_import_0__.f)();

    {
      x
    }
    (0,__vite_ssr_import_0__.f)();

    switch (1) {
      case 1:
        x;
        (0,__vite_ssr_import_0__.f)();
        break
    }

    if(0){}(0,__vite_ssr_import_0__.f)();

    if(0){}else{}(0,__vite_ssr_import_0__.f)();

    switch(1){}(0,__vite_ssr_import_0__.f)();

    {}(0,__vite_ssr_import_0__.f)(1)
    ";
        test_same(code, expected);
    }

    #[test]
    fn does_not_break_minified_code() {
        // Based on https://unpkg.com/@headlessui/vue@1.7.23/dist/components/transitions/transition.js
        test_same(
            "import O from 'a';
    const c = () => {
      if(true){return}O(1,{})
    }",
            "const __vite_ssr_import_0__ = await __vite_ssr_import__('a', { importedNames: ['default'] });
const c = () => {
  if (true) {
    return;
  }
  (0, __vite_ssr_import_0__.default)(1, {});
};",
        );
    }

    #[test]
    fn deps() {
        let code = r#"
        import a from "a";
        export { b } from "b";
        export * from "c";
        export * as d from "d";
        import("e")
        export * as A from "a";
        "#;
        let expected = "
        Object.defineProperty(__vite_ssr_exports__, 'b', {
          enumerable: true,
          configurable: true,
          get() {
                  return __vite_ssr_import_1__.b;
          }
        });
        Object.defineProperty(__vite_ssr_exports__, 'd', {
          enumerable: true,
          configurable: true,
          get() {
                  return __vite_ssr_import_3__;
          }
        });
        Object.defineProperty(__vite_ssr_exports__, 'A', {
               enumerable: true,
               configurable: true,
               get() {
                       return __vite_ssr_import_4__;
               }
        });
        const __vite_ssr_import_0__ = await __vite_ssr_import__('a', { importedNames: ['default'] });
        const __vite_ssr_import_1__ = await __vite_ssr_import__('b', { importedNames: ['b'] });
        const __vite_ssr_import_2__ = await __vite_ssr_import__('c');
        __vite_ssr_exportAll__(__vite_ssr_import_2__);
        const __vite_ssr_import_3__ = await __vite_ssr_import__('d');
        const __vite_ssr_import_4__ = await __vite_ssr_import__('a');
        __vite_ssr_dynamic_import__('e');
        ";
        test_same_and_deps(code, expected, &["a", "b", "c", "d"], &["e"]);
    }
}
