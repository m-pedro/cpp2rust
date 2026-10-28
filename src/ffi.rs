use std::os::raw::{c_char, c_int};

// Declare the external C++ function we compiled in build.rs
extern "C" {
    pub fn generate_data_cpp(output_path: *const c_char, count: usize) -> c_int;
}
