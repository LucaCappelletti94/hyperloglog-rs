#[derive(Debug, Clone, Copy)]
pub struct BitReader<'a> {
    data: &'a [u32],
    word_idx: usize,
    buffer: u64,
    bits_in_buffer: usize,
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

    pub fn seek(&mut self, bit_idx: usize) {
        self.word_idx = bit_idx / 32;
        let in_word_idx = bit_idx % 32;
        self.buffer = (self.data[self.word_idx] << in_word_idx) as u64;
        self.bits_in_buffer = 32 - in_word_idx;
    }

    pub fn tell(&self) -> usize {
        self.word_idx * 32 + self.bits_in_buffer
    }

    #[inline]
    pub fn read_bits(&mut self, mut n_bits: usize) -> u64 {
        debug_assert!(n_bits <= 64);
        debug_assert!(self.bits_in_buffer < 64);
        debug_assert!(
            self.data.len() > self.word_idx,
            "Reader in illegal state: data len is {}, but word index is {}.",
            self.data.len(),
            self.word_idx
        );

        if n_bits <= self.bits_in_buffer {
            let result = self.buffer >> (64 - n_bits - 1) >> 1;
            self.bits_in_buffer -= n_bits;
            self.buffer <<= n_bits;
            return result;
        }

        let mut result: u64 = self.buffer >> (64 - 1 - self.bits_in_buffer) >> 1_u8;
        n_bits -= self.bits_in_buffer;

        while n_bits > 32 {
            let new_word: u64 = self.data[self.word_idx].to_be() as u64;
            self.word_idx += 1;
            result = (result << 32) | new_word;
            n_bits -= 32;
        }

        debug_assert!(n_bits > 0);
        debug_assert!(n_bits <= 32);

        let new_word = self.data[self.word_idx].to_be();
        self.word_idx += 1;
        self.bits_in_buffer = 32 - n_bits;
        let upcasted: u64 = new_word as u64;
        let final_bits: u64 = upcasted >> self.bits_in_buffer;
        result = (result << (n_bits - 1) << 1) | final_bits;
        self.buffer = ((new_word as u64) << (64 - self.bits_in_buffer - 1)) << 1;

        result
    }

    #[inline]
    pub fn read_unary(&mut self) -> u64 {
        debug_assert!(self.bits_in_buffer < 64);
        debug_assert!(
            self.data.len() > self.word_idx,
            "Reader in illegal state: data len is {}, but word index is {}.",
            self.data.len(),
            self.word_idx
        );

        let zeros: usize = self.buffer.leading_zeros() as _;

        if zeros < self.bits_in_buffer {
            self.buffer = self.buffer << zeros << 1;
            self.bits_in_buffer -= zeros + 1;
            return zeros as u64;
        }

        let mut result: u64 = self.bits_in_buffer as _;

        debug_assert!(
            self.data[self.word_idx..].iter().any(|word| *word!=0),
            "At least a word after the index {}/{} should be different from zero, otherwise the subsequent loop will go out of bounds. Got: {:?}.",
            self.word_idx,
            self.data.len(),
            &self.data
        );

        loop {
            let new_word = self.data[self.word_idx].to_be();
            self.word_idx += 1;

            if new_word != 0 {
                let zeros: usize = new_word.leading_zeros() as _;
                self.buffer = (new_word as u64) << (32 + zeros) << 1;
                self.bits_in_buffer = 32 - zeros - 1;
                return result + zeros as u64;
            }
            result += 32 as u64;
        }
    }

    #[inline]
    pub fn skip_bits(&mut self, mut n_bits: usize) {
        debug_assert!(self.bits_in_buffer < 64);
        debug_assert!(
            self.data.len() > self.word_idx,
            "Reader in illegal state: data len is {}, but word index is {}.",
            self.data.len(),
            self.word_idx
        );

        if n_bits <= self.bits_in_buffer {
            self.bits_in_buffer -= n_bits;
            self.buffer <<= n_bits;
            return;
        }

        n_bits -= self.bits_in_buffer;

        while n_bits > 32 {
            self.word_idx += 1;
            n_bits -= 32;
        }

        let new_word = self.data[self.word_idx];
        self.word_idx += 1;
        self.bits_in_buffer = 32 - n_bits;

        self.buffer = (new_word as u64) << (64 - 1 - self.bits_in_buffer) << 1;
    }

    pub fn read_minimal_binary(&mut self, max: u64) -> u64 {
        let l = max.ilog2();
        let mut prefix = self.read_bits(l as _);
        let limit = (1 << (l + 1)) - max;

        if prefix < limit {
            prefix
        } else {
            prefix <<= 1;
            prefix |= self.read_bits(1);
            prefix - limit
        }
    }

    pub fn read_gamma(&mut self) -> u64 {
        let len = self.read_unary();
        self.read_bits(len as usize) + (1 << len) - 1
    }

    pub fn read_delta(&mut self) -> u64 {
        let len = self.read_gamma();
        self.read_bits(len as usize) + (1 << len) - 1
    }

    pub fn read_golomb(&mut self, b: usize) -> u64 {
        self.read_unary() * b as u64 + self.read_minimal_binary(b as u64)
    }

    pub fn read_rice(&mut self, b: usize) -> u64 {
        (self.read_unary() << b) + self.read_bits(b)
    }

    pub fn read_exp_golomb(&mut self, b: usize) -> u64 {
        (self.read_gamma() << b) + self.read_bits(b)
    }

    pub fn read_zeta(&mut self, k: usize) -> u64 {
        let h = self.read_unary();
        let u = 1 << ((h + 1) * k as u64);
        let l = 1 << (h * k as u64);
        let res = self.read_minimal_binary(u - l);
        l + res - 1
    }

    pub fn read_pi(&mut self, k: usize) -> u64 {
        let l = self.read_unary() + 1;
        let v = self.read_bits(k);
        let h = l * (1 << k) - v;
        let r = h - 1;
        let rem = self.read_bits(r as usize);
        (1 << r) + rem - 1
    }

    pub fn read_pi_web(&mut self, k: usize) -> u64 {
        if self.read_bits(1) == 1 {
            0
        } else {
            self.read_pi(k) + 1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::bitwriter::BitWriter;
    use super::*;
    use crate::prelude::*;

    #[test]
    #[allow(unsafe_code)]
    fn test_read_write_unary() {
        let mut expected = [0u64; 6];
        let mut buffer = [0u64; 16];

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in iter_random_values::<u8>(6 as u64, None, None).enumerate() {
                writer.write_unary(value as u64);
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, 32) };
        let mut reader = BitReader::new(&transmuted_buffer);

        for value in expected {
            assert_eq!(
                reader.read_unary(),
                value,
                "Unary encoded value does not match the expected value."
            );
        }
    }

    #[test]
    #[allow(unsafe_code)]
    fn test_read_write_minimal_binary() {
        let mut expected = [0u64; 6];
        let mut buffer = [0u64; 16];

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in iter_random_values::<u8>(6 as u64, None, None).enumerate() {
                writer.write_minimal_binary(value as u64, 255);
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, 32) };
        let mut reader = BitReader::new(&transmuted_buffer);

        for value in expected {
            assert_eq!(
                reader.read_minimal_binary(255),
                value,
                "Minimal binary encoded value does not match the expected value."
            );
        }
    }

    #[test]
    #[allow(unsafe_code)]
    fn test_read_write_gamma() {
        let mut expected = [0u64; 6];
        let mut buffer = [0u64; 16];

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in iter_random_values::<u8>(6 as u64, None, None).enumerate() {
                writer.write_gamma(value as u64);
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, 32) };
        let mut reader = BitReader::new(&transmuted_buffer);

        for value in expected {
            assert_eq!(
                reader.read_gamma(),
                value,
                "Gamma encoded value does not match the expected value."
            );
        }
    }

    #[test]
    #[allow(unsafe_code)]
    fn test_read_write_delta() {
        let mut expected = [0u64; 6];
        let mut buffer = [0u64; 16];

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in iter_random_values::<u8>(6 as u64, None, None).enumerate() {
                writer.write_delta(value as u64);
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, 32) };
        let mut reader = BitReader::new(&transmuted_buffer);

        for value in expected {
            assert_eq!(
                reader.read_delta(),
                value,
                "Delta encoded value does not match the expected value."
            );
        }
    }

    #[test]
    #[allow(unsafe_code)]
    fn test_read_write_golomb() {
        let mut expected = [0u64; 6];
        let mut buffer = [0u64; 16];

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in iter_random_values::<u8>(6 as u64, None, None).enumerate() {
                writer.write_golomb(value as u64, 17);
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, 32) };
        let mut reader = BitReader::new(&transmuted_buffer);

        for value in expected {
            assert_eq!(
                reader.read_golomb(17),
                value,
                "Golomb encoded value does not match the expected value."
            );
        }
    }

    #[test]
    #[allow(unsafe_code)]
    fn test_read_write_rice() {
        let mut expected = [0u64; 6];
        let mut buffer = [0u64; 16];

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in iter_random_values::<u8>(6 as u64, None, None).enumerate() {
                writer.write_rice(value as u64, 17);
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, 32) };
        let mut reader = BitReader::new(&transmuted_buffer);

        for value in expected {
            assert_eq!(
                reader.read_rice(17),
                value,
                "Rice encoded value does not match the expected value."
            );
        }
    }

    #[test]
    #[allow(unsafe_code)]
    fn test_read_write_exp_golomb() {
        let mut expected = [0u64; 6];
        let mut buffer = [0u64; 16];

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in iter_random_values::<u8>(6 as u64, None, None).enumerate() {
                writer.write_exp_golomb(value as u64, 17);
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, 32) };
        let mut reader = BitReader::new(&transmuted_buffer);

        for value in expected {
            assert_eq!(
                reader.read_exp_golomb(17),
                value,
                "Exponential Golomb encoded value does not match the expected value."
            );
        }
    }

    #[test]
    #[allow(unsafe_code)]
    fn test_read_write_zeta() {
        let mut expected = [0u64; 6];
        let mut buffer = [0u64; 16];

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in iter_random_values::<u8>(6 as u64, None, None).enumerate() {
                writer.write_zeta(value as u64, 17);
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, 32) };
        let mut reader = BitReader::new(&transmuted_buffer);

        for value in expected {
            assert_eq!(
                reader.read_zeta(17),
                value,
                "Zeta encoded value does not match the expected value."
            );
        }
    }

    #[test]
    #[allow(unsafe_code)]
    fn test_read_write_pi() {
        let mut expected = [0u64; 6];
        let mut buffer = [0u64; 16];

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in iter_random_values::<u8>(6 as u64, None, None).enumerate() {
                writer.write_pi(value as u64, 17);
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, 32) };
        let mut reader = BitReader::new(&transmuted_buffer);

        for value in expected {
            assert_eq!(
                reader.read_pi(17),
                value,
                "Pi encoded value does not match the expected value."
            );
        }
    }

    #[test]
    #[allow(unsafe_code)]
    fn test_read_write_pi_web() {
        let mut expected = [0u64; 6];
        let mut buffer = [0u64; 16];

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in iter_random_values::<u8>(6 as u64, None, None).enumerate() {
                writer.write_pi_web(value as u64, 17);
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, 32) };
        let mut reader = BitReader::new(&transmuted_buffer);

        for value in expected {
            assert_eq!(
                reader.read_pi_web(17),
                value,
                "Pi-Web encoded value does not match the expected value."
            );
        }
    }
}
