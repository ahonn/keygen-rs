#[path = "../common/mod.rs"]
mod common;

use chrono::{Duration, Utc};
use keygen_rs::{
    errors::Error,
    group::{CreateGroupRequest, Group},
    license::{License, LicenseCreateRequest, PaginationOptions},
    token::{CreateTokenRequest, Token},
    user::{CreateUserRequest, User, UserRole},
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Error> {
    common::load_env();
    common::configure_admin()?;

    let policy_id = common::required_env("KEYGEN_POLICY_ID");
    let suffix = common::unique_suffix();

    let mut created_group: Option<Group> = None;
    let mut created_owner_user_id: Option<String> = None;
    let mut created_attached_user_id: Option<String> = None;
    let mut created_license: Option<License> = None;
    let mut created_token: Option<Token> = None;

    let result = async {
        let group = Group::create(CreateGroupRequest {
            name: format!("example-license-group-{suffix}"),
            max_users: Some(5),
            max_licenses: Some(5),
            max_machines: Some(5),
            metadata: None,
        })
        .await?;
        println!("Created temp group: {}", group.id);
        created_group = Some(group.clone());

        let owner_user = User::create(CreateUserRequest {
            email: format!("license-owner-{suffix}@example.com"),
            first_name: Some("License".to_string()),
            last_name: Some("Example".to_string()),
            role: Some(UserRole::User),
            permissions: None,
            metadata: None,
        })
        .await?;
        println!("Created temp owner user: {}", owner_user.id);
        created_owner_user_id = Some(owner_user.id.clone());

        let attached_user = User::create(CreateUserRequest {
            email: format!("license-attached-{suffix}@example.com"),
            first_name: Some("Attached".to_string()),
            last_name: Some("Example".to_string()),
            role: Some(UserRole::User),
            permissions: None,
            metadata: None,
        })
        .await?;
        println!("Created temp attached user: {}", attached_user.id);
        created_attached_user_id = Some(attached_user.id.clone());

        let mut metadata = HashMap::new();
        metadata.insert(
            "example".to_string(),
            serde_json::Value::String("license-relationship-lifecycle".to_string()),
        );

        let license = License::create(
            LicenseCreateRequest::new(policy_id.clone())
                .with_name(format!("Example License {suffix}"))
                .with_metadata(metadata),
        )
        .await?;
        println!("Created temp license: {} ({})", license.id, license.key);
        created_license = Some(license.clone());

        let license = license.change_owner(&owner_user.id).await?;
        println!("Changed license owner to {}", owner_user.id);
        created_license = Some(license.clone());

        license
            .attach_users(std::slice::from_ref(&attached_user.id))
            .await?;
        let attached_users = license
            .users(Some(&PaginationOptions {
                limit: Some(10),
                page_number: None,
                page_size: None,
            }))
            .await?;
        println!("Attached users: {}", attached_users.len());

        let license = license.change_group(&group.id).await?;
        println!("Changed license group to {}", group.id);

        let license = license.change_policy(&policy_id).await?;
        println!("Re-applied license policy {}", policy_id);
        created_license = Some(license.clone());

        let token = license
            .generate_token(Some(CreateTokenRequest {
                name: Some(format!("example-license-token-{suffix}")),
                expiry: Some((Utc::now() + Duration::hours(1)).to_rfc3339()),
                permissions: None,
                metadata: None,
            }))
            .await?;
        println!(
            "Generated license token: {} (token value returned: {})",
            token.id,
            token.token.is_some()
        );
        created_token = Some(token);

        license
            .detach_users(std::slice::from_ref(&attached_user.id))
            .await?;
        println!("Detached user from license");

        Ok(())
    }
    .await;

    if let Some(token) = created_token.as_ref() {
        if let Err(err) = token.revoke().await {
            eprintln!("Cleanup: failed to revoke temp token {}: {err:?}", token.id);
        }
    }
    if let Some(license) = created_license.as_ref() {
        if let Err(err) = license.delete().await {
            eprintln!(
                "Cleanup: failed to delete temp license {}: {err:?}",
                license.id
            );
        }
    }
    if let Some(user_id) = created_attached_user_id.as_ref() {
        if let Err(err) = User::delete(user_id).await {
            eprintln!("Cleanup: failed to delete temp user {}: {err:?}", user_id);
        }
    }
    if let Some(user_id) = created_owner_user_id.as_ref() {
        if let Err(err) = User::delete(user_id).await {
            eprintln!("Cleanup: failed to delete temp user {}: {err:?}", user_id);
        }
    }
    if let Some(group) = created_group.as_ref() {
        if let Err(err) = group.delete().await {
            eprintln!("Cleanup: failed to delete temp group {}: {err:?}", group.id);
        }
    }

    result
}
