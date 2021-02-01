use std::fs::File;
use std::io::Write;

use rayon::prelude::*;

#[derive(Debug, Copy, Clone)]
pub struct Stats {
    pub avalanche: [[f64; 32]; 32],
    pub avalanche_avg_bias: [[f64; 32]; 32], // Average avalanche bias over many seeds.
    pub tree_bias: [[f64; 32]; 32],
}

pub const STATS_ZERO: Stats = Stats {
    avalanche: [[0.0; 32]; 32],
    avalanche_avg_bias: [[0.0; 32]; 32],
    tree_bias: [[0.0; 32]; 32],
};

/// Measures the statistics of the provided hash function.
pub fn measure_stats<F>(hash: F, rounds: u32, print_progress: bool) -> Stats
where
    F: Fn(u32, u32) -> u32 + Sync, // (input, seed) -> output
{
    // Break up the rounds into chunks that we can hoist off to different
    // threads.
    let sub_rounds = 256;
    let loop_rounds = (rounds / sub_rounds) + ((rounds % sub_rounds) != 0) as u32;
    let rounds = loop_rounds * sub_rounds;

    if print_progress {
        print!("Progress..");
        std::io::stdout().flush();
    }
    let data = (0..loop_rounds)
        .into_par_iter()
        .map(|lr| {
            if print_progress && (lr % (loop_rounds / 53).max(1)) == 0 {
                let stdout = std::io::stdout();
                let mut out = stdout.lock();
                out.write_all(b".");
                out.flush();
            }

            // Run tests and collect data.
            let seed = rand::random::<u32>();
            let mut data = STATS_ZERO;
            for i in 0..sub_rounds {
                // Avalanche and avalanche bias.
                let input_1 = rand::random::<u32>();
                let output_1 = hash(input_1, seed);
                for bit_in in 0..32 {
                    let input_2 = input_1 ^ (1 << bit_in);
                    let output_2 = hash(input_2, seed);
                    let diff_1 = output_1 ^ output_2;
                    for bit_out in 0..32 {
                        if (diff_1 & (1 << bit_out)) != 0 {
                            data.avalanche[bit_in][bit_out] += 1.0;
                            data.avalanche_avg_bias[bit_in][bit_out] += 1.0;
                        }
                    }
                }

                // Tree seeding bias.
                let seed2 = rand::random::<u32>();
                let input_3 = rand::random::<u32>();
                let output_3 = hash(input_3, seed2);
                let input_4 = rand::random::<u32>();
                let output_4 = hash(input_4, seed2);
                let mut x = output_3 ^ output_4;
                let mut y = input_3 ^ input_4;
                while x & 1 == 0 && y & 1 == 0 && (x != 0 || y != 0) {
                    x >>= 1;
                    y >>= 1;
                }
                y = y.reverse_bits() >> 26;
                x = x.reverse_bits() >> 26;
                data.tree_bias[x as usize & 0b11111][y as usize & 0b11111] += 0.5;
            }

            // Process data.
            for i in 0..32 {
                for j in 0..32 {
                    data.avalanche_avg_bias[i][j] =
                        (data.avalanche_avg_bias[i][j] - (0.5 * sub_rounds as f64)).abs();
                }
            }

            data
        })
        .reduce(
            || STATS_ZERO,
            |mut a, b| {
                for i in 0..32 {
                    for j in 0..32 {
                        a.avalanche[i][j] += b.avalanche[i][j];
                        a.avalanche_avg_bias[i][j] += b.avalanche_avg_bias[i][j];
                        a.tree_bias[i][j] += b.tree_bias[i][j];
                    }
                }
                a
            },
        );
    if print_progress {
        print!(
            "\r                                                                                \r"
        );
    }

    let mut stats = STATS_ZERO;
    for i in 0..32 {
        for j in 0..32 {
            stats.avalanche[i][j] += data.avalanche[i][j] / rounds as f64;
            stats.avalanche_avg_bias[i][j] += data.avalanche_avg_bias[i][j] * 2.0 / rounds as f64;
            stats.tree_bias[i][j] += data.tree_bias[i][j] / rounds as f64 * 32.0 * 32.0;
        }
    }

    stats
}

pub fn print_stats(stats: Stats) {
    // Calculate reduced stats
    let mut reduced_stats = [0.0f64; 32]; // (avg, max)
    for bit_in in 0..32 {
        for bit_out in (bit_in + 1)..32 {
            reduced_stats[bit_out] += stats.avalanche_avg_bias[bit_in][bit_out] / bit_out as f64;
        }
    }

    // Calculate average bias.
    let mut avg_bias = 0.0;
    for bit_in in 0..32 {
        for bit_out in (bit_in + 1)..32 {
            avg_bias += stats.avalanche_avg_bias[bit_in][bit_out];
        }
    }
    avg_bias /= (32 * 31 / 2) as f64;

    // Print info.
    println!("Per-output-bit average bias:\n{:0.2?}", reduced_stats);
    println!("Total average bias:\n{:0.3}", avg_bias);
}

pub fn write_stats_image(stats: Stats, file: &mut File) {
    const BIT_PIXEL_SIZE: usize = 8;
    const WIDTH: usize = BIT_PIXEL_SIZE * 32 * 3;
    const HEIGHT: usize = BIT_PIXEL_SIZE * 32;
    let mut image = vec![0x00u8; 4 * WIDTH * HEIGHT];
    let mut plot = |x: usize, y: usize, color: u8| {
        let min_x = x * BIT_PIXEL_SIZE;
        let min_y = y * BIT_PIXEL_SIZE;
        let max_x = min_x + BIT_PIXEL_SIZE;
        let max_y = min_y + BIT_PIXEL_SIZE;

        for y in min_y..max_y {
            for x in min_x..max_x {
                image[(y * WIDTH + x) * 4] = color;
                image[(y * WIDTH + x) * 4 + 1] = color;
                image[(y * WIDTH + x) * 4 + 2] = color;
                image[(y * WIDTH + x) * 4 + 3] = 0xFF;
            }
        }
    };

    for bit_in in 0..32 {
        for bit_out in 0..32 {
            let color_avalanche =
                (stats.avalanche[bit_in][bit_out].min(1.0).max(0.0) * 255.0) as u8;
            let color_avalanche_bias =
                (stats.avalanche_avg_bias[bit_in][bit_out].min(1.0).max(0.0) * 255.0) as u8;
            let color_tree = (stats.tree_bias[bit_in][bit_out].min(1.0).max(0.0) * 255.0) as u8;
            plot(bit_out, bit_in, color_avalanche);
            plot(bit_out + 32, bit_in, color_avalanche_bias);
            plot(bit_out + 64, bit_in, color_tree);
        }
    }
    png_encode_mini::write_rgba_from_u8(file, &image, WIDTH as u32, HEIGHT as u32);
}
