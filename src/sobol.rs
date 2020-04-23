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
        index >>= j + 1;
    }

    (result as u32) << 16
}

/// Scrambles `n` using Owen scrambling and the given scramble parameter.
#[inline(always)]
fn owen_scramble_u32(mut n: u32, scramble: u32) -> u32 {
    // Do Owen scrambling.
    //
    // This uses the technique presented in the paper "Stratified Sampling for
    // Stochastic Transparency" by Laine and Karras.
    // The basic idea is that we're running a special kind of hash function
    // that only allows avalanche to happen downwards (i.e. a bit is only
    // affected by the bits higher than it).  This is achieved by first
    // reversing the bits and then doing mixing via multiplication by even
    // numbers.
    //
    // Normally this would be considered a poor hash function, because normally
    // you want all bits to have an equal chance of affecting all other bits.
    // But in this case that only-downward behavior is exactly what we want,
    // because it ends up being equivalent to Owen scrambling.
    //
    // Note that the application of the scramble parameter here via addition
    // does not invalidate the Owen scramble as long as it is done after the
    // bit the reversal.
    //
    // The permutation constants here were selected through an optimization
    // process to maximize low-bias avalanche between bits.

    n = n.reverse_bits();
    // let perms = [0x6C50B47C, 0xB82F1E52, 0xC7AFE638, 0x8D22F6E];
    let perms = [0x97b756bc, 0x4b0a8a12, 0x75c77e36];
    n = n.wrapping_add(hash_u32(scramble, 0xa14a177d));
    for &p in perms.iter() {
        n ^= n.wrapping_mul(p);
        n += n << 1;
    }
    n = n.reverse_bits();

    // Return the scrambled value.
    n
}

#[inline(always)]
fn u32_to_0_1_f32(n: u32) -> f32 {
    n as f32 * (1.0 / (1u64 << 32) as f32)
}

fn hash_u32(n: u32, seed: u32) -> u32 {
    let mut hash = n;
    for _ in 0..2 {
        hash = hash.wrapping_mul(0x736caf6f);
        hash ^= hash.wrapping_shr(16);
        hash ^= seed;
    }

    hash
}
