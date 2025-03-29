use rustc_version::{Channel, version_meta};

fn main() {
    if matches!(version_meta().unwrap().channel, Channel::Dev | Channel::Nightly) {
        println!("cargo:rustc-cfg=oxc_quote_is_nightly");
    }
}
