// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! Url

use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::{ParseError, Url};

use rusqlite::{
    Result as SqlResult,
    types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef},
};

/// Url Error
#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    /// Url error
    #[error(transparent)]
    Url(#[from] ParseError),
    /// Invalid URL structure
    #[error("invalid URL")]
    InvalidUrl,
    #[error("invalide transmision scheme. {0} is expected, got {1}")]
    InvalidScheme(&'static str, String),
}

/// MintUrl Url
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct NodeUrl(pub(crate) Url);

fn parse_node_url(url_string: &str) -> Result<Url, Error> {
    let url = Url::parse(url_string)?;
    #[cfg(feature = "tls")]
    if url.scheme() != "https" {
        return Err(Error::InvalidScheme("https", url.scheme().to_string()));
    }
    #[cfg(feature = "tls")]
    if url.domain().is_none() {
        return Err(Error::InvalidUrl);
    }

    #[cfg(not(feature = "tls"))]
    if url.scheme() != "http" {
        return Err(Error::InvalidScheme("http", url.scheme().to_string()));
    }

    Ok(url)
}

impl FromStr for NodeUrl {
    type Err = Error;

    fn from_str(url: &str) -> Result<Self, Self::Err> {
        let parsed_url = parse_node_url(url)?;
        Ok(Self(parsed_url))
    }
}

impl fmt::Display for NodeUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ToSql for NodeUrl {
    fn to_sql(&self) -> SqlResult<ToSqlOutput<'_>> {
        Ok(self.0.as_str().into())
    }
}

impl FromSql for NodeUrl {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s = String::column_result(value)?;

        NodeUrl::from_str(&s).map_err(|e| rusqlite::types::FromSqlError::Other(Box::new(e)))
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
