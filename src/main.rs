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
mod models;
mod cli;

fn main() {
    cli::execute_command(cli::create_cli_app().get_matches());
}
