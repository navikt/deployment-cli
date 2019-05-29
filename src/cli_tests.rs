use crate::{create_cli_app, execute_command};
use mockito::mock;


macro_rules! assert_ok {
    ($x:expr) => {
        {
            assert!($x.is_ok(), format!("{:?}", $x.unwrap_err()));
            $x.unwrap()
        }
    }
}

#[test]
fn test_deploy_payload_write_to_file() {
    let args = vec!["deployment-cli", "deploy", "payload", "--cluster", "prod-fss", "--team", "plattform", "--version", "1.0.0", "--resource", "testdata/nais.yaml", "--outputfile", "target/payload.json"];
    let matches = create_cli_app().get_matches_from_safe(args);

    let result =  assert_ok!(matches);

    execute_command(result);
}

#[test]
fn test_deploy_payload_write_to_stdout() {
    let args = vec!["deployment-cli", "deploy", "payload", "--cluster", "prod-fss", "--team", "plattform", "--version", "1.0.0", "--resource", "testdata/nais.yaml"];
    let matches = create_cli_app().get_matches_from_safe(args);

    let result = assert_ok!(matches);

    execute_command(result);
}

#[test]
fn test_create_deployment() {
    let deployments_mock = mock("POST", "/repos/navikt/testapp/deployments")
        .with_body_from_file("testdata/deployment_create_response.json")
        .match_header("Authorization", "Basic dGVzdHVzZXI6dGVzdHBhc3N3b3Jk")
        .expect(1)
        .create();
    let status_mock = mock("GET", "/repos/navikt/testapp/deployments/1/statuses")
        .with_body_from_file("testdata/statuses.json")
        .expect(1)
        .create();
    let args = vec!["deployment-cli", "deploy", "create", "--cluster", "prod-fss", "--team", "plattform", "--version", "1.0.0", "--resource", "testdata/nais.yaml", "--repository", "navikt/testapp", "--username", "testuser", "--password", "testpassword"];
    let matches = create_cli_app().get_matches_from_safe(args);

    let result = assert_ok!(matches);

    execute_command(result);
    deployments_mock.assert();
    status_mock.assert();
}

