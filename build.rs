use std::env;
use std::path::PathBuf;

fn main() {
    let root_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let whisper_cpp_dir = root_dir.join("whisper.cpp");

    // Build whisper.cpp using CMake
    let dst = cmake::build(&whisper_cpp_dir);

    // Link against libwhisper
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=whisper");

    // Link against C++ standard library on non-Windows platforms
    if cfg!(not(target_os = "windows")) {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }

    // Rebuild if whisper.cpp files change
    println!("cargo:rerun-if-changed={}", whisper_cpp_dir.display());
}
