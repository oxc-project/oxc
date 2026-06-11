//! Linking (single-threaded, cheap): turn per-file surfaces into the frozen
//! [`ProgramEnv`] shared by all checking threads.
//!
//! Everything tsgo's checker does lazily per-checker (alias chasing, star
//! re-export expansion, export tables) happens here once, eagerly — the module
//! graph is frozen by the time checking starts.

use std::path::PathBuf;

use oxc_diagnostics::OxcDiagnostic;
use rustc_hash::FxHashMap;

use crate::{
    ir::{
        FileId, INTRINSIC_COUNT, ObjectShape, RefTarget, SymbolData, SymbolId, SymbolKind, Type,
        TypeId, TypeTable,
    },
    loader::{FileKind, LoadedFile, RawResolution},
    surface::{ImportBindingKind, SurfaceDeclKind, SurfaceExport, SurfaceReexport},
};

/// Resolution of one module specifier, file-id based (post-link).
#[derive(Debug, Clone, Copy)]
pub enum ModuleResolution {
    /// Resolved to a program file.
    File(FileId),
    /// Resolved outside the typed program (plain `.js`, `.json`, ...).
    External,
    /// Not resolved → TS2307 at use sites.
    NotFound,
}

/// One name in a module's export table. A single export name can carry a
/// value meaning, a type meaning, or both (class/enum).
#[derive(Debug, Clone, Copy, Default)]
pub struct ExportEntry {
    /// Value-meaning symbol.
    pub value: Option<SymbolId>,
    /// Type-meaning symbol.
    pub ty: Option<SymbolId>,
}

/// Per-file linked data.
#[derive(Debug)]
pub struct FileData {
    /// Absolute path.
    pub path: PathBuf,
    /// Source text (pass B re-parses it; diagnostics render against it).
    pub source_text: String,
    /// Classification.
    pub kind: FileKind,
    /// Whether parsing failed (file is skipped by pass B).
    pub parse_failed: bool,
    /// Export table.
    pub exports: FxHashMap<Box<str>, ExportEntry>,
    /// Exports cannot be enumerated (`export =`, parse failure, star
    /// re-export of an opaque module). Imports from it are `any`, silently.
    pub opaque_exports: bool,
    /// Surface declarations by name — pass B resolves own-file type
    /// annotations against this before falling back to inline lowering.
    pub local_type_names: FxHashMap<Box<str>, SymbolId>,
    /// Specifier → resolution for every module reference in the file.
    pub resolutions: FxHashMap<Box<str>, ModuleResolution>,
    /// Pass-A diagnostics (parse errors, forced isolated-declarations
    /// violations).
    pub diagnostics: Vec<OxcDiagnostic>,
}

/// The frozen program environment: the only data checking threads share.
#[derive(Debug)]
pub struct ProgramEnv {
    /// All files, indexed by [`FileId`].
    pub files: Vec<FileData>,
    /// All symbols, indexed by [`SymbolId`].
    pub symbols: Vec<SymbolData>,
    /// The global type table.
    pub types: TypeTable,
    /// Effective `strictNullChecks`.
    pub strict_null_checks: bool,
}

// The whole point of the IR: the environment crosses threads by `&` only.
const _: fn() = || {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ProgramEnv>();
};

impl ProgramEnv {
    /// Look up a symbol.
    pub fn symbol(&self, id: SymbolId) -> &SymbolData {
        &self.symbols[id.index()]
    }

    /// Look up a file.
    pub fn file(&self, id: FileId) -> &FileData {
        &self.files[id.index()]
    }
}

