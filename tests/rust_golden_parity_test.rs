use cpp2rust::aggregator::aggregate_buffer_par;
use cpp2rust::mmap::MappedLogFile;
use std::fs;
use std::path::Path;

#[test]
fn test_pure_rust_rayon_matches_cpp_golden_100k() {
    let log_path = Path::new("data/access.log");
    let golden_path = Path::new("cpp/tests/golden/expected_100k.txt");

    if !log_path.exists() || !golden_path.exists() {
        eprintln!("Skipping golden parity test because data/access.log was not found.");
        return;
    }

    let expected = fs::read_to_string(golden_path).expect("Failed to read expected_100k.txt");
    let mapped = MappedLogFile::open(log_path).expect("Failed to mmap data/access.log");

    let stats = aggregate_buffer_par(mapped.as_slice());
    let actual = stats.format_golden();

    assert_eq!(
        actual.trim(),
        expected.trim(),
        "Pure Rust Rayon aggregator diverged from C++ golden snapshot!"
    );
}
