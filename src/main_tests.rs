use mockito::mock;
use super::client::fetch_status;
use crate::models::DeploymentStatus;
use crate::{decode_private_key, generate_jwt, JwtClaims};

use base64;
use jwt;

const PRIVATE_KEY: &[u8] = include_bytes!("../testdata/testkey_windows_newlines");
const PRIVATE_KEY_B64: &[u8] = include_bytes!("../testdata/testkey_windows_newlines.b64");
const PUBLIC_KEY_DER: &[u8] = include_bytes!("../testdata/testkey_windows_newlines.pub.der");

#[test]
fn test_successful_github_status() {
    let status_mock = mock("GET", "/repos/navikt/deployment-cli/deployments/1/statuses")
        .with_body_from_file("testdata/statuses.json")
        .expect(1)
        .create();

    let statuses = fetch_status("navikt/deployment-cli", &1u64, "user", "pass")
        .unwrap();

    assert_eq!(Some(DeploymentStatus{
        id: 1,
        state: "success".to_owned()
    }), super::get_final_status(statuses));

    status_mock.assert();
}

#[test]
fn test_no_final_github_status() {
    let status_mock = mock("GET", "/repos/navikt/deployment-cli/deployments/1/statuses")
        .with_body_from_file("testdata/statuses_no_final.json")
        .expect(1)
        .create();

    let statuses = fetch_status("navikt/deployment-cli", &1u64, "user", "pass")
        .unwrap();

    assert_eq!(None, super::get_final_status(statuses));

    status_mock.assert();
}

#[test]
fn test_der_with_windows_newlines() {
    let bytes = decode_private_key(PRIVATE_KEY.to_vec());
}

#[test]
fn test_generate_valid_jwt() {
    let key_bytes = decode_private_key(PRIVATE_KEY.to_vec());
    let jwt = generate_jwt("abcd", &key_bytes);
    jwt::decode::<JwtClaims>(&jwt, PUBLIC_KEY_DER, &jwt::Validation::new(jwt::Algorithm::RS256)).unwrap();
}

fn decode_public_key(binary: Vec<u8>) -> Vec<u8> {
    if let Ok(key_string) = ::std::str::from_utf8(&binary) {
        if key_string.starts_with("-----BEGIN PUBLIC KEY-----") {
            let base64 = key_string
                .replace("\r", "")
                .replace("\n", "");
            // Strip header and footer
            base64::decode(&base64[26..(base64.len() - 24)]).unwrap()
        } else {
            binary
        }
    } else {
        binary
    }
}
