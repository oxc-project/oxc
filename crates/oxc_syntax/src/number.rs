/// <https://tc39.es/ecma262/#sec-numeric-types-number-tostring>
#[cfg(feature = "to_js_string")]
pub trait ToJsString {
    fn to_js_string(&self) -> String;
}

#[cfg(feature = "to_js_string")]
impl ToJsString for f64 {
    fn to_js_string(&self) -> String {
        let mut buffer = zmij_ecma::Buffer::new();
        buffer.format(*self).to_string()
    }
}
