use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::to_js_error;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub id: String,
    pub kind: String,
    pub token: Option<String>,
    pub name: Option<String>,
    pub expiry: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub created: String,
    pub updated: String,
}

impl From<keygen_rs::token::Token> for Token {
    fn from(t: keygen_rs::token::Token) -> Self {
        Token {
            id: t.id,
            kind: serde_json::to_value(&t.kind)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_default(),
            token: t.token,
            name: t.name,
            expiry: t.expiry,
            permissions: t.permissions,
            created: t.created,
            updated: t.updated,
        }
    }
}

fn make_token(id: String) -> keygen_rs::token::Token {
    keygen_rs::token::Token {
        id,
        kind: keygen_rs::token::TokenKind::ActivationToken,
        token: None,
        name: None,
        expiry: None,
        permissions: None,
        metadata: None,
        created: String::new(),
        updated: String::new(),
    }
}

#[wasm_bindgen(js_name = "listTokens")]
pub async fn list_tokens(options: JsValue) -> Result<JsValue, JsError> {
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

    let keygen_opts = opts.map(|o| keygen_rs::token::ListTokensOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
    });

    let tokens: Vec<Token> = keygen_rs::token::Token::list(keygen_opts)
        .await
        .map(|ts| ts.into_iter().map(Token::from).collect())
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&tokens).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "getToken")]
pub async fn get_token(id: String) -> Result<JsValue, JsError> {
    let token = keygen_rs::token::Token::get(&id)
        .await
        .map(Token::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&token).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "regenerateToken")]
pub async fn regenerate_token(id: String, request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        name: Option<String>,
        expiry: Option<String>,
        permissions: Option<Vec<String>>,
        metadata: Option<serde_json::Value>,
    }

    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;

    let r = keygen_rs::token::RegenerateTokenRequest {
        name: req.name,
        expiry: req.expiry,
        permissions: req.permissions,
        metadata: crate::opt_metadata(req.metadata)?,
    };

    let token = make_token(id);
    let result = token
        .regenerate(r)
        .await
        .map(Token::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "revokeToken")]
pub async fn revoke_token(id: String) -> Result<(), JsError> {
    let token = make_token(id);
    token.revoke().await.map_err(to_js_error)
}
