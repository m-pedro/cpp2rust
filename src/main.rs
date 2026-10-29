use cpp2rust::ffi::{analyze_file_via_ffi, format_golden_via_ffi};
use std::env;
use std::process;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage: logengine_rs <path/to/access.log> [--golden]");
        process::exit(1);
    }

    let filepath = &args[0];
    let golden_mode = args.iter().any(|a| a == "--golden");

    let start = Instant::now();
    let stats = match analyze_file_via_ffi(filepath, 0) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    };
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    if golden_mode {
        match format_golden_via_ffi(&stats) {
            Ok(report) => print!("{}", report),
            Err(err) => {
                eprintln!("Formatting error: {}", err);
                process::exit(1);
            }
        }
    } else {
        println!("=== Hybrid Rust+FFI LogEngine (Day 7) ===");
        println!(
            "Parsed Lines   : {} (Failures: {})",
            stats.total_lines, stats.parse_failures
        );
        println!("Total Bytes    : {}", stats.total_bytes);
        println!(
            "Status Codes   : 2xx={} | 3xx={} | 4xx={} | 5xx={}",
            stats.status_2xx, stats.status_3xx, stats.status_4xx, stats.status_5xx
        );
        println!(
            "Latency (ms)   : Avg={:.2} | p95={:.2} | p99={:.2} | Max={:.2}",
            stats.avg_latency_ms,
            stats.p95_latency_ms,
            stats.p99_latency_ms,
            stats.max_latency_ms
        );
        println!("Top {} IPs      :", stats.top_ips_len);
        for i in 0..stats.top_ips_len {
            println!(
                "  #{} {:<15} ({} reqs)",
                i + 1,
                stats.top_ips[i].ip_str(),
                stats.top_ips[i].count
            );
        }
        println!("Elapsed Time   : {:.2} ms", elapsed_ms);
    }
}
