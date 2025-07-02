use crate::client::Client;
use crate::errors::Error;
use crate::KeygenResponseData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserRole {
    Admin,
    Environment,
    Product,
    User,
    License,
    Read,
    Write,
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
    pub role: UserRole,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
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
    pub role: UserRole,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
    pub role: UserRole,
    pub password: String,
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
            role: data.attributes.role,
            metadata: data.attributes.metadata,
            created: data.attributes.created,
            updated: data.attributes.updated,
        }
    }
}

/// Create a new user
pub async fn create(request: CreateUserRequest) -> Result<User, Error> {
    let client = Client::default();
    let body = serde_json::json!({
        "data": {
            "type": "users",
            "attributes": request
        }
    });
    let response = client.post("users", Some(&body), None::<&()>).await?;
    let user_response: UserResponse = serde_json::from_value(response.body)?;
    Ok(User::from(user_response.data))
}

/// List all users
pub async fn list() -> Result<Vec<User>, Error> {
    let client = Client::default();
    let response = client.get("users", None::<&()>).await?;
    let users_response: UsersResponse = serde_json::from_value(response.body)?;
    Ok(users_response.data.into_iter().map(User::from).collect())
}

/// Get a specific user by ID
pub async fn get(user_id: &str) -> Result<User, Error> {
    let client = Client::default();
    let endpoint = format!("users/{}", user_id);
    let response = client.get(&endpoint, None::<&()>).await?;
    let user_response: UserResponse = serde_json::from_value(response.body)?;
    Ok(User::from(user_response.data))
}

/// Update a user
pub async fn update(user_id: &str, request: UpdateUserRequest) -> Result<User, Error> {
    let client = Client::default();
    let endpoint = format!("users/{}", user_id);
    let body = serde_json::json!({
        "data": {
            "type": "users",
            "id": user_id,
            "attributes": request
        }
    });
    let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
    let user_response: UserResponse = serde_json::from_value(response.body)?;
    Ok(User::from(user_response.data))
}

/// Delete a user
pub async fn delete(user_id: &str) -> Result<(), Error> {
    let client = Client::default();
    let endpoint = format!("users/{}", user_id);
    client.delete::<(), ()>(&endpoint, None::<&()>).await?;
    Ok(())
}

/// Ban a user
pub async fn ban(user_id: &str) -> Result<User, Error> {
    let client = Client::default();
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
    let client = Client::default();
    let endpoint = format!("users/{}/actions/unban", user_id);
    let body = serde_json::json!({
        "meta": {}
    });
    let response = client.post(&endpoint, Some(&body), None::<&()>).await?;
    let user_response: UserResponse = serde_json::from_value(response.body)?;
    Ok(User::from(user_response.data))
}

/// Change a user's password
pub async fn change_password(user_id: &str, current_password: &str, new_password: &str) -> Result<(), Error> {
    let client = Client::default();
    let endpoint = format!("users/{}/actions/change-password", user_id);
    let body = serde_json::json!({
        "meta": {
            "currentPassword": current_password,
            "newPassword": new_password
        }
    });
    client.post::<_, (), _>(&endpoint, Some(&body), None::<&()>).await?;
    Ok(())
}

/// Reset a user's password
pub async fn reset_password(user_id: &str, new_password: &str) -> Result<(), Error> {
    let client = Client::default();
    let endpoint = format!("users/{}/actions/reset-password", user_id);
    let body = serde_json::json!({
        "meta": {
            "password": new_password
        }
    });
    client.post::<_, (), _>(&endpoint, Some(&body), None::<&()>).await?;
    Ok(())
}

/// Generate a user token
pub async fn generate_token(user_id: &str, name: Option<&str>, expiry: Option<&str>) -> Result<serde_json::Value, Error> {
    let client = Client::default();
    let endpoint = format!("users/{}/tokens", user_id);
    let mut meta = serde_json::Map::new();
    if let Some(name) = name {
        meta.insert("name".to_string(), serde_json::Value::String(name.to_string()));
    }
    if let Some(expiry) = expiry {
        meta.insert("expiry".to_string(), serde_json::Value::String(expiry.to_string()));
    }
    let body = serde_json::json!({
        "data": {
            "type": "tokens",
            "attributes": {
                "name": name.unwrap_or("User Token")
            }
        }
    });
    let response = client.post(&endpoint, Some(&body), None::<&()>).await?;
    Ok(response.body)
}