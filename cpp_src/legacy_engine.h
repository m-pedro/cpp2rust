#ifndef LEGACY_ENGINE_H
#define LEGACY_ENGINE_H
#include <cstddef>

class LegacyEngine {
public:
    LegacyEngine();
    void generate_to_file(const char* output_path, size_t count);
};

#endif
