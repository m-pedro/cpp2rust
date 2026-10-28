#include "log_engine.h"

#include <cassert>
#include <cmath>
#include <cstring>
#include <iostream>
#include <string>

namespace {

bool approx_eq(double a, double b, double eps = 1e-4) {
    return std::fabs(a - b) < eps;
}

void test_single_line_parser() {
    const char* line =
        "192.168.1.10 - - [25/Oct/2026:00:00:01 +0000] \"POST /api/v1/checkout HTTP/1.1\" 201 2048 15.75";
    LogRecord rec{};
    int rc = parse_log_line(line, std::strlen(line), &rec);
    assert(rc == 0);
    assert(std::string(rec.ip.ptr, rec.ip.len) == "192.168.1.10");
    assert(std::string(rec.timestamp.ptr, rec.timestamp.len) == "25/Oct/2026:00:00:01 +0000");
    assert(std::string(rec.method.ptr, rec.method.len) == "POST");
    assert(std::string(rec.path.ptr, rec.path.len) == "/api/v1/checkout");
    assert(rec.status_code == 201);
    assert(rec.bytes_sent == 2048);
    assert(approx_eq(rec.latency_ms, 15.75));
    std::cout << "[PASS] test_single_line_parser\n";
}

void test_malformed_lines_do_not_overflow_buffer() {
    // Under AddressSanitizer, any out-of-bounds read here will immediately abort
    const char* bad_inputs[] = {
        "",
        "192.168.1.1",
        "192.168.1.1 - - [unclosed_timestamp",
        "192.168.1.1 - - [25/Oct/2026:00:00:01 +0000] \"GET /only_two_parts\"",
        "192.168.1.1 - - [25/Oct/2026:00:00:01 +0000] \"GET / health\" 9999 100 1.0",
        "192.168.1.1 - - [25/Oct/2026:00:00:01 +0000] \"GET /api HTTP/1.1\" 200 abc 1.0",
    };

    for (const char* bad : bad_inputs) {
        LogRecord rec{};
        int rc = parse_log_line(bad, std::strlen(bad), &rec);
        assert(rc < 0 && "Expected malformed input to return negative error code");
    }
    std::cout << "[PASS] test_malformed_lines_do_not_overflow_buffer (ASan verified)\n";
}

void test_multithreaded_determinism_and_parity() {
    const std::string sample_log =
        "10.0.0.1 - - [25/Oct/2026:00:00:01 +0000] \"GET /api/v1/users HTTP/1.1\" 200 500 10.00\n"
        "10.0.0.2 - - [25/Oct/2026:00:00:02 +0000] \"POST /api/v1/orders HTTP/1.1\" 201 1000 20.00\n"
        "10.0.0.1 - - [25/Oct/2026:00:00:03 +0000] \"GET /static/app.js HTTP/1.1\" 304 0 5.00\n"
        "10.0.0.3 - - [25/Oct/2026:00:00:04 +0000] \"GET /login HTTP/1.1\" 404 128 15.00\n"
        "10.0.0.1 - - [25/Oct/2026:00:00:05 +0000] \"POST /api/v1/checkout HTTP/1.1\" 500 64 200.00\n"
        "corrupted_line_that_should_be_counted_as_failure\n";

    AggregatedStats single_thread{};
    AggregatedStats multi_thread{};

    assert(aggregate_buffer_mt(sample_log.data(), sample_log.size(), 1, &single_thread) == 0);
    assert(aggregate_buffer_mt(sample_log.data(), sample_log.size(), 4, &multi_thread) == 0);

    // Verify exact parity between 1 thread and 4 threads
    assert(single_thread.total_lines == 5);
    assert(multi_thread.total_lines == 5);
    assert(single_thread.parse_failures == 1);
    assert(multi_thread.parse_failures == 1);
    assert(single_thread.total_bytes == 1692);
    assert(multi_thread.total_bytes == 1692);

    assert(multi_thread.status_2xx == 2);
    assert(multi_thread.status_3xx == 1);
    assert(multi_thread.status_4xx == 1);
    assert(multi_thread.status_5xx == 1);

    assert(approx_eq(multi_thread.avg_latency_ms, 50.0));
    assert(approx_eq(multi_thread.max_latency_ms, 200.0));
    assert( std::strcmp(multi_thread.top_ips[0].ip, "10.0.0.1") == 0 );
    assert(multi_thread.top_ips[0].count == 3);

    std::cout << "[PASS] test_multithreaded_determinism_and_parity (1 thread == 4 threads)\n";
}

} // namespace

int main() {
    std::cout << "=== Running Day 4 C++ ASan/UBSan Test Suite ===\n";
    test_single_line_parser();
    test_malformed_lines_do_not_overflow_buffer();
    test_multithreaded_determinism_and_parity();
    std::cout << "All tests passed!\n";
    return 0;
}
