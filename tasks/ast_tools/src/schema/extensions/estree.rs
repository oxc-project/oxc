/// Configuration for ESTree generator on a struct.
#[derive(Default, Debug)]
pub struct ESTreeStruct {
    pub rename: Option<String>,
    pub via: Option<String>,
    pub add_ts: Option<String>,
    pub always_flatten: bool,
    pub no_type: bool,
    pub custom_serialize: bool,
}

/// Configuration for ESTree generator on an enum.
#[derive(Default, Debug)]
pub struct ESTreeEnum {
    pub no_rename_variants: bool,
    pub custom_ts_def: bool,
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
