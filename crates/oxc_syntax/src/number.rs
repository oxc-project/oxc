#[cfg(feature = "to_js_string")]
use oxc_str::CompactStr;

/// <https://tc39.es/ecma262/#sec-numeric-types-number-tostring>
#[cfg(feature = "to_js_string")]
pub trait ToJsString {
    fn to_js_string(&self) -> String;

    /// Like [`ToJsString::to_js_string`], but returns a [`CompactStr`], which stores the result
    /// inline on 64-bit targets.
    fn to_js_compact_str(&self) -> CompactStr;
}

#[cfg(feature = "to_js_string")]
impl ToJsString for f64 {
    fn to_js_string(&self) -> String {
        let mut buffer = zmij_ecma::Buffer::new();
        buffer.format(*self).to_string()
    }

    fn to_js_compact_str(&self) -> CompactStr {
        let mut buffer = zmij_ecma::Buffer::new();
        CompactStr::from(buffer.format(*self))
    }
}
