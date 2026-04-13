fn main() {
    println!("cargo:rerun-if-env-changed=BUILD_TAG");

    if let Ok(v) = std::env::var("BUILD_TAG")
        && v.len() > 0
    {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={}", v);
    }
}
