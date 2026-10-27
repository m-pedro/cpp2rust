#include "log_engine.h"

#include <algorithm>
#include <cmath>
#include <cstring>
#include <mutex>
#include <string>
#include <string_view>
#include <thread>
#include <unordered_map>
#include <vector>

namespace {

struct ThreadLocalState {
    uint64_t total_lines = 0;
    uint64_t parse_failures = 0;
    uint64_t total_bytes = 0;
    uint64_t status_2xx = 0;
    uint64_t status_3xx = 0;
    uint64_t status_4xx = 0;
    uint64_t status_5xx = 0;
    double latency_sum = 0.0;
    double max_latency = 0.0;
    std::vector<double> latencies;
    std::unordered_map<std::string_view, uint64_t> ip_counts;
};

void process_chunk(const char* chunk_start, const char* chunk_end, ThreadLocalState& local) {
    const char* cur = chunk_start;
    while (cur < chunk_end) {
        const char* newline = static_cast<const char*>(
            std::memchr(cur, '\n', static_cast<size_t>(chunk_end - cur))
        );
        const char* line_end = newline ? newline : chunk_end;
        size_t line_len = static_cast<size_t>(line_end - cur);

        if (line_len > 0) {
            LogRecord rec{};
            if (parse_log_line(cur, line_len, &rec) == 0) {
                ++local.total_lines;
                local.total_bytes += rec.bytes_sent;
                local.latency_sum += rec.latency_ms;
                if (rec.latency_ms > local.max_latency) {
                    local.max_latency = rec.latency_ms;
                }
                local.latencies.push_back(rec.latency_ms);

                if (rec.status_code >= 200 && rec.status_code < 300) {
                    ++local.status_2xx;
                } else if (rec.status_code >= 300 && rec.status_code < 400) {
                    ++local.status_3xx;
                } else if (rec.status_code >= 400 && rec.status_code < 500) {
                    ++local.status_4xx;
                } else if (rec.status_code >= 500 && rec.status_code < 600) {
                    ++local.status_5xx;
                }

                std::string_view ip_sv(rec.ip.ptr, rec.ip.len);
                local.ip_counts[ip_sv] += 1;
            } else {
                ++local.parse_failures;
            }
        }
        cur = newline ? (newline + 1) : chunk_end;
    }
}

double compute_percentile(std::vector<double>& sorted_vals, double pct) {
    if (sorted_vals.empty()) {
        return 0.0;
    }
    double rank = (pct / 100.0) * static_cast<double>(sorted_vals.size() - 1);
    size_t idx = static_cast<size_t>(std::round(rank));
    if (idx >= sorted_vals.size()) {
        idx = sorted_vals.size() - 1;
    }
    return sorted_vals[idx];
}

} // namespace

extern "C" int aggregate_buffer_mt(
    const char* data,
    size_t size,
    size_t num_threads,
    AggregatedStats* out_stats
) {
    if (!data || size == 0 || !out_stats) {
        return -1;
    }

    std::memset(out_stats, 0, sizeof(AggregatedStats));

    if (num_threads == 0) {
        unsigned int hw = std::thread::hardware_concurrency();
        num_threads = hw > 0 ? static_cast<size_t>(hw) : 4;
    }

    // 1. Compute newline-aligned chunk boundaries so no log line is split in half
    std::vector<const char*> boundaries;
    boundaries.reserve(num_threads + 1);
    boundaries.push_back(data);

    const char* data_end = data + size;
    size_t approx_chunk = size / num_threads;

    for (size_t i = 1; i < num_threads; ++i) {
        const char* candidate = data + (i * approx_chunk);
        if (candidate < boundaries.back()) {
            candidate = boundaries.back();
        }
        if (candidate < data_end) {
            const char* nl = static_cast<const char*>(
                std::memchr(candidate, '\n', static_cast<size_t>(data_end - candidate))
            );
            candidate = nl ? (nl + 1) : data_end;
        }
        boundaries.push_back(candidate);
    }
    boundaries.push_back(data_end);

    // 2. Spawn worker threads and merge into shared state under a mutex
    std::mutex merge_mutex;
    ThreadLocalState merged;
    merged.latencies.reserve(size / 80);

    std::vector<std::thread> workers;
    workers.reserve(num_threads);

    for (size_t i = 0; i < num_threads; ++i) {
        const char* start = boundaries[i];
        const char* end = boundaries[i + 1];
        if (start >= end) {
            continue;
        }

        workers.emplace_back([start, end, &merge_mutex, &merged]() {
            ThreadLocalState local;
            local.latencies.reserve(static_cast<size_t>(end - start) / 80);
            process_chunk(start, end, local);

            std::lock_guard<std::mutex> lock(merge_mutex);
            merged.total_lines += local.total_lines;
            merged.parse_failures += local.parse_failures;
            merged.total_bytes += local.total_bytes;
            merged.status_2xx += local.status_2xx;
            merged.status_3xx += local.status_3xx;
            merged.status_4xx += local.status_4xx;
            merged.status_5xx += local.status_5xx;
            merged.latency_sum += local.latency_sum;
            if (local.max_latency > merged.max_latency) {
                merged.max_latency = local.max_latency;
            }
            merged.latencies.insert(
                merged.latencies.end(),
                local.latencies.begin(),
                local.latencies.end()
            );
            for (const auto& kv : local.ip_counts) {
                merged.ip_counts[kv.first] += kv.second;
            }
        });
    }

    for (auto& t : workers) {
        if (t.joinable()) {
            t.join();
        }
    }

    // 3. Populate final AggregatedStats struct
    out_stats->total_lines = merged.total_lines;
    out_stats->parse_failures = merged.parse_failures;
    out_stats->total_bytes = merged.total_bytes;
    out_stats->status_2xx = merged.status_2xx;
    out_stats->status_3xx = merged.status_3xx;
    out_stats->status_4xx = merged.status_4xx;
    out_stats->status_5xx = merged.status_5xx;
    out_stats->max_latency_ms = merged.max_latency;

    if (merged.total_lines > 0) {
        out_stats->avg_latency_ms = merged.latency_sum / static_cast<double>(merged.total_lines);
        std::sort(merged.latencies.begin(), merged.latencies.end());
        out_stats->p95_latency_ms = compute_percentile(merged.latencies, 95.0);
        out_stats->p99_latency_ms = compute_percentile(merged.latencies, 99.0);
    }

    // 4. Sort Top IPs deterministically (by count DESC, then IP string ASC)
    std::vector<std::pair<std::string_view, uint64_t>> sorted_ips(
        merged.ip_counts.begin(), merged.ip_counts.end()
    );
    std::sort(sorted_ips.begin(), sorted_ips.end(), [](const auto& a, const auto& b) {
        if (a.second != b.second) {
            return a.second > b.second;
        }
        return a.first < b.first;
    });

    size_t top_n = std::min(sorted_ips.size(), static_cast<size_t>(MAX_TOP_IPS));
    out_stats->top_ips_len = top_n;
    for (size_t i = 0; i < top_n; ++i) {
        size_t copy_len = std::min(sorted_ips[i].first.size(), sizeof(out_stats->top_ips[i].ip) - 1);
        std::memcpy(out_stats->top_ips[i].ip, sorted_ips[i].first.data(), copy_len);
        out_stats->top_ips[i].ip[copy_len] = '\0';
        out_stats->top_ips[i].count = sorted_ips[i].second;
    }

    return 0;
}
