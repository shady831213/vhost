use std::env;
use v_build_utils::OtherDir;
fn main() {
    if env::var("CARGO_FEATURE_PYO3").is_ok() {
        OtherDir::new("python").unwrap().add_dir("python").unwrap();
    }
    println!("cargo:rerun-if-changed=build.rs");
}
