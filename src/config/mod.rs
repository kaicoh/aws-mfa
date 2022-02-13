use lazy_static::lazy_static;
use regex::Regex;

pub mod mfa;

lazy_static! {
    static ref RE_PROFILE: Regex = Regex::new(r"\[(.+)\]").unwrap();
}

fn config_dir() -> String {
    let home = std::env::var("HOME").expect("env HOME is required");
    format!("{}/.aws", home)
}

fn capture_profile(line: &str) -> Option<&str> {
    capture_keywords(&RE_PROFILE, line)
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
