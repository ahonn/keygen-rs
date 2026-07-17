use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::error::Error as StdError;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ApiContractVersion {
    major: u8,
    minor: u8,
}

impl ApiContractVersion {
    pub const V1_7: Self = Self { major: 1, minor: 7 };
    pub const V1_8: Self = Self { major: 1, minor: 8 };
    pub const CURRENT: Self = Self::V1_8;
    pub const MINIMUM_SUPPORTED: Self = Self::V1_7;

    pub const fn as_str(self) -> &'static str {
        match (self.major, self.minor) {
            (1, 7) => "1.7",
            (1, 8) => "1.8",
            _ => unreachable!(),
        }
    }
}

impl Default for ApiContractVersion {
    fn default() -> Self {
        Self::CURRENT
    }
}

impl fmt::Display for ApiContractVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ApiContractVersion {
    type Err = ApiVersionParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "1.7" => Ok(Self::V1_7),
            "1.8" => Ok(Self::V1_8),
            _ => Err(ApiVersionParseError::new(value)),
        }
    }
}

impl TryFrom<&str> for ApiContractVersion {
    type Error = ApiVersionParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<String> for ApiContractVersion {
    type Error = ApiVersionParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Serialize for ApiContractVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ApiContractVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiVersionParseError {
    value: String,
}

impl ApiVersionParseError {
    fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for ApiVersionParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unsupported Keygen API version '{}'; supported versions are 1.7 and 1.8",
            self.value
        )
    }
}

impl StdError for ApiVersionParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_versions() {
        assert_eq!("1.7".parse(), Ok(ApiContractVersion::V1_7));
        assert_eq!("1.8".parse(), Ok(ApiContractVersion::V1_8));
    }

    #[test]
    fn rejects_legacy_future_and_prefixed_versions() {
        for value in ["1.6", "1.9", "2.0", "v1.8", "1.8.0"] {
            assert!(value.parse::<ApiContractVersion>().is_err());
        }
    }

    #[test]
    fn serde_uses_the_header_representation() {
        let encoded = serde_json::to_string(&ApiContractVersion::V1_8).unwrap();
        let decoded: ApiContractVersion = serde_json::from_str(&encoded).unwrap();

        assert_eq!(encoded, "\"1.8\"");
        assert_eq!(decoded, ApiContractVersion::V1_8);
    }
}
