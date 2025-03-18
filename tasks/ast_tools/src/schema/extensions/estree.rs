/// Configuration for ESTree generator on a struct.
#[derive(Default, Debug)]
pub struct ESTreeStruct {
    pub rename: Option<String>,
    pub via: Option<String>,
    pub skip: bool,
    pub flatten: bool,
    pub no_type: bool,
    /// Additional fields to add to struct in ESTree AST.
    /// `(name, converter)` where `name` is the name of the field, and `converter` is name of
    /// a converter meta type.
    pub add_fields: Vec<(String, String)>,
    /// Custom field order.
    /// Contains field indices. Entries are:
    /// * Actual struct field: index of the field.
    /// * Added field: `struct_def.fields.len() + added_field_index`.
    /// Does not include `type` field, if it's automatically added.
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
}

/// Configuration for ESTree generator on an enum.
#[derive(Default, Debug)]
pub struct ESTreeEnum {
    pub via: Option<String>,
    pub skip: bool,
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
    pub rename: Option<String>,
    pub via: Option<String>,
    pub ts_type: Option<String>,
    /// Field index of field to append to this one
    pub append_field_index: Option<usize>,
    pub skip: bool,
    pub flatten: bool,
    pub no_flatten: bool,
    // `true` for fields containing a `&str` or `Atom` which does not need escaping in JSON
    pub json_safe: bool,
    pub is_ts: bool,
}

/// Configuration for ESTree generator on an enum variant.
#[derive(Default, Debug)]
pub struct ESTreeEnumVariant {
    pub rename: Option<String>,
    pub via: Option<String>,
    pub is_ts: bool,
}

/// Configuration for ESTree generator on a meta type.
#[derive(Default, Debug)]
pub struct ESTreeMeta {
    pub ts_type: Option<String>,
    // JS code for raw transfer deserializer
    pub raw_deser: Option<String>,
    /// `true` if meta type is for a struct field which is present only in TS AST
    pub is_ts: bool,
}
