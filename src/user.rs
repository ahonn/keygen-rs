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
    pub role: Option<UserRole>,
    pub permissions: Option<Vec<String>>,
    pub password: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub group_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
    pub role: Option<UserRole>,
    pub permissions: Option<Vec<String>>,
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
            created: data.attributes.created,
            updated: data.attributes.updated,
        }
    }
}

/// Create a new user
pub async fn create(request: CreateUserRequest) -> Result<User, Error> {
    let client = Client::default();
    
    let mut attributes = serde_json::Map::new();
    attributes.insert("email".to_string(), serde_json::Value::String(request.email));
    
    if let Some(first_name) = request.first_name {
        attributes.insert("firstName".to_string(), serde_json::Value::String(first_name));
    }
    if let Some(last_name) = request.last_name {
        attributes.insert("lastName".to_string(), serde_json::Value::String(last_name));
    }
    if let Some(role) = request.role {
        attributes.insert("role".to_string(), serde_json::to_value(role)?);
    }
    if let Some(permissions) = request.permissions {
        attributes.insert("permissions".to_string(), serde_json::to_value(permissions)?);
    }
    if let Some(password) = request.password {
        attributes.insert("password".to_string(), serde_json::Value::String(password));
    }
    if let Some(metadata) = request.metadata {
        attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
    }
    
    let mut data = serde_json::Map::new();
    data.insert("type".to_string(), serde_json::Value::String("users".to_string()));
    data.insert("attributes".to_string(), serde_json::Value::Object(attributes));
    
    if let Some(group_id) = request.group_id {
        let mut relationships = serde_json::Map::new();
        let mut group = serde_json::Map::new();
        let mut group_data = serde_json::Map::new();
        group_data.insert("type".to_string(), serde_json::Value::String("groups".to_string()));
        group_data.insert("id".to_string(), serde_json::Value::String(group_id));
        group.insert("data".to_string(), serde_json::Value::Object(group_data));
        relationships.insert("group".to_string(), serde_json::Value::Object(group));
        data.insert("relationships".to_string(), serde_json::Value::Object(relationships));
    }
    
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
    #[serde(flatten)]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// List users with optional filtering and pagination
pub async fn list(options: Option<ListUsersOptions>) -> Result<Vec<User>, Error> {
    let client = Client::default();
    let response = client.get("users", options.as_ref()).await?;
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