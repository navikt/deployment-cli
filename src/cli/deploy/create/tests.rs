use crate::client;
use crate::models::{DeploymentState, DeploymentStatus};

#[test]
fn test_successful_github_status() {
    let status_mock = mock("GET", "/repos/navikt/deployment-cli/deployments/1/statuses")
        .with_body_from_file("testdata/statuses.json")
        .expect(1)
        .create();

    let statuses = client::fetch_status("navikt/deployment-cli", &1u64, "user", "pass")
        .unwrap();

    assert_eq!(Some(DeploymentStatus{
        id: 1,
        target_url: "http://localhost".to_owned(),
        state: DeploymentState::Success
    }), super::get_final_status(statuses));

    status_mock.assert();
}

#[test]
fn test_no_final_github_status() {
    let status_mock = mock("GET", "/repos/navikt/deployment-cli/deployments/1/statuses")
        .with_body_from_file("testdata/statuses_no_final.json")
        .expect(1)
        .create();

    let statuses = client::fetch_status("navikt/deployment-cli", &1u64, "user", "pass")
        .unwrap();

    assert_eq!(None, super::get_final_status(statuses));

    status_mock.assert();
}
