use lazy_static::lazy_static;
use regex::Regex;
use std::io::BufRead;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

lazy_static! {
    static ref RE_USERNAME: Regex = Regex::new(r"\[(.+)\]").unwrap();
    static ref RE_DEVICE_ARN: Regex = Regex::new(r"^device_arn=(.+)").unwrap();
}

#[derive(Debug)]
enum ConfigRow {
    Username(String),
    DeviceArn(String),
    Other,
}

#[derive(Debug)]
pub struct Config {
    username: String,
    device_arn: String,
}

pub fn get_device_arn(username: &str, configs: Vec<Config>) -> Option<String> {
    configs
        .iter()
        .find(|conf| conf.username == username)
        .map(|conf| conf.device_arn.to_string())
}

pub fn read_config(read: Box<dyn BufRead>) -> Result<Vec<Config>> {
    let mut configs: Vec<Config> = vec![];
    let mut username: String = "".to_string();

    for line in read.lines() {
        match read_config_line(&line?) {
            ConfigRow::Username(_username) => {
                username = _username;
            },
            ConfigRow::DeviceArn(device_arn) => {
                if !username.is_empty() {
                    configs.push(Config {
                        username: username.to_string(),
                        device_arn,
                    });

                    username = "".to_string();
                }
            },
            _ => {},
        }
    }

    Ok(configs)
}

fn read_config_line(line: &str) -> ConfigRow {
    if let Some(username) = capture_username(line) {
        return ConfigRow::Username(username.to_string());
    }

    if let Some(device_arn) = capture_device_arn(line) {
        return ConfigRow::DeviceArn(device_arn.to_string());
    }

    ConfigRow::Other
}

fn capture_username(line: &str) -> Option<&str> {
    capture_keywords(&RE_USERNAME, line)
}

fn capture_device_arn(line: &str) -> Option<&str> {
    capture_keywords(&RE_DEVICE_ARN, line)
}

fn capture_keywords<'a, 'b>(reg: &'a Regex, line: &'b str) -> Option<&'b str> {
    reg.captures(line)
        .map(|caps| caps.get(1))
        .flatten()
        .map(|mat| mat.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod capture_username {
        use super::*;

        #[test]
        fn it_returns_none_when_not_match_regexp() {
            assert!(capture_username("").is_none());
        }

        #[test]
        fn it_returns_username_from_captures() {
            assert_eq!(
                capture_username("[tanaka]").unwrap(),
                "tanaka"
            );
        }
    }

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
        use std::fs::File;
        use std::io::BufReader;

        #[test]
        fn it_read_config_with_one_user() {
            let reader = BufReader::new(File::open("mock/test-config1").unwrap());
            let result = read_config(Box::new(reader));
            assert!(result.is_ok());

            let configs = result.unwrap();
            assert_eq!(configs.len(), 1);

            let config = configs.get(0).unwrap();
            assert_eq!(config.username, "tanaka");
            assert_eq!(config.device_arn, "arn:aws:iam::012345678901:mfa/tanaka");
        }

        #[test]
        fn it_read_config_with_multiple_users() {
            let reader = BufReader::new(File::open("mock/test-config2").unwrap());
            let result = read_config(Box::new(reader));
            assert!(result.is_ok());

            let configs = result.unwrap();
            assert_eq!(configs.len(), 2);

            let config = configs.get(0).unwrap();
            assert_eq!(config.username, "tanaka");
            assert_eq!(config.device_arn, "arn:aws:iam::012345678901:mfa/tanaka");

            let config = configs.get(1).unwrap();
            assert_eq!(config.username, "satoh");
            assert_eq!(config.device_arn, "arn:aws:iam::012345678901:mfa/satoh");
        }
    }

    mod get_device_arn {
        use super::*;

        #[test]
        fn it_finds_device_arn_from_configs() {
            let result = get_device_arn("suzuki", configs());
            assert!(result.is_some());
            assert_eq!(result.unwrap(), "suzuki-device");
        }

        #[test]
        fn it_returns_none_when_not_found_device_arn() {
            let result = get_device_arn("satoh", configs());
            assert!(result.is_none());
        }

        fn configs() -> Vec<Config> {
            vec![
                Config{ username: "tanaka".to_owned(), device_arn: "tanaka-device".to_owned() },
                Config{ username: "suzuki".to_owned(), device_arn: "suzuki-device".to_owned() },
            ]
        }
    }
}
