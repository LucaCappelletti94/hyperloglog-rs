#[derive(Debug, Clone)]
pub(super) struct BitReader<'a> {
    data: core::slice::Iter<'a, u32>,
    word_idx: u32,
    buffer: u64,
    bits_in_buffer: u8,
}

impl<'a> BitReader<'a> {
    #[inline]
    pub fn new(data: &'a [u32]) -> Self {
        Self {
            data: data.iter(),
            word_idx: 0,
            buffer: 0,
            bits_in_buffer: 0,
        }
    }

    #[inline]
    /// Creates a new BitReader skipping ahead `bit_index` bits.
    pub fn skip(mut data: &'a [u32], bit_index: u32) -> Self {
        data = &data[bit_index as usize / 32..];
        let mut reader = Self::new(data);
        reader.word_idx = bit_index / 32;
        reader.read_bits((bit_index % 32) as u8);
        reader
    }

    #[inline]
    /// Returns the position of the last bit that was read.
    pub fn last_read_bit_position(&self) -> u32 {
        self.word_idx * 32 - u32::from(self.bits_in_buffer)
    }

    #[inline]
    /// Returns the position of the last bit that has been positioned in the read buffer.
    pub fn last_buffered_bit_position(&self) -> u32 {
        self.word_idx * 32 + u32::from(self.bits_in_buffer)
    }

    #[inline]
    pub fn read_bits(&mut self, mut n_bits: u8) -> u64 {
        debug_assert!(n_bits <= 64);
        debug_assert!(self.bits_in_buffer < 64);

        if n_bits <= self.bits_in_buffer {
            let result = self.buffer >> (64 - n_bits - 1) >> 1;
            self.bits_in_buffer -= n_bits;
            self.buffer <<= n_bits;
            return result;
        }

        let mut result: u64 = self.buffer >> (64 - 1 - self.bits_in_buffer) >> 1_u8;
        n_bits -= self.bits_in_buffer;

        if n_bits > 32 {
            let new_word = u64::from(self.data.next().unwrap().to_be());
            self.word_idx += 1;
            result = (result << 32) | new_word;
            n_bits -= 32;
        }

        debug_assert!(n_bits > 0);
        debug_assert!(n_bits <= 32);

        let new_word = self.data.next().unwrap().to_be();
        self.word_idx += 1;
        self.bits_in_buffer = 32 - n_bits;
        let upcasted: u64 = u64::from(new_word);
        let final_bits: u64 = upcasted >> self.bits_in_buffer;
        result = (result << (n_bits - 1) << 1) | final_bits;
        self.buffer = (upcasted << (64 - self.bits_in_buffer - 1)) << 1;

        result
    }

    #[inline]
    pub fn read_unary(&mut self) -> u8 {
        debug_assert!(self.bits_in_buffer < 64);

        let zeros = self.buffer.leading_zeros() as u8;

        if zeros < self.bits_in_buffer {
            self.buffer = self.buffer << zeros << 1;
            self.bits_in_buffer -= zeros + 1;
            return zeros;
        }

        let mut result = self.bits_in_buffer;

        loop {
            let new_word = self.data.next().unwrap().to_be();
            self.word_idx += 1;

            if new_word != 0 {
                let zeros = new_word.leading_zeros() as u8;
                self.buffer = u64::from(new_word) << (32 + zeros) << 1;
                self.bits_in_buffer = 32 - zeros - 1;
                return result + zeros;
            }
            result += 32u8;
        }
    }

    #[inline]
    pub fn read_rice(&mut self, b: u8) -> u32 {
        (u32::from(self.read_unary()) << b) + self.read_bits(b) as u32
    }
}

#[inline]
/// Returns the number of bits required to encode a given value using a Rice code.
///
/// # Arguments
/// * `uniform` - The uniform value to encode.
/// * `b1` - The rice coefficient to use for the uniform value.
/// * `geometric` - The geometric value to encode.
pub fn len_rice(uniform_delta: u32, b1: u8, geometric_minus_one: u8, b2: u8) -> u32 {
    (uniform_delta >> b1) + 1 + u32::from(b1) + u32::from(geometric_minus_one >> b2) + 1 + u32::from(b2)
}

#[cfg(test)]
mod tests {
    use core::u32;

    use super::super::bitwriter::BitWriter;
    use super::*;
    use crate::prelude::*;

    #[test]
    #[allow(unsafe_code)]
    fn test_read_write_unary() {
        let mut positions = [0usize; 1000];
        let mut expected = [0u64; 1000];
        let mut buffer = [0u64; 2000];

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in iter_random_values::<u8>(1_000, Some(127), None).enumerate() {
                positions[i] = writer.write_unary(value) as usize;
                if i > 0 {
                    positions[i] += positions[i - 1];
                }
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, buffer.len() * 2) };
        let mut reader: BitReader = BitReader::new(&transmuted_buffer);

        for (value, position) in expected.into_iter().zip(positions) {
            assert_eq!(
                reader.read_unary(),
                value as u8,
                "Unary encoded value does not match the expected value."
            );
            assert_eq!(
                reader.last_read_bit_position() as usize,
                position,
                "Unary encoded value does not match the expected value."
            );
        }
    }
}
