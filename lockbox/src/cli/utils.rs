use std::str::FromStr;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use humantime::parse_duration;
use validator::ValidationError;
use anyhow::{Result, anyhow};


pub(crate) fn parse_nl_duration(input: &str) -> Result<DateTime<Utc>> {
    let duration = Duration::from_std(
        parse_duration(input)?
    )?;

    Ok(Utc::now() + duration)
}

pub(crate) fn parse_key_value(input: &str) -> Result<(String, String)> {
    let parts = input
        .splitn(2, "=")
        .collect::<Vec<&str>>();

    if parts.len() != 2 {
        return Err(anyhow!("Invalid key-value pair: {}", input));
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}

pub(crate) fn validate_vec_string_len<const LEN: usize>(items: &Vec<String>) -> Result<(), ValidationError> {
    for item in items {
        if item.len() > LEN {
            let mut err = ValidationError::new("string_length_exceeded");
            err.add_param("item".into(), item.into());
            err.add_param("max_length".into(), &LEN);
            return Err(err);
        }
    }

    Ok(())
}

pub(crate) fn validate_vec_nullable_string_len<const LEN: usize>(items: &Vec<NullableString>) -> Result<(), ValidationError> {
    validate_vec_string_len::<LEN>(
        &items
            .iter()
            .filter_map(|ns| ns.as_inner().as_ref().cloned())
            .collect()
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NullableString(pub Option<String>);

impl NullableString {
    pub fn new(value: Option<String>) -> Self {
        NullableString(value)
    }

    pub fn into_inner(self) -> Option<String> {
        self.0
    }

    pub fn as_inner(&self) -> &Option<String> {
        &self.0
    }
}

impl FromStr for NullableString {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("null") {
            Ok(NullableString(None))
        } else {
            Ok(NullableString(Some(s.to_string())))
        }
    }
}

impl From<NullableString> for Option<String> {
    fn from(ns: NullableString) -> Self {
        ns.0
    }
}

impl From<Option<String>> for NullableString {
    fn from(opt: Option<String>) -> Self {
        NullableString(opt)
    }
}

impl From<String> for NullableString {
    fn from(s: String) -> Self {
        NullableString(Some(s))
    }
}

impl From<NullableString> for String {
    fn from(ns: NullableString) -> Self {
        ns.0.unwrap_or_default()
    }
}
