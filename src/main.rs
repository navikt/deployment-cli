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

#[cfg(test)]
#[macro_use]
mod test_helpers;

mod github_client;
mod deployment_client;
mod models;
mod cli;

fn main() {
    if let Err(err) = cli::execute_command(&cli::create_cli_app().get_matches()) {
        println!("Error: {}", err.as_fail());
        for cause in err.iter_causes() {
            println!("Caused by:");
            println!("{}", cause);
        }
        println!("{}", err.backtrace());
        ::std::process::exit(1);
    }
}
