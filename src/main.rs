use anyhow::anyhow;
use aws_mfa::config;
use clap::{app_from_crate, Arg};
use serde::Deserialize;
use std::process::{Command, Output};

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
const PROFILE: &str = "profile";

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let matches = app_from_crate!()
        .arg(
            Arg::new(MFA_CODE)
                .value_name("MFA_CODE")
                .help("MFA one time pass code")
                .required(true),
        )
        .arg(
            Arg::new(PROFILE)
                .short('p')
                .long("profile")
                .takes_value(true)
                .value_name("PROFILE")
                .help("profile name in AWS CLI credentials"),
        )
        .get_matches();

    let code = matches.value_of(MFA_CODE).unwrap();
    let (use_profile, profile) = match matches.value_of(PROFILE) {
        Some(p) => (true, p),
        None => (false, "default"),
    };

    let device_arn = config::mfa::get_device_arn(profile)?;
    let Output {
        status,
        stdout,
        stderr,
    } = Command::new("aws")
        .arg("sts")
        .arg("get-session-token")
        .args(["--serial-number", &device_arn])
        .args(["--token-code", code])
        .args(profile_args(use_profile, profile))
        .output()?;

    if status.success() {
        let SessionTokens { credentials } = serde_json::from_slice(&stdout)?;
        println!("AccessKeyId: {}", credentials.access_key_id);
        println!("SecretAccessKey: {}", credentials.secret_access_key);
        println!("SessionToken: {}", credentials.session_token);
        Ok(())
    } else {
        Err(anyhow!("{}", String::from_utf8(stderr)?))
    }
}

fn profile_args(use_profile: bool, profile: &str) -> Vec<&str> {
    if use_profile {
        vec!["--profile", profile]
    } else {
        vec![]
    }
}
