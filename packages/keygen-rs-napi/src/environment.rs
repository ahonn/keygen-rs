use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
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

#[napi(object)]
#[derive(Clone)]
pub struct EnvironmentToken {
    pub id: String,
    pub token: String,
    pub name: Option<String>,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
}

impl From<keygen_rs::environment::EnvironmentToken> for EnvironmentToken {
    fn from(t: keygen_rs::environment::EnvironmentToken) -> Self {
        EnvironmentToken {
            id: t.id,
            token: t.token,
            name: t.name,
            created: t.created,
            updated: t.updated,
            account_id: t.account_id,
        }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct CreateEnvironmentRequest {
    pub name: String,
    pub code: String,
    pub isolation_strategy: Option<String>,
}

#[napi(object)]
#[derive(Clone)]
pub struct UpdateEnvironmentRequest {
    pub name: Option<String>,
    pub code: Option<String>,
}

#[napi(object)]
#[derive(Clone)]
pub struct ListEnvironmentsOptions {
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub page_number: Option<u32>,
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

use crate::parse_enum;

#[napi]
pub async fn create_environment(request: CreateEnvironmentRequest) -> Result<Environment> {
    let req = keygen_rs::environment::CreateEnvironmentRequest {
        name: request.name,
        code: request.code,
        isolation_strategy: request
            .isolation_strategy
            .as_deref()
            .map(|s| parse_enum(s, "isolation strategy"))
            .transpose()?,
    };

    keygen_rs::environment::Environment::create(req)
        .await
        .map(Environment::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn list_environments(
    options: Option<ListEnvironmentsOptions>,
) -> Result<Vec<Environment>> {
    let opts = options.map(|o| keygen_rs::environment::ListEnvironmentsOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
    });

    keygen_rs::environment::Environment::list(opts)
        .await
        .map(|result| {
            result
                .environments
                .into_iter()
                .map(Environment::from)
                .collect()
        })
        .map_err(to_napi_error)
}

#[napi]
pub async fn get_environment(id: String) -> Result<Environment> {
    keygen_rs::environment::Environment::get(&id)
        .await
        .map(Environment::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn update_environment(
    id: String,
    request: UpdateEnvironmentRequest,
) -> Result<Environment> {
    let env = make_environment(id);

    let req = keygen_rs::environment::UpdateEnvironmentRequest {
        name: request.name,
        code: request.code,
    };

    env.update(req)
        .await
        .map(Environment::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn delete_environment(id: String) -> Result<()> {
    let env = make_environment(id);
    env.delete().await.map_err(to_napi_error)
}

#[napi]
pub async fn generate_environment_token(
    id: String,
    name: Option<String>,
    expiry: Option<String>,
    permissions: Option<Vec<String>>,
) -> Result<EnvironmentToken> {
    let env = make_environment(id);

    let req = keygen_rs::environment::CreateEnvironmentTokenRequest {
        name,
        expiry,
        permissions,
    };

    env.generate_token(Some(req))
        .await
        .map(EnvironmentToken::from)
        .map_err(to_napi_error)
}
