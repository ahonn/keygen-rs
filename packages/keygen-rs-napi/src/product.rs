use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::to_napi_error;
use crate::token_module::Token;

#[napi(object)]
#[derive(Clone)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    pub distribution_strategy: Option<String>,
    pub url: Option<String>,
    pub platforms: Option<Vec<String>>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
}

impl From<keygen_rs::product::Product> for Product {
    fn from(p: keygen_rs::product::Product) -> Self {
        Product {
            id: p.id,
            name: p.name,
            code: p.code,
            distribution_strategy: p
                .distribution_strategy
                .as_ref()
                .and_then(|ds| serde_json::to_value(ds).ok())
                .and_then(|v| v.as_str().map(String::from)),
            url: p.url,
            platforms: p.platforms.map(|ps| {
                ps.iter()
                    .map(|p| {
                        serde_json::to_value(p)
                            .ok()
                            .and_then(|v| v.as_str().map(String::from))
                            .unwrap_or_default()
                    })
                    .collect()
            }),
            permissions: p.permissions,
            metadata: p
                .metadata
                .map(|m| serde_json::to_value(m).unwrap_or_default()),
            created: p.created,
            updated: p.updated,
            account_id: p.account_id,
        }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct CreateProductRequest {
    pub name: String,
    pub code: String,
    pub distribution_strategy: Option<String>,
    pub url: Option<String>,
    pub platforms: Option<Vec<String>>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    pub distribution_strategy: Option<String>,
    pub url: Option<String>,
    pub platforms: Option<Vec<String>>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct ListProductsOptions {
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub page_number: Option<u32>,
}

#[napi(object)]
#[derive(Clone)]
pub struct CreateTokenRequest {
    pub name: Option<String>,
    pub expiry: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

fn make_product(id: String) -> keygen_rs::product::Product {
    keygen_rs::product::Product {
        id,
        name: String::new(),
        code: None,
        distribution_strategy: None,
        url: None,
        platforms: None,
        permissions: None,
        metadata: None,
        created: String::new(),
        updated: String::new(),
        account_id: None,
    }
}

fn parse_distribution_strategy(s: &str) -> Result<keygen_rs::product::DistributionStrategy> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).map_err(|e| {
        napi::Error::new(
            Status::InvalidArg,
            format!("Invalid distribution strategy: {e}"),
        )
    })
}

fn parse_platforms(ps: Vec<String>) -> Result<Vec<keygen_rs::product::Platform>> {
    ps.into_iter()
        .map(|s| {
            serde_json::from_value::<keygen_rs::product::Platform>(serde_json::Value::String(s))
                .map_err(|e| napi::Error::new(Status::InvalidArg, format!("Invalid platform: {e}")))
        })
        .collect()
}

use crate::to_metadata;

#[napi]
pub async fn create_product(request: CreateProductRequest) -> Result<Product> {
    let req = keygen_rs::product::CreateProductRequest {
        name: request.name,
        code: request.code,
        distribution_strategy: request
            .distribution_strategy
            .as_deref()
            .map(parse_distribution_strategy)
            .transpose()?,
        url: request.url,
        platforms: request.platforms.map(parse_platforms).transpose()?,
        permissions: request.permissions,
        metadata: request.metadata.map(to_metadata).transpose()?,
    };

    keygen_rs::product::Product::create(req)
        .await
        .map(Product::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn list_products(options: Option<ListProductsOptions>) -> Result<Vec<Product>> {
    let opts = options.map(|o| keygen_rs::product::ListProductsOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
    });

    keygen_rs::product::Product::list(opts)
        .await
        .map(|ps| ps.into_iter().map(Product::from).collect())
        .map_err(to_napi_error)
}

#[napi]
pub async fn get_product(id: String) -> Result<Product> {
    keygen_rs::product::Product::get(&id)
        .await
        .map(Product::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn update_product(id: String, request: UpdateProductRequest) -> Result<Product> {
    let product = make_product(id);

    let req = keygen_rs::product::UpdateProductRequest {
        name: request.name,
        code: request.code,
        distribution_strategy: request
            .distribution_strategy
            .as_deref()
            .map(parse_distribution_strategy)
            .transpose()?,
        url: request.url,
        platforms: request.platforms.map(parse_platforms).transpose()?,
        permissions: request.permissions,
        metadata: request.metadata.map(to_metadata).transpose()?,
    };

    product
        .update(req)
        .await
        .map(Product::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn delete_product(id: String) -> Result<()> {
    let product = make_product(id);
    product.delete().await.map_err(to_napi_error)
}

#[napi]
pub async fn generate_product_token(
    id: String,
    request: Option<CreateTokenRequest>,
) -> Result<Token> {
    let product = make_product(id);
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

    product
        .generate_token_with_options(req)
        .await
        .map(Token::from)
        .map_err(to_napi_error)
}
