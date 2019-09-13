use crate::cli::{create_cli_app, execute_command};
use mockito::{mock, Matcher, Mock};

const PRIVATE_KEY_B64: &'static str = include_str!("../../../testdata/testkey_windows_newlines.b64");
const EXPECTED_PAYLOAD: &'static str = include_str!("../../../testdata/expected_payload.json");
const EXPECTED_PAYLOAD_WITH_VAR_OVERRIDE: &'static str = include_str!("../../../testdata/expected_payload_var_override.json");
const EXPECTED_PAYLOAD_WITH_VARS: &'static str = include_str!("../../../testdata/expected_payload_with_vars.json");
const EXPECTED_PAYLOAD_RAW_RESOURCE: &'static str = include_str!("../../../testdata/expected_payload_raw.json");

const JWT_MATCHER: &'static str = "Bearer .+\\..+\\..+";

fn json_matcher<M: Into<String>>(json: M) -> Matcher {
    Matcher::JsonString(json.into())
}

#[test]
fn test_deploy_payload_write_to_file() {
    let args = vec!["deployment-cli", "deploy", "payload", "--cluster", "prod-fss", "--team", "plattform", "--var", "version=1.0.0", "--resource", "testdata/nais.yaml", "--outputfile", "target/payload.json"];
    let matches = create_cli_app().get_matches_from_safe(args);

    let result = assert_ok!(matches);

    assert_ok!(execute_command(&result));
}

#[test]
fn test_deploy_payload_write_to_stdout() {
    let args = vec!["deployment-cli", "deploy", "payload", "--cluster", "prod-fss", "--team", "plattform", "--var", "version=1.0.0", "--resource", "testdata/nais.yaml"];
    let matches = create_cli_app().get_matches_from_safe(args);

    let result = assert_ok!(matches);

    assert_ok!(execute_command(&result));
}

#[test]
fn test_deploy_payload_write_to_file_with_vars() {
    let args = vec!["deployment-cli", "deploy", "payload", "--cluster", "prod-fss", "--team", "plattform", "--var", "version=1.0.0", "--resource", "testdata/nais.yaml", "--outputfile", "target/payload_with_vars.json", "--vars", "testdata/vars.json"];
    let matches = create_cli_app().get_matches_from_safe(args);

    let result = assert_ok!(matches);

    assert_ok!(execute_command(&result));
}

fn status_mock() -> Mock {
    mock("GET", "/repos/navikt/testapp/deployments/1/statuses")
        .with_body_from_file("testdata/statuses.json")
        .expect(1)
        .create()
}

fn deployment_mock<B: Into<mockito::Matcher>, A: Into<mockito::Matcher>>(expected_body: B, auth_matcher: A) -> Mock {
    mock("POST", "/repos/navikt/testapp/deployments")
        .with_body_from_file("testdata/deployment_create_response.json")
        .match_header("Authorization", auth_matcher)
        .expect(1)
        .match_body(expected_body)
        .create()
}

fn installations_mock() -> Mock {
    mock("GET", "/app/installations")
        .with_body_from_file("testdata/installations.json")
        .match_header("Authorization", jwt_auth())
        .expect(1)
        .create()
}

fn access_token_mock() -> Mock {
    mock("POST", "/app/installations/123/access_tokens")
        .with_body_from_file("testdata/installation_access_token.json")
        .match_header("Authorization", jwt_auth())
        .expect(1)
        .create()
}

fn basic_auth() -> &'static str { "Basic dGVzdHVzZXI6dGVzdHBhc3N3b3Jk" }


fn gh_app_auth() -> &'static str { "Basic eC1hY2Nlc3MtdG9rZW46YWJjZGU=" }
fn gh_token_auth() -> &'static str { "Basic eC1hY2Nlc3MtdG9rZW46djEuYWJjNjkxMjM=" }
fn jwt_auth() -> mockito::Matcher { mockito::Matcher::Regex(JWT_MATCHER.to_owned()) }

#[test]
fn test_create_deployment() {
    let deployments_mock = deployment_mock(json_matcher(EXPECTED_PAYLOAD), basic_auth());
    let status_mock = status_mock();
    let args = vec!["deployment-cli", "deploy", "create", "--cluster", "prod-fss", "--team", "plattform", "--var", "version=1.0.0", "--resource", "testdata/nais.yaml", "--repository", "navikt/testapp", "--username", "testuser", "--password", "testpassword"];
    let matches = create_cli_app().get_matches_from_safe(args);

    let result = assert_ok!(matches);

    assert_ok!(execute_command(&result));
    deployments_mock.assert();
    status_mock.assert();
}

#[test]
fn test_create_deployment_with_vars() {
    let deployments_mock = deployment_mock(json_matcher(EXPECTED_PAYLOAD_WITH_VARS), basic_auth());
    let status_mock = status_mock();
    let args = vec!["deployment-cli", "deploy", "create", "--cluster", "prod-fss", "--team", "plattform", "--var", "version=1.0.0", "--resource", "testdata/nais.yaml", "--repository", "navikt/testapp", "--username", "testuser", "--password", "testpassword", "--vars", "testdata/vars.json"];
    let matches = create_cli_app().get_matches_from_safe(args);

    let result = assert_ok!(matches);

    assert_ok!(execute_command(&result));
    deployments_mock.assert();
    status_mock.assert();
}

