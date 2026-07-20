// shared types and utility functions
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tempfile::{Builder, NamedTempFile};
use tokio::fs;
use tracing::error;
use uuid::Uuid;

// *** Error types ***
/// Errors in types and their uses
#[derive(thiserror::Error)]
#[non_exhaustive]
pub enum TypeError {
    /// Error converting data from one type to another type
    #[error("Type Conversion Error: {0}")]
    ConversionError(String),
    /// Error in reading or writing data
    #[error("Input / Output Error: {0}")]
    InputOutputError(String),
    /// Error in getting requested resources
    #[error("Resource Allocation Error: {0}")]
    ResourceAllocationError(String),
    /// Error because the provided value is not valid
    #[error("Validation Error: {0}")]
    ValidationError(String),
}

impl std::fmt::Debug for TypeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        let error_string = match self {
            Self::ConversionError(message) => {
                format!("Conversion Error: {message}")
            }
            Self::InputOutputError(message) => {
                format!("Input / Output Error: {message}")
            }
            Self::ResourceAllocationError(message) => {
                format!("Resource Allocation Error: {message}")
            }
            Self::ValidationError(message) => {
                format!("Validation Error: {message}")
            }
        };
        write!(formatter, "{error_string}")
    }
}

impl From<std::io::Error> for TypeError {
    fn from(error: std::io::Error) -> Self {
        Self::InputOutputError(format!("IO Error: {error}"))
    }
}

impl From<url::ParseError> for TypeError {
    fn from(error: url::ParseError) -> Self {
        Self::ValidationError(format!("Error parsing URL: {error}"))
    }
}

impl From<reqwest::Error> for TypeError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_builder() {
            Self::ValidationError(format!("{error}"))
        } else if error.is_body() || error.is_decode() {
            return Self::ConversionError(format!("{error}"));
        } else {
            return Self::InputOutputError(format!("{error}"));
        }
    }
}


/// convert TypeError types into server responses
impl axum::response::IntoResponse for TypeError {
    fn into_response(self) -> axum::response::Response {
        let body = format!("{self}");
        let status_code = match self {
            Self::ValidationError(_) => axum::http::StatusCode::BAD_REQUEST,
            _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status_code, body).into_response()
    }
}

#[test]
fn test_vec_to_string() {
    let empty = Vec::new();
    let empty_printable = vec_to_string::<&str>(&empty, "\n");
    assert_eq!(empty_printable, "NONE");

    let strings = vec![
        "Roses are red",
        "Violet is blue",
        "Some poems rhyme",
        "Others don't",
    ];
    let printable = vec_to_string::<&str>(&strings, "\n");
    //println!("{}", printable);
    assert_eq!(
        printable,
        "Roses are red\nViolet is blue\nSome poems rhyme\nOthers don't"
    )
}

/// Convert a vec to a line-per-item displayable version
pub fn vec_to_string<D>(items: &[D], delimiter: &str) -> String
where
    D: std::fmt::Display,
{
    if items.is_empty() {
        return String::from("NONE");
    }
    Vec::from_iter(items.iter().map(|i| i.to_string()))
        .join(delimiter)
        .to_string()
}

// *** ID type ***
#[test]
fn test_request_id() {
    for _i in 0..100 {
        let v1 = RequestId::new();
        let v2 = RequestId::new();
        assert_ne!(v1, v2);
        //println!("{} != {}", v1, v2);
    }
}

/// RequestId wraps a UUIDv4 in hypenated String form
#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RequestId(String);

impl RequestId {
    pub const MAX_LENGTH: usize = uuid::fmt::Hyphenated::LENGTH;

    pub fn new() -> Self {
        let id = Uuid::new_v4();
        let mut buf = [0_u8; Self::MAX_LENGTH];
        Self(String::from(id.hyphenated().encode_lower(&mut buf)))
    }

    pub fn from_string(string: String) -> Result<Self, TypeError> {
        if string.len() > Self::MAX_LENGTH {
            let error_message = format!(
                "provided value {} exceeds max length {}",
                string,
                Self::MAX_LENGTH
            );
            error!("{}", error_message);
            return Err(TypeError::ValidationError(error_message));
        }
        Ok(Self(string))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl std::fmt::Debug for RequestId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "RequestId: {}", self.0)
    }
}


