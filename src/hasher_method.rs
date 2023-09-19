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

impl HasherMethod for SipHasher13 {
    fn hash<H: Hash>(key: &H) -> u64 {
        let mut hasher = SipHasher13::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
}

impl HasherMethod for SipHasher24 {
    fn hash<H: Hash>(key: &H) -> u64 {
        let mut hasher = SipHasher24::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
}

impl HasherMethod for HighwayHasher {
    fn hash<H: Hash>(key: &H) -> u64 {
        let mut hasher = HighwayHasher::default();
        key.hash(&mut hasher);
        hasher.finalize64()
    }
}

impl HasherMethod for MetroHasher {
    fn hash<H: Hash>(key: &H) -> u64 {
        let mut hasher = MetroHasher::default();
        key.hash(&mut hasher);
        hasher.finish()
    }
}
