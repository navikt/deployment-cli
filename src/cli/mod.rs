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
                ::std::process::exit(1);
            },
            Ok(value) => value
        }
    }
}

mod deploy;
pub mod token;

use clap::{App, Arg, SubCommand, ArgMatches};

const ALLOWED_CLUSTERS: &[&'static str] = &["dev-fss", "dev-sbs", "prod-fss", "prod-sbs", "staging-gcp", "dev-gcp", "prod-gcp"];

pub fn execute_command(args: ArgMatches) {
    if let Some(token_command) = args.subcommand_matches("token") {
        exit_on_err!(token::handle_token_command(token_command));
    }

    if let Some(deploy_command) = args.subcommand_matches("deploy") {
        exit_on_err!(deploy::handle_deploy_command(deploy_command));
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

pub fn create_cli_app<'a, 'b>() -> App<'a, 'b> {
    let application_version = option_env!("CIRCLE_TAG")
        .unwrap_or(option_env!("CIRCLE_SHA1").unwrap_or("Unknown"));

    App::new("deployment-cli")
        .version(application_version)
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
            .help("DEPRECATED: Version number to be deployed")
            .takes_value(true)
            .global(true))
        .arg(Arg::with_name("var")
            .long("var")
            .help("Overrides templating variable")
            .multiple(true)
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