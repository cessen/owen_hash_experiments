#![allow(unused)]

mod hash_gen;
mod sobol;
mod stats;

use std::fs::File;
use std::io::Write;

use hash_gen::{exec_hash_slice, HashOp};
use stats::{measure_stats, print_stats, write_stats_image, Stats, STATS_ZERO};

fn main() {
    // Set rayon per-thread stack size, because by default it's too small
    // for what we're doing.
    rayon::ThreadPoolBuilder::new()
        .stack_size(1024 * 1024 * 16)
        .build_global()
        .unwrap();

    // Parse command line arguments.
    let args = clap::App::new("Sample Testing")
        .version("0.123456789")
        .about("")
        .arg(clap::Arg::with_name("test").long("test"))
        .arg(clap::Arg::with_name("search").long("search"))
        .arg(clap::Arg::with_name("reference").long("ref"))
        .arg(
            clap::Arg::with_name("number")
                .takes_value(true)
                .required(false),
        )
        .get_matches();

    // Pick what to do based on command line arguments.
    if args.is_present("test") {
        let rounds = args
            .value_of("number")
            .unwrap_or("4000000")
            .parse()
            .unwrap();
        do_test(rounds, true);
    } else if args.is_present("search") {
        let rounds = args.value_of("number").unwrap_or("10000").parse().unwrap();
        do_hash_search(rounds, true);
    } else {
        let image_resolution = 320;
        let image_count = args.value_of("number").unwrap_or("4").parse().unwrap();
        let sample_function = if args.is_present("reference") {
            |i, d, seed| sobol::sample_owen_reference(i, d, seed)
        } else {
            |i, d, seed| sobol::sample_owen_fast(i, d, seed)
        };

        for seed in 0..image_count {
            generate_samples_image(
                sample_function,
                image_resolution,
                &[256, 1024, 4096],
                seed,
                &format!("{:02}.png", seed),
            );
        }
    }
}

//=======================================================================
// SUB-COMMANDS
//=======================================================================

/// Generates a bunch of 2d Owen-scrambled Sobol points, and writes them
/// to an image.
fn generate_samples_image<F>(
    sample: F,
    resolution: usize,
    point_counts: &[u32], // A list of point-counts, which will be drawn sequentially in the image, left-to-right.
    seed: u32,
    image_path: &str,
) where
    F: Fn(u32, u32, u32) -> f32, // (sample_index, dimension, seed) -> coordinate
{
    const POINT_RADIUS: usize = 2;

    let width = resolution * point_counts.len();
    let height = resolution;
    let mut image = vec![0xffu8; width * height * 4];

    // Draws a point on the image.
    let mut plot = |x: usize, y: usize| {
        let min_x = x.saturating_sub(POINT_RADIUS);
        let min_y = y.saturating_sub(POINT_RADIUS);
        let max_x = (x + POINT_RADIUS + 1).min(width);
        let max_y = (y + POINT_RADIUS + 1).min(height);

        for yy in min_y..max_y {
            for xx in min_x..max_x {
                let x2 = x as isize - xx as isize;
                let y2 = y as isize - yy as isize;
                if (((x2 * x2) + (y2 * y2)) as f64).sqrt() <= POINT_RADIUS as f64 {
                    image[(yy * width + xx) * 4] = 0x00;
                    image[(yy * width + xx) * 4 + 1] = 0x00;
                    image[(yy * width + xx) * 4 + 2] = 0x00;
                    image[(yy * width + xx) * 4 + 3] = 0xFF;
                }
            }
        }
    };

    // Plot the points at the various point counts.
    for (set_idx, &point_count) in point_counts.iter().enumerate() {
        for i in 0..point_count {
            let x = sample(i, 0, seed);
            let y = sample(i, 1, seed + 1);
            plot(
                (x * (resolution - 1) as f32) as usize + (resolution * set_idx),
                (y * (resolution - 1) as f32) as usize,
            );
        }
    }

    let mut file = File::create(image_path).unwrap();
    png_encode_mini::write_rgba_from_u8(&mut file, &image, width as u32, height as u32);
}

/// Tests the statistics of a hash, and prints the results to the console.
/// Optionally writes a png image as well.
fn do_test(rounds: u32, with_image: bool) {
    let stats = measure_stats(
        |n, seed| {
            let mut n = n;

            // Reference Owen scramble implementation, performed on
            // reversed bits.
            n = n.reverse_bits();
            n = sobol::owen_scramble_reference_u32(n, seed);
            n = n.reverse_bits();

            // // Original Laine-Karras hash.
            // n = n.wrapping_add(seed);
            // n ^= n.wrapping_mul(0x6c50b47c);
            // n ^= n.wrapping_mul(0xb82f1e52);
            // n ^= n.wrapping_mul(0xc7afe638);
            // n ^= n.wrapping_mul(0x8d22f6e6);

            // // Improved version 2.
            // // From https://psychopath.io/post/2021_01_02_sobol_sampling_take_2
            // n = n.wrapping_add(seed);
            // n ^= 0xdc967795;
            // n = n.wrapping_mul(0x97b756bb);
            // n ^= 0x866350b1;
            // n = n.wrapping_mul(0x9e3779cd);

            // // Run a generated hash.  Note: a constant of zero in an op
            // // indicates using the seed.
            // n = exec_hash_slice(
            //     // Fast, reasonable quality.
            //     &[
            //         HashOp::Add(0),
            //         HashOp::MulXor(0x3354734a),
            //         HashOp::ShlAdd(2),
            //         HashOp::MulXor(0),
            //     ],
            //     // // Medium-fast, good quality.
            //     // &[
            //     //     HashOp::Add(0),
            //     //     HashOp::MulXor(0x046e2f26),
            //     //     HashOp::Mul(0),
            //     //     HashOp::MulXor(0x75d5ab5c),
            //     //     HashOp::Mul(0xdc4d0c55),
            //     // ],
            //     n,
            //     seed,
            // );

            n
        },
        rounds,
        true,
    );

    // Print stats.
    print_stats(stats);
    println!();

    // Write avalanche image.
    if with_image {
        write_stats_image(stats, &mut File::create("stats.png").unwrap());
    }
}

