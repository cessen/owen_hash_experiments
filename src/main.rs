#![allow(unused)]

use std::fs::File;

mod halton;
mod r2;
mod sobol;

fn main() {
    // let (perms, stats) = optimize(
    //     2500,
    //     8, // Simultaneous candidates to use.
    //     2, // Bits to ignore.
    //     || {
    //         [
    //             rand::random::<u32>(),
    //             rand::random::<u32>(),
    //             // rand::random::<u32>(),
    //             // rand::random::<u32>(),
    //         ]
    //     },
    //     |n| {
    //         let idx = rand::random::<u8>() as usize % n.len();
    //         let mut n = n;
    //         n[idx] = n[idx] ^ (1 << (rand::random::<u8>() % 32));
    //         n
    //     },
    //     |a, n| {
    //         let mut b = a;
    //         for p in n.chunks(2) {
    //             b ^= b.wrapping_mul(p[0] & !1);
    //             b = b.wrapping_mul(p[1] | 1);
    //         }
    //         b
    //     },
    // );

    // for x in perms.iter() {
    //     println!("{:032b}", *x);
    // }
    // print!("[");
    // for p in perms.iter() {
    //     print!("0x{:08x?}, ", *p);
    // }
    // println!("]");
    // println!("stats: {:0.3?}", stats);

    //-------------------------------------------------------------------

    // for &hash_rounds in [1, 2, 3, 4, 8, 16, 32, 64, 128].iter() {
    //     let variant_rounds = 64;
    //     let avalanche_rounds = (1 << 14);

    //     let avalanche_stats = (0..variant_rounds)
    //         .map(|seed| {
    //             let rand_ints: Vec<u32> = (0..(hash_rounds * 4))
    //                 .map(|_| rand::random::<u32>())
    //                 .collect();

    //             measure_avalanche(avalanche_rounds, |n| {
    //                 let mut n = n;

    //                 // // LK rounds
    //                 // n += hash_u32(seed, 0);
    //                 // for i in 0..hash_rounds {
    //                 //     n ^= n.wrapping_mul(rand_ints[i] << 1);
    //                 // }

    //                 // // Improved v4
    //                 // n += hash_u32(seed, 0);
    //                 // for i in 0..hash_rounds {
    //                 //     n ^= n.wrapping_mul(rand_ints[i] & !1);
    //                 //     n = n.wrapping_mul(rand_ints[i + hash_rounds] | 1);
    //                 // }

    //                 // // // Improved v4 with optimized constants.
    //                 // let perms = [
    //                 //     (0x9ac7ea2a, 0x7d1e78d3), // Only this first pair is optimized.
    //                 //     (0x2ce68764, 0x9dd00551),
    //                 //     (0x79b82526, 0x2dfc1a6b),
    //                 //     (0xf358b1d0, 0x38743c65),
    //                 // ];
    //                 // n += hash_u32(seed, 0);
    //                 // for (p1, p2) in perms
    //                 //     .iter()
    //                 //     .map(|n| *n)
    //                 //     .chain(rand_ints.chunks(2).map(|a| (a[0], a[1])))
    //                 //     .take(hash_rounds)
    //                 // {
    //                 //     n ^= n.wrapping_mul(p1 & !1);
    //                 //     n = n.wrapping_mul(p2 | 1);
    //                 // }

    //                 // Add Xor version
    //                 n += hash_u32(seed, 0);
    //                 for p in rand_ints.chunks(2).cycle().take(hash_rounds) {
    //                     n = n.wrapping_add(p[0]);
    //                     n ^= p[1];
    //                 }

    //                 // n = n.reverse_bits();
    //                 // n = sobol::owen_scramble_slow(n, seed);
    //                 // n = n.reverse_bits();

    //                 n
    //             })
    //         })
    //         .fold([(0.0f64, 0.0f64); 32], |a, b| {
    //             let mut c = [(0.0f64, 0.0f64); 32];
    //             for i in 0..32 {
    //                 c[i].0 = a[i].0 + (b[i].0 / variant_rounds as f64);
    //                 c[i].1 = a[i].1.max(b[i].1);
    //             }
    //             c
    //         });

    //     // println!("\n{:0.2?}", avalanche_stats);

    //     let avg_bias = (&avalanche_stats[1..])
    //         .iter()
    //         .map(|n| n.0)
    //         .fold(0.0f64, |a, b| a + b)
    //         / 31.0;
    //     let max_bias = (&avalanche_stats[6..])
    //         .iter()
    //         .map(|n| n.1)
    //         .fold(0.0f64, |a, b| a.max(b));
    //     let avg_max_bias = (&avalanche_stats[1..])
    //         .iter()
    //         .map(|n| n.1)
    //         .fold(0.0f64, |a, b| a + b)
    //         / 31.0;

    //     // Average bias
    //     println!(
    //         "{} rounds: ({:0.3} | {:0.3} | {:0.3})",
    //         hash_rounds, avg_bias, avg_max_bias, max_bias
    //     );
    // }

    //-------------------------------------------------------------------

    const RES: usize = 384;
    const SETS: &[u32] = &[64, 256, 1024, 4096];
    const PLOT_RADIUS: usize = 2;

    for seed in 0..8 {
        let width = RES * SETS.len();
        let height = RES;
        let mut image = vec![0xffu8; width * height * 4];
        let mut file = File::create(&format!("{:02}.png", seed)).unwrap();

        let mut plot = |x: usize, y: usize| {
            let min_x = x.saturating_sub(PLOT_RADIUS);
            let min_y = y.saturating_sub(PLOT_RADIUS);
            let max_x = (x + PLOT_RADIUS + 1).min(width);
            let max_y = (y + PLOT_RADIUS + 1).min(height);

            for yy in min_y..max_y {
                for xx in min_x..max_x {
                    let x2 = x as isize - xx as isize;
                    let y2 = y as isize - yy as isize;
                    if (((x2 * x2) + (y2 * y2)) as f64).sqrt() <= PLOT_RADIUS as f64 {
                        image[(yy * width + xx) * 4] = 0x00;
                        image[(yy * width + xx) * 4 + 1] = 0x00;
                        image[(yy * width + xx) * 4 + 2] = 0x00;
                        image[(yy * width + xx) * 4 + 3] = 0xFF;
                    }
                }
            }
        };

        let scramble_1 = seed * 2;
        let scramble_2 = seed * 2 + 1;
        for si in 0..SETS.len() {
            for i in 0..SETS[si] {
                let x = sobol::sample_owen(0, i, scramble_1);
                let y = sobol::sample_owen(1, i, scramble_2);

                plot(
                    (x * (RES - 1) as f32) as usize + (RES * si),
                    (y * (RES - 1) as f32) as usize,
                );
            }
        }

        png_encode_mini::write_rgba_from_u8(&mut file, &image, width as u32, height as u32);
    }
}

