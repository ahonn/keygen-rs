use crate::client::Client;
use crate::errors::Error;
use crate::KeygenResponseData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TokenKind {
    #[serde(rename = "activation-token")]
    ActivationToken,
    #[serde(rename = "product-token")]
    ProductToken,
    #[serde(rename = "user-token")]
    UserToken,
    #[serde(rename = "support-token")]
    SupportToken,
    #[serde(rename = "sales-token")]
    SalesToken,
    #[serde(rename = "developer-token")]
    DeveloperToken,
    #[serde(rename = "admin-token")]
    AdminToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAttributes {
    pub kind: TokenKind,
    pub token: Option<String>, // Only present on creation/regeneration
    pub name: Option<String>,
    pub expiry: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TokenResponse {
    pub data: KeygenResponseData<TokenAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TokensResponse {
    pub data: Vec<KeygenResponseData<TokenAttributes>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListTokensOptions {
    pub limit: Option<u32>,
    #[serde(rename = "page[size]")]
    pub page_size: Option<u32>,
    #[serde(rename = "page[number]")]
    pub page_number: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegenerateTokenRequest {
    pub name: Option<String>,
    pub expiry: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub id: String,
    pub kind: TokenKind,
    pub token: Option<String>,
    pub name: Option<String>,
    pub expiry: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: String,
    pub updated: String,
}

impl Token {
    pub(crate) fn from(data: KeygenResponseData<TokenAttributes>) -> Token {
        Token {
            id: data.id,
            kind: data.attributes.kind,
            token: data.attributes.token,
            name: data.attributes.name,
            expiry: data.attributes.expiry,
            permissions: data.attributes.permissions,
            metadata: data.attributes.metadata,
            created: data.attributes.created,
            updated: data.attributes.updated,
        }
    }

    /// List tokens with optional pagination and filtering
    pub async fn list(options: Option<ListTokensOptions>) -> Result<Vec<Token>, Error> {
        let client = Client::default();
        let response = client.get("tokens", options.as_ref()).await?;
        let tokens_response: TokensResponse = serde_json::from_value(response.body)?;
        Ok(tokens_response.data.into_iter().map(Token::from).collect())
    }

    /// Get a token by ID
    pub async fn get(id: &str) -> Result<Token, Error> {
        let client = Client::default();
        let endpoint = format!("tokens/{}", id);
        let response = client.get(&endpoint, None::<&()>).await?;
        let token_response: TokenResponse = serde_json::from_value(response.body)?;
        Ok(Token::from(token_response.data))
    }

    /// Regenerate a token
    pub async fn regenerate(&self, request: RegenerateTokenRequest) -> Result<Token, Error> {
        let client = Client::default();
        let endpoint = format!("tokens/{}", self.id);

        let mut attributes = serde_json::Map::new();
        if let Some(name) = request.name {
            attributes.insert("name".to_string(), serde_json::Value::String(name));
        }
        if let Some(expiry) = request.expiry {
            attributes.insert("expiry".to_string(), serde_json::Value::String(expiry));
        }
        if let Some(permissions) = request.permissions {
            attributes.insert(
                "permissions".to_string(),
                serde_json::to_value(permissions)?,
            );
        }
        if let Some(metadata) = request.metadata {
            attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }

        let body = serde_json::json!({
            "data": {
                "type": "tokens",
                "attributes": attributes
            }
        });

        let response = client.put(&endpoint, Some(&body), None::<&()>).await?;
        let token_response: TokenResponse = serde_json::from_value(response.body)?;
        Ok(Token::from(token_response.data))
    }

    /// Revoke a token
    pub async fn revoke(&self) -> Result<(), Error> {
        let client = Client::default();
        let endpoint = format!("tokens/{}", self.id);
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expiry) = &self.expiry {
            if let Ok(expiry_time) = chrono::DateTime::parse_from_rfc3339(expiry) {
                return chrono::Utc::now() > expiry_time;
            }
        }
        false
    }

    /// Check if token has specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        if let Some(permissions) = &self.permissions {
            permissions.contains(&permission.to_string())
        } else {
            false
        }
    }

    /// Get token value (only available after generation/regeneration)
    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }
}
