use mockito::mock;
use super::client::fetch_status;
use crate::models::DeploymentStatus;
use crate::{decode_key};

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
    let key = include_bytes!("../testdata/testkey_windows_newlines");
    let bytes = decode_key(key.to_vec());
}
