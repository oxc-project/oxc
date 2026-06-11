//! Surface extraction (pass A): lower a declaration program into an owned,
//! `Send + Sync` [`FileSurface`].
//!
//! For `.ts`/`.tsx` files the input is the output of the forced
//! `IsolatedDeclarations` transform; for `.d.ts` files it is the parsed program
//! itself. Either way the input is declaration-shaped, which is exactly why no
//! other file is needed: name references lower to *pending* refs that the
//! linker resolves once every file is known.

use oxc_ast::ast::{
    BindingPattern, Class, Declaration, ExportDefaultDeclarationKind, Expression, Function,
    ImportDeclarationSpecifier, Program, Statement, TSEnumDeclaration, TSInterfaceDeclaration,
    TSModuleDeclaration, TSModuleDeclarationName, TSTypeAliasDeclaration, UnaryOperator,
    VariableDeclarator,
};
use oxc_span::Span;
use rustc_hash::FxHashMap;

use crate::{
    ir::{INTRINSIC_COUNT, RefTarget, Type, TypeId, TypeTable},
    lower::{Lowerer, NameTarget, ResolveName, TypeSink},
};

/// What a surface declaration is (mirrors [`crate::ir::SymbolKind`], with
/// type ids local to the surface).
#[derive(Debug)]
pub enum SurfaceDeclKind {
    /// `declare const` / `declare function` — a value.
    Value {
        /// Declared (or initializer-inferred) type, surface-local id.
        ty: TypeId,
    },
    /// `type X = ...`
    TypeAlias {
        /// Aliased type, surface-local id.
        ty: TypeId,
        /// Declared type parameters.
        params: Vec<SurfaceTypeParam>,
    },
    /// `interface X { ... }`
    Interface {
        /// Body shape, surface-local id.
        ty: TypeId,
        /// Declared type parameters.
        params: Vec<SurfaceTypeParam>,
        /// Lowered `extends` heritage references, flattened by the linker.
        extends: Vec<TypeId>,
    },
    /// `class X` — carries its instance shape (surface-local id).
    Class {
        /// Instance type (an object shape).
        instance: TypeId,
    },
    /// `enum X` — carries its member list.
    Enum {
        /// Declared members in order.
        members: Vec<crate::ir::EnumMemberInfo>,
    },
    /// `namespace X` — opaque in v0.
    Namespace,
}

/// A declared type parameter (surface-local type ids).
#[derive(Debug)]
pub struct SurfaceTypeParam {
    /// Parameter name.
    pub name: Box<str>,
    /// `extends` constraint.
    pub constraint: Option<TypeId>,
    /// Default type.
    pub default: Option<TypeId>,
}

/// A top-level declaration in the surface.
#[derive(Debug)]
pub struct SurfaceDecl {
    /// Declared name.
    pub name: Box<str>,
    /// Span of the declaration name.
    pub span: Span,
    /// What it is.
    pub kind: SurfaceDeclKind,
}

/// How an import binding maps to the target module.
#[derive(Debug, Clone)]
pub enum ImportBindingKind {
    /// `import { imported as local }`.
    Named(Box<str>),
    /// `import local from`.
    Default,
    /// `import * as local from`.
    Namespace,
}

/// One local binding introduced by an import.
#[derive(Debug)]
pub struct ImportBinding {
    /// Local name.
    pub local: Box<str>,
    /// What it binds in the target module.
    pub kind: ImportBindingKind,
}

/// An `import ... from "specifier"` in the surface.
#[derive(Debug)]
pub struct SurfaceImport {
    /// The module specifier text.
    pub specifier: Box<str>,
    /// Bindings introduced (empty for side-effect imports).
    pub bindings: Vec<ImportBinding>,
}

/// An export entry of the surface.
#[derive(Debug)]
pub enum SurfaceExport {
    /// An exported declaration: `export const x` / `export default function f`.
    Decl {
        /// Index into [`FileSurface::decls`].
        decl: u32,
        /// Exported name (the declaration name, or `"default"`).
        name: Box<str>,
    },
    /// `export { local as exported }` — `local` resolves to a declaration or
    /// an import binding.
    Named {
        /// Local name being exported.
        local: Box<str>,
        /// Exported name.
        exported: Box<str>,
    },
}

