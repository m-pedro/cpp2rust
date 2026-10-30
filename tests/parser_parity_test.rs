use cpp2rust::ffi::{parse_log_line, CLogRecord};
use cpp2rust::mmap::MappedLogFile;
use cpp2rust::parser::parse_log_line_rs;
use std::os::raw::c_char;
use std::path::Path;

#[test]
fn test_rust_vs_cpp_parser_line_by_line_parity() {
    let sample_lines = [
        &b"10.0.0.1 - - [25/Oct/2026:00:00:01 +0000] \"GET /api/v1/users HTTP/1.1\" 200 1024 4.52"[..],
        &b"172.16.0.12 - - [25/Oct/2026:00:00:02 +0000] \"POST /api/v1/checkout HTTP/1.1\" 503 64 342.10"[..],
        &b"192.168.1.25 - - [25/Oct/2026:00:00:03 +0000] \"DELETE /api/v1/items/42 HTTP/1.1\" 204 0 1.05"[..],
    ];

    for line in sample_lines {
        let rs_rec = parse_log_line_rs(line).expect("Rust parser failed on valid line");

        let mut c_rec: CLogRecord = unsafe { std::mem::zeroed() };
        let rc = unsafe {
            parse_log_line(
                line.as_ptr() as *const c_char,
                line.len(),
                &mut c_rec as *mut CLogRecord,
            )
        };
        assert_eq!(rc, 0, "C++ parser failed on valid line");

        unsafe {
            assert_eq!(rs_rec.ip, c_rec.ip.as_str());
            assert_eq!(rs_rec.timestamp, c_rec.timestamp.as_str());
            assert_eq!(rs_rec.method, c_rec.method.as_str());
            assert_eq!(rs_rec.path, c_rec.path.as_str());
        }
        assert_eq!(rs_rec.status_code, c_rec.status_code);
        assert_eq!(rs_rec.bytes_sent, c_rec.bytes_sent);
        assert!((rs_rec.latency_ms - c_rec.latency_ms).abs() < 1e-6);
    }
}

#[test]
fn test_rust_mmap_and_parser_on_100k_dataset() {
    let log_path = Path::new("data/access.log");
    if !log_path.exists() {
        return;
    }

    let mapped = MappedLogFile::open(log_path).expect("Failed to mmap data/access.log in Rust");
    let slice = mapped.as_slice();

    let mut total_parsed = 0u64;
    let mut total_bytes = 0u64;
    let mut status_5xx = 0u64;

    for line in slice.split(|&b| b == b'\n') {
        if line.is_empty() {
            continue;
        }
        let rec = parse_log_line_rs(line).expect("Failed to parse line from access.log");
        total_parsed += 1;
        total_bytes += rec.bytes_sent;
        if (500..600).contains(&rec.status_code) {
            status_5xx += 1;
        }
    }

    assert_eq!(total_parsed, 100_000);
    assert!(total_bytes > 0);
    assert!(status_5xx > 0);
}
