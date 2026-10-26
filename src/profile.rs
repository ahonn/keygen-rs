//! The bearer and token used to authenticate a [`crate::KeygenClient`].

use crate::client::Client;
use crate::errors::Error;
use serde::{Deserialize, Serialize};

/// The stable identity and token projection returned by Keygen's `/me` endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentProfile {
    bearer: ProfileBearer,
    token: CurrentToken,
}

impl CurrentProfile {
    pub fn bearer(&self) -> &ProfileBearer {
        &self.bearer
    }

    pub fn token(&self) -> &CurrentToken {
        &self.token
    }
}

/// The JSON:API identity of the resource that owns the current token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileBearer {
    id: String,
    resource_type: String,
}

impl ProfileBearer {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn resource_type(&self) -> &str {
        &self.resource_type
    }
}

/// The current token fields needed for expiry and permission preflight checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentToken {
    id: String,
    kind: String,
    expiry: Option<String>,
    permissions: Option<Vec<String>>,
}

impl CurrentToken {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn kind(&self) -> &str {
        &self.kind
    }

    pub fn expiry(&self) -> Option<&str> {
        self.expiry.as_deref()
    }

    /// Returns `None` when the server does not expose token permissions, as
    /// Keygen CE omits them. An empty set is returned as `Some(&[])`.
    pub fn permissions(&self) -> Option<&[String]> {
        self.permissions.as_deref()
    }

    /// Describes how this token constrains a permission, not whether a request
    /// will ultimately be authorized for a specific resource.
    pub fn permission_constraint(&self, permission: &str) -> TokenPermissionConstraint {
        let Some(permissions) = &self.permissions else {
            return TokenPermissionConstraint::Inherited;
        };

        if permissions.iter().any(|candidate| candidate == permission) {
            TokenPermissionConstraint::Explicit
        } else if permissions.iter().any(|candidate| candidate == "*") {
            TokenPermissionConstraint::Inherited
        } else {
            TokenPermissionConstraint::Excluded
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[must_use = "the token permission constraint should be handled"]
pub enum TokenPermissionConstraint {
    /// The token names the requested permission directly.
    Explicit,
    /// The token uses `*`, or the server does not expose token permissions,
    /// so the bearer's permission decision applies.
    Inherited,
    /// The token has an explicit set that omits the requested permission.
    Excluded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfileDocument {
    data: ResourceIdentifier,
    #[serde(default)]
    included: Vec<IncludedResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ResourceIdentifier {
    id: String,
    #[serde(rename = "type")]
    resource_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IncludedResource {
    id: String,
    #[serde(rename = "type")]
    resource_type: String,
    attributes: serde_json::Value,
    #[serde(default)]
    relationships: IncludedRelationships,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct IncludedRelationships {
    #[serde(default)]
    bearer: Option<Relationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Relationship {
    #[serde(default)]
    data: Option<ResourceIdentifier>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurrentTokenAttributes {
    kind: String,
    expiry: Option<String>,
    permissions: Option<Vec<String>>,
}

pub(crate) async fn get_current_profile_with_client(
    client: &Client,
) -> Result<CurrentProfile, Error> {
    let response = client.get::<(), ProfileDocument>("me", None::<&()>).await?;
    CurrentProfile::try_from(response.body)
}

impl TryFrom<ProfileDocument> for CurrentProfile {
    type Error = Error;

    fn try_from(document: ProfileDocument) -> Result<Self, Self::Error> {
        let bearer = document.data;
        let mut matching_tokens = document.included.into_iter().filter(|resource| {
            resource.resource_type == "tokens"
                && resource
                    .relationships
                    .bearer
                    .as_ref()
                    .and_then(|relationship| relationship.data.as_ref())
                    == Some(&bearer)
        });

        let token_resource = matching_tokens.next().ok_or_else(|| {
            Error::InvalidResponse(
                "the /me response did not include a token for the current bearer".to_string(),
            )
        })?;

        if matching_tokens.next().is_some() {
            return Err(Error::InvalidResponse(
                "the /me response included multiple tokens for the current bearer".to_string(),
            ));
        }

        let attributes: CurrentTokenAttributes = serde_json::from_value(token_resource.attributes)?;

        Ok(Self {
            bearer: ProfileBearer {
                id: bearer.id,
                resource_type: bearer.resource_type,
            },
            token: CurrentToken {
                id: token_resource.id,
                kind: attributes.kind,
                expiry: attributes.expiry,
                permissions: attributes.permissions,
            },
        })
    }
}
