#pragma once

#include <cstdint>

#include "siphash.h"

uint32_t sobol(uint32_t index, uint32_t dim);

void sobol4d(uint32_t index, uint32_t X[4]);

inline uint32_t hash_combine(uint32_t seed, uint32_t v)
{
  return seed ^ (v + (seed << 6) + (seed >> 2));
}

inline uint32_t reverse_bits(uint32_t x)
{
  x = (((x & 0xaaaaaaaa) >> 1) | ((x & 0x55555555) << 1));
  x = (((x & 0xcccccccc) >> 2) | ((x & 0x33333333) << 2));
  x = (((x & 0xf0f0f0f0) >> 4) | ((x & 0x0f0f0f0f) << 4));
  x = (((x & 0xff00ff00) >> 8) | ((x & 0x00ff00ff) << 8));
  return ((x >> 16) | (x << 16));
}


inline uint32_t hash_u32(uint32_t x, uint64_t seed1, uint64_t seed2) {
    uint64_t out;
    uint64_t k[] = {seed1, seed2};

    siphash((uint8_t *)(&x), 4, (uint8_t *)(k), (uint8_t *)(&out), 8);

    return out;
}

//------------------------------------------------------

inline uint32_t nested_uniform_scramble_base2(uint32_t x, uint32_t seed) {
    uint32_t in_bits = x;
    uint32_t out_bits = x;

    // Do the Owen scramble.
    for (uint32_t bit = 0; bit < 31; ++bit) {
        uint32_t high_mask = ~((1 << (bit + 1)) - 1);
        uint32_t hash = hash_u32(in_bits & high_mask, seed, bit);
        out_bits ^= hash & (1 << bit);
    }

    // Flip the highest bit as well, based on the seed.
    out_bits ^= hash_u32(0, seed, 31) & (1 << 31);

    return out_bits;
}


//------------------------------------------------------

inline uint32_t nested_uniform_scramble_base2_original_lk(uint32_t x, uint32_t seed) {
  x = reverse_bits(x);

  x += seed;
  x ^= x * 0x6c50b47cu;
  x ^= x * 0xb82f1e52u;
  x ^= x * 0xc7afe638u;
  x ^= x * 0x8d22f6e6u;

  x = reverse_bits(x);
  return x;
}

inline void shuffled_scrambled_sobol4d_original_lk(uint32_t index, uint32_t seed,
                                       uint32_t X[4])
{
  index = nested_uniform_scramble_base2_original_lk(index, seed);
  sobol4d(index, X);
  for (int i = 0; i < 4; i++) {
    X[i] = nested_uniform_scramble_base2_original_lk(X[i], hash_combine(seed, i));
  }
}

//------------------------------------------------------

inline uint32_t nested_uniform_scramble_base2_v2(uint32_t x, uint32_t seed) {
  x = reverse_bits(x);

  x += seed;
  x ^= 0xdc967795;
  x *= 0x97b756bb;
  x ^= 0x866350b1;
  x *= 0x9e3779cd;

  x = reverse_bits(x);
  return x;
}

inline void shuffled_scrambled_sobol4d_v2(uint32_t index, uint32_t seed,
                                       uint32_t X[4])
{
  index = nested_uniform_scramble_base2_v2(index, seed);
  sobol4d(index, X);
  for (int i = 0; i < 4; i++) {
    X[i] = nested_uniform_scramble_base2_v2(X[i], hash_combine(seed, i));
  }
}

//------------------------------------------------------

inline uint32_t nested_uniform_scramble_base2_5round(uint32_t x, uint32_t seed) {
  x = reverse_bits(x);

  x *= 0x788aeeed;
  x ^= x * 0x41506a02;
  x += seed;
  x *= seed | 1;
  x ^= x * 0x7483dc64;

  x = reverse_bits(x);
  return x;
}

inline void shuffled_scrambled_sobol4d_5round(uint32_t index, uint32_t seed,
                                       uint32_t X[4])
{
  index = nested_uniform_scramble_base2_5round(index, seed);
  sobol4d(index, X);
  for (int i = 0; i < 4; i++) {
    X[i] = nested_uniform_scramble_base2_5round(X[i], hash_combine(seed, i));
  }
}

//------------------------------------------------------

inline uint32_t nested_uniform_scramble_base2_fast(uint32_t x, uint32_t seed) {
  x = reverse_bits(x);

  x += x << 2;
  x ^= x * 0xfe9b5742;
  x += seed;
  x *= seed | 1;

  x = reverse_bits(x);
  return x;
}

inline void shuffled_scrambled_sobol4d_fast(uint32_t index, uint32_t seed,
                                       uint32_t X[4])
{
  index = nested_uniform_scramble_base2_fast(index, seed);
  sobol4d(index, X);
  for (int i = 0; i < 4; i++) {
    X[i] = nested_uniform_scramble_base2_fast(X[i], hash_combine(seed, i));
  }
}


