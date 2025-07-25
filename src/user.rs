use crate::client::Client;
use crate::errors::Error;
use crate::KeygenResponseData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
pub(crate) struct UsersResponse {
    pub data: Vec<KeygenResponseData<UserAttributes>>,
    pub meta: Option<serde_json::Value>,
    pub links: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersListResult {
    pub users: Vec<User>,
    pub meta: Option<serde_json::Value>,
    pub links: Option<serde_json::Value>,
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
}

/// Create a new user
pub async fn create(request: CreateUserRequest) -> Result<User, Error> {
    let client = Client::default()?;

    let mut attributes = serde_json::Map::new();
    attributes.insert(
        "email".to_string(),
        serde_json::Value::String(request.email),
    );

    if let Some(first_name) = request.first_name {
        attributes.insert(
            "firstName".to_string(),
            serde_json::Value::String(first_name),
        );
    }
    if let Some(last_name) = request.last_name {
        attributes.insert("lastName".to_string(), serde_json::Value::String(last_name));
    }
    if let Some(role) = request.role {
        attributes.insert("role".to_string(), serde_json::to_value(role)?);
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

    let mut data = serde_json::Map::new();
    data.insert(
        "type".to_string(),
        serde_json::Value::String("users".to_string()),
    );
    data.insert(
        "attributes".to_string(),
        serde_json::Value::Object(attributes),
    );

    let body = serde_json::json!({
        "data": data
    });

    let response = client.post("users", Some(&body), None::<&()>).await?;
    let user_response: UserResponse = serde_json::from_value(response.body)?;
    Ok(User::from(user_response.data))
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListUsersOptions {
    pub limit: Option<u32>,
    #[serde(rename = "page[size]")]
    pub page_size: Option<u32>,
    #[serde(rename = "page[number]")]
    pub page_number: Option<u32>,
    pub status: Option<UserStatus>,
    pub assigned: Option<bool>,
    pub product: Option<String>,
    pub group: Option<String>,
    pub roles: Option<Vec<UserRole>>,
    pub sort: Option<String>,
    pub include: Option<String>,
    #[serde(flatten)]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// List users with optional filtering and pagination, returning pagination metadata
pub async fn list(options: Option<ListUsersOptions>) -> Result<UsersListResult, Error> {
    let client = Client::default()?;
    let response = client.get("users", options.as_ref()).await?;
    let users_response: UsersResponse = serde_json::from_value(response.body)?;
    Ok(UsersListResult {
        users: users_response.data.into_iter().map(User::from).collect(),
        meta: users_response.meta,
        links: users_response.links,
    })
}

/// Get a specific user by ID
pub async fn get(user_id: &str) -> Result<User, Error> {
    let client = Client::default()?;
    let endpoint = format!("users/{}", user_id);
    let response = client.get(&endpoint, None::<&()>).await?;
    let user_response: UserResponse = serde_json::from_value(response.body)?;
    Ok(User::from(user_response.data))
}

/// Update a user
pub async fn update(user_id: &str, request: UpdateUserRequest) -> Result<User, Error> {
    let client = Client::default()?;
    let endpoint = format!("users/{}", user_id);
    let mut attributes = serde_json::Map::new();

    if let Some(email) = request.email {
        attributes.insert("email".to_string(), serde_json::Value::String(email));
    }
    if let Some(first_name) = request.first_name {
        attributes.insert(
            "firstName".to_string(),
            serde_json::Value::String(first_name),
        );
    }
    if let Some(last_name) = request.last_name {
        attributes.insert("lastName".to_string(), serde_json::Value::String(last_name));
    }
    if let Some(role) = request.role {
        attributes.insert("role".to_string(), serde_json::to_value(role)?);
    }
    if let Some(metadata) = request.metadata {
        attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
    }

    let body = serde_json::json!({
        "data": {
            "type": "users",
            "id": user_id,
            "attributes": attributes
        }
    });
    let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
    let user_response: UserResponse = serde_json::from_value(response.body)?;
    Ok(User::from(user_response.data))
}

/// Delete a user
pub async fn delete(user_id: &str) -> Result<(), Error> {
    let client = Client::default()?;
    let endpoint = format!("users/{}", user_id);
    client.delete::<(), ()>(&endpoint, None::<&()>).await?;
    Ok(())
}

/// Ban a user
pub async fn ban(user_id: &str) -> Result<User, Error> {
    let client = Client::default()?;
    let endpoint = format!("users/{}/actions/ban", user_id);
    let body = serde_json::json!({
        "meta": {}
    });
    let response = client.post(&endpoint, Some(&body), None::<&()>).await?;
    let user_response: UserResponse = serde_json::from_value(response.body)?;
    Ok(User::from(user_response.data))
}

/// Unban a user
pub async fn unban(user_id: &str) -> Result<User, Error> {
    let client = Client::default()?;
    let endpoint = format!("users/{}/actions/unban", user_id);
    let body = serde_json::json!({
        "meta": {}
    });
    let response = client.post(&endpoint, Some(&body), None::<&()>).await?;
    let user_response: UserResponse = serde_json::from_value(response.body)?;
    Ok(User::from(user_response.data))
}



