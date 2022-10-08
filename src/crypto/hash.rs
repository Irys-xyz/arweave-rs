use ring::digest::{Context, SHA256, SHA384};

pub trait Hasher {
    fn hash_sha256(&self, message: &[u8]) -> [u8; 32];
    fn hash_sha384(&self, message: &[u8]) -> [u8; 48];
    fn hash_all_sha256(&self, messages: Vec<&[u8]>) -> [u8; 32];
    fn hash_all_sha384(&self, messages: Vec<&[u8]>) -> [u8; 48];
    fn copy_into_slice_32(&self, m: &[u8]) -> [u8; 32];
    fn copy_into_slice_48(&self, m: &[u8]) -> [u8; 48];
    fn concat_u8_48(&self, left: [u8; 48], right: [u8; 48]) -> [u8; 96];
}

pub struct RingHasher {}

impl Default for RingHasher {
    fn default() -> Self {
        RingHasher {}
    }
}

impl RingHasher {
    pub fn new() -> Self {
        RingHasher::default()
    }
}

impl Hasher for RingHasher {
    fn copy_into_slice_32(&self, m: &[u8]) -> [u8; 32] {
        let mut result: [u8; 32] = [0; 32];
        result.copy_from_slice(m);
        result
    }

    fn copy_into_slice_48(&self, m: &[u8]) -> [u8; 48] {
        let mut result: [u8; 48] = [0; 48];
        result.copy_from_slice(m);
        result
    }

    fn concat_u8_48(&self, left: [u8; 48], right: [u8; 48]) -> [u8; 96] {
        let mut iter = left.into_iter().chain(right);
        let result = [(); 96].map(|_| iter.next().expect("Could not get concat two arrays"));
        result
    }

    fn hash_sha256(&self, message: &[u8]) -> [u8; 32] {
        let mut context = Context::new(&SHA256);
        context.update(message);
        let mut result: [u8; 32] = [0; 32];
        result.copy_from_slice(context.finish().as_ref());
        result
    }

    fn hash_sha384(&self, message: &[u8]) -> [u8; 48] {
        let mut context = Context::new(&SHA384);
        context.update(message);
        let mut result: [u8; 48] = [0; 48];
        result.copy_from_slice(context.finish().as_ref());
        result
    }

    /// Returns a SHA256 hash of the the concatenated SHA256 hashes of a vector of messages.
    fn hash_all_sha256(&self, messages: Vec<&[u8]>) -> [u8; 32] {
        let hash: Vec<u8> = messages
            .into_iter()
            .map(|m| self.hash_sha256(m))
            .into_iter()
            .flatten()
            .collect();
        let hash = self.hash_sha256(&hash);
        hash
    }

    /// Returns a SHA384 hash of the the concatenated SHA384 hashes of a vector messages.
    fn hash_all_sha384(&self, messages: Vec<&[u8]>) -> [u8; 48] {
        let hash: Vec<u8> = messages
            .into_iter()
            .map(|m| self.hash_sha384(m))
            .into_iter()
            .flatten()
            .collect();
        let hash = self.hash_sha384(&hash);
        hash
    }
}
