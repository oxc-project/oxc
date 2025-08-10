mod dec;
mod enc;

pub use dec::decode as decode_uri_chars;
pub use enc::{encode as encode_uri_chars, is_uri_always_unescaped};
