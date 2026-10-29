use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::slice;

pub const MAX_TOP_IPS: usize = 5;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CStringView {
    pub ptr: *const c_char,
    pub len: usize,
}

impl CStringView {
    /// Safely converts the C non-owning pointer+length span into a Rust `&str`.
    ///
    /// # Safety
    /// Caller must ensure `self.ptr` points to valid memory for at least `self.len` bytes.
    pub unsafe fn as_str<'a>(&self) -> &'a str {
        if self.ptr.is_null() || self.len == 0 {
            return "";
        }
        let bytes = unsafe { slice::from_raw_parts(self.ptr as *const u8, self.len) };
        std::str::from_utf8(bytes).unwrap_or("")
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CMappedFile {
    pub data: *const c_char,
    pub size: usize,
    pub fd: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CLogRecord {
    pub ip: CStringView,
    pub timestamp: CStringView,
    pub method: CStringView,
    pub path: CStringView,
    pub status_code: u16,
    pub bytes_sent: u64,
    pub latency_ms: f64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CIpCount {
    pub ip: [c_char; 40],
    pub count: u64,
}

impl CIpCount {
    pub fn ip_str(&self) -> String {
        let cstr = unsafe { CStr::from_ptr(self.ip.as_ptr()) };
        cstr.to_string_lossy().into_owned()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CAggregatedStats {
    pub total_lines: u64,
    pub parse_failures: u64,
    pub total_bytes: u64,
    pub status_2xx: u64,
    pub status_3xx: u64,
    pub status_4xx: u64,
    pub status_5xx: u64,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub max_latency_ms: f64,
    pub top_ips_len: usize,
    pub top_ips: [CIpCount; MAX_TOP_IPS],
}

impl Default for CAggregatedStats {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

extern "C" {
    pub fn mmap_open(filepath: *const c_char, out_file: *mut CMappedFile) -> i32;
    pub fn mmap_close(file: *mut CMappedFile);
    pub fn parse_log_line(
        line_start: *const c_char,
        line_len: usize,
        out_record: *mut CLogRecord,
    ) -> i32;
    pub fn aggregate_buffer_mt(
        data: *const c_char,
        size: usize,
        num_threads: usize,
        out_stats: *mut CAggregatedStats,
    ) -> i32;
    pub fn c_api_analyze_file(
        filepath: *const c_char,
        num_threads: usize,
        out_stats: *mut CAggregatedStats,
    ) -> i32;
    pub fn c_api_format_golden(
        stats: *const CAggregatedStats,
        out_buf: *mut c_char,
        buf_capacity: usize,
    ) -> i32;
}

/// Safe Rust wrapper around `c_api_analyze_file`
pub fn analyze_file_via_ffi(filepath: &str, num_threads: usize) -> Result<CAggregatedStats, String> {
    let c_path = CString::new(filepath).map_err(|_| "Filepath contains interior null byte")?;
    let mut stats = CAggregatedStats::default();

    let rc = unsafe { c_api_analyze_file(c_path.as_ptr(), num_threads, &mut stats) };
    if rc != 0 {
        return Err(format!("c_api_analyze_file failed with error code {}", rc));
    }
    Ok(stats)
}

/// Safe Rust wrapper around `c_api_format_golden`
pub fn format_golden_via_ffi(stats: &CAggregatedStats) -> Result<String, String> {
    let mut buf = vec![0u8; 2048];
    let written = unsafe {
        c_api_format_golden(
            stats as *const CAggregatedStats,
            buf.as_mut_ptr() as *mut c_char,
            buf.len(),
        )
    };
    if written < 0 {
        return Err("c_api_format_golden buffer overflow or error".to_string());
    }
    buf.truncate(written as usize);
    String::from_utf8(buf).map_err(|e| e.to_string())
}