/// Link loaded files into a [`ProgramEnv`].
pub fn link(loaded: Vec<LoadedFile>, strict_null_checks: bool) -> ProgramEnv {
    let path_to_file: FxHashMap<PathBuf, FileId> = loaded
        .iter()
        .enumerate()
        .map(|(i, file)| (file.path.clone(), FileId(u32::try_from(i).unwrap())))
        .collect();

    let mut types = TypeTable::new();
    let mut symbols: Vec<SymbolData> = Vec::new();
    // Interface symbol → relocated `extends` reference type ids, flattened
    // after the ref-rewrite pass.
    let mut iface_extends: FxHashMap<SymbolId, Vec<TypeId>> = FxHashMap::default();
    let mut files: Vec<FileData> = Vec::with_capacity(loaded.len());
    // Per file: surface decl index → SymbolId, and the global id range of its
    // relocated types.
    let mut file_symbols: Vec<Vec<SymbolId>> = Vec::with_capacity(loaded.len());
    let mut file_type_bases: Vec<u32> = Vec::with_capacity(loaded.len());

    // Step 1: relocate types, create symbols, build direct export tables.
    for (index, file) in loaded.iter().enumerate() {
        let file_id = FileId(u32::try_from(index).unwrap());
        let base = types.len();
        file_type_bases.push(base);
        let relocate = |id: TypeId| relocate_id(id, base);
        for ty in &file.surface.types {
            types.push(relocate_type(ty, relocate));
        }

        let mut decl_symbols = Vec::with_capacity(file.surface.decls.len());
        let mut local_type_names = FxHashMap::default();
        for decl in &file.surface.decls {
            let kind = match &decl.kind {
                SurfaceDeclKind::Value { ty } => SymbolKind::Value { ty: relocate(*ty) },
                SurfaceDeclKind::TypeAlias { ty, params } => SymbolKind::TypeAlias {
                    ty: relocate(*ty),
                    params: relocate_params(params, relocate),
                },
                SurfaceDeclKind::Interface { ty, params, extends } => {
                    if !extends.is_empty() {
                        iface_extends.insert(
                            SymbolId(u32::try_from(symbols.len()).unwrap()),
                            extends.iter().map(|e| relocate(*e)).collect(),
                        );
                    }
                    SymbolKind::Interface {
                        ty: relocate(*ty),
                        params: relocate_params(params, relocate),
                    }
                }
                SurfaceDeclKind::Class { instance } => {
                    SymbolKind::Class { instance: relocate(*instance) }
                }
                SurfaceDeclKind::Enum { members } => {
                    SymbolKind::Enum { members: members.clone().into_boxed_slice() }
                }
                SurfaceDeclKind::Namespace => SymbolKind::Namespace,
            };
            let symbol_id = SymbolId(u32::try_from(symbols.len()).unwrap());
            symbols.push(SymbolData {
                name: decl.name.clone(),
                file: file_id,
                span: decl.span,
                kind,
            });
            decl_symbols.push(symbol_id);
            local_type_names.entry(decl.name.clone()).or_insert(symbol_id);
        }

        let mut exports: FxHashMap<Box<str>, ExportEntry> = FxHashMap::default();
        for export in &file.surface.exports {
            match export {
                SurfaceExport::Decl { decl, name } => {
                    let symbol_id = decl_symbols[*decl as usize];
                    add_export(&mut exports, name, symbol_id, &symbols);
                }
                SurfaceExport::Named { local, exported } => {
                    // `export { local as exported }` where `local` is a
                    // declaration. Import-binding locals are handled in the
                    // re-export fixpoint below.
                    if let Some(position) = file.surface.decls.iter().position(|d| &d.name == local)
                    {
                        add_export(&mut exports, exported, decl_symbols[position], &symbols);
                    }
                }
            }
        }

        let resolutions = file
            .resolutions
            .iter()
            .map(|(specifier, resolution)| {
                let resolution = match resolution {
                    RawResolution::File(path) => path_to_file
                        .get(path)
                        .map_or(ModuleResolution::External, |id| ModuleResolution::File(*id)),
                    RawResolution::External => ModuleResolution::External,
                    RawResolution::NotFound => ModuleResolution::NotFound,
                };
                (specifier.clone(), resolution)
            })
            .collect();

        file_symbols.push(decl_symbols);
        files.push(FileData {
            path: file.path.clone(),
            source_text: String::new(), // moved in at the end
            kind: file.kind,
            parse_failed: file.parse_failed,
            exports,
            opaque_exports: file.surface.opaque_exports,
            local_type_names,
            resolutions,
            diagnostics: Vec::new(), // moved in at the end
        });
    }

    // Step 2: re-export fixpoint. Copying is monotone and first-wins, so
    // iterating to quiescence is cycle-safe.
    loop {
        let mut changed = false;
        for index in 0..loaded.len() {
            let mut additions: Vec<(Box<str>, ExportEntry)> = Vec::new();
            let mut make_opaque = false;
            // `export { local as exported }` where `local` is an import
            // binding — semantically a named re-export.
            for export in &loaded[index].surface.exports {
                let SurfaceExport::Named { local, exported } = export else { continue };
                if files[index].exports.contains_key(exported) {
                    continue;
                }
                let Some((surface_import, binding)) =
                    loaded[index].surface.imports.iter().find_map(|import| {
                        import.bindings.iter().find(|b| &b.local == local).map(|b| (import, b))
                    })
                else {
                    continue;
                };
                let entry = match &binding.kind {
                    ImportBindingKind::Named(imported) => {
                        resolve_specifier(&files[index], &surface_import.specifier).and_then(
                            |target| files[target.index()].exports.get(&**imported).copied(),
                        )
                    }
                    ImportBindingKind::Default => {
                        resolve_specifier(&files[index], &surface_import.specifier).and_then(
                            |target| files[target.index()].exports.get("default").copied(),
                        )
                    }
                    ImportBindingKind::Namespace => None,
                };
                if let Some(entry) = entry {
                    additions.push((exported.clone(), entry));
                } else {
                    // Unresolvable through the import: export the name opaquely.
                    additions.push((exported.clone(), ExportEntry::default()));
                }
            }
            for reexport in &loaded[index].surface.reexports {
                match reexport {
                    SurfaceReexport::Named { specifier, imported, exported } => {
                        match resolve_specifier(&files[index], specifier) {
                            Some(target) => {
                                let target_file = &files[target.index()];
                                if let Some(entry) = target_file.exports.get(imported) {
                                    if !files[index].exports.contains_key(exported) {
                                        additions.push((exported.clone(), *entry));
                                    }
                                } else if target_file.opaque_exports
                                    && !files[index].exports.contains_key(exported)
                                {
                                    additions.push((exported.clone(), ExportEntry::default()));
                                }
                            }
                            // Re-export from an unresolved/external module:
                            // the name exists with unknown meaning.
                            None => {
                                if !files[index].exports.contains_key(exported) {
                                    additions.push((exported.clone(), ExportEntry::default()));
                                }
                            }
                        }
                    }
                    SurfaceReexport::Star { specifier } => {
                        if let Some(target) = resolve_specifier(&files[index], specifier)
                            && target.index() != index
                        {
                            if files[target.index()].opaque_exports {
                                make_opaque = true;
                            }
                            for (name, entry) in &files[target.index()].exports {
                                // `export *` skips `default` per spec.
                                if &**name != "default" && !files[index].exports.contains_key(name)
                                {
                                    additions.push((name.clone(), *entry));
                                }
                            }
                        }
                    }
                    SurfaceReexport::StarAs { exported } => {
                        if !files[index].exports.contains_key(exported) {
                            let symbol_id = SymbolId(u32::try_from(symbols.len()).unwrap());
                            symbols.push(SymbolData {
                                name: exported.clone(),
                                file: FileId(u32::try_from(index).unwrap()),
                                span: oxc_span::SPAN,
                                kind: SymbolKind::Namespace,
                            });
                            additions.push((
                                exported.clone(),
                                ExportEntry { value: Some(symbol_id), ty: None },
                            ));
                        }
                    }
                }
            }
            if make_opaque && !files[index].opaque_exports {
                files[index].opaque_exports = true;
                changed = true;
            }
            if !additions.is_empty() {
                changed = true;
                files[index].exports.extend(additions);
            }
        }
        if !changed {
            break;
        }
    }

    // Step 3: rewrite pending refs now that every export table is complete.
    for (index, file) in loaded.iter().enumerate() {
        let base = file_type_bases[index];
        let len = u32::try_from(file.surface.types.len()).unwrap();
        for offset in 0..len {
            let id = TypeId(base + offset);
            let Type::Ref { target, .. } = types.get(id) else { continue };
            let new_target = match target {
                RefTarget::PendingLocal(decl) => {
                    RefTarget::Symbol(file_symbols[index][*decl as usize])
                }
                RefTarget::PendingImport { import, member } => {
                    resolve_import_ref(&files, index, file, *import, member.as_deref())
                }
                RefTarget::Symbol(_) | RefTarget::Local(_) | RefTarget::Unresolved => continue,
            };
            let Type::Ref { target, .. } = types.get_mut(id) else { unreachable!() };
            *target = new_target;
        }
    }

    // Step 3.5: flatten interface inheritance (single-level identifier bases,
    // non-generic) so inherited members participate in relations.
    let iface_list: Vec<SymbolId> = iface_extends.keys().copied().collect();
    for symbol_id in iface_list {
        flatten_interface(symbol_id, &iface_extends, &symbols, &mut types, 0);
    }

    // Move bulky per-file data in.
    for (data, file) in files.iter_mut().zip(loaded) {
        data.source_text = file.source_text;
        data.diagnostics = file.diagnostics;
    }

    ProgramEnv { files, symbols, types, strict_null_checks }
}

