mod create;
#[cfg(test)]
mod tests;

use std::fs::{OpenOptions, File};
use std::io::Read;

use clap::ArgMatches;
use failure::{Error, ResultExt};
use handlebars::Handlebars;
use serde_json::Value;

use crate::models::{DeploymentRequest, Kubernetes, Payload};

pub fn handle_deploy_command(subcommand: &ArgMatches) -> Result<(), Error> {
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
        create::handle_deploy_create_command(create_command, &deployment_payload)?;
    };
    Ok(())
}