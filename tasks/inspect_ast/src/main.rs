#![allow(dead_code)]

use layout_inspect::inspect;

use oxc_ast::ast::FakeForTestingInheritedTypes;

pub fn main() {
    let types = inspect::<FakeForTestingInheritedTypes>();
    let json = serde_json::to_string_pretty(&types).unwrap();
    println!("{json}");
}
