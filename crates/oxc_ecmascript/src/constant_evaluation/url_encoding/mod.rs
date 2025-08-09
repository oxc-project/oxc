mod enc;
mod dec;

pub use enc::{encode as encode_uri_chars, is_uri_always_unescaped};
pub use dec::decode as decode_uri_chars;
