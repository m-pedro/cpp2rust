#ifndef LOG_ENGINE_H
#define LOG_ENGINE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define MAX_TOP_IPS 5

/* Non-owning zero-copy slice (maps directly to Rust's &[u8] / &str) */
typedef struct {
    const char* ptr;
    size_t len;
} StringView;

/* Memory-mapped file handle managed by mmap_reader.c */
typedef struct {
    const char* data;
    size_t size;
    int fd;
} MappedFile;

/* Parsed representation of a single HTTP access log line */
typedef struct {
    StringView ip;
    StringView timestamp;
    StringView method;
    StringView path;
    uint16_t status_code;
    uint64_t bytes_sent;
    double latency_ms;
} LogRecord;

/* Top IP entry with fixed-size null-terminated C string for FFI safety */
typedef struct {
    char ip[40];
    uint64_t count;
} IpCount;

/* Complete aggregated summary across all worker threads */
typedef struct {
    uint64_t total_lines;
    uint64_t parse_failures;
    uint64_t total_bytes;
    uint64_t status_2xx;
    uint64_t status_3xx;
    uint64_t status_4xx;
    uint64_t status_5xx;
    double avg_latency_ms;
    double p95_latency_ms;
    double p99_latency_ms;
    double max_latency_ms;
    size_t top_ips_len;
    IpCount top_ips[MAX_TOP_IPS];
} AggregatedStats;

/* Day 2: C POSIX mmap file lifecycle */
int mmap_open(const char* filepath, MappedFile* out_file);
void mmap_close(MappedFile* file);

/* Day 2: C++ zero-copy pointer-arithmetic line parser */
int parse_log_line(const char* line_start, size_t line_len, LogRecord* out_record);

/* Day 3: Multithreaded buffer aggregator (num_threads = 0 uses hardware_concurrency) */
int aggregate_buffer_mt(
    const char* data,
    size_t size,
    size_t num_threads,
    AggregatedStats* out_stats
);

#ifdef __cplusplus
}
#endif

#endif /* LOG_ENGINE_H */
