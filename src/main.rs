extern crate handlebars;

extern crate base64;
extern crate clap;
#[macro_use]
extern crate failure;
extern crate jsonwebtoken as jwt;
#[cfg(test)]
extern crate mockito;
extern crate reqwest;
extern crate serde_json;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate rpassword;


mod client;
#[cfg(test)]
mod cli_tests;
#[cfg(test)]
mod main_tests;
mod models;

use std::fs::{OpenOptions, File};
use std::io::Read;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use clap::{App, Arg, SubCommand, ArgMatches};
use handlebars::Handlebars;
use serde_json::Value;

use models::*;
use jwt::{Header, Algorithm};
use failure::{Error, Fail, ResultExt};
use std::fmt;
use std::process::exit;

const ALLOWED_CLUSTERS: &[&'static str] = &["dev-fss", "dev-sbs", "prod-fss", "prod-sbs", "staging-gcp", "dev-gcp", "prod-gcp"];
const FINAL_STATUSES: &[DeploymentState] = &[DeploymentState::Failure, DeploymentState::Error, DeploymentState::Success];
const OKAY_STATUSES: &[DeploymentState] = &[DeploymentState::Success];

fn create_cli_app<'a, 'b>() -> App<'a, 'b> {
    App::new("deployment-cli")
        .version("0.1")
        .author("Kevin Sillerud<kevin.sillerud@nav.no>")
        .about("Application simplifying deployment through https://github.com/navikt/deployment")

        // TODO: Make this a part of the subcommand rather then showing it globally in the help thingie
        .arg(Arg::with_name("resource")
            .short("r")
            .long("resource")
            .help("Kubernetes resource to apply (current only supports YAML files)")
            .multiple(true)
            .global(true)
            .takes_value(true))
        .arg(Arg::with_name("variables")
            .short("v")
            .long("vars")
            .help("Path to json file containing additional variables to use when templating")
            .takes_value(true)
            .global(true))
        .arg(Arg::with_name("ref")
            .short("g")
            .long("ref")
            .help("Reference used for deployment")
            .takes_value(true)
            .global(true)
            .default_value("master"))
        .arg(Arg::with_name("cluster")
            .short("c")
            .long("cluster")
            .possible_values(ALLOWED_CLUSTERS)
            .help("Which cluster to deploy to")
            .takes_value(true)
            .global(true)
            .default_value("dev-fss"))
        .arg(Arg::with_name("team")
            .long("team")
            .short("t")
            .help("Which team this deploy is for")
            .takes_value(true)
            .global(true))
        .arg(Arg::with_name("version")
            .long("version")
            .help("Version number to be deployed")
            .takes_value(true)
            .global(true))
        .arg(Arg::with_name("auto-merge")
            .long("auto-merge")
            .help("Should github try to automatically merge the default branch into ref")
            .takes_value(true)
            .default_value("false")
            .possible_values(&["true", "false"])
            .global(true))

        .subcommand(with_credentials_args(SubCommand::with_name("token")
            .about("Generate github apps token")
            .arg(Arg::with_name("account")
                .long("account")
                .help("Account for the installation id")
                .takes_value(true)
                .env("ACCOUNT")
                .default_value("navikt"))))

        .subcommand(SubCommand::with_name("deploy")
            .about("Command for github deployments")

            .subcommand(with_credentials_args(SubCommand::with_name("create")
                .about("Create a github deployment")
                .arg(Arg::with_name("username")
                    .short("u")
                    .long("username")
                    .takes_value(true)
                    .env("DEPLOYMENT_USERNAME")
                    .required_unless("appid"))
                .arg(Arg::with_name("password")
                    .short("p")
                    .long("password")
                    .takes_value(true)
                    .env("DEPLOYMENT_PASSWORD")
                    .requires("username"))

                .arg(Arg::with_name("repository")
                    .long("repository")
                    .help("Repository to create the deployment request on")
                    .takes_value(true)
                    .required(true))
                .arg(Arg::with_name("await")
                    .long("await")
                    .help("Await a result in the github status(number of seconds)")
                    .default_value("180")
                    .required(true))
                .arg(Arg::with_name("poll-interval")
                    .long("poll-interval")
                    .help("Specifies the interval in ms used for polling while awaiting a github status update")
                    .default_value("1000")
                    .required(true))))

            .subcommand(SubCommand::with_name("payload")
                .about("Templates the deployment payload for the github deployment api, useful for manual curl calls/debugging")
                .arg(Arg::with_name("outputfile")
                    .short("o")
                    .long("outputfile")
                    .help("File to output to, if omitted it will print to stdout")
                    .takes_value(true))))
}

macro_rules! exit_on_err {
    ($input:expr) => {
        match $input {
            Err(err) => {
                println!("Error: {}", err.as_fail());
                for cause in err.iter_causes() {
                    println!("Caused by:");
                    println!("{}", cause);
                }
                println!("{}", err.backtrace());
                exit(1);
            },
            Ok(value) => value
        }
    }
}

fn main() {
    execute_command(create_cli_app().get_matches());
}

fn execute_command(args: ArgMatches) {
    if let Some(token_command) = args.subcommand_matches("token") {
        handle_token_command(token_command);
    }

    if let Some(deploy_command) = args.subcommand_matches("deploy") {
        exit_on_err!(handle_deploy_command(deploy_command));
    }
}

fn with_credentials_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app
        .arg(Arg::with_name("appid")
            .short("a")
            .long("appid")
            .help("Application ID for github apps")
            .takes_value(true)
            .env("GITHUB_APP_ID")
            .required_unless("username")
            .group("github-apps-auth"))
        .arg(Arg::with_name("key")
            .short("k")
            .long("key")
            .help("Path to private key for github application")
            .takes_value(true)
            .env("GITHUB_APP_KEY")
            .required_unless_one(&["key-base64", "username"]))
        .arg(Arg::with_name("key-base64")
            .long("key-base64")
            .help("Private key for github applications, base64 encoded PEM")
            .takes_value(true)
            .env("GITHUB_APP_KEY_BASE64")
            .required_unless_one(&["key", "username"]))
}

