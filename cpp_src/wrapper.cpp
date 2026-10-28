#include <fstream>
#include "legacy_engine.h" // Original Day 1-4 code

extern "C" {
    // A C-compatible wrapper function that Rust can call safely
    int generate_data_cpp(const char* output_path, size_t count) {
        try {
            LegacyEngine engine;
            engine.generate_to_file(output_path, count);
            return 0; // Success
        } catch (...) {
            return -1; // Error
        }
    }
}
