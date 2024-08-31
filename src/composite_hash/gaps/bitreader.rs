#[derive(Debug, Clone, Copy)]
pub struct BitReader<'a> {
    data: &'a [u32],
    word_idx: usize,
    buffer: u64,
    bits_in_buffer: usize,
}

impl<'a> From<BitReader<'a>> for &'a [u32] {
    #[inline(always)]
    fn from(iter: BitReader<'a>) -> Self {
        iter.data
    }
}

impl<'a> From<BitReader<'a>> for &'a [u8] {
    #[inline(always)]
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

    // #[inline]
    // pub fn skip_bits(&mut self, mut n_bits: usize) {
    //     debug_assert!(self.bits_in_buffer < 64);
    //     debug_assert!(
    //         self.data.len() > self.word_idx,
    //         "Reader in illegal state: data len is {}, but word index is {}.",
    //         self.data.len(),
    //         self.word_idx
    //     );

    //     if n_bits <= self.bits_in_buffer {
    //         self.bits_in_buffer -= n_bits;
    //         self.buffer <<= n_bits;
    //         return;
    //     }

    //     n_bits -= self.bits_in_buffer;

    //     while n_bits > 32 {
    //         self.word_idx += 1;
    //         n_bits -= 32;
    //     }

    //     let new_word = self.data[self.word_idx];
    //     self.word_idx += 1;
    //     self.bits_in_buffer = 32 - n_bits;

    //     self.buffer = (new_word as u64) << (64 - 1 - self.bits_in_buffer) << 1;
    // }

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

pub fn len_rice(n: u64, b: usize) -> usize {
    (n >> b) as usize + 1 + b
}

pub fn len_gamma(mut n: u64) -> usize {
    n += 1;
    let number_of_bits_to_write = n.ilog2();
    2 * number_of_bits_to_write as usize + 1
}

pub fn len_exp_golomb(n: u64, k: usize) -> usize {
    len_gamma(n >> k) + k
}

pub fn len_golomb(n: u64, b: u64) -> usize {
    (n / b) as usize + 1 + len_minimal_binary(n % b, b)
}

pub fn len_minimal_binary(n: u64, max: u64) -> usize {
    if max == 0 {
        return 0;
    }
    let l = max.ilog2();
    let limit = (1 << (l + 1)) - max;
    let mut result = l as usize;
    if n >= limit {
        result += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use core::u32;

    use super::super::bitwriter::BitWriter;
    use super::*;
    use crate::prelude::*;

    fn len_delta(n: u64) -> usize {
        let l = (n + 1).ilog2();
        l as usize + len_gamma(l as _)
    }

    fn len_pi(mut n: u64, k: u64) -> usize {
        n += 1; // π codes are indexed from 1
        let rem = n.ilog2() as usize;
        let h = 1 + rem;
        let l = h.div_ceil(1 << k);
        k as usize + l + rem
    }

    fn len_pi_web(n: u64, k: u64) -> usize {
        1 + if n == 0 { 0 } else { len_pi(n - 1, k) }
    }

    fn len_zeta(mut n: u64, k: u64) -> usize {
        n += 1;
        let h = n.ilog2() as u64 / k;
        let u = 1 << ((h + 1) * k);
        let l = 1 << (h * k);
        h as usize + 1 + len_minimal_binary(n - l, u - l)
    }

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
    fn test_read_write_minimal_binary() {
        let mut expected = [0u64; 1000];
        let mut buffer = [0u64; 2000];

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in iter_random_values::<u32>(1_000, None, None).enumerate() {
                writer.write_minimal_binary(value as u64, u32::MAX.into());
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, buffer.len() * 2) };
        let mut reader: BitReader = BitReader::new(&transmuted_buffer);

        for value in expected {
            assert_eq!(
                reader.read_minimal_binary(u32::MAX.into()),
                value,
                "Minimal binary encoded value does not match the expected value."
            );
        }
    }

    #[test]
    #[allow(unsafe_code)]
    fn test_read_write_gamma() {
        let mut expected = [0u64; 1000];
        let mut buffer = [0u64; 2000];

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in iter_random_values::<u32>(1_000, None, None).enumerate() {
                writer.write_gamma(value as u64);
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, buffer.len() * 2) };
        let mut reader: BitReader = BitReader::new(&transmuted_buffer);

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
        let mut expected = [0u64; 1000];
        let mut buffer = [0u64; 2000];

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in iter_random_values::<u32>(1_000, None, None).enumerate() {
                writer.write_delta(value as u64);
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, buffer.len() * 2) };
        let mut reader: BitReader = BitReader::new(&transmuted_buffer);

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
        let mut expected = [0u64; 100];
        let mut buffer = [0u64; 2850];

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in
                iter_random_values::<u16>(expected.len() as u64, None, None).enumerate()
            {
                writer.write_golomb(value as u64, 37);
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, buffer.len() * 2) };
        let mut reader: BitReader = BitReader::new(&transmuted_buffer);

        for value in expected {
            assert_eq!(
                reader.read_golomb(37),
                value,
                "Golomb encoded value does not match the expected value."
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

    #[test]
    #[allow(unsafe_code)]
    fn test_read_write_exp_golomb() {
        let mut expected = [0u64; 1000];
        let mut buffer = [0u64; 2000];
        let max_value = u32::MAX;

        debug_assert!(
            len_exp_golomb(u64::from(max_value), 17) * expected.len() / 64 < buffer.len(),
            "The buffer len ({}) is not enough to store the upper bound of the encoded values ({}).",
            buffer.len(),
            len_exp_golomb(u64::from(max_value), 17) * expected.len() / 64
        );

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in iter_random_values::<u32>(1_000, None, None).enumerate() {
                writer.write_exp_golomb(value as u64, 17);
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, buffer.len() * 2) };
        let mut reader: BitReader = BitReader::new(&transmuted_buffer);

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
        let mut expected = [0u64; 1000];
        let mut buffer = [0u64; 2000];

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in iter_random_values::<u32>(1_000, None, None).enumerate() {
                writer.write_zeta(value as u64, 17);
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, buffer.len() * 2) };
        let mut reader: BitReader = BitReader::new(&transmuted_buffer);

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
        let mut expected = [0u64; 1000];
        let mut buffer = [0u64; 2000];

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in iter_random_values::<u32>(1_000, None, None).enumerate() {
                writer.write_pi(value as u64, 17);
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, buffer.len() * 2) };
        let mut reader: BitReader = BitReader::new(&transmuted_buffer);

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
        let mut expected = [0u64; 1000];
        let mut buffer = [0u64; 2000];

        {
            let mut writer = BitWriter::new(&mut buffer);

            for (i, value) in iter_random_values::<u32>(1_000, None, None).enumerate() {
                writer.write_pi_web(value as u64, 17);
                expected[i] = value as u64;
            }
        }

        let transmuted_buffer =
            unsafe { core::slice::from_raw_parts(buffer.as_ptr() as *const u32, buffer.len() * 2) };
        let mut reader: BitReader = BitReader::new(&transmuted_buffer);

        for value in expected {
            assert_eq!(
                reader.read_pi_web(17),
                value,
                "Pi-Web encoded value does not match the expected value."
            );
        }
    }
}
