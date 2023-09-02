// Instruction:
//
// run `cargo run -p oxc_resolver --example resolver -- `pwd` test.js`
// or `cargo watch -x "run -p oxc_resolver --example resolver" -- `pwd` test.js`
//
// use `OXC_RESOLVER=DEBUG` for tracing

use std::{env, path::PathBuf};

use oxc_resolver::{AliasValue, ResolveOptions, Resolver};

fn init_tracing_subscriber() {
    use tracing_subscriber::{filter::Targets, fmt, prelude::*, registry};
    registry()
        .with(std::env::var("OXC_RESOLVER").map_or_else(
            |_| Targets::new(),
            |env_var| {
                use std::str::FromStr;
                Targets::from_str(&env_var).unwrap()
            },
        ))
        .with(fmt::layer())
        .init();
}

fn main() {
    init_tracing_subscriber();

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
