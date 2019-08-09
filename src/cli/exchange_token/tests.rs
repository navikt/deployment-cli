use crate::cli::{create_cli_app, execute_command};
use failure::Error;
use mockito::mock;

#[test]
fn test_exchange_token() {
    mock("POST", "/api/v1/tokens")
        .expect(1)
        .with_status(201)
        .with_body("{}")
        .create();

    let args = vec!["deployment-cli", "exchange_token", "--team", "plattform", "--repository", "deployment-cli", "--correlation-id", "trackable_id", "--shared-secret", "abcde"];
    let matches = create_cli_app().get_matches_from_safe(args);

    let result = assert_ok!(matches);

    assert_ok!(execute_command(&result));
}