//! Copies runtime assets (config.toml, beep.wav, SDL2.dll on Windows) next
//! to the built executable, so `cargo build`/`cargo run` work on a fresh
//! checkout without any manual setup beyond the SDL2 headers/import lib
//! needed to link at all.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // OUT_DIR is target/<profile>/build/<pkg>-<hash>/out; the directory
    // holding the built executables is three levels up from there.
    let exe_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("unexpected OUT_DIR layout")
        .to_path_buf();

    for file in ["config.toml", "beep.wav", "SDL2.dll"] {
        let src = manifest_dir.join(file);
        if !src.exists() {
            continue;
        }

        let dst = exe_dir.join(file);
        if let Err(err) = fs::copy(&src, &dst) {
            println!(
                "cargo:warning=failed to copy {file} to {}: {err}",
                dst.display()
            );
        }

        println!("cargo:rerun-if-changed={file}");
    }
}
