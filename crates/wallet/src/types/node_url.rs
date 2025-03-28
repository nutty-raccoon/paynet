// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Url

use core::fmt;
use core::str::FromStr;

use rusqlite::{
    Result,
    types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::ParseError;

/// Url Error
#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    /// Url error
    #[error(transparent)]
    Url(#[from] ParseError),
    /// Invalid URL structure
    #[error("Invalid URL")]
    InvalidUrl,
}

/// MintUrl Url
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeUrl(String);

impl NodeUrl {
    // This should only be used on values that have been previously parsed,
    // most likely read from the database.
    pub(crate) fn new_unchecked(url: String) -> Self {
        Self(url)
    }
}

fn format_url(url: &str) -> Result<String, Error> {
    if url.is_empty() {
        return Err(Error::InvalidUrl);
    }
    let url = url.trim_end_matches('/');
    // https://URL.com/path/TO/resource -> https://url.com/path/TO/resource
    let mut split_url = url.split("://");
    let protocol = split_url.next().ok_or(Error::InvalidUrl)?.to_lowercase();
    let mut split_address = split_url.next().ok_or(Error::InvalidUrl)?.split('/');
    let host = split_address
        .next()
        .ok_or(Error::InvalidUrl)?
        .to_lowercase();
    let path = split_address.collect::<Vec<&str>>().join("/");
    let mut formatted_url = format!("{}://{}", protocol, host);
    if !path.is_empty() {
        formatted_url.push_str(&format!("/{}", path));
    }
    Ok(formatted_url)
}

impl FromStr for NodeUrl {
    type Err = Error;

    fn from_str(url: &str) -> Result<Self, Self::Err> {
        let formatted_url = format_url(url);
        match formatted_url {
            Ok(url) => Ok(Self(url)),
            Err(_) => Err(Error::InvalidUrl),
        }
    }
}

impl<'de> Deserialize<'de> for NodeUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for NodeUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for NodeUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl AsRef<str> for NodeUrl {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<&NodeUrl> for tonic::transport::Endpoint {
    fn from(value: &NodeUrl) -> Self {
        value.0.parse().unwrap()
    }
}

impl ToSql for NodeUrl {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok(self.as_ref().into())
    }
}

impl FromSql for NodeUrl {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        String::column_result(value).map(NodeUrl::new_unchecked)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_trim_trailing_slashes() {
        let very_unformatted_url = "http://url-to-check.com////";
        let unformatted_url = "http://url-to-check.com/";
        let formatted_url = "http://url-to-check.com";

        let very_trimmed_url = NodeUrl::from_str(very_unformatted_url).unwrap();
        assert_eq!(formatted_url, very_trimmed_url.to_string());

        let trimmed_url = NodeUrl::from_str(unformatted_url).unwrap();
        assert_eq!(formatted_url, trimmed_url.to_string());

        let unchanged_url = NodeUrl::from_str(formatted_url).unwrap();
        assert_eq!(formatted_url, unchanged_url.to_string());
    }
    #[test]
    fn test_case_insensitive() {
        let wrong_cased_url = "http://URL-to-check.com";
        let correct_cased_url = "http://url-to-check.com";

        let cased_url_formatted = NodeUrl::from_str(wrong_cased_url).unwrap();
        assert_eq!(correct_cased_url, cased_url_formatted.to_string());

        let wrong_cased_url_with_path = "http://URL-to-check.com/PATH/to/check";
        let correct_cased_url_with_path = "http://url-to-check.com/PATH/to/check";

        let cased_url_with_path_formatted = NodeUrl::from_str(wrong_cased_url_with_path).unwrap();
        assert_eq!(
            correct_cased_url_with_path,
            cased_url_with_path_formatted.to_string()
        );
    }
}
