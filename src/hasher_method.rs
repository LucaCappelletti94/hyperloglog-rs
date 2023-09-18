use core::hash::{Hash, Hasher};

use fasthash::MetroHasher;
use highway::{HighwayHash, HighwayHasher};
use siphasher::sip::{SipHasher13, SipHasher24};

pub trait HasherMethod {
    /// Hashes the given key.
    ///
    /// # Arguments
    /// * `key` - The key to hash.
    ///
    /// # Returns
    /// The hash of the given key.
    ///
    fn hash<H: Hash>(key: &H) -> u64;
}

#[derive(Default, Clone, Copy, Debug)]
pub struct DoubleSipHasher13 {
    hasher: SipHasher13,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct DoubleSipHasher24 {
    hasher: SipHasher24,
}

#[derive(Default, Clone, Debug)]
pub struct DoubleHighwayHasher {
    hasher: HighwayHasher,
}

#[derive(Default, Clone, Debug)]
pub struct DoubleMetroHasher {
    hasher: MetroHasher,
}

impl HasherMethod for SipHasher13 {
    fn hash<H: Hash>(key: &H) -> u64 {
        let mut hasher = SipHasher13::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
}

impl HasherMethod for DoubleSipHasher13 {
    fn hash<H: Hash>(key: &H) -> u64 {
        let mut hasher = DoubleSipHasher13::default();
        key.hash(&mut hasher.hasher);
        hasher.hasher.finish()
    }
}

impl HasherMethod for SipHasher24 {
    fn hash<H: Hash>(key: &H) -> u64 {
        let mut hasher = SipHasher24::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
}

impl HasherMethod for DoubleSipHasher24 {
    fn hash<H: Hash>(key: &H) -> u64 {
        let mut hasher = DoubleSipHasher24::default();
        key.hash(&mut hasher.hasher);
        hasher.hasher.finish()
    }
}

impl HasherMethod for HighwayHasher {
    fn hash<H: Hash>(key: &H) -> u64 {
        let mut hasher = HighwayHasher::default();
        key.hash(&mut hasher);
        hasher.finalize64()
    }
}

impl HasherMethod for DoubleHighwayHasher {
    fn hash<H: Hash>(key: &H) -> u64 {
        let mut hasher = DoubleHighwayHasher::default();
        key.hash(&mut hasher.hasher);
        hasher.hasher.finalize64()
    }
}

impl HasherMethod for MetroHasher {
    fn hash<H: Hash>(key: &H) -> u64 {
        let mut hasher = MetroHasher::default();
        key.hash(&mut hasher);
        hasher.finish()
    }
}

impl HasherMethod for DoubleMetroHasher {
    fn hash<H: Hash>(key: &H) -> u64 {
        let mut hasher = DoubleMetroHasher::default();
        key.hash(&mut hasher.hasher);
        hasher.hasher.finish()
    }
}
