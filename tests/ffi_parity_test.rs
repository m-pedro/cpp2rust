use cpp2rust::ffi::{
    aggregate_buffer_mt, analyze_file_via_ffi, format_golden_via_ffi, parse_log_line,
    CAggregatedStats, CLogRecord,
};
use std::fs;
use std::os::raw::c_char;
use std::path::Path;

#[test]
fn test_ffi_single_line_parse() {
    let line =
        b"10.0.0.42 - - [25/Oct/2026:12:34:56 +0000] \"GET /api/v1/users HTTP/1.1\" 200 4096 8.25";
    let mut rec: CLogRecord = unsafe { std::mem::zeroed() };

    let rc = unsafe {
        parse_log_line(
            line.as_ptr() as *const c_char,
            line.len(),
            &mut rec as *mut CLogRecord,
        )
    };

    assert_eq!(rc, 0, "Expected FFI parse_log_line to return 0");
    unsafe {
        assert_eq!(rec.ip.as_str(), "10.0.0.42");
        assert_eq!(rec.timestamp.as_str(), "25/Oct/2026:12:34:56 +0000");
        assert_eq!(rec.method.as_str(), "GET");
        assert_eq!(rec.path.as_str(), "/api/v1/users");
    }
    assert_eq!(rec.status_code, 200);
    assert_eq!(rec.bytes_sent, 4096);
    assert!((rec.latency_ms - 8.25).abs() < 1e-6);
}

#[test]
fn test_ffi_in_memory_aggregation() {
    let sample = concat!(
        "192.168.1.10 - - [25/Oct/2026:00:00:01 +0000] \"GET /healthz HTTP/1.1\" 200 100 2.00\n",
        "192.168.1.10 - - [25/Oct/2026:00:00:02 +0000] \"POST /login HTTP/1.1\" 500 200 100.00\n",
    );

    let mut stats = CAggregatedStats::default();
    let rc = unsafe {
        aggregate_buffer_mt(
            sample.as_ptr() as *const c_char,
            sample.len(),
            2,
            &mut stats,
        )
    };

    assert_eq!(rc, 0);
    assert_eq!(stats.total_lines, 2);
    assert_eq!(stats.total_bytes, 300);
    assert_eq!(stats.status_2xx, 1);
    assert_eq!(stats.status_5xx, 1);
    assert_eq!(stats.top_ips[0].ip_str(), "192.168.1.10");
    assert_eq!(stats.top_ips[0].count, 2);
}

#[test]
fn test_ffi_matches_cpp_golden_100k_file() {
    let log_path = Path::new("data/access.log");
    let golden_path = Path::new("cpp/tests/golden/expected_100k.txt");

    if !log_path.exists() || !golden_path.exists() {
        eprintln!(
            "Skipping 100k golden check because {:?} or {:?} was not found.",
            log_path, golden_path
        );
        return;
    }

    let expected = fs::read_to_string(golden_path)
        .expect("Failed to read cpp/tests/golden/expected_100k.txt");

    let stats = analyze_file_via_ffi(log_path.to_str().unwrap(), 4)
        .expect("FFI analysis failed on data/access.log");
    let actual = format_golden_via_ffi(&stats).expect("Golden formatting failed");

    assert_eq!(
        actual.trim(),
        expected.trim(),
        "Rust FFI output diverged from C++ golden snapshot!"
    );
}
