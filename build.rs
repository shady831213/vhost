use v_build_utils::OtherDir;
fn main() {
    OtherDir::new("python").unwrap().add_dir("python").unwrap();
    OtherDir::new("sv").unwrap().add_dir("sv").unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
