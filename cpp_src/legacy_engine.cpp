#include "legacy_engine.h"
#include <fstream>

LegacyEngine::LegacyEngine() {}

void LegacyEngine::generate_to_file(const char* output_path, size_t count) {
    std::ofstream out(output_path);
    for (size_t i = 0; i < count; ++i) {
        // You will need to update this string to match your exact expected_100k.txt format later
        out << "Data row " << i << "\n"; 
    }
}
