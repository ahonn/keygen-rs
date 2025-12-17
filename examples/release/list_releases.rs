use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    release::{ListReleasesOptions, Release, ReleaseChannel, ReleaseStatus},
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Set up configuration with Admin Token
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    })?;

    // List all releases (no filters)
    println!("=== All Releases ===");
    match Release::list(None).await {
        Ok(releases) => {
            println!("Found {} releases:", releases.len());
            for release in &releases {
                print_release(release);
            }
        }
        Err(e) => {
            println!("Failed to list releases: {e:?}");
        }
    }

    // List only stable releases
    println!("\n=== Stable Releases ===");
    let options = ListReleasesOptions {
        channel: Some(ReleaseChannel::Stable),
        limit: Some(10),
        ..Default::default()
    };

    match Release::list(Some(options)).await {
        Ok(releases) => {
            println!("Found {} stable releases:", releases.len());
            for release in &releases {
                print_release(release);
            }
        }
        Err(e) => {
            println!("Failed to list stable releases: {e:?}");
        }
    }

    // List only published releases
    println!("\n=== Published Releases ===");
    let options = ListReleasesOptions {
        status: Some(ReleaseStatus::Published),
        limit: Some(10),
        ..Default::default()
    };

    match Release::list(Some(options)).await {
        Ok(releases) => {
            println!("Found {} published releases:", releases.len());
            for release in &releases {
                print_release(release);
            }
        }
        Err(e) => {
            println!("Failed to list published releases: {e:?}");
        }
    }

    Ok(())
}

fn print_release(release: &Release) {
    println!("  ---");
    println!("  ID: {}", release.id);
    println!("  Version: {}", release.version);
    println!("  Channel: {:?}", release.channel);
    println!("  Status: {:?}", release.status);
    println!("  Name: {:?}", release.name);
    println!("  Tag: {:?}", release.tag);
    println!("  Created: {}", release.created);
    println!("  Product ID: {:?}", release.product_id);
}