/// A re-export with a module specifier.
#[derive(Debug)]
pub enum SurfaceReexport {
    /// `export { imported as exported } from "specifier"`.
    Named {
        /// The module specifier text.
        specifier: Box<str>,
        /// Name in the target module.
        imported: Box<str>,
        /// Exported name here.
        exported: Box<str>,
    },
    /// `export * from "specifier"`.
    Star {
        /// The module specifier text.
        specifier: Box<str>,
    },
    /// `export * as exported from "specifier"` — opaque namespace in v0
    /// (the target module is not modeled, so the specifier is not stored).
    StarAs {
        /// Exported name here.
        exported: Box<str>,
    },
}

/// The owned, thread-safe surface of one file.
#[derive(Debug, Default)]
pub struct FileSurface {
    /// Top-level declarations.
    pub decls: Vec<SurfaceDecl>,
    /// Surface-local types; index `i` has id `INTRINSIC_COUNT + i` until the
    /// linker relocates them into the global table.
    pub types: Vec<Type>,
    /// Imports (used by exported types).
    pub imports: Vec<SurfaceImport>,
    /// Export entries.
    pub exports: Vec<SurfaceExport>,
    /// Re-exports.
    pub reexports: Vec<SurfaceReexport>,
    /// `true` when exports cannot be enumerated (`export =`, parse failure).
    /// Imports from an opaque module succeed with type `any`.
    pub opaque_exports: bool,
}

/// Pending declaration collected in the first pass, lowered in the second.
enum DeclItem<'b, 'a> {
    Var(&'b VariableDeclarator<'a>),
    Func(&'b Function<'a>),
    Class(&'b Class<'a>),
    Alias(&'b TSTypeAliasDeclaration<'a>),
    Interface(&'b TSInterfaceDeclaration<'a>),
    Enum(&'b TSEnumDeclaration<'a>),
    Namespace(#[expect(dead_code)] &'b TSModuleDeclaration<'a>),
    /// `export default <expression>` — typed `any` in v0.
    ExpressionDefault,
}

struct DeclWork<'b, 'a> {
    name: Box<str>,
    span: Span,
    item: DeclItem<'b, 'a>,
}

/// Resolves type names against the surface's own declarations and imports;
/// everything else is an unknown global (no lib.d.ts in v0) and stays
/// unresolved.
struct SurfaceResolver<'s> {
    decl_names: &'s FxHashMap<Box<str>, u32>,
    import_bindings: &'s FxHashMap<Box<str>, (u32, ImportBindingKind)>,
}

impl ResolveName for SurfaceResolver<'_> {
    fn resolve(&self, name: &str) -> NameTarget {
        if let Some(&idx) = self.decl_names.get(name) {
            return NameTarget::Ref(RefTarget::PendingLocal(idx));
        }
        if let Some((import, kind)) = self.import_bindings.get(name) {
            return match kind {
                ImportBindingKind::Named(imported) => NameTarget::Ref(RefTarget::PendingImport {
                    import: *import,
                    member: Some(imported.clone()),
                }),
                ImportBindingKind::Default => {
                    NameTarget::Ref(RefTarget::PendingImport { import: *import, member: None })
                }
                // A namespace object used directly in type position.
                ImportBindingKind::Namespace => NameTarget::Ref(RefTarget::Unresolved),
            };
        }
        NameTarget::Ref(RefTarget::Unresolved)
    }

    fn resolve_qualified(&self, left: &str, member: &str, _span: Span) -> NameTarget {
        if let Some((import, ImportBindingKind::Namespace)) = self.import_bindings.get(left) {
            return NameTarget::Ref(RefTarget::PendingImport {
                import: *import,
                member: Some(member.into()),
            });
        }
        NameTarget::Ref(RefTarget::Unresolved)
    }
}

