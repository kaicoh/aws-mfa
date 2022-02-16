use anyhow::anyhow;
use aws_mfa::config::credentials::{
    copy_credentials as backup_credentials, credentials_path, ConfigFile as CredFile,
};
use aws_mfa::config::mfa::Config as MfaConfig;
use aws_mfa::{config, Result, SessionTokens};
use clap::{app_from_crate, Arg, ArgMatches};
use std::process::{Command, Output};

const ARG_MFA_CODE: &str = "mfa_code";
const ARG_PROFILE: &str = "profile";
const ARG_MFA_PROFILE: &str = "mfa-profile";
const ARG_DURATION: &str = "duration";
const ARG_BACKUP_FILE: &str = "backup_file";

const DEFAULT_MFA_PROFILE: &str = "mfa";
const DEFAULT_DURATION: &str = "900";
const DEFAULT_BACKUP_FILE: &str = "credentials_bk";

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
                .help(
                    format!(
                        "expiration duration(in seconds) [default: {}]",
                        DEFAULT_DURATION
                    )
                    .as_ref(),
                ),
        )
        .arg(
            Arg::new(ARG_MFA_PROFILE)
                .short('m')
                .long("mfa-profile")
                .takes_value(true)
                .value_name("MFA_PROFILE")
                .help(
                    format!(
                        "profile name for mfa credentials [default: {}]",
                        DEFAULT_MFA_PROFILE
                    )
                    .as_ref(),
                ),
        )
        .arg(
            Arg::new(ARG_BACKUP_FILE)
                .short('b')
                .long("backup")
                .takes_value(true)
                .value_name("BACKUP FILE")
                .help(
                    format!(
                        "filename for credentials backup [default: {}]",
                        DEFAULT_BACKUP_FILE
                    )
                    .as_ref(),
                ),
        )
        .get_matches();

    let code = matches.value_of(ARG_MFA_CODE).unwrap();
    let config = MfaConfig::read()?;

    let mfa_profile = get_mfa_profile(&matches, &config);
    let backup = get_backup_name(&matches, &config);

    // Ref: https://aws.amazon.com/premiumsupport/knowledge-center/authenticate-mfa-cli/?nc1=h_ls
    // root user: 900(15 minutes) <= duration <= 3600(1 hour)
    // other: 900(15 minutes) <= duration <= 129600(36 hours)
    let duration = get_duration(&matches, &config)
        .parse::<u32>()
        .map_err(|e| anyhow!("Parse error: cannot parse duration (in seconds): {}", e))?;

    let (use_profile, profile) = match matches.value_of(ARG_PROFILE) {
        Some(p) => (true, p),
        None => (false, "default"),
    };

    let device_arn = config::mfa::get_device_arn(profile, &config)?;
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

        backup_credentials(&backup)?;
        write_mfa_credentials(&mfa_profile, &tokens)
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

fn get_backup_name(matches: &ArgMatches, config: &MfaConfig) -> String {
    if let Some(f) = matches.value_of(ARG_BACKUP_FILE) {
        return f.to_string();
    }

    if let Some(f) = &config.backup_file {
        return f.to_string();
    }

    DEFAULT_BACKUP_FILE.to_string()
}

fn get_mfa_profile(matches: &ArgMatches, config: &MfaConfig) -> String {
    if let Some(p) = matches.value_of(ARG_MFA_PROFILE) {
        return p.to_string();
    }

    if let Some(p) = &config.mfa_profile {
        return p.to_string();
    }

    DEFAULT_MFA_PROFILE.to_string()
}

fn get_duration(matches: &ArgMatches, config: &MfaConfig) -> String {
    if let Some(d) = matches.value_of(ARG_DURATION) {
        return d.to_string();
    }

    if let Some(d) = &config.duration {
        return d.to_string();
    }

    DEFAULT_DURATION.to_string()
}

fn write_mfa_credentials(mfa_profile: &str, tokens: &SessionTokens) -> Result<()> {
    let cred = tokens.to_aws_credential(mfa_profile);
    let config = CredFile::from_path(credentials_path())?;

    config
        .remove_credential(mfa_profile)
        .set_credential(cred)
        .write(credentials_path())
}