fn add_export(
    exports: &mut FxHashMap<Box<str>, ExportEntry>,
    name: &str,
    symbol_id: SymbolId,
    symbols: &[SymbolData],
) {
    let entry = exports.entry(name.into()).or_default();
    let kind = &symbols[symbol_id.index()].kind;
    if kind.is_value() && entry.value.is_none() {
        entry.value = Some(symbol_id);
    }
    if kind.is_type() && entry.ty.is_none() {
        entry.ty = Some(symbol_id);
    }
}

fn resolve_specifier(file: &FileData, specifier: &str) -> Option<FileId> {
    match file.resolutions.get(specifier) {
        Some(ModuleResolution::File(id)) => Some(*id),
        _ => None,
    }
}

/// Resolve a pending type-position import ref against the linked export
/// tables. `member: None` means the default export.
fn resolve_import_ref(
    files: &[FileData],
    index: usize,
    loaded: &LoadedFile,
    import: u32,
    member: Option<&str>,
) -> RefTarget {
    let Some(surface_import) = loaded.surface.imports.get(import as usize) else {
        return RefTarget::Unresolved;
    };
    let Some(target) = resolve_specifier(&files[index], &surface_import.specifier) else {
        return RefTarget::Unresolved;
    };
    let target_file = &files[target.index()];
    let name = member.unwrap_or("default");
    match target_file.exports.get(name) {
        // Type position: prefer the type meaning, fall back to the value
        // meaning (its declared type is still useful for `typeof`-free v0).
        Some(entry) => entry.ty.or(entry.value).map_or(RefTarget::Unresolved, RefTarget::Symbol),
        None => RefTarget::Unresolved,
    }
}

