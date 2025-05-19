fn main() {
    println!("cargo:rustc-check-cfg=cfg(nightly)");
    if rustversion::cfg!(nightly) {
        println!("cargo:rustc-cfg=nightly");
    }
}
