use std::{
    env,
    path::{Path, PathBuf},
};

use oxc_formatter_core::test_support::{GenerateConfig, generate_tests};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = PathBuf::from(out_dir).join("generated_tests.rs");

    let config = GenerateConfig { extensions: &["css", "scss", "less"] };

    generate_tests(&dest_path, Path::new("tests/fixtures"), &config).unwrap();
}
