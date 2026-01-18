use std::mem;

use oxc_index::IndexVec;
use quote::ToTokens;
use rustc_hash::FxHashMap;
use syn::{
    AttrStyle, Attribute, Expr, ExprLit, Field, Fields, GenericArgument, Generics, Ident, ItemEnum,
    ItemStruct, Lit, Meta, MetaList, PathArguments, PathSegment, Type, TypePath, TypeReference,
    Variant, Visibility as SynVisibility, punctuated::Punctuated, token::Comma,
};

use crate::{
    DERIVES, GENERATORS, Result,
    codegen::Codegen,
    schema::{
        BoxDef, CellDef, Def, EnumDef, FieldDef, File, FileId, MetaId, MetaType, OptionDef,
        PointerDef, PointerKind, PrimitiveDef, Schema, StructDef, TypeDef, TypeId, VariantDef,
        VecDef, Visibility,
    },
    utils::{FxIndexMap, FxIndexSet, ident_name},
};

use super::{
    Derives,
    attr::{AttrLocation, AttrPart, AttrPartListElement, AttrPositions, AttrProcessor},
    skeleton::{EnumSkeleton, Skeleton, StructSkeleton},
};

/// Parse [`Skeleton`]s into [`TypeDef`]s.
pub fn parse(
    skeletons: FxIndexMap<String, Skeleton>,
    meta_skeletons: FxIndexMap<String, Skeleton>,
    files: IndexVec<FileId, File>,
    codegen: &Codegen,
) -> Schema {
    // Split `skeletons` into an `IndexSet<String>` (type names) and `IndexVec<TypeId, Skeleton>` (skeletons)
    let (type_names, skeletons_vec) = skeletons.into_iter().unzip();
    // Split `meta_skeletons` into an `IndexSet<String>` (meta names) and `IndexVec<MetaId, Skeleton>` (skeletons)
    let (meta_names, meta_skeletons_vec) = meta_skeletons.into_iter().unzip();

    let parser = Parser::new(type_names, meta_names, files, codegen);
    parser.parse_all(skeletons_vec, meta_skeletons_vec)
}

/// Types parser.
struct Parser<'c> {
    /// Index hash set indexed by type ID, containing type names
    type_names: FxIndexSet<String>,
    /// Index hash set indexed by meta ID, containing meta names
    meta_names: FxIndexSet<String>,
    /// Source files
    files: IndexVec<FileId, File>,
    /// Reference to `CodeGen`
    codegen: &'c Codegen,
    /// Extra types which don't have type definitions in the source files
    /// e.g. primitives (`u8` etc), `Option`s, `Box`es, `Vec`s, `Cell`s
    extra_types: Vec<TypeDef>,
    // These `FxHashMap`s:
    // * Key: Inner type's `TypeId`.
    // * Value: Outer type's (`Option`/`Box`/`Vec`/`Cell`) `TypeId`.
    // i.e. if `Expression` has ID 1, and `Option<Expression>` has ID 2, then key is 1 and value is 2
    // `options` hash map.
    options: FxHashMap<TypeId, TypeId>,
    boxes: FxHashMap<TypeId, TypeId>,
    vecs: FxHashMap<TypeId, TypeId>,
    cells: FxHashMap<TypeId, TypeId>,
    non_nulls: FxHashMap<TypeId, TypeId>,
}

impl<'c> Parser<'c> {
    /// Create [`Parser`].
    fn new(
        type_names: FxIndexSet<String>,
        meta_names: FxIndexSet<String>,
        files: IndexVec<FileId, File>,
        codegen: &'c Codegen,
    ) -> Self {
        Self {
            type_names,
            meta_names,
            files,
            codegen,
            extra_types: vec![],
            options: FxHashMap::default(),
            boxes: FxHashMap::default(),
            vecs: FxHashMap::default(),
            cells: FxHashMap::default(),
            non_nulls: FxHashMap::default(),
        }
    }

    /// Parse all [`Skeleton`]s into [`TypeDef`]s and return [`Schema`].
    fn parse_all(
        mut self,
        skeletons: IndexVec<TypeId, Skeleton>,
        meta_skeletons: IndexVec<MetaId, Skeleton>,
    ) -> Schema {
        // Parse `#[ast_meta]` types from `Skeleton` to `MetaType`
        let metas = meta_skeletons
            .into_iter_enumerated()
            .map(|(meta_id, skeleton)| self.parse_meta_type(meta_id, skeleton))
            .collect::<IndexVec<_, _>>();

        // Parse `#[ast]` types from `Skeleton` to `TypeDef`
        let mut types = skeletons
            .into_iter_enumerated()
            .map(|(type_id, skeleton)| self.parse_type(type_id, skeleton))
            .collect::<IndexVec<_, _>>();
        types.extend(mem::take(&mut self.extra_types));

        // Set container IDs on type defs
        self.set_container_ids(&mut types);

        // Generate `HashMap` mapping name of `#[ast_meta]` type to its `MetaId`
        let meta_names = self
            .meta_names
            .into_iter()
            .enumerate()
            .map(|(meta_id, meta_name)| (meta_name, MetaId::from_usize(meta_id)))
            .collect();

        // Generate `HashMap` mapping name of `#[ast]` type to its `TypeId`
        let type_names = self
            .type_names
            .into_iter()
            .enumerate()
            .map(|(type_id, type_name)| (type_name, TypeId::from_usize(type_id)))
            .collect();

        Schema { types, type_names, metas, meta_names, files: self.files }
    }

