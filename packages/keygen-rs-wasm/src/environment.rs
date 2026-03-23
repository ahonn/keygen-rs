use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::to_js_error;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub code: String,
    pub isolation_strategy: String,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
}

impl From<keygen_rs::environment::Environment> for Environment {
    fn from(e: keygen_rs::environment::Environment) -> Self {
        Environment {
            id: e.id,
            name: e.name,
            code: e.code,
            isolation_strategy: serde_json::to_value(&e.isolation_strategy)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_default(),
            created: e.created,
            updated: e.updated,
            account_id: e.account_id,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentToken {
    pub id: String,
    pub token: String,
    pub name: Option<String>,
    pub expiry: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub created: String,
    pub updated: String,
    pub environment_id: String,
    pub account_id: Option<String>,
}

impl From<keygen_rs::environment::EnvironmentToken> for EnvironmentToken {
    fn from(t: keygen_rs::environment::EnvironmentToken) -> Self {
        EnvironmentToken {
            id: t.id,
            token: t.token,
            name: t.name,
            expiry: t.expiry,
            permissions: t.permissions,
            created: t.created,
            updated: t.updated,
            environment_id: t.environment_id,
            account_id: t.account_id,
        }
    }
}

fn parse_isolation_strategy(s: &str) -> Result<keygen_rs::environment::IsolationStrategy, JsError> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .map_err(|e| JsError::new(&format!("Invalid isolation strategy: {e}")))
}

fn make_environment(id: String) -> keygen_rs::environment::Environment {
    keygen_rs::environment::Environment {
        id,
        name: String::new(),
        code: String::new(),
        isolation_strategy: keygen_rs::environment::IsolationStrategy::default(),
        created: String::new(),
        updated: String::new(),
        account_id: None,
    }
}

#[wasm_bindgen(js_name = "createEnvironment")]
pub async fn create_environment(request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        name: String,
        code: String,
        isolation_strategy: Option<String>,
    }
    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;

    let r = keygen_rs::environment::CreateEnvironmentRequest {
        name: req.name,
        code: req.code,
        isolation_strategy: req
            .isolation_strategy
            .as_deref()
            .map(parse_isolation_strategy)
            .transpose()?,
    };

    let env = keygen_rs::environment::Environment::create(r)
        .await
        .map(Environment::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&env).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "listEnvironments")]
pub async fn list_environments(options: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct Opts {
        limit: Option<u32>,
        page_size: Option<u32>,
        page_number: Option<u32>,
    }
    let opts: Option<Opts> = if options.is_undefined() || options.is_null() {
        None
    } else {
        Some(serde_wasm_bindgen::from_value(options).map_err(|e| JsError::new(&e.to_string()))?)
    };

    let list_opts = opts.map(|o| keygen_rs::environment::ListEnvironmentsOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
    });

    let environments: Vec<Environment> = keygen_rs::environment::Environment::list(list_opts)
        .await
        .map(|result| {
            result
                .environments
                .into_iter()
                .map(Environment::from)
                .collect()
        })
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&environments).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "getEnvironment")]
pub async fn get_environment(id: String) -> Result<JsValue, JsError> {
    let env = keygen_rs::environment::Environment::get(&id)
        .await
        .map(Environment::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&env).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "updateEnvironment")]
pub async fn update_environment(id: String, request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        name: Option<String>,
        code: Option<String>,
    }
    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;

    let env = make_environment(id);

    let r = keygen_rs::environment::UpdateEnvironmentRequest {
        name: req.name,
        code: req.code,
    };

    let environment = env
        .update(r)
        .await
        .map(Environment::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&environment).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "deleteEnvironment")]
pub async fn delete_environment(id: String) -> Result<(), JsError> {
    let env = make_environment(id);
    env.delete().await.map_err(to_js_error)
}

#[wasm_bindgen(js_name = "generateEnvironmentToken")]
pub async fn generate_environment_token(id: String, opts: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Opts {
        name: Option<String>,
        expiry: Option<String>,
        permissions: Option<Vec<String>>,
    }
    let opts: Option<Opts> = if opts.is_undefined() || opts.is_null() {
        None
    } else {
        Some(serde_wasm_bindgen::from_value(opts).map_err(|e| JsError::new(&e.to_string()))?)
    };

    let env = make_environment(id);

    let req = opts.map(|o| keygen_rs::environment::CreateEnvironmentTokenRequest {
        name: o.name,
        expiry: o.expiry,
        permissions: o.permissions,
    });

    let token = env
        .generate_token(req)
        .await
        .map(EnvironmentToken::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&token).map_err(|e| JsError::new(&e.to_string()))
}
