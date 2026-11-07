//! Rate limit status reported by the Keygen API.
//!
//! Keygen sends the `X-RateLimit-*` headers on API responses and adds `Retry-After`
//! when it rejects a request with `429 Too Many Requests`. See
//! <https://keygen.sh/docs/api/rate-limiting/>.
//!
//! The SDK never retries a rate-limited request. Read the status from
//! [`Error::rate_limit`](crate::errors::Error::rate_limit) after a 429, or from
//! [`KeygenClient::last_rate_limit`](crate::KeygenClient::last_rate_limit) after any
//! response, and decide when to send the next request.

use chrono::{DateTime, Utc};
use reqwest::header::HeaderMap;
use std::time::Duration;

/// The rate limit status of the window that is closest to its limit.
///
/// A field is `None` when the response did not carry its header.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct RateLimit {
    /// `X-RateLimit-Window`: the name of the window, for example `30s`.
    pub window: Option<String>,
    /// `X-RateLimit-Count`: requests made in the window.
    pub count: Option<u64>,
    /// `X-RateLimit-Limit`: requests allowed in the window.
    pub limit: Option<u64>,
    /// `X-RateLimit-Remaining`: requests left in the window.
    pub remaining: Option<u64>,
    /// `X-RateLimit-Reset`: when the window resets, in UTC epoch seconds.
    pub reset: Option<u64>,
    /// `Retry-After`: seconds to wait before the next request. Only sent with a 429.
    pub retry_after: Option<u64>,
}

impl RateLimit {
    /// Reads the rate limit headers. Returns `None` when the response has none of them.
    pub(crate) fn from_headers(headers: &HeaderMap) -> Option<Self> {
        let text = |name: &str| {
            headers
                .get(name)
                .and_then(|value| value.to_str().ok())
                .map(str::trim)
                .filter(|value| !value.is_empty())
        };
        let number = |name: &str| text(name).and_then(|value| value.parse::<u64>().ok());

        let rate_limit = Self {
            window: text("X-RateLimit-Window").map(str::to_owned),
            count: number("X-RateLimit-Count"),
            limit: number("X-RateLimit-Limit"),
            remaining: number("X-RateLimit-Remaining"),
            reset: number("X-RateLimit-Reset"),
            retry_after: number("Retry-After"),
        };
        (rate_limit != Self::default()).then_some(rate_limit)
    }

    /// `Retry-After` as a duration.
    pub fn retry_after_duration(&self) -> Option<Duration> {
        self.retry_after.map(Duration::from_secs)
    }

    /// `X-RateLimit-Reset` as a timestamp.
    pub fn reset_at(&self) -> Option<DateTime<Utc>> {
        self.reset
            .and_then(|reset| i64::try_from(reset).ok())
            .and_then(|reset| DateTime::from_timestamp(reset, 0))
    }

    /// Whether the window has no requests left.
    pub fn is_exhausted(&self) -> bool {
        self.remaining == Some(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderValue;

    #[test]
    fn reads_every_header() {
        let mut headers = HeaderMap::new();
        for (name, value) in [
            ("X-RateLimit-Window", "30s"),
            ("X-RateLimit-Count", "60"),
            ("X-RateLimit-Limit", "60"),
            ("X-RateLimit-Remaining", "0"),
            ("X-RateLimit-Reset", "1700000030"),
            ("Retry-After", "12"),
        ] {
            headers.insert(name, HeaderValue::from_static(value));
        }

        let rate_limit = RateLimit::from_headers(&headers).unwrap();

        assert_eq!(rate_limit.window.as_deref(), Some("30s"));
        assert_eq!(rate_limit.count, Some(60));
        assert_eq!(rate_limit.limit, Some(60));
        assert_eq!(rate_limit.remaining, Some(0));
        assert!(rate_limit.is_exhausted());
        assert_eq!(
            rate_limit.retry_after_duration(),
            Some(Duration::from_secs(12))
        );
        assert_eq!(rate_limit.reset_at().unwrap().timestamp(), 1_700_000_030);
    }

    #[test]
    fn is_none_without_headers() {
        assert_eq!(RateLimit::from_headers(&HeaderMap::new()), None);
    }
}
