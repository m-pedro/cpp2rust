#include "log_engine.h"

#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <iostream>
#include <thread>

void print_golden_report(const AggregatedStats& s) {
    std::printf("total_lines=%llu\n", static_cast<unsigned long long>(s.total_lines));
    std::printf("parse_failures=%llu\n", static_cast<unsigned long long>(s.parse_failures));
    std::printf("total_bytes=%llu\n", static_cast<unsigned long long>(s.total_bytes));
    std::printf("status_2xx=%llu\n", static_cast<unsigned long long>(s.status_2xx));
    std::printf("status_3xx=%llu\n", static_cast<unsigned long long>(s.status_3xx));
    std::printf("status_4xx=%llu\n", static_cast<unsigned long long>(s.status_4xx));
    std::printf("status_5xx=%llu\n", static_cast<unsigned long long>(s.status_5xx));
    std::printf("avg_latency_ms=%.4f\n", s.avg_latency_ms);
    std::printf("p95_latency_ms=%.2f\n", s.p95_latency_ms);
    std::printf("p99_latency_ms=%.2f\n", s.p99_latency_ms);
    std::printf("max_latency_ms=%.2f\n", s.max_latency_ms);
    for (size_t i = 0; i < s.top_ips_len; ++i) {
        std::printf(
            "top_ip_%zu=%s:%llu\n",
            i + 1,
            s.top_ips[i].ip,
            static_cast<unsigned long long>(s.top_ips[i].count)
        );
    }
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::cerr << "Usage: " << argv[0] << " <path/to/access.log> [-t threads] [--golden]\n";
        return 1;
    }

    const char* filepath = argv[1];
    size_t threads = std::thread::hardware_concurrency();
    if (threads == 0) threads = 4;
    bool golden_mode = false;

    for (int i = 2; i < argc; ++i) {
        if (std::strcmp(argv[i], "--golden") == 0) {
            golden_mode = true;
        } else if (std::strcmp(argv[i], "-t") == 0 && i + 1 < argc) {
            threads = static_cast<size_t>(std::strtoul(argv[++i], nullptr, 10));
        }
    }

    MappedFile file{};
    if (mmap_open(filepath, &file) != 0) {
        return 1;
    }

    auto start_time = std::chrono::high_resolution_clock::now();

    AggregatedStats stats{};
    if (aggregate_buffer_mt(file.data, file.size, threads, &stats) != 0) {
        mmap_close(&file);
        return 1;
    }

    auto end_time = std::chrono::high_resolution_clock::now();
    double elapsed_ms = std::chrono::duration<double, std::milli>(end_time - start_time).count();

    if (golden_mode) {
        print_golden_report(stats);
    } else {
        std::printf("=== C++ Multithreaded LogEngine (Day 3) ===\n");
        std::printf("Worker Threads : %zu\n", threads);
        std::printf("File Size      : %.2f MB\n", file.size / (1024.0 * 1024.0));
        std::printf("Parsed Lines   : %llu (Failures: %llu)\n",
                    static_cast<unsigned long long>(stats.total_lines),
                    static_cast<unsigned long long>(stats.parse_failures));
        std::printf("Total Bytes    : %llu\n", static_cast<unsigned long long>(stats.total_bytes));
        std::printf("Status Codes   : 2xx=%llu | 3xx=%llu | 4xx=%llu | 5xx=%llu\n",
                    static_cast<unsigned long long>(stats.status_2xx),
                    static_cast<unsigned long long>(stats.status_3xx),
                    static_cast<unsigned long long>(stats.status_4xx),
                    static_cast<unsigned long long>(stats.status_5xx));
        std::printf("Latency (ms)   : Avg=%.2f | p95=%.2f | p99=%.2f | Max=%.2f\n",
                    stats.avg_latency_ms, stats.p95_latency_ms,
                    stats.p99_latency_ms, stats.max_latency_ms);
        std::printf("Top %zu IPs      :\n", stats.top_ips_len);
        for (size_t i = 0; i < stats.top_ips_len; ++i) {
            std::printf("  #%zu %-15s (%llu reqs)\n",
                        i + 1, stats.top_ips[i].ip,
                        static_cast<unsigned long long>(stats.top_ips[i].count));
        }
        std::printf("Elapsed Time   : %.2f ms\n", elapsed_ms);
    }

    mmap_close(&file);
    return 0;
}
