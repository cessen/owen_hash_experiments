#include <iostream>
#include <string>
#include "genpoints.h"
#include "sobol.h"
#include "faure05.h"
#include "owenhash.h"
#include "pcg.h"

#include <glm/vec2.hpp>
#include <glm/vec3.hpp>
#include <glm/glm.hpp>
using glm::vec2;
using glm::vec3;

uint32_t hash(uint32_t x)
{
    // finalizer from murmurhash3
    x ^= x >> 16;
    x *= 0x85ebca6bu;
    x ^= x >> 13;
    x *= 0xc2b2ae35u;
    x ^= x >> 16;
    return x;
}

extern "C" void genpoints(const char* seqname, uint32_t n, uint32_t dim, uint32_t seed, float* x)
{
    constexpr float S = float(1.0/(1ul<<32));
    seed = hash(seed);

    std::string seq(seqname);
    if (seq == "random") {
        pcg32_random_t rng;
        pcg32_srandom_r(&rng, hash_combine(seed, dim), 0);
        for (uint32_t i = 0; i < n; i++)
            x[i] = pcg32_random_r(&rng) * S;
    }
    else if (seq == "faure05") {
        std::vector<int> digits;
        digits.resize(13);
        for (uint32_t i = 0; i < n; i++) {
            extractDigits(i, 5, digits);
            faure05(dim, digits);
            x[i] = radicalInverse(5, digits);
        }
    }
    else if (seq == "sobol") {
        for (uint32_t i = 0; i < n; i++)
            x[i] = (sobol(i,dim)) * S;
    }
    else if (seq == "sobol_rds") {
        seed = hash_combine(seed, hash(dim));
        for (uint32_t i = 0; i < n; i++)
            x[i] = (sobol(i,dim) ^ seed) * S;
    }
    else if (seq == "sobol_owen") {
        for (uint32_t i = 0; i < n; i++) {
            uint32_t index = nested_uniform_scramble_base2(i, seed);
            x[i] = nested_uniform_scramble_base2(sobol(index, dim), hash_combine(seed, dim)) * S;
        }
    }
    else if (seq == "sobol_owen_hash_lk") {
        for (uint32_t i = 0; i < n; i++) {
            uint32_t index = nested_uniform_scramble_base2_original_lk(i, seed);
            x[i] = nested_uniform_scramble_base2_original_lk(sobol(index, dim), hash_combine(seed, dim)) * S;
        }
    }
    else if (seq == "sobol_owen_hash_v2") {
        for (uint32_t i = 0; i < n; i++) {
            uint32_t index = nested_uniform_scramble_base2_v2(i, seed);
            x[i] = nested_uniform_scramble_base2_v2(sobol(index, dim), hash_combine(seed, dim)) * S;
        }
    }
    else if (seq == "sobol_owen_hash_fast") {
        for (uint32_t i = 0; i < n; i++) {
            uint32_t index = nested_uniform_scramble_base2_fast(i, seed);
            x[i] = nested_uniform_scramble_base2_fast(sobol(index, dim), hash_combine(seed, dim)) * S;
        }
    }
    else if (seq == "sobol_owen_hash_good") {
        for (uint32_t i = 0; i < n; i++) {
            uint32_t index = nested_uniform_scramble_base2_5round(i, seed);
            x[i] = nested_uniform_scramble_base2_5round(sobol(index, dim), hash_combine(seed, dim)) * S;
        }
    }
    else {
        std::cerr << "unknown sequence: " << seq << "\n";
        abort();
    }
}
