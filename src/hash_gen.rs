use rand::random;

// A single operation in an Owen-scramble hash.
//
// For all operations, having a constant of zero is abused
// to mean "use the passed seed".  This is because for all
// operations a constant of zero is either effectively a no-op,
// or it's completely invalid for this kind of hash anyway.
#[derive(Debug, Copy, Clone)]
pub enum HashOp {
    Nop,         // Do nothing
    Xor(u32),    // x ^= constant
    Add(u32),    // x += constant
    Mul(u32),    // x *= odd_constant
    ShlXor(u32), // x ^= x << constant[1, 31]
    ShlAdd(u32), // x += x << constant[1, 31]
    MulXor(u32), // x ^= x * even_constant
}

impl HashOp {
    pub fn gen_random() -> HashOp {
        // 1/4 chance of selecting the seed, otherwise random constant.
        let constant = if (random::<u32>() & 0b11) == 0 {
            0
        } else {
            random::<u32>()
        };

        match random::<u32>() % 3 {
            0 => HashOp::Add(constant),
            1 => HashOp::Mul(constant | 1),
            2 => HashOp::MulXor(constant & !1),
            // 3 => HashOp::Xor(constant),
            // 4 => HashOp::ShlXor((constant % 31) + 1),
            // 5 => HashOp::ShlAdd((constant % 31) + 1),
            // 6 => HashOp::Nop,
            _ => unreachable!(),
        }
    }

    pub fn new_constant(&self) -> HashOp {
        match *self {
            HashOp::Nop => *self,

            HashOp::Xor(c) => {
                if c == 0 {
                    *self
                } else {
                    HashOp::Xor(random::<u32>())
                }
            }

            HashOp::Add(c) => {
                if c == 0 {
                    *self
                } else {
                    HashOp::Add(random::<u32>())
                }
            }

            HashOp::Mul(c) => {
                if c == 0 {
                    *self
                } else {
                    HashOp::Mul(random::<u32>() | 1)
                }
            }

            HashOp::ShlXor(c) => {
                if c == 0 {
                    *self
                } else {
                    HashOp::ShlXor((random::<u32>() % 31) + 1)
                }
            }

            HashOp::ShlAdd(c) => {
                if c == 0 {
                    *self
                } else {
                    HashOp::ShlAdd((random::<u32>() % 31) + 1)
                }
            }

            HashOp::MulXor(c) => {
                if c == 0 {
                    *self
                } else {
                    HashOp::MulXor(random::<u32>() & !1)
                }
            }
        }
    }

    pub fn exec(&self, x: u32, seed: u32) -> u32 {
        match *self {
            HashOp::Nop => x,

            HashOp::Xor(c) => {
                if c == 0 {
                    x ^ seed
                } else {
                    x ^ c
                }
            }

            HashOp::Add(c) => {
                if c == 0 {
                    x.wrapping_add(seed)
                } else {
                    x.wrapping_add(c)
                }
            }

            HashOp::Mul(c) => {
                if c == 0 {
                    x.wrapping_mul(seed | 1)
                } else {
                    x.wrapping_mul(c)
                }
            }

            HashOp::ShlXor(c) => {
                if c == 0 {
                    x ^ (x << (seed & 0b11111))
                } else {
                    x ^ (x << c)
                }
            }

            HashOp::ShlAdd(c) => {
                if c == 0 {
                    x.wrapping_add(x << (seed & 0b11111))
                } else {
                    x.wrapping_add(x << c)
                }
            }

            HashOp::MulXor(c) => {
                if c == 0 {
                    x ^ x.wrapping_mul(seed & !1)
                } else {
                    x ^ x.wrapping_mul(c)
                }
            }
        }
    }

    pub fn uses_mul_and_seed(&self) -> bool {
        match *self {
            HashOp::Nop => false,
            HashOp::Xor(c) => false,
            HashOp::Add(c) => false,
            HashOp::Mul(c) => c == 0,
            HashOp::ShlXor(c) => false,
            HashOp::ShlAdd(c) => false,
            HashOp::MulXor(c) => c == 0,
        }
    }
}

/// Runs a slice of `HashOp`s as a hash function on the given
/// value with the given seed.
pub fn exec_hash_slice(hash_ops: &[HashOp], x: u32, seed: u32) -> u32 {
    let mut x = x;
    for op in hash_ops.iter() {
        x = op.exec(x, seed);
    }
    x
}
