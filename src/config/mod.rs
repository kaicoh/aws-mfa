use lazy_static::lazy_static;
use std::path::{Path, PathBuf};

pub mod credentials;
pub mod mfa;

lazy_static! {
    static ref CONF_DIR: String = {
        let home = std::env::var("HOME").expect("env HOME is required");
        format!("{}/.aws", home)
    };
}

fn config_file(filename: &str) -> PathBuf {
    Path::new(&*CONF_DIR).join(filename)
}
