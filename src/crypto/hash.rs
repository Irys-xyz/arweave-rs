use ring::digest::{Context, SHA256, SHA384};

pub struct Hasher {}

impl Hasher {
    pub fn copy_into_slice_32(m: &[u8]) -> [u8; 32] {
        let mut result: [u8; 32] = [0; 32];
        result.copy_from_slice(m);
        result
    }

    pub fn copy_into_slice_48(m: &[u8]) -> [u8; 48] {
        let mut result: [u8; 48] = [0; 48];
        result.copy_from_slice(m);
        result
    }

    pub fn concat_u8_48(left: [u8; 48], right: [u8; 48]) -> [u8; 96] {
        let mut iter = left.into_iter().chain(right);
        let result = [(); 96].map(|_| iter.next().unwrap());
        result
    }

    pub fn hash_sha256(message: &[u8]) -> [u8; 32] {
        let mut context = Context::new(&SHA256);
        context.update(message);
        let mut result: [u8; 32] = [0; 32];
        result.copy_from_slice(context.finish().as_ref());
        result
    }

    pub fn hash_sha384(message: &[u8]) -> [u8; 48] {
        let mut context = Context::new(&SHA384);
        context.update(message);
        let mut result: [u8; 48] = [0; 48];
        result.copy_from_slice(context.finish().as_ref());
        result
    }

    /// Returns a SHA256 hash of the the concatenated SHA256 hashes of a vector of messages.
    pub fn hash_all_sha256(messages: Vec<&[u8]>) -> [u8; 32] {
        let hash: Vec<u8> = messages
            .into_iter()
            .map(|m| Hasher::hash_sha256(m))
            .into_iter()
            .flatten()
            .collect();
        let hash = Hasher::hash_sha256(&hash);
        hash
    }

    /// Returns a SHA384 hash of the the concatenated SHA384 hashes of a vector messages.
    pub fn hash_all_sha384(messages: Vec<&[u8]>) -> [u8; 48] {
        let hash: Vec<u8> = messages
            .into_iter()
            .map(|m| Hasher::hash_sha384(m))
            .into_iter()
            .flatten()
            .collect();
        let hash = Hasher::hash_sha384(&hash);
        hash
    }
}