/// Build the surface of a declaration program.
pub fn build_surface(program: &Program<'_>) -> FileSurface {
    let mut surface = FileSurface::default();
    let mut works: Vec<DeclWork<'_, '_>> = Vec::new();
    let mut decl_names: FxHashMap<Box<str>, u32> = FxHashMap::default();
    let mut import_bindings: FxHashMap<Box<str>, (u32, ImportBindingKind)> = FxHashMap::default();

    // Pass 1: collect declarations, imports, exports.
    for stmt in &program.body {
        match stmt {
            Statement::ImportDeclaration(import) => {
                let import_idx = u32::try_from(surface.imports.len()).unwrap();
                let mut bindings = Vec::new();
                if let Some(specifiers) = &import.specifiers {
                    for spec in specifiers {
                        let (local, kind) = match spec {
                            ImportDeclarationSpecifier::ImportSpecifier(named) => (
                                named.local.name.as_str(),
                                ImportBindingKind::Named(named.imported.name().as_str().into()),
                            ),
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                                (default.local.name.as_str(), ImportBindingKind::Default)
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(ns) => {
                                (ns.local.name.as_str(), ImportBindingKind::Namespace)
                            }
                        };
                        import_bindings.insert(local.into(), (import_idx, kind.clone()));
                        bindings.push(ImportBinding { local: local.into(), kind });
                    }
                }
                surface.imports.push(SurfaceImport {
                    specifier: import.source.value.as_str().into(),
                    bindings,
                });
            }
            Statement::ExportNamedDeclaration(export) => {
                if let Some(decl) = &export.declaration {
                    collect_declaration(decl, &mut works, &mut decl_names, &mut surface, true);
                } else if let Some(source) = &export.source {
                    for spec in &export.specifiers {
                        surface.reexports.push(SurfaceReexport::Named {
                            specifier: source.value.as_str().into(),
                            imported: spec.local.name().as_str().into(),
                            exported: spec.exported.name().as_str().into(),
                        });
                    }
                } else {
                    for spec in &export.specifiers {
                        surface.exports.push(SurfaceExport::Named {
                            local: spec.local.name().as_str().into(),
                            exported: spec.exported.name().as_str().into(),
                        });
                    }
                }
            }
            Statement::ExportAllDeclaration(export) => {
                let specifier: Box<str> = export.source.value.as_str().into();
                match &export.exported {
                    Some(name) => surface
                        .reexports
                        .push(SurfaceReexport::StarAs { exported: name.name().as_str().into() }),
                    None => surface.reexports.push(SurfaceReexport::Star { specifier }),
                }
            }
            Statement::ExportDefaultDeclaration(export) => {
                let work = match &export.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(func) => DeclWork {
                        name: func
                            .id
                            .as_ref()
                            .map_or_else(|| "default".into(), |id| id.name.as_str().into()),
                        span: func.id.as_ref().map_or(export.span, |id| id.span),
                        item: DeclItem::Func(func),
                    },
                    ExportDefaultDeclarationKind::ClassDeclaration(class) => DeclWork {
                        name: class
                            .id
                            .as_ref()
                            .map_or_else(|| "default".into(), |id| id.name.as_str().into()),
                        span: class.id.as_ref().map_or(export.span, |id| id.span),
                        item: DeclItem::Class(class),
                    },
                    ExportDefaultDeclarationKind::TSInterfaceDeclaration(iface) => DeclWork {
                        name: iface.id.name.as_str().into(),
                        span: iface.id.span,
                        item: DeclItem::Interface(iface),
                    },
                    _ => DeclWork {
                        name: "default".into(),
                        span: export.span,
                        item: DeclItem::ExpressionDefault,
                    },
                };
                let idx = push_work(work, &mut works, &mut decl_names);
                surface.exports.push(SurfaceExport::Decl { decl: idx, name: "default".into() });
            }
            Statement::TSExportAssignment(_) => {
                // `export =` — exports become opaque in v0.
                surface.opaque_exports = true;
            }
            _ => {
                if let Some(decl) = as_declaration(stmt) {
                    collect_declaration(decl, &mut works, &mut decl_names, &mut surface, false);
                }
            }
        }
    }

    // Pass 2: lower declaration bodies with names resolvable.
    let mut sink = TypeSink::new(INTRINSIC_COUNT);
    let resolver = SurfaceResolver { decl_names: &decl_names, import_bindings: &import_bindings };
    for work in works {
        let kind = match work.item {
            DeclItem::Var(declarator) => {
                let mut lowerer = Lowerer::new(&mut sink, &resolver);
                let ty = match &declarator.type_annotation {
                    Some(annotation) => lowerer.lower_type(&annotation.type_annotation),
                    None => declarator.init.as_ref().map_or(TypeTable::ANY, |init| {
                        infer_literal_initializer(&mut lowerer, init)
                    }),
                };
                SurfaceDeclKind::Value { ty }
            }
            DeclItem::Func(func) => {
                let shape = Lowerer::new(&mut sink, &resolver)
                    .lower_function_shape(&func.params, func.return_type.as_deref());
                let ty = sink.push(Type::Function(Box::new(shape)));
                SurfaceDeclKind::Value { ty }
            }
            DeclItem::Alias(alias) => {
                let params =
                    lower_type_params(&mut sink, &resolver, alias.type_parameters.as_deref());
                let names = params.iter().map(|p| p.name.clone()).collect();
                let ty = Lowerer::with_params(&mut sink, &resolver, names)
                    .lower_type(&alias.type_annotation);
                SurfaceDeclKind::TypeAlias { ty, params }
            }
            DeclItem::Interface(iface) => {
                let params =
                    lower_type_params(&mut sink, &resolver, iface.type_parameters.as_deref());
                let names: Vec<Box<str>> = params.iter().map(|p| p.name.clone()).collect();
                // The body alone; the linker flattens inheritance.
                let ty = Lowerer::with_params(&mut sink, &resolver, names)
                    .lower_members(&iface.body.body, false);
                let extends = iface
                    .extends
                    .iter()
                    .filter_map(|heritage| {
                        use oxc_ast::ast::Expression;
                        if heritage.type_arguments.is_some() {
                            return None; // generic bases unmodeled
                        }
                        let Expression::Identifier(base) = &heritage.expression else {
                            return None;
                        };
                        match resolver.resolve(base.name.as_str()) {
                            NameTarget::Ref(target) => {
                                Some(sink.push(Type::Ref { target, args: Box::new([]) }))
                            }
                            NameTarget::Inline(ty) => Some(ty),
                            NameTarget::EnumMember { .. } => None,
                        }
                    })
                    .collect();
                SurfaceDeclKind::Interface { ty, params, extends }
            }
            DeclItem::Class(class) => {
                let instance = lower_class_instance(&mut sink, &resolver, class);
                SurfaceDeclKind::Class { instance }
            }
            DeclItem::Enum(enum_decl) => {
                SurfaceDeclKind::Enum { members: enum_member_infos(enum_decl) }
            }
            DeclItem::Namespace(_) => SurfaceDeclKind::Namespace,
            DeclItem::ExpressionDefault => SurfaceDeclKind::Value { ty: TypeTable::ANY },
        };
        surface.decls.push(SurfaceDecl { name: work.name, span: work.span, kind });
    }
    surface.types = sink.into_types();
    surface
}

