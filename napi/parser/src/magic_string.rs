use napi_derive::napi;

#[napi]
pub struct MagicString(string_wizard::MagicString<'static>);

impl MagicString {
    pub fn new(s: String) -> Self {
        Self(string_wizard::MagicString::new(s))
    }
}

#[napi]
impl MagicString {}
