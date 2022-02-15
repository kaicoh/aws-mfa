use crate::Result;

use anyhow::anyhow;
use lazy_static::lazy_static;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::string::ToString;

lazy_static! {
    static ref RE_PROFILE: Regex = Regex::new(r"\[(.+)\]").unwrap();
}

#[derive(Debug)]
pub struct ConfigFile {
    credentials: Vec<Credential>,
}

impl ConfigFile {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let reader = BufReader::new(File::open(path)?);
        let mut credentials: Vec<Credential> = Vec::new();
        let mut profile = "".to_string();
        let mut lines: Vec<String> = Vec::new();

        for l in reader.lines() {
            let line = l?;

            if let Some(p) = capture_profile(&line) {
                Self::add_credential(&profile, &lines, &mut credentials);

                profile = p.to_string();
                lines = Vec::new();
            } else if !line.is_empty() {
                lines.push(line)
            }
        }

        Self::add_credential(&profile, &lines, &mut credentials);

        Ok(ConfigFile { credentials })
    }

    fn add_credential(p: &str, ls: &[String], creds: &mut Vec<Credential>) {
        if !p.is_empty() {
            let cred = Credential::new(p, ls);
            creds.push(cred);
        }
    }

    pub fn remove_credential(self, profile: &str) -> Self {
        let credentials = self
            .credentials
            .into_iter()
            .filter(|cred| cred.profile != profile)
            .collect();
        Self { credentials }
    }

    pub fn set_credential(self, cred: Credential) -> Self {
        let mut credentials = self.credentials;
        credentials.push(cred);
        Self { credentials }
    }

    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        std::fs::write(path, self.to_string())
            .map_err(|e| anyhow!("Error writing to credentials: {}", e))
    }
}

impl ToString for ConfigFile {
    fn to_string(&self) -> String {
        self.credentials
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join("\n\n")
    }
}

#[derive(Debug)]
pub struct Credential {
    profile: String,
    lines: Vec<String>,
}

impl Credential {
    pub fn new(profile: &str, lines: &[String]) -> Self {
        Self {
            profile: profile.to_string(),
            lines: lines.to_owned(),
        }
    }
}

impl ToString for Credential {
    fn to_string(&self) -> String {
        format!("[{}]\n{}", self.profile, self.lines.join("\n"))
    }
}

pub fn copy_credentials(backup: &str) -> Result<()> {
    let org_path = credentials_path();
    let backup_path = super::config_file(backup);
    std::fs::copy(org_path, backup_path)
        .map(drop)
        .map_err(anyhow::Error::new)
}

pub fn credentials_path() -> PathBuf {
    super::config_file("credentials")
}

fn capture_profile(line: &str) -> Option<&str> {
    RE_PROFILE
        .captures(line)
        .map(|caps| caps.get(1))
        .flatten()
        .map(|mat| mat.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod configfile {
        use super::*;

        #[test]
        fn it_gets_configfile_from_path() {
            let result = ConfigFile::from_path("mock/test-credentials1");
            assert!(result.is_ok());

            let ConfigFile { credentials } = result.unwrap();
            assert_eq!(credentials.len(), 2);

            let cred = credentials.get(0).unwrap();
            assert_eq!(cred.profile, "tanaka");
            assert_eq!(
                cred.lines,
                vec![
                    "aws_access_key_id=ABCDEFGHIJKLMNOPQRST",
                    "aws_secret_access_key=abcdefghijklmnopqrstuvwxyz+-#$1234567890",
                ]
            );

            let cred = credentials.get(1).unwrap();
            assert_eq!(cred.profile, "suzuki");
            assert_eq!(cred.lines, vec!["xxxxxxxxxxxxxxxx", "yyyyyyyyyyyy",]);
        }

        #[test]
        fn it_remove_credential_when_found_profile() {
            let config = configfile();
            let ConfigFile { credentials } = config.remove_credential("tanaka");
            assert_eq!(credentials.len(), 1);

            let cred = credentials.get(0).unwrap();
            assert_eq!(cred.profile, "suzuki");
            assert_eq!(cred.lines, vec!["foobar", "barbaz"]);
        }

        #[test]
        fn it_does_not_remove_credential_when_not_found_profile() {
            let config = configfile();
            let ConfigFile { credentials } = config.remove_credential("satoh");
            assert_eq!(credentials.len(), 2);
        }

        #[test]
        fn it_sets_credential() {
            let config = configfile();
            let cred = Credential::new("satoh", &vec!["foobarbaz".to_owned()]);
            let ConfigFile { credentials } = config.set_credential(cred);
            assert_eq!(credentials.len(), 3);
        }

        #[test]
        fn it_writes() {
            let config = ConfigFile {
                credentials: vec![
                    Credential::new("tanaka", &vec!["foobarbaz".to_owned()]),
                    Credential::new("takahashi", &vec!["foo".to_owned(), "bar".to_owned()]),
                    Credential::new("saito", &vec![]),
                ],
            };

            let path = "mock/test-credentials2";
            config.write(path).unwrap();
            let content = std::fs::read_to_string(path).unwrap();
            assert_eq!(content, config.to_string());
        }

        fn configfile() -> ConfigFile {
            ConfigFile {
                credentials: vec![
                    Credential::new("tanaka", &vec!["foo".to_owned(), "bar".to_owned()]),
                    Credential::new("suzuki", &vec!["foobar".to_owned(), "barbaz".to_owned()]),
                ],
            }
        }
    }

    mod credential {
        use super::*;

        #[test]
        fn it_returns_string() {
            let cred = Credential::new("tanaka", &vec!["foo".to_owned(), "bar".to_owned()]);
            assert_eq!(cred.to_string(), "[tanaka]\nfoo\nbar");
        }
    }

    mod capture_profile {
        use super::*;

        #[test]
        fn it_returns_none_when_not_match_regexp() {
            assert!(capture_profile("").is_none());
        }

        #[test]
        fn it_returns_profile_from_captures() {
            assert_eq!(capture_profile("[tanaka]").unwrap(), "tanaka");
        }
    }
}