fn hash_u32(n: u32, seed: u32) -> u32 {
    let mut hash = n;
    let perms = [0x29aaaaa6, 0x54aad35a, 0x2ab35aaa];
    for p in perms.iter() {
        hash = hash.wrapping_mul(*p);
        hash ^= hash.wrapping_shr(16);
        hash ^= seed;
    }

    hash
}

fn optimize<T: Copy, F1, F2, F3>(
    rounds: usize,
    candidates: usize,
    ignore_bits: usize, // Ignore the lowest N bits when scoring.
    generate: F1,
    mutate: F2,
    execute: F3,
) -> (T, [f64; 32])
where
    F1: Fn() -> T,
    F2: Fn(T) -> T,
    F3: Fn(u32, T) -> u32,
{
    let mut current: Vec<_> = (0..candidates)
        .map(|_| (generate(), std::f64::INFINITY, [(0.0f64, 0.0f64); 32]))
        .collect();

    println!();
    for round in 0..rounds {
        print!("\rround {}/{}", round, rounds);
        let do_score = |a| {
            const EX_ROUNDS: u32 = 100;
            let stats = measure_avalanche(EX_ROUNDS, |n| execute(n, a));

            // Calculate score.
            let mut score = 0.0;
            for bit in 1.max(ignore_bits)..32 {
                score += stats[bit].1; // Sum max bias
            }

            (score, stats)
        };

        for i in 0..candidates {
            let n = if i == (candidates - 1) || candidates == 1 {
                generate()
            } else {
                mutate(current[i].0)
            };
            let (score, stats) = do_score(n);
            for i2 in 0..candidates {
                if score < current[i2].1 {
                    current[i2] = (n, score, stats);
                    break;
                }
            }
        }
    }
    println!();

    let mut final_stats = [0.0f64; 32];
    for i in 0..32 {
        final_stats[i] = current[0].2[i].0; // Average bias
    }

    (current[0].0, final_stats)
}

fn measure_avalanche<F>(rounds: u32, hash: F) -> [(f64, f64); 32]
// (average bias, max bias)
where
    F: Fn(u32) -> u32,
{
    // Accumulate test data.
    let mut stats = [[0u32; 32]; 32];
    for i in 0..rounds {
        let b = rand::random::<u32>();
        let c = hash(b);
        for bit_in in 0..32 {
            let b2 = b ^ (1 << bit_in);
            let c2 = hash(b2);
            let diff = c ^ c2;
            for bit_out in (bit_in + 1)..32 {
                if (diff & (1 << bit_out)) != 0 {
                    stats[bit_in][bit_out] += 1;
                }
            }
        }
    }

    // Calculate full stats.
    let mut stats2 = [[0.0f64; 32]; 32];
    for bit_in in 0..32 {
        for bit_out in (bit_in + 1)..32 {
            let mut s = (stats[bit_in][bit_out] as f64) * 2.0 / (rounds as f64) - 1.0;
            stats2[bit_in][bit_out] = s;
        }
    }

    // Calculate reduced stats
    let mut final_stats = [(0.0f64, 0.0f64); 32];
    for i in 0..32 {
        for j in (i + 1)..32 {
            final_stats[j].0 += stats2[i][j].abs() / j as f64;
            if stats2[i][j].abs() > final_stats[j].1.abs() {
                final_stats[j].1 = stats2[i][j].abs();
            }
        }
    }

    final_stats
}
