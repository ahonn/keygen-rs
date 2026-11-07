//! Search through Keygen's undocumented search endpoint.
//!
//! **Unstable.** `POST /v1/accounts/<account>/search` is used by the Keygen dashboard
//! but is not in Keygen's API reference. Keygen can change or remove it without notice,
//! so this module needs the `unstable-search` feature and is exempt from this crate's
//! semver guarantees. The behavior below follows keygen-api's `SearchesController`.
//!
//! A search runs on one resource type and returns one [`Page`] of that type. Each
//! [`SearchQuery`] term names a searchable attribute and a value. Keygen matches string
//! values partially and needs at least 3 characters per string value, including each
//! metadata value. [`SearchOp::And`] (the default) returns resources that match every
//! term; [`SearchOp::Or`] returns resources that match any term.
//!
//! Searchable attributes in keygen-api at the time of writing:
//!
//! | Method | Attributes |
//! |---|---|
//! | [`SearchService::licenses`] | `id`, `key`, `name`, `metadata`, `owner`, `user`, `product`, `policy` |
//! | [`SearchService::machines`] | `id`, `fingerprint`, `name`, `metadata`, `license`, `owner`, `user`, `product`, `policy` |
//! | [`SearchService::users`] | `id`, `email`, `firstName`, `lastName`, `fullName`, `name`, `metadata`, `role` |
//!
//! An unknown attribute or a short value makes Keygen answer `400 Bad Request`, which
//! arrives as [`Error::Api`]. Searching needs an admin, developer, read-only, sales
//! agent or support agent token.

use crate::client::Client;
use crate::errors::Error;
use crate::keygen_client::KeygenClient;
use crate::license::{License, LicenseAttributes};
use crate::machine::{Machine, MachineAttributes};
use crate::pagination::{list_query, ListDocument, Page, PaginationOptions};
use crate::user::{User, UserAttributes};
use crate::KeygenResponseData;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

/// How the terms of a [`SearchQuery`] combine.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SearchOp {
    /// A resource must match every term.
    #[default]
    And,
    /// A resource must match at least one term.
    Or,
}

/// The terms of a search.
///
/// ```ignore
/// let query = SearchQuery::new()
///     .term("email", "alice@")
///     .metadata("plan", "pro")
///     .op(SearchOp::Or);
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SearchQuery {
    terms: Map<String, Value>,
    metadata: Map<String, Value>,
    op: SearchOp,
}

impl SearchQuery {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a term for an attribute, for example `("email", "alice@")`. Use the
    /// camelCase attribute names of the API, such as `firstName`.
    pub fn term(mut self, attribute: impl Into<String>, value: impl Into<Value>) -> Self {
        self.terms.insert(attribute.into(), value.into());
        self
    }

    /// Adds a term for a metadata key, for example `("plan", "pro")`.
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Sets how the terms combine. The default is [`SearchOp::And`].
    pub fn op(mut self, op: SearchOp) -> Self {
        self.op = op;
        self
    }

    /// Whether the query has no terms. Keygen rejects an empty query.
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty() && self.metadata.is_empty()
    }

    /// The JSON body of a search for `resource_type`.
    pub(crate) fn body(&self, resource_type: &str) -> Value {
        let mut query = self.terms.clone();
        if !self.metadata.is_empty() {
            query.insert("metadata".to_string(), Value::Object(self.metadata.clone()));
        }
        json!({ "meta": { "type": resource_type, "query": query, "op": self.op } })
    }
}

async fn search_with_client<A, T>(
    client: &Client,
    resource_type: &str,
    query: &SearchQuery,
    options: Option<&PaginationOptions>,
    convert: impl FnMut(KeygenResponseData<A>) -> T,
) -> Result<Page<T>, Error>
where
    A: DeserializeOwned,
{
    // Keygen reads pagination from the query string, not from the body.
    let pagination = list_query(options)?;
    let response = client
        .post(
            "search",
            Some(&query.body(resource_type)),
            pagination.as_ref(),
        )
        .await?;
    let document: ListDocument<A> = serde_json::from_value(response.body)?;
    Ok(document.into_page(convert))
}

/// Search operations bound to one [`KeygenClient`]. See the [module docs](self).
pub struct SearchService<'a> {
    client: &'a KeygenClient,
}

impl<'a> SearchService<'a> {
    pub(crate) fn new(client: &'a KeygenClient) -> Self {
        Self { client }
    }

    /// Searches licenses. Each license is bound to this client.
    pub async fn licenses(
        &self,
        query: &SearchQuery,
        options: Option<&PaginationOptions>,
    ) -> Result<Page<License>, Error> {
        let page = search_with_client::<LicenseAttributes, _>(
            &self.client.transport,
            "licenses",
            query,
            options,
            License::from,
        )
        .await?;
        Ok(page.map(|license| {
            license.with_client(self.client.transport_arc(), self.client.config_arc())
        }))
    }

    /// Searches machines. Each machine carries this client's configuration.
    pub async fn machines(
        &self,
        query: &SearchQuery,
        options: Option<&PaginationOptions>,
    ) -> Result<Page<Machine>, Error> {
        let page = search_with_client::<MachineAttributes, _>(
            &self.client.transport,
            "machines",
            query,
            options,
            Machine::from,
        )
        .await?;
        Ok(page.map(|machine| machine.with_config_arc(self.client.config_arc())))
    }

    /// Searches users.
    pub async fn users(
        &self,
        query: &SearchQuery,
        options: Option<&PaginationOptions>,
    ) -> Result<Page<User>, Error> {
        search_with_client::<UserAttributes, _>(
            &self.client.transport,
            "users",
            query,
            options,
            User::from,
        )
        .await
    }
}
