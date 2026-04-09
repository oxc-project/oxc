use serde::{Deserialize, Serialize};

mod boxed;
mod string;
mod vec;

#[macro_export]
macro_rules! assert_eq_json {
    ($a:ident, $b:ident) => {
        assert_eq!(
            serde_json::to_string(&$a).unwrap(),
            serde_json::to_string(&$b).unwrap(),
        )
    };
}
use assert_eq_json;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "t", content = "c")]
enum Test {
    First,
    Second,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde()]
struct Mixed {
    i: i32,
    s: std::string::String,
    o: Option<std::string::String>,
    e: Test,
}
