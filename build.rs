fn main() {
    // Re-run build.rs automatically if any C or C++ file changes
    println!("cargo:rerun-if-changed=cpp/include/log_engine.h");
    println!("cargo:rerun-if-changed=cpp/include/c_api.h");
    println!("cargo:rerun-if-changed=cpp/src/mmap_reader.c");
    println!("cargo:rerun-if-changed=cpp/src/parser.cpp");
    println!("cargo:rerun-if-changed=cpp/src/aggregator.cpp");
    println!("cargo:rerun-if-changed=cpp/src/c_api.cpp");

    // 1. Compile the C11 POSIX mmap reader
    cc::Build::new()
        .file("cpp/src/mmap_reader.c")
        .include("cpp/include")
        .std("c11")
        .opt_level(3)
        .warnings(true)
        .compile("legacy_c");

    // 2. Compile the C++17 parser, multithreaded aggregator, and C ABI wrapper
    cc::Build::new()
        .cpp(true)
        .file("cpp/src/parser.cpp")
        .file("cpp/src/aggregator.cpp")
        .file("cpp/src/c_api.cpp")
        .include("cpp/include")
        .std("c++17")
        .opt_level(3)
        .warnings(true)
        .compile("legacy_cpp");
}