    /// Set container IDs on type defs.
    ///
    /// i.e. if `Option<Expression>` exists in AST, set `option_id` on type def for `Expression`
    /// to the `TypeId` of `Option<Expression>`.
    ///
    /// Same for `Box`, `Vec` and `Cell`.
    fn set_container_ids(&self, types: &mut IndexVec<TypeId, TypeDef>) {
        for (&inner_type_id, &option_id) in &self.options {
            match &mut types[inner_type_id] {
                TypeDef::Struct(def) => def.containers.option_id = Some(option_id),
                TypeDef::Enum(def) => def.containers.option_id = Some(option_id),
                TypeDef::Primitive(def) => def.containers.option_id = Some(option_id),
                TypeDef::Option(def) => def.containers.option_id = Some(option_id),
                TypeDef::Box(def) => def.containers.option_id = Some(option_id),
                TypeDef::Vec(def) => def.containers.option_id = Some(option_id),
                TypeDef::Cell(def) => def.containers.option_id = Some(option_id),
                TypeDef::Pointer(def) => def.containers.option_id = Some(option_id),
            }
        }

        for (&inner_type_id, &box_id) in &self.boxes {
            match &mut types[inner_type_id] {
                TypeDef::Struct(def) => def.containers.box_id = Some(box_id),
                TypeDef::Enum(def) => def.containers.box_id = Some(box_id),
                TypeDef::Primitive(def) => def.containers.box_id = Some(box_id),
                TypeDef::Option(def) => def.containers.box_id = Some(box_id),
                TypeDef::Box(def) => def.containers.box_id = Some(box_id),
                TypeDef::Vec(def) => def.containers.box_id = Some(box_id),
                TypeDef::Cell(def) => def.containers.box_id = Some(box_id),
                TypeDef::Pointer(def) => def.containers.box_id = Some(box_id),
            }
        }

        for (&inner_type_id, &vec_id) in &self.vecs {
            match &mut types[inner_type_id] {
                TypeDef::Struct(def) => def.containers.vec_id = Some(vec_id),
                TypeDef::Enum(def) => def.containers.vec_id = Some(vec_id),
                TypeDef::Primitive(def) => def.containers.vec_id = Some(vec_id),
                TypeDef::Option(def) => def.containers.vec_id = Some(vec_id),
                TypeDef::Box(def) => def.containers.vec_id = Some(vec_id),
                TypeDef::Vec(def) => def.containers.vec_id = Some(vec_id),
                TypeDef::Cell(def) => def.containers.vec_id = Some(vec_id),
                TypeDef::Pointer(def) => def.containers.vec_id = Some(vec_id),
            }
        }

        for (&inner_type_id, &cell_id) in &self.cells {
            match &mut types[inner_type_id] {
                TypeDef::Struct(def) => def.containers.cell_id = Some(cell_id),
                TypeDef::Enum(def) => def.containers.cell_id = Some(cell_id),
                TypeDef::Primitive(def) => def.containers.cell_id = Some(cell_id),
                TypeDef::Option(def) => def.containers.cell_id = Some(cell_id),
                TypeDef::Box(def) => def.containers.cell_id = Some(cell_id),
                TypeDef::Vec(def) => def.containers.cell_id = Some(cell_id),
                TypeDef::Cell(def) => def.containers.cell_id = Some(cell_id),
                TypeDef::Pointer(def) => def.containers.cell_id = Some(cell_id),
            }
        }

        for (&inner_type_id, &non_null_id) in &self.non_nulls {
            match &mut types[inner_type_id] {
                TypeDef::Struct(def) => def.containers.non_null_id = Some(non_null_id),
                TypeDef::Enum(def) => def.containers.non_null_id = Some(non_null_id),
                TypeDef::Primitive(def) => def.containers.non_null_id = Some(non_null_id),
                TypeDef::Option(def) => def.containers.non_null_id = Some(non_null_id),
                TypeDef::Box(def) => def.containers.non_null_id = Some(non_null_id),
                TypeDef::Vec(def) => def.containers.non_null_id = Some(non_null_id),
                TypeDef::Cell(def) => def.containers.non_null_id = Some(non_null_id),
                TypeDef::Pointer(def) => def.containers.non_null_id = Some(non_null_id),
            }
        }
    }