fn handle_token_command(subcommand: &ArgMatches) {
    let account = subcommand.value_of("account").unwrap();
    println!("{}", exit_on_err!(installation_token_for(subcommand, account)).token);
}

fn handle_deploy_command(subcommand: &ArgMatches) -> Result<(), Error> {
    let reg = Handlebars::new();

    let mut config: Value = if let Some(config_path) = subcommand.value_of("variables") {
        let file = File::open(config_path).context(format!("Unable to open resource file {}", config_path))?;
        serde_json::from_reader(file).context(format!("Unable to parse json config {}", config_path))?
    } else {
        Value::Null
    };

    let resource_matches: Vec<&str> = if let Some(values) = subcommand.values_of("resource") {
        values.collect()
    } else {
        vec![]
    };

    let git_ref = subcommand.value_of("ref").unwrap();
    let cluster = subcommand.value_of("cluster").unwrap();
    let auto_merge: bool = subcommand.value_of("auto-merge").unwrap()
        .parse()
        .unwrap();
    let team = subcommand.value_of("team")
        .ok_or(format_err!("To create a deployment you need to specify a team"))?;
    let version = subcommand.value_of("version")
        .ok_or(format_err!("To create a deployment you need to specify a version"))?;

    config["ref"] = Value::String(git_ref.to_owned());
    config["cluster"] = Value::String(cluster.to_owned());
    config["team"] = Value::String(team.to_owned());
    config["version"] = Value::String(version.to_owned());

    let resources: Vec<Value> = resource_matches
        .iter()
        .map(|v| File::open(v).expect(format!("Unable to open placeholder file {}", v).as_str()))
        .map(| mut f| {
            let mut string = String::new();
            f.read_to_string(&mut string).expect("Failed to read resource file");
            string
        })
        .map(|s| reg.render_template(s.as_str(), &config)
            .expect("Failed to render template"))
        .inspect(|s| println!("{}", s))
        // TODO: Support json payloads
        .map(|s| serde_yaml::from_str(s.as_str()).expect("Unable to parse JSON from templated output"))
        .collect();

    let deployment_payload = DeploymentRequest {
        git_ref: git_ref.to_owned(),
        auto_merge: auto_merge,
        description: format!("Automated deployment request to {}", cluster),
        environment: cluster.to_owned(),
        required_contexts: vec![],
        payload: Payload {
            version: vec![1, 0, 0],
            team: team.to_owned(),
            kubernetes: Kubernetes {
                resources: resources
            }
        }
    };

    if let Some(payload_subcmd) = subcommand.subcommand_matches("payload") {
        if let Some(output_file) = payload_subcmd.value_of("outputfile") {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(output_file)
                .context(format!("Failed to write to output file {}", output_file))?;
            serde_json::to_writer(file, &deployment_payload)
        } else {
            serde_json::to_writer(std::io::stdout(), &deployment_payload)
        }.context("Failed to serialize json")?;
    }

    if let Some(create_command) = subcommand.subcommand_matches("create") {
        let repository = create_command.value_of("repository").unwrap();

        let (username, password) = exit_on_err!(credentials(create_command, repository));

        let deployment_response = client::create_deployment(repository, &deployment_payload, username, password.as_str())
            .context("Failed to create deployment")?;

        let deployment_id = serde_json::from_str::<Value>(deployment_response.as_str())?
            .get("id")
            .ok_or(format_err!("Dit not receive a id in deployment response"))?
            .as_u64()
            .ok_or(format_err!("Unable to parse deployment id as u64"))?;

        if let Err(error) = await_deploy(create_command, repository, &deployment_id, username, password.as_str()) {
            println!("Failed to deploy application: {}", error);
            exit(1);
        }

        println!("{:?}", deployment_response);
    };
    Ok(())
}

