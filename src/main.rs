#![allow(unused)]

use std::fs::File;

mod halton;
mod r2;
mod sobol;

fn main() {
    // let (perms, stats) = optimize(
    //     1 << 8,
    //     4, // Simultaneous candidates to use.
    //     0, // Bits to ignore.
    //     || {
    //         [
    //             rand::random::<u32>() & (!1),
    //             rand::random::<u32>() & (!1),
    //             rand::random::<u32>() & (!1),
    //         ]
    //     },
    //     |n| {
    //         let idx = rand::random::<u8>() as usize % n.len();
    //         let mut n = n;
    //         n[idx] = n[idx] ^ (1 << ((rand::random::<u8>() % 31) + 1));
    //         n
    //     },
    //     |a, n| {
    //         let mut b = a;
    //         for p in n.iter() {
    //             b ^= b.wrapping_mul(*p);
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

    // let perms = [0x1313e844u32, 0xa14a177e, 0x18c8e432];
    // let perms = [0xa56bb1c6u32, 0xef577134, 0xd0e5e808, 0x200bd50a];

    //------------------------------------------------

    const RES: usize = 384;
    const SETS: &[u32] = &[64, 256, 1024, 4096];
    const DIMS: usize = 1;
    const PLOT_RADIUS: usize = 2;

    let dlist: &[u32] = &[
        0, 1, //2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        //25, 26, 27, 28, 29,
    ];

    for di1 in 0..(dlist.len().saturating_sub(DIMS)) {
        let d1 = dlist[di1];

        let width = RES * SETS.len();
        let height = RES * DIMS;
        let mut image = vec![0xffu8; width * height * 4];
        let mut file = File::create(&format!("dim_{:02}.png", di1)).unwrap();

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

        for di2 in 0..DIMS {
            let d2 = dlist[di1 + 1 + di2];
            let scramble_1 = ((0 + di1 + di2) * 17) as u32;
            let scramble_2 = ((1 + di1 + di2) * 13) as u32;
            let scramble_3 = ((2 + di1 + di2) * 31) as u32;
            for si in 0..SETS.len() {
                for i in 0..SETS[si] {
                    let i = sobol::owen_scramble_u32(i, scramble_3);
                    // let x = sobol::sample(d1, i);
                    // let y = sobol::sample(d2, i);
                    let x = sobol::sample_owen(d1, i, scramble_1);
                    let y = sobol::sample_owen(d2, i, scramble_2);

                    plot(
                        (x * (RES - 1) as f32) as usize + (RES * si),
                        (y * (RES - 1) as f32) as usize + (RES * (DIMS - 1 - di2 as usize)),
                    );
                }
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
        .map(|_| (generate(), std::f64::INFINITY, [[0.0f64; 32]; 32]))
        .collect();

    for _ in 0..rounds {
        let do_score = |a| {
            const EX_ROUNDS: u32 = 1024;
            let mut stats = [[0u32; 32]; 32];
            for i in 0..EX_ROUNDS {
                let b = if i < 32 { i } else { rand::random::<u32>() };
                let c = execute(b, a);
                for bit_in in 0..32 {
                    let b2 = b ^ (1 << bit_in);
                    let c2 = execute(b2, a);
                    let diff = c ^ c2;
                    for bit_out in (bit_in + 1)..32 {
                        if (diff & (1 << bit_out)) != 0 {
                            stats[bit_in][bit_out] += 1;
                        }
                    }
                }
            }

            // Collect the stats.
            let mut stats2 = [[0.0f64; 32]; 32];
            for bit_in in 0..32 {
                for bit_out in (bit_in + 1)..32 {
                    let mut s = (stats[bit_in][bit_out] as f64) / (EX_ROUNDS as f64) - 0.5;
                    stats2[bit_in][bit_out] = s;
                }
            }

            // Calculate score.
            let mut score = 0.0;
            for bit_in in 0..32 {
                for bit_out in (bit_in + 1).max(ignore_bits)..32 {
                    score += stats2[bit_in][bit_out] * stats2[bit_in][bit_out];
                }
            }

            (score, stats2)
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

    let mut final_stats = [0.0f64; 32];
    for i in 0..32 {
        for j in (i + 1)..32 {
            final_stats[j] += current[0].2[i][j].abs() / j as f64;
        }
    }

    (current[0].0, final_stats)
}
