use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::to_napi_error;
use crate::token_module::Token;

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
    pub last_seen_at: Option<String>,
    pub ban_reason: Option<String>,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
}

impl From<keygen_rs::user::User> for User {
    fn from(u: keygen_rs::user::User) -> Self {
        User {
            id: u.id,
            email: u.email,
            first_name: u.first_name,
            last_name: u.last_name,
            full_name: u.full_name,
            status: crate::enum_to_string(&u.status).unwrap_or_default(),
            role: crate::enum_to_string(&u.role).unwrap_or_default(),
            permissions: u.permissions,
            metadata: u
                .metadata
                .map(|m| serde_json::to_value(m).unwrap_or_default()),
            last_seen_at: u.last_seen_at,
            ban_reason: u.ban_reason,
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
    pub assigned: Option<bool>,
    pub product: Option<String>,
    pub group: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct CreateTokenRequest {
    pub name: Option<String>,
    pub expiry: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct UpdatePasswordRequest {
    pub current_password: Option<String>,
    pub password: String,
}

#[napi(object)]
#[derive(Clone)]
pub struct ResetPasswordRequest {
    pub email: Option<String>,
}

use crate::{parse_enum, to_metadata};

fn make_minimal_user(id: String) -> keygen_rs::user::User {
    keygen_rs::user::User {
        id,
        email: String::new(),
        first_name: None,
        last_name: None,
        full_name: None,
        status: keygen_rs::user::UserStatus::Active,
        role: keygen_rs::user::UserRole::User,
        permissions: None,
        metadata: None,
        last_seen_at: None,
        ban_reason: None,
        created: String::new(),
        updated: String::new(),
    }
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

    keygen_rs::user::User::create(req)
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
                assigned: o.assigned,
                product: o.product,
                group: o.group,
                roles: o
                    .role
                    .as_deref()
                    .map(|s| {
                        parse_enum::<keygen_rs::user::UserRole>(s, "user role").map(|r| vec![r])
                    })
                    .transpose()?,
                metadata: o.metadata.map(to_metadata).transpose()?,
                ..Default::default()
            })
        })
        .transpose()?;

    keygen_rs::user::User::list(opts)
        .await
        .map(|result| result.users.into_iter().map(User::from).collect())
        .map_err(to_napi_error)
}

#[napi]
pub async fn get_user(id: String) -> Result<User> {
    keygen_rs::user::User::get(&id)
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

    make_minimal_user(id)
        .update(req)
        .await
        .map(User::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn delete_user(id: String) -> Result<()> {
    make_minimal_user(id).delete().await.map_err(to_napi_error)
}

#[napi]
pub async fn ban_user(id: String) -> Result<User> {
    make_minimal_user(id)
        .ban()
        .await
        .map(User::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn unban_user(id: String) -> Result<User> {
    make_minimal_user(id)
        .unban()
        .await
        .map(User::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn generate_user_token(id: String, request: Option<CreateTokenRequest>) -> Result<Token> {
    let req = request
        .map(|request| -> Result<keygen_rs::token::CreateTokenRequest> {
            Ok(keygen_rs::token::CreateTokenRequest {
                name: request.name,
                expiry: request.expiry,
                permissions: request.permissions,
                metadata: request.metadata.map(to_metadata).transpose()?,
            })
        })
        .transpose()?;

    make_minimal_user(id)
        .generate_token(req)
        .await
        .map(Token::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn change_user_group(id: String, group_id: String) -> Result<User> {
    make_minimal_user(id)
        .change_group(&group_id)
        .await
        .map(User::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn update_user_password(id: String, request: UpdatePasswordRequest) -> Result<User> {
    let req = keygen_rs::user::UpdatePasswordRequest {
        current_password: request.current_password,
        password: request.password,
    };

    make_minimal_user(id)
        .update_password(req)
        .await
        .map(User::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn reset_user_password(
    id: String,
    request: Option<ResetPasswordRequest>,
) -> Result<User> {
    let req = request.map(|request| keygen_rs::user::ResetPasswordRequest {
        email: request.email,
    });

    make_minimal_user(id)
        .reset_password(req)
        .await
        .map(User::from)
        .map_err(to_napi_error)
}
