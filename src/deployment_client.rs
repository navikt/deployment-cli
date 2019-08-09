use reqwest::{Client, Response};

use crate::github_client::ClientError;
use crate::github_client::execute;

#[cfg(test)]
fn request_deployment_token_url() -> String {
    mockito::server_url()
}

#[cfg(not(test))]
fn request_deployment_token_url() -> &'static str {
    "https://deployment-token-generator.nais.io"
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct TokenRequest<'a> {
    repository: &'a str,
    sources: Vec<&'a str>,
    sinks: Vec<&'a str>,
}

pub fn request_tokens(repository: &str, sources: Vec<&str>, sinks: Vec<&str>, team_name: &str, shared_secret: &str, correlation_id: &str) -> Result<String, ClientError> {
    let client = Client::new();

    println!("{}/api/v1/tokens", request_deployment_token_url());
    Ok(execute(client.post(format!("{}/api/v1/tokens", request_deployment_token_url()).as_str())
        .header("X-Correlation-Id", correlation_id)
        .json(&TokenRequest { repository, sources, sinks })
        .basic_auth(team_name, Some(shared_secret)))?
        .text()?)
}
