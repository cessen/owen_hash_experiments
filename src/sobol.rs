//! An implementation of the Sobol low discrepancy sequence.

use super::hash_u32;

use super::hash_gen::{exec_hash_slice, HashOp};

// The following `include` provides `MAX_DIMENSION` and `VECTORS`.
// See the build.rs file for how this included file is generated.
include!(concat!(env!("OUT_DIR"), "/vectors.inc"));

/// Computes one component of one point from the Sobol sequence.
///
/// `index` specifies the point within the sequence and `dimension` specifies
/// the component of that point.
#[inline]
pub fn sample(index: u32, dimension: u32) -> f32 {
    u32_to_0_1_f32(sobol_u32(index, dimension))
}

/// Same as `sample()` except applies Owen scrambling using a fast hash-based
/// approach.
#[inline]
pub fn sample_owen_fast(index: u32, dimension: u32, seed: u32) -> f32 {
    u32_to_0_1_f32(owen_scramble_fast_u32(sobol_u32(index, dimension), seed))
}

/// Same as `sample_owen_fast()` except it uses a slower "ground-truth"
/// implementation of Owen scrambling.
#[inline]
pub fn sample_owen_reference(index: u32, dimension: u32, seed: u32) -> f32 {
    u32_to_0_1_f32(owen_scramble_reference_u32(
        sobol_u32(index, dimension),
        seed,
    ))
}

//----------------------------------------------------------------------

/// Utility for converting a u32 to a float in [0.0, 1.0).
fn u32_to_0_1_f32(n: u32) -> f32 {
    n as f32 * (1.0 / (1u64 << 32) as f32)
}

/// The actual core Sobol samplng code.  Used by the above functions.
fn sobol_u32(index: u32, dimension: u32) -> u32 {
    assert!(dimension < MAX_DIMENSION);
    let vecs = &VECTORS[dimension as usize];

    let mut index = index;
    let mut result = 0;
    let mut i = 0;
    while index != 0 {
        let j = index.trailing_zeros();
        result ^= vecs[(i + j) as usize];
        i += j + 1;
        index >>= j;
        index >>= 1;
    }

    result
}

/// Scrambles `n` using fast hash-based Owen scrambling.
///
/// Various hashes are included below, and can be uncommented to try them out.
pub fn owen_scramble_fast_u32(x: u32, seed: u32) -> u32 {
    let mut x = x.reverse_bits();

    // Randomize the seed value.
    let seed = hash_u32(seed, 0xa14a177d);

    // // Original Laine-Karras hash.
    // x = x.wrapping_add(seed);
    // x ^= x.wrapping_mul(0x6c50b47c);
    // x ^= x.wrapping_mul(0xb82f1e52);
    // x ^= x.wrapping_mul(0xc7afe638);
    // x ^= x.wrapping_mul(0x8d22f6e6);

    // // "Improved" version 2.  Not actually that good.
    // // From https://psychopath.io/post/2021_01_02_sobol_sampling_take_2
    // x = x.wrapping_add(seed);
    // x ^= 0xdc967795;
    // x = x.wrapping_mul(0x97b756bb);
    // x ^= 0x866350b1;
    // x = x.wrapping_mul(0x9e3779cd);

    // // Fast, reasonable quality.
    // x = x.wrapping_add(x << 2);
    // x ^= x.wrapping_mul(0xfe9b5742);
    // x = x.wrapping_add(seed);
    // x = x.wrapping_mul(seed | 1);

    // // Medium-fast, best quality so far.
    x = x.wrapping_mul(0x788aeeed);
    x ^= x.wrapping_mul(0x41506a02);
    x = x.wrapping_add(seed);
    x = x.wrapping_mul(seed | 1);
    x ^= x.wrapping_mul(0x7483dc64);

    // x = exec_hash_slice(
    //     // Good 2-mul hash.
    //     &[HashOp::ShlAdd(2), HashOp::MulXor(0xfe9b5742), HashOp::Add(0), HashOp::Mul(0), ],

    //     // // Best quality so far.
    //     // &[HashOp::Mul(0x788aeeed), HashOp::MulXor(0x41506a02), HashOp::Add(0), HashOp::Mul(0), HashOp::MulXor(0x7483dc64), ],

    //     x,
    //     seed,
    // );

    x.reverse_bits()
}

/// Same as `owen_scramble_fast_u32()` above, except uses a slower
/// "ground truth" algorithm for Owen scrambling.
pub fn owen_scramble_reference_u32(n: u32, seed: u32) -> u32 {
    // A high-quality, seedable hash function.
    // See https://en.wikipedia.org/wiki/SipHash
    fn siphash(n: u32, seed: u32) -> u32 {
        use std::hash::Hasher;
        let mut hasher = siphasher::sip::SipHasher13::new_with_keys(0, seed as u64);
        hasher.write_u32(n);
        hasher.finish() as u32
    }

    // The Owen scramble.
    let in_bits = n;
    let mut out_bits = n;
    out_bits ^= siphash(0, (seed << 5) | 31) & (1 << 31); // Do highest bit, which isn't handled by the loop.
    for bit in 0..31 {
        let high_mask = !(1u32 << (bit + 1)).wrapping_sub(1);
        let hash = siphash(in_bits & high_mask, (seed << 5) | bit);
        out_bits ^= hash & (1 << bit);
    }

    out_bits
}
