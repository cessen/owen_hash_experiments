//! An implementation of the Sobol low discrepancy sequence.
//!
//! Includes variants with random digit scrambling, Cranley-Patterson rotation,
//! and Owen scrambling.

use super::hash_u32;

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

/// Same as `sample_owen()` except it uses a slower, full, ground-truth
/// implementation of Owen scrambling.
#[inline]
pub fn sample_owen_slow(dimension: u32, index: u32, scramble: u32) -> f32 {
    u32_to_0_1_f32(owen_scramble_slow(sobol_u32(dimension, index), scramble))
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

    // LK version
    // n = n.wrapping_add(scramble);
    // n ^= n.wrapping_mul(0x6c50b47c);
    // n ^= n.wrapping_mul(0xb82f1e52);
    // n ^= n.wrapping_mul(0xc7afe638);
    // n ^= n.wrapping_mul(0x8d22f6e6);

    // // LK rounds
    // n = n.wrapping_add(scramble);
    // for i in 0..64 {
    //     n ^= n.wrapping_mul(RAND_INTS[i] << 1);
    // }

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

    // // Improved version 2
    // n = n.wrapping_add(scramble);
    // n ^= 0xdc967795;
    // n = n.wrapping_mul(0x97b756bb);
    // n ^= 0x866350b1;
    // n = n.wrapping_mul(0x9e3779cd);

    // // Improved version 3
    // n = n.wrapping_add(scramble);
    // for p in RAND_INTS.chunks(2).take(2) {
    //     n = n.wrapping_mul(p[0] | 1);
    //     n ^= p[1];
    // }

    // // Improved version 4
    // // This version is designed to minimize bias at all costs, which
    // // isn't actually the behavior of a full per-bit hash.  However,
    // // it is very fast and probably great for the typical use-cases of
    // // Owen scrambling.  It only really needs one round, but the
    // // additional constants are provided for the paranoid.
    // let perms: &[(u32, u32)] = &[
    //    // Optimized constants.
    //     (0xa2d0f65a, 0x22bbe06d),
    //     (0xeb8e0374, 0x0c8c8841),
    //     (0xed3a0b98, 0xd1f0ca7b),
    // ];
    // n = n.wrapping_add(scramble);
    // for &(p1, p2) in perms.iter().take(1) {
    //     n ^= n.wrapping_mul(p1);
    //     n = n.wrapping_mul(p2);
    // }

    // Improved version 5
    // This version is designed to match the behavior of a full per-bit
    // hash.  It needs two rounds to get reasonably close, and the third
    // round brings it very close.
    let perms: &[(u32, u32)] = &[
        // Optimized constants.
        (0xfadfb1ea, 0x410237b9),
        (0x12889fc2, 0xc3708fa3),
        (0x94951132, 0x8f39c67f),
    ];
    let seed1 = hash_u32(scramble, 0);
    let seed2 = hash_u32(scramble, 1);
    n = n.wrapping_mul(seed1 | 1);
    for &(p1, p2) in perms.iter().take(3) {
        n = n.wrapping_add(seed2);
        n ^= n.wrapping_mul(p1);
        n = n.wrapping_mul(p2);
    }

    // // Add Xor version
    // n = n.wrapping_add(scramble);
    // for p in RAND_INTS.chunks(2).cycle().take(100) {
    //     n = n.wrapping_add(p[0]);
    //     n ^= p[1];
    // }

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
pub fn owen_scramble_slow(n: u32, scramble: u32) -> u32 {
    let seed = hash_u32(scramble, 0);
    let in_bits = n;
    let mut out_bits = n;

    // Do the Owen scramble.
    for bit in 0..31 {
        let high_mask = !(1u32 << (bit + 1)).wrapping_sub(1);
        let hash = hash_u32(in_bits & high_mask, (seed << 5) | bit);
        out_bits ^= hash & (1 << bit);
    }

    // Flip the highest bit as well, based on the seed.
    out_bits ^= hash_u32(0, (seed << 5) | 31) & (1 << 31);

    out_bits
}

#[inline(always)]
fn u32_to_0_1_f32(n: u32) -> f32 {
    n as f32 * (1.0 / (1u64 << 32) as f32)
}

pub const RAND_INTS: &[u32] = &[
    0x0d583be9, 0xc515155f, 0xf313ebeb, 0x35546639, 0x584fd9fd, 0x52668d72, 0xb94e8e47, 0x8af11bf0,
    0x86756b14, 0xb6852a95, 0x1448c74a, 0xc95a2bc3, 0x0f3485ba, 0x52bbfcbf, 0xbf67ac4e, 0x34502d35,
    0xed3c2a97, 0x56c38d17, 0xda0b1f46, 0xde735f59, 0xbfb36a9b, 0x8192580f, 0x53255152, 0xa1372fe4,
    0x6fecce0d, 0xa733f9a2, 0x85714709, 0xc966fd6e, 0x7bfab5c0, 0xc7ed7fdd, 0x4e1efe70, 0x47690356,
    0x4904cc9c, 0xfba8227e, 0xe6689eec, 0x9ccfc9be, 0x2f87ed7a, 0x03984077, 0x0ab63301, 0x92b45b26,
    0x1b3ac43c, 0xf5ddb82b, 0x57350966, 0x2210b02a, 0xacbbc820, 0xa056d98e, 0xf630e99f, 0xccebe027,
    0x3830e73a, 0x82dddf43, 0x140277c8, 0x3ae7d2d2, 0x1da65ab9, 0x843ca648, 0x87e36b81, 0x7c5b8e0d,
    0xd5e33e5f, 0xc3ce5d40, 0xe89f77bd, 0x4ef637df, 0x0e5490ad, 0x132ee23e, 0x21a3eaa1, 0x263cd6b4,
    0xb7fc2474, 0xe4668d58, 0x202dac5e, 0x4eb0fe58, 0xead371a8, 0x70553dcb, 0xe72056a1, 0xd7c8711f,
    0x6000444f, 0x52126de2, 0x5a0cb661, 0xf358b1d0, 0x38743c64, 0xf2a3c979, 0xc5fb213f, 0x2ce68765,
    0x9dd00550, 0x79b82528, 0x2dfc1a6c, 0xbb0fb9b9, 0x8061fa00, 0xa165b73e, 0x7db1af5b, 0xdc56ce31,
    0x2dcf64bd, 0x1a7be25e, 0x73e06500, 0x9112d06d, 0xd3542107, 0xa548b15b, 0x5653a4f8, 0xcb5071dd,
    0x7eb64496, 0x8c8ad21b, 0x2543bdb3, 0x374e1b9e, 0x559c84cf, 0x36474c7a, 0xb422562c, 0xc84d64de,
    0x9b01ebfc, 0xa2c4a518, 0xc221c3e7, 0x8fea9a2f, 0x8649a42c, 0x0bbc9cf0, 0x281f8c78, 0x64988dc0,
    0x26f6a4f6, 0xf33a8c9d, 0x43a72954, 0x6bd476e5, 0x5875f459, 0x9050ced5, 0xeca39d44, 0x5d533d20,
    0x465c6848, 0xa068fea3, 0xd476c9dd, 0x1d6f83e2, 0x14c58fa0, 0xa2f475a3, 0x1284ef7f, 0x75bb7b32,
    0x9dd0912f, 0x071121e8, 0x80ec2e7c, 0xfeea81d0, 0xb741901f, 0xbffc4032, 0xb8a794a6, 0x09eb19e3,
    0x3bafe495, 0x7f4489d3, 0x2fc47769, 0xbaca3ad7, 0x1d0b8292, 0xe7527fce, 0x7587fc94, 0xf4b60714,
    0x55958ecc, 0xab9b7c3e, 0x70f582db, 0x78b31708, 0x39e168c5, 0xc0f54dd0, 0x5697ea6c, 0xbb0023a1,
    0xc1c446c3, 0xfcf2da1a, 0xa6fb900d, 0xc904c6e6, 0xe092557f, 0xaaee4e89, 0xbfb8322f, 0x8a6b8c6e,
    0xc6e03fa7, 0xd34b8801, 0x8cf5d8a8, 0xe0120455, 0xaa98d008, 0x49b55b75, 0x87fe9042, 0x14bc7bbb,
    0x987dfbbe, 0xb4062c82, 0x8fce320a, 0xca5979bf, 0x2f24b1ae, 0x19845549, 0x565ed62e, 0xddbf4a10,
    0x495f5872, 0x1139006a, 0x88b5f60c, 0xb80185c7, 0xf9542f3b, 0x18421c4a, 0x3f45498e, 0x1a4d2f5a,
    0x07598c4c, 0x7cb71127, 0x108bb096, 0x9265f0e0, 0x83e58d1f, 0xd1268285, 0xc1922967, 0x51975ea9,
    0xddf5a2cc, 0x8ea9e033, 0x135cd3ae, 0xee179459, 0x9b5f1c5e, 0x15941e23, 0xf944a317, 0x08351f74,
    0x673fba13, 0x4e375b90, 0xb56427f4, 0xed605b66, 0x3fb66371, 0x772d4d0c, 0x7201a974, 0x3042d3ca,
    0x974ef5f9, 0xed34a0eb, 0x1a27146a, 0x946f3646, 0x7d665b75, 0xfde17ec9, 0x95a8c7ae, 0x6b6f439f,
    0xaa04260f, 0x35706e11, 0x5b02c819, 0x6a0ccdc8, 0x2d6c3911, 0x1f362c78, 0xed17ebfd, 0xefd1ba22,
    0x09c910a1, 0xd8d3cec9, 0xc82b6ebe, 0xc5fbfcf9, 0x2f762e0d, 0xb6194396, 0xcee2e5a4, 0xc2e67cc0,
    0x3ae67079, 0xb8d98a82, 0xe48ca4d4, 0xf7577f89, 0x9e7ae0ea, 0x39b0831c, 0xa6393b34, 0xef777a94,
    0xd7ec846b, 0xd91066bd, 0xc7fe718d, 0x87205d5b, 0x7615275f, 0x653880c5, 0x9d9f1220, 0x1bd7ec6a,
    0x565e7016, 0x487fb46a, 0x735e05e0, 0x4e581a19, 0x34a4a923, 0xe4e2bab5, 0x96880484, 0xd15cf37d,
];
