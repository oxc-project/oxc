/// Configuration for ESTree generator on a struct.
#[derive(Default, Debug)]
pub struct ESTreeStruct {
    /// Value of `type` field in ESTree AST for this struct.
    /// If `None`, defaults to the name of the Rust struct.
    pub rename: Option<String>,
    /// Name of custom converter `#[ast_meta]` type to use to serialize this struct.
    pub via: Option<String>,
    /// If `true`, do not create an `ESTree` impl for this struct, and skip any struct fields containing it.
    pub skip: bool,
    /// Always flatten this struct.
    pub flatten: bool,
    /// If `true`, do not add a `type` field to this struct.
    pub no_type: bool,
    /// Additional fields to add to struct in ESTree AST.
    /// `(name, converter)` where `name` is the name of the field, and `converter` is name of
    /// a converter meta type.
    pub add_fields: Vec<(String, String)>,
    /// Custom field order.
    /// Contains field indices. Entries are:
    /// * Actual struct field: index of the field.
    /// * Added field: `struct_def.fields.len() + added_field_index`.
    ///   Does not include `type` field, if it's automatically added.
    pub field_indices: Vec<u8>,
    /// TS alias.
    /// e.g. `#[estree(ts_alias = "null")]` means this type won't have a type def generated,
    /// and any struct / enum referencing it will substitute `null` as the type.
    pub ts_alias: Option<String>,
    /// Type should not have a TS type definition.
    pub no_ts_def: bool,
    /// Additional custom TS type definition to add along with the generated one.
    /// Does not include `export`.
    pub add_ts_def: Option<String>,
    /// If `true`
    pub no_parent: bool,
}

/// Configuration for ESTree generator on an enum.
#[derive(Default, Debug)]
pub struct ESTreeEnum {
    /// Name of custom converter `#[ast_meta]` type to use to serialize this enum.
    pub via: Option<String>,
    /// If `true`, do not create an `ESTree` impl for this enum, and skip any struct fields containing it.
    pub skip: bool,
    /// Control value for fieldless variant.
    /// If `false` (default), variant value will be variant's name converted to camel case.
    /// If `true`, variant value will be the variant's name exactly as in the Rust type.
    /// Applies to all the enum's variants.
    /// Can be overridden with `#[estree(rename = "...")]` on individual variants.
    pub no_rename_variants: bool,
    /// TS alias.
    /// e.g. `#[estree(ts_alias = "null")]` means this type won't have a type def generated,
    /// and any struct / enum referencing it will substitute `null` as the type.
    pub ts_alias: Option<String>,
    /// Type should not have a TS type definition.
    pub no_ts_def: bool,
    /// Additional custom TS type definition to add along with the generated one.
    /// Does not include `export`.
    pub add_ts_def: Option<String>,
}

/// Configuration for ESTree generator on a struct field.
#[derive(Default, Debug)]
pub struct ESTreeStructField {
    /// Rename field. Name should be camel case.
    pub rename: Option<String>,
    /// Name of custom serializer `#[ast_meta]` type to use to serialize this field.
    pub via: Option<String>,
    /// TS type of this field.
    pub ts_type: Option<String>,
    /// Field index of field to prepend to this one
    pub prepend_field_index: Option<usize>,
    /// Field index of field to append to this one
    pub append_field_index: Option<usize>,
    /// Skip this struct field.
    /// Field will also be skipped if the type of the field is marked `#[estree(skip)]`.
    pub skip: bool,
    /// Flatten field.
    pub flatten: bool,
    /// No not flatten field. Overrides `#[estree(flatten)]` on the type of the field.
    pub no_flatten: bool,
    /// `true` for fields containing a `&str` or `Atom` which does not need escaping in JSON
    pub json_safe: bool,
    /// `true` if field is only included in JS ESTree AST (not TS-ESTree AST).
    pub is_js: bool,
    /// `true` if field is only included in TS-ESTree AST (not JS ESTree AST).
    pub is_ts: bool,
}

/// Configuration for ESTree generator on an enum variant.
#[derive(Default, Debug)]
pub struct ESTreeEnumVariant {
    /// Skip this enum variant.
    pub skip: bool,
    /// Override variant value for fieldless variant.
    /// Otherwise, value depends on whether `#[estree(no_rename_variants)]` is present on enum.
    /// See `no_rename_variants` field of [`ESTreeEnum`] for more info.
    pub rename: Option<String>,
    /// Name of custom serializer `#[ast_meta]` type to use to serialize this variant.
    pub via: Option<String>,
}

/// Configuration for ESTree converter/serializer on a meta type.
#[derive(Default, Debug)]
pub struct ESTreeMeta {
    /// TS type of the value produced by this converter/serializer.
    pub ts_type: Option<String>,
    /// JS code for raw transfer deserializer.
    pub raw_deser: Option<String>,
    /// `true` if meta type is for a struct field which is present only in JS AST.
    pub is_js: bool,
    /// `true` if meta type is for a struct field which is present only in TS AST.
    pub is_ts: bool,
}