    /// Get [`TypeId`] for type name.
    fn type_id(&mut self, name: &str) -> TypeId {
        // Get type ID if already known
        if let Some(type_id) = self.type_names.get_index_of(name) {
            return TypeId::from_usize(type_id);
        }

        // Generate new type for known primitives/special cases
        let primitive = |name| TypeDef::Primitive(PrimitiveDef::new(name));

        let type_def = match name {
            "bool" => primitive("bool"),
            "u8" => primitive("u8"),
            "u16" => primitive("u16"),
            "u32" => primitive("u32"),
            "u64" => primitive("u64"),
            "u128" => primitive("u128"),
            "usize" => primitive("usize"),
            "i8" => primitive("i8"),
            "i16" => primitive("i16"),
            "i32" => primitive("i32"),
            "i64" => primitive("i64"),
            "i128" => primitive("i128"),
            "isize" => primitive("isize"),
            "f32" => primitive("f32"),
            "f64" => primitive("f64"),
            "NonZeroU8" => primitive("NonZeroU8"),
            "NonZeroU16" => primitive("NonZeroU16"),
            "NonZeroU32" => primitive("NonZeroU32"),
            "NonZeroU64" => primitive("NonZeroU64"),
            "NonZeroU128" => primitive("NonZeroU128"),
            "NonZeroUsize" => primitive("NonZeroUsize"),
            "NonZeroI8" => primitive("NonZeroI8"),
            "NonZeroI16" => primitive("NonZeroI16"),
            "NonZeroI32" => primitive("NonZeroI32"),
            "NonZeroI64" => primitive("NonZeroI64"),
            "NonZeroI128" => primitive("NonZeroI128"),
            "NonZeroIsize" => primitive("NonZeroIsize"),
            "AtomicBool" => primitive("AtomicBool"),
            "AtomicU8" => primitive("AtomicU8"),
            "AtomicU16" => primitive("AtomicU16"),
            "AtomicU32" => primitive("AtomicU32"),
            "AtomicU64" => primitive("AtomicU64"),
            "AtomicU128" => primitive("AtomicU128"),
            "AtomicUsize" => primitive("AtomicUsize"),
            "AtomicI8" => primitive("AtomicI8"),
            "AtomicI16" => primitive("AtomicI16"),
            "AtomicI32" => primitive("AtomicI32"),
            "AtomicI64" => primitive("AtomicI64"),
            "AtomicI128" => primitive("AtomicI128"),
            "AtomicIsize" => primitive("AtomicIsize"),
            "AtomicPtr" => primitive("AtomicPtr"),
            "&str" => primitive("&str"),
            "Atom" => primitive("Atom"),
            "NonMaxU32" => primitive("NonMaxU32"),
            "NodeId" => primitive("NodeId"),
            // TODO: Remove the need for this by adding
            // `#[cfg_attr(target_pointer_width = "64", repr(align(8)))]` to all AST types
            "PointerAlign" => primitive("PointerAlign"),
            _ => panic!("Unknown type: {name}"),
        };
        self.create_new_type(type_def)
    }

    /// Get type name for a [`TypeId`].
    fn type_name(&self, type_id: TypeId) -> &str {
        &self.type_names[type_id.index()]
    }

    /// Create a new type definition.
    fn create_new_type(&mut self, mut type_def: TypeDef) -> TypeId {
        let type_id = TypeId::from_usize(self.type_names.len());

        match &mut type_def {
            TypeDef::Struct(def) => def.id = type_id,
            TypeDef::Enum(def) => def.id = type_id,
            TypeDef::Primitive(def) => def.id = type_id,
            TypeDef::Option(def) => def.id = type_id,
            TypeDef::Box(def) => def.id = type_id,
            TypeDef::Vec(def) => def.id = type_id,
            TypeDef::Cell(def) => def.id = type_id,
            TypeDef::Pointer(def) => def.id = type_id,
        }

        let was_inserted = self.type_names.insert(type_def.name().to_string());
        assert!(was_inserted, "{}", type_def.name());

        self.extra_types.push(type_def);

        type_id
    }

    /// Parse [`Skeleton`] to yield a [`TypeDef`].
    fn parse_type(&mut self, type_id: TypeId, skeleton: Skeleton) -> TypeDef {
        match skeleton {
            Skeleton::Struct(skeleton) => self.parse_struct(type_id, skeleton),
            Skeleton::Enum(skeleton) => self.parse_enum(type_id, skeleton),
        }
    }

    /// Parse [`StructSkeleton`] to yield a [`TypeDef`].
    fn parse_struct(&mut self, type_id: TypeId, skeleton: StructSkeleton) -> TypeDef {
        let StructSkeleton { name, item, is_foreign, file_id } = skeleton;
        let has_lifetime = check_generics(&item.generics, &name);
        let fields = self.parse_fields(&item.fields);
        let visibility = convert_visibility(&item.vis);
        let (generated_derives, plural_name) =
            self.get_generated_derives_and_plural_name(&item.attrs, &name);
        let derives = Self::get_derives(&item.attrs);
        let mut type_def = TypeDef::Struct(StructDef::new(
            type_id,
            name,
            plural_name,
            has_lifetime,
            is_foreign,
            file_id,
            visibility,
            derives,
            generated_derives,
            fields,
        ));

        // Parse attrs on type and fields
        self.parse_type_attrs(&mut type_def, &item.attrs);
        self.parse_field_attrs(&mut type_def, &item, generated_derives);

        type_def
    }

