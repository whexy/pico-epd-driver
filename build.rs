use std::{env, fs, path::PathBuf};

fn main() {
    // Locate OUT_DIR
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // Copy `memory.x` from crate root into OUT_DIR
    let src = PathBuf::from("memory.x");
    let dst = out_dir.join("memory.x");
    fs::copy(&src, &dst).expect("copy memory.x");

    // Tell rustc where to find it
    println!("cargo:rustc-link-search={}", out_dir.display());

    // Re-run build script if memory.x changes
    println!("cargo:rerun-if-changed=memory.x");

    // Extra linker args
    println!("cargo:rustc-link-arg=--nmagic");
    println!("cargo:rustc-link-arg=-Tlink.x");
}
