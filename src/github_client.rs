use reqwest::{Client, Response, RequestBuilder};
use crate::models::{Repository, InstallationToken, DeploymentRequest, DeploymentStatus};
use crate::github_client::ClientError::NotOk;
use std::io::Read;

#[cfg(test)]
fn github_url() -> String {
    mockito::server_url()
}

#[cfg(not(test))]
fn github_url() -> &'static str {
    "https://api.github.com"
}

#[derive(Fail, Debug)]
pub enum ClientError {
    #[fail(display = "HTTP call returned unexpected result code {}, response: {}", status_code, response)]
    NotOk{ status_code: u16, response: String },
    #[fail(display = "Failed to execute HTTP call {}", error)]
    HttpError { error: reqwest::Error }
}

impl From<reqwest::Error> for ClientError {
    fn from(err: reqwest::Error) -> Self {
        ClientError::HttpError { error: err }
    }
}

pub fn execute(request_builder: RequestBuilder) -> Result<Response, ClientError> {
    let mut response = request_builder.send()?;
    let status = response.status();
    if !status.is_success() {
        let mut response_text = String::new();
        if let Err(e) = response.read_to_string(&mut response_text) {
            response_text += format!("{:?}", e).as_str();
        }
        return Err(NotOk { status_code: status.as_u16(), response: response_text })
    }
    return Ok(response)
}

pub fn fetch_installations(jwt: &str) -> Result<Vec<Repository>, ClientError> {
    let client = Client::new();

    Ok(execute(client.get(format!("{}/app/installations", github_url()).as_str())
        .header("Accept", "application/vnd.github.machine-man-preview+json")
        .bearer_auth(jwt))?
        .json()?)
}

pub fn fetch_installation_token(installation_id: &u64, jwt: &str) -> Result<InstallationToken, ClientError> {
    let client = Client::new();

    Ok(execute(client.post(format!("{}/app/installations/{}/access_tokens", github_url(), installation_id).as_str())
        .header("Accept", "application/vnd.github.machine-man-preview+json")
        .bearer_auth(jwt))?
        .json()?)
}

pub fn create_deployment(repo: &str, deployment_payload: &DeploymentRequest, username: &str, password: &str) -> Result<String, ClientError> {
    let client = Client::new();

    Ok(execute(client.post(format!("{}/repos/{}/deployments", github_url(), repo).as_str())
        .json(deployment_payload)
        .basic_auth(username, Some(password)))?
        .text()?)
}

pub fn fetch_status(repo: &str, id: &u64, username: &str, password: &str) -> Result<Vec<DeploymentStatus>, ClientError> {
    let client = Client::new();

    Ok(execute(client.get(format!("{}/repos/{}/deployments/{}/statuses", github_url(), repo, id).as_str())
        .basic_auth(username, Some(password)))?
        .json()?)
}
