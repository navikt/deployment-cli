use crate::models::JwtClaims;
use super::{decode_private_key, generate_jwt};

const PRIVATE_KEY: &[u8] = include_bytes!("../../../testdata/testkey_windows_newlines");
const PUBLIC_KEY_DER: &[u8] = include_bytes!("../../../testdata/testkey_windows_newlines.pub.der");

#[test]
fn test_der_with_windows_newlines() {
    decode_private_key(PRIVATE_KEY.to_vec()).unwrap();
}

#[test]
fn test_generate_valid_jwt() {
    let key_bytes = decode_private_key(PRIVATE_KEY.to_vec()).unwrap();
    let jwt = generate_jwt("abcd", &key_bytes).unwrap();
    jwt::decode::<JwtClaims>(&jwt, PUBLIC_KEY_DER, &jwt::Validation::new(jwt::Algorithm::RS256)).unwrap();
}
