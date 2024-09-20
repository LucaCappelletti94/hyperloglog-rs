
#[inline]
/// Returns the bucketed index for the given cardinality estimate.
///
/// # Arguments
/// * `cardinality` - The cardinality estimate.
///
/// # Implementation details
/// The values are bucketed as follows:
///
/// 0..512 -> 0..512
/// 512..1024 -> 512..768
/// 1024..2048 -> 768..1024
/// 2048..4096 -> 1024..1280
/// 4096..8192 -> 1280..1536
/// 8192..16384 -> 1536..1792
/// 16384..32768 -> 1792..2048
/// 32768..65536 -> 2048..2304
/// 65536..131072 -> 2304..2560
/// 131072..262144 -> 2560..2816
///
/// and so on.
pub fn cardinality_estimate_to_index(cardinality: u64) -> usize {
    if cardinality <= 512 {
        return cardinality as usize;
    }

    let log2 = cardinality.next_power_of_two().trailing_zeros() as usize;

    (cardinality as usize - (1 << (log2 - 1))) / (1 << (log2 - 9))
        + 512
        + (256 as usize * (log2 - 10))
}

#[inline]
pub fn index_to_cardinality_estimate(index: usize) -> u64 {
    if index <= 512 {
        return index as u64;
    }

    let log2 = (index - 512) / 256 + 10;

    (1 << (log2 - 1)) + (index - 512 - 256 * (log2 - 10)) as u64 * (1 << (log2 - 9))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cardinality_estimate_to_index() {
        for i in 0..512 {
            assert_eq!(i as usize, cardinality_estimate_to_index(i));
        }

        for i in 512..1024 {
            assert_eq!(512 + (i - 512) / 2, cardinality_estimate_to_index(i as u64));
        }

        for i in 1024..2048 {
            assert_eq!(
                768 + (i - 1024) / 4,
                cardinality_estimate_to_index(i as u64)
            );
        }

        for i in 2048..4096 {
            assert_eq!(
                1024 + (i - 2048) / 8,
                cardinality_estimate_to_index(i as u64)
            );
        }

        for i in 4096..8192 {
            assert_eq!(
                1280 + (i - 4096) / 16,
                cardinality_estimate_to_index(i as u64)
            );
        }

        for i in 8192..16384 {
            assert_eq!(
                1536 + (i - 8192) / 32,
                cardinality_estimate_to_index(i as u64)
            );
        }

        for i in 16384..32768 {
            assert_eq!(
                1792 + (i - 16384) / 64,
                cardinality_estimate_to_index(i as u64)
            );
        }
    }

    #[test]
    fn test_index_to_cardinality_estimate() {
        for i in 0..512 {
            assert_eq!(i as u64, index_to_cardinality_estimate(i));
        }

        for i in 512..768 {
            assert_eq!(512 + (i - 512) as u64 * 2, index_to_cardinality_estimate(i));
        }

        for i in 768..1024 {
            assert_eq!(1024 + (i - 768) as u64 * 4, index_to_cardinality_estimate(i));
        }

        for i in 1024..1280 {
            assert_eq!(2048 + (i - 1024) as u64 * 8, index_to_cardinality_estimate(i));
        }
    }

    #[test]
    fn test_cardinality_estimate_to_index_to_cardinality_estimate() {
        for i in 0..1000 {
            assert_eq!(i, cardinality_estimate_to_index(index_to_cardinality_estimate(i)));
        }
    }
}
