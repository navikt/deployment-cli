use crate::client;
use crate::models::{JwtClaims, InstallationToken};

use std::fs::File;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::ArgMatches;
use failure::{Error, ResultExt};
use jwt::{Header, Algorithm};

#[cfg(test)]
mod tests;

pub fn handle_token_command(subcommand: &ArgMatches) -> Result<(), Error> {
    let account = subcommand.value_of("account").unwrap();
    println!("{}", installation_token_for(subcommand, account)?.token);
    Ok(())
}

pub fn installation_token_for(subcommand: &ArgMatches, account: &str) -> Result<InstallationToken, Error> {
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