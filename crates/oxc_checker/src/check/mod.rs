//! Checking (pass B): each file is checked independently, in parallel, in any
//! order, against the frozen [`ProgramEnv`].
//!
//! The file's own AST is per-thread scratch — re-parsed here, dropped after.
//! Newly created types (inferred literals, lowered annotations) live in a
//! per-file overlay on top of the frozen global table.

mod infer;
pub mod print;
mod relate;

pub use relate::{Relation, relate};

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Declaration, ExportNamedDeclaration, Expression, Function, ImportDeclaration,
    ImportDeclarationSpecifier, Program, ReturnStatement, Statement, TSType, VariableDeclaration,
    VariableDeclarationKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use rayon::prelude::*;
use rustc_hash::FxHashMap;

use crate::{
    diagnostics,
    ir::{FileId, Member, ObjectShape, RefTarget, SymbolId, SymbolKind, Type, TypeId, TypeTable},
    link::{ExportEntry, ModuleResolution, ProgramEnv},
    lower::{Lowerer, NameTarget, ResolveName, TypeSink},
};

/// Read access to global + per-file types, write access to per-file types.
pub struct TypeView<'e> {
    /// The frozen environment.
    pub env: &'e ProgramEnv,
    sink: TypeSink,
}

impl<'e> TypeView<'e> {
    /// Create a view whose overlay starts after the global table.
    pub fn new(env: &'e ProgramEnv) -> Self {
        Self { env, sink: TypeSink::new(env.types.len()) }
    }

    /// Look up any type id (global or overlay).
    pub fn get(&self, id: TypeId) -> &Type {
        if id.0 < self.sink.base() { self.env.types.get(id) } else { self.sink.get_local(id) }
    }

    /// Append an overlay type.
    pub fn push(&mut self, ty: Type) -> TypeId {
        self.sink.push(ty)
    }
}

/// Check every checkable file in parallel. Returns diagnostics per file,
/// indexed by [`FileId`].
pub fn check_program(env: &ProgramEnv) -> Vec<Vec<OxcDiagnostic>> {
    (0..env.files.len())
        .into_par_iter()
        .map(|index| {
            let file_id = FileId(u32::try_from(index).unwrap());
            let file = env.file(file_id);
            if file.kind == crate::loader::FileKind::Ts && !file.parse_failed {
                check_file(env, file_id)
            } else {
                Vec::new()
            }
        })
        .collect()
}

/// How a name in type position resolves within the checked file.
enum TypeNameEntry {
    /// A program symbol (own-file surface decl, or import).
    Symbol(SymbolId),
    /// An inline-lowered local type (non-exported alias/interface).
    Inline(TypeId),
    /// A local generic declaration (index into `FileChecker::local_generics`).
    LocalGeneric(u32),
    /// Known to exist but untyped (opaque module) — `any`.
    Opaque,
}

/// A checker-local generic alias/interface, instantiated per reference.
struct LocalGeneric {
    name: Box<str>,
    params: Box<[crate::ir::TypeParamInfo]>,
    body: TypeId,
    is_interface: bool,
}

struct CheckResolver<'m, 'e> {
    env: &'e ProgramEnv,
    type_names: &'m FxHashMap<Box<str>, TypeNameEntry>,
    namespace_imports: &'m FxHashMap<Box<str>, FileId>,
    /// Diagnostics produced during name resolution (TS2694), drained by the
    /// caller after lowering.
    pending: std::cell::RefCell<Vec<OxcDiagnostic>>,
}

impl ResolveName for CheckResolver<'_, '_> {
    fn resolve(&self, name: &str) -> NameTarget {
        match self.type_names.get(name) {
            Some(TypeNameEntry::Symbol(symbol)) => NameTarget::Ref(RefTarget::Symbol(*symbol)),
            Some(TypeNameEntry::Inline(ty)) => NameTarget::Inline(*ty),
            Some(TypeNameEntry::LocalGeneric(index)) => NameTarget::Ref(RefTarget::Local(*index)),
            Some(TypeNameEntry::Opaque) => NameTarget::Inline(TypeTable::ANY),
            // Unknown global — no lib.d.ts in v0, so stay silent.
            None => NameTarget::Ref(RefTarget::Unresolved),
        }
    }

    fn resolve_qualified(&self, left: &str, member: &str, span: oxc_span::Span) -> NameTarget {
        // `Status.Active` in type position: an enum member type.
        if let Some(TypeNameEntry::Symbol(symbol)) = self.type_names.get(left)
            && let SymbolKind::Enum { members } = &self.env.symbol(*symbol).kind
        {
            if let Some(index) = members.iter().position(|m| &*m.name == member) {
                return NameTarget::EnumMember {
                    symbol: *symbol,
                    index: u32::try_from(index).unwrap_or(u32::MAX),
                };
            }
            return NameTarget::Ref(RefTarget::Unresolved);
        }
        if let Some(file_id) = self.namespace_imports.get(left) {
            let file = self.env.file(*file_id);
            if let Some(entry) = file.exports.get(member)
                && let Some(symbol) = entry.ty.or(entry.value)
            {
                return NameTarget::Ref(RefTarget::Symbol(symbol));
            }
            if file.opaque_exports {
                return NameTarget::Inline(TypeTable::ANY);
            }
            self.pending.borrow_mut().push(diagnostics::namespace_no_member(
                &module_display_path(&file.path),
                member,
                span,
            ));
        }
        NameTarget::Ref(RefTarget::Unresolved)
    }
}

struct FileChecker<'e> {
    env: &'e ProgramEnv,
    file_id: FileId,
    view: TypeView<'e>,
    type_names: FxHashMap<Box<str>, TypeNameEntry>,
    value_names: FxHashMap<Box<str>, TypeId>,
    /// Straight-line top-level flow: the precise current type of each
    /// top-level binding (narrowed to initializer/assignment types).
    flow_values: FxHashMap<Box<str>, TypeId>,
    /// Whether identifier reads may consult `flow_values` — true only during
    /// the straight-line top-level walk, not inside function bodies.
    use_flow: bool,
    namespace_imports: FxHashMap<Box<str>, FileId>,
    /// Local (non-exported) generic declarations, instantiated per reference.
    local_generics: Vec<LocalGeneric>,
    /// Narrowed types inside function bodies (typeof guards, discriminant
    /// switches), innermost scope last.
    narrow_stack: Vec<FxHashMap<Box<str>, TypeId>>,
    /// Whether the narrow state models every guard on the current path —
    /// unmodeled guards make identifier checks fall back to conservative
    /// rules.
    narrow_reliable: bool,
    diagnostics: Vec<OxcDiagnostic>,
}

fn check_file(env: &ProgramEnv, file_id: FileId) -> Vec<OxcDiagnostic> {
    let file = env.file(file_id);
    let source_type = SourceType::from_path(&file.path).unwrap_or_else(|_| SourceType::ts());
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &file.source_text, source_type).parse();
    if !parsed.errors.is_empty() {
        // Pass A already recorded the parse errors.
        return Vec::new();
    }

    let mut checker = FileChecker {
        env,
        file_id,
        view: TypeView::new(env),
        type_names: FxHashMap::default(),
        value_names: FxHashMap::default(),
        flow_values: FxHashMap::default(),
        use_flow: true,
        namespace_imports: FxHashMap::default(),
        local_generics: Vec::new(),
        narrow_stack: Vec::new(),
        narrow_reliable: true,
        diagnostics: Vec::new(),
    };
    checker.run(&parsed.program);
    checker.diagnostics
}

