//! Implementation for a writer of codes on a bit stream.

#[derive(Debug)]
/// A writer of codes on a bit stream.
pub struct BitWriter<'a> {
    data: &'a mut [u64],
    word_idx: usize,
    buffer: u64,
    space_left_in_buffer: usize,
}

impl<'a> core::ops::Drop for BitWriter<'a> {
    fn drop(&mut self) {
        self.flush()
    }
}

impl<'a> BitWriter<'a> {
    pub fn new(data: &'a mut [u64]) -> Self {
        Self {
            data,
            word_idx: 0,
            buffer: 0,
            space_left_in_buffer: 64,
        }
    }

    pub fn flush(&mut self) {
        let to_flush = 64 - self.space_left_in_buffer;
        if to_flush != 0 {
            let buffer = self.buffer << self.space_left_in_buffer;
            let old_word = u64::from_be(self.data[self.word_idx]);
            let clean_old_word = (old_word << to_flush) >> to_flush;
            let new_word = (clean_old_word | buffer).to_be();
            self.data[self.word_idx] = new_word;
        }
    }

    pub fn seek(&mut self, bits_idx: usize) {
        self.flush();
        self.word_idx = bits_idx / 64;
        let idx_in_word = bits_idx % 64;
        self.space_left_in_buffer = 64 - idx_in_word;
        self.buffer = self.data[self.word_idx];
    }

    pub fn tell(&self) -> usize {
        self.word_idx * 64 + (64 - self.space_left_in_buffer)
    }

    #[inline]
    pub fn write_bits(&mut self, value: u64, n_bits: usize) -> usize {
        debug_assert!(n_bits <= 64);
        debug_assert!(self.space_left_in_buffer > 0);

        if n_bits < self.space_left_in_buffer {
            self.buffer <<= n_bits;
            self.buffer |= value & !(u64::MAX << n_bits as u32);
            self.space_left_in_buffer -= n_bits;
            return n_bits;
        }

        self.buffer = self.buffer << (self.space_left_in_buffer - 1) << 1;
        self.buffer |= value << (64 - n_bits) >> (64 - self.space_left_in_buffer);
        self.data[self.word_idx] = self.buffer.to_be();
        self.word_idx += 1;

        let mut to_write = n_bits - self.space_left_in_buffer;

        for _ in 0..to_write / 64 {
            to_write -= 64;
            self.data[self.word_idx] = (value >> to_write).to_be();
            self.word_idx += 1;
        }

        self.space_left_in_buffer = 64 - to_write;
        self.buffer = value;
        n_bits
    }

    #[inline]
    pub fn write_unary(&mut self, mut value: u64) -> usize {
        debug_assert_ne!(value, u64::MAX);
        debug_assert!(self.space_left_in_buffer > 0);

        let code_length = value + 1;

        if code_length <= self.space_left_in_buffer as u64 {
            self.space_left_in_buffer -= code_length as usize;
            self.buffer = self.buffer << value << 1;
            self.buffer |= 1;
            if self.space_left_in_buffer == 0 {
                self.data[self.word_idx] = self.buffer.to_be();
                self.word_idx += 1;
                self.space_left_in_buffer = 64;
            }
            return code_length as usize;
        }

        self.buffer = self.buffer << (self.space_left_in_buffer - 1) << 1;
        self.data[self.word_idx] = self.buffer.to_be();
        self.word_idx += 1;

        value -= self.space_left_in_buffer as u64;

        for _ in 0..value / 64 {
            self.data[self.word_idx] = 0;
            self.word_idx += 1;
        }

        value %= 64;

        if value == 64 - 1 {
            self.data[self.word_idx] = 1_u64.to_be();
            self.word_idx += 1;
            self.space_left_in_buffer = 64;
        } else {
            self.buffer = 1;
            self.space_left_in_buffer = 64 - (value as usize + 1);
        }

        code_length as usize
    }

    pub fn write_minimal_binary(&mut self, value: u64, max: u64) -> usize {
        let l = max.ilog2();
        let limit = (1 << (l + 1)) - max;

        if value < limit {
            self.write_bits(value, l as _)
        } else {
            let to_write = value + limit;
            self.write_bits(to_write >> 1, l as _);
            self.write_bits(to_write & 1, 1);
            (l + 1) as usize
        }
    }

    pub fn write_gamma(&mut self, mut value: u64) -> usize {
        value += 1;
        let n_bits = value.ilog2();
        self.write_unary(n_bits as u64) + self.write_bits(value, n_bits as usize)
    }

    pub fn write_delta(&mut self, mut value: u64) -> usize {
        value += 1;
        let n_bits = value.ilog2();
        self.write_gamma(n_bits as u64) + self.write_bits(value, n_bits as usize)
    }

    pub fn write_rice(&mut self, value: u64, b: u64) -> usize {
        self.write_unary(value >> b) + self.write_bits(value, b as usize)
    }

    pub fn write_golomb(&mut self, value: u64, b: u64) -> usize {
        self.write_unary(value / b) + self.write_minimal_binary(value % b, b)
    }

    pub fn write_exp_golomb(&mut self, value: u64, b: u64) -> usize {
        self.write_gamma(value >> b) + self.write_bits(value, b as usize)
    }

    pub fn write_zeta(&mut self, mut value: u64, k: u64) -> usize {
        value += 1;
        let h = value.ilog2() as u64 / k;
        let u = 1 << ((h + 1) * k);
        let l = 1 << (h * k);

        debug_assert!(l <= value, "{} <= {}", l, value);
        debug_assert!(value < u, "{} < {}", value, u);

        self.write_unary(h) + self.write_minimal_binary(value - l, u - l)
    }

    pub fn write_pi(&mut self, mut value: u64, k: u64) -> usize {
        value += 1;
        let r = value.ilog2() as usize;
        let h = 1 + r;
        let l = h.div_ceil(1 << k);
        let v = (l * (1 << k) - h) as u64;
        let rem = value & !(u64::MAX << r);

        let mut written_bits = 0;
        written_bits += self.write_unary((l - 1) as u64);
        written_bits += self.write_bits(v, k as usize);
        written_bits += self.write_bits(rem, r);

        written_bits
    }

    pub fn write_pi_web(&mut self, value: u64, k: u64) -> usize {
        if value == 0 {
            self.write_bits(1, 1)
        } else {
            self.write_bits(0, 1) + self.write_pi(value - 1, k)
        }
    }
}