/// Add non-exported enums from the *original* program to a surface built
/// from declaration output (the ID transform drops unreferenced locals, but
/// the checker needs their symbols for member access and nominal relations).
pub fn augment_surface_with_local_enums(surface: &mut FileSurface, program: &Program<'_>) {
    let existing: rustc_hash::FxHashSet<Box<str>> =
        surface.decls.iter().map(|d| d.name.clone()).collect();
    for stmt in &program.body {
        let enum_decl = match stmt {
            Statement::TSEnumDeclaration(e) => e,
            Statement::ExportNamedDeclaration(export) => match &export.declaration {
                Some(Declaration::TSEnumDeclaration(e)) => e,
                _ => continue,
            },
            _ => continue,
        };
        let name: Box<str> = enum_decl.id.name.as_str().into();
        if existing.contains(&name) {
            continue;
        }
        surface.decls.push(SurfaceDecl {
            name,
            span: enum_decl.id.span,
            kind: SurfaceDeclKind::Enum { members: enum_member_infos(enum_decl) },
        });
    }
}

/// Lower a class body to its public instance shape. Private/protected
/// members, index signatures, and `extends` make the shape inexact, as do
/// computed keys; statics and constructors are not instance members.
pub fn lower_class_instance<R: ResolveName>(
    sink: &mut TypeSink,
    resolver: &R,
    class: &Class<'_>,
) -> TypeId {
    use oxc_ast::ast::{ClassElement, MethodDefinitionKind, TSAccessibility};
    let mut members: Vec<crate::ir::Member> = Vec::new();
    let mut inexact = class.super_class.is_some();
    let mut lowerer = Lowerer::new(sink, resolver);
    for element in &class.body.body {
        match element {
            ClassElement::PropertyDefinition(prop) => {
                if prop.r#static {
                    continue;
                }
                if matches!(
                    prop.accessibility,
                    Some(TSAccessibility::Private | TSAccessibility::Protected)
                ) {
                    inexact = true;
                    continue;
                }
                let Some(name) = crate::lower::property_key_name(&prop.key) else {
                    inexact = true;
                    continue;
                };
                let ty = lowerer.lower_annotation(prop.type_annotation.as_deref());
                members.push(crate::ir::Member { name, ty, optional: prop.optional });
            }
            ClassElement::MethodDefinition(method) => {
                if method.r#static || method.kind == MethodDefinitionKind::Constructor {
                    continue;
                }
                if matches!(
                    method.accessibility,
                    Some(TSAccessibility::Private | TSAccessibility::Protected)
                ) {
                    inexact = true;
                    continue;
                }
                let Some(name) = crate::lower::property_key_name(&method.key) else {
                    inexact = true;
                    continue;
                };
                match method.kind {
                    MethodDefinitionKind::Method => {
                        let shape = lowerer.lower_function_shape(
                            &method.value.params,
                            method.value.return_type.as_deref(),
                        );
                        let ty = lowerer.sink.push(Type::Function(Box::new(shape)));
                        members.push(crate::ir::Member { name, ty, optional: false });
                    }
                    MethodDefinitionKind::Get => {
                        let ty = lowerer.lower_annotation(method.value.return_type.as_deref());
                        members.push(crate::ir::Member { name, ty, optional: false });
                    }
                    MethodDefinitionKind::Set => {
                        let ty = method.value.params.items.first().map_or(TypeTable::ANY, |p| {
                            lowerer.lower_annotation(p.type_annotation.as_deref())
                        });
                        members.push(crate::ir::Member { name, ty, optional: false });
                    }
                    MethodDefinitionKind::Constructor => unreachable!(),
                }
            }
            ClassElement::TSIndexSignature(_) | ClassElement::AccessorProperty(_) => {
                inexact = true;
            }
            ClassElement::StaticBlock(_) => {}
        }
    }
    sink.push(Type::Object(crate::ir::ObjectShape { members: members.into_boxed_slice(), inexact }))
}

