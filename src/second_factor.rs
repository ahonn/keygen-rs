//! Second factors (TOTP two-factor authentication) of users.
//!
//! See <https://keygen.sh/docs/api/users/#second-factors>.
//!
//! An admin token can call every endpoint, but Keygen still checks a secret that only
//! the user holds: creating a second factor needs the user's password (or an OTP when
//! the user already has an enabled second factor), enabling or disabling one needs an
//! OTP from it, and deleting an enabled one needs an OTP. An admin cannot remove a
//! user's enabled second factor without that OTP.

use crate::client::Client;
use crate::errors::Error;
use crate::keygen_client::KeygenClient;
use crate::pagination::{list_query, ListDocument, Page, PaginationOptions};
use crate::KeygenResponseData;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SecondFactorAttributes {
    #[serde(default)]
    pub uri: Option<String>,
    #[serde(default)]
    pub secret: Option<String>,
    pub enabled: bool,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
struct SecondFactorResponse {
    data: KeygenResponseData<SecondFactorAttributes>,
}

/// A TOTP second factor of a user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecondFactor {
    pub id: String,
    /// The `otpauth://` provisioning URI. Only present while the factor is not enabled.
    pub uri: Option<String>,
    /// The TOTP secret. Only present while the factor is not enabled.
    pub secret: Option<String>,
    pub enabled: bool,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub account_id: Option<String>,
    pub environment_id: Option<String>,
    pub user_id: Option<String>,
}

impl SecondFactor {
    pub(crate) fn from(data: KeygenResponseData<SecondFactorAttributes>) -> Self {
        // keygen-api serializes the user as `user`; the API reference shows `owner`.
        let user_id = data
            .relationships
            .other_data("user")
            .map(|d| d.id)
            .or_else(|| data.relationships.owner_id());
        Self {
            account_id: data.relationships.account_id(),
            environment_id: data.relationships.environment_id(),
            user_id,
            id: data.id,
            uri: data.attributes.uri,
            secret: data.attributes.secret,
            enabled: data.attributes.enabled,
            created: data.attributes.created,
            updated: data.attributes.updated,
        }
    }

    fn from_body(body: Value) -> Result<Self, Error> {
        let response: SecondFactorResponse = serde_json::from_value(body)?;
        Ok(Self::from(response.data))
    }

    pub(crate) async fn list_with_client(
        client: &Client,
        user_id: &str,
        options: Option<&PaginationOptions>,
    ) -> Result<Page<SecondFactor>, Error> {
        let query = list_query(options)?;
        let response = client.get(&path(user_id, None), query.as_ref()).await?;
        let document: ListDocument<SecondFactorAttributes> = serde_json::from_value(response.body)?;
        Ok(document.into_page(SecondFactor::from))
    }

    pub(crate) async fn get_with_client(
        client: &Client,
        user_id: &str,
        id: &str,
    ) -> Result<SecondFactor, Error> {
        let response = client.get(&path(user_id, Some(id)), None::<&()>).await?;
        Self::from_body(response.body)
    }

    pub(crate) async fn create_with_client(
        client: &Client,
        user_id: &str,
        credential: &SecondFactorCredential,
    ) -> Result<SecondFactor, Error> {
        let meta = match credential {
            SecondFactorCredential::Password(password) => json!({ "password": password }),
            SecondFactorCredential::Otp(otp) => json!({ "otp": otp }),
        };
        let body = json!({ "data": { "type": "second-factors" }, "meta": meta });
        let response = client
            .post(&path(user_id, None), Some(&body), None::<&()>)
            .await?;
        Self::from_body(response.body)
    }

    pub(crate) async fn update_with_client(
        client: &Client,
        user_id: &str,
        id: &str,
        enabled: bool,
        otp: &str,
    ) -> Result<SecondFactor, Error> {
        let body = json!({
            "data": { "type": "second-factors", "attributes": { "enabled": enabled } },
            "meta": { "otp": otp }
        });
        let response = client
            .patch(&path(user_id, Some(id)), Some(&body), None::<&()>)
            .await?;
        Self::from_body(response.body)
    }

    pub(crate) async fn delete_with_client(
        client: &Client,
        user_id: &str,
        id: &str,
        otp: Option<&str>,
    ) -> Result<(), Error> {
        let endpoint = path(user_id, Some(id));
        match otp {
            Some(otp) => {
                let body = json!({ "meta": { "otp": otp } });
                client.delete::<_, ()>(&endpoint, Some(&body)).await?;
            }
            None => {
                client.delete::<(), ()>(&endpoint, None::<&()>).await?;
            }
        }
        Ok(())
    }
}

/// The secret that proves the user's consent to add a second factor.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SecondFactorCredential {
    /// The user's password. Required when the user has no enabled second factor.
    Password(String),
    /// An OTP from the user's enabled second factor. Required when the user has one.
    Otp(String),
}

fn path(user_id: &str, id: Option<&str>) -> String {
    match id {
        Some(id) => format!("users/{user_id}/second-factors/{id}"),
        None => format!("users/{user_id}/second-factors"),
    }
}

/// Second factor operations bound to one [`KeygenClient`].
///
/// `user_id` is the ID or email of the user.
pub struct SecondFactorService<'a> {
    client: &'a KeygenClient,
}

impl<'a> SecondFactorService<'a> {
    pub(crate) fn new(client: &'a KeygenClient) -> Self {
        Self { client }
    }

    /// Lists the second factors of a user, most recent first.
    pub async fn list(
        &self,
        user_id: &str,
        options: Option<&PaginationOptions>,
    ) -> Result<Page<SecondFactor>, Error> {
        SecondFactor::list_with_client(&self.client.transport, user_id, options).await
    }

    pub async fn get(&self, user_id: &str, id: &str) -> Result<SecondFactor, Error> {
        SecondFactor::get_with_client(&self.client.transport, user_id, id).await
    }

    /// Adds a second factor to a user. The new factor is disabled; show its
    /// [`uri`](SecondFactor::uri) or [`secret`](SecondFactor::secret) to the user, then
    /// enable it with [`SecondFactorService::update`] and an OTP from it.
    pub async fn create(
        &self,
        user_id: &str,
        credential: &SecondFactorCredential,
    ) -> Result<SecondFactor, Error> {
        SecondFactor::create_with_client(&self.client.transport, user_id, credential).await
    }

    /// Enables or disables a second factor. `otp` must come from this second factor.
    pub async fn update(
        &self,
        user_id: &str,
        id: &str,
        enabled: bool,
        otp: &str,
    ) -> Result<SecondFactor, Error> {
        SecondFactor::update_with_client(&self.client.transport, user_id, id, enabled, otp).await
    }

    /// Deletes a second factor. `otp` is required when the user's second factor is
    /// enabled.
    pub async fn delete(&self, user_id: &str, id: &str, otp: Option<&str>) -> Result<(), Error> {
        SecondFactor::delete_with_client(&self.client.transport, user_id, id, otp).await
    }
}
