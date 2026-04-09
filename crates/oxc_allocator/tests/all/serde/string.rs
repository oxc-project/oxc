#![cfg(all(feature = "collections", feature = "serde"))]

use super::assert_eq_json;

use bumpalo::{collections::string::String, Bump};

macro_rules! compare_std_str {
    (in $bump:ident; $x:expr) => {
        (
            String::from_str_in($x, &$bump),
            std::string::String::from($x),
        )
    };
}

#[test]
fn test_string_serializes() {
    let bump = Bump::new();
    let (str, std_str) = compare_std_str![in bump; "hello world !"];
    assert_eq_json!(str, std_str);
    let de: std::string::String =
        serde_json::from_str(&serde_json::to_string(&str).unwrap()).unwrap();
    assert_eq!(de, std_str);
}