/// Collect enum member names and value kinds.
pub fn enum_member_infos(enum_decl: &TSEnumDeclaration<'_>) -> Vec<crate::ir::EnumMemberInfo> {
    enum_decl
        .body
        .members
        .iter()
        .map(|member| {
            use oxc_ast::ast::Expression;
            let name: Box<str> = member.id.static_name().as_str().into();
            let is_string = matches!(
                member.initializer,
                Some(Expression::StringLiteral(_) | Expression::TemplateLiteral(_))
            );
            crate::ir::EnumMemberInfo { name, is_string }
        })
        .collect()
}

/// Lower a `<T extends C = D, ...>` declaration, with all parameter names in
/// scope for the constraints/defaults.
pub fn lower_type_params<R: ResolveName>(
    sink: &mut TypeSink,
    resolver: &R,
    decl: Option<&oxc_ast::ast::TSTypeParameterDeclaration<'_>>,
) -> Vec<SurfaceTypeParam> {
    let Some(decl) = decl else { return Vec::new() };
    let names: Vec<Box<str>> = decl.params.iter().map(|p| p.name.name.as_str().into()).collect();
    decl.params
        .iter()
        .map(|p| {
            let mut lowerer = Lowerer::with_params(sink, resolver, names.clone());
            SurfaceTypeParam {
                name: p.name.name.as_str().into(),
                constraint: p.constraint.as_ref().map(|c| lowerer.lower_type(c)),
                default: p.default.as_ref().map(|d| lowerer.lower_type(d)),
            }
        })
        .collect()
}

fn as_declaration<'b, 'a>(stmt: &'b Statement<'a>) -> Option<&'b Declaration<'a>> {
    match stmt {
        Statement::VariableDeclaration(_)
        | Statement::FunctionDeclaration(_)
        | Statement::ClassDeclaration(_)
        | Statement::TSTypeAliasDeclaration(_)
        | Statement::TSInterfaceDeclaration(_)
        | Statement::TSEnumDeclaration(_)
        | Statement::TSModuleDeclaration(_)
        | Statement::TSImportEqualsDeclaration(_) => stmt.as_declaration(),
        _ => None,
    }
}

