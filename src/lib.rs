use clap::ArgMatches;
use config::credentials::Credential as AwsCredential;
use config::mfa::Config;
use serde::Deserialize;

pub use anyhow::Result;
pub mod config;

pub const ARG_MFA_CODE: &str = "mfa_code";
pub const ARG_PROFILE: &str = "profile";
pub const ARG_MFA_PROFILE: &str = "mfa-profile";
pub const ARG_DURATION: &str = "duration";
pub const ARG_BACKUP_FILE: &str = "backup_file";

pub const DEFAULT_MFA_PROFILE: &str = "mfa";
pub const DEFAULT_DURATION: &str = "900";
pub const DEFAULT_BACKUP_FILE: &str = "credentials_bk";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SessionTokens {
    credentials: Credentials,
}

impl SessionTokens {
    pub fn to_aws_credential(&self, profile: &str) -> AwsCredential {
        let Credentials {
            access_key_id,
            secret_access_key,
            session_token,
            ..
        } = &self.credentials;

        let lines = vec![
            format!("aws_access_key_id={}", access_key_id),
            format!("aws_secret_access_key={}", secret_access_key),
            format!("aws_session_token={}", session_token),
        ];

        AwsCredential::new(profile, &lines)
    }
}

// AWS Credentials
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Credentials {
    access_key_id: String,
    secret_access_key: String,
    session_token: String,
    #[allow(dead_code)]
    expiration: String,
}

// CLI Options
#[derive(Debug)]
pub struct Options<'a> {
    matches: &'a ArgMatches,
    config: &'a Config,
}

impl<'a> Options<'a> {
    pub fn new(matches: &'a ArgMatches, config: &'a Config) -> Self {
        Self { matches, config }
    }

    pub fn backup_file(&self) -> String {
        if let Some(f) = self.matches.value_of(ARG_BACKUP_FILE) {
            return f.to_string();
        }

        if let Some(f) = &self.config.backup_file {
            return f.to_string();
        }

        DEFAULT_BACKUP_FILE.to_string()
    }

    pub fn mfa_profile(&self) -> String {
        if let Some(p) = self.matches.value_of(ARG_MFA_PROFILE) {
            return p.to_string();
        }

        if let Some(p) = &self.config.mfa_profile {
            return p.to_string();
        }

        DEFAULT_MFA_PROFILE.to_string()
    }

    pub fn duration(&self) -> String {
        if let Some(d) = self.matches.value_of(ARG_DURATION) {
            return d.to_string();
        }

        if let Some(d) = &self.config.duration {
            return d.to_string();
        }

        DEFAULT_DURATION.to_string()
    }
}
