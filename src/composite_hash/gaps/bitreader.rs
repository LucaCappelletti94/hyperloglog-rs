#[derive(Debug, Clone)]
pub(super) struct BitReader<'a> {
    data: core::slice::Iter<'a, u32>,
    word_idx: usize,
    buffer: u64,
    bits_in_buffer: usize,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u32]) -> Self {
        Self {
            data: data.iter(),
            word_idx: 0,
            buffer: 0,
            bits_in_buffer: 0,
        }
    }

    #[inline]
    /// Returns the position of the last bit that was read.
    pub fn last_read_bit_position(&self) -> usize {
        self.word_idx * 32 - self.bits_in_buffer
    }

    #[inline]
    /// Returns the position of the last bit that has been positioned in the read buffer.
    pub fn last_buffered_bit_position(&self) -> usize {
        self.word_idx * 32 + self.bits_in_buffer
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
            let new_word = self.data.next().unwrap().to_be();
            self.word_idx += 1; 

            if new_word != 0 {
                let zeros: usize = new_word.leading_zeros() as _;
                self.buffer = u64::from(new_word) << (32 + zeros) << 1;
                self.bits_in_buffer = 32 - zeros - 1;
                return result + zeros as u64;
            }
            result += 32u64;
        }
    }

    #[inline]
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
}
