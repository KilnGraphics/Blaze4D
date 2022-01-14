/// Implements the Xoshiro256++ random number algorithm.
/// See https://prng.di.unimi.it/xoshiro256plusplus.c
#[derive(Copy, Clone)]
pub struct Xoshiro256PlusPlus {
    s: [u64; 4],
}

impl Xoshiro256PlusPlus {
    const JUMP : [u64; 4] = [0x180ec6d33cfd0abau64, 0xd5a61266f0c9392cu64, 0xa9582618e03fc9aau64, 0x39abdc4529b1661cu64];
    const LONG_JUMP : [u64; 4] = [ 0x76e15d3efefdcbbfu64, 0xc5004e441c522fb3u64, 0x77710069854ee241u64, 0x39109bb02acbe635u64 ];

    /// Creates a new random number generator with specified seed
    pub const fn from_seed(seed: [u64; 4]) -> Self {
        Self{ s: seed }
    }

    const fn rotl(x: u64, k: i32) -> u64 {
        (x << k) | (x >> (64 - k))
    }

    /// Generates a new random number
    pub fn gen(&mut self) -> u64 {
        let result = Self::rotl(self.s[0] + self.s[3], 23) + self.s[0];

        let t = self.s[1] << 17;

        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];

        self.s[2] ^= t;

        self.s[3] = Self::rotl(self.s[3], 45);

        result
    }

    /// Utility function to create the jump and long_jump functions
    fn update_with<const SIZE: usize>(&mut self, update: [u64; SIZE]) {
        let mut s0 = 0u64;
        let mut s1 = 0u64;
        let mut s2 = 0u64;
        let mut s3 = 0u64;

        for i in 0..update.len() {
            for b in 0..64u32 {
                if (update[i] & (1u64 << b)) != 0u64 {
                    s0 ^= self.s[0];
                    s1 ^= self.s[1];
                    s2 ^= self.s[2];
                    s3 ^= self.s[3];
                }
                self.gen();
            }
        }

        self.s[0] = s0;
        self.s[1] = s1;
        self.s[2] = s2;
        self.s[3] = s3;
    }

    /// This function is equivalent to 2^128 calls to [`Self::gen`]. It can be used to generate
    /// 2^128 non-overlapping subsequences for parallel computations.
    pub fn jump(&mut self) {
        self.update_with(Self::JUMP)
    }

    /// This function is equivalent to 2^192 calls to [`Self::gen`]. It can be used to generate
    /// 2^64 starting points from each of which [`Self::jump`] will generate 2^64 non-overlapping
    /// subsequences for parallel distributed computations.
    pub fn long_jump(&mut self) {
        self.update_with(Self::LONG_JUMP)
    }
}

impl Iterator for Xoshiro256PlusPlus {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.gen())
    }
}