#include "c_api.h"

#include <cstdio>
#include <cstring>

extern "C" int c_api_analyze_file(
    const char* filepath,
    size_t num_threads,
    AggregatedStats* out_stats
) {
    if (!filepath || !out_stats) {
        return -1;
    }

    MappedFile file{};
    if (mmap_open(filepath, &file) != 0) {
        return -2;
    }

    int rc = aggregate_buffer_mt(file.data, file.size, num_threads, out_stats);
    mmap_close(&file);
    return rc == 0 ? 0 : -3;
}

extern "C" int c_api_format_golden(
    const AggregatedStats* stats,
    char* out_buf,
    size_t buf_capacity
) {
    if (!stats || !out_buf || buf_capacity == 0) {
        return -1;
    }

    int written = std::snprintf(
        out_buf,
        buf_capacity,
        "total_lines=%llu\n"
        "parse_failures=%llu\n"
        "total_bytes=%llu\n"
        "status_2xx=%llu\n"
        "status_3xx=%llu\n"
        "status_4xx=%llu\n"
        "status_5xx=%llu\n"
        "avg_latency_ms=%.4f\n"
        "p95_latency_ms=%.2f\n"
        "p99_latency_ms=%.2f\n"
        "max_latency_ms=%.2f\n",
        static_cast<unsigned long long>(stats->total_lines),
        static_cast<unsigned long long>(stats->parse_failures),
        static_cast<unsigned long long>(stats->total_bytes),
        static_cast<unsigned long long>(stats->status_2xx),
        static_cast<unsigned long long>(stats->status_3xx),
        static_cast<unsigned long long>(stats->status_4xx),
        static_cast<unsigned long long>(stats->status_5xx),
        stats->avg_latency_ms,
        stats->p95_latency_ms,
        stats->p99_latency_ms,
        stats->max_latency_ms
    );

    if (written < 0 || static_cast<size_t>(written) >= buf_capacity) {
        return -1;
    }

    size_t offset = static_cast<size_t>(written);
    for (size_t i = 0; i < stats->top_ips_len; ++i) {
        int line_len = std::snprintf(
            out_buf + offset,
            buf_capacity - offset,
            "top_ip_%zu=%s:%llu\n",
            i + 1,
            stats->top_ips[i].ip,
            static_cast<unsigned long long>(stats->top_ips[i].count)
        );
        if (line_len < 0 || offset + static_cast<size_t>(line_len) >= buf_capacity) {
            return -1;
        }
        offset += static_cast<size_t>(line_len);
    }

    return static_cast<int>(offset);
}
