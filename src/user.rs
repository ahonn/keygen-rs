use crate::client::Client;
use crate::errors::Error;
use crate::insert_optional;
use crate::keygen_client::KeygenClient;
use crate::pagination::{list_query, ListDocument, Page};
use crate::token::{token_request_attributes, CreateTokenRequest, Token, TokenResponse};
use crate::KeygenResponseData;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

fn serialize_metadata_filter<S>(
    metadata: &Option<HashMap<String, serde_json::Value>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeMap;
    match metadata {
        Some(map) => {
            let mut ser_map = serializer.serialize_map(Some(map.len()))?;
            for (key, value) in map {
                ser_map.serialize_entry(&format!("metadata[{key}]"), value)?;
            }
            ser_map.end()
        }
        None => serializer.serialize_none(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserStatus {
    Active,
    Inactive,
    Banned,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum UserRole {
    Admin,
    Developer,
    Environment,
    Product,
    License,
    #[serde(rename = "sales-agent")]
    SalesAgent,
    #[serde(rename = "support-agent")]
    SupportAgent,
    #[serde(rename = "read-only")]
    ReadOnly,
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAttributes {
    pub email: String,
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
    pub status: UserStatus,
    pub role: UserRole,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    #[serde(rename = "lastSeenAt")]
    pub last_seen_at: Option<String>,
    #[serde(rename = "banReason")]
    pub ban_reason: Option<String>,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: Option<String>,
    pub status: UserStatus,
    pub role: UserRole,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub last_seen_at: Option<String>,
    pub ban_reason: Option<String>,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct UserResponse {
    pub data: KeygenResponseData<UserAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersListResult {
    pub users: Vec<User>,
    pub meta: Option<serde_json::Value>,
    pub links: Option<serde_json::Value>,
}

impl From<Page<User>> for UsersListResult {
    fn from(page: Page<User>) -> Self {
        Self {
            users: page.data,
            meta: page.meta,
            links: page.links,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
    pub role: Option<UserRole>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
    pub role: Option<UserRole>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdatePasswordRequest {
    #[serde(rename = "oldPassword")]
    pub old_password: Option<String>,
    #[serde(rename = "newPassword")]
    pub new_password: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    #[serde(rename = "passwordResetToken")]
    pub password_reset_token: String,
    #[serde(rename = "newPassword")]
    pub new_password: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListUsersOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(rename = "page[size]", skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(rename = "page[number]", skip_serializing_if = "Option::is_none")]
    pub page_number: Option<u32>,
    /// Cursor returned by [`Page::next_cursor`](crate::pagination::Page::next_cursor).
    /// Ignored when `page_number` is set.
    #[serde(rename = "page[cursor]", skip_serializing_if = "Option::is_none")]
    pub page_cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<UserStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<UserRole>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<String>,
    #[serde(
        flatten,
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_metadata_filter"
    )]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl User {
    pub(crate) fn from(data: KeygenResponseData<UserAttributes>) -> User {
        User {
            id: data.id,
            email: data.attributes.email,
            first_name: data.attributes.first_name,
            last_name: data.attributes.last_name,
            full_name: data.attributes.full_name,
            status: data.attributes.status,
            role: data.attributes.role,
            permissions: data.attributes.permissions,
            metadata: data.attributes.metadata,
            last_seen_at: data.attributes.last_seen_at,
            ban_reason: data.attributes.ban_reason,
            created: data.attributes.created,
            updated: data.attributes.updated,
        }
    }

    /// Create a new user
    pub async fn create(request: CreateUserRequest) -> Result<User, Error> {
        Self::create_with_client(&Client::from_global_config()?, request).await
    }

    pub(crate) async fn create_with_client(
        client: &Client,
        request: CreateUserRequest,
    ) -> Result<User, Error> {
        let mut attributes = serde_json::Map::new();
        attributes.insert("email".to_string(), serde_json::json!(request.email));
        insert_optional(&mut attributes, "firstName", request.first_name)?;
        insert_optional(&mut attributes, "lastName", request.last_name)?;
        insert_optional(&mut attributes, "role", request.role)?;
        insert_optional(&mut attributes, "permissions", request.permissions)?;
        insert_optional(&mut attributes, "metadata", request.metadata)?;

        let body = serde_json::json!({
            "data": {
                "type": "users",
                "attributes": attributes
            }
        });

        let response = client.post("users", Some(&body), None::<&()>).await?;
        let user_response: UserResponse = serde_json::from_value(response.body)?;
        Ok(User::from(user_response.data))
    }

    /// List users with optional filtering and pagination, returning pagination metadata
    pub async fn list(options: Option<ListUsersOptions>) -> Result<UsersListResult, Error> {
        let page = Self::list_with_client(&Client::from_global_config()?, options.as_ref()).await?;
        Ok(page.into())
    }

    pub(crate) async fn list_with_client(
        client: &Client,
        options: Option<&ListUsersOptions>,
    ) -> Result<Page<User>, Error> {
        let query = list_query(options)?;
        let response = client.get("users", query.as_ref()).await?;
        let document: ListDocument<UserAttributes> = serde_json::from_value(response.body)?;
        Ok(document.into_page(User::from))
    }

    /// Get a specific user by ID
    pub async fn get(user_id: &str) -> Result<User, Error> {
        Self::get_with_client(&Client::from_global_config()?, user_id).await
    }

    pub(crate) async fn get_with_client(client: &Client, user_id: &str) -> Result<User, Error> {
        let endpoint = format!("users/{user_id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let user_response: UserResponse = serde_json::from_value(response.body)?;
        Ok(User::from(user_response.data))
    }

    /// Update this user
    pub async fn update(&self, request: UpdateUserRequest) -> Result<User, Error> {
        Self::update_with_client(&Client::from_global_config()?, &self.id, request).await
    }

    pub(crate) async fn update_with_client(
        client: &Client,
        id: &str,
        request: UpdateUserRequest,
    ) -> Result<User, Error> {
        let endpoint = format!("users/{id}");
        let mut attributes = serde_json::Map::new();
        insert_optional(&mut attributes, "email", request.email)?;
        insert_optional(&mut attributes, "firstName", request.first_name)?;
        insert_optional(&mut attributes, "lastName", request.last_name)?;
        insert_optional(&mut attributes, "role", request.role)?;
        insert_optional(&mut attributes, "metadata", request.metadata)?;

        let body = serde_json::json!({
            "data": {
                "type": "users",
                "id": id,
                "attributes": attributes
            }
        });
        let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
        let user_response: UserResponse = serde_json::from_value(response.body)?;
        Ok(User::from(user_response.data))
    }

    /// Delete this user
    pub async fn delete(&self) -> Result<(), Error> {
        Self::delete_with_client(&Client::from_global_config()?, &self.id).await
    }

    pub(crate) async fn delete_with_client(client: &Client, id: &str) -> Result<(), Error> {
        let endpoint = format!("users/{id}");
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }

    /// Ban this user
    pub async fn ban(&self) -> Result<User, Error> {
        Self::ban_with_client(&Client::from_global_config()?, &self.id).await
    }

    pub(crate) async fn ban_with_client(client: &Client, id: &str) -> Result<User, Error> {
        let endpoint = format!("users/{id}/actions/ban");
        let body = serde_json::json!({ "meta": {} });
        let response = client.post(&endpoint, Some(&body), None::<&()>).await?;
        let user_response: UserResponse = serde_json::from_value(response.body)?;
        Ok(User::from(user_response.data))
    }

    /// Unban this user
    pub async fn unban(&self) -> Result<User, Error> {
        Self::unban_with_client(&Client::from_global_config()?, &self.id).await
    }

    pub(crate) async fn unban_with_client(client: &Client, id: &str) -> Result<User, Error> {
        let endpoint = format!("users/{id}/actions/unban");
        let body = serde_json::json!({ "meta": {} });
        let response = client.post(&endpoint, Some(&body), None::<&()>).await?;
        let user_response: UserResponse = serde_json::from_value(response.body)?;
        Ok(User::from(user_response.data))
    }

    /// Generate a token for this user.
    pub async fn generate_token(
        &self,
        request: Option<CreateTokenRequest>,
    ) -> Result<Token, Error> {
        Self::generate_token_with_client(&Client::from_global_config()?, &self.id, request).await
    }

    pub(crate) async fn generate_token_with_client(
        client: &Client,
        id: &str,
        request: Option<CreateTokenRequest>,
    ) -> Result<Token, Error> {
        let endpoint = format!("users/{id}/tokens");
        let attributes = token_request_attributes(request.as_ref())?;
        let body = serde_json::json!({
            "data": {
                "type": "tokens",
                "attributes": attributes
            }
        });
        let response = client.post(&endpoint, Some(&body), None::<&()>).await?;
        let token_response: TokenResponse = serde_json::from_value(response.body)?;
        Ok(Token::from(token_response.data))
    }

    /// Change or assign this user's group.
    pub async fn change_group(&self, group_id: &str) -> Result<User, Error> {
        Self::change_group_with_client(&Client::from_global_config()?, &self.id, group_id).await
    }

    pub(crate) async fn change_group_with_client(
        client: &Client,
        id: &str,
        group_id: &str,
    ) -> Result<User, Error> {
        let endpoint = format!("users/{id}/group");
        let body = serde_json::json!({
            "data": {
                "type": "groups",
                "id": group_id
            }
        });
        let response = client.put(&endpoint, Some(&body), None::<&()>).await?;
        let user_response: UserResponse = serde_json::from_value(response.body)?;
        Ok(User::from(user_response.data))
    }

    /// Update this user's password.
    pub async fn update_password(&self, request: UpdatePasswordRequest) -> Result<User, Error> {
        Self::update_password_with_client(&Client::from_global_config()?, &self.id, request).await
    }

    pub(crate) async fn update_password_with_client(
        client: &Client,
        id: &str,
        request: UpdatePasswordRequest,
    ) -> Result<User, Error> {
        let endpoint = format!("users/{id}/actions/update-password");
        let mut meta = serde_json::Map::new();
        insert_optional(&mut meta, "oldPassword", request.old_password)?;
        meta.insert(
            "newPassword".to_string(),
            serde_json::Value::String(request.new_password),
        );
        let body = serde_json::json!({ "meta": meta });
        let response = client.post(&endpoint, Some(&body), None::<&()>).await?;
        let user_response: UserResponse = serde_json::from_value(response.body)?;
        Ok(User::from(user_response.data))
    }

    /// Reset this user's password using a reset token.
    pub async fn reset_password(&self, request: ResetPasswordRequest) -> Result<User, Error> {
        Self::reset_password_with_client(&Client::from_global_config()?, &self.id, request).await
    }

    pub(crate) async fn reset_password_with_client(
        client: &Client,
        id: &str,
        request: ResetPasswordRequest,
    ) -> Result<User, Error> {
        let endpoint = format!("users/{id}/actions/reset-password");
        let body = serde_json::json!({
            "meta": {
                "passwordResetToken": request.password_reset_token,
                "newPassword": request.new_password
            }
        });
        let response = client.post(&endpoint, Some(&body), None::<&()>).await?;
        let user_response: UserResponse = serde_json::from_value(response.body)?;
        Ok(User::from(user_response.data))
    }
}

/// User operations bound to one [`KeygenClient`].
pub struct UserService<'a> {
    client: &'a KeygenClient,
}

impl<'a> UserService<'a> {
    pub(crate) fn new(client: &'a KeygenClient) -> Self {
        Self { client }
    }

    pub async fn create(&self, request: CreateUserRequest) -> Result<User, Error> {
        User::create_with_client(&self.client.transport, request).await
    }

    pub async fn list(&self, options: Option<&ListUsersOptions>) -> Result<Page<User>, Error> {
        User::list_with_client(&self.client.transport, options).await
    }

    pub async fn get(&self, id: &str) -> Result<User, Error> {
        User::get_with_client(&self.client.transport, id).await
    }

    pub async fn update(&self, id: &str, request: UpdateUserRequest) -> Result<User, Error> {
        User::update_with_client(&self.client.transport, id, request).await
    }

    pub async fn delete(&self, id: &str) -> Result<(), Error> {
        User::delete_with_client(&self.client.transport, id).await
    }

    pub async fn ban(&self, id: &str) -> Result<User, Error> {
        User::ban_with_client(&self.client.transport, id).await
    }

    pub async fn unban(&self, id: &str) -> Result<User, Error> {
        User::unban_with_client(&self.client.transport, id).await
    }

    pub async fn generate_token(
        &self,
        id: &str,
        request: Option<CreateTokenRequest>,
    ) -> Result<Token, Error> {
        User::generate_token_with_client(&self.client.transport, id, request).await
    }

    pub async fn change_group(&self, id: &str, group_id: &str) -> Result<User, Error> {
        User::change_group_with_client(&self.client.transport, id, group_id).await
    }

    pub async fn update_password(
        &self,
        id: &str,
        request: UpdatePasswordRequest,
    ) -> Result<User, Error> {
        User::update_password_with_client(&self.client.transport, id, request).await
    }

    pub async fn reset_password(
        &self,
        id: &str,
        request: ResetPasswordRequest,
    ) -> Result<User, Error> {
        User::reset_password_with_client(&self.client.transport, id, request).await
    }
}
