#ifndef C_API_H
#define C_API_H

#include "log_engine.h"
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/*
 * High-level FFI entry point: memory-maps `filepath`, runs the multithreaded
 * C++ aggregator across `num_threads`, populates `out_stats`, and unmaps the file.
 * Returns 0 on success, negative error code on failure.
 */
int c_api_analyze_file(
    const char* filepath,
    size_t num_threads,
    AggregatedStats* out_stats
);

/*
 * Formats `stats` into deterministic key=value golden text inside `out_buf`.
 * Returns number of bytes written (excluding null terminator), or -1 on error.
 */
int c_api_format_golden(
    const AggregatedStats* stats,
    char* out_buf,
    size_t buf_capacity
);

#ifdef __cplusplus
}
#endif

#endif /* C_API_H */
