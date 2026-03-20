use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub full_name: Option<String>,
    pub status: String,
    pub role: String,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
}

/// Serialize a serde-serializable enum to a String, falling back to an empty string.
fn enum_to_string<T: serde::Serialize>(val: &T) -> String {
    serde_json::to_value(val)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default()
}

impl From<keygen_rs::user::User> for User {
    fn from(u: keygen_rs::user::User) -> Self {
        User {
            id: u.id,
            email: u.email,
            first_name: u.first_name,
            last_name: u.last_name,
            full_name: u.full_name,
            status: enum_to_string(&u.status),
            role: enum_to_string(&u.role),
            permissions: u.permissions,
            metadata: u
                .metadata
                .map(|m| serde_json::to_value(m).unwrap_or_default()),
            created: u.created,
            updated: u.updated,
            account_id: None,
        }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct CreateUserRequest {
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: String,
    pub password: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct ListUsersOptions {
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub page_number: Option<u32>,
    pub role: Option<String>,
    pub status: Option<String>,
}

fn parse_enum<T: serde::de::DeserializeOwned>(s: &str, label: &str) -> Result<T> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .map_err(|e| napi::Error::new(Status::InvalidArg, format!("Invalid {label}: {e}")))
}

fn to_metadata(
    v: serde_json::Value,
) -> Result<std::collections::HashMap<String, serde_json::Value>> {
    serde_json::from_value(v)
        .map_err(|e| napi::Error::new(Status::InvalidArg, format!("Invalid metadata: {e}")))
}

#[napi]
pub async fn create_user(request: CreateUserRequest) -> Result<User> {
    let role: keygen_rs::user::UserRole = parse_enum(&request.role, "user role")?;

    let req = keygen_rs::user::CreateUserRequest {
        email: request.email,
        first_name: request.first_name,
        last_name: request.last_name,
        role: Some(role),
        permissions: request.permissions,
        metadata: request.metadata.map(to_metadata).transpose()?,
    };

    keygen_rs::user::create(req)
        .await
        .map(User::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn list_users(options: Option<ListUsersOptions>) -> Result<Vec<User>> {
    let opts = options
        .map(|o| -> Result<keygen_rs::user::ListUsersOptions> {
            Ok(keygen_rs::user::ListUsersOptions {
                limit: o.limit,
                page_size: o.page_size,
                page_number: o.page_number,
                status: o
                    .status
                    .as_deref()
                    .map(|s| parse_enum(s, "user status"))
                    .transpose()?,
                roles: o
                    .role
                    .as_deref()
                    .map(|s| {
                        parse_enum::<keygen_rs::user::UserRole>(s, "user role").map(|r| vec![r])
                    })
                    .transpose()?,
                ..Default::default()
            })
        })
        .transpose()?;

    keygen_rs::user::list(opts)
        .await
        .map(|result| result.users.into_iter().map(User::from).collect())
        .map_err(to_napi_error)
}

#[napi]
pub async fn get_user(id: String) -> Result<User> {
    keygen_rs::user::get(&id)
        .await
        .map(User::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn update_user(id: String, request: UpdateUserRequest) -> Result<User> {
    let req = keygen_rs::user::UpdateUserRequest {
        email: request.email,
        first_name: request.first_name,
        last_name: request.last_name,
        role: request
            .role
            .as_deref()
            .map(|s| parse_enum(s, "user role"))
            .transpose()?,
        metadata: request.metadata.map(to_metadata).transpose()?,
    };

    keygen_rs::user::update(&id, req)
        .await
        .map(User::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn delete_user(id: String) -> Result<()> {
    keygen_rs::user::delete(&id).await.map_err(to_napi_error)
}

#[napi]
pub async fn ban_user(id: String) -> Result<User> {
    keygen_rs::user::ban(&id)
        .await
        .map(User::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn unban_user(id: String) -> Result<User> {
    keygen_rs::user::unban(&id)
        .await
        .map(User::from)
        .map_err(to_napi_error)
}
