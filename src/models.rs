use serde_json::Value;

#[derive(Serialize)]
pub struct DeploymentRequest {
    #[serde(rename = "ref")]
    pub git_ref: String,
    pub description: String,
    pub environment: String,
    pub payload: Payload,
    pub required_contexts: Vec<String>
}

#[derive(Serialize)]
pub struct Payload {
    pub version: Vec<u32>,
    pub team: String,
    pub kubernetes: Kubernetes
}

#[derive(Serialize)]
pub struct Kubernetes {
    pub resources: Vec<Value>
}

#[derive(Serialize)]
pub struct JwtClaims {
    pub iat: u64,
    pub exp: u64,
    pub iss: String
}

#[derive(Deserialize, Debug)]
pub struct Repository {
    pub id: u64,
    pub account: Account
}

#[derive(Deserialize, Debug)]
pub struct Account {
    pub id: u64,
    pub login: String
}

#[derive(Deserialize, Debug)]
pub struct InstallationToken {
    pub token: String,
    pub expires_at: String
}

#[derive(Deserialize, Debug)]
pub struct DeploymentStatus {
    pub id: u64,
    pub status: String
}