fn credentials<'a>(subcommand: &'a ArgMatches, repository: &str) -> Result<(&'a str, String), Error> {
    let username = if let Some(username) = subcommand.value_of("username") {
        username
    } else {
        "x-access-token"
    };

    let password = if let Some(password) = subcommand.value_of("password") {
        password.to_owned()
    } else if subcommand.is_present("appid") {
        let account = repository.split("/")
            .next()
            .ok_or(format_err!("Repository format should be <user/org>/<repository>, got {}", repository))?;
        installation_token_for(subcommand, account)?.token
    } else {
        rpassword::read_password_from_tty(Some("Please enter github password: "))?
    };

    Ok((username, password))
}

#[derive(Fail, Debug)]
pub struct AwaitFailure {
    status: DeploymentStatus
}

impl fmt::Display for AwaitFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.status.state {
            DeploymentState::Error => write!(f, "Deploy returned status error, this usually means there is a configuration fault in deployment/deployment-cli. For more information check {}", self.status.target_url),
            DeploymentState::Failure => write!(f, "Deploy returned status failure, this usually means there is a configuration error in your Kubernetes resource. For more information check {}", self.status.target_url),
            DeploymentState::TimedOut => write!(f, "deployment-cli timed out waiting for deployment statuses, this usually means your application is failing to start(In a reboot loop or taking too long), check your application logs and the logs from deployment at {}", self.status.target_url),
            _ => write!(f, "Deploy returned an unknown status, for more information check {}", self.status.target_url),
        }
    }
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
            let statuses = client::fetch_status(repository, &deployment_id, username, password)
                .context("Failed to fetch statuses for deployment")?;

            if let Some(final_status) = get_final_status(statuses) {
                return if OKAY_STATUSES.contains(&final_status.state) {
                    Ok(())
                } else {
                    Err(AwaitFailure { status: final_status }.into())
                }
            }
            thread::sleep(Duration::from_millis(poll_interval))
        }
        let mut last_status = client::fetch_status(repository, &deployment_id, username, password)?
            .get(0)
            .cloned()
            .unwrap_or(DeploymentStatus {
                id: 0,
                state: DeploymentState::TimedOut,
                target_url: "Unknown".to_owned()
            });
        last_status.state = DeploymentState::TimedOut;
        Err(AwaitFailure{ status: last_status }.into())
    } else {
        Ok(())
    }
}

fn get_final_status(statuses: Vec<DeploymentStatus>) -> Option<DeploymentStatus> {
    statuses.iter()
        .find(|e| FINAL_STATUSES.contains(&e.state))
        .cloned()
}

fn installation_token_for(subcommand: &ArgMatches, account: &str) -> Result<InstallationToken, Error> {
    let app_id = subcommand.value_of("appid").unwrap();
    let pem = extract_key(subcommand);


    fetch_installation_token(app_id, account, pem?.as_slice())
}

fn extract_key(subcommand: &ArgMatches) -> Result<Vec<u8>, Error> {
    let binary = if let Some(app_key) = subcommand.value_of("key") {
        let mut bytes = vec![];

        File::open(app_key)
            .context("Failed to open Github apps key")?
            .read_to_end(&mut bytes)
            .context("Failed to read contents of github apps key")?;
        bytes
    } else {
        let key_base64 = subcommand.value_of("key-base64").unwrap();
        base64::decode(key_base64)
            .context("Failed to decode base64 enoded Github app private key")?
    };

    decode_private_key(binary)
}

fn decode_private_key(binary: Vec<u8>) -> Result<Vec<u8>, Error> {
    Ok(if let Ok(key_string) = ::std::str::from_utf8(&binary) {
        if key_string.starts_with("-----BEGIN RSA PRIVATE KEY-----") {
            let base64 = key_string
                .replace("\r", "")
                .replace("\n", "");
            // Strip header and footer
            base64::decode(&base64[31..(base64.len() - 29)])
                .context("Failed to base64 decode Github app private key")?
        } else {
            binary
        }
    } else {
        binary
    })
}

fn fetch_installation_token(app_id: &str, account: &str, pem: &[u8]) -> Result<InstallationToken, Error> {
    let jwt = generate_jwt(app_id, pem)?;
    let installation = client::fetch_installations(jwt.as_str())
        .context("Failed to fetch installation token")?;

    let installation_id = installation.iter()
        .find(| v | v.account.login.as_str() == account)
        .ok_or(format_err!("Unable to find the account {} in the list of installations. Is the Github app used for authenticating installed on this account?", account))?
        .id;
    Ok(client::fetch_installation_token(&installation_id, jwt.as_str())?)
}

fn generate_jwt(application_id: &str, private_key: &[u8]) -> Result<String, Error> {
    let current_time_unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let jwt_claims = JwtClaims {
        iss: application_id.to_owned(),
        exp: current_time_unix+300,
        iat: current_time_unix
    };
    Ok(jwt::encode(&Header::new(Algorithm::RS256), &jwt_claims, private_key)
        .context("Failed to generate JWT used to authenticate as a Github app")?)
}
