extern crate cmake;

use std::env;
use std::fs::copy;
use std::path::Path;
use std::path::PathBuf;

const COMMON_FILES: &[&str] = &[
    "glslang",
    "HLSL",
    "OGLCompiler",
    "OSDependent",
    "SPIRV",
    "SPVRemapper",
];

fn main() {
    let target = env::var("TARGET").unwrap();
    let mut bin_dir = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap());
    bin_dir.push("build");

    if target.contains("x86_64-pc-windows-msvc") {
        bin_dir.push("windows");
    } else if target.contains("i686-pc-windows-msvc") {
        bin_dir = build_windows_i686();
    } else if target.contains("x86_64-unknown-linux-gnu") {
        bin_dir.push("linux");
    } else if target.contains("x86_64-apple-darwin") {
        bin_dir.push("osx");
    } else if target.contains("android") {
        if target.contains("aarch64") {
            bin_dir.push("android-arm64-v8a");
        } else if target.contains("armv7") {
            bin_dir.push("android-armeabi-v7a");
        } else {
            panic!("Missing Android target support {}", target);
        }
    } else {
        panic!("Missing target support {}", target);
    };

    // Link order matters, make sure dependents are linked before their dependencies.
    println!("cargo:rustc-link-search={}", bin_dir.to_str().unwrap());
    println!("cargo:rustc-link-lib=glslang.glsltospirv");
    println!("cargo:rustc-link-lib=HLSL.glsltospirv");
    println!("cargo:rustc-link-lib=OGLCompiler.glsltospirv");
    println!("cargo:rustc-link-lib=OSDependent.glsltospirv");
    println!("cargo:rustc-link-lib=SPIRV.glsltospirv");
    println!("cargo:rustc-link-lib=SPVRemapper.glsltospirv");
    if target.contains("x86_64-unknown-linux-gnu") || target.contains("x86_64-apple-darwin") {
        println!("cargo:rustc-link-lib=SPIRV-Tools-opt.glsltospirv");
        println!("cargo:rustc-link-lib=SPIRV-Tools.glsltospirv");
    }
    if target.contains("android") {
        println!("cargo:rustc-link-lib=c++_shared");
    }
}

fn build_windows_i686() -> PathBuf {
    // Prepare directories
    let cargo_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_dir = cargo_dir.join("glslang");

    let out_dir_env = env::var("OUT_DIR").unwrap().clone();
    let out_dir = Path::new(&out_dir_env);
    let install_dir = out_dir.join("install");
    let library_dir = install_dir.join("lib");

    // Re-use libraries if they exist
    if library_dir.exists() {
        return library_dir;
    }

    // Set up "install" subdirectory
    match std::fs::create_dir_all(&install_dir) {
        Ok(_) => {}
        Err(err) => panic!("Unable to create directory: {:?}", err),
    }

    // Configure and run build
    cmake::Config::new(&source_dir)
        .define("CMAKE_INSTALL_PREFIX", &install_dir)
        .define("ENABLE_GLSLANG_BINARIES", "OFF")
        .profile("Release")
        .build_target("install")
        .build();

    // Rename library files
    COMMON_FILES.iter().for_each(|file| {
        match copy(
            library_dir.join(file).with_extension("lib"),
            library_dir.join(file).with_extension("glsltospirv.lib"),
        ) {
            Ok(_) => {}
            Err(err) => panic!("Error copying glslang libaries: {}", err),
        }
    });

    return library_dir;
}
