// Tue Jan 15 2026 - Alex

use std::fmt;

/// Compute various hash digests for data
pub struct HashComputer;

impl HashComputer {
    /// Compute CRC32 hash
    pub fn crc32(data: &[u8]) -> u32 {
        let mut crc: u32 = 0xFFFFFFFF;
        
        for byte in data {
            let index = ((crc ^ (*byte as u32)) & 0xFF) as usize;
            crc = CRC32_TABLE[index] ^ (crc >> 8);
        }
        
        !crc
    }

    /// Compute Adler32 checksum
    pub fn adler32(data: &[u8]) -> u32 {
        const MOD_ADLER: u32 = 65521;
        let mut a: u32 = 1;
        let mut b: u32 = 0;

        for byte in data {
            a = (a + *byte as u32) % MOD_ADLER;
            b = (b + a) % MOD_ADLER;
        }

        (b << 16) | a
    }

    /// Compute FNV-1a hash (32-bit)
    pub fn fnv1a_32(data: &[u8]) -> u32 {
        const FNV_PRIME: u32 = 0x01000193;
        const FNV_OFFSET: u32 = 0x811c9dc5;

        let mut hash = FNV_OFFSET;
        for byte in data {
            hash ^= *byte as u32;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
    }

    /// Compute FNV-1a hash (64-bit)
    pub fn fnv1a_64(data: &[u8]) -> u64 {
        const FNV_PRIME: u64 = 0x00000100000001B3;
        const FNV_OFFSET: u64 = 0xcbf29ce484222325;

        let mut hash = FNV_OFFSET;
        for byte in data {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
    }

    /// Compute xxHash32
    pub fn xxhash32(data: &[u8], seed: u32) -> u32 {
        const PRIME1: u32 = 0x9E3779B1;
        const PRIME2: u32 = 0x85EBCA77;
        const PRIME3: u32 = 0xC2B2AE3D;
        const PRIME4: u32 = 0x27D4EB2F;
        const PRIME5: u32 = 0x165667B1;

        let len = data.len();
        let mut h32: u32;
        let mut p = 0;

        if len >= 16 {
            let limit = len - 15;
            let mut v1 = seed.wrapping_add(PRIME1).wrapping_add(PRIME2);
            let mut v2 = seed.wrapping_add(PRIME2);
            let mut v3 = seed;
            let mut v4 = seed.wrapping_sub(PRIME1);

            while p < limit {
                v1 = round32(v1, read_u32_le(&data[p..]));
                p += 4;
                v2 = round32(v2, read_u32_le(&data[p..]));
                p += 4;
                v3 = round32(v3, read_u32_le(&data[p..]));
                p += 4;
                v4 = round32(v4, read_u32_le(&data[p..]));
                p += 4;
            }

            h32 = v1.rotate_left(1)
                .wrapping_add(v2.rotate_left(7))
                .wrapping_add(v3.rotate_left(12))
                .wrapping_add(v4.rotate_left(18));
        } else {
            h32 = seed.wrapping_add(PRIME5);
        }

        h32 = h32.wrapping_add(len as u32);

        while p + 4 <= len {
            h32 = h32.wrapping_add(read_u32_le(&data[p..]).wrapping_mul(PRIME3));
            h32 = h32.rotate_left(17).wrapping_mul(PRIME4);
            p += 4;
        }

        while p < len {
            h32 = h32.wrapping_add((data[p] as u32).wrapping_mul(PRIME5));
            h32 = h32.rotate_left(11).wrapping_mul(PRIME1);
            p += 1;
        }

        h32 ^= h32 >> 15;
        h32 = h32.wrapping_mul(PRIME2);
        h32 ^= h32 >> 13;
        h32 = h32.wrapping_mul(PRIME3);
        h32 ^= h32 >> 16;

        h32
    }

    /// Compute MurmurHash3 (32-bit)
    pub fn murmur3_32(data: &[u8], seed: u32) -> u32 {
        const C1: u32 = 0xcc9e2d51;
        const C2: u32 = 0x1b873593;
        const R1: u32 = 15;
        const R2: u32 = 13;
        const M: u32 = 5;
        const N: u32 = 0xe6546b64;

        let len = data.len();
        let nblocks = len / 4;
        let mut hash = seed;

        // Body
        for i in 0..nblocks {
            let block_start = i * 4;
            let mut k = read_u32_le(&data[block_start..]);

            k = k.wrapping_mul(C1);
            k = k.rotate_left(R1);
            k = k.wrapping_mul(C2);

            hash ^= k;
            hash = hash.rotate_left(R2);
            hash = hash.wrapping_mul(M).wrapping_add(N);
        }

        // Tail
        let tail_start = nblocks * 4;
        let mut k1: u32 = 0;

        match len & 3 {
            3 => {
                k1 ^= (data[tail_start + 2] as u32) << 16;
                k1 ^= (data[tail_start + 1] as u32) << 8;
                k1 ^= data[tail_start] as u32;
                k1 = k1.wrapping_mul(C1);
                k1 = k1.rotate_left(R1);
                k1 = k1.wrapping_mul(C2);
                hash ^= k1;
            }
            2 => {
                k1 ^= (data[tail_start + 1] as u32) << 8;
                k1 ^= data[tail_start] as u32;
                k1 = k1.wrapping_mul(C1);
                k1 = k1.rotate_left(R1);
                k1 = k1.wrapping_mul(C2);
                hash ^= k1;
            }
            1 => {
                k1 ^= data[tail_start] as u32;
                k1 = k1.wrapping_mul(C1);
                k1 = k1.rotate_left(R1);
                k1 = k1.wrapping_mul(C2);
                hash ^= k1;
            }
            _ => {}
        }

        // Finalization
        hash ^= len as u32;
        hash = fmix32(hash);

        hash
    }

    /// Compute DJB2 hash
    pub fn djb2(data: &[u8]) -> u64 {
        let mut hash: u64 = 5381;
        for byte in data {
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(*byte as u64);
        }
        hash
    }

    /// Compute SDBM hash
    pub fn sdbm(data: &[u8]) -> u64 {
        let mut hash: u64 = 0;
        for byte in data {
            hash = (*byte as u64)
                .wrapping_add(hash << 6)
                .wrapping_add(hash << 16)
                .wrapping_sub(hash);
        }
        hash
    }

    /// Compute polynomial rolling hash
    pub fn rolling_hash(data: &[u8], base: u64, modulo: u64) -> u64 {
        let mut hash: u64 = 0;
        for byte in data {
            hash = (hash.wrapping_mul(base) % modulo).wrapping_add(*byte as u64) % modulo;
        }
        hash
    }

    /// Compute all hashes
    pub fn compute_all(data: &[u8]) -> HashResults {
        HashResults {
            crc32: Self::crc32(data),
            adler32: Self::adler32(data),
            fnv1a_32: Self::fnv1a_32(data),
            fnv1a_64: Self::fnv1a_64(data),
            xxhash32: Self::xxhash32(data, 0),
            murmur3_32: Self::murmur3_32(data, 0),
            djb2: Self::djb2(data),
        }
    }
}

/// Hash results collection
#[derive(Debug, Clone)]
pub struct HashResults {
    pub crc32: u32,
    pub adler32: u32,
    pub fnv1a_32: u32,
    pub fnv1a_64: u64,
    pub xxhash32: u32,
    pub murmur3_32: u32,
    pub djb2: u64,
}

impl fmt::Display for HashResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "CRC32:      {:08X}", self.crc32)?;
        writeln!(f, "Adler32:    {:08X}", self.adler32)?;
        writeln!(f, "FNV1a-32:   {:08X}", self.fnv1a_32)?;
        writeln!(f, "FNV1a-64:   {:016X}", self.fnv1a_64)?;
        writeln!(f, "xxHash32:   {:08X}", self.xxhash32)?;
        writeln!(f, "Murmur3-32: {:08X}", self.murmur3_32)?;
        writeln!(f, "DJB2:       {:016X}", self.djb2)?;
        Ok(())
    }
}