    /// Parse attributes on struct's fields with parsers provided by [`Derive`]s and [`Generator`]s.
    ///
    /// [`Derive`]: crate::Derive
    /// [`Generator`]: crate::Generator
    fn parse_field_attrs(
        &self,
        type_def: &mut TypeDef,
        item: &ItemStruct,
        generated_derives: Derives,
    ) {
        let struct_def = type_def.as_struct_mut().unwrap();
        for (field_index, field) in item.fields.iter().enumerate() {
            for attr in &field.attrs {
                if !matches!(attr.style, AttrStyle::Outer) {
                    continue;
                }
                let Some(attr_ident) = attr.path().get_ident() else { continue };
                let attr_name = ident_name(attr_ident);

                if let Some((processor, positions)) = self.codegen.attr_processor(&attr_name) {
                    // Check attribute is legal in this position
                    // and has the relevant trait `#[generate_derive]`-ed on it
                    check_attr_position(
                        positions,
                        AttrPositions::StructField,
                        struct_def.name(),
                        &attr_name,
                        "struct field",
                    );
                    check_attr_is_derived(
                        processor,
                        generated_derives,
                        struct_def.name(),
                        &attr_name,
                    );

                    let location = AttrLocation::StructField(struct_def, field_index);
                    let result = process_attr(processor, &attr_name, location, &attr.meta);
                    assert!(
                        result.is_ok(),
                        "Invalid use of `#[{attr_name}]` on `{}::{}` struct field",
                        struct_def.name(),
                        struct_def.fields[field_index].name()
                    );
                }
            }
        }
    }

    /// Parse [`EnumSkeleton`] to yield a [`TypeDef`].
    fn parse_enum(&mut self, type_id: TypeId, skeleton: EnumSkeleton) -> TypeDef {
        let EnumSkeleton { name, item, inherits, is_foreign, file_id } = skeleton;
        let has_lifetime = check_generics(&item.generics, &name);
        let variants = item.variants.iter().map(|variant| self.parse_variant(variant)).collect();
        let inherits = inherits.into_iter().map(|name| self.type_id(&name)).collect();
        let visibility = convert_visibility(&item.vis);
        let (generated_derives, plural_name) =
            self.get_generated_derives_and_plural_name(&item.attrs, &name);
        let derives = Self::get_derives(&item.attrs);
        let mut type_def = TypeDef::Enum(EnumDef::new(
            type_id,
            name,
            plural_name,
            has_lifetime,
            is_foreign,
            file_id,
            visibility,
            derives,
            generated_derives,
            variants,
            inherits,
        ));

        // Parse attrs on type and variants
        self.parse_type_attrs(&mut type_def, &item.attrs);
        self.parse_variant_attrs(&mut type_def, &item, generated_derives);

        type_def
    }

    /// Parse attributes on enum's variants with parsers provided by [`Derive`]s and [`Generator`]s.
    ///
    /// [`Derive`]: crate::Derive
    /// [`Generator`]: crate::Generator
    fn parse_variant_attrs(
        &self,
        type_def: &mut TypeDef,
        item: &ItemEnum,
        generated_derives: Derives,
    ) {
        let enum_def = type_def.as_enum_mut().unwrap();
        for (variant_index, variant) in item.variants.iter().enumerate() {
            for attr in &variant.attrs {
                if !matches!(attr.style, AttrStyle::Outer) {
                    continue;
                }
                let Some(attr_ident) = attr.path().get_ident() else { continue };
                let attr_name = ident_name(attr_ident);

                if let Some((processor, positions)) = self.codegen.attr_processor(&attr_name) {
                    // Check attribute is legal in this position
                    // and has the relevant trait `#[generate_derive]`-ed on it
                    check_attr_position(
                        positions,
                        AttrPositions::EnumVariant,
                        enum_def.name(),
                        &attr_name,
                        "enum variant",
                    );
                    check_attr_is_derived(
                        processor,
                        generated_derives,
                        enum_def.name(),
                        &attr_name,
                    );

                    let location = AttrLocation::EnumVariant(enum_def, variant_index);
                    let result = process_attr(processor, &attr_name, location, &attr.meta);
                    assert!(
                        result.is_ok(),
                        "Invalid use of `#[{attr_name}]` on `{}::{}` enum variant",
                        enum_def.name(),
                        enum_def.variants[variant_index].name(),
                    );
                }
            }
        }
    }

