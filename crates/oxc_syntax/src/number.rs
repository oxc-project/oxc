#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumberBase {
    Float,
    Decimal,
    Binary,
    Octal,
    Hex,
}

impl NumberBase {
    pub fn is_base_10(&self) -> bool {
        matches!(self, Self::Float | Self::Decimal)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BigintBase {
    Decimal,
    Binary,
    Octal,
    Hex,
}

impl BigintBase {
    pub fn is_base_10(&self) -> bool {
        self == &Self::Decimal
    }
}

/// <https://tc39.es/ecma262/#sec-numeric-types-number-tostring>
#[cfg(feature = "to_js_string")]
pub trait ToJsString {
    fn to_js_string(&self) -> String;
}

#[cfg(feature = "to_js_string")]
impl ToJsString for f64 {
    fn to_js_string(&self) -> String {
        let mut buffer = ryu_js::Buffer::new();
        buffer.format(*self).to_string()
    }
}
