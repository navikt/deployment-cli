use std::fmt;
use std::thread;
use std::time::{Duration, SystemTime};

use clap::ArgMatches;
use failure::{Error, ResultExt};
use serde_json::Value;

use crate::github_client;
use crate::models::{DeploymentState, DeploymentStatus, DeploymentRequest};

const FINAL_STATUSES: &[DeploymentState] = &[DeploymentState::Failure, DeploymentState::Error, DeploymentState::Success];
const OKAY_STATUSES: &[DeploymentState] = &[DeploymentState::Success];

#[derive(Fail, Debug)]
pub struct AwaitFailure {
    status: DeploymentStatus
}

impl fmt::Display for AwaitFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.status.state {
            DeploymentState::Error => write!(f, "Deploy returned the status \"error\", this usually means there is a configuration fault in deployment or deployment-cli. For more information check {}", self.status.target_url),
            DeploymentState::Failure => write!(f, "Deploy returned the status \"failure\", this usually means there is a configuration error in your Kubernetes resource. For more information check {}", self.status.target_url),
            DeploymentState::TimedOut => write!(f, "deployment-cli timed out waiting for deployment statuses, this usually means your application is failing to start(in a reboot loop or taking too long), check your application logs and the logs from deployment at {}", self.status.target_url),
            _ => write!(f, "Deploy returned an unknown status, for more information check {}", self.status.target_url),
        }
    }
}

pub fn handle_deploy_create_command(create_command: &ArgMatches, deployment_payload: &DeploymentRequest) -> Result<(), Error> {
    let repository = create_command.value_of("repository").unwrap();

    if !repository.contains("/") {
        return Err(format_err!("Repository format should be <user/org>/<repository>, got \"{}\"", repository));
    }

    let (username, password) = credentials(create_command, repository)?;

    let deployment_response = github_client::create_deployment(repository, deployment_payload, username, password.as_str())
        .context("Failed to create deployment")?;
    println!("{:?}", deployment_response);

    let deployment_id = serde_json::from_str::<Value>(deployment_response.as_str())?
        .get("id")
        .ok_or(format_err!("Dit not receive a id in deployment response"))?
        .as_u64()
        .ok_or(format_err!("Unable to parse deployment id as u64"))?;

    await_deploy(create_command, repository, &deployment_id, username, password.as_str())?;
    Ok(())
}

fn credentials<'a>(subcommand: &'a ArgMatches, repository: &str) -> Result<(&'a str, String), Error> {
    let username = if let Some(username) = subcommand.value_of("username") {
        username
    } else {
        "x-access-token"
    };

    let password = if let Some(token) = subcommand.value_of("token") {
        token.to_owned()
    } else if let Some(password) = subcommand.value_of("password") {
        password.to_owned()
    } else if subcommand.is_present("appid") {
        let account = repository.split("/")
            .next()
            .ok_or(format_err!("Repository format should be <user/org>/<repository>, got \"{}\"", repository))?;
        crate::cli::token::installation_token_for(subcommand, account)?.token
    } else {
        rpassword::read_password_from_tty(Some("Please enter github password: "))?
    };

    Ok((username, password))
}

fn await_deploy(subcommand: &ArgMatches, repository: &str, deployment_id: &u64, username: &str, password: &str) -> Result<(), Error> {
    let await_seconds = subcommand.value_of("await")
        .unwrap()
        .parse::<u64>()
        .context("Provided await value could not be parsed as a number")?;

    if await_seconds != 0 {
        let poll_interval = subcommand.value_of("poll-interval")
            .unwrap()
            .parse::<u64>()
            .context("Provided poll-interval could not be parsed as a number")?;

        let start_time = SystemTime::now();
        while SystemTime::now().duration_since(start_time).unwrap() < Duration::from_secs(await_seconds) {
            let statuses = github_client::fetch_status(repository, &deployment_id, username, password)
                .context("Failed to fetch statuses for deployment")?;

            if let Some(final_status) = get_final_status(statuses) {
                return if OKAY_STATUSES.contains(&final_status.state) {
                    Ok(())
                } else {
                    Err(AwaitFailure { status: final_status }.into())
                };
            }
            thread::sleep(Duration::from_millis(poll_interval))
        }
        let mut last_status = github_client::fetch_status(repository, &deployment_id, username, password)?
            .get(0)
            .cloned()
            .unwrap_or(DeploymentStatus {
                id: 0,
                state: DeploymentState::TimedOut,
                target_url: "Unknown".to_owned(),
            });
        last_status.state = DeploymentState::TimedOut;
        Err(AwaitFailure { status: last_status }.into())
    } else {
        Ok(())
    }
}

fn get_final_status(statuses: Vec<DeploymentStatus>) -> Option<DeploymentStatus> {
    statuses.iter()
        .find(|e| FINAL_STATUSES.contains(&e.state))
        .cloned()
}