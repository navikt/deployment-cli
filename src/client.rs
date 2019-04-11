use reqwest::{Client, Response, RequestBuilder};
use super::{Repository, InstallationToken, DeploymentRequest};
use crate::client::ClientError::NotOk;
use std::io::Read;

pub fn fetch_installations(jwt: &str) -> Result<Vec<Repository>, ClientError> {
    let client = Client::new();

    Ok(execute(client.get("https://api.github.com/app/installations")
        .header("Accept", "application/vnd.github.machine-man-preview+json")
        .bearer_auth(jwt))?
        .json()?)
}

pub fn fetch_installation_token(installation_id: &u64, jwt: &str) -> Result<InstallationToken, ClientError> {
    let client = Client::new();

    Ok(execute(client.post(format!("https://api.github.com/app/installations/{}/access_tokens", installation_id).as_str())
        .header("Accept", "application/vnd.github.machine-man-preview+json")
        .bearer_auth(jwt))?
        .json()?)
}

pub fn create_deployment(repo: &str, deployment_payload: &DeploymentRequest, username: &str, password: &str) -> Result<String, ClientError> {
    let client = Client::new();

    Ok(execute(client.post(format!("https://api.github.com/repos/{}/deployments", repo).as_str())
        .json(deployment_payload)
        .basic_auth(username, Some(password)))?
        .text()?)
}

fn execute(request_builder: RequestBuilder) -> Result<Response, ClientError> {
    let mut response = request_builder.send()?;
    let status = response.status();
    if !status.is_success() {
        let mut response_text = String::new();
        if let Err(e) = response.read_to_string(&mut response_text) {
            response_text += format!("{:?}", e).as_str();
        }
        return Err(NotOk(status.as_u16(), response_text))
    }
    return Ok(response)
}

#[derive(Debug)]
pub enum ClientError {
    NotOk(u16, String),
    HttpError(reqwest::Error)
}

impl From<reqwest::Error> for ClientError {
    fn from(err: reqwest::Error) -> Self {
        ClientError::HttpError(err)
    }
}
