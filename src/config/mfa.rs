use anyhow::anyhow;
use lazy_static::lazy_static;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

type Result<T> = crate::Result<T>;

lazy_static! {
    static ref RE_DEVICE_ARN: Regex = Regex::new(r"^device_arn=(.+)").unwrap();
}

#[derive(Debug)]
enum ConfigRow {
    Profile(String),
    DeviceArn(String),
    Other,
}

#[derive(Debug)]
struct Config {
    profile: String,
    device_arn: String,
}

pub fn get_device_arn(profile: &str) -> Result<String> {
    let configs = get_file().and_then(read_config)?;
    search_device_arn(profile, configs)
        .ok_or_else(|| anyhow!("Not Found mfa device arn for profile: {}", profile))
}

fn get_file() -> Result<BufReader<File>> {
    let path = Path::new(&super::config_dir()).join("mfa-config");
    Ok(BufReader::new(File::open(path)?))
}

fn search_device_arn(profile: &str, configs: Vec<Config>) -> Option<String> {
    configs
        .iter()
        .find(|conf| conf.profile == profile)
        .map(|conf| conf.device_arn.to_string())
}

fn read_config(read: BufReader<File>) -> Result<Vec<Config>> {
    let mut configs: Vec<Config> = vec![];
    let mut profile: String = "".to_string();

    for line in read.lines() {
        match read_config_line(&line?) {
            ConfigRow::Profile(_profile) => {
                profile = _profile;
            }
            ConfigRow::DeviceArn(device_arn) => {
                if !profile.is_empty() {
                    configs.push(Config {
                        profile,
                        device_arn,
                    });
                    profile = "".to_string();
                }
            }
            _ => {}
        }
    }

    Ok(configs)
}

fn read_config_line(line: &str) -> ConfigRow {
    if let Some(profile) = super::capture_profile(line) {
        return ConfigRow::Profile(profile.to_string());
    }

    if let Some(device_arn) = capture_device_arn(line) {
        return ConfigRow::DeviceArn(device_arn.to_string());
    }

    ConfigRow::Other
}

fn capture_device_arn(line: &str) -> Option<&str> {
    super::capture_keywords(&RE_DEVICE_ARN, line)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod capture_device_arn {
        use super::*;

        #[test]
        fn it_returns_none_when_not_match_regexp() {
            assert!(capture_device_arn("").is_none());
        }

        #[test]
        fn it_returns_device_arn_from_captures() {
            assert_eq!(
                capture_device_arn("device_arn=arn:aws:iam::012345678901:mfa/tanaka").unwrap(),
                "arn:aws:iam::012345678901:mfa/tanaka",
            );
        }
    }

    mod read_config {
        use super::*;

        #[test]
        fn it_read_config_with_one_user() {
            let result = read_file("mock/test-config1");
            assert!(result.is_ok());

            let configs = result.unwrap();
            assert_eq!(configs.len(), 1);

            let config = configs.get(0).unwrap();
            assert_eq!(config.profile, "tanaka");
            assert_eq!(config.device_arn, "arn:aws:iam::012345678901:mfa/tanaka");
        }

        #[test]
        fn it_read_config_with_multiple_users() {
            let result = read_file("mock/test-config2");
            assert!(result.is_ok());

            let configs = result.unwrap();
            assert_eq!(configs.len(), 2);

            let config = configs.get(0).unwrap();
            assert_eq!(config.profile, "tanaka");
            assert_eq!(config.device_arn, "arn:aws:iam::012345678901:mfa/tanaka");

            let config = configs.get(1).unwrap();
            assert_eq!(config.profile, "satoh");
            assert_eq!(config.device_arn, "arn:aws:iam::012345678901:mfa/satoh");
        }

        fn read_file(path: &str) -> Result<Vec<Config>> {
            let reader = BufReader::new(File::open(path).unwrap());
            read_config(reader)
        }
    }

    mod search_device_arn {
        use super::*;

        #[test]
        fn it_finds_device_arn_from_configs() {
            let result = search_device_arn("suzuki", configs());
            assert!(result.is_some());
            assert_eq!(result.unwrap(), "suzuki-device");
        }

        #[test]
        fn it_returns_none_when_not_found_device_arn() {
            let result = search_device_arn("satoh", configs());
            assert!(result.is_none());
        }

        fn configs() -> Vec<Config> {
            vec![
                Config {
                    profile: "tanaka".to_owned(),
                    device_arn: "tanaka-device".to_owned(),
                },
                Config {
                    profile: "suzuki".to_owned(),
                    device_arn: "suzuki-device".to_owned(),
                },
            ]
        }
    }
}
