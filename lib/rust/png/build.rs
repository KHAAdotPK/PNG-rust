use std::process::Command;
use std::path::Path;
use std::env;

fn main() {
    // Get the OUT_DIR path from Cargo env var
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set by Cargo");

    // Build sundry.c -> png.dll + png.lib
    let status_start = Command::new("cl.exe")
        .args(&[
            "/LD",
            "..\\C\\sundry.c",
            "/DGRAMMAR_END_OF_TOKEN_MARKER=\" \"",
            "/DGRAMMAR_END_OF_TOKEN_MARKER_SIZE=1",
            "/link",
            &format!("/OUT:{}/png.dll", out_dir),
            &format!("/IMPLIB:{}/png.lib", out_dir),
        ])
        .status()
        .expect("Failed to compile sundry.c");

    assert!(status_start.success(), "Failed to compile png.dll");
    
    // Tell Rust to link from OUT_DIR
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=dylib=clap");
}
