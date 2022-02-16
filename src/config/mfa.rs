use crate::Result;

use anyhow::anyhow;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    devices: Vec<Device>,
    pub backup_file: Option<String>,
    pub duration: Option<String>,
    pub mfa_profile: Option<String>,
}

impl Config {
    pub fn read() -> Result<Self> {
        let path = super::config_file("mfa.yml");
        get_config(path)
    }
}

#[derive(Debug, Deserialize)]
struct Device {
    profile: String,
    arn: String,
}

pub fn get_device_arn(profile: &str, config: &Config) -> Result<String> {
    search_device_arn(profile, config)
        .ok_or_else(|| anyhow!("Not Found mfa device arn for profile: {}", profile))
}

fn get_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let conf = std::fs::read_to_string(&path)
        .map_err(|e| anyhow!("{}: {}", e, path.as_ref().to_str().unwrap()))?;
    serde_yaml::from_str(&conf).map_err(anyhow::Error::new)
}

fn search_device_arn(profile: &str, config: &Config) -> Option<String> {
    config
        .devices
        .iter()
        .find(|device| device.profile == profile)
        .map(|device| device.arn.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod get_config {
        use super::*;

        #[test]
        fn it_read_config_with_one_profile() {
            let result = get_config("mock/test-config1.yml");
            assert!(result.is_ok());

            let config = result.unwrap();
            assert_eq!(config.devices.len(), 1);
            assert!(config.backup_file.is_none());
            assert!(config.duration.is_none());
            assert!(config.mfa_profile.is_none());

            let device = config.devices.get(0).unwrap();
            assert_eq!(device.profile, "tanaka");
            assert_eq!(device.arn, "arn:aws:iam::012345678901:mfa/tanaka");
        }

        #[test]
        fn it_read_config_with_multiple_profiles() {
            let result = get_config("mock/test-config2.yml");
            assert!(result.is_ok());

            let config = result.unwrap();
            assert_eq!(config.devices.len(), 2);
            assert_eq!(config.backup_file, Some("test_bk".to_owned()));
            assert_eq!(config.duration, Some("1000".to_owned()));
            assert_eq!(config.mfa_profile, Some("test_mfa".to_owned()));

            let device = config.devices.get(0).unwrap();
            assert_eq!(device.profile, "tanaka");
            assert_eq!(device.arn, "arn:aws:iam::012345678901:mfa/tanaka");

            let device = config.devices.get(1).unwrap();
            assert_eq!(device.profile, "satoh");
            assert_eq!(device.arn, "arn:aws:iam::012345678901:mfa/satoh");
        }
    }

    mod search_device_arn {
        use super::*;

        #[test]
        fn it_finds_device_arn_from_configs() {
            let result = search_device_arn("suzuki", &test_config());
            assert!(result.is_some());
            assert_eq!(result.unwrap(), "suzuki-device");
        }

        #[test]
        fn it_returns_none_when_not_found_device_arn() {
            let result = search_device_arn("satoh", &test_config());
            assert!(result.is_none());
        }

        fn test_config() -> Config {
            Config {
                devices: vec![
                    Device {
                        profile: "tanaka".to_owned(),
                        arn: "tanaka-device".to_owned(),
                    },
                    Device {
                        profile: "suzuki".to_owned(),
                        arn: "suzuki-device".to_owned(),
                    },
                ],
                backup_file: None,
                duration: None,
                mfa_profile: None,
            }
        }
    }
}
