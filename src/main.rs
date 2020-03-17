#![allow(unused)]

use std::fs::File;

mod halton;
mod sobol;

fn main() {
    // let sobol_vecs = sobol::num_gen::generate_direction_vectors(16);

    let (perms, stats) = optimize(
        1 << 14,
        1,
        4,
        || {
            [
                rand::random::<u32>() & (!1),
                rand::random::<u32>() & (!1),
                rand::random::<u32>() & (!1),
            ]
        },
        |n| {
            let idx = rand::random::<u8>() as usize % n.len();
            let mut n = n;
            n[idx] = n[idx] ^ (1 << ((rand::random::<u8>() % 31) + 1));
            n
        },
        |a, n| {
            let mut b = a;
            for p in n.iter() {
                b ^= b.wrapping_mul(*p);
            }
            b
        },
    );

    for x in perms.iter() {
        println!("{:032b}", *x);
    }
    print!("[");
    for p in perms.iter() {
        print!("0x{:08x?}, ", *p);
    }
    println!("]");
    println!("stats: {:0.3?}", stats);

    // let perms = [0x08afbbe0, 0xa7389b46, 0x42bf6dbc];
    // let perms = [0x8457ddf0, 0x539c4da3, 0xa15fb6de, ];

    //------------------------------------------------

    const WIDTH: usize = 512;
    const HEIGHT: usize = 512;
    let mut image = [0xffu8; WIDTH * HEIGHT * 4];
    let mut file = File::create("test.png").unwrap();

    let mut plot = |x: usize, y: usize| {
        let min_x = x.saturating_sub(1);
        let min_y = y.saturating_sub(1);
        let max_x = (x + 2).min(WIDTH);
        let max_y = (y + 2).min(HEIGHT);

        for yy in min_y..max_y {
            for xx in min_x..max_x {
                image[(yy * WIDTH + xx) * 4] = 0x00;
                image[(yy * WIDTH + xx) * 4 + 1] = 0x00;
                image[(yy * WIDTH + xx) * 4 + 2] = 0x00;
                image[(yy * WIDTH + xx) * 4 + 3] = 0xFF;
            }
        }
    };

    let scramble_1 = hash_u32(0, 0);
    let scramble_2 = hash_u32(1, 0);
    let dim_1 = 19;
    let dim_2 = 20;
    for i in 0..1024 {
        // let x = (sobol::sample(dim_1, i) * (WIDTH - 1) as f32) as usize;
        // let y = (sobol::sample(dim_2, i) * (HEIGHT - 1) as f32) as usize;

        // let x = (sobol::sample_rd_scramble(dim_1, i, scramble_1) * (WIDTH - 1) as f32) as usize;
        // let y = (sobol::sample_rd_scramble(dim_2, i, scramble_2) * (HEIGHT - 1) as f32) as usize;

        let x = (sobol::sample_owen_scramble(dim_1, i, scramble_1, &perms) * (WIDTH - 1) as f32)
            as usize;
        let y = (sobol::sample_owen_scramble(dim_2, i, scramble_2, &perms) * (HEIGHT - 1) as f32)
            as usize;

        // let x = (halton::sample(dim_1, i + scramble_1)* (WIDTH - 1) as f32) as usize;
        // let y = (halton::sample(dim_2, i + scramble_1) * (HEIGHT - 1) as f32) as usize;

        plot(x, y);
    }

    png_encode_mini::write_rgba_from_u8(&mut file, &image, WIDTH as u32, HEIGHT as u32);
}

fn hash_u32(n: u32, seed: u32) -> u32 {
    let mut hash = n;
    for _ in 0..16 {
        hash = hash.wrapping_mul(1_936_502_639);
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
    const CAND: usize = 4;
    let mut current: Vec<_> = (0..CAND)
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
