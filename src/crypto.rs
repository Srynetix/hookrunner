use hmac::{Hmac, Mac};
use sha2::Sha256;

pub fn is_valid_signature<'a>(signature: &str, body: &'a [u8], secret: &str) -> bool {
    let mut hmac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    hmac.update(body);

    let decoded = hex::decode(signature).unwrap();
    match hmac.verify_slice(&decoded) {
        Ok(()) => true,
        Err(_) => false,
    }
}
