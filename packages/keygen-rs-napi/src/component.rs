use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
pub struct Component {
    pub id: String,
    pub fingerprint: String,
    pub name: String,
    pub metadata: Option<serde_json::Value>,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
    pub environment_id: Option<String>,
    pub product_id: Option<String>,
    pub license_id: Option<String>,
    pub machine_id: Option<String>,
}

impl From<keygen_rs::component::Component> for Component {
    fn from(c: keygen_rs::component::Component) -> Self {
        Component {
            id: c.id,
            fingerprint: c.fingerprint,
            name: c.name,
            metadata: c
                .metadata
                .map(|m| serde_json::to_value(m).unwrap_or_default()),
            created: c.created.to_rfc3339(),
            updated: c.updated.to_rfc3339(),
            account_id: c.account_id,
            environment_id: c.environment_id,
            product_id: c.product_id,
            license_id: c.license_id,
            machine_id: c.machine_id,
        }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct CreateComponentRequest {
    pub fingerprint: String,
    pub name: String,
    pub metadata: Option<serde_json::Value>,
    pub machine_id: String,
}

#[napi(object)]
#[derive(Clone)]
pub struct UpdateComponentRequest {
    pub name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct ListComponentsOptions {
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub page_number: Option<u32>,
    pub machine: Option<String>,
    pub license: Option<String>,
    pub owner: Option<String>,
    pub user: Option<String>,
    pub product: Option<String>,
}

#[napi]
pub async fn create_component(request: CreateComponentRequest) -> Result<Component> {
    let req = keygen_rs::component::CreateComponentRequest {
        fingerprint: request.fingerprint,
        name: request.name,
        metadata: crate::opt_metadata(request.metadata)?,
        machine_id: request.machine_id,
    };
    keygen_rs::component::Component::create(req)
        .await
        .map(Component::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn list_components(options: Option<ListComponentsOptions>) -> Result<Vec<Component>> {
    let opts = options.map(|o| keygen_rs::component::ListComponentsOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
        machine: o.machine,
        license: o.license,
        owner: o.owner,
        user: o.user,
        product: o.product,
    });
    keygen_rs::component::Component::list(opts)
        .await
        .map(|list| list.into_iter().map(Component::from).collect())
        .map_err(to_napi_error)
}

#[napi]
pub async fn get_component(id: String) -> Result<Component> {
    keygen_rs::component::Component::get(&id)
        .await
        .map(Component::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn update_component(id: String, request: UpdateComponentRequest) -> Result<Component> {
    let comp = keygen_rs::component::Component {
        id,
        ..Default::default()
    };
    let req = keygen_rs::component::UpdateComponentRequest {
        name: request.name,
        metadata: crate::opt_metadata(request.metadata)?,
    };
    comp.update(req)
        .await
        .map(Component::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn delete_component(id: String) -> Result<()> {
    let comp = keygen_rs::component::Component {
        id,
        ..Default::default()
    };
    comp.delete().await.map_err(to_napi_error)
}
