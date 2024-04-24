#![allow(dead_code)]

use layout_inspect::inspect;

use oxc_ast::ast::Program;

pub fn main() {
    let types = inspect::<Program>();
    let json = serde_json::to_string_pretty(&types).unwrap();
    println!("{json}");
}