// Helper functions
fn round32(acc: u32, input: u32) -> u32 {
    const PRIME1: u32 = 0x9E3779B1;
    const PRIME2: u32 = 0x85EBCA77;
    acc.wrapping_add(input.wrapping_mul(PRIME2))
        .rotate_left(13)
        .wrapping_mul(PRIME1)
}

fn fmix32(mut h: u32) -> u32 {
    h ^= h >> 16;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> 16;
    h
}

fn read_u32_le(data: &[u8]) -> u32 {
    u32::from_le_bytes([data[0], data[1], data[2], data[3]])
}

/// CRC32 lookup table
static CRC32_TABLE: [u32; 256] = [
    0x00000000, 0x77073096, 0xEE0E612C, 0x990951BA,
    0x076DC419, 0x706AF48F, 0xE963A535, 0x9E6495A3,
    0x0EDB8832, 0x79DCB8A4, 0xE0D5E91E, 0x97D2D988,
    0x09B64C2B, 0x7EB17CBD, 0xE7B82D07, 0x90BF1D91,
    0x1DB71064, 0x6AB020F2, 0xF3B97148, 0x84BE41DE,
    0x1ADAD47D, 0x6DDDE4EB, 0xF4D4B551, 0x83D385C7,
    0x136C9856, 0x646BA8C0, 0xFD62F97A, 0x8A65C9EC,
    0x14015C4F, 0x63066CD9, 0xFA0F3D63, 0x8D080DF5,
    0x3B6E20C8, 0x4C69105E, 0xD56041E4, 0xA2677172,
    0x3C03E4D1, 0x4B04D447, 0xD20D85FD, 0xA50AB56B,
    0x35B5A8FA, 0x42B2986C, 0xDBBBC9D6, 0xACBCF940,
    0x32D86CE3, 0x45DF5C75, 0xDCD60DCF, 0xABD13D59,
    0x26D930AC, 0x51DE003A, 0xC8D75180, 0xBFD06116,
    0x21B4F4B5, 0x56B3C423, 0xCFBA9599, 0xB8BDA50F,
    0x2802B89E, 0x5F058808, 0xC60CD9B2, 0xB10BE924,
    0x2F6F7C87, 0x58684C11, 0xC1611DAB, 0xB6662D3D,
    0x76DC4190, 0x01DB7106, 0x98D220BC, 0xEFD5102A,
    0x71B18589, 0x06B6B51F, 0x9FBFE4A5, 0xE8B8D433,
    0x7807C9A2, 0x0F00F934, 0x9609A88E, 0xE10E9818,
    0x7F6A0DBB, 0x086D3D2D, 0x91646C97, 0xE6635C01,
    0x6B6B51F4, 0x1C6C6162, 0x856530D8, 0xF262004E,
    0x6C0695ED, 0x1B01A57B, 0x8208F4C1, 0xF50FC457,
    0x65B0D9C6, 0x12B7E950, 0x8BBEB8EA, 0xFCB9887C,
    0x62DD1DDF, 0x15DA2D49, 0x8CD37CF3, 0xFBD44C65,
    0x4DB26158, 0x3AB551CE, 0xA3BC0074, 0xD4BB30E2,
    0x4ADFA541, 0x3DD895D7, 0xA4D1C46D, 0xD3D6F4FB,
    0x4369E96A, 0x346ED9FC, 0xAD678846, 0xDA60B8D0,
    0x44042D73, 0x33031DE5, 0xAA0A4C5F, 0xDD0D7CC9,
    0x5005713C, 0x270241AA, 0xBE0B1010, 0xC90C2086,
    0x5768B525, 0x206F85B3, 0xB966D409, 0xCE61E49F,
    0x5EDEF90E, 0x29D9C998, 0xB0D09822, 0xC7D7A8B4,
    0x59B33D17, 0x2EB40D81, 0xB7BD5C3B, 0xC0BA6CAD,
    0xEDB88320, 0x9ABFB3B6, 0x03B6E20C, 0x74B1D29A,
    0xEAD54739, 0x9DD277AF, 0x04DB2615, 0x73DC1683,
    0xE3630B12, 0x94643B84, 0x0D6D6A3E, 0x7A6A5AA8,
    0xE40ECF0B, 0x9309FF9D, 0x0A00AE27, 0x7D079EB1,
    0xF00F9344, 0x8708A3D2, 0x1E01F268, 0x6906C2FE,
    0xF762575D, 0x806567CB, 0x196C3671, 0x6E6B06E7,
    0xFED41B76, 0x89D32BE0, 0x10DA7A5A, 0x67DD4ACC,
    0xF9B9DF6F, 0x8EBEEFF9, 0x17B7BE43, 0x60B08ED5,
    0xD6D6A3E8, 0xA1D1937E, 0x38D8C2C4, 0x4FDFF252,
    0xD1BB67F1, 0xA6BC5767, 0x3FB506DD, 0x48B2364B,
    0xD80D2BDA, 0xAF0A1B4C, 0x36034AF6, 0x41047A60,
    0xDF60EFC3, 0xA867DF55, 0x316E8EEF, 0x4669BE79,
    0xCB61B38C, 0xBC66831A, 0x256FD2A0, 0x5268E236,
    0xCC0C7795, 0xBB0B4703, 0x220216B9, 0x5505262F,
    0xC5BA3BBE, 0xB2BD0B28, 0x2BB45A92, 0x5CB36A04,
    0xC2D7FFA7, 0xB5D0CF31, 0x2CD99E8B, 0x5BDEAE1D,
    0x9B64C2B0, 0xEC63F226, 0x756AA39C, 0x026D930A,
    0x9C0906A9, 0xEB0E363F, 0x72076785, 0x05005713,
    0x95BF4A82, 0xE2B87A14, 0x7BB12BAE, 0x0CB61B38,
    0x92D28E9B, 0xE5D5BE0D, 0x7CDCEFB7, 0x0BDBDF21,
    0x86D3D2D4, 0xF1D4E242, 0x68DDB3F8, 0x1FDA836E,
    0x81BE16CD, 0xF6B9265B, 0x6FB077E1, 0x18B74777,
    0x88085AE6, 0xFF0F6A70, 0x66063BCA, 0x11010B5C,
    0x8F659EFF, 0xF862AE69, 0x616BFFD3, 0x166CCF45,
    0xA00AE278, 0xD70DD2EE, 0x4E048354, 0x3903B3C2,
    0xA7672661, 0xD06016F7, 0x4969474D, 0x3E6E77DB,
    0xAED16A4A, 0xD9D65ADC, 0x40DF0B66, 0x37D83BF0,
    0xA9BCAE53, 0xDEBB9EC5, 0x47B2CF7F, 0x30B5FFE9,
    0xBDBDF21C, 0xCABAC28A, 0x53B39330, 0x24B4A3A6,
    0xBAD03605, 0xCDD70693, 0x54DE5729, 0x23D967BF,
    0xB3667A2E, 0xC4614AB8, 0x5D681B02, 0x2A6F2B94,
    0xB40BBE37, 0xC30C8EA1, 0x5A05DF1B, 0x2D02EF8D,
];

