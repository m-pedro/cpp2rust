#include "log_engine.h"

#include <cstdlib>
#include <cstring>

namespace {

inline const char* find_char(const char* cur, const char* end, char target) {
    while (cur < end && *cur != target) {
        ++cur;
    }
    return cur;
}

inline const char* skip_spaces(const char* cur, const char* end) {
    while (cur < end && *cur == ' ') {
        ++cur;
    }
    return cur;
}

bool parse_uint64(const char*& cur, const char* end, uint64_t& out_val) {
    cur = skip_spaces(cur, end);
    if (cur >= end || *cur < '0' || *cur > '9') {
        return false;
    }
    uint64_t val = 0;
    while (cur < end && *cur >= '0' && *cur <= '9') {
        val = (val * 10) + static_cast<uint64_t>(*cur - '0');
        ++cur;
    }
    out_val = val;
    return true;
}

bool parse_double(const char*& cur, const char* end, double& out_val) {
    cur = skip_spaces(cur, end);
    if (cur >= end) {
        return false;
    }
    /* Copy numeric token into stack buffer so strtod has a guaranteed null terminator */
    char buf[32];
    size_t len = 0;
    while (cur < end && *cur != ' ' && *cur != '\n' && *cur != '\r' && len < sizeof(buf) - 1) {
        buf[len++] = *cur++;
    }
    buf[len] = '\0';
    if (len == 0) {
        return false;
    }
    char* parse_end = nullptr;
    out_val = std::strtod(buf, &parse_end);
    return parse_end != buf;
}

} // namespace

extern "C" int parse_log_line(const char* line_start, size_t line_len, LogRecord* out_record) {
    if (!line_start || line_len == 0 || !out_record) {
        return -1;
    }

    const char* cur = line_start;
    const char* end = line_start + line_len;

    // Trim trailing '\r' if present
    if (cur < end && *(end - 1) == '\r') {
        --end;
    }

    // 1. Parse IP address (up to first space)
    const char* ip_end = find_char(cur, end, ' ');
    if (ip_end == cur || ip_end >= end) return -1;
    out_record->ip = {cur, static_cast<size_t>(ip_end - cur)};

    // 2. Advance to '[' for timestamp
    const char* ts_open = find_char(ip_end, end, '[');
    if (ts_open >= end) return -2;
    ++ts_open; // skip '['
    const char* ts_close = find_char(ts_open, end, ']');
    if (ts_close >= end) return -2;
    out_record->timestamp = {ts_open, static_cast<size_t>(ts_close - ts_open)};

    // 3. Advance to opening '"' of HTTP request: "METHOD /path HTTP/1.1"
    const char* req_open = find_char(ts_close + 1, end, '"');
    if (req_open >= end) return -3;
    cur = req_open + 1;

    const char* method_end = find_char(cur, end, ' ');
    if (method_end >= end) return -3;
    out_record->method = {cur, static_cast<size_t>(method_end - cur)};

    cur = method_end + 1;
    const char* path_end = find_char(cur, end, ' ');
    if (path_end >= end) return -3;
    out_record->path = {cur, static_cast<size_t>(path_end - cur)};

    const char* req_close = find_char(path_end, end, '"');
    if (req_close >= end) return -3;
    cur = req_close + 1;

    // 4. Parse HTTP Status Code
    uint64_t status_u64 = 0;
    if (!parse_uint64(cur, end, status_u64) || status_u64 > 999) {
        return -4;
    }
    out_record->status_code = static_cast<uint16_t>(status_u64);

    // 5. Parse Bytes Sent
    if (!parse_uint64(cur, end, out_record->bytes_sent)) {
        return -5;
    }

    // 6. Parse Latency (ms)
    if (!parse_double(cur, end, out_record->latency_ms)) {
        return -6;
    }

    return 0;
}
