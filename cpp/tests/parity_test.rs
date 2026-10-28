use std::ffi::CString;
use std::fs;
use tempfile::NamedTempFile;
use cpp2rust::ffi::generate_data_cpp; // Assuming your src/lib.rs exports `ffi`

#[test]
fn test_output_parity_100k() {
    // 1. Setup isolated temporary file for C++ to write into
    let temp_file = NamedTempFile::new().expect("Failed to create temporary file");
    let temp_path_str = temp_file.path().to_str().expect("Path contains invalid UTF-8");
    
    // Convert Rust string to C-compatible null-terminated string
    let c_path = CString::new(temp_path_str).expect("CString::new failed");

    // 2. Execute legacy C++ code via FFI
    let result = unsafe {
        // Request 100,000 records
        generate_data_cpp(c_path.as_ptr(), 100_000)
    };
    
    // Ensure the C++ function didn't throw an exception (returned 0)
    assert_eq!(result, 0, "C++ FFI call returned an error code");

    // 3. Load generated data and the expected baseline
    let generated_output = fs::read_to_string(temp_file.path())
        .expect("Failed to read the FFI-generated output file");
        
    // Place your baseline file in a tests/fixtures/ directory
    let expected_output = fs::read_to_string("tests/fixtures/expected_100k.txt")
        .expect("Failed to read expected_100k.txt baseline");

    // 4. Verify absolute parity
    assert_eq!(
        generated_output, 
        expected_output, 
        "Output parity failure! The FFI wrapper output does not match expected_100k.txt"
    );
}
