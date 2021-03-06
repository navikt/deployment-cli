use serde_json::Value;

#[derive(Serialize, Debug, Clone)]
pub struct DeploymentRequest {
    #[serde(rename = "ref")]
    pub git_ref: String,
    pub auto_merge: bool,
    pub description: String,
    pub environment: String,
    pub payload: Payload,
    pub required_contexts: Vec<String>
}

#[derive(Serialize, Debug, Clone)]
pub struct Payload {
    pub version: Vec<u32>,
    pub team: String,
    pub kubernetes: Kubernetes
}

#[derive(Serialize, Debug, Clone)]
pub struct Kubernetes {
    pub resources: Vec<Value>
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct JwtClaims {
    pub iat: u64,
    pub exp: u64,
    pub iss: String
}

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Repository {
    pub id: u64,
    pub account: Account
}

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct Account {
    pub id: u64,
    pub login: String
}

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct InstallationToken {
    pub token: String,
    pub expires_at: String
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentState {
    Failure,
    Error,
    Success,
    TimedOut,
    #[serde(other)]
    Other
}

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct DeploymentStatus {
    pub id: u64,
    pub state: DeploymentState,
    pub target_url: String
}
