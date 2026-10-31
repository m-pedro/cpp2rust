# cpp2rust

A high-performance HTTP access log parser and latency aggregator, incrementally migrated from **legacy C11/C++17** (`mmap` + raw pointer arithmetic + `std::mutex`) to **safe, zero-copy Rust** (`memmap2` + `&[u8]` slices + `rayon` data parallelism) over 15 daily commits.

## Architecture & Migration Highlights

- **Zero-Copy Parsing (`#![forbid(unsafe_code)]`):** Eliminates per-line heap allocations by borrowing `&'a str` fields directly from the memory-mapped byte slice (`LogRecord<'a>`), providing compile-time lifetime guarantees against use-after-free bugs.
- **Incremental FFI Strangler Pattern:** Used an `extern "C"` ABI boundary (`liblegacy.a` linked via `build.rs`) and a deterministic 100,000-line golden snapshot (`expected_100k.txt`) to verify byte-for-byte parity at every step of the migration before retiring the C/C++ bridge on Day 13.
- **Lock-Free Data Parallelism:** Replaced 180+ lines of manual C++ thread-chunking and `std::mutex` merging with a lock-free `Rayon` work-stealing `fold`/`reduce` pipeline and parallel unstable sorting (`par_sort_unstable_by`) for p95/p99 tail latency computation.

---

## Repository Structure

```text
cpp2rust/
├── CMakeLists.txt                # Legacy C11/C++17 build & ASan/UBSan test target
├── Cargo.toml                    # Pure Rust package manifest
├── benches/
│   └── engine_bench.rs           # Criterion.rs throughput & latency benchmarks
├── cpp/                          # Reference C11/C++17 baseline engine
│   ├── include/log_engine.h
│   ├── src/{mmap_reader.c, parser.cpp, aggregator.cpp, main.cpp}
│   └── tests/{test_main.cpp, golden/expected_100k.txt}
├── scripts/
│   └── gen_logs.py               # Deterministic synthetic HTTP access log generator
├── src/
│   ├── lib.rs                    # Pure Rust library exports
│   ├── types.rs                  # Zero-copy LogRecord<'a> & ParseError enum
│   ├── mmap.rs                   # Safe RAII memmap2 file reader
│   ├── parser.rs                 # #[forbid(unsafe_code)] byte-slice tokenizer
│   ├── aggregator.rs             # Lock-free Rayon parallel map-reduce aggregator
│   └── main.rs                   # Pure Rust CLI entry point
└── tests/
    ├── malformed_input_test.rs   # Edge-case, overflow & truncation safety suite
    └── rust_golden_parity_test.rs# End-to-end parity check against C++ golden output
```

---

## Quickstart

### 1. Generate Synthetic HTTP Logs
```bash
python3 scripts/gen_logs.py -n 100000 -o data/access.log
```

### 2. Run the Pure Rust Engine
```bash
cargo run --release -- data/access.log
```

### 3. Run Unit, Malformed-Input, and Golden Parity Tests
```bash
cargo test
```

### 4. Run Criterion Throughput Benchmarks
```bash
cargo bench
```

---

## C++17 vs. Pure Rust Comparison (100,000 Lines / 8.54 MB)

| Metric | Legacy C11/C++17 (`logengine_cpp`) | Pure Rust (`logengine_rs`) |
|---|---|---|
| **Memory Mapping** | Manual POSIX `mmap` / `munmap` (`mmap_reader.c`) | RAII `MappedLogFile` (`memmap2` + `Drop`) |
| **Parser Safety** | Raw `const char*` pointer arithmetic + `strtoul` | `#![forbid(unsafe_code)]` `&'a [u8]` slice combinators |
| **Concurrency Model** | `std::thread` + `std::mutex` critical section | Lock-free `rayon` parallel `map`/`reduce` + `par_sort` |
| **Error Handling** | Magic negative return codes (`-1` .. `-11`) | Strongly typed `Result<LogRecord<'a>, ParseError>` |
| **Golden Output Parity** | Baseline reference (`expected_100k.txt`) | 100% byte-for-byte identical |

---

## 15-Day Incremental Migration Roadmap

- [x] **Day 1:** Initialize CMake build system and deterministic synthetic HTTP log generator (`scripts/gen_logs.py`)
- [x] **Day 2:** Implement POSIX `mmap` file reader (`mmap_reader.c`) and raw pointer log line parser (`parser.cpp`)
- [x] **Day 3:** Add multithreaded status code, top-IP, and p95/p99 latency aggregator (`aggregator.cpp`)
- [x] **Day 4:** Add AddressSanitizer, UndefinedBehaviorSanitizer, and golden parity test output
- [x] **Day 5:** Expose `extern "C"` ABI wrapper headers for incremental Rust FFI calls
- [x] **Day 6:** Initialize Cargo workspace and link `liblegacy.a` via `build.rs`
- [x] **Day 7:** Add Rust FFI bindings (`src/ffi.rs`) and verify output parity against C++ golden output
- [x] **Day 8:** Define idiomatic zero-copy Rust `LogRecord<'a>` struct and `ParseError` enum
- [x] **Day 9:** Replace C `mmap_reader.c` with safe Rust `memmap2` wrapper and RAII `Drop` semantics
- [x] **Day 10:** Port C++ pointer-arithmetic parser to zero-copy Rust `&[u8]` slice combinators
- [x] **Day 11:** Add malformed-input and property-based edge case tests in Rust
- [x] **Day 12:** Replace `std::thread` + `std::mutex` aggregator with lock-free `Rayon` map-reduce
- [x] **Day 13:** Remove legacy C++ FFI bridge and enforce `#[forbid(unsafe_code)]` on parser/aggregator
- [x] **Day 14:** Add `Criterion.rs` benchmarks comparing C++17 vs. Rust throughput and latency
- [x] **Day 15:** Publish C++-to-Rust migration retrospective and benchmark results
