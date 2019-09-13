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
    let mut config: Value = if let Some(config_path) = subcommand.value_of("variables") {
        let file = File::open(config_path).context(format!("Unable to open resource file {}", config_path))?;
        serde_json::from_reader(file).context(format!("Unable to parse json config {}", config_path))?
    } else {
        Value::Null
    };

    let git_ref = subcommand.value_of("ref").unwrap();
    let cluster = subcommand.value_of("cluster").unwrap();
    let auto_merge: bool = subcommand.value_of("auto-merge").unwrap()
        .parse()
        .unwrap();
    let team = subcommand.value_of("team")
        .ok_or(format_err!("To create a deployment you need to specify a team"))?;

    config["ref"] = Value::String(git_ref.to_owned());
    config["cluster"] = Value::String(cluster.to_owned());
    config["team"] = Value::String(team.to_owned());

    if let Some(version) = subcommand.value_of("version") {
        config["version"] = Value::String(version.to_owned());
    }

    if let Some(overrides) = subcommand.values_of("var") {
        for var in overrides {
            let equals_index = var.find('=')
                .ok_or(format_err!("Invalid format for variable override, expected <name>=<value>"))?;
            let key = &var[0..equals_index];
            let value = &var[equals_index+1..];
            config[key] = Value::String(value.to_owned());
        }
    }

    let resources = get_resources(subcommand, &config);

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
                resources: resources?
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

fn get_resources(subcommand: &ArgMatches, config: &Value) -> Result<Vec<Value>, Error> {
    let reg = Handlebars::new();

    let resource_matches: Vec<&str> = if let Some(values) = subcommand.values_of("resource") {
        values.collect()
    } else {
        vec![]
    };

    let raw_resource_matches: Vec<&str> = if let Some(values) = subcommand.values_of("raw-resource") {
        values.collect()
    } else {
        vec![]
    };

    let mut result: Vec<Value> = Vec::new();
    for file_name in resource_matches {
        let mut file = File::open(file_name)
            .context(format!("Unable to open placeholder file {}", file_name))?;

        let mut resource_template = String::new();
        file.read_to_string(&mut resource_template)
            .context(format!("Failed to read resource file {}", file_name))?;

        let resource = reg.render_template(resource_template.as_str(), config)
            .context(format!("Failed to render template for file {}", file_name))?;

        let value: Value = serde_yaml::from_str(resource.as_str()).context(format!("Failed to parse json for {}", file_name))?;

        if let Some(values) = value.as_array() {
            result.extend_from_slice(values.as_slice());
        } else {
            result.push(value);
        }
    }

    for file_name in raw_resource_matches {
        let file = File::open(file_name)?;

        let value: Value = serde_yaml::from_reader(&file)?;
        result.push(value)
    }
    Ok(result)
}