impl FileChecker<'_> {
    fn run(&mut self, program: &Program<'_>) {
        // 1. Own-file surface declarations are resolvable by name.
        for (name, &symbol) in &self.env.file(self.file_id).local_type_names {
            if self.env.symbol(symbol).kind.is_type() {
                self.type_names.insert(name.clone(), TypeNameEntry::Symbol(symbol));
            }
        }

        // 2. Imports bind names and are validated (hoisted, so before the walk).
        for stmt in &program.body {
            match stmt {
                Statement::ImportDeclaration(import) => self.bind_import(import),
                Statement::ExportNamedDeclaration(export) => self.validate_export_from(export),
                Statement::ExportAllDeclaration(export) => {
                    self.validate_specifier_resolves(
                        export.source.value.as_str(),
                        export.source.span,
                    );
                }
                _ => {}
            }
        }

        // 3. Local (non-surface) type declarations lower inline, in source
        //    order. Forward references between them degrade to silence.
        for stmt in &program.body {
            let decl = match stmt {
                Statement::ExportNamedDeclaration(export) => export.declaration.as_ref(),
                _ => stmt.as_declaration(),
            };
            match decl {
                Some(Declaration::TSTypeAliasDeclaration(alias)) => {
                    let name = alias.id.name.as_str();
                    if !self.type_names.contains_key(name) {
                        let entry = if let Some(type_params) = &alias.type_parameters {
                            let resolver = CheckResolver {
                                env: self.env,
                                type_names: &self.type_names,
                                namespace_imports: &self.namespace_imports,
                                pending: std::cell::RefCell::new(Vec::new()),
                            };
                            let params = crate::surface::lower_type_params(
                                &mut self.view.sink,
                                &resolver,
                                Some(type_params),
                            );
                            let names = params.iter().map(|p| p.name.clone()).collect();
                            let body = Lowerer::with_params(&mut self.view.sink, &resolver, names)
                                .lower_type(&alias.type_annotation);
                            let index = u32::try_from(self.local_generics.len()).unwrap();
                            self.local_generics.push(LocalGeneric {
                                name: name.into(),
                                params: params
                                    .into_iter()
                                    .map(|p| crate::ir::TypeParamInfo {
                                        name: p.name,
                                        constraint: p.constraint,
                                        default: p.default,
                                    })
                                    .collect(),
                                body,
                                is_interface: false,
                            });
                            TypeNameEntry::LocalGeneric(index)
                        } else {
                            let ty = self.lower_ts_type(&alias.type_annotation);
                            TypeNameEntry::Inline(self.name_wrap(name, ty))
                        };
                        self.type_names.insert(name.into(), entry);
                    }
                }
                Some(Declaration::TSInterfaceDeclaration(iface)) => {
                    // TS2314/TS2344 in `extends` clauses.
                    for heritage in &iface.extends {
                        if let Expression::Identifier(base) = &heritage.expression {
                            self.validate_reference(
                                base.name.as_str(),
                                heritage.type_arguments.as_deref(),
                                heritage.span,
                            );
                        }
                    }
                    let name = iface.id.name.as_str();
                    if !self.type_names.contains_key(name) {
                        let resolver = CheckResolver {
                            env: self.env,
                            type_names: &self.type_names,
                            namespace_imports: &self.namespace_imports,
                            pending: std::cell::RefCell::new(Vec::new()),
                        };
                        let entry = if let Some(type_params) = &iface.type_parameters {
                            let params = crate::surface::lower_type_params(
                                &mut self.view.sink,
                                &resolver,
                                Some(type_params),
                            );
                            let names: Vec<Box<str>> =
                                params.iter().map(|p| p.name.clone()).collect();
                            let body = Lowerer::with_params(&mut self.view.sink, &resolver, names)
                                .lower_interface(iface);
                            let index = u32::try_from(self.local_generics.len()).unwrap();
                            self.local_generics.push(LocalGeneric {
                                name: name.into(),
                                params: params
                                    .into_iter()
                                    .map(|p| crate::ir::TypeParamInfo {
                                        name: p.name,
                                        constraint: p.constraint,
                                        default: p.default,
                                    })
                                    .collect(),
                                body,
                                is_interface: true,
                            });
                            TypeNameEntry::LocalGeneric(index)
                        } else {
                            let ty =
                                Lowerer::new(&mut self.view.sink, &resolver).lower_interface(iface);
                            TypeNameEntry::Inline(self.name_wrap(name, ty))
                        };
                        self.type_names.insert(name.into(), entry);
                    }
                }
                _ => {}
            }
        }

        // 4. Function declarations hoist: bind their shapes first.
        for stmt in &program.body {
            let func = match stmt {
                Statement::FunctionDeclaration(func) => func,
                Statement::ExportNamedDeclaration(export) => match &export.declaration {
                    Some(Declaration::FunctionDeclaration(func)) => func,
                    _ => continue,
                },
                _ => continue,
            };
            if let Some(id) = &func.id {
                let resolver = CheckResolver {
                    env: self.env,
                    type_names: &self.type_names,
                    namespace_imports: &self.namespace_imports,
                    pending: std::cell::RefCell::new(Vec::new()),
                };
                let shape = Lowerer::new(&mut self.view.sink, &resolver)
                    .lower_function_shape(&func.params, func.return_type.as_deref());
                let ty = self.view.push(Type::Function(Box::new(shape)));
                self.value_names.insert(id.name.as_str().into(), ty);
            }
        }

        // 5. The checking walk.
        for stmt in &program.body {
            self.check_statement(stmt);
        }
    }

    fn lower_ts_type(&mut self, ty: &TSType<'_>) -> TypeId {
        self.validate_type_args(ty);
        let id = self.lower_ts_type_unvalidated(ty);
        self.expand_generics(id, 0)
    }

    fn lower_ts_type_unvalidated(&mut self, ty: &TSType<'_>) -> TypeId {
        let resolver = CheckResolver {
            env: self.env,
            type_names: &self.type_names,
            namespace_imports: &self.namespace_imports,
            pending: std::cell::RefCell::new(Vec::new()),
        };
        let id = Lowerer::new(&mut self.view.sink, &resolver).lower_type(ty);
        self.diagnostics.extend(resolver.pending.into_inner());
        id
    }

    /// Generic parameter info for a named type reference, when generic.
    fn generic_params_of(
        &self,
        name: &str,
    ) -> Option<(Box<str>, Vec<crate::ir::TypeParamInfo>, bool)> {
        match self.type_names.get(name)? {
            TypeNameEntry::Symbol(symbol) => match &self.env.symbol(*symbol).kind {
                SymbolKind::TypeAlias { params, .. } if !params.is_empty() => {
                    Some((name.into(), params.to_vec(), false))
                }
                SymbolKind::Interface { params, .. } if !params.is_empty() => {
                    Some((name.into(), params.to_vec(), true))
                }
                _ => None,
            },
            TypeNameEntry::LocalGeneric(index) => {
                let generic = &self.local_generics[*index as usize];
                Some((generic.name.clone(), generic.params.to_vec(), generic.is_interface))
            }
            _ => None,
        }
    }

    /// TS2314 (wrong arity) / TS2344 (constraint violation) over the
    /// annotation AST, where argument spans live.
    fn validate_type_args(&mut self, ty: &TSType<'_>) {
        use oxc_ast::ast::TSTypeName;
        match ty {
            TSType::TSTypeReference(reference) => {
                if let Some(args) = &reference.type_arguments {
                    for arg in &args.params {
                        self.validate_type_args(arg);
                    }
                }
                let TSTypeName::IdentifierReference(ident) = &reference.type_name else { return };
                self.validate_reference(
                    ident.name.as_str(),
                    reference.type_arguments.as_deref(),
                    reference.span,
                );
            }
            TSType::TSUnionType(union) => {
                for member in &union.types {
                    self.validate_type_args(member);
                }
            }
            TSType::TSIntersectionType(intersection) => {
                for member in &intersection.types {
                    self.validate_type_args(member);
                }
            }
            TSType::TSArrayType(array) => self.validate_type_args(&array.element_type),
            TSType::TSTupleType(tuple) => {
                use oxc_ast::ast::TSTupleElement;
                for element in &tuple.element_types {
                    match element {
                        TSTupleElement::TSOptionalType(_) | TSTupleElement::TSRestType(_) => {}
                        _ => self.validate_type_args(element.to_ts_type()),
                    }
                }
            }
            TSType::TSParenthesizedType(paren) => {
                self.validate_type_args(&paren.type_annotation);
            }
            TSType::TSTypeOperatorType(op) => self.validate_type_args(&op.type_annotation),
            _ => {}
        }
    }

    /// Arity (TS2314) and constraint (TS2344) validation for one named type
    /// reference — shared by annotations and interface `extends` clauses.
    fn validate_reference(
        &mut self,
        name: &str,
        type_arguments: Option<&oxc_ast::ast::TSTypeParameterInstantiation<'_>>,
        span: oxc_span::Span,
    ) {
        let Some((name, params, is_interface)) = self.generic_params_of(name) else {
            return;
        };
        let provided = type_arguments.map_or(0, |a| a.params.len());
        let required = params.iter().take_while(|p| p.default.is_none()).count();
        if provided < required || provided > params.len() {
            // Interfaces display with their parameter list.
            let display = if is_interface {
                let names: Vec<&str> = params.iter().map(|p| &*p.name).collect();
                format!("{name}<{}>", names.join(", "))
            } else {
                name.to_string()
            };
            self.diagnostics.push(diagnostics::wrong_type_arity(&display, required, span));
            return;
        }
        // Constraint checks against provided arguments.
        let Some(args) = type_arguments else { return };
        let arg_tys: Vec<TypeId> = args
            .params
            .iter()
            .map(|a| {
                let id = self.lower_ts_type_unvalidated(a);
                self.expand_generics(id, 0)
            })
            .collect();
        for (i, param) in params.iter().enumerate() {
            let (Some(constraint), Some(&arg_ty)) = (param.constraint, arg_tys.get(i)) else {
                continue;
            };
            let constraint = self.substitute(constraint, &arg_tys, 0);
            if relate(&self.view, arg_ty, constraint) == Relation::False {
                self.diagnostics.push(diagnostics::constraint_not_satisfied(
                    &self.view,
                    arg_ty,
                    constraint,
                    args.params[i].span(),
                ));
            }
        }
    }

    /// Instantiate generic references throughout a lowered type.
    fn expand_generics(&mut self, id: TypeId, depth: u32) -> TypeId {
        if depth > 16 {
            return TypeTable::UNSUPPORTED;
        }
        match self.view.get(id).clone() {
            Type::Ref { target, args } if !args.is_empty() => {
                let args: Vec<TypeId> =
                    args.iter().map(|a| self.expand_generics(*a, depth + 1)).collect();
                let (name, params, body) = match &target {
                    RefTarget::Symbol(symbol) => match &self.env.symbol(*symbol).kind {
                        SymbolKind::TypeAlias { ty, params }
                        | SymbolKind::Interface { ty, params }
                            if !params.is_empty() =>
                        {
                            (self.env.symbol(*symbol).name.clone(), params.to_vec(), *ty)
                        }
                        _ => return id,
                    },
                    RefTarget::Local(index) => {
                        let generic = &self.local_generics[*index as usize];
                        (generic.name.clone(), generic.params.to_vec(), generic.body)
                    }
                    _ => return id,
                };
                // Fill missing trailing arguments from defaults.
                let mut final_args = args;
                for param in params.iter().skip(final_args.len()) {
                    let filled = match param.default {
                        Some(default) => self.substitute(default, &final_args, depth + 1),
                        None => TypeTable::ANY,
                    };
                    final_args.push(filled);
                }
                let expanded = self.substitute(body, &final_args, depth + 1);
                // Wrap with the rendered instantiation name for diagnostics
                // (`Boxed<number>`).
                let rendered_args: Vec<String> =
                    final_args.iter().map(|a| print::type_to_string(&self.view, *a)).collect();
                let rendered = format!("{name}<{}>", rendered_args.join(", "));
                self.view.push(Type::Named { name: rendered.into_boxed_str(), ty: expanded })
            }
            Type::Ref { target: RefTarget::Local(index), args } if args.is_empty() => {
                // Bare reference to a local generic: defaults fill what they
                // can (arity validation flags genuinely missing arguments).
                let generic = &self.local_generics[index as usize];
                let (body, params) = (generic.body, generic.params.to_vec());
                let mut args: Vec<TypeId> = Vec::new();
                for param in &params {
                    let filled = match param.default {
                        Some(default) => self.substitute(default, &args, depth + 1),
                        None => TypeTable::ANY,
                    };
                    args.push(filled);
                }
                self.substitute(body, &args, depth + 1)
            }
            Type::Ref { target: RefTarget::Symbol(symbol), args } if args.is_empty() => {
                match &self.env.symbol(symbol).kind {
                    SymbolKind::TypeAlias { ty, params } | SymbolKind::Interface { ty, params }
                        if !params.is_empty() =>
                    {
                        let (body, params) = (*ty, params.to_vec());
                        let mut args: Vec<TypeId> = Vec::new();
                        for param in &params {
                            let filled = match param.default {
                                Some(default) => self.substitute(default, &args, depth + 1),
                                None => TypeTable::ANY,
                            };
                            args.push(filled);
                        }
                        self.substitute(body, &args, depth + 1)
                    }
                    _ => id,
                }
            }
            Type::Union(members) => {
                let members: Vec<TypeId> =
                    members.iter().map(|m| self.expand_generics(*m, depth + 1)).collect();
                self.view.push(Type::Union(members.into_boxed_slice()))
            }
            Type::Intersection(members) => {
                let members: Vec<TypeId> =
                    members.iter().map(|m| self.expand_generics(*m, depth + 1)).collect();
                self.view.push(Type::Intersection(members.into_boxed_slice()))
            }
            Type::Array(elem) => {
                let elem = self.expand_generics(elem, depth + 1);
                self.view.push(Type::Array(elem))
            }
            Type::Readonly(inner) => {
                let inner = self.expand_generics(inner, depth + 1);
                self.view.push(Type::Readonly(inner))
            }
            Type::Tuple(members) => {
                let members: Vec<TypeId> =
                    members.iter().map(|m| self.expand_generics(*m, depth + 1)).collect();
                self.view.push(Type::Tuple(members.into_boxed_slice()))
            }
            Type::Object(shape) => {
                let members: Vec<Member> = shape
                    .members
                    .iter()
                    .map(|m| Member {
                        name: m.name.clone(),
                        ty: self.expand_generics(m.ty, depth + 1),
                        optional: m.optional,
                    })
                    .collect();
                self.view.push(Type::Object(ObjectShape {
                    members: members.into_boxed_slice(),
                    inexact: shape.inexact,
                }))
            }
            Type::Function(shape) => {
                let params: Vec<crate::ir::Param> = shape
                    .params
                    .iter()
                    .map(|p| crate::ir::Param {
                        name: p.name.clone(),
                        ty: self.expand_generics(p.ty, depth + 1),
                        optional: p.optional,
                    })
                    .collect();
                let ret = self.expand_generics(shape.ret, depth + 1);
                self.view.push(Type::Function(Box::new(crate::ir::FunctionShape {
                    params: params.into_boxed_slice(),
                    ret,
                })))
            }
            Type::Named { name, ty } => {
                let ty = self.expand_generics(ty, depth + 1);
                self.view.push(Type::Named { name, ty })
            }
            Type::Fresh(inner) => {
                let inner = self.expand_generics(inner, depth + 1);
                self.view.push(Type::Fresh(inner))
            }
            _ => id,
        }
    }

    /// Replace [`Type::TypeParam`] markers with `args`, instantiating nested
    /// generic references along the way.
    fn substitute(&mut self, id: TypeId, args: &[TypeId], depth: u32) -> TypeId {
        if depth > 16 {
            return TypeTable::UNSUPPORTED;
        }
        match self.view.get(id).clone() {
            Type::TypeParam { index, .. } => {
                args.get(index as usize).copied().unwrap_or(TypeTable::ANY)
            }
            Type::Ref { target, args: ref_args } => {
                let ref_args: Vec<TypeId> =
                    ref_args.iter().map(|a| self.substitute(*a, args, depth + 1)).collect();
                let rebuilt =
                    self.view.push(Type::Ref { target, args: ref_args.into_boxed_slice() });
                self.expand_generics(rebuilt, depth + 1)
            }
            Type::Union(members) => {
                let members: Vec<TypeId> =
                    members.iter().map(|m| self.substitute(*m, args, depth + 1)).collect();
                self.view.push(Type::Union(members.into_boxed_slice()))
            }
            Type::Intersection(members) => {
                let members: Vec<TypeId> =
                    members.iter().map(|m| self.substitute(*m, args, depth + 1)).collect();
                self.view.push(Type::Intersection(members.into_boxed_slice()))
            }
            Type::Array(elem) => {
                let elem = self.substitute(elem, args, depth + 1);
                self.view.push(Type::Array(elem))
            }
            Type::Readonly(inner) => {
                let inner = self.substitute(inner, args, depth + 1);
                self.view.push(Type::Readonly(inner))
            }
            Type::Tuple(members) => {
                let members: Vec<TypeId> =
                    members.iter().map(|m| self.substitute(*m, args, depth + 1)).collect();
                self.view.push(Type::Tuple(members.into_boxed_slice()))
            }
            Type::Object(shape) => {
                let members: Vec<Member> = shape
                    .members
                    .iter()
                    .map(|m| Member {
                        name: m.name.clone(),
                        ty: self.substitute(m.ty, args, depth + 1),
                        optional: m.optional,
                    })
                    .collect();
                self.view.push(Type::Object(ObjectShape {
                    members: members.into_boxed_slice(),
                    inexact: shape.inexact,
                }))
            }
            Type::Function(shape) => {
                let params: Vec<crate::ir::Param> = shape
                    .params
                    .iter()
                    .map(|p| crate::ir::Param {
                        name: p.name.clone(),
                        ty: self.substitute(p.ty, args, depth + 1),
                        optional: p.optional,
                    })
                    .collect();
                let ret = self.substitute(shape.ret, args, depth + 1);
                self.view.push(Type::Function(Box::new(crate::ir::FunctionShape {
                    params: params.into_boxed_slice(),
                    ret,
                })))
            }
            Type::Named { name, ty } => {
                let ty = self.substitute(ty, args, depth + 1);
                self.view.push(Type::Named { name, ty })
            }
            Type::Fresh(inner) => {
                let inner = self.substitute(inner, args, depth + 1);
                self.view.push(Type::Fresh(inner))
            }
            _ => id,
        }
    }

    fn resolution(&self, specifier: &str) -> Option<ModuleResolution> {
        self.env.file(self.file_id).resolutions.get(specifier).copied()
    }

    fn validate_specifier_resolves(&mut self, specifier: &str, span: oxc_span::Span) -> bool {
        match self.resolution(specifier) {
            None | Some(ModuleResolution::NotFound) => {
                self.diagnostics.push(diagnostics::module_not_found(specifier, span));
                false
            }
            Some(_) => true,
        }
    }

    fn symbol_value_type(&mut self, entry: ExportEntry) -> TypeId {
        let Some(symbol) = entry.value else { return TypeTable::ANY };
        let ty = match self.env.symbol(symbol).kind {
            SymbolKind::Value { ty } => ty,
            SymbolKind::Class { .. } => return self.view.push(Type::ClassValue(symbol)),
            SymbolKind::Enum { .. } => return self.view.push(Type::EnumValue(symbol)),
            _ => TypeTable::ANY,
        };
        // Surface annotations may carry generic references (`Boxed<number>`).
        self.expand_generics(ty, 0)
    }

    /// Bind a same-file class/enum declaration's value side and run class
    /// implements checks.
    fn bind_class_or_enum(&mut self, name: &str) {
        let Some(&symbol) = self.env.file(self.file_id).local_type_names.get(name) else {
            return;
        };
        let value = match self.env.symbol(symbol).kind {
            SymbolKind::Class { .. } => Type::ClassValue(symbol),
            SymbolKind::Enum { .. } => Type::EnumValue(symbol),
            _ => return,
        };
        let ty = self.view.push(value);
        self.value_names.insert(name.into(), ty);
    }

    /// TS2420/TS2416: check a class declaration against its `implements`
    /// clause.
    fn check_class(&mut self, class: &oxc_ast::ast::Class<'_>) {
        use oxc_ast::ast::{ClassElement, TSTypeName};
        let Some(id) = &class.id else { return };
        self.bind_class_or_enum(id.name.as_str());
        let Some(&symbol) = self.env.file(self.file_id).local_type_names.get(id.name.as_str())
        else {
            return;
        };
        let SymbolKind::Class { instance } = self.env.symbol(symbol).kind else { return };
        let Some(own_peeled) = self.peel(instance) else { return };
        let Type::Object(own_shape) = self.view.get(own_peeled) else { return };
        let own_members: Vec<Member> = own_shape
            .members
            .iter()
            .map(|m| Member { name: m.name.clone(), ty: m.ty, optional: m.optional })
            .collect();
        let own_exact = !own_shape.inexact;

        for implement in &class.implements {
            let TSTypeName::IdentifierReference(iface_name) = &implement.expression else {
                continue;
            };
            // Only non-generic identifier targets in v0.
            if implement.type_arguments.is_some() {
                continue;
            }
            let resolver = CheckResolver {
                env: self.env,
                type_names: &self.type_names,
                namespace_imports: &self.namespace_imports,
                pending: std::cell::RefCell::new(Vec::new()),
            };
            let iface_ty = match resolver.resolve(iface_name.name.as_str()) {
                NameTarget::Ref(target @ RefTarget::Symbol(_)) => {
                    self.view.push(Type::Ref { target, args: Box::new([]) })
                }
                NameTarget::Inline(ty) => ty,
                _ => continue,
            };
            let Some(iface_peeled) = self.peel(iface_ty) else { continue };
            let Type::Object(iface_shape) = self.view.get(iface_peeled) else { continue };
            let iface_members: Vec<Member> = iface_shape
                .members
                .iter()
                .map(|m| Member { name: m.name.clone(), ty: m.ty, optional: m.optional })
                .collect();

            let mut missing = false;
            for iface_member in &iface_members {
                match own_members.iter().find(|m| m.name == iface_member.name) {
                    Some(own_member) => {
                        if relate(&self.view, own_member.ty, iface_member.ty) == Relation::False {
                            // tsc reports the mismatch at the class element key.
                            let span = class
                                .body
                                .body
                                .iter()
                                .find_map(|element| match element {
                                    ClassElement::PropertyDefinition(p) => {
                                        crate::lower::property_key_name(&p.key)
                                            .filter(|n| *n == iface_member.name)
                                            .map(|_| p.key.span())
                                    }
                                    ClassElement::MethodDefinition(m) => {
                                        crate::lower::property_key_name(&m.key)
                                            .filter(|n| *n == iface_member.name)
                                            .map(|_| m.key.span())
                                    }
                                    _ => None,
                                })
                                .unwrap_or(id.span);
                            self.diagnostics.push(diagnostics::property_not_assignable_to_base(
                                &self.view,
                                &iface_member.name,
                                id.name.as_str(),
                                iface_ty,
                                span,
                            ));
                        }
                    }
                    None => {
                        if !iface_member.optional && own_exact {
                            missing = true;
                        }
                    }
                }
            }
            if missing {
                self.diagnostics.push(diagnostics::incorrectly_implements(
                    &self.view,
                    id.name.as_str(),
                    iface_ty,
                    id.span,
                ));
            }
        }
    }

    fn bind_import(&mut self, import: &ImportDeclaration<'_>) {
        let specifier = import.source.value.as_str();
        let resolved = match self.resolution(specifier) {
            None | Some(ModuleResolution::NotFound) => {
                self.diagnostics.push(diagnostics::module_not_found(specifier, import.source.span));
                None
            }
            Some(ModuleResolution::External) => None,
            Some(ModuleResolution::File(file_id)) => Some(file_id),
        };

        let Some(specifiers) = &import.specifiers else { return };
        for spec in specifiers {
            match spec {
                ImportDeclarationSpecifier::ImportSpecifier(named) => {
                    let imported = named.imported.name();
                    let local = named.local.name.as_str();
                    let entry = resolved.map(|file_id| {
                        let target = self.env.file(file_id);
                        (target.exports.get(imported.as_str()).copied(), target.opaque_exports)
                    });
                    match entry {
                        Some((Some(entry), _)) => {
                            let value_ty = self.symbol_value_type(entry);
                            self.value_names.insert(local.into(), value_ty);
                            let type_entry = match entry.ty {
                                Some(symbol) => TypeNameEntry::Symbol(symbol),
                                None => TypeNameEntry::Opaque,
                            };
                            self.type_names.insert(local.into(), type_entry);
                        }
                        Some((None, opaque)) => {
                            if !opaque {
                                self.diagnostics.push(diagnostics::no_exported_member(
                                    specifier,
                                    imported.as_str(),
                                    named.span,
                                ));
                            }
                            self.bind_any(local);
                        }
                        None => self.bind_any(local),
                    }
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                    let local = default.local.name.as_str();
                    let entry = resolved.map(|file_id| {
                        let target = self.env.file(file_id);
                        (target.exports.get("default").copied(), target.opaque_exports, file_id)
                    });
                    match entry {
                        Some((Some(entry), _, _)) => {
                            let value_ty = self.symbol_value_type(entry);
                            self.value_names.insert(local.into(), value_ty);
                            let type_entry = match entry.ty {
                                Some(symbol) => TypeNameEntry::Symbol(symbol),
                                None => TypeNameEntry::Opaque,
                            };
                            self.type_names.insert(local.into(), type_entry);
                        }
                        Some((None, opaque, file_id)) => {
                            if !opaque {
                                // tsc prints the resolved module path, and
                                // suggests a same-named named export (TS2613).
                                let target = self.env.file(file_id);
                                let module = module_display_path(&target.path);
                                let diagnostic = if target.exports.contains_key(local) {
                                    diagnostics::no_default_export_hint(
                                        &module,
                                        local,
                                        default.local.span,
                                    )
                                } else {
                                    diagnostics::no_default_export_resolved(
                                        &module,
                                        default.local.span,
                                    )
                                };
                                self.diagnostics.push(diagnostic);
                            }
                            self.bind_any(local);
                        }
                        None => self.bind_any(local),
                    }
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(ns) => {
                    let local = ns.local.name.as_str();
                    if let Some(file_id) = resolved {
                        self.namespace_imports.insert(local.into(), file_id);
                    }
                    self.value_names.insert(local.into(), TypeTable::ANY);
                }
            }
        }
    }

    fn bind_any(&mut self, local: &str) {
        self.value_names.insert(local.into(), TypeTable::ANY);
        self.type_names.insert(local.into(), TypeNameEntry::Opaque);
    }

    fn validate_export_from(&mut self, export: &ExportNamedDeclaration<'_>) {
        let Some(source) = &export.source else { return };
        let specifier = source.value.as_str();
        if !self.validate_specifier_resolves(specifier, source.span) {
            return;
        }
        if let Some(ModuleResolution::File(file_id)) = self.resolution(specifier) {
            let target = self.env.file(file_id);
            if target.opaque_exports {
                return;
            }
            for spec in &export.specifiers {
                let name = spec.local.name();
                if !target.exports.contains_key(name.as_str()) {
                    self.diagnostics.push(diagnostics::no_exported_member(
                        specifier,
                        name.as_str(),
                        spec.span,
                    ));
                }
            }
        }
    }

    fn check_statement(&mut self, stmt: &Statement<'_>) {
        match stmt {
            Statement::VariableDeclaration(var) => self.check_variable_declaration(var),
            Statement::FunctionDeclaration(func) => self.check_function(func),
            Statement::ClassDeclaration(class) => self.check_class(class),
            Statement::TSEnumDeclaration(enum_decl) => {
                self.bind_class_or_enum(enum_decl.id.name.as_str());
            }
            Statement::ExportNamedDeclaration(export) => match &export.declaration {
                Some(Declaration::VariableDeclaration(var)) => {
                    self.check_variable_declaration(var);
                }
                Some(Declaration::FunctionDeclaration(func)) => self.check_function(func),
                Some(Declaration::ClassDeclaration(class)) => self.check_class(class),
                Some(Declaration::TSEnumDeclaration(enum_decl)) => {
                    self.bind_class_or_enum(enum_decl.id.name.as_str());
                }
                _ => {}
            },
            Statement::ExportDefaultDeclaration(export) => {
                if let Some(expr) = export.declaration.as_expression() {
                    self.infer(expr);
                }
            }
            Statement::ExpressionStatement(expr_stmt) => {
                if let Expression::AssignmentExpression(assign) = &expr_stmt.expression {
                    self.check_assignment(assign);
                } else {
                    self.infer(&expr_stmt.expression);
                }
            }
            _ => {}
        }
    }

    /// Top-level `name = expr` statements: check against the declared binding
    /// type and update straight-line flow.
    fn check_assignment(&mut self, assign: &oxc_ast::ast::AssignmentExpression<'_>) {
        use oxc_ast::ast::{AssignmentOperator, AssignmentTarget};
        let source = self.infer(&assign.right);
        let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assign.left else { return };
        if assign.operator != AssignmentOperator::Assign {
            self.flow_values.remove(ident.name.as_str());
            return;
        }
        if let Some(&declared) = self.value_names.get(ident.name.as_str()) {
            // tsc reports assignment mismatches at the left-hand side.
            self.check_assignable(Some(&assign.right), source, declared, ident.span, Head::Assign);
        }
        self.flow_values.insert(ident.name.as_str().into(), source);
    }

    fn check_variable_declaration(&mut self, var: &VariableDeclaration<'_>) {
        let is_const = var.kind == VariableDeclarationKind::Const;
        for declarator in &var.declarations {
            let BindingPattern::BindingIdentifier(ident) = &declarator.id else {
                continue;
            };
            let target =
                declarator.type_annotation.as_ref().map(|a| self.lower_ts_type(&a.type_annotation));
            let source = declarator.init.as_ref().map(|init| self.infer(init));

            if let (Some(target), Some(source)) = (target, source) {
                // Inside function bodies, identifier initializers may be
                // flow-narrowed — bind only.
                let narrowable = !self.use_flow
                    && !self.narrow_reliable
                    && matches!(
                        declarator.init.as_ref().map(strip_parens),
                        Some(Expression::Identifier(_))
                    );
                if !narrowable {
                    // tsc reports declaration mismatches at the declaration
                    // node, whose error span starts at the name.
                    self.check_assignable(
                        declarator.init.as_ref(),
                        source,
                        target,
                        ident.span,
                        Head::Assign,
                    );
                }
            }

            let binding = target.unwrap_or_else(|| {
                source.map_or(TypeTable::ANY, |source| self.type_for_binding(source, is_const))
            });
            self.value_names.insert(ident.name.as_str().into(), binding);
            // Straight-line top-level flow: reads see the initializer's
            // (regular) type — but tsc's assignment narrowing only *filters
            // unions*, where `boolean` counts as `true | false`. Non-union
            // declared/widened types stay pinned.
            if let Some(source) = source {
                let narrows = match target {
                    Some(t) => {
                        self.peel(t).is_some_and(|p| matches!(self.view.get(p), Type::Union(_)))
                    }
                    None => {
                        is_const
                            || self.peel(binding).is_some_and(|p| {
                                matches!(self.view.get(p), Type::Union(_) | Type::Boolean)
                            })
                    }
                };
                if narrows && self.use_flow {
                    // Keep freshness: it is tsc's widening flavor and must
                    // survive const bindings for later `let` initializers.
                    self.flow_values.insert(ident.name.as_str().into(), source);
                }
            }
        }
    }

    /// Binding type for an initializer in a mutable location: fresh top-level
    /// literals stay literal under `const` and widen under `let`/`var`;
    /// members of fresh object/tuple literals widen either way (properties
    /// are mutable). `as const` produces non-fresh types and passes through.
    fn type_for_binding(&mut self, source: TypeId, is_const: bool) -> TypeId {
        let Type::Fresh(inner) = self.view.get(source) else { return source };
        let inner = *inner;
        if self.is_literal(inner) {
            // tsc's *widening* literal types: a const binding keeps the
            // widening flavor (so a later `let` initialized from it still
            // widens); `let` widens immediately. `as const` produced a
            // non-fresh literal and never reaches here.
            return if is_const { source } else { self.widen_literal(inner) };
        }
        // Object/tuple literal: widen fresh literal members (no contextual type).
        self.widen_for_contextual(source, None)
    }

    /// Allocate a type and wrap it as fresh (expression-originated).
    pub(super) fn fresh(&mut self, ty: Type) -> TypeId {
        let inner = self.view.push(ty);
        self.view.push(Type::Fresh(inner))
    }

    /// Wrap a local alias body with its name for diagnostics — except when it
    /// aliases a single literal or intrinsic, which tsc prints expanded (its
    /// interned literal types never carry alias symbols).
    fn name_wrap(&mut self, name: &str, ty: TypeId) -> TypeId {
        match self.view.get(ty) {
            Type::Union(_)
            | Type::Intersection(_)
            | Type::Object(_)
            | Type::Function(_)
            | Type::Array(_)
            | Type::Tuple(_)
            | Type::Readonly(_)
            | Type::Ref { .. } => self.view.push(Type::Named { name: name.into(), ty }),
            _ => ty,
        }
    }

    /// Span of the `satisfies` keyword — tsc's effective error position for
    /// unelaborated satisfies failures.
    pub(super) fn satisfies_keyword_span(
        &self,
        satisfies: &oxc_ast::ast::TSSatisfiesExpression<'_>,
    ) -> oxc_span::Span {
        let source = &self.env.file(self.file_id).source_text;
        let start = satisfies.expression.span().end as usize;
        let end = satisfies.type_annotation.span().start as usize;
        if let Some(offset) = source.get(start..end).and_then(|s| s.find("satisfies")) {
            let s = u32::try_from(start + offset).unwrap();
            oxc_span::Span::new(s, s + 9)
        } else {
            satisfies.span
        }
    }

    /// Strip freshness without widening (tsc `getRegularTypeOfLiteralType`,
    /// applied deeply for literal containers).
    fn regular_type(&mut self, source: TypeId) -> TypeId {
        match self.view.get(source) {
            Type::Fresh(inner) => {
                let inner = *inner;
                match self.view.get(inner) {
                    Type::Object(shape) => {
                        let shape = ObjectShape {
                            members: shape.members.to_vec().into_boxed_slice(),
                            inexact: shape.inexact,
                        };
                        let members = shape
                            .members
                            .iter()
                            .map(|m| Member {
                                name: m.name.clone(),
                                ty: self.regular_type(m.ty),
                                optional: m.optional,
                            })
                            .collect();
                        self.view
                            .push(Type::Object(ObjectShape { members, inexact: shape.inexact }))
                    }
                    Type::Tuple(members) => {
                        let copied: Vec<TypeId> = members.to_vec();
                        let members = copied.into_iter().map(|m| self.regular_type(m)).collect();
                        self.view.push(Type::Tuple(members))
                    }
                    _ => inner,
                }
            }
            _ => source,
        }
    }

    fn check_function(&mut self, func: &Function<'_>) {
        // Bind the function name to its declared shape.
        if let Some(id) = &func.id {
            let resolver = CheckResolver {
                env: self.env,
                type_names: &self.type_names,
                namespace_imports: &self.namespace_imports,
                pending: std::cell::RefCell::new(Vec::new()),
            };
            let shape = Lowerer::new(&mut self.view.sink, &resolver)
                .lower_function_shape(&func.params, func.return_type.as_deref());
            let ty = self.view.push(Type::Function(Box::new(shape)));
            self.value_names.insert(id.name.as_str().into(), ty);
        }

        // Check `return` statements against an annotated return type.
        let (Some(return_type), Some(body)) = (&func.return_type, &func.body) else { return };
        if func.r#async || func.generator {
            return; // Promise / iterator wrapping is unmodeled in v0.
        }
        let target = self.lower_ts_type(&return_type.type_annotation);
        let mut returns = Vec::new();
        collect_returns(&body.statements, &mut returns);

        // TS2355: an annotated non-void function with zero return statements.
        if returns.is_empty() && self.requires_return_value(target) {
            self.diagnostics
                .push(diagnostics::must_return_value(return_type.type_annotation.span()));
            return;
        }
        // TS2366: value returns exist but the end of the body is definitely
        // reachable (conservative: `Maybe` stays silent).
        if !returns.is_empty()
            && returns.iter().any(|r| r.argument.is_some())
            && self.requires_return_value(target)
            && definitely_returns(&body.statements) == Returns::No
        {
            self.diagnostics
                .push(diagnostics::lacks_ending_return(return_type.type_annotation.span()));
        }

        let shadowed = self.bind_params(&func.params);
        self.check_return_body(target, body, false);
        self.unbind_params(shadowed);
    }

    /// Whether a declared return type demands a `return` with a value
    /// (neither `undefined`, `void`, nor `any` — `never`/`unknown` and
    /// unresolvable types stay silent).
    fn requires_return_value(&self, target: TypeId) -> bool {
        let Some(peeled) = self.peel(target) else { return false };
        match self.view.get(peeled) {
            Type::Void
            | Type::Undefined
            | Type::Any
            | Type::Unknown
            | Type::Never
            | Type::Unsupported => false,
            Type::Union(members) => members.to_vec().iter().all(|m| self.requires_return_value(*m)),
            _ => true,
        }
    }

    fn widen_literal(&self, ty: TypeId) -> TypeId {
        match self.view.get(ty) {
            Type::StringLiteral(_) => TypeTable::STRING,
            Type::NumberLiteral(_) => TypeTable::NUMBER,
            Type::BooleanLiteral(_) => TypeTable::BOOLEAN,
            Type::BigIntLiteral(_) => TypeTable::BIGINT,
            _ => ty,
        }
    }

    fn is_literal(&self, ty: TypeId) -> bool {
        matches!(
            self.view.get(ty),
            Type::StringLiteral(_)
                | Type::NumberLiteral(_)
                | Type::BooleanLiteral(_)
                | Type::BigIntLiteral(_)
        )
    }

    /// Strip aliases/names/freshness down to a structural type id, when v0
    /// can see through everything on the way.
    fn peel(&self, ty: TypeId) -> Option<TypeId> {
        match relate::resolve(&self.view, ty) {
            relate::Resolved::Concrete(id) => Some(id),
            _ => None,
        }
    }

    /// tsc's `checkExpressionForMutableLocation` +
    /// `getWidenedLiteralLikeTypeForContextualType`: *fresh* literals widen to
    /// their base primitive unless the contextual type contains a literal of
    /// the same kind; fresh object/tuple literals apply the rule per member
    /// against the corresponding contextual member. Non-fresh types pass
    /// through untouched (assertions, identifiers, `as const`).
    fn widen_for_contextual(&mut self, source: TypeId, contextual: Option<TypeId>) -> TypeId {
        self.widen_for_contextual_in(source, contextual, true)
    }

    #[expect(clippy::needless_collect)] // the collect ends the `&self.view` borrow
    fn widen_for_contextual_in(
        &mut self,
        source: TypeId,
        contextual: Option<TypeId>,
        top_level: bool,
    ) -> TypeId {
        let Type::Fresh(inner) = self.view.get(source) else { return source };
        let inner = *inner;
        match self.view.get(inner) {
            Type::StringLiteral(_)
            | Type::NumberLiteral(_)
            | Type::BooleanLiteral(_)
            | Type::BigIntLiteral(_) => {
                // A top-level literal checked against a declared type is not
                // a mutable location — only members/elements widen.
                if top_level {
                    return inner;
                }
                let kind = std::mem::discriminant(self.view.get(inner));
                if contextual.is_some_and(|c| self.contains_literal_kind(c, kind, 0)) {
                    inner
                } else {
                    self.widen_literal(inner)
                }
            }
            Type::Object(shape) => {
                let members: Vec<Member> = shape
                    .members
                    .iter()
                    .map(|m| Member { name: m.name.clone(), ty: m.ty, optional: m.optional })
                    .collect();
                let inexact = shape.inexact;
                let members = members
                    .into_iter()
                    .map(|m| {
                        let ctx = contextual.and_then(|c| self.contextual_member(c, &m.name, 0));
                        Member {
                            ty: self.widen_for_contextual_in(m.ty, ctx, false),
                            name: m.name,
                            optional: m.optional,
                        }
                    })
                    .collect();
                self.view.push(Type::Object(ObjectShape { members, inexact }))
            }
            Type::Tuple(members) => {
                let members: Vec<TypeId> = members.to_vec();
                let members = members
                    .into_iter()
                    .enumerate()
                    .map(|(i, m)| {
                        let ctx = contextual.and_then(|c| self.contextual_element(c, i));
                        self.widen_for_contextual_in(m, ctx, false)
                    })
                    .collect();
                self.view.push(Type::Tuple(members))
            }
            _ => inner,
        }
    }

    /// Contextual type of an object member, looked through the (peeled)
    /// target; for union targets, the first constituent carrying the member.
    fn contextual_member(&self, contextual: TypeId, name: &str, depth: u32) -> Option<TypeId> {
        if depth > 8 {
            return None;
        }
        match self.view.get(self.peel(contextual)?) {
            Type::Object(shape) => shape.members.iter().find(|m| &*m.name == name).map(|m| m.ty),
            Type::Union(members) => {
                members.iter().find_map(|m| self.contextual_member(*m, name, depth + 1))
            }
            _ => None,
        }
    }

    /// Contextual type of an array/tuple element at `index` (readonly
    /// wrappers are transparent; tuple modifiers unwrap).
    fn contextual_element(&self, contextual: TypeId, index: usize) -> Option<TypeId> {
        match self.view.get(self.peel(contextual)?) {
            Type::Array(elem) => Some(*elem),
            Type::Tuple(members) => {
                let elem = members.get(index).copied().or_else(|| {
                    // Past the fixed members: a trailing rest element covers.
                    members
                        .last()
                        .copied()
                        .filter(|m| matches!(self.view.get(*m), Type::RestElem(_)))
                })?;
                match self.view.get(elem) {
                    Type::OptionalElem(inner) => Some(*inner),
                    Type::RestElem(array) => match self.view.get(*array) {
                        Type::Array(e) => Some(*e),
                        _ => None,
                    },
                    _ => Some(elem),
                }
            }
            Type::Readonly(inner) => self.contextual_element(*inner, index),
            _ => None,
        }
    }

    fn contains_literal_kind(
        &self,
        target: TypeId,
        kind: std::mem::Discriminant<Type>,
        depth: u32,
    ) -> bool {
        if depth > 32 {
            return false;
        }
        match self.view.get(target) {
            ty @ (Type::StringLiteral(_)
            | Type::NumberLiteral(_)
            | Type::BooleanLiteral(_)
            | Type::BigIntLiteral(_)) => std::mem::discriminant(ty) == kind,
            Type::Union(members) | Type::Intersection(members) => {
                members.iter().any(|m| self.contains_literal_kind(*m, kind, depth + 1))
            }
            Type::Fresh(inner) | Type::Named { ty: inner, .. } => {
                self.contains_literal_kind(*inner, kind, depth + 1)
            }
            Type::Ref { target: RefTarget::Symbol(symbol), args } if args.is_empty() => {
                match self.env.symbol(*symbol).kind {
                    SymbolKind::TypeAlias { ty, ref params }
                    | SymbolKind::Interface { ty, ref params }
                        if params.is_empty() =>
                    {
                        self.contains_literal_kind(ty, kind, depth + 1)
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// tsc `typeCouldHaveTopLevelSingletonTypes`: does the target contain
    /// unit types (literals, `null`, `undefined`) at the top level, looking
    /// through unions/intersections/aliases?
    #[expect(clippy::match_same_arms)] // unit kinds listed separately on purpose
    fn target_has_units(&self, target: TypeId, depth: u32) -> bool {
        if depth > 32 {
            return false;
        }
        match self.view.get(target) {
            Type::StringLiteral(_)
            | Type::NumberLiteral(_)
            | Type::BooleanLiteral(_)
            | Type::BigIntLiteral(_)
            | Type::Null
            | Type::Undefined => true,
            Type::Union(members) | Type::Intersection(members) => {
                members.iter().any(|m| self.target_has_units(*m, depth + 1))
            }
            Type::Fresh(inner) | Type::Named { ty: inner, .. } => {
                self.target_has_units(*inner, depth + 1)
            }
            Type::EnumMember { .. } => true,
            Type::Ref { target: RefTarget::Symbol(symbol), args } if args.is_empty() => {
                match self.env.symbol(*symbol).kind {
                    SymbolKind::TypeAlias { ty, ref params }
                    | SymbolKind::Interface { ty, ref params }
                        if params.is_empty() =>
                    {
                        self.target_has_units(ty, depth + 1)
                    }
                    // Enum types contain unit (member) types.
                    SymbolKind::Enum { .. } => true,
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// tsc `reportRelationError` source generalization: a literal source
    /// prints as its base primitive when the target is not `never` and has no
    /// top-level unit types.
    fn generalize_source_for_message(&mut self, source: TypeId, target: TypeId) -> TypeId {
        let source = match self.view.get(source) {
            Type::Fresh(inner) => *inner,
            _ => source,
        };
        let target_is_never =
            self.peel(target).is_some_and(|t| matches!(self.view.get(t), Type::Never));
        let keep = target_is_never || self.target_has_units(target, 0);
        // Enum member sources generalize to their enum (tsc's base type).
        if let Type::EnumMember { symbol, .. } = self.view.get(source) {
            let symbol = *symbol;
            return if keep {
                source
            } else {
                self.view.push(Type::Ref { target: RefTarget::Symbol(symbol), args: Box::new([]) })
            };
        }
        if !self.is_literal(source) {
            return source;
        }
        if keep { source } else { self.widen_literal(source) }
    }

    /// Required target members definitely absent from the source — the
    /// TS2741/2739/2740 classification.
    fn missing_props(&self, source: TypeId, target: TypeId) -> Vec<Box<str>> {
        let (Some(s), Some(t)) = (self.peel(source), self.peel(target)) else {
            return Vec::new();
        };
        let (Type::Object(s_shape), Type::Object(t_shape)) = (self.view.get(s), self.view.get(t))
        else {
            return Vec::new();
        };
        if s_shape.inexact {
            return Vec::new();
        }
        t_shape
            .members
            .iter()
            .filter(|tm| !tm.optional && !s_shape.members.iter().any(|sm| sm.name == tm.name))
            .map(|tm| tm.name.clone())
            .collect()
    }

    /// Report a definite relation failure with tsc's code selection.
    fn report_failure(&mut self, source: TypeId, target: TypeId, span: oxc_span::Span, head: Head) {
        // TS4104: readonly source against a mutable array/tuple target.
        if let (Some(s), Some(t)) = (self.peel(source), self.peel(target))
            && matches!(self.view.get(s), Type::Readonly(_))
            && matches!(self.view.get(t), Type::Array(_) | Type::Tuple(_))
        {
            self.diagnostics
                .push(diagnostics::readonly_to_mutable(&self.view, source, target, span));
            return;
        }
        let missing = self.missing_props(source, target);
        let shown = match self.view.get(source) {
            Type::Fresh(inner) => *inner,
            _ => source,
        };
        match missing.len() {
            0 => {
                let shown = self.generalize_source_for_message(source, target);
                let diagnostic = match head {
                    Head::Assign => diagnostics::not_assignable(&self.view, shown, target, span),
                    Head::Satisfies => diagnostics::not_satisfies(&self.view, shown, target, span),
                    Head::Argument => {
                        diagnostics::argument_not_assignable(&self.view, shown, target, span)
                    }
                };
                self.diagnostics.push(diagnostic);
            }
            1 => {
                let name = missing[0].clone();
                self.diagnostics
                    .push(diagnostics::missing_property(&self.view, &name, shown, target, span));
            }
            _ => {
                self.diagnostics.push(diagnostics::missing_properties(
                    &self.view, &missing, shown, target, span,
                ));
            }
        }
    }

    /// The central assignability check with tsc's elaboration: widen the
    /// source for its contextual (target) type, relate, and on failure try to
    /// produce inner errors inside array/object literal expressions before
    /// falling back to the outer error.
    fn check_assignable(
        &mut self,
        expr: Option<&Expression<'_>>,
        source: TypeId,
        target: TypeId,
        outer_span: oxc_span::Span,
        head: Head,
    ) -> bool {
        // Excess property checks are freshness-based and independent of the
        // relation outcome (a passing relation can still have excess props).
        let excess = expr.is_some_and(|expr| self.check_excess(strip_parens(expr), target));
        let checked = self.widen_for_contextual(source, Some(target));
        if relate(&self.view, checked, target) != Relation::False {
            return excess;
        }
        if let Some(expr) = expr
            && self.elaborate(strip_parens(expr), target)
        {
            return true;
        }
        if excess {
            return true; // the excess-property error stands in for the outer error
        }
        self.report_failure(checked, target, outer_span, head);
        true
    }

    /// Bind annotated parameters as value names for the duration of a
    /// function body check; returns the shadowed entries for restoration.
    pub(super) fn bind_params(
        &mut self,
        params: &oxc_ast::ast::FormalParameters<'_>,
    ) -> Vec<(Box<str>, Option<TypeId>)> {
        let mut shadowed = Vec::new();
        for param in &params.items {
            let BindingPattern::BindingIdentifier(id) = &param.pattern else { continue };
            let ty = param
                .type_annotation
                .as_ref()
                .map_or(TypeTable::ANY, |a| self.lower_ts_type(&a.type_annotation));
            let name: Box<str> = id.name.as_str().into();
            shadowed.push((name.clone(), self.value_names.insert(name, ty)));
        }
        shadowed
    }

    /// Restore bindings shadowed by [`Self::bind_params`].
    pub(super) fn unbind_params(&mut self, shadowed: Vec<(Box<str>, Option<TypeId>)>) {
        for (name, previous) in shadowed {
            match previous {
                Some(ty) => {
                    self.value_names.insert(name, ty);
                }
                None => {
                    self.value_names.remove(&name);
                }
            }
        }
    }

    /// Check the body of a function-like with an annotated return type
    /// (shared by function declarations, arrows, and function expressions).
    pub(super) fn check_return_body(
        &mut self,
        target: TypeId,
        body: &oxc_ast::ast::FunctionBody<'_>,
        is_expression_body: bool,
    ) {
        let saved_flow = std::mem::take(&mut self.use_flow);
        if is_expression_body {
            if let Some(Statement::ExpressionStatement(stmt)) = body.statements.first()
                && !matches!(&stmt.expression, Expression::Identifier(_))
            {
                let source = self.infer(&stmt.expression);
                // tsc reports concise-body mismatches at the expression.
                self.check_assignable(
                    Some(&stmt.expression),
                    source,
                    target,
                    stmt.expression.span(),
                    Head::Assign,
                );
            }
        } else {
            let saved_reliable = self.narrow_reliable;
            self.narrow_reliable = true;
            self.narrow_stack.push(FxHashMap::default());
            self.walk_body(&body.statements, target);
            self.narrow_stack.pop();
            self.narrow_reliable = saved_reliable;
        }
        self.use_flow = saved_flow;
    }

    /// Statement-ordered function body walk with narrow tracking.
    fn walk_body(&mut self, statements: &[Statement<'_>], ret: TypeId) {
        for stmt in statements {
            match stmt {
                Statement::VariableDeclaration(var) => self.check_variable_declaration(var),
                Statement::ExpressionStatement(expr_stmt) => {
                    if let Expression::AssignmentExpression(assign) = &expr_stmt.expression {
                        self.check_body_assignment(assign);
                    } else {
                        self.infer(&expr_stmt.expression);
                    }
                }
                Statement::ReturnStatement(ret_stmt) => self.check_return(ret_stmt, ret),
                Statement::BlockStatement(block) => {
                    self.narrow_stack.push(FxHashMap::default());
                    self.walk_body(&block.body, ret);
                    self.narrow_stack.pop();
                }
                Statement::IfStatement(if_stmt) => self.walk_if(if_stmt, ret),
                Statement::SwitchStatement(switch) => self.walk_switch(switch, ret),
                Statement::ForOfStatement(for_of) => {
                    if let oxc_ast::ast::ForStatementLeft::VariableDeclaration(decl) = &for_of.left
                        && let Some(declarator) = decl.declarations.first()
                        && let BindingPattern::BindingIdentifier(id) = &declarator.id
                    {
                        let iterated = self.infer(&for_of.right);
                        let elem = self.contextual_element(iterated, 0).unwrap_or(TypeTable::ANY);
                        self.value_names.insert(id.name.as_str().into(), elem);
                    }
                    self.walk_loop_body(std::slice::from_ref(&for_of.body), ret);
                }
                Statement::ForStatement(s) => {
                    self.walk_loop_body(std::slice::from_ref(&s.body), ret);
                }
                Statement::ForInStatement(s) => {
                    self.walk_loop_body(std::slice::from_ref(&s.body), ret);
                }
                Statement::WhileStatement(s) => {
                    self.walk_loop_body(std::slice::from_ref(&s.body), ret);
                }
                Statement::DoWhileStatement(s) => {
                    self.walk_loop_body(std::slice::from_ref(&s.body), ret);
                }
                Statement::LabeledStatement(labeled) => {
                    self.walk_body(std::slice::from_ref(&labeled.body), ret);
                }
                Statement::TryStatement(try_stmt) => {
                    let saved = self.narrow_reliable;
                    self.narrow_reliable = false;
                    self.narrow_stack.push(FxHashMap::default());
                    self.walk_body(&try_stmt.block.body, ret);
                    self.narrow_stack.pop();
                    if let Some(handler) = &try_stmt.handler {
                        self.narrow_stack.push(FxHashMap::default());
                        self.walk_body(&handler.body.body, ret);
                        self.narrow_stack.pop();
                    }
                    if let Some(finalizer) = &try_stmt.finalizer {
                        self.narrow_stack.push(FxHashMap::default());
                        self.walk_body(&finalizer.body, ret);
                        self.narrow_stack.pop();
                    }
                    self.narrow_reliable = saved;
                }
                _ => {}
            }
        }
    }

    /// Loop bodies merge over the back edge — narrowing is unreliable inside.
    fn walk_loop_body(&mut self, statements: &[Statement<'_>], ret: TypeId) {
        let saved = self.narrow_reliable;
        self.narrow_reliable = false;
        self.narrow_stack.push(FxHashMap::default());
        self.walk_body(statements, ret);
        self.narrow_stack.pop();
        self.narrow_reliable = saved;
    }

    fn check_return(&mut self, ret_stmt: &ReturnStatement<'_>, target: TypeId) {
        let source = ret_stmt.argument.as_ref().map_or(TypeTable::UNDEFINED, |arg| self.infer(arg));
        if matches!(&ret_stmt.argument, Some(Expression::Identifier(_))) && !self.narrow_reliable {
            // Conservative fallback inside unmodeled guards: only definitely
            // un-narrowable shapes against unit-free targets.
            let narrowable = match self.peel(source) {
                None => true,
                Some(p) => matches!(self.view.get(p), Type::Union(_) | Type::Boolean),
            };
            if narrowable
                || self.target_has_units(target, 0)
                || self.peel(target).is_some_and(|t| matches!(self.view.get(t), Type::Never))
            {
                return;
            }
        }
        self.check_assignable(
            ret_stmt.argument.as_ref(),
            source,
            target,
            ret_stmt.span,
            Head::Assign,
        );
    }

    /// `name = expr` inside a function body: check against the declared type
    /// and update the narrow state.
    fn check_body_assignment(&mut self, assign: &oxc_ast::ast::AssignmentExpression<'_>) {
        use oxc_ast::ast::{AssignmentOperator, AssignmentTarget};
        let source = self.infer(&assign.right);
        let AssignmentTarget::AssignmentTargetIdentifier(ident) = &assign.left else { return };
        let name = ident.name.as_str();
        let declared = self.value_names.get(name).copied();
        if assign.operator != AssignmentOperator::Assign {
            if let Some(scope) = self.narrow_stack.last_mut() {
                scope.remove(name);
            }
            return;
        }
        if let Some(declared) = declared {
            let rhs_identifier = matches!(strip_parens(&assign.right), Expression::Identifier(_));
            if !rhs_identifier || self.narrow_reliable {
                self.check_assignable(
                    Some(&assign.right),
                    source,
                    declared,
                    ident.span,
                    Head::Assign,
                );
            }
            // Assignment narrows to the assigned type when it fits, else the
            // declared type stands.
            let regular = self.regular_type(source);
            let narrowed = if relate(&self.view, regular, declared) == Relation::True {
                regular
            } else {
                declared
            };
            if let Some(scope) = self.narrow_stack.last_mut() {
                scope.insert(name.into(), narrowed);
            }
        }
    }

    /// The current (possibly narrowed) type of a binding.
    fn current_type_of(&self, name: &str) -> Option<TypeId> {
        for scope in self.narrow_stack.iter().rev() {
            if let Some(&ty) = scope.get(name) {
                return Some(ty);
            }
        }
        self.value_names.get(name).copied()
    }

    fn walk_if(&mut self, if_stmt: &oxc_ast::ast::IfStatement<'_>, ret: TypeId) {
        if let Some((name, kind, positive)) = parse_typeof_guard(&if_stmt.test)
            && let Some(current) = self.current_type_of(name)
            && let (Some(then_ty), Some(else_ty)) = (
                self.narrow_by_typeof(current, kind, positive),
                self.narrow_by_typeof(current, kind, !positive),
            )
        {
            let mut scope = FxHashMap::default();
            scope.insert(Box::from(name), then_ty);
            self.narrow_stack.push(scope);
            self.walk_body(std::slice::from_ref(&if_stmt.consequent), ret);
            self.narrow_stack.pop();
            if let Some(alternate) = &if_stmt.alternate {
                let mut scope = FxHashMap::default();
                scope.insert(Box::from(name), else_ty);
                self.narrow_stack.push(scope);
                self.walk_body(std::slice::from_ref(alternate), ret);
                self.narrow_stack.pop();
            }
            // Assertion-style narrowing: a definitely-returning branch
            // applies its complement to the rest of the body.
            if statement_returns(&if_stmt.consequent) == Returns::Yes {
                if let Some(scope) = self.narrow_stack.last_mut() {
                    scope.insert(Box::from(name), else_ty);
                }
            } else if if_stmt
                .alternate
                .as_ref()
                .is_some_and(|a| statement_returns(a) == Returns::Yes)
                && let Some(scope) = self.narrow_stack.last_mut()
            {
                scope.insert(Box::from(name), then_ty);
            }
            return;
        }
        // Unmodeled guard: walk branches conservatively.
        let saved = self.narrow_reliable;
        self.narrow_reliable = false;
        self.infer(&if_stmt.test);
        self.narrow_stack.push(FxHashMap::default());
        self.walk_body(std::slice::from_ref(&if_stmt.consequent), ret);
        self.narrow_stack.pop();
        if let Some(alternate) = &if_stmt.alternate {
            self.narrow_stack.push(FxHashMap::default());
            self.walk_body(std::slice::from_ref(alternate), ret);
            self.narrow_stack.pop();
        }
        self.narrow_reliable = saved;
    }

    /// Filter a union by a `typeof` test. `None` means a constituent was
    /// unclassifiable (narrowing would be unsound).
    fn narrow_by_typeof(&mut self, ty: TypeId, kind: TypeofKind, keep: bool) -> Option<TypeId> {
        let peeled = self.peel(ty)?;
        let members: Vec<TypeId> = match self.view.get(peeled) {
            Type::Union(members) => members.to_vec(),
            _ => vec![peeled],
        };
        let mut kept = Vec::new();
        for member in members {
            let class = self.classify_typeof(member)?;
            if (class == kind) == keep {
                kept.push(member);
            }
        }
        Some(match kept.len() {
            0 => TypeTable::NEVER,
            1 => kept[0],
            _ => self.view.push(Type::Union(kept.into_boxed_slice())),
        })
    }

    fn classify_typeof(&self, member: TypeId) -> Option<TypeofKind> {
        let peeled = self.peel(member)?;
        Some(match self.view.get(peeled) {
            Type::String | Type::StringLiteral(_) => TypeofKind::String,
            Type::Number | Type::NumberLiteral(_) => TypeofKind::Number,
            Type::Boolean | Type::BooleanLiteral(_) => TypeofKind::Boolean,
            Type::BigInt | Type::BigIntLiteral(_) => TypeofKind::BigInt,
            Type::Undefined | Type::Void => TypeofKind::Undefined,
            Type::Symbol => TypeofKind::Symbol,
            Type::Function(_) => TypeofKind::Function,
            Type::Null
            | Type::Object(_)
            | Type::Array(_)
            | Type::Tuple(_)
            | Type::Readonly(_)
            | Type::ObjectKeyword => TypeofKind::Object,
            Type::EnumMember { symbol, index } => {
                if matches!(
                    &self.env.symbol(*symbol).kind,
                    SymbolKind::Enum { members } if members.get(*index as usize).is_some_and(|m| m.is_string)
                ) {
                    TypeofKind::String
                } else {
                    TypeofKind::Number
                }
            }
            _ => return None,
        })
    }

    fn walk_switch(&mut self, switch: &oxc_ast::ast::SwitchStatement<'_>, ret: TypeId) {
        if let Some((name, arms)) = self.discriminant_switch_info(&switch.discriminant) {
            let mut matched = vec![false; arms.len()];
            for case in &switch.cases {
                let narrowed = match &case.test {
                    Some(test) => {
                        let literal_ty = self.infer(test);
                        let literal = self.regular_type(literal_ty);
                        let mut kept = Vec::new();
                        for (i, (constituent, prop_ty)) in arms.iter().enumerate() {
                            if relate(&self.view, literal, *prop_ty) == Relation::True {
                                matched[i] = true;
                                kept.push(*constituent);
                            }
                        }
                        kept
                    }
                    None => arms
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| !matched[*i])
                        .map(|(_, (c, _))| *c)
                        .collect(),
                };
                let ty = match narrowed.len() {
                    0 => TypeTable::NEVER,
                    1 => narrowed[0],
                    _ => self.view.push(Type::Union(narrowed.into_boxed_slice())),
                };
                let mut scope = FxHashMap::default();
                scope.insert(Box::from(name.as_ref()), ty);
                self.narrow_stack.push(scope);
                self.walk_body(&case.consequent, ret);
                self.narrow_stack.pop();
            }
            return;
        }
        // Non-discriminant switch: conservative.
        let saved = self.narrow_reliable;
        self.narrow_reliable = false;
        self.infer(&switch.discriminant);
        for case in &switch.cases {
            if let Some(test) = &case.test {
                self.infer(test);
            }
            self.narrow_stack.push(FxHashMap::default());
            self.walk_body(&case.consequent, ret);
            self.narrow_stack.pop();
        }
        self.narrow_reliable = saved;
    }

    /// `switch (x.prop)` where `x` is a union of shapes all carrying a
    /// unit-typed `prop`: the discriminant arms.
    #[expect(clippy::type_complexity)]
    fn discriminant_switch_info(
        &self,
        discriminant: &Expression<'_>,
    ) -> Option<(Box<str>, Vec<(TypeId, TypeId)>)> {
        let Expression::StaticMemberExpression(member) = strip_parens(discriminant) else {
            return None;
        };
        let Expression::Identifier(object) = &member.object else { return None };
        let name = object.name.as_str();
        let current = self.current_type_of(name)?;
        let peeled = self.peel(current)?;
        let Type::Union(constituents) = self.view.get(peeled) else { return None };
        let constituents = constituents.to_vec();
        let prop = member.property.name.as_str();
        let mut arms = Vec::new();
        for constituent in constituents {
            let shape_id = self.peel(constituent)?;
            let Type::Object(shape) = self.view.get(shape_id) else { return None };
            let member_ty = shape.members.iter().find(|m| &*m.name == prop)?.ty;
            if !self.target_has_units(member_ty, 0) {
                return None;
            }
            arms.push((constituent, member_ty));
        }
        Some((name.into(), arms))
    }

    /// Recursive excess-property check over object/array literal expressions
    /// (tsc reports TS2353 at each unknown property of a fresh literal).
    fn check_excess(&mut self, expr: &Expression<'_>, target: TypeId) -> bool {
        use oxc_ast::ast::{ArrayExpressionElement, ObjectPropertyKind, PropertyKind};
        match expr {
            Expression::ObjectExpression(object) => {
                // Discriminated union targets are handled by the union
                // elaboration (excess is checked against the selected arm).
                if let Some(peeled) = self.peel(target)
                    && let Type::Union(constituents) = self.view.get(peeled)
                {
                    let constituents = constituents.to_vec();
                    if self.has_union_discriminant(object, &constituents) {
                        return false;
                    }
                }
                let mut reported = false;
                if let Some(known) = self.known_member_names(target, 0) {
                    for property in &object.properties {
                        let ObjectPropertyKind::ObjectProperty(prop) = property else { continue };
                        if prop.computed {
                            continue;
                        }
                        let Some(name) = crate::lower::property_key_name(&prop.key) else {
                            continue;
                        };
                        if !known.contains(&name) {
                            self.diagnostics.push(diagnostics::excess_property(
                                &self.view,
                                &name,
                                target,
                                prop.key.span(),
                            ));
                            reported = true;
                        }
                    }
                }
                // Recurse into known members.
                let members: Vec<(Box<str>, TypeId)> =
                    match self.peel(target).map(|p| self.view.get(p)) {
                        Some(Type::Object(shape)) => {
                            shape.members.iter().map(|m| (m.name.clone(), m.ty)).collect()
                        }
                        _ => Vec::new(),
                    };
                for property in &object.properties {
                    let ObjectPropertyKind::ObjectProperty(prop) = property else { continue };
                    if prop.computed || prop.kind != PropertyKind::Init {
                        continue;
                    }
                    let Some(name) = crate::lower::property_key_name(&prop.key) else { continue };
                    if let Some((_, member_ty)) = members.iter().find(|(n, _)| *n == name) {
                        reported |= self.check_excess(strip_parens(&prop.value), *member_ty);
                    }
                }
                reported
            }
            Expression::ArrayExpression(array) => {
                let mut reported = false;
                for (i, element) in array.elements.iter().enumerate() {
                    let element = match element {
                        ArrayExpressionElement::SpreadElement(_)
                        | ArrayExpressionElement::Elision(_) => continue,
                        _ => element.to_expression(),
                    };
                    if let Some(elem_target) = self.contextual_element(target, i) {
                        reported |= self.check_excess(strip_parens(element), elem_target);
                    }
                }
                reported
            }
            _ => false,
        }
    }

    /// Elaborate into a literal expression; returns true when at least one
    /// inner error was reported (the outer error is then suppressed).
    fn elaborate(&mut self, expr: &Expression<'_>, target: TypeId) -> bool {
        match expr {
            Expression::ArrayExpression(array) => self.elaborate_array(array, target),
            Expression::ObjectExpression(object) => self.elaborate_object(object, target),
            _ => false,
        }
    }

    fn elaborate_array(
        &mut self,
        array: &oxc_ast::ast::ArrayExpression<'_>,
        target: TypeId,
    ) -> bool {
        use oxc_ast::ast::ArrayExpressionElement;
        let Some(peeled) = self.peel(target) else { return false };
        #[expect(clippy::items_after_statements)]
        enum Elems {
            Array(TypeId),
            Tuple(Vec<TypeId>),
        }
        let elems = match self.view.get(peeled) {
            Type::Array(elem) => Elems::Array(*elem),
            Type::Tuple(members) => Elems::Tuple(members.to_vec()),
            Type::Readonly(inner) => match self.view.get(*inner) {
                Type::Array(elem) => Elems::Array(*elem),
                Type::Tuple(members) => Elems::Tuple(members.to_vec()),
                _ => return false,
            },
            _ => return false,
        };
        let mut reported = false;
        for (i, element) in array.elements.iter().enumerate() {
            let element = match element {
                ArrayExpressionElement::SpreadElement(_) | ArrayExpressionElement::Elision(_) => {
                    continue;
                }
                _ => element.to_expression(),
            };
            let elem_target = match &elems {
                Elems::Array(e) => *e,
                // Optional elements unwrap; rest-covered positions are not
                // elaborated (tsc reports the outer tuple error instead), and
                // excess elements have no target slot.
                Elems::Tuple(ms) => match ms.get(i) {
                    Some(t) => match self.view.get(*t) {
                        Type::OptionalElem(inner) => *inner,
                        Type::RestElem(_) => continue,
                        _ => *t,
                    },
                    None => continue,
                },
            };
            let source = self.infer(element);
            let checked = self.widen_for_contextual(source, Some(elem_target));
            if relate(&self.view, checked, elem_target) == Relation::False {
                let element = strip_parens(element);
                if !self.elaborate(element, elem_target) {
                    self.report_failure(checked, elem_target, element.span(), Head::Assign);
                }
                reported = true;
            }
        }
        reported
    }

    fn elaborate_object(
        &mut self,
        object: &oxc_ast::ast::ObjectExpression<'_>,
        target: TypeId,
    ) -> bool {
        use oxc_ast::ast::{ObjectPropertyKind, PropertyKind};
        let mut reported = false;

        // (Excess properties are handled by `check_excess`, independently of
        // the relation outcome.)

        // Union targets elaborate through discriminant matching.
        if let Some(peeled) = self.peel(target)
            && let Type::Union(constituents) = self.view.get(peeled)
        {
            let constituents = constituents.to_vec();
            return self.elaborate_object_union(object, &constituents) || reported;
        }
        // Member-wise elaboration needs a single object target.
        let Some(peeled) = self.peel(target) else { return reported };
        let Type::Object(shape) = self.view.get(peeled) else { return reported };
        let members: Vec<Member> = shape
            .members
            .iter()
            .map(|m| Member { name: m.name.clone(), ty: m.ty, optional: m.optional })
            .collect();
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(prop) = property else { continue };
            if prop.computed || prop.kind != PropertyKind::Init {
                continue;
            }
            let Some(name) = crate::lower::property_key_name(&prop.key) else { continue };
            let Some(member) = members.iter().find(|m| m.name == name) else { continue };
            let member_ty = member.ty;
            let source = self.infer(&prop.value);
            let checked = self.widen_for_contextual(source, Some(member_ty));
            if relate(&self.view, checked, member_ty) == Relation::False {
                let value = strip_parens(&prop.value);
                if !self.elaborate(value, member_ty) {
                    // tsc reports property mismatches at the property name.
                    self.report_failure(checked, member_ty, prop.key.span(), Head::Assign);
                }
                reported = true;
            }
        }
        reported
    }

    /// tsc's discriminant-directed union elaboration: literal-valued
    /// properties present as unit types in every constituent select the arm.
    fn elaborate_object_union(
        &mut self,
        object: &oxc_ast::ast::ObjectExpression<'_>,
        constituents: &[TypeId],
    ) -> bool {
        use oxc_ast::ast::{ObjectPropertyKind, PropertyKind};
        // Constituent shapes (bail if any is not an object shape).
        let mut shapes: Vec<(TypeId, Vec<Member>)> = Vec::new();
        for &constituent in constituents {
            let Some(peeled) = self.peel(constituent) else { return false };
            let Type::Object(shape) = self.view.get(peeled) else { return false };
            let members = shape
                .members
                .iter()
                .map(|m| Member { name: m.name.clone(), ty: m.ty, optional: m.optional })
                .collect();
            shapes.push((constituent, members));
        }

        // Discriminant properties: literal-valued source props that exist as
        // unit-ish members in every constituent.
        let mut surviving: Vec<bool> = vec![true; shapes.len()];
        let mut any_discriminant = false;
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(prop) = property else { continue };
            if prop.computed || prop.kind != PropertyKind::Init {
                continue;
            }
            let Some(name) = crate::lower::property_key_name(&prop.key) else { continue };
            let value_ty = self.infer(&prop.value);
            let literal = self.regular_type(value_ty);
            if !self.is_literal(self.peel(literal).unwrap_or(literal)) {
                continue;
            }
            let member_tys: Option<Vec<TypeId>> = shapes
                .iter()
                .map(|(_, members)| members.iter().find(|m| m.name == name).map(|m| m.ty))
                .collect();
            let Some(member_tys) = member_tys else { continue };
            if !member_tys.iter().all(|&t| self.target_has_units(t, 0)) {
                continue;
            }
            any_discriminant = true;
            let mut prop_matches = vec![false; shapes.len()];
            let mut any_match = false;
            for (i, &member_ty) in member_tys.iter().enumerate() {
                if relate(&self.view, literal, member_ty) == Relation::True {
                    prop_matches[i] = true;
                    any_match = true;
                }
            }
            if !any_match {
                // No arm accepts this discriminant: report at the value with
                // the union of the arms' member types.
                let mut union: Vec<TypeId> = Vec::new();
                for &t in &member_tys {
                    if !union.contains(&t) {
                        union.push(t);
                    }
                }
                let union_ty = if union.len() == 1 {
                    union[0]
                } else {
                    self.view.push(Type::Union(union.into_boxed_slice()))
                };
                let shown = self.generalize_source_for_message(literal, union_ty);
                self.diagnostics.push(diagnostics::not_assignable(
                    &self.view,
                    shown,
                    union_ty,
                    prop.key.span(),
                ));
                return true;
            }
            for (s, m) in surviving.iter_mut().zip(prop_matches) {
                *s &= m;
            }
        }
        if !any_discriminant {
            return false;
        }
        let matching: Vec<usize> = (0..shapes.len()).filter(|&i| surviving[i]).collect();
        if matching.len() != 1 {
            return false;
        }
        // Exactly one arm: excess + member elaboration against it, printing
        // the arm type.
        let arm = shapes[matching[0]].0;
        let mut reported = self.check_excess_against(object, arm);
        let members = shapes[matching[0]].1.clone();
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(prop) = property else { continue };
            if prop.computed || prop.kind != PropertyKind::Init {
                continue;
            }
            let Some(name) = crate::lower::property_key_name(&prop.key) else { continue };
            let Some(member) = members.iter().find(|m| m.name == name) else { continue };
            let member_ty = member.ty;
            let source = self.infer(&prop.value);
            let checked = self.widen_for_contextual(source, Some(member_ty));
            if relate(&self.view, checked, member_ty) == Relation::False {
                let value = strip_parens(&prop.value);
                if !self.elaborate(value, member_ty) {
                    self.report_failure(checked, member_ty, prop.key.span(), Head::Assign);
                }
                reported = true;
            }
        }
        reported
    }

    /// Excess properties of one object literal against one concrete target
    /// (the discriminant-selected arm).
    fn check_excess_against(
        &mut self,
        object: &oxc_ast::ast::ObjectExpression<'_>,
        target: TypeId,
    ) -> bool {
        use oxc_ast::ast::ObjectPropertyKind;
        let Some(known) = self.known_member_names(target, 0) else { return false };
        let mut reported = false;
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(prop) = property else { continue };
            if prop.computed {
                continue;
            }
            let Some(name) = crate::lower::property_key_name(&prop.key) else { continue };
            if !known.contains(&name) {
                self.diagnostics.push(diagnostics::excess_property(
                    &self.view,
                    &name,
                    target,
                    prop.key.span(),
                ));
                reported = true;
            }
        }
        reported
    }

    /// Whether any property of the literal acts as a union discriminant —
    /// when true, generic union excess checks defer to the discriminant
    /// elaboration.
    fn has_union_discriminant(
        &self,
        object: &oxc_ast::ast::ObjectExpression<'_>,
        constituents: &[TypeId],
    ) -> bool {
        use oxc_ast::ast::{ObjectPropertyKind, PropertyKind};
        let mut shapes: Vec<Vec<Member>> = Vec::new();
        for &constituent in constituents {
            let Some(peeled) = self.peel(constituent) else { return false };
            let Type::Object(shape) = self.view.get(peeled) else { return false };
            shapes.push(
                shape
                    .members
                    .iter()
                    .map(|m| Member { name: m.name.clone(), ty: m.ty, optional: m.optional })
                    .collect(),
            );
        }
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(prop) = property else { continue };
            if prop.computed || prop.kind != PropertyKind::Init {
                continue;
            }
            let Some(name) = crate::lower::property_key_name(&prop.key) else { continue };
            if !matches!(
                strip_parens(&prop.value),
                Expression::StringLiteral(_)
                    | Expression::NumericLiteral(_)
                    | Expression::BooleanLiteral(_)
            ) {
                continue;
            }
            let all_unit = shapes.iter().all(|members| {
                members
                    .iter()
                    .find(|m| m.name == name)
                    .is_some_and(|m| self.target_has_units(m.ty, 0))
            });
            if all_unit {
                return true;
            }
        }
        false
    }

    /// The full member-name set of the target, when knowable (exact object,
    /// or union of exact objects) — gates the excess property check.
    fn known_member_names(&self, target: TypeId, depth: u32) -> Option<Vec<Box<str>>> {
        if depth > 8 {
            return None;
        }
        match self.view.get(self.peel(target)?) {
            Type::Object(shape) => {
                if shape.inexact {
                    return None;
                }
                Some(shape.members.iter().map(|m| m.name.clone()).collect())
            }
            Type::Union(members) => {
                let members = members.to_vec();
                let mut names = Vec::new();
                for member in members {
                    names.extend(self.known_member_names(member, depth + 1)?);
                }
                Some(names)
            }
            _ => None,
        }
    }
}

/// Outer head message for a failed relation.
#[derive(Clone, Copy)]
enum Head {
    /// TS2322.
    Assign,
    /// TS1360 (`satisfies`).
    Satisfies,
    /// TS2345 (call argument).
    Argument,
}

/// `typeof x` result classes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TypeofKind {
    String,
    Number,
    Boolean,
    BigInt,
    Undefined,
    Symbol,
    Function,
    Object,
}

/// Parse `typeof IDENT === "kind"` (and `!==`/`==`/`!=`) guards.
fn parse_typeof_guard<'b>(test: &'b Expression<'_>) -> Option<(&'b str, TypeofKind, bool)> {
    use oxc_ast::ast::{BinaryOperator, UnaryOperator};
    let Expression::BinaryExpression(binary) = test else { return None };
    let positive = match binary.operator {
        BinaryOperator::StrictEquality | BinaryOperator::Equality => true,
        BinaryOperator::StrictInequality | BinaryOperator::Inequality => false,
        _ => return None,
    };
    #[expect(clippy::match_same_arms)] // operand order symmetry
    let (unary, literal) = match (&binary.left, &binary.right) {
        (Expression::UnaryExpression(u), Expression::StringLiteral(s)) => (u, s),
        (Expression::StringLiteral(s), Expression::UnaryExpression(u)) => (u, s),
        _ => return None,
    };
    if unary.operator != UnaryOperator::Typeof {
        return None;
    }
    let Expression::Identifier(ident) = &unary.argument else { return None };
    let kind = match literal.value.as_str() {
        "string" => TypeofKind::String,
        "number" => TypeofKind::Number,
        "boolean" => TypeofKind::Boolean,
        "bigint" => TypeofKind::BigInt,
        "undefined" => TypeofKind::Undefined,
        "symbol" => TypeofKind::Symbol,
        "function" => TypeofKind::Function,
        "object" => TypeofKind::Object,
        _ => return None,
    };
    Some((ident.name.as_str(), kind, positive))
}

/// Conservative "does this statement list definitely return/throw" — `Maybe`
/// keeps unported constructs (switch exhaustiveness, try/finally, infinite
/// loops with breaks) silent.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Returns {
    Yes,
    No,
    Maybe,
}

fn definitely_returns(statements: &[Statement<'_>]) -> Returns {
    let mut result = Returns::No;
    for stmt in statements {
        match statement_returns(stmt) {
            Returns::Yes => return Returns::Yes,
            Returns::Maybe => result = Returns::Maybe,
            Returns::No => {}
        }
    }
    result
}

#[expect(clippy::match_same_arms)] // loop kinds listed separately on purpose
fn statement_returns(stmt: &Statement<'_>) -> Returns {
    match stmt {
        Statement::ReturnStatement(_) | Statement::ThrowStatement(_) => Returns::Yes,
        Statement::BlockStatement(block) => definitely_returns(&block.body),
        Statement::IfStatement(if_stmt) => {
            let Some(alternate) = &if_stmt.alternate else { return Returns::No };
            let consequent = statement_returns(&if_stmt.consequent);
            let alternate = statement_returns(alternate);
            match (consequent, alternate) {
                (Returns::Yes, Returns::Yes) => Returns::Yes,
                (Returns::No, Returns::No) => Returns::No,
                _ => Returns::Maybe,
            }
        }
        // `while (true)` / `for (;;)` only exit via return/throw/break —
        // break analysis is unported, so: Maybe.
        Statement::WhileStatement(while_stmt) => match &while_stmt.test {
            Expression::BooleanLiteral(b) if b.value => Returns::Maybe,
            _ => Returns::No,
        },
        Statement::ForStatement(for_stmt) if for_stmt.test.is_none() => Returns::Maybe,
        Statement::ForStatement(_)
        | Statement::ForOfStatement(_)
        | Statement::ForInStatement(_)
        | Statement::DoWhileStatement(_) => Returns::No,
        // Exhaustiveness analysis is unported.
        Statement::SwitchStatement(_) | Statement::TryStatement(_) => Returns::Maybe,
        Statement::LabeledStatement(labeled) => statement_returns(&labeled.body),
        _ => Returns::No,
    }
}

/// tsc prints resolved module paths without the extension.
fn module_display_path(path: &std::path::Path) -> String {
    let s = path.to_string_lossy();
    for ext in [".d.ts", ".d.mts", ".d.cts", ".ts", ".tsx", ".mts", ".cts"] {
        if let Some(stripped) = s.strip_suffix(ext) {
            return stripped.to_string();
        }
    }
    s.into_owned()
}

fn strip_parens<'b, 'a>(expr: &'b Expression<'a>) -> &'b Expression<'a> {
    let mut expr = expr;
    while let Expression::ParenthesizedExpression(paren) = expr {
        expr = &paren.expression;
    }
    expr
}

/// Collect `return` statements belonging to this function body, skipping
/// nested function/class statements (function *expressions* live inside
/// expressions, which this walk does not descend into).
fn collect_returns<'b, 'a>(
    statements: &'b [Statement<'a>],
    out: &mut Vec<&'b ReturnStatement<'a>>,
) {
    for stmt in statements {
        match stmt {
            Statement::ReturnStatement(ret) => out.push(ret),
            Statement::BlockStatement(block) => collect_returns(&block.body, out),
            Statement::IfStatement(if_stmt) => {
                collect_returns(std::slice::from_ref(&if_stmt.consequent), out);
                if let Some(alternate) = &if_stmt.alternate {
                    collect_returns(std::slice::from_ref(alternate), out);
                }
            }
            Statement::ForStatement(for_stmt) => {
                collect_returns(std::slice::from_ref(&for_stmt.body), out);
            }
            Statement::ForInStatement(for_stmt) => {
                collect_returns(std::slice::from_ref(&for_stmt.body), out);
            }
            Statement::ForOfStatement(for_stmt) => {
                collect_returns(std::slice::from_ref(&for_stmt.body), out);
            }
            Statement::WhileStatement(while_stmt) => {
                collect_returns(std::slice::from_ref(&while_stmt.body), out);
            }
            Statement::DoWhileStatement(do_stmt) => {
                collect_returns(std::slice::from_ref(&do_stmt.body), out);
            }
            Statement::SwitchStatement(switch) => {
                for case in &switch.cases {
                    collect_returns(&case.consequent, out);
                }
            }
            Statement::TryStatement(try_stmt) => {
                collect_returns(&try_stmt.block.body, out);
                if let Some(handler) = &try_stmt.handler {
                    collect_returns(&handler.body.body, out);
                }
                if let Some(finalizer) = &try_stmt.finalizer {
                    collect_returns(&finalizer.body, out);
                }
            }
            Statement::LabeledStatement(labeled) => {
                collect_returns(std::slice::from_ref(&labeled.body), out);
            }
            _ => {}
        }
    }
}
