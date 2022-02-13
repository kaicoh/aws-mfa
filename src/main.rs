use anyhow::anyhow;
use aws_mfa::{config, Result, SessionTokens};
use clap::{app_from_crate, Arg};
use std::process::{Command, Output};

const ARG_MFA_CODE: &str = "mfa_code";
const ARG_PROFILE: &str = "profile";
const ARG_DURATION: &str = "duration";

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let matches = app_from_crate!()
        .arg(
            Arg::new(ARG_MFA_CODE)
                .value_name("MFA_CODE")
                .help("MFA one time pass code")
                .required(true),
        )
        .arg(
            Arg::new(ARG_PROFILE)
                .short('p')
                .long("profile")
                .takes_value(true)
                .value_name("PROFILE")
                .help("profile name in AWS CLI credentials"),
        )
        .arg(
            Arg::new(ARG_DURATION)
                .short('d')
                .long("duration-seconds")
                .takes_value(true)
                .value_name("DURATION")
                .help("expiration duration (in seconds)")
                .default_value("900"),
        )
        .get_matches();

    let code = matches.value_of(ARG_MFA_CODE).unwrap();

    // Ref: https://aws.amazon.com/premiumsupport/knowledge-center/authenticate-mfa-cli/?nc1=h_ls
    // root user: 900(15 minutes) <= duration <= 3600(1 hour)
    // other: 900(15 minutes) <= duration <= 129600(36 hours)
    let duration = matches
        .value_of(ARG_DURATION)
        .unwrap()
        .parse::<u32>()
        .map_err(|e| anyhow!("Parse error: cannot parse duration (in seconds): {}", e))?;

    let (use_profile, profile) = match matches.value_of(ARG_PROFILE) {
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
        .args(["--duration-seconds", duration.to_string().as_ref()])
        .args(profile_args(use_profile, profile))
        .output()?;

    if status.success() {
        let tokens: SessionTokens = serde_json::from_slice(&stdout)?;
        println!("{:#?}", tokens);
        config::credentials::copy_credentials()
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
