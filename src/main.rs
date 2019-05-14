extern crate handlebars;

extern crate base64;
extern crate clap;
extern crate jsonwebtoken as jwt;
extern crate reqwest;
extern crate serde_json;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate rpassword;

mod client;
mod models;

use std::fs::File;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{App, Arg, SubCommand, ArgMatches};
use handlebars::Handlebars;
use serde_json::Value;

use models::*;
use jwt::{Header, Algorithm};

const ALLOWED_CLUSTERS: &[&'static str] = &["dev-fss", "dev-sbs", "prod-fss", "prod-sbs"];

fn main() {
    let matches = App::new("deployment-cli")
        .version("0.1")
        .author("Kevin Sillerud<kevin.sillerud@nav.no>")
        .about("Application simplifying deployment through https://github.com/navikt/deployment")

        // TODO: Make this a part of the subcommand rather then showing it globally in the help thingie
        .arg(Arg::with_name("resource")
            .short("r")
            .long("resource")
            .help("Kubernetes resource to apply")
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

        .subcommand(SubCommand::with_name("token")
            .about("Generate github apps token")
            .arg(Arg::with_name("appid")
                .short("a")
                .long("appid")
                .help("Application ID for github apps")
                .takes_value(true)
                .env("GITHUB_APP_ID")
                .required(true))
            .arg(Arg::with_name("key")
                .short("k")
                .long("key")
                .help("Private key for github application")
                .takes_value(true)
                .env("GITHUB_APP_KEY")
                .required_unless("key-base64"))
            .arg(Arg::with_name("key-base64")
                .long("key-base64")
                .help("Private key for github applications, base64 encoded PEM")
                .takes_value(true)
                .env("GITHUB_APP_KEY_BASE64")
                .required_unless("key"))
            .arg(Arg::with_name("account")
                .long("account")
                .help("Account for the installation id")
                .takes_value(true)
                .env("ACCOUNT")
                .default_value("navikt")))
        .subcommand(SubCommand::with_name("deploy")
            .about("Command for github deployments")
            .subcommand(SubCommand::with_name("create")
                .about("Create a github deployment")
                .arg(Arg::with_name("username")
                    .short("u")
                    .long("username")
                    .takes_value(true)
                    .env("DEPLOYMENT_USERNAME")
                    .required_unless("appid")
                    .group("username-password-auth"))
                .arg(Arg::with_name("password")
                    .short("p")
                    .long("password")
                    .takes_value(true)
                    .env("DEPLOYMENT_PASSWORD")
                    .group("username-password-auth"))

                // GitHub apps support
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
                    .help("Private key for github application")
                    .takes_value(true)
                    .env("GITHUB_APP_KEY")
                    .required_unless_one(&["key-base64", "username"]))
                .arg(Arg::with_name("key-base64")
                    .long("key-base64")
                    .help("Private key for github applications, base64 encoded PEM")
                    .takes_value(true)
                    .env("GITHUB_APP_KEY_BASE64")
                    .required_unless_one(&["key", "username"]))

                .arg(Arg::with_name("repository")
                    .long("repository")
                    .help("Repository to create the deployment request on")
                    .takes_value(true)
                    .required(true)))
            .subcommand(SubCommand::with_name("payload")
                .about("Templates the deployment payload for the github deployment api, useful for manual curl calls/debugging")
                .arg(Arg::with_name("outputfile")
                    .short("o")
                    .long("outputfile")
                    .help("File to output to, if omitted it will print to stdout")
                    .takes_value(true))))

        .get_matches();

    if let Some(token_command) = matches.subcommand_matches("token") {
        handle_token_command(token_command);
    }

    if let Some(deploy_command) = matches.subcommand_matches("deploy") {
        handle_deploy_command(deploy_command);
    }

}

fn handle_token_command(subcommand: &ArgMatches) {
    let account = subcommand.value_of("account").unwrap();
    println!("{}", installation_token_for(subcommand, account).token);
}

fn handle_deploy_command(subcommand: &ArgMatches) {
    let reg = Handlebars::new();

    let mut config: Value = if let Some(config_path) = subcommand.value_of("config") {
        let file = File::open(config_path).expect(format!("Unable to open file {}", config_path).as_str());
        serde_json::from_reader(file).expect("Unable to parse json config")
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
    let team = subcommand.value_of("team")
        .expect("To create a deployment you need to specify a team");
    let version = subcommand.value_of("version")
        .expect("To create a deployment you need to specify a version");

    config["ref"] = Value::String(git_ref.to_owned());
    config["cluster"] = Value::String(cluster.to_owned());
    config["team"] = Value::String(team.to_owned());
    config["version"] = Value::String(version.to_owned());

    let resources: Vec<Value> = resource_matches
        .iter()
        .map(|v| File::open(v).expect(format!("Unable to open file {}", v).as_str()))
        .map(| mut f| {
            let mut string = String::new();
            f.read_to_string(&mut string).expect("Failed to read resource file");
            string
        })
        .map(|s| reg.render_template(s.as_str(), &config).expect("Failed to render template"))
        .inspect(|s| println!("{}", s))
        // TODO: Support json payloads
        .map(|s| serde_yaml::from_str(s.as_str()).expect("Unable to parse JSON from templated output"))
        .collect();

    let deployment_payload = DeploymentRequest {
        git_ref: git_ref.to_owned(),
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

    if let Some(_) = subcommand.subcommand_matches("payload") {
        println!("{}", serde_json::to_string(&deployment_payload).expect("Unable to write deployment request to stdout"))
    }

    if let Some(create_command) = subcommand.subcommand_matches("create") {
        let repository = create_command.value_of("repository").unwrap();

        let username = if let Some(username) = create_command.value_of("username") {
            username
        } else {
            "x-access-token"
        };

        let password = if let Some(password) = create_command.value_of("password") {
            password.to_owned()
        } else if create_command.is_present("appid") {
            let account = repository.split("/")
                .next()
                .expect("Repository format should be <user/org>/<repository>");
            installation_token_for(create_command, account).token
        } else {
            rpassword::read_password_from_tty(Some("Please enter github password: "))
                .expect("Failed to read password from stdin")
        };

        let deployment_response = client::create_deployment(repository, &deployment_payload, username, password.as_str())
            .expect("Failed to create deployment");
        println!("{:?}", deployment_response);
    }
}

fn installation_token_for(subcommand: &ArgMatches, account: &str) -> InstallationToken {
    let app_id = subcommand.value_of("appid").unwrap();
    let pem = extract_key(subcommand);


    fetch_installation_token(app_id, account, pem.as_slice())
        .expect("Failed to fetch installation token")
}

fn extract_key(subcommand: &ArgMatches) -> Vec<u8> {
    let binary = if let Some(app_key) = subcommand.value_of("key") {
        let mut bytes = vec![];

        File::open(app_key)
            .expect("Failed to open github apps key")
            .read_to_end(&mut bytes)
            .expect("Failed to read contents of github apps key");
        bytes
    } else {
        let key_base64 = subcommand.value_of("key-base64").unwrap();
        base64::decode(key_base64).expect("Failed to decode base64 pem file")
    };

    if let Ok(key_string) = String::from_utf8(binary.clone()) {
        if key_string.starts_with("-----BEGIN RSA PRIVATE KEY-----") {
            let base64 = key_string
                .replace("\r", "")
                .replace("\n", "");
            // Strip header and footer
            base64::decode(&base64[31..(key_string.len() - 56)]).unwrap()
        } else {
            binary
        }
    } else {
        binary
    }
}

fn fetch_installation_token(app_id: &str, account: &str, pem: &[u8]) -> Result<InstallationToken, client::ClientError> {
    let jwt = generate_jwt(app_id, pem);
    let installation = client::fetch_installations(jwt.as_str()).unwrap();

    let installation_id = installation.iter()
        .find(| v | v.account.login.as_str() == account)
        .expect(format!("Unable to find account {}", account).as_str())
        .id;
    client::fetch_installation_token(&installation_id, jwt.as_str())
}

fn generate_jwt(application_id: &str, private_key: &[u8]) -> String {
    let current_time_unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let jwt_claims = JwtClaims {
        iss: application_id.to_owned(),
        exp: current_time_unix+300,
        iat: current_time_unix
    };
    jwt::encode(&Header::new(Algorithm::RS256), &jwt_claims, private_key)
        .expect("Failed to encode jwt")
}