/// Rolling hash for string matching
pub struct RollingHash {
    hash: u64,
    base: u64,
    modulo: u64,
    base_pow: u64,
    window: Vec<u8>,
    window_size: usize,
}

impl RollingHash {
    pub fn new(window_size: usize) -> Self {
        Self::with_params(window_size, 31, 1_000_000_007)
    }

    pub fn with_params(window_size: usize, base: u64, modulo: u64) -> Self {
        // Precompute base^window_size
        let mut base_pow = 1u64;
        for _ in 0..window_size {
            base_pow = (base_pow * base) % modulo;
        }

        Self {
            hash: 0,
            base,
            modulo,
            base_pow,
            window: Vec::with_capacity(window_size),
            window_size,
        }
    }

    /// Add a byte to the window
    pub fn push(&mut self, byte: u8) {
        if self.window.len() == self.window_size {
            // Remove oldest byte
            let old = self.window.remove(0);
            self.hash = (self.hash + self.modulo - (old as u64 * self.base_pow) % self.modulo) % self.modulo;
        }

        // Add new byte
        self.window.push(byte);
        self.hash = (self.hash * self.base + byte as u64) % self.modulo;
    }

    /// Get current hash
    pub fn hash(&self) -> u64 {
        self.hash
    }

    /// Get current window
    pub fn window(&self) -> &[u8] {
        &self.window
    }

