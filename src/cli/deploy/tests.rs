use crate::cli::{create_cli_app, execute_command};
use mockito::mock;

const PRIVATE_KEY_B64: &'static str = include_str!("../../../testdata/testkey_windows_newlines.b64");
const EXPECTED_PAYLOAD: &'static str = include_str!("../../../testdata/expected_payload.json");
const EXPECTED_PAYLOAD_WITH_VARS: &'static str = include_str!("../../../testdata/expected_payload_with_vars.json");

const JWT_MATCHER: &'static str = "Bearer .+\\..+\\..+";

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
fn test_deploy_payload_write_to_file_with_vars() {
    let args = vec!["deployment-cli", "deploy", "payload", "--cluster", "prod-fss", "--team", "plattform", "--version", "1.0.0", "--resource", "testdata/nais.yaml", "--outputfile", "target/payload_with_vars.json", "--vars", "testdata/vars.json"];
    let matches = create_cli_app().get_matches_from_safe(args);

    let result = assert_ok!(matches);

    execute_command(result);
}

#[test]
fn test_create_deployment() {
    let deployments_mock = mock("POST", "/repos/navikt/testapp/deployments")
        .with_body_from_file("testdata/deployment_create_response.json")
        .match_header("Authorization", "Basic dGVzdHVzZXI6dGVzdHBhc3N3b3Jk")
        .match_body(EXPECTED_PAYLOAD.trim())
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

#[test]
fn test_create_deployment_with_vars() {
    let deployments_mock = mock("POST", "/repos/navikt/testapp/deployments")
        .with_body_from_file("testdata/deployment_create_response.json")
        .match_header("Authorization", "Basic dGVzdHVzZXI6dGVzdHBhc3N3b3Jk")
        .match_body(EXPECTED_PAYLOAD_WITH_VARS.trim())
        .expect(1)
        .create();
    let status_mock = mock("GET", "/repos/navikt/testapp/deployments/1/statuses")
        .with_body_from_file("testdata/statuses.json")
        .expect(1)
        .create();
    let args = vec!["deployment-cli", "deploy", "create", "--cluster", "prod-fss", "--team", "plattform", "--version", "1.0.0", "--resource", "testdata/nais.yaml", "--repository", "navikt/testapp", "--username", "testuser", "--password", "testpassword", "--vars", "testdata/vars.json"];
    let matches = create_cli_app().get_matches_from_safe(args);

    let result = assert_ok!(matches);

    execute_command(result);
    deployments_mock.assert();
    status_mock.assert();
}

#[test]
fn test_create_deployment_github_app() {
    println!("{}", PRIVATE_KEY_B64);
    let args = vec!["deployment-cli", "deploy", "create", "--cluster", "prod-fss", "--team", "plattform", "--version", "1.0.0", "--resource", "testdata/nais.yaml", "--repository", "navikt/testapp", "--appid", "1234", "--key", "testdata/testkey_windows_newlines"];
    let installations_mock = mock("GET", "/app/installations")
        .with_body_from_file("testdata/installations.json")
        .match_header("Authorization", mockito::Matcher::Regex(JWT_MATCHER.to_owned()))
        .expect(1)
        .create();
    let access_token_mock = mock("POST", "/app/installations/123/access_tokens")
        .with_body_from_file("testdata/installation_access_token.json")
        .match_header("Authorization", mockito::Matcher::Regex(JWT_MATCHER.to_owned()))
        .expect(1)
        .create();
    let deployments_mock = mock("POST", "/repos/navikt/testapp/deployments")
        .with_body_from_file("testdata/deployment_create_response.json")
        .match_header("Authorization", "Basic eC1hY2Nlc3MtdG9rZW46YWJjZGU=")
        .match_body(EXPECTED_PAYLOAD.trim())
        .expect(1)
        .create();
    let status_mock = mock("GET", "/repos/navikt/testapp/deployments/1/statuses")
        .with_body_from_file("testdata/statuses.json")
        .expect(1)
        .create();

    let matches = create_cli_app().get_matches_from_safe(args);

    let result = assert_ok!(matches);
    execute_command(result);

    installations_mock.assert();
    access_token_mock.assert();
    deployments_mock.assert();
    status_mock.assert();
}

#[test]
fn test_create_deployment_github_app_base64_key() {
    println!("{}", PRIVATE_KEY_B64);
    let args = vec!["deployment-cli", "deploy", "create", "--cluster", "prod-fss", "--team", "plattform", "--version", "1.0.0", "--resource", "testdata/nais.yaml", "--repository", "navikt/testapp", "--appid", "1234", "--key-base64", PRIVATE_KEY_B64.trim()];
    let installations_mock = mock("GET", "/app/installations")
        .with_body_from_file("testdata/installations.json")
        .match_header("Authorization", mockito::Matcher::Regex(JWT_MATCHER.to_owned()))
        .expect(1)
        .create();
    let access_token_mock = mock("POST", "/app/installations/123/access_tokens")
        .with_body_from_file("testdata/installation_access_token.json")
        .match_header("Authorization", mockito::Matcher::Regex(JWT_MATCHER.to_owned()))
        .expect(1)
        .create();
    let deployments_mock = mock("POST", "/repos/navikt/testapp/deployments")
        .with_body_from_file("testdata/deployment_create_response.json")
        .match_header("Authorization", "Basic eC1hY2Nlc3MtdG9rZW46YWJjZGU=")
        .match_body(EXPECTED_PAYLOAD.trim())
        .expect(1)
        .create();
    let status_mock = mock("GET", "/repos/navikt/testapp/deployments/1/statuses")
        .with_body_from_file("testdata/statuses.json")
        .expect(1)
        .create();

    let matches = create_cli_app().get_matches_from_safe(args);

    let result = assert_ok!(matches);
    execute_command(result);

    installations_mock.assert();
    access_token_mock.assert();
    deployments_mock.assert();
    status_mock.assert();
}
