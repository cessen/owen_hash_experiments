#pragma once

#include <cstdint>

extern "C" void genpoints(const char* seq, uint32_t n, uint32_t dim, uint32_t seed, float* x);
