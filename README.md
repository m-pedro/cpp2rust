# cpp2rust
C++ to Rust translation project

A high-performance HTTP access log parser and latency aggregator, incrementally migrated from **legacy C11/C++17** (`mmap` + raw pointer arithmetic) to **safe, zero-copy Rust** (`memmap2` + `&[u8]` slices + `rayon` data parallelism) over 15 daily commits.

## Architecture & Migration Goals

- **Zero-Copy Parsing:** Avoids per-line heap allocations (`std::string` / `String`) by operating directly on memory-mapped byte slices (`StringView` in C/C++ $\rightarrow$ `&'a [u8]` in Rust).
- **Incremental FFI Strangler Pattern:** Uses a `CMake` static library (`liblegacy.a`) linked into Cargo via `build.rs` and `extern "C"` ABI headers so C++ modules can be replaced one by one while keeping the integration test suite green.
- **Memory & Thread Safety:** Eliminates manual `munmap` lifecycle bugs, raw pointer buffer overreads, and `std::mutex` lock contention using Rust's ownership model and `#[forbid(unsafe_code)]` in the final parser.

---

## Repository Structure

```text
cpp2rust-logengine/
├── CMakeLists.txt           # Builds legacy C11/C++17 library & CLI
├── README.md                # Project documentation & migration roadmap
├── scripts/
│   └── gen_logs.py          # Deterministic synthetic HTTP access log generator
└── cpp/
    ├── include/
    │   └── log_engine.h     # Shared C ABI structs (StringView, LogRecord)
    └── src/
        ├── mmap_reader.c    # POSIX mmap/madvise file reader in C11
        ├── parser.cpp       # Zero-copy pointer-arithmetic tokenizer in C++17
        └── main.cpp         # C++ baseline CLI entry point
```

---

## Quickstart (Days 1 & 2 Baseline)

### 1. Generate Synthetic HTTP Logs
Generate 100,000 deterministic Nginx/Apache log records (~9.5 MB) with realistic status code distributions and tail-latency spikes:

```bash
python3 scripts/gen_logs.py -n 100000 -o data/access.log
```

### 2. Build the C/C++ Baseline Engine
Requires a C11/C++17 compiler (`clang` or `gcc`) and `cmake` (>= 3.16):

```bash
cmake -B build
cmake --build build
```

To build with **AddressSanitizer (ASan)** and **UndefinedBehaviorSanitizer (UBSan)** enabled:

```bash
cmake -B build -DENABLE_SANITIZERS=ON
cmake --build build
```

### 3. Run the Log Parser
```bash
./build/logengine_cpp data/access.log
```

**Sample Output:**
```text
=== C++ LogEngine Baseline (Day 2) ===
File Size      : 9.42 MB
Parsed Lines   : 100000
Parse Failures : 0
5xx Responses  : 1984
Avg Latency    : 39.81 ms
Elapsed Time   : 8.40 ms
```

---

## 15-Day Incremental Migration Roadmap

- [x] **Day 1:** Initialize CMake build system and deterministic synthetic HTTP log generator (`scripts/gen_logs.py`)
- [x] **Day 2:** Implement POSIX `mmap` file reader (`mmap_reader.c`) and raw pointer log line parser (`parser.cpp`)
- [ ] **Day 3:** Add multithreaded status code, top-IP, and p95/p99 latency aggregator (`aggregator.cpp`)
- [ ] **Day 4:** Add AddressSanitizer, UndefinedBehaviorSanitizer, and golden parity test output
- [ ] **Day 5:** Expose `extern "C"` ABI wrapper headers for incremental Rust FFI calls
- [ ] **Day 6:** Initialize Cargo workspace and link `liblegacy.a` via `build.rs`
- [ ] **Day 7:** Add Rust FFI bindings (`src/ffi.rs`) and verify output parity against C++ golden output
- [ ] **Day 8:** Define idiomatic zero-copy Rust `LogRecord<'a>` struct and `ParseError` enum
- [ ] **Day 9:** Replace C `mmap_reader.c` with safe Rust `memmap2` wrapper and RAII `Drop` semantics
- [ ] **Day 10:** Port C++ pointer-arithmetic parser to zero-copy Rust `&[u8]` slice combinators
- [ ] **Day 11:** Add malformed-input and property-based edge case tests in Rust
- [ ] **Day 12:** Replace `std::thread` + `std::mutex` aggregator with lock-free `Rayon` map-reduce
- [ ] **Day 13:** Remove legacy C++ FFI bridge and enforce `#[forbid(unsafe_code)]` on parser/aggregator
- [ ] **Day 14:** Add `Criterion.rs` benchmarks comparing C++17 vs. Rust throughput and peak RSS memory
- [ ] **Day 15:** Publish C++-to-Rust migration retrospective and benchmark results