/// Randomly searches for better hashes, and prints the result to console.
/// Optionally also saves statistics png images of the top produced hashes.
///
/// All this does is generate hashes randomly, and keep the highest-scoring
/// ones.  No fancy mutation approaches or whatnot, unfortunately.
fn do_hash_search(rounds: usize, with_image: bool) {
    use std::collections::HashMap;

    const HASH_OP_COUNT: usize = 3;
    const CANDIDATE_COUNT: usize = 8;
    const STAT_ROUNDS: u32 = 1 << 18;

    // Method to use to generate new hashes.
    let generate = || {
        // Generate a totally random hash, but ensuring at least one
        // op that involves multiplying by the seed (which seems to be critical
        // to all decent hashes).
        let mut hash = [HashOp::Nop; 8];
        let mut any_mul_seed = false;
        while !any_mul_seed {
            for i in 0..hash.len() {
                hash[i] = HashOp::gen_random();
                any_mul_seed |= hash[i].uses_mul_and_seed();
            }
        }
        hash

        // // Start with an existing hash, and just generate a new random
        // // constant for one of the operations.
        // [
        //     HashOp::Add(0),
        //     HashOp::MulXor(0x046e2f26),
        //     HashOp::Mul(0x75d5ab5b).new_constant(),
        //     HashOp::MulXor(0),
        //     HashOp::Mul(0xdc4d0c55),
        // ]
    };

    //----------------
    // Do actual optimization process.
    //----------------

    let mut candidates: Vec<_> = (0..CANDIDATE_COUNT)
        .map(|_| (generate(), std::f64::INFINITY, STATS_ZERO))
        .collect();
    let last_idx = candidates.len() - 1;

    println!();
    for round in 0..rounds {
        print!("\rround {}/{}", round, rounds);
        std::io::stdout().flush();

        // Generate and score a new hash.
        let new_hash = generate();
        let (stats, score) = {
            let stats = measure_stats(
                |n, seed| exec_hash_slice(&new_hash[..], n, seed),
                STAT_ROUNDS,
                false,
            );
            (stats, score_stats(&stats))
        };

        // If it beats the current lowest-scoring hash, replace it.
        if score < candidates[last_idx].1 {
            candidates[last_idx] = (new_hash, score, stats);
            candidates.sort_unstable_by(|x, y| x.1.partial_cmp(&y.1).unwrap());
        }
    }
    println!();

    // Print out the top hashes, and (optionally) write statistics png images
    // for them as well.
    for (i, c) in candidates.iter().enumerate() {
        println!("Score: {}", c.1);

        print!("&[");
        for p in c.0.iter() {
            print!("HashOp::{:?}, ", *p);
        }
        println!("]");
        print_stats(c.2);
        println!();

        if with_image {
            write_stats_image(
                c.2,
                &mut File::create(&format!("candidate_{:02}.png", i + 1)).unwrap(),
            );
        }
    }
}

//=======================================================================
// UTILS
//=======================================================================

fn hash_u32(n: u32, seed: u32) -> u32 {
    // Seeding.  This is totally hacked together, but it seems to work well.
    let mut n = 1 + n.wrapping_add(seed.wrapping_mul(0x736caf6f));

    // From https://github.com/skeeto/hash-prospector
    n ^= n >> 17;
    n = n.wrapping_mul(0xed5ad4bb);
    n ^= n >> 11;
    n = n.wrapping_mul(0xac4c1b51);
    n ^= n >> 15;
    n = n.wrapping_mul(0x31848bab);
    n ^= n >> 14;

    n
}

/// Scores the given hash statistics.  Used for searching for better hashes.
///
/// Lower score is better (like golf!).
fn score_stats(stats: &Stats) -> f64 {
    let mut score = 0.0;

    // Tree bias metric
    for x in 0..32 {
        for y in (x + 1)..32 {
            let diff = stats.tree_bias[x][y] - 0.5;

            // In practice, it seems we only need to worry about extreme,
            // values, as avoiding extremes seems to bring everything to
            // a good place.
            score += if diff.abs() > 0.45 { 1.0 } else { 0.0 };
        }
    }

    // Avalanche bias metric, trying to match the expected bias of a
    // proper full Owen scramble.  The first sixteen values here were computed
    // analytically, and the remaining were approximated following a strong
    // trend in the values by that point, and should be "reasonably" accurate.
    const TARGET_BIAS: [f64; 32] = [
        0.0, 1.0, 0.5, 0.375, 0.273437, 0.19638, 0.139949, 0.099346, 0.070386, 0.049819, 0.035244,
        0.024927, 0.017628, 0.012466, 0.008815, 0.006233, 0.004407, 0.003117, 0.002204, 0.001558,
        0.001102, 0.000779, 0.000551, 0.000390, 0.000275, 0.000195, 0.000138, 0.000097, 0.000069,
        0.000049, 0.000034, 0.000024,
    ];
    for bit_out in 0..32 {
        for bit_in in 0..bit_out {
            let diff = stats.avalanche_bias[bit_in][bit_out] - TARGET_BIAS[bit_out];
            score += diff * diff;
        }
    }

    score
}