    /// Parse struct fields to [`FieldDef`]s.
    ///
    /// [`Vec<FieldDef>`]: FieldDef
    fn parse_fields(&mut self, fields: &Fields) -> Vec<FieldDef> {
        fields.iter().enumerate().map(|(index, field)| self.parse_field(field, index)).collect()
    }

    /// Parse struct field to [`FieldDef`].
    fn parse_field(&mut self, field: &Field, index: usize) -> FieldDef {
        let name = match &field.ident {
            Some(ident) => ident_name(ident),
            None => index.to_string(),
        };

        let ty = &field.ty;
        let type_id = self
            .parse_type_name(ty)
            .unwrap_or_else(|| panic!("Cannot parse type reference: {}", ty.to_token_stream()));
        let visibility = convert_visibility(&field.vis);

        // Get doc comment
        let mut doc_comment = None;
        for attr in &field.attrs {
            if let Meta::NameValue(name_value) = &attr.meta
                && name_value.path.is_ident("doc")
                && let Expr::Lit(expr_lit) = &name_value.value
                && let Lit::Str(lit_str) = &expr_lit.lit
            {
                doc_comment = Some(lit_str.value().trim().to_string());
                break;
            }
        }

        FieldDef::new(name, type_id, visibility, doc_comment)
    }

    /// Parse enum variant to [`VariantDef`].
    fn parse_variant(&mut self, variant: &Variant) -> VariantDef {
        let name = ident_name(&variant.ident);

        let field_type_id = if variant.fields.is_empty() {
            None
        } else {
            assert!(
                variant.fields.len() == 1,
                "Only enum variants with a single field are supported: {name}"
            );
            let field = variant.fields.iter().next().unwrap();
            let type_id = self.parse_type_name(&field.ty).unwrap_or_else(|| {
                panic!("Cannot parse type reference: {}", field.ty.to_token_stream())
            });
            Some(type_id)
        };

        let discriminant = {
            let Some((_, discriminant)) = variant.discriminant.as_ref() else {
                panic!("All enum variants must have explicit discriminants: {name}");
            };
            let Expr::Lit(ExprLit { lit: Lit::Int(lit), .. }) = discriminant else {
                panic!("Invalid enum discriminant {discriminant:?} on {name}");
            };
            let Ok(discriminant) = lit.base10_parse() else {
                panic!("Invalid base10 enum discriminant {discriminant:?} on {name}");
            };
            discriminant
        };

        VariantDef::new(name, field_type_id, discriminant)
    }

    /// Resolve type name to its [`TypeId`].
    fn parse_type_name(&mut self, ty: &Type) -> Option<TypeId> {
        match ty {
            Type::Path(type_path) => self.parse_type_path(type_path),
            Type::Reference(type_ref) => self.parse_type_reference(type_ref),
            _ => None,
        }
    }

    fn parse_type_path(&mut self, type_path: &TypePath) -> Option<TypeId> {
        let segment = type_path_segment(type_path)?;
        let name = ident_name(&segment.ident);
        match &segment.arguments {
            PathArguments::None => Some(self.type_id(&name)),
            PathArguments::Parenthesized(_) => None,
            PathArguments::AngleBracketed(angled) => {
                // Get first arg, skipping over lifetime arg
                let mut args = angled.args.iter();
                let arg = match args.next().unwrap() {
                    GenericArgument::Lifetime(_) => args.next(),
                    arg => Some(arg),
                };

                if let Some(arg) = arg {
                    self.parse_complex_type_path(&name, arg)
                } else {
                    Some(self.type_id(&name))
                }
            }
        }
    }

    fn parse_complex_type_path(
        &mut self,
        wrapper_name: &str,
        arg: &GenericArgument,
    ) -> Option<TypeId> {
        let GenericArgument::Type(ty) = arg else { return None };

        let inner_type_id = self.parse_type_name(ty)?;

        let type_id = match wrapper_name {
            "Option" => self.options.get(&inner_type_id).copied().unwrap_or_else(|| {
                let name = format!("Option<{}>", self.type_name(inner_type_id));
                let type_def = TypeDef::Option(OptionDef::new(name, inner_type_id));
                let type_id = self.create_new_type(type_def);
                self.options.insert(inner_type_id, type_id);
                type_id
            }),
            "Box" => self.boxes.get(&inner_type_id).copied().unwrap_or_else(|| {
                let name = format!("Box<{}>", self.type_name(inner_type_id));
                let type_def = TypeDef::Box(BoxDef::new(name, inner_type_id));
                let type_id = self.create_new_type(type_def);
                self.boxes.insert(inner_type_id, type_id);
                type_id
            }),
            "Vec" => self.vecs.get(&inner_type_id).copied().unwrap_or_else(|| {
                let name = format!("Vec<{}>", self.type_name(inner_type_id));
                let type_def = TypeDef::Vec(VecDef::new(name, inner_type_id));
                let type_id = self.create_new_type(type_def);
                self.vecs.insert(inner_type_id, type_id);
                type_id
            }),
            "Cell" => self.cells.get(&inner_type_id).copied().unwrap_or_else(|| {
                let name = format!("Cell<{}>", self.type_name(inner_type_id));
                let type_def = TypeDef::Cell(CellDef::new(name, inner_type_id));
                let type_id = self.create_new_type(type_def);
                self.cells.insert(inner_type_id, type_id);
                type_id
            }),
            "NonNull" => self.non_nulls.get(&inner_type_id).copied().unwrap_or_else(|| {
                let name = format!("NonNull<{}>", self.type_name(inner_type_id));
                let type_def =
                    TypeDef::Pointer(PointerDef::new(name, inner_type_id, PointerKind::NonNull));
                let type_id = self.create_new_type(type_def);
                self.non_nulls.insert(inner_type_id, type_id);
                type_id
            }),
            _ => return None,
        };
        Some(type_id)
    }

