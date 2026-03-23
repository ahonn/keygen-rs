use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
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

#[napi(object)]
#[derive(Clone)]
pub struct ListTokensOptions {
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub page_number: Option<u32>,
    pub bearer_type: Option<String>,
    pub bearer_id: Option<String>,
}

#[napi(object)]
#[derive(Clone)]
pub struct RegenerateTokenRequest {
    pub name: Option<String>,
    pub expiry: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
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

#[napi]
pub async fn list_tokens(options: Option<ListTokensOptions>) -> Result<Vec<Token>> {
    let opts = options.map(|o| keygen_rs::token::ListTokensOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
        bearer_type: o.bearer_type,
        bearer_id: o.bearer_id,
    });

    keygen_rs::token::Token::list(opts)
        .await
        .map(|ts| ts.into_iter().map(Token::from).collect())
        .map_err(to_napi_error)
}

#[napi]
pub async fn get_token(id: String) -> Result<Token> {
    keygen_rs::token::Token::get(&id)
        .await
        .map(Token::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn regenerate_token(id: String, request: RegenerateTokenRequest) -> Result<Token> {
    let token = make_token(id);

    let req = keygen_rs::token::RegenerateTokenRequest {
        name: request.name,
        expiry: request.expiry,
        permissions: request.permissions,
        metadata: crate::opt_metadata(request.metadata)?,
    };

    token
        .regenerate(req)
        .await
        .map(Token::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn revoke_token(id: String) -> Result<()> {
    let token = make_token(id);
    token.revoke().await.map_err(to_napi_error)
}
