use serde::Deserialize;

pub use anyhow::Result;
pub mod config;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SessionTokens {
    #[allow(dead_code)]
    credentials: Credentials,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Credentials {
    #[allow(dead_code)]
    access_key_id: String,
    #[allow(dead_code)]
    secret_access_key: String,
    #[allow(dead_code)]
    session_token: String,
    #[allow(dead_code)]
    expiration: String,
}
