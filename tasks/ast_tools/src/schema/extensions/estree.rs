/// Configuration for ESTree generator on a struct.
#[derive(Default, Debug)]
pub struct ESTreeStruct {
    pub rename: Option<String>,
    pub via: Option<String>,
    pub skip: bool,
    pub flatten: bool,
    pub no_type: bool,
    /// `true` if serializer is implemented manually and should not be generated
    pub custom_serialize: bool,
    pub add_entry: Option<Vec<(String, String)>>,
    /// Additional fields to add to TS type definition
    pub add_ts: Option<String>,
    /// Custom TS type definition. Does not include `export`.
    /// Empty string if type should not have a TS type definition.
    pub custom_ts_def: Option<String>,
    /// Additional custom TS type definition to add along with the generated one.
    /// Does not include `export`.
    pub add_ts_def: Option<String>,
}

/// Configuration for ESTree generator on an enum.
#[derive(Default, Debug)]
pub struct ESTreeEnum {
    pub skip: bool,
    pub no_rename_variants: bool,
    /// `true` if serializer is implemented manually and should not be generated
    pub custom_serialize: bool,
    /// Custom TS type definition. Does not include `export`.
    /// Empty string if type should not have a TS type definition.
    pub custom_ts_def: Option<String>,
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
    pub is_ts: bool,
}

/// Configuration for ESTree generator on an enum variant.
#[derive(Default, Debug)]
pub struct ESTreeEnumVariant {
    pub rename: Option<String>,
    pub is_ts: bool,
}