#[test]
fn test_create_deployment_with_var_overrides() {
    let status_mock = status_mock();
    let deployments_mock = deployment_mock(json_matcher(EXPECTED_PAYLOAD_WITH_VAR_OVERRIDE), basic_auth());
    let args = vec!["deployment-cli", "deploy", "create", "--cluster", "prod-fss", "--team", "plattform", "--var", "version=1.0.0", "--resource", "testdata/nais_with_var_override.yaml", "--repository", "navikt/testapp", "--username", "testuser", "--password", "testpassword", "--var", "namespace=overridden", "--var", "name=thisismy=name"];
    let matches = create_cli_app().get_matches_from_safe(args);

    let result = assert_ok!(matches);

    assert_ok!(execute_command(&result));
    deployments_mock.assert();
    status_mock.assert();
}

#[test]
fn test_create_deployment_with_deprecated_version_flag() {
    let deployments_mock = deployment_mock(json_matcher(EXPECTED_PAYLOAD_WITH_VARS), basic_auth());
    let status_mock = status_mock();
    let args = vec!["deployment-cli", "deploy", "create", "--cluster", "prod-fss", "--team", "plattform", "--version", "1.0.0", "--resource", "testdata/nais.yaml", "--repository", "navikt/testapp", "--username", "testuser", "--password", "testpassword", "--vars", "testdata/vars.json"];
    let matches = create_cli_app().get_matches_from_safe(args);

    let result = assert_ok!(matches);

    assert_ok!(execute_command(&result));
    deployments_mock.assert();
    status_mock.assert();
}

#[test]
fn test_create_deployment_with_raw_resource() {
    let deployments_mock = deployment_mock(json_matcher(EXPECTED_PAYLOAD_RAW_RESOURCE), basic_auth());
    let status_mock = status_mock();
    let args = vec!["deployment-cli", "deploy", "create", "--cluster", "prod-fss", "--team", "plattform", "--raw-resource", "testdata/nais_valid_raw.yaml", "--repository", "navikt/testapp", "--username", "testuser", "--password", "testpassword"];
    let matches = create_cli_app().get_matches_from_safe(args);

    let result = assert_ok!(matches);

    assert_ok!(execute_command(&result));
    deployments_mock.assert();
    status_mock.assert();
}

#[test]
fn test_create_deployment_github_app() {
    println!("{}", PRIVATE_KEY_B64);
    let args = vec!["deployment-cli", "deploy", "create", "--cluster", "prod-fss", "--team", "plattform", "--var", "version=1.0.0", "--resource", "testdata/nais.yaml", "--repository", "navikt/testapp", "--appid", "1234", "--key", "testdata/testkey_windows_newlines"];
    let installations_mock = installations_mock();
    let access_token_mock = access_token_mock();
    let deployments_mock = deployment_mock(json_matcher(EXPECTED_PAYLOAD), gh_app_auth());
    let status_mock = status_mock();

    let matches = create_cli_app().get_matches_from_safe(args);

    let result = assert_ok!(matches);
    assert_ok!(execute_command(&result));

    installations_mock.assert();
    access_token_mock.assert();
    deployments_mock.assert();
    status_mock.assert();
}

#[test]
fn test_create_deployment_github_app_base64_key() {
    println!("{}", PRIVATE_KEY_B64);
    let args = vec!["deployment-cli", "deploy", "create", "--cluster", "prod-fss", "--team", "plattform", "--var", "version=1.0.0", "--resource", "testdata/nais.yaml", "--repository", "navikt/testapp", "--appid", "1234", "--key-base64", PRIVATE_KEY_B64.trim()];
    let installations_mock = installations_mock();
    let access_token_mock = access_token_mock();
    let deployments_mock = deployment_mock(json_matcher(EXPECTED_PAYLOAD), gh_app_auth());
    let status_mock = status_mock();

    let matches = create_cli_app().get_matches_from_safe(args);

    let result = assert_ok!(matches);
    assert_ok!(execute_command(&result));

    installations_mock.assert();
    access_token_mock.assert();
    deployments_mock.assert();
    status_mock.assert();
}

#[test]
fn test_create_deployment_github_token() {
    let deployments_mock = deployment_mock(json_matcher(EXPECTED_PAYLOAD), gh_token_auth());
    let status_mock = status_mock();
    let args = vec!["deployment-cli", "deploy", "create", "--cluster", "prod-fss", "--team", "plattform", "--var", "version=1.0.0", "--resource", "testdata/nais.yaml", "--repository", "navikt/testapp", "--token", "v1.abc69123"];
    let matches = create_cli_app().get_matches_from_safe(args);

    let result = assert_ok!(matches);
    assert_ok!(execute_command(&result));

    deployments_mock.assert();
    status_mock.assert();
}
