use serde::{de, Deserialize, Deserializer, Serializer};
use std::time::Duration;

#[derive(Debug)]
struct InvalidDuration;

impl std::fmt::Display for InvalidDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid duration format")
    }
}

impl std::error::Error for InvalidDuration {}

pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&duration_to_string(duration))
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    string_to_duration(&s).map_err(|e| de::Error::custom(format!("{}", e)))
}

fn string_to_duration(s: &str) -> Result<Duration, InvalidDuration> {
    let multiplier = match s.chars().last() {
        Some('s') => Ok(Duration::from_secs(1)),
        Some('m') => Ok(Duration::from_secs(60)),
        Some('h') => Ok(Duration::from_secs(3600)),
        _ => Err(InvalidDuration),
    }?;
    let value = s[..s.len() - 1]
        .parse::<u64>()
        .map_err(|_| InvalidDuration)?;
    Ok(Duration::from_secs(multiplier.as_secs() * value))
}

fn duration_to_string(duration: &Duration) -> String {
    if duration.as_secs() % 3600 == 0 {
        format!("{}h", duration.as_secs() / 3600)
    } else if duration.as_secs() % 60 == 0 {
        format!("{}m", duration.as_secs() / 60)
    } else {
        format!("{}s", duration.as_secs())
    }
}
