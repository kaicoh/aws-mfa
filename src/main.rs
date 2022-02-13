use clap::{app_from_crate, Arg};
use serde::Deserialize;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

type Result<T> = aws_mfa::Result<T>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SessionTokens {
    credentials: Credentials,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Credentials {
    access_key_id: String,
    secret_access_key: String,
    session_token: String,
    #[allow(dead_code)]
    expiration: String,
}

const MFA_CODE: &str = "mfa_code";

fn main() -> Result<()> {
    let matches = app_from_crate!()
        .arg(
            Arg::new(MFA_CODE)
                .value_name("MFA_CODE")
                .help("MFA code")
                .required(true),
        )
        .get_matches();

    let code = matches.value_of(MFA_CODE).unwrap();

    let configfile = get_configfile()?;
    let device_arn = get_device_arn("default", configfile)?;

    let output = Command::new("aws")
        .arg("sts")
        .arg("get-session-token")
        .args(["--serial-number", &device_arn])
        .args(["--token-code", code])
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                let SessionTokens { credentials } = serde_json::from_slice(&out.stdout)?;
                println!("AccessKeyId: {}", credentials.access_key_id);
                println!("SecretAccessKey: {}", credentials.secret_access_key);
                println!("SessionToken: {}", credentials.session_token);
            } else {
                eprintln!("{}", String::from_utf8(out.stderr)?);
            }
        }
        Err(err) => eprintln!("{}", err),
    }

    Ok(())
}

fn get_device_arn(user: &str, read: Box<dyn BufRead>) -> Result<String> {
    let configs = aws_mfa::config::mfa::read_config(read)?;
    match aws_mfa::config::mfa::get_device_arn(user, configs) {
        Some(device_arn) => Ok(device_arn),
        None => panic!("Not Found mfa device arn for {}", user),
    }
}

fn get_configfile() -> Result<Box<dyn BufRead>> {
    let home = std::env::var("HOME").expect("env HOME is required");
    let filepath = format!("{}/.aws/mfa-config", home);
    let file = File::open(Path::new(&filepath))?;
    Ok(Box::new(BufReader::new(file)))
}
