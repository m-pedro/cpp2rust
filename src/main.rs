#![forbid(unsafe_code)]

use cpp2rust::aggregator::aggregate_buffer_par;
use cpp2rust::mmap::MappedLogFile;
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

    let mapped = match MappedLogFile::open(filepath) {
        Ok(m) => m,
        Err(err) => {
            eprintln!("Error opening {}: {}", filepath, err);
            process::exit(1);
        }
    };

    let start = Instant::now();
    let stats = aggregate_buffer_par(mapped.as_slice());
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    if golden_mode {
        print!("{}", stats.format_golden());
    } else {
        println!("=== Pure Rust Rayon LogEngine (Day 13) ===");
        println!("File Size      : {:.2} MB", mapped.len() as f64 / (1024.0 * 1024.0));
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
        println!("Top {} IPs      :", stats.top_ips.len());
        for (i, entry) in stats.top_ips.iter().enumerate() {
            println!("  #{} {:<15} ({} reqs)", i + 1, entry.ip, entry.count);
        }
        println!("Elapsed Time   : {:.2} ms", elapsed_ms);
    }
}
