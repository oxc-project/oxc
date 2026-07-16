/// Details for `ContentEq` derive on a struct or enum.
#[derive(Default, Debug)]
pub struct ContentEqType {
    /// `true` if type should ignored by `ContentEq`
    pub skip: bool,
}

/// Details for `ContentEq` derive on a struct field.
#[derive(Default, Debug)]
pub struct ContentEqStructField {
    /// `true` if field should ignored by `ContentEq`
    pub skip: bool,
}

/// Details for `ContentEq` derive on an enum variant.
#[derive(Default, Debug)]
pub struct ContentEqEnumVariant {
    /// `true` if variant's field should be ignored by `ContentEq`
    pub skip: bool,
}
