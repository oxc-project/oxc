use oxc_index::IndexVec;
use quote::ToTokens;
use rustc_hash::FxHashMap;
use syn::{
    punctuated::Punctuated, AttrStyle, Attribute, Expr, ExprLit, Field, Fields, GenericArgument,
    Generics, Ident, ItemEnum, ItemStruct, Lit, Meta, PathArguments, PathSegment, Token, Type,
    TypePath, TypeReference, Variant, Visibility as SynVisibility,
};

use crate::{
    codegen::Codegen,
    schema::{
        BoxDef, CellDef, Def, EnumDef, FieldDef, File, FileId, OptionDef, PrimitiveDef, Schema,
        StructDef, TypeDef, TypeId, VariantDef, VecDef, Visibility,
    },
    Result, DERIVES, GENERATORS,
};

use super::{
    attr::{AttrLocation, AttrPart, AttrPositions, AttrProcessor},
    ident_name,
    skeleton::{EnumSkeleton, Skeleton, StructSkeleton},
    Derives, FxIndexMap, FxIndexSet,
};

/// Parse [`Skeleton`]s into [`TypeDef`]s.
pub fn parse(
    skeletons: FxIndexMap<String, Skeleton>,
    files: IndexVec<FileId, File>,
    codegen: &Codegen,
) -> Schema {
    // Split `skeletons` into a `IndexSet<String>` (type names) and `IndexVec<TypeId, Skeleton>` (skeletons)
    let (type_names, skeletons_vec) = skeletons.into_iter().unzip();

    let parser = Parser::new(type_names, files, codegen);
    parser.parse_all(skeletons_vec)
}

/// Types parser.
struct Parser<'c> {
    /// Index hash set indexed by type ID, containing type names
    type_names: FxIndexSet<String>,
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
}

impl<'c> Parser<'c> {
    /// Create [`Parser`].
    fn new(
        type_names: FxIndexSet<String>,
        files: IndexVec<FileId, File>,
        codegen: &'c Codegen,
    ) -> Self {
        Self {
            type_names,
            files,
            codegen,
            extra_types: vec![],
            options: FxHashMap::default(),
            boxes: FxHashMap::default(),
            vecs: FxHashMap::default(),
            cells: FxHashMap::default(),
        }
    }