    fn parse_type_reference(&mut self, type_ref: &TypeReference) -> Option<TypeId> {
        if type_ref.mutability.is_some() {
            return None;
        }
        let Type::Path(type_path) = &*type_ref.elem else { return None };
        let segment = type_path_segment(type_path)?;
        if segment.ident != "str" || segment.arguments != PathArguments::None {
            return None;
        }
        Some(self.type_id("&str"))
    }

    /// Parse attributes on struct or enum with parsers provided by [`Derive`]s and [`Generator`]s.
    ///
    /// [`Derive`]: crate::Derive
    /// [`Generator`]: crate::Generator
    fn parse_type_attrs(&self, type_def: &mut TypeDef, attrs: &[Attribute]) {
        for attr in attrs {
            if !matches!(attr.style, AttrStyle::Outer) {
                continue;
            }
            let Some(attr_ident) = attr.path().get_ident() else { continue };
            let attr_name = ident_name(attr_ident);

            if attr_name == "ast" {
                self.parse_ast_attr(type_def, attr);
                continue;
            }

            if let Some((processor, positions)) = self.codegen.attr_processor(&attr_name) {
                // Check attribute is legal in this position and this type has the relevant trait
                // `#[generate_derive]`-ed on it (unless the derive stated legal positions as
                // `AttrPositions::StructNotDerived` or `AttrPositions::EnumNotDerived`)
                let location = match type_def {
                    TypeDef::Struct(struct_def) => {
                        let found_in_positions = match processor {
                            AttrProcessor::Derive(derive_id) => {
                                let is_derived = struct_def.generates_derive(derive_id);
                                if is_derived {
                                    AttrPositions::Struct
                                } else {
                                    AttrPositions::StructNotDerived
                                }
                            }
                            AttrProcessor::Generator(_) => AttrPositions::StructMaybeDerived,
                        };

                        check_attr_position(
                            positions,
                            found_in_positions,
                            struct_def.name(),
                            &attr_name,
                            "struct",
                        );

                        AttrLocation::Struct(struct_def)
                    }
                    TypeDef::Enum(enum_def) => {
                        let found_in_positions = match processor {
                            AttrProcessor::Derive(derive_id) => {
                                let is_derived = enum_def.generates_derive(derive_id);
                                if is_derived {
                                    AttrPositions::Enum
                                } else {
                                    AttrPositions::EnumNotDerived
                                }
                            }
                            AttrProcessor::Generator(_) => AttrPositions::EnumMaybeDerived,
                        };

                        check_attr_position(
                            positions,
                            found_in_positions,
                            enum_def.name(),
                            &attr_name,
                            "enum",
                        );

                        AttrLocation::Enum(enum_def)
                    }
                    _ => unreachable!(),
                };

                let result = process_attr(processor, &attr_name, location, &attr.meta);
                assert!(
                    result.is_ok(),
                    "Invalid use of `#[{attr_name}]` on `{}` type",
                    type_def.name()
                );
            }
        }
    }

    /// Parse `#[ast]` attribute parts on struct or enum with parsers provided by [`Derive`]s
    /// and [`Generator`]s.
    ///
    /// e.g. `#[ast(visit)]`
    ///
    /// [`Derive`]: crate::Derive
    /// [`Generator`]: crate::Generator
    fn parse_ast_attr(&self, type_def: &mut TypeDef, attr: &Attribute) {
        let parts = match &attr.meta {
            Meta::Path(_) => return,
            Meta::List(meta_list) => meta_list
                .parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                .map_err(|_| ()),
            Meta::NameValue(_) => Err(()),
        };
        let Ok(parts) = parts else {
            panic!("Unable to parse `#[ast]` attribute on `{}` type", type_def.name());
        };

        for meta in &parts {
            let attr_name = meta.path().get_ident().unwrap().to_string();
            if attr_name == "foreign" {
                continue;
            }

            if let Some((processor, positions)) = self.codegen.attr_processor(&attr_name) {
                // Check attribute is legal in this position
                // and has the relevant trait `#[generate_derive]`-ed on it
                check_attr_position(
                    positions,
                    AttrPositions::AstAttr,
                    type_def.name(),
                    &attr_name,
                    "`#[ast]` attr",
                );
                check_attr_is_derived(
                    processor,
                    type_def.generated_derives(),
                    type_def.name(),
                    &attr_name,
                );

                let location = match type_def {
                    TypeDef::Struct(struct_def) => AttrLocation::StructAstAttr(struct_def),
                    TypeDef::Enum(enum_def) => AttrLocation::EnumAstAttr(enum_def),
                    _ => unreachable!(),
                };
                let result = process_attr(processor, &attr_name, location, meta);
                assert!(
                    result.is_ok(),
                    "Invalid use of `#[ast({attr_name})]` on `{}` type",
                    type_def.name()
                );
            } else {
                panic!("Unknown attribute `#[ast({attr_name})]` on `{}` type", type_def.name());
            }
        }
    }

