use crate::deployment_client;
use clap::ArgMatches;
use failure::Error;

#[cfg(test)]
mod tests;

pub fn exchange_token_command(subcommand: &ArgMatches) -> Result<(), Error> {
    let repository = subcommand.value_of("repository").unwrap();
    let sinks: Vec<&str> = subcommand.values_of("sink").unwrap().collect();
    let sources: Vec<&str> = subcommand.values_of("src").unwrap().collect();
    let shared_secret = subcommand.value_of("shared-secret").unwrap();
    let team = subcommand.value_of("team")
        .ok_or(format_err!("To fetch a token for this deploy job you need to specify a team"))?;
    let correlation_id = subcommand.value_of("correlation-id").unwrap();

    let response = deployment_client::request_tokens(repository, sources, sinks, shared_secret, team, correlation_id)?;
    println!("{}", response);
    Ok(())
}
