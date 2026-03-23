use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::to_js_error;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

fn enum_to_string<T: serde::Serialize>(val: &T) -> Option<String> {
    serde_json::to_value(val)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
}

fn parse_enum<T: serde::de::DeserializeOwned>(s: &str, label: &str) -> Result<T, JsError> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .map_err(|e| JsError::new(&format!("Invalid {label}: {e}")))
}

impl From<keygen_rs::user::User> for User {
    fn from(u: keygen_rs::user::User) -> Self {
        User {
            id: u.id,
            email: u.email,
            first_name: u.first_name,
            last_name: u.last_name,
            full_name: u.full_name,
            status: enum_to_string(&u.status).unwrap_or_default(),
            role: enum_to_string(&u.role).unwrap_or_default(),
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

#[wasm_bindgen(js_name = "createUser")]
pub async fn create_user(request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        email: String,
        first_name: Option<String>,
        last_name: Option<String>,
        role: String,
        permissions: Option<Vec<String>>,
        metadata: Option<serde_json::Value>,
    }
    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;

    let role: keygen_rs::user::UserRole = parse_enum(&req.role, "user role")?;

    let r = keygen_rs::user::CreateUserRequest {
        email: req.email,
        first_name: req.first_name,
        last_name: req.last_name,
        role: Some(role),
        permissions: req.permissions,
        metadata: crate::opt_metadata(req.metadata)?,
    };

    let user = keygen_rs::user::create(r)
        .await
        .map(User::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&user).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "listUsers")]
pub async fn list_users(options: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct Opts {
        limit: Option<u32>,
        page_size: Option<u32>,
        page_number: Option<u32>,
        role: Option<String>,
        status: Option<String>,
    }
    let opts: Option<Opts> = if options.is_undefined() || options.is_null() {
        None
    } else {
        Some(serde_wasm_bindgen::from_value(options).map_err(|e| JsError::new(&e.to_string()))?)
    };

    let list_opts = opts
        .map(|o| -> Result<keygen_rs::user::ListUsersOptions, JsError> {
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

    let users: Vec<User> = keygen_rs::user::list(list_opts)
        .await
        .map(|result| result.users.into_iter().map(User::from).collect())
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&users).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "getUser")]
pub async fn get_user(id: String) -> Result<JsValue, JsError> {
    let user = keygen_rs::user::get(&id)
        .await
        .map(User::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&user).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "updateUser")]
pub async fn update_user(id: String, request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        email: Option<String>,
        first_name: Option<String>,
        last_name: Option<String>,
        role: Option<String>,
        metadata: Option<serde_json::Value>,
    }
    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;

    let r = keygen_rs::user::UpdateUserRequest {
        email: req.email,
        first_name: req.first_name,
        last_name: req.last_name,
        role: req
            .role
            .as_deref()
            .map(|s| parse_enum(s, "user role"))
            .transpose()?,
        metadata: crate::opt_metadata(req.metadata)?,
    };

    let user = keygen_rs::user::update(&id, r)
        .await
        .map(User::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&user).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "deleteUser")]
pub async fn delete_user(id: String) -> Result<(), JsError> {
    keygen_rs::user::delete(&id).await.map_err(to_js_error)
}

#[wasm_bindgen(js_name = "banUser")]
pub async fn ban_user(id: String) -> Result<JsValue, JsError> {
    let user = keygen_rs::user::ban(&id)
        .await
        .map(User::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&user).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "unbanUser")]
pub async fn unban_user(id: String) -> Result<JsValue, JsError> {
    let user = keygen_rs::user::unban(&id)
        .await
        .map(User::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&user).map_err(|e| JsError::new(&e.to_string()))
}