    /// Parse all [`Skeleton`]s into [`TypeDef`]s and return [`Schema`].
    fn parse_all(mut self, skeletons: IndexVec<TypeId, Skeleton>) -> Schema {
        let mut types = skeletons
            .into_iter_enumerated()
            .map(|(type_id, skeleton)| self.parse_type(type_id, skeleton))
            .collect::<IndexVec<_, _>>();
        types.extend(self.extra_types);

        let type_names = self
            .type_names
            .into_iter()
            .enumerate()
            .map(|(type_id, type_name)| (type_name, TypeId::from_usize(type_id)))
            .collect();

        Schema { types, type_names, files: self.files }
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
            "&str" => primitive("&str"),
            "Atom" => primitive("Atom"),
            "ScopeId" => primitive("ScopeId"),
            "SymbolId" => primitive("SymbolId"),
            "ReferenceId" => primitive("ReferenceId"),
            // TODO: Remove the need for this by adding
            // `#[cfg_attr(target_pointer_width = "64", repr(align(8)))]` to all AST types
            "PointerAlign" => primitive("PointerAlign"),
            // Cannot be parsed normally as is defined inside `bitflags!` macro.
            // TODO: Find a way to encode this in the actual file.
            // e.g. `#[ast(alias_for(RegExpFlags))] struct RegExpFlagsAlias(u8);`
            "RegExpFlags" => TypeDef::Struct(StructDef::new(
                TypeId::DUMMY,
                "RegExpFlags".to_string(),
                false,
                self.get_file_id("oxc_ast", "::ast::literal"),
                Derives::none(),
                vec![FieldDef::new(
                    "inner".to_string(),
                    self.type_id("u8"),
                    Visibility::Private,
                    None,
                )],
            )),
            _ => panic!("Unknown type: {name}"),
        };
        self.create_new_type(type_def)
    }

    /// Get type name for a [`TypeId`].
    fn type_name(&mut self, type_id: TypeId) -> &str {
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
        }

        let was_inserted = self.type_names.insert(type_def.name().to_string());
        assert!(was_inserted);

        self.extra_types.push(type_def);

        type_id
    }

    /// Get [`FileId`] for file with provided crate and import path.
    fn get_file_id(&self, krate: &str, import_path: &str) -> FileId {
        let file_and_id = self
            .files
            .iter_enumerated()
            .find(|(_, file)| file.krate() == krate && file.import_path() == import_path);
        match file_and_id {
            Some((file_id, _)) => file_id,
            None => panic!("Could not find file with import path: {import_path}"),
        }
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
        let StructSkeleton { name, item, file_id } = skeleton;
        let has_lifetime = check_generics(&item.generics, &name);
        let fields = self.parse_fields(&item.fields);
        let generated_derives = self.get_generated_derives(&item.attrs, &name);
        let mut type_def = TypeDef::Struct(StructDef::new(
            type_id,
            name,
            has_lifetime,
            file_id,
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
        let EnumSkeleton { name, item, inherits, file_id } = skeleton;
        let has_lifetime = check_generics(&item.generics, &name);
        let variants = item.variants.iter().map(|variant| self.parse_variant(variant)).collect();
        let inherits = inherits.into_iter().map(|name| self.type_id(&name)).collect();
        let generated_derives = self.get_generated_derives(&item.attrs, &name);
        let mut type_def = TypeDef::Enum(EnumDef::new(
            type_id,
            name,
            has_lifetime,
            file_id,
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
        let name = match field.ident.as_ref() {
            Some(ident) => ident_name(ident),
            None => index.to_string(),
        };

        let ty = &field.ty;
        let type_id = self
            .parse_type_name(ty)
            .unwrap_or_else(|| panic!("Cannot parse type reference: {}", ty.to_token_stream()));
        let visibility = match &field.vis {
            SynVisibility::Public(_) => Visibility::Public,
            SynVisibility::Restricted(_) => Visibility::Restricted,
            SynVisibility::Inherited => Visibility::Private,
        };

        // Get doc comment
        let mut doc_comment = None;
        for attr in &field.attrs {
            if let Meta::NameValue(name_value) = &attr.meta {
                if name_value.path.is_ident("doc") {
                    if let Expr::Lit(expr_lit) = &name_value.value {
                        if let Lit::Str(lit_str) = &expr_lit.lit {
                            doc_comment = Some(lit_str.value().trim().to_string());
                            break;
                        }
                    }
                }
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
    fn parse_type_attrs(&mut self, type_def: &mut TypeDef, attrs: &[Attribute]) {
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
                // Check attribute is legal in this position
                match type_def {
                    TypeDef::Struct(struct_def) => {
                        check_attr_position(
                            positions,
                            AttrPositions::Struct,
                            struct_def.name(),
                            &attr_name,
                            "struct",
                        );
                    }
                    TypeDef::Enum(enum_def) => {
                        check_attr_position(
                            positions,
                            AttrPositions::Enum,
                            enum_def.name(),
                            &attr_name,
                            "enum",
                        );
                    }
                    _ => unreachable!(),
                }

                // Check this type has the relevant trait `#[generate_derive]`-ed on it
                check_attr_is_derived(
                    processor,
                    type_def.generated_derives(),
                    type_def.name(),
                    &attr_name,
                );

                let location = match type_def {
                    TypeDef::Struct(struct_def) => AttrLocation::Struct(struct_def),
                    TypeDef::Enum(enum_def) => AttrLocation::Enum(enum_def),
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
    fn parse_ast_attr(&mut self, type_def: &mut TypeDef, attr: &Attribute) {
        let parts = match &attr.meta {
            Meta::Path(_) => return,
            Meta::List(meta_list) => meta_list
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .map_err(|_| ()),
            Meta::NameValue(_) => Err(()),
        };
        let Ok(parts) = parts else {
            panic!("Unable to parse `#[ast]` attribute on `{}` type", type_def.name());
        };

        for meta in &parts {
            let attr_name = meta.path().get_ident().unwrap().to_string();
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
    fn get_generated_derives(&self, attrs: &[Attribute], type_name: &str) -> Derives {
        let mut derives = Derives::none();
        for attr in attrs {
            if attr.path().is_ident("generate_derive") {
                let args = attr.parse_args_with(Punctuated::<Ident, Token![,]>::parse_terminated);
                let Ok(args) = args else {
                    panic!("Unable to parse `#[generated_derives]` on `{type_name}` type");
                };
                for arg in args {
                    let derive_id = self.codegen.get_derive_id_by_name(&ident_name(&arg));
                    derives.add(derive_id);
                }
            }
        }

        derives
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
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .map_err(|_| ())?;
            for meta in parts {
                match &meta {
                    Meta::Path(path) => {
                        let part_name = path.get_ident().ok_or(())?.to_string();
                        process_attr_part(
                            processor,
                            attr_name,
                            location.unpack(),
                            AttrPart::Tag(&part_name),
                        )?;
                    }
                    Meta::List(meta_list) => {
                        let part_name = meta_list.path.get_ident().ok_or(())?.to_string();
                        process_attr_part(
                            processor,
                            attr_name,
                            location.unpack(),
                            AttrPart::List(&part_name, meta_list),
                        )?;
                    }
                    Meta::NameValue(name_value) => {
                        let part_name = name_value.path.get_ident().ok_or(())?.to_string();
                        let str = convert_expr_to_string(&name_value.value);
                        process_attr_part(
                            processor,
                            attr_name,
                            location.unpack(),
                            AttrPart::String(&part_name, str),
                        )?;
                    }
                };
            }
            Ok(())
        }
        Meta::NameValue(_) => Err(()),
    }
}

/// Convert an [`Expr`] to a string.
///
/// If the `Expr` is a string literal, get the value of the string.
/// Otherwise print the `Expr` as a string.
///
/// This function is also used in `Visit` generator.
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
    found_in_position: AttrPositions,
    type_name: &str,
    attr_name: &str,
    position_debug_str: &str,
) {
    assert!(
        expected_positions.contains(found_in_position),
        "`{type_name}` type has `#[{attr_name}]` attribute on a {position_debug_str}, \
        but `#[{attr_name}]` is not legal in this position."
    );
}