    /// Check if window is full
    pub fn is_full(&self) -> bool {
        self.window.len() == self.window_size
    }

    /// Reset the hash
    pub fn reset(&mut self) {
        self.hash = 0;
        self.window.clear();
    }
}

/// Hash-based string search (Rabin-Karp)
pub struct RabinKarp {
    pattern_hash: u64,
    pattern_len: usize,
    base: u64,
    modulo: u64,
    base_pow: u64,
}

impl RabinKarp {
    pub fn new(pattern: &[u8]) -> Self {
        let base = 256u64;
        let modulo = 1_000_000_007u64;

        // Compute pattern hash
        let pattern_hash = HashComputer::rolling_hash(pattern, base, modulo);

        // Compute base^(pattern_len-1)
        let mut base_pow = 1u64;
        for _ in 0..pattern.len().saturating_sub(1) {
            base_pow = (base_pow * base) % modulo;
        }

        Self {
            pattern_hash,
            pattern_len: pattern.len(),
            base,
            modulo,
            base_pow,
        }
    }

    /// Search for pattern in text, return all match positions
    pub fn search(&self, text: &[u8]) -> Vec<usize> {
        let mut matches = Vec::new();

        if text.len() < self.pattern_len {
            return matches;
        }

        // Compute initial hash for first window
        let mut text_hash = 0u64;
        for i in 0..self.pattern_len {
            text_hash = (text_hash * self.base + text[i] as u64) % self.modulo;
        }

        // Check first window
        if text_hash == self.pattern_hash {
            matches.push(0);
        }

        // Slide window
        for i in self.pattern_len..text.len() {
            // Remove leading byte, add trailing byte
            let old = text[i - self.pattern_len] as u64;
            let new = text[i] as u64;

            text_hash = (text_hash + self.modulo - (old * self.base_pow) % self.modulo) % self.modulo;
            text_hash = (text_hash * self.base + new) % self.modulo;

            if text_hash == self.pattern_hash {
                matches.push(i - self.pattern_len + 1);
            }
        }

        matches
    }
}

