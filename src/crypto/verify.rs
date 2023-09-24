use crate::error::Error;
use data_encoding::BASE64URL;
use jsonwebkey as jwk;
use rand::thread_rng;
use rsa::{pkcs8::DecodePublicKey, PaddingScheme, PublicKey, RsaPublicKey};
use sha2::Digest;

pub fn verify(pub_key: &[u8], message: &[u8], signature: &[u8]) -> Result<(), Error> {
    let jwt_str = format!(
        "{{\"kty\":\"RSA\",\"e\":\"AQAB\",\"n\":\"{}\"}}",
        BASE64URL.encode(pub_key)
    );
    let jwk: jwk::JsonWebKey = jwt_str.parse().unwrap();

    let pub_key = RsaPublicKey::from_public_key_der(jwk.key.to_der().as_slice()).unwrap();
    let mut hasher = sha2::Sha256::new();
    hasher.update(&message);
    let hashed = &hasher.finalize();

    let rng = thread_rng();
    let padding = PaddingScheme::PSS {
        salt_rng: Box::new(rng),
        digest: Box::new(sha2::Sha256::new()),
        salt_len: None,
    };
    pub_key
        .verify(padding, hashed, signature)
        .map(|_| ())
        .map_err(|_| Error::InvalidSignature)
}