    /// Get derives which are generated with `#[generate_derive(...)]` attrs.
    fn get_generated_derives_and_plural_name(
        &self,
        attrs: &[Attribute],
        type_name: &str,
    ) -> (Derives, Option<String>) {
        let mut derives = Derives::none();
        let mut plural_name = None;
        for attr in attrs {
            if attr.path().is_ident("generate_derive") {
                let args = attr.parse_args_with(Punctuated::<Ident, Comma>::parse_terminated);
                let Ok(args) = args else {
                    panic!("Unable to parse `#[generated_derives]` on `{type_name}` type");
                };
                for arg in args {
                    let derive_id = self.codegen.get_derive_id_by_name(&ident_name(&arg));
                    derives.add(derive_id);
                }
            } else if attr.path().is_ident("plural") {
                let ident = attr.parse_args::<Ident>();
                let Ok(ident) = ident else {
                    panic!("Unable to parse `#[plural]` on `{type_name}` type");
                };
                assert!(
                    plural_name.is_none(),
                    "Multiple `#[plural]` attributes on `{type_name}` type"
                );
                plural_name = Some(ident.to_string());
            }
        }

        (derives, plural_name)
    }

    fn get_derives(attrs: &[Attribute]) -> Vec<String> {
        let mut derives = Vec::new();
        for attr in attrs {
            if attr.path().is_ident("derive") {
                let args = attr.parse_args_with(Punctuated::<Ident, Comma>::parse_terminated);
                let Ok(args) = args else {
                    panic!("Unable to parse `#[derive]` attribute");
                };
                for arg in args {
                    derives.push(arg.to_string());
                }
            }
        }
        derives
    }

    /// Parse [`Skeleton`] to yield a [`MetaType`].
    fn parse_meta_type(&self, meta_id: MetaId, skeleton: Skeleton) -> MetaType {
        let (type_name, file_id, attrs) = match skeleton {
            Skeleton::Struct(skeleton) => (skeleton.name, skeleton.file_id, skeleton.item.attrs),
            Skeleton::Enum(skeleton) => (skeleton.name, skeleton.file_id, skeleton.item.attrs),
        };

        let mut meta_type = MetaType::new(meta_id, type_name.clone(), file_id);

        // Process attributes
        for attr in &attrs {
            if !matches!(attr.style, AttrStyle::Outer) {
                continue;
            }
            let Some(attr_ident) = attr.path().get_ident() else { continue };
            let attr_name = ident_name(attr_ident);
            if attr_name == "ast_meta" {
                continue;
            }

            let Some((processor, positions)) = self.codegen.attr_processor(&attr_name) else {
                continue;
            };

            // Check attribute is legal in this position
            check_attr_position(
                positions,
                AttrPositions::Meta,
                &type_name,
                &attr_name,
                "meta type",
            );

            let location = AttrLocation::Meta(&mut meta_type);
            let result = process_attr(processor, &attr_name, location, &attr.meta);
            assert!(result.is_ok(), "Invalid use of `#[{attr_name}]` on `{type_name}` meta type");
        }

        meta_type
    }
}

/// Check generics.
///
/// Return `true` if type has a lifetime.
///
/// # Panics
/// Panics if type has type params, const params, or more than one lifetime.
fn check_generics(generics: &Generics, name: &str) -> bool {
    assert!(
        generics.type_params().next().is_none(),
        "Types with generic type params are not supported: {name}"
    );
    assert!(
        generics.const_params().next().is_none(),
        "Types with generic const params are not supported: {name}"
    );

    match generics.lifetimes().count() {
        0 => false,
        1 => true,
        _ => panic!("Types with more than 1 lifetime are not supported: {name}"),
    }
}

/// Get first segment from `TypePath`.
///
/// Returns `None` if has `qself` or leading colon, or if more than 1 segment.
fn type_path_segment(type_path: &TypePath) -> Option<&PathSegment> {
    if type_path.qself.is_some() || type_path.path.leading_colon.is_some() {
        return None;
    }

    let segments = &type_path.path.segments;
    if segments.len() != 1 {
        return None;
    }
    segments.first()
}