/// Bloom filter for membership testing
pub struct BloomFilter {
    bits: Vec<bool>,
    num_hashes: usize,
    size: usize,
}

impl BloomFilter {
    /// Create bloom filter for given expected number of elements and false positive rate
    pub fn new(expected_elements: usize, false_positive_rate: f64) -> Self {
        let size = optimal_size(expected_elements, false_positive_rate);
        let num_hashes = optimal_hashes(size, expected_elements);

        Self {
            bits: vec![false; size],
            num_hashes,
            size,
        }
    }

    /// Insert an element
    pub fn insert(&mut self, item: &[u8]) {
        for i in 0..self.num_hashes {
            let hash = self.get_hash(item, i);
            self.bits[hash] = true;
        }
    }

    /// Check if element might be in set
    pub fn might_contain(&self, item: &[u8]) -> bool {
        for i in 0..self.num_hashes {
            let hash = self.get_hash(item, i);
            if !self.bits[hash] {
                return false;
            }
        }
        true
    }

    fn get_hash(&self, item: &[u8], seed: usize) -> usize {
        let h1 = HashComputer::murmur3_32(item, seed as u32) as usize;
        let h2 = HashComputer::fnv1a_32(item) as usize;
        (h1.wrapping_add(seed.wrapping_mul(h2))) % self.size
    }

    /// Get approximate fill ratio
    pub fn fill_ratio(&self) -> f64 {
        let set_bits = self.bits.iter().filter(|&&b| b).count();
        set_bits as f64 / self.size as f64
    }
}

fn optimal_size(n: usize, p: f64) -> usize {
    let ln2 = std::f64::consts::LN_2;
    let m = -(n as f64 * p.ln()) / (ln2 * ln2);
    m.ceil() as usize
}

fn optimal_hashes(m: usize, n: usize) -> usize {
    let ln2 = std::f64::consts::LN_2;
    let k = (m as f64 / n as f64) * ln2;
    k.round() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32() {
        let data = b"hello world";
        let crc = HashComputer::crc32(data);
        assert_ne!(crc, 0);
    }

    #[test]
    fn test_rolling_hash() {
        let mut rh = RollingHash::new(4);
        for &b in b"test" {
            rh.push(b);
        }
        let hash1 = rh.hash();

        rh.reset();
        for &b in b"test" {
            rh.push(b);
        }
        let hash2 = rh.hash();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_rabin_karp() {
        let rk = RabinKarp::new(b"test");
        let matches = rk.search(b"this is a test string with test");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_bloom_filter() {
        let mut bloom = BloomFilter::new(100, 0.01);
        bloom.insert(b"hello");
        bloom.insert(b"world");

        assert!(bloom.might_contain(b"hello"));
        assert!(bloom.might_contain(b"world"));
    }
}
