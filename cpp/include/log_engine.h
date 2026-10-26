#ifndef LOG_ENGINE_H
#define LOG_ENGINE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

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

/* C POSIX mmap file lifecycle */
int mmap_open(const char* filepath, MappedFile* out_file);
void mmap_close(MappedFile* file);

/* C++ zero-copy pointer-arithmetic line parser (returns 0 on success, <0 on error) */
int parse_log_line(const char* line_start, size_t line_len, LogRecord* out_record);

#ifdef __cplusplus
}
#endif

#endif /* LOG_ENGINE_H */
