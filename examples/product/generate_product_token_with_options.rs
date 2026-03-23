#[path = "../common/mod.rs"]
mod common;

use chrono::{Duration, Utc};
use keygen_rs::{
    errors::Error,
    product::Product,
    token::{CreateTokenRequest, ListTokensOptions},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    common::load_env();
    common::configure_admin()?;

    let product_id = common::product_id_from_env();
    let product = Product::get(&product_id).await?;
    let suffix = common::unique_suffix();

    let token = product
        .generate_token_with_options(Some(CreateTokenRequest {
            name: Some(format!("example-product-token-{suffix}")),
            expiry: Some((Utc::now() + Duration::hours(1)).to_rfc3339()),
            permissions: None,
            metadata: None,
        }))
        .await?;

    println!(
        "Generated product token: {} (token value returned: {})",
        token.id,
        token.token.is_some()
    );

    let listed = keygen_rs::token::Token::list(Some(ListTokensOptions {
        limit: Some(10),
        bearer_type: Some("products".to_string()),
        bearer_id: Some(product.id.clone()),
        ..Default::default()
    }))
    .await?;

    let found = listed.iter().any(|item| item.id == token.id);
    println!(
        "Bearer-filtered token count for product {}: {} (generated token found: {})",
        product.id,
        listed.len(),
        found
    );

    token.revoke().await?;
    println!("Revoked temp product token: {}", token.id);

    Ok(())
}
