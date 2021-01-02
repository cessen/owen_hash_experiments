//! An implementation of the Sobol low discrepancy sequence.
//!
//! Includes variants with random digit scrambling, Cranley-Patterson rotation,
//! and Owen scrambling.

// The following `include` provides `MAX_DIMENSION` and `VECTORS`.
// See the build.rs file for how this included file is generated.
include!(concat!(env!("OUT_DIR"), "/vectors.inc"));

/// Compute one component of one sample from the Sobol'-sequence, where
/// `dimension` specifies the component and `index` specifies the sample
/// within the sequence.
#[inline]
pub fn sample(dimension: u32, index: u32) -> f32 {
    u32_to_0_1_f32(sobol_u32(dimension, index))
}

/// Same as `sample()` except applies Owen scrambling using the given scramble
/// parameter.
///
/// To get proper Owen scrambling, you need to use a different scramble
/// value for each dimension, and those values should be generated more-or-less
/// randomly.  For example, using a 32-bit hash of the dimension parameter
/// works well.
#[inline]
pub fn sample_owen(dimension: u32, index: u32, scramble: u32) -> f32 {
    u32_to_0_1_f32(owen_scramble_u32(sobol_u32(dimension, index), scramble))
}

//----------------------------------------------------------------------

/// The actual core Sobol samplng code.  Used by the other functions.
#[inline(always)]
fn sobol_u32(dimension: u32, index: u32) -> u32 {
    assert!(dimension < MAX_DIMENSION);
    let vecs = &VECTORS[dimension as usize];
    let mut index = index as u16;

    let mut result = 0;
    let mut i = 0;
    while index != 0 {
        let j = index.trailing_zeros();
        result ^= vecs[(i + j) as usize];
        i += j + 1;
        index >>= j;
        index >>= 1;
    }

    (result as u32) << 16
}

/// Scrambles `n` using Owen scrambling and the given scramble parameter.
#[inline(always)]
pub fn owen_scramble_u32(mut n: u32, scramble: u32) -> u32 {
    // This uses the technique presented in the paper "Stratified Sampling for
    // Stochastic Transparency" by Laine and Karras.

    n = n.reverse_bits();

    let scramble = hash_u32(scramble, 0xa14a177d);

    // // LK version
    // n = n.wrapping_add(scramble);
    // n ^= n.wrapping_mul(0x6c50b47c);
    // n ^= n.wrapping_mul(0xb82f1e52);
    // n ^= n.wrapping_mul(0xc7afe638);
    // n ^= n.wrapping_mul(0x8d22f6e6);

    // // Improved version 1
    // n = n.wrapping_add(scramble);
    // n ^= n.wrapping_mul(0x6c50b47c);
    // n *= 3;
    // n ^= n.wrapping_mul(0xb82f1e52);
    // n *= 3;
    // n ^= n.wrapping_mul(0xc7afe638);
    // n *= 3;
    // n ^= n.wrapping_mul(0x8d22f6e6);
    // n *= 3;

    // Improved version 2
    n = n.wrapping_add(scramble);
    n ^= 0xdc967795;
    n = n.wrapping_mul(0x97b756bb);
    n ^= 0x866350b1;
    n = n.wrapping_mul(0x9e3779cd);

    n = n.reverse_bits();

    // Return the scrambled value.
    n
}

/// Same as `lk_scramble()` except uses a slower more full version of
/// hashing.
///
/// This is mainly intended to help validate the faster scrambling function,
/// and likely shouldn't be used for real things.  It is significantly
/// slower.
#[allow(dead_code)]
#[inline]
fn owen_scramble_slow(mut n: u32, scramble: u32) -> u32 {
    n = n.reverse_bits();
    n = n.wrapping_add(hash_u32(scramble, 0));
    for i in 0..31 {
        let low_mask = (1u32 << i).wrapping_sub(1);
        let low_bits_hash = hash_u32((n & low_mask) ^ hash_u32(i, 0), 0);
        n ^= low_bits_hash & !low_mask;
    }
    n.reverse_bits()
}

#[inline(always)]
fn u32_to_0_1_f32(n: u32) -> f32 {
    n as f32 * (1.0 / (1u64 << 32) as f32)
}

fn hash_u32(n: u32, seed: u32) -> u32 {
    let mut hash = n;
    for _ in 0..5 {
        hash = hash.wrapping_mul(0x736caf6f);
        hash ^= hash.wrapping_shr(16);
        hash ^= seed;
    }

    hash
}
