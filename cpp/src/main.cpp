#include "log_engine.h"

#include <chrono>
#include <cstring>
#include <iostream>

int main(int argc, char** argv) {
    if (argc < 2) {
        std::cerr << "Usage: " << argv[0] << " <path/to/access.log>\n";
        return 1;
    }

    MappedFile file{};
    if (mmap_open(argv[1], &file) != 0) {
        return 1;
    }

    auto start_time = std::chrono::high_resolution_clock::now();

    const char* cur = file.data;
    const char* end = file.data + file.size;

    uint64_t total_lines = 0;
    uint64_t error_5xx_count = 0;
    uint64_t parse_failures = 0;
    double total_latency_ms = 0.0;

    while (cur < end) {
        const char* newline = static_cast<const char*>(
            std::memchr(cur, '\n', static_cast<size_t>(end - cur))
        );
        const char* line_end = newline ? newline : end;
        size_t line_len = static_cast<size_t>(line_end - cur);

        if (line_len > 0) {
            LogRecord rec{};
            if (parse_log_line(cur, line_len, &rec) == 0) {
                ++total_lines;
                total_latency_ms += rec.latency_ms;
                if (rec.status_code >= 500) {
                    ++error_5xx_count;
                }
            } else {
                ++parse_failures;
            }
        }
        cur = newline ? (newline + 1) : end;
    }

    auto end_time = std::chrono::high_resolution_clock::now();
    double elapsed_ms = std::chrono::duration<double, std::milli>(end_time - start_time).count();

    std::cout << "=== C++ LogEngine Baseline (Day 2) ===\n"
              << "File Size      : " << (file.size / (1024.0 * 1024.0)) << " MB\n"
              << "Parsed Lines   : " << total_lines << "\n"
              << "Parse Failures : " << parse_failures << "\n"
              << "5xx Responses  : " << error_5xx_count << "\n"
              << "Avg Latency    : " << (total_lines ? total_latency_ms / total_lines : 0.0) << " ms\n"
              << "Elapsed Time   : " << elapsed_ms << " ms\n";

    mmap_close(&file);
    return 0;
}
