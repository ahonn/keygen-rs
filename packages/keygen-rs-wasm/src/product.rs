use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::to_js_error;
use crate::token_module::Token;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

fn parse_distribution_strategy(
    s: &str,
) -> Result<keygen_rs::product::DistributionStrategy, JsError> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .map_err(|e| JsError::new(&format!("Invalid distribution strategy: {e}")))
}

fn parse_platforms(ps: Vec<String>) -> Result<Vec<keygen_rs::product::Platform>, JsError> {
    ps.into_iter()
        .map(|s| {
            serde_json::from_value::<keygen_rs::product::Platform>(serde_json::Value::String(s))
                .map_err(|e| JsError::new(&format!("Invalid platform: {e}")))
        })
        .collect()
}

#[wasm_bindgen(js_name = "createProduct")]
pub async fn create_product(request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        name: String,
        code: String,
        distribution_strategy: Option<String>,
        url: Option<String>,
        platforms: Option<Vec<String>>,
        permissions: Option<Vec<String>>,
        metadata: Option<serde_json::Value>,
    }
    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;

    let r = keygen_rs::product::CreateProductRequest {
        name: req.name,
        code: req.code,
        distribution_strategy: req
            .distribution_strategy
            .as_deref()
            .map(parse_distribution_strategy)
            .transpose()?,
        url: req.url,
        platforms: req.platforms.map(parse_platforms).transpose()?,
        permissions: req.permissions,
        metadata: crate::opt_metadata(req.metadata)?,
    };

    let product = keygen_rs::product::Product::create(r)
        .await
        .map(Product::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&product).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "listProducts")]
pub async fn list_products(options: JsValue) -> Result<JsValue, JsError> {
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

    let list_opts = opts.map(|o| keygen_rs::product::ListProductsOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
    });

    let products: Vec<Product> = keygen_rs::product::Product::list(list_opts)
        .await
        .map(|ps| ps.into_iter().map(Product::from).collect())
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&products).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "getProduct")]
pub async fn get_product(id: String) -> Result<JsValue, JsError> {
    let product = keygen_rs::product::Product::get(&id)
        .await
        .map(Product::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&product).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "updateProduct")]
pub async fn update_product(id: String, request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        name: Option<String>,
        code: Option<String>,
        distribution_strategy: Option<String>,
        url: Option<String>,
        platforms: Option<Vec<String>>,
        permissions: Option<Vec<String>>,
        metadata: Option<serde_json::Value>,
    }
    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;

    let p = make_product(id);

    let r = keygen_rs::product::UpdateProductRequest {
        name: req.name,
        code: req.code,
        distribution_strategy: req
            .distribution_strategy
            .as_deref()
            .map(parse_distribution_strategy)
            .transpose()?,
        url: req.url,
        platforms: req.platforms.map(parse_platforms).transpose()?,
        permissions: req.permissions,
        metadata: crate::opt_metadata(req.metadata)?,
    };

    let product = p.update(r).await.map(Product::from).map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&product).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "deleteProduct")]
pub async fn delete_product(id: String) -> Result<(), JsError> {
    let product = make_product(id);
    product.delete().await.map_err(to_js_error)
}

#[wasm_bindgen(js_name = "generateProductToken")]
pub async fn generate_product_token(id: String, request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        name: Option<String>,
        expiry: Option<String>,
        permissions: Option<Vec<String>>,
        metadata: Option<serde_json::Value>,
    }

    let req: Option<Req> = if request.is_undefined() || request.is_null() {
        None
    } else {
        Some(serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?)
    };

    let req = req
        .map(
            |request| -> Result<keygen_rs::token::CreateTokenRequest, JsError> {
                Ok(keygen_rs::token::CreateTokenRequest {
                    name: request.name,
                    expiry: request.expiry,
                    permissions: request.permissions,
                    metadata: crate::opt_metadata(request.metadata)?,
                })
            },
        )
        .transpose()?;

    let token = make_product(id)
        .generate_token_with_options(req)
        .await
        .map(Token::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&token).map_err(|e| JsError::new(&e.to_string()))
}
