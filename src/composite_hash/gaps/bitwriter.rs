//! Implementation for a writer of codes on a bit stream.

#[derive(Debug)]
/// A writer of codes on a bit stream.
pub struct BitWriter<'a> {
    data: &'a mut [u64],
    word_idx: usize,
    buffer: u64,
    space_left_in_buffer: u8,
}

impl<'a> Drop for BitWriter<'a> {
    #[inline]
    fn drop(&mut self) {
        self.flush();
    }
}

impl<'a> BitWriter<'a> {
    #[inline]
    pub fn new(data: &'a mut [u64]) -> Self {
        debug_assert!(!data.is_empty());
        Self {
            data,
            word_idx: 0,
            buffer: 0,
            space_left_in_buffer: 64,
        }
    }

    #[inline]
    pub fn flush(&mut self) {
        let to_flush = 64 - self.space_left_in_buffer;
        if to_flush != 0 {
            let buffer = self.buffer << self.space_left_in_buffer;
            let old_word = u64::from_be(self.data[self.word_idx]);
            let clean_old_word = (old_word << to_flush) >> to_flush;
            let new_word = clean_old_word | buffer;
            self.data[self.word_idx] = new_word.to_be();
        }
    }

    #[inline]
    pub fn seek(&mut self, bits_idx: u32) {
        self.flush();
        self.word_idx = (bits_idx / 64) as usize;
        let idx_in_word = bits_idx % 64;
        self.space_left_in_buffer = 64 - idx_in_word as u8;
        let word = u64::from_be(self.data[self.word_idx]);
        if idx_in_word == 0 {
            self.buffer = 0;
        } else {
            self.buffer = word >> (64 - idx_in_word);
        }
    }

    #[inline(always)]
    pub fn tell(&self) -> u32 {
        (self.word_idx as u32 + 1) * 64 - u32::from(self.space_left_in_buffer)
    }

    #[inline(always)]
    pub(super) fn write_bits<U: Into<u64>>(&mut self, value: U, n_bits: u8) -> u8 {
        debug_assert!(n_bits <= 64);
        debug_assert!(self.space_left_in_buffer > 0);

        if n_bits == 0 {
            return 0;
        }

        let value = value.into();

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

        self.space_left_in_buffer += 64 - n_bits;
        self.buffer = value;
        n_bits
    }

    #[inline(always)]
    pub(super) fn write_unary<U: Into<u64>>(&mut self, value: U) -> u8 {
        debug_assert!(self.space_left_in_buffer > 0);
        let mut value = value.into();

        let code_length = (value + 1) as u8;

        if code_length <= self.space_left_in_buffer {
            self.space_left_in_buffer -= code_length;
            self.buffer = self.buffer << value << 1;
            self.buffer |= 1;
            if self.space_left_in_buffer == 0 {
                self.data[self.word_idx] = self.buffer.to_be();
                self.word_idx += 1;
                self.space_left_in_buffer = 64;
            }
            return code_length;
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
            self.space_left_in_buffer = 64 - (value as u8 + 1);
        }

        code_length
    }

    #[inline(always)]
    pub fn write_rice(
        &mut self,
        uniform_delta: u32,
        geometric_minus_one: u8,
        b1: u8,
    ) -> usize {
        usize::from(self.write_unary(uniform_delta >> b1))
            + usize::from(self.write_bits(uniform_delta, b1))
            + usize::from(self.write_unary(geometric_minus_one))
    }
}

#[cfg(test)]
mod testing_writer {
    use super::*;

    #[test]
    fn test_hand_picked_no_ops() {
        let expected = [
            0b01110111_11000010_11110100_00100011_11101100_10100100_11001010_01110110_u64.to_be(),
            0b11011101_00110110_10011010_00111110_10100000_11110101_01100101_01001010_u64.to_be(),
        ];
        let mut buffer: [u64; 2] = expected.clone();

        let mut writer = BitWriter::new(&mut buffer);
        writer.seek(13);
        drop(writer);

        assert_eq!(
            buffer,
            expected,
            "Buffer is not as expected. Buffer[0] Xor Expected: {:064b}",
            (buffer[0] ^ expected[0]).to_be()
        );
    }
}
