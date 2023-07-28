// Instruction:
// run `cargo run -p oxc_resolver --example resolver -- `pwd` test.js`
// or `cargo watch -x "run -p oxc_resolver --example resolver" -- `pwd` test.js`

use std::{env, path::PathBuf};

use oxc_resolver::{AliasValue, ResolveOptions, Resolver};

fn main() {
    let path = env::args().nth(1).expect("require path");
    let request = env::args().nth(2).expect("require request");
    let path = PathBuf::from(path).canonicalize().unwrap();

    println!("path: {path:?}");
    println!("request: {request}");

    let options = ResolveOptions {
        alias: vec![("/asdf".into(), vec![AliasValue::Path("./test.js".into())])],
        ..ResolveOptions::default()
    };
    let resolved_path = Resolver::new(options).resolve(path, &request);

    println!("Result: {resolved_path:?}");
}
