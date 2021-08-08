extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=ttf2mesh/ttf2mesh.c");
    println!("cargo:rustc-link-lib=m");

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // check that submodule has been initialized
    if !manifest_dir.join("ttf2mesh/ttf2mesh.h").is_file() {
        panic!("ttf2mesh.h not found - have you initialized the submodule? (`git submodule update --init`)");
    }

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    let patch_path = manifest_dir.join("ttf2mesh-fix.patch");
    let patch_file = std::fs::File::open(patch_path).unwrap();

    //// patching, see https://github.com/fetisov/ttf2mesh/pull/2
    if std::process::Command::new("patch")
        .arg("-v")
        .status()
        .is_err()
    {
        panic!("Can not apply ttf2mesh patch. Please make sure that you have `patch` command installed.")
    }

    std::process::Command::new("patch")
        .current_dir(manifest_dir.join("ttf2mesh"))
        .args("-r - --forward -p1".split(" "))
        .stdin(patch_file)
        .status()
        .unwrap();
    //// /patching

    cc::Build::new()
        .flag("-Wall")
        .flag("-Wextra")
        .flag("-pedantic")
        .flag("-std=c99")
        .flag("-D_POSIX_C_SOURCE=199309L")
        .file("ttf2mesh/ttf2mesh.c")
        .compile("ttf2mesh");
}