fn push_work<'b, 'a>(
    work: DeclWork<'b, 'a>,
    works: &mut Vec<DeclWork<'b, 'a>>,
    decl_names: &mut FxHashMap<Box<str>, u32>,
) -> u32 {
    let idx = u32::try_from(works.len()).unwrap();
    decl_names.entry(work.name.clone()).or_insert(idx);
    works.push(work);
    idx
}

fn collect_declaration<'b, 'a>(
    decl: &'b Declaration<'a>,
    works: &mut Vec<DeclWork<'b, 'a>>,
    decl_names: &mut FxHashMap<Box<str>, u32>,
    surface: &mut FileSurface,
    exported: bool,
) {
    let mut add = |work: DeclWork<'b, 'a>| {
        let name = work.name.clone();
        let idx = push_work(work, works, decl_names);
        if exported {
            surface.exports.push(SurfaceExport::Decl { decl: idx, name });
        }
    };
    match decl {
        Declaration::VariableDeclaration(var) => {
            for declarator in &var.declarations {
                // Destructuring declarations are not valid in declaration files.
                if let BindingPattern::BindingIdentifier(id) = &declarator.id {
                    add(DeclWork {
                        name: id.name.as_str().into(),
                        span: id.span,
                        item: DeclItem::Var(declarator),
                    });
                }
            }
        }
        Declaration::FunctionDeclaration(func) => {
            if let Some(id) = &func.id {
                add(DeclWork {
                    name: id.name.as_str().into(),
                    span: id.span,
                    item: DeclItem::Func(func),
                });
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                add(DeclWork {
                    name: id.name.as_str().into(),
                    span: id.span,
                    item: DeclItem::Class(class),
                });
            }
        }
        Declaration::TSTypeAliasDeclaration(alias) => add(DeclWork {
            name: alias.id.name.as_str().into(),
            span: alias.id.span,
            item: DeclItem::Alias(alias),
        }),
        Declaration::TSInterfaceDeclaration(iface) => add(DeclWork {
            name: iface.id.name.as_str().into(),
            span: iface.id.span,
            item: DeclItem::Interface(iface),
        }),
        Declaration::TSEnumDeclaration(enum_decl) => add(DeclWork {
            name: enum_decl.id.name.as_str().into(),
            span: enum_decl.id.span,
            item: DeclItem::Enum(enum_decl),
        }),
        Declaration::TSModuleDeclaration(module) => {
            // Only identifier-named namespaces; ambient `declare module "x"`
            // and `declare global` are unmodeled in v0.
            if let TSModuleDeclarationName::Identifier(id) = &module.id {
                add(DeclWork {
                    name: id.name.as_str().into(),
                    span: id.span,
                    item: DeclItem::Namespace(module),
                });
            }
        }
        // `declare global` blocks and `import =` aliases are unmodeled in v0.
        Declaration::TSGlobalDeclaration(_) | Declaration::TSImportEqualsDeclaration(_) => {}
    }
}

/// Infer the type of a declaration-file initializer
/// (`export declare const version = 1`).
fn infer_literal_initializer<R: ResolveName>(
    lowerer: &mut Lowerer<'_, R>,
    init: &Expression<'_>,
) -> TypeId {
    match init {
        Expression::StringLiteral(s) => {
            lowerer.sink.push(Type::StringLiteral(s.value.as_str().into()))
        }
        Expression::NumericLiteral(n) => lowerer.sink.push(Type::NumberLiteral(n.value)),
        Expression::BooleanLiteral(b) => lowerer.sink.push(Type::BooleanLiteral(b.value)),
        Expression::BigIntLiteral(b) => {
            lowerer.sink.push(Type::BigIntLiteral(b.value.as_str().into()))
        }
        Expression::TemplateLiteral(t) if t.expressions.is_empty() && t.quasis.len() == 1 => {
            let text = t.quasis[0].value.cooked.as_deref().unwrap_or("");
            lowerer.sink.push(Type::StringLiteral(text.into()))
        }
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::UnaryNegation => {
            if let Expression::NumericLiteral(n) = &unary.argument {
                lowerer.sink.push(Type::NumberLiteral(-n.value))
            } else {
                TypeTable::ANY
            }
        }
        _ => TypeTable::ANY,
    }
}