/// Merge inherited members into an interface's own shape, in place. Returns
/// whether the flattened shape is exact (all bases resolved and exact).
fn flatten_interface(
    symbol_id: SymbolId,
    iface_extends: &FxHashMap<SymbolId, Vec<TypeId>>,
    symbols: &[SymbolData],
    types: &mut TypeTable,
    depth: u32,
) -> bool {
    let SymbolKind::Interface { ty, ref params } = symbols[symbol_id.index()].kind else {
        return false;
    };
    if !params.is_empty() || depth > 8 {
        return false;
    }
    let Type::Object(own) = types.get(ty) else { return false };
    let mut members: Vec<crate::ir::Member> = own.members.to_vec();
    let mut exact = !own.inexact;
    let Some(extends) = iface_extends.get(&symbol_id) else {
        return exact;
    };
    for &base_ref in extends {
        let base_symbol = match types.get(base_ref) {
            Type::Ref { target: RefTarget::Symbol(base), args } if args.is_empty() => *base,
            _ => {
                exact = false;
                continue;
            }
        };
        let base_exact = flatten_interface(base_symbol, iface_extends, symbols, types, depth + 1);
        let SymbolKind::Interface { ty: base_ty, .. } = symbols[base_symbol.index()].kind else {
            exact = false;
            continue;
        };
        let Type::Object(base_shape) = types.get(base_ty) else {
            exact = false;
            continue;
        };
        // The copy ends the `types` borrow before `members` grows.
        #[expect(clippy::unnecessary_to_owned)]
        for base_member in base_shape.members.to_vec() {
            if !members.iter().any(|m| m.name == base_member.name) {
                members.push(base_member);
            }
        }
        exact &= base_exact;
    }
    *types.get_mut(ty) =
        Type::Object(ObjectShape { members: members.into_boxed_slice(), inexact: !exact });
    exact
}

fn relocate_params(
    params: &[crate::surface::SurfaceTypeParam],
    relocate: impl Fn(TypeId) -> TypeId + Copy,
) -> Box<[crate::ir::TypeParamInfo]> {
    params
        .iter()
        .map(|p| crate::ir::TypeParamInfo {
            name: p.name.clone(),
            constraint: p.constraint.map(relocate),
            default: p.default.map(relocate),
        })
        .collect()
}

fn relocate_id(id: TypeId, base: u32) -> TypeId {
    if id.0 < INTRINSIC_COUNT { id } else { TypeId(id.0 - INTRINSIC_COUNT + base) }
}

/// Deep-copy a surface type, remapping embedded type ids into the global table.
fn relocate_type(ty: &Type, relocate: impl Fn(TypeId) -> TypeId + Copy) -> Type {
    match ty {
        Type::Union(members) => Type::Union(members.iter().map(|t| relocate(*t)).collect()),
        Type::Intersection(members) => {
            Type::Intersection(members.iter().map(|t| relocate(*t)).collect())
        }
        Type::Array(elem) => Type::Array(relocate(*elem)),
        Type::Tuple(members) => Type::Tuple(members.iter().map(|t| relocate(*t)).collect()),
        Type::Object(shape) => Type::Object(ObjectShape {
            members: shape
                .members
                .iter()
                .map(|m| crate::ir::Member {
                    name: m.name.clone(),
                    ty: relocate(m.ty),
                    optional: m.optional,
                })
                .collect(),
            inexact: shape.inexact,
        }),
        Type::Function(shape) => Type::Function(Box::new(crate::ir::FunctionShape {
            params: shape
                .params
                .iter()
                .map(|p| crate::ir::Param {
                    name: p.name.clone(),
                    ty: relocate(p.ty),
                    optional: p.optional,
                })
                .collect(),
            ret: relocate(shape.ret),
        })),
        Type::Ref { target, args } => {
            Type::Ref { target: target.clone(), args: args.iter().map(|t| relocate(*t)).collect() }
        }
        // Leaf types carry no ids.
        _ => ty.clone(),
    }
}