/// timestamp to show when API messages were created
/// Use UTC representation internally, but convert to
/// local time when printing for user friendliness
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    pub const FORMAT: &'static str = "%FT%T%.f%:z";
    pub const DISPLAY_FORMAT: &'static str = "%FT%T%.3f%:z";

    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn parse_from_str(string: &str) -> Result<Self, TypeError> {
        match string.parse::<DateTime<Utc>>() {
            Ok(datetime) => Ok(Self(datetime)),
            Err(error) => Err(TypeError::ValidationError(format!(
                "Error parsing ISO 8601 timestamp from provided string {string}: {error}"
            ))),
        }
    }

    pub fn from_microseconds(microseconds: i64) -> Result<Self, TypeError> {
        match DateTime::<Utc>::from_timestamp_micros(microseconds) {
            Some(datetime) => Ok(Self(datetime)),
            None => Err(TypeError::ValidationError(format!(
                "Invalid ISO 8601 timestamp microseconds value {microseconds}; must be a valid signed 64-bit number"
            ))),
        }
    }

    pub fn value(&self) -> String {
        format!("{}", self.0.format(Self::FORMAT))
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        let local: DateTime<Local> = DateTime::from(self.0);
        write!(formatter, "{}", local.format(Timestamp::DISPLAY_FORMAT))
    }
}


/// Shared Utility Functions
// copy the 'from' file to the 'to' file path if it is not already present
// NOTE:  the 'to' file path must include the wanted filename
pub async fn copy_if_not_found(from: impl AsRef<Path>, to: impl AsRef<Path>) -> std::io::Result<()> {
    match fs::try_exists(&to).await {
        Ok(true) => {
            // already present, nothing to do
            Ok(())
        }
        Ok(false) => {
            // not found, let's copy it
            match fs::copy(&from, &to).await {
                Ok(_bytes_copied) => Ok(()),
                Err(error) => Err(error),
            }
        }
        Err(error) => Err(error),
    }
}

// generate a String of the given length with pseudorandom content
pub fn generate_random_string(len: usize) -> String {
    use rand::distr::Alphanumeric;
    use rand::{RngExt, rng};

    rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

/// get the cause of an error    
pub fn error_root_cause(mut err: &(dyn std::error::Error + 'static)) -> String {
    use std::fmt::Write;

    let mut s = format!("{err}");
    while let Some(src) = err.source() {
        let _ = write!(s, "\n\tCaused by: {src}");
        err = src;
    }
    s
}

#[tokio::test]
async fn test_write_string_to_named_tempfile() {
    use crate::shared::generate_random_string;

    let prefix = generate_random_string(12);
    let suffix = generate_random_string(3);
    let contents = generate_random_string(1023);
    let named_tempfile = write_string_to_named_tempfile(&prefix, &suffix, &contents)
        .await
        .unwrap();
    println!("Wrote {contents} to named_tempfile {named_tempfile:?}");
    let read_contents = fs::read_to_string(named_tempfile.path()).await.unwrap();
    assert_eq!(contents, read_contents);
}

// write a given Rust string to a named temporary file
// used for KLV rulesets that will be used by ffmpeg for KLV stream editing
pub async fn write_string_to_named_tempfile(
    prefix: &str,
    suffix: &str,
    contents: &str,
) -> std::io::Result<NamedTempFile> {
    match Builder::new().prefix(prefix).suffix(suffix).tempfile() {
        Ok(named_tempfile) => match fs::write(&named_tempfile, contents.as_bytes()).await {
            Ok(()) => Ok(named_tempfile),
            Err(error) => {
                let error_message = format!(
                    "Error writing string to named tempfile at path {:?}: {error}",
                    named_tempfile.path()
                );
                error!("{error_message}");
                Err(std::io::Error::new(error.kind(), error_message))
            }
        },
        Err(error) => {
            let error_message = format!(
                "Error creating named tempfile for prefix {prefix}, suffix {suffix}: {error}"
            );
            error!("{}", error_message);
            Err(std::io::Error::new(error.kind(), error_message))
        }
    }
}

/// sleep for a randomized amount up to 'max_delay_in_seconds' before continuing
pub async fn start_after_random_delay(max_delay_in_seconds: u8) {
    use rand::distr::Uniform;
    use rand::rng;
    use rand::RngExt;
    let range = Uniform::try_from(1..max_delay_in_seconds).unwrap();
    let delay = rng().sample(range);
    tokio::time::sleep(std::time::Duration::from_secs(delay.into())).await;
}

#[test]
fn test_hostname() {
    let name = hostname();
    println!("hostname is {name}");
    assert!(!name.is_empty());
}

pub fn hostname() -> String {
    match std::env::var("HOSTNAME") {
        Ok(hostname) => hostname,
        Err(error) => match error {
            std::env::VarError::NotPresent => match std::env::var("COMPUTERNAME") {
                Ok(hostname) => hostname,
                Err(_error) => "localhost".to_string(),
            },
            _ => "localhost".to_string(),
        },
    }
}
