use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub key: String,
    pub engine: String,
    pub metadata: Option<serde_json::Value>,
    pub created: String,
    pub updated: String,
    pub product_id: Option<String>,
    pub account_id: Option<String>,
    pub environment_id: Option<String>,
}

impl From<keygen_rs::package::Package> for Package {
    fn from(p: keygen_rs::package::Package) -> Self {
        let engine = p
            .engine
            .as_ref()
            .and_then(|e| serde_json::to_value(e).ok())
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        Package {
            id: p.id,
            name: p.name,
            key: p.key,
            engine,
            metadata: p
                .metadata
                .map(|m| serde_json::to_value(m).unwrap_or_default()),
            created: p.created,
            updated: p.updated,
            product_id: p.product_id,
            account_id: p.account_id,
            environment_id: p.environment_id,
        }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct CreatePackageRequest {
    pub product_id: String,
    pub name: String,
    pub key: String,
    pub engine: String,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct UpdatePackageRequest {
    pub name: Option<String>,
    pub key: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct ListPackagesOptions {
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub page_number: Option<u32>,
    pub product: Option<String>,
    pub engine: Option<String>,
}

fn to_hashmap(
    val: Option<serde_json::Value>,
) -> Option<std::collections::HashMap<String, serde_json::Value>> {
    val.and_then(|v| serde_json::from_value(v).ok())
}

fn parse_engine(s: &str) -> std::result::Result<keygen_rs::package::PackageEngine, napi::Error> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .map_err(|e| napi::Error::new(Status::InvalidArg, format!("Invalid package engine: {e}")))
}

fn make_minimal_package(id: String) -> keygen_rs::package::Package {
    keygen_rs::package::Package {
        id,
        name: String::new(),
        key: String::new(),
        engine: None,
        metadata: None,
        created: String::new(),
        updated: String::new(),
        product_id: None,
        account_id: None,
        environment_id: None,
    }
}

#[napi]
pub async fn create_package(request: CreatePackageRequest) -> Result<Package> {
    let engine = parse_engine(&request.engine)?;
    let req = keygen_rs::package::CreatePackageRequest {
        name: request.name,
        key: request.key,
        product_id: request.product_id,
        engine: Some(engine),
        metadata: to_hashmap(request.metadata),
    };
    keygen_rs::package::Package::create(req)
        .await
        .map(Package::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn list_packages(options: Option<ListPackagesOptions>) -> Result<Vec<Package>> {
    let opts = match options {
        Some(o) => {
            let engine = match o.engine {
                Some(ref e) => Some(parse_engine(e)?),
                None => None,
            };
            Some(keygen_rs::package::ListPackagesOptions {
                limit: o.limit,
                page_size: o.page_size,
                page_number: o.page_number,
                product: o.product,
                engine,
            })
        }
        None => None,
    };
    keygen_rs::package::Package::list(opts)
        .await
        .map(|list| list.into_iter().map(Package::from).collect())
        .map_err(to_napi_error)
}

#[napi]
pub async fn get_package(id: String) -> Result<Package> {
    keygen_rs::package::Package::get(&id)
        .await
        .map(Package::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn update_package(id: String, request: UpdatePackageRequest) -> Result<Package> {
    let pkg = make_minimal_package(id);
    let req = keygen_rs::package::UpdatePackageRequest {
        name: request.name,
        key: request.key,
        metadata: to_hashmap(request.metadata),
    };
    pkg.update(req)
        .await
        .map(Package::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn delete_package(id: String) -> Result<()> {
    let pkg = make_minimal_package(id);
    pkg.delete().await.map_err(to_napi_error)
}
