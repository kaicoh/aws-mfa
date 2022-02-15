use config::credentials::Credential as AwsCredential;
use serde::Deserialize;

pub use anyhow::Result;
pub mod config;

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Credentials {
    access_key_id: String,
    secret_access_key: String,
    session_token: String,
    #[allow(dead_code)]
    expiration: String,
}
