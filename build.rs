fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper or legacy source changes
    println!("cargo:rerun-if-changed=cpp_src/wrapper.cpp");
    println!("cargo:rerun-if-changed=cpp_src/legacy_engine.cpp");
    println!("cargo:rerun-if-changed=cpp_src/legacy_engine.h");

    // Compile the C++ files into a static library named "liblegacy.a"
    cc::Build::new()
        .cpp(true) // Enable C++ compiler
        .flag_if_supported("-std=c++17") // Use appropriate C++ standard
        .file("cpp_src/wrapper.cpp")
        .file("cpp_src/legacy_engine.cpp")
        .compile("legacy"); 
}