/// Process attribute with a processor (derive or generator).
fn process_attr(
    processor: AttrProcessor,
    attr_name: &str,
    mut location: AttrLocation,
    meta: &Meta,
) -> Result<()> {
    match meta {
        Meta::Path(_) => process_attr_part(processor, attr_name, location, AttrPart::None),
        Meta::List(meta_list) => {
            let parts = meta_list
                .parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                .map_err(|_| ())?;
            for meta in parts {
                match &meta {
                    Meta::Path(path) => {
                        let part_name = ident_name(path.get_ident().ok_or(())?);
                        process_attr_part(
                            processor,
                            attr_name,
                            location.unpack(),
                            AttrPart::Tag(&part_name),
                        )?;
                    }
                    Meta::NameValue(name_value) => {
                        let part_name = ident_name(name_value.path.get_ident().ok_or(())?);
                        let str = convert_expr_to_string(&name_value.value);
                        process_attr_part(
                            processor,
                            attr_name,
                            location.unpack(),
                            AttrPart::String(&part_name, str),
                        )?;
                    }
                    Meta::List(meta_list) => {
                        let part_name = ident_name(meta_list.path.get_ident().ok_or(())?);
                        let list = parse_attr_part_list(meta_list)?;
                        process_attr_part(
                            processor,
                            attr_name,
                            location.unpack(),
                            AttrPart::List(&part_name, list),
                        )?;
                    }
                }
            }
            Ok(())
        }
        Meta::NameValue(_) => Err(()),
    }
}

fn parse_attr_part_list(meta_list: &MetaList) -> Result<Vec<AttrPartListElement>> {
    let metas =
        meta_list.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated).map_err(|_| ())?;
    let list_elements = metas
        .into_iter()
        .map(|meta| match meta {
            Meta::Path(path) => {
                let part_name = ident_name(path.get_ident().ok_or(())?);
                Ok(AttrPartListElement::Tag(part_name))
            }
            Meta::NameValue(name_value) => {
                let part_name = ident_name(name_value.path.get_ident().ok_or(())?);
                let str = convert_expr_to_string(&name_value.value);
                Ok(AttrPartListElement::String(part_name, str))
            }
            Meta::List(meta_list) => {
                let part_name = ident_name(meta_list.path.get_ident().ok_or(())?);
                let list = parse_attr_part_list(&meta_list)?;
                Ok(AttrPartListElement::List(part_name, list))
            }
        })
        .collect::<Result<Vec<_>>>()?;

    if list_elements.is_empty() {
        return Err(());
    }

    Ok(list_elements)
}

/// Convert an [`Expr`] to a string.
///
/// If the `Expr` is a string literal, get the value of the string.
/// Otherwise print the `Expr` as a string.
pub fn convert_expr_to_string(expr: &Expr) -> String {
    if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = expr {
        s.value()
    } else {
        expr.to_token_stream().to_string()
    }
}

fn process_attr_part(
    processor: AttrProcessor,
    attr_name: &str,
    location: AttrLocation,
    part: AttrPart,
) -> Result<()> {
    match processor {
        AttrProcessor::Derive(derive_id) => {
            DERIVES[derive_id].parse_attr(attr_name, location, part)
        }
        AttrProcessor::Generator(generator_id) => {
            GENERATORS[generator_id].parse_attr(attr_name, location, part)
        }
    }
}

/// If attribute is processed by a derive, check that trait is derived on the type.
fn check_attr_is_derived(
    processor: AttrProcessor,
    generated_derives: Derives,
    type_name: &str,
    attr_name: &str,
) {
    let AttrProcessor::Derive(derive_id) = processor else { return };
    if generated_derives.has(derive_id) {
        return;
    }

    let trait_name = DERIVES[derive_id].trait_name();
    panic!(
        "`{type_name}` type has `#[{attr_name}]` attribute, but `{trait_name}` trait \
        that handles `#[{attr_name}]` is not derived on `{type_name}`.\n\
        Expected `#[generate_derive({trait_name})]` to be present."
    );
}

/// Check attribute is in a legal position.
fn check_attr_position(
    expected_positions: AttrPositions,
    found_in_positions: AttrPositions,
    type_name: &str,
    attr_name: &str,
    position_debug_str: &str,
) {
    assert!(
        expected_positions.intersects(found_in_positions),
        "`{type_name}` type has `#[{attr_name}]` attribute on a {position_debug_str}, \
        but `#[{attr_name}]` is not legal in this position."
    );
}

/// Convert `syn::Visibility` to our `Visibility` type.
fn convert_visibility(vis: &SynVisibility) -> Visibility {
    match vis {
        SynVisibility::Public(_) => Visibility::Public,
        SynVisibility::Restricted(_) => Visibility::Restricted,
        SynVisibility::Inherited => Visibility::Private,
    }
}
