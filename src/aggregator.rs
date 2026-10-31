#![forbid(unsafe_code)]

use crate::parser::parse_log_line_rs;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fmt::Write;

pub const MAX_TOP_IPS: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpCount {
    pub ip: String,
    pub count: u64,
}

#[derive(Debug, Clone, Default)]
pub struct AggregatedStats {
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
    pub top_ips: Vec<IpCount>,
}

impl AggregatedStats {
    /// Formats the stats into the exact deterministic key=value format as C++ `--golden`.
    pub fn format_golden(&self) -> String {
        let mut out = String::with_capacity(512);
        let _ = writeln!(out, "total_lines={}", self.total_lines);
        let _ = writeln!(out, "parse_failures={}", self.parse_failures);
        let _ = writeln!(out, "total_bytes={}", self.total_bytes);
        let _ = writeln!(out, "status_2xx={}", self.status_2xx);
        let _ = writeln!(out, "status_3xx={}", self.status_3xx);
        let _ = writeln!(out, "status_4xx={}", self.status_4xx);
        let _ = writeln!(out, "status_5xx={}", self.status_5xx);
        let _ = writeln!(out, "avg_latency_ms={:.4}", self.avg_latency_ms);
        let _ = writeln!(out, "p95_latency_ms={:.2}", self.p95_latency_ms);
        let _ = writeln!(out, "p99_latency_ms={:.2}", self.p99_latency_ms);
        let _ = writeln!(out, "max_latency_ms={:.2}", self.max_latency_ms);
        for (i, entry) in self.top_ips.iter().enumerate() {
            let _ = writeln!(out, "top_ip_{}={}:{}", i + 1, entry.ip, entry.count);
        }
        out
    }
}

#[derive(Default)]
struct ChunkState<'a> {
    total_lines: u64,
    parse_failures: u64,
    total_bytes: u64,
    status_2xx: u64,
    status_3xx: u64,
    status_4xx: u64,
    status_5xx: u64,
    latency_sum: f64,
    max_latency: f64,
    latencies: Vec<f64>,
    ip_counts: HashMap<&'a str, u64>,
}

impl<'a> ChunkState<'a> {
    fn process_slice(slice: &'a [u8]) -> Self {
        let mut state = ChunkState {
            latencies: Vec::with_capacity(slice.len() / 80),
            ..Default::default()
        };

        for line in slice.split(|&b| b == b'\n') {
            if line.is_empty() || line == b"\r" {
                continue;
            }
            match parse_log_line_rs(line) {
                Ok(rec) => {
                    state.total_lines += 1;
                    state.total_bytes += rec.bytes_sent;
                    state.latency_sum += rec.latency_ms;
                    if rec.latency_ms > state.max_latency {
                        state.max_latency = rec.latency_ms;
                    }
                    state.latencies.push(rec.latency_ms);

                    match rec.status_code {
                        200..=299 => state.status_2xx += 1,
                        300..=399 => state.status_3xx += 1,
                        400..=499 => state.status_4xx += 1,
                        500..=599 => state.status_5xx += 1,
                        _ => {}
                    }

                    *state.ip_counts.entry(rec.ip).or_insert(0) += 1;
                }
                Err(_) => {
                    state.parse_failures += 1;
                }
            }
        }
        state
    }

    fn merge(mut self, other: Self) -> Self {
        self.total_lines += other.total_lines;
        self.parse_failures += other.parse_failures;
        self.total_bytes += other.total_bytes;
        self.status_2xx += other.status_2xx;
        self.status_3xx += other.status_3xx;
        self.status_4xx += other.status_4xx;
        self.status_5xx += other.status_5xx;
        self.latency_sum += other.latency_sum;
        if other.max_latency > self.max_latency {
            self.max_latency = other.max_latency;
        }
        self.latencies.extend(other.latencies);
        for (ip, count) in other.ip_counts {
            *self.ip_counts.entry(ip).or_insert(0) += count;
        }
        self
    }
}

/// Splits `data` into `num_chunks` newline-aligned sub-slices without copying bytes.
fn split_newline_aligned(data: &[u8], num_chunks: usize) -> Vec<&[u8]> {
    if data.is_empty() {
        return Vec::new();
    }
    let chunks = num_chunks.max(1);
    let approx = data.len() / chunks;
    if approx == 0 {
        return vec![data];
    }

    let mut slices = Vec::with_capacity(chunks);
    let mut start = 0;

    for i in 1..chunks {
        let candidate = (i * approx).max(start);
        if candidate >= data.len() {
            break;
        }
        match data[candidate..].iter().position(|&b| b == b'\n') {
            Some(offset) => {
                let end = candidate + offset + 1;
                slices.push(&data[start..end]);
                start = end;
            }
            None => break,
        }
    }

    if start < data.len() {
        slices.push(&data[start..]);
    }
    slices
}

fn compute_percentile(sorted: &[f64], pct: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (pct / 100.0) * ((sorted.len() - 1) as f64);
    let idx = (rank.round() as usize).min(sorted.len() - 1);
    sorted[idx]
}

/// Aggregates an entire HTTP access log byte slice in parallel using Rayon.
pub fn aggregate_buffer_par(data: &[u8]) -> AggregatedStats {
    if data.is_empty() {
        return AggregatedStats::default();
    }

    let num_chunks = (rayon::current_num_threads() * 4).max(4);
    let chunks = split_newline_aligned(data, num_chunks);

    let mut merged = chunks
        .into_par_iter()
        .map(ChunkState::process_slice)
        .reduce(ChunkState::default, ChunkState::merge);

    let mut stats = AggregatedStats {
        total_lines: merged.total_lines,
        parse_failures: merged.parse_failures,
        total_bytes: merged.total_bytes,
        status_2xx: merged.status_2xx,
        status_3xx: merged.status_3xx,
        status_4xx: merged.status_4xx,
        status_5xx: merged.status_5xx,
        max_latency_ms: merged.max_latency,
        ..Default::default()
    };

    if merged.total_lines > 0 {
        stats.avg_latency_ms = merged.latency_sum / (merged.total_lines as f64);
        merged.latencies.par_sort_unstable_by(|a, b| a.total_cmp(b));
        stats.p95_latency_ms = compute_percentile(&merged.latencies, 95.0);
        stats.p99_latency_ms = compute_percentile(&merged.latencies, 99.0);
    }

    let mut sorted_ips: Vec<(&str, u64)> = merged.ip_counts.into_iter().collect();
    sorted_ips.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

    stats.top_ips = sorted_ips
        .into_iter()
        .take(MAX_TOP_IPS)
        .map(|(ip, count)| IpCount {
            ip: ip.to_string(),
            count,
        })
        .collect();

    stats
}
