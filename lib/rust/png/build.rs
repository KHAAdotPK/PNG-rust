use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;

fn main() {
    // Get the OUT_DIR path from Cargo env var
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set by Cargo");
    println!("warning: {} OUT_DIR is {}", env::var("CARGO_PKG_NAME").unwrap_or_default(), out_dir);
    
    // Since we're in PNG-rust\lib\rust\png, we need to go back to the PNG-rust directory
    // Then navigate to the Debug folder
    
    // Get current directory to help debug
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    println!("warning: Current directory is: {:?}", current_dir);
    
    // Try both absolute and relative paths to locate the zlib files
    let debug_paths = vec![
        Path::new("../../../zlib/Debug"),             // Relative path from lib/rust/png up to ./zlib/Debug
    ];
    
    // Try to find the Debug directory with zlib files
    let mut zlib_debug_dir = None;
    for path in debug_paths {
        let dll_path = path.join("zd.dll");
        let lib_path = path.join("zd.lib");
        
        println!("warning: Checking for zlib files at: {:?} and {:?}", dll_path, lib_path);
        
        if dll_path.exists() && lib_path.exists() {
            zlib_debug_dir = Some(path);
            println!("warning: Found zlib files at: {:?}", path);
            break;
        }
    }
    
    // If we couldn't find the files, panic with helpful message
    let zlib_debug_dir = zlib_debug_dir.expect("Could not find zd.dll and zd.lib in any of the checked paths");
    
    // Now that we found the directory, get the file paths
    let zd_dll_path = zlib_debug_dir.join("zd.dll");
    let zd_lib_path = zlib_debug_dir.join("zd.lib");
    
    // Copy zlib files to OUT_DIR
    let out_dll_path = Path::new(&out_dir).join("zd.dll");
    let out_lib_path = Path::new(&out_dir).join("zd.lib");
    
    println!("warning: Copying zlib files to: {:?} and {:?}", out_dll_path, out_lib_path);
    
    fs::copy(&zd_dll_path, &out_dll_path).expect("Failed to copy zd.dll");
    fs::copy(&zd_lib_path, &out_lib_path).expect("Failed to copy zd.lib");
    
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=dylib=zd"); // Link with zd.lib
    
    // Try to find sundry.c in various possible locations
    let sundry_paths = vec![        
        Path::new("../C/sundry.c"), // Relative path from lib/rust/png up to lib/rust/C        
    ];
    
    let mut sundry_path = None;
    for path in sundry_paths {
        println!("warning: Checking for sundry.c at: {:?}", path);
        if path.exists() {
            sundry_path = Some(path.to_path_buf());
            println!("warning: Found sundry.c at: {:?}", path);
            break;
        }
    }
    
    // If we couldn't find sundry.c, panic with helpful message
    let sundry_path = sundry_path.expect("Could not find sundry.c in any of the checked paths");
    
    // Try to find zlib headers in various possible locations
    let zlib_include_paths = vec![        
        Path::new("../../../zlib"), // Relative path from lib/rust/png up to ./zlib        
    ];
    
    let mut zlib_include_dir = None;
    for path in zlib_include_paths {
        let zconf_h = path.join("zconf.h");
        let zlib_h = path.join("zlib.h");
        
        println!("warning: Checking for zlib headers at: {:?}", path);
        if zconf_h.exists() || zlib_h.exists() {
            zlib_include_dir = Some(path.to_path_buf());
            println!("warning: Found zlib headers at: {:?}", path);
            break;
        }
    }
    
    // If we couldn't find zlib headers, panic with helpful message
    let zlib_include_dir = zlib_include_dir.expect("Could not find zlib headers in any of the checked paths");
    
    // Now compile sundry.c
    let status = Command::new("cl.exe")
        .args(&[
            "/LD",
            &sundry_path.to_string_lossy(),
            "/DGRAMMAR_END_OF_TOKEN_MARKER=\" \"",
            "/DGRAMMAR_END_OF_TOKEN_MARKER_SIZE=1",
            &format!("/I{}", zlib_include_dir.display()), // Include zlib headers
            "/link",
            &format!("/OUT:{}/png.dll", out_dir),
            &format!("/IMPLIB:{}/png.lib", out_dir),
            &format!("{}/zd.lib", out_dir), // Link with copied zd.lib
        ])
        .status()
        .expect("Failed to compile sundry.c");
    
    assert!(status.success(), "Failed to compile png.dll");
    
    // Tell Rust to link with our compiled PNG library
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=dylib=png");
    
    // Make sure we reload if sundry.c changes or zlib files change
    println!("cargo:rerun-if-changed={}", sundry_path.display());
    println!("cargo:rerun-if-changed={}", zd_dll_path.display());
    println!("cargo:rerun-if-changed={}", zd_lib_path.display());
}



/*use std::process::Command;
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
    println!("cargo:rustc-link-lib=dylib=png");
}*/
