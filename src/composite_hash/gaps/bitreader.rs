#[derive(Debug, Clone, Copy)]
pub struct BitReader<'a> {
    data: &'a [u32],
    word_idx: usize,
    buffer: u64,
    bits_in_buffer: usize,
}

impl<'a> From<BitReader<'a>> for &'a [u32] {
    fn from(iter: BitReader<'a>) -> Self {
        iter.data
    }
}

impl<'a> From<BitReader<'a>> for &'a [u8] {
    #[allow(unsafe_code)]
    fn from(iter: BitReader<'a>) -> Self {
        unsafe {
            core::slice::from_raw_parts(
                iter.data.as_ptr() as *const u8,
                iter.data.len() * core::mem::size_of::<u32>(),
            )
        }
    }
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u32]) -> Self {
        Self {
            data,
            word_idx: 0,
            buffer: 0,
            bits_in_buffer: 0,
        }
    }

    /// Returns the position of the last bit that was read.
    pub fn last_read_bit_position(&self) -> usize {
        self.word_idx * 32 - self.bits_in_buffer
    }

    /// Returns the position of the last bit that has been positioned in the read buffer.
    pub fn last_buffered_bit_position(&self) -> usize {
        self.word_idx * 32 + self.bits_in_buffer
    }

    /// Returns the new word.
    fn new_word(&mut self) -> u32 {
        let new_word = self.data[self.word_idx];
        self.word_idx += 1;

        new_word.to_be()
    }

    #[inline]
    pub fn read_bits(&mut self, mut n_bits: usize) -> u64 {
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

        while n_bits > 32 {
            let new_word: u64 = u64::from(self.new_word());
            result = (result << 32) | new_word;
            n_bits -= 32;
        }

        debug_assert!(n_bits > 0);
        debug_assert!(n_bits <= 32);

        let new_word = self.new_word();
        self.bits_in_buffer = 32 - n_bits;
        let upcasted: u64 = u64::from(new_word);
        let final_bits: u64 = upcasted >> self.bits_in_buffer;
        result = (result << (n_bits - 1) << 1) | final_bits;
        self.buffer = (upcasted << (64 - self.bits_in_buffer - 1)) << 1;

        result
    }

    #[inline]
    pub fn read_unary(&mut self) -> u64 {
        debug_assert!(self.bits_in_buffer < 64);

        let zeros: usize = self.buffer.leading_zeros() as _;

        if zeros < self.bits_in_buffer {
            self.buffer = self.buffer << zeros << 1;
            self.bits_in_buffer -= zeros + 1;
            return zeros as u64;
        }

        let mut result: u64 = self.bits_in_buffer as _;

        loop {
            let new_word = self.new_word();

            if new_word != 0 {
                let zeros: usize = new_word.leading_zeros() as _;
                self.buffer = u64::from(new_word) << (32 + zeros) << 1;
                self.bits_in_buffer = 32 - zeros - 1;
                return result + zeros as u64;
            }
            result += 32 as u64;
        }
    }

    pub fn read_rice(&mut self, b: u8) -> u64 {
        (self.read_unary() << b) + self.read_bits(usize::from(b))
    }
}

/// Returns the number of bits required to encode a given value using a Rice code.
/// 
/// # Arguments
/// * `n` - The value to be encoded.
/// * `b` - The number of bits used to encode the remainder.
pub fn len_rice(n: u64, b: u8) -> usize {
    usize::try_from(n >> b).unwrap() + 1 + usize::from(b)
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
                positions[i] = writer.write_unary(value as u64);
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
                value,
                "Unary encoded value does not match the expected value."
            );
            assert_eq!(
                reader.last_read_bit_position(),
                position,
                "Unary encoded value does not match the expected value."
            );
        }
    }

    #[test]
    #[allow(unsafe_code)]
    fn test_read_write_rice() {
        let mut expected = [0u64; 1000];
        let mut buffer = [0u64; 2000];
        let maximum_value = u16::MAX;

        debug_assert!(
            len_rice(u64::from(maximum_value), 17) * expected.len() / 64 < buffer.len(),
            "The buffer len ({}) is not enough to store the upper bound of the encoded values ({}).",
            buffer.len(),
            len_rice(u64::from(maximum_value), 17) * expected.len() / 64
        );

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in
                iter_random_values::<u16>(1_000, Some(maximum_value), None).enumerate()
            {
                writer.write_rice(value as u64, 17);
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, buffer.len() * 2) };
        let mut reader: BitReader = BitReader::new(&transmuted_buffer);

        for value in expected {
            assert_eq!(
                reader.read_rice(17),
                value,
                "Rice encoded value does not match the expected value."
            );
        }
    }
}
