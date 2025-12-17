use keygen_rs::{
    artifact::{Artifact, ListArtifactsOptions},
    config::{self, KeygenConfig},
    errors::Error,
};
use std::env;

/// This example demonstrates listing and getting artifacts.
///
/// Note: Creating artifacts in keygen.sh is a two-step process:
/// 1. POST /artifacts - creates the artifact record and returns a redirect URL
/// 2. PUT to the S3 URL - uploads the actual file content
///
/// The current SDK focuses on the metadata operations (list, get, update, yank).
/// For full artifact upload, you may need to handle the redirect separately.
#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();

    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    })?;

    // List all artifacts
    let options = ListArtifactsOptions {
        limit: Some(10),
        ..Default::default()
    };

    match Artifact::list(Some(options)).await {
        Ok(artifacts) => {
            println!("Found {} artifacts:", artifacts.len());
            for artifact in &artifacts {
                println!("  - {} [{}]", artifact.filename, artifact.id);
                println!("    Status: {:?}", artifact.status);
                if let Some(platform) = &artifact.platform {
                    print!("    Platform: {}", platform);
                }
                if let Some(arch) = &artifact.arch {
                    println!(", Arch: {}", arch);
                } else {
                    println!();
                }
            }

            // If we have artifacts, get details of the first one
            if let Some(first) = artifacts.first() {
                println!("\n=== Artifact Details ===");
                match Artifact::get(&first.id).await {
                    Ok(artifact) => {
                        println!("  ID: {}", artifact.id);
                        println!("  Filename: {}", artifact.filename);
                        println!("  Filetype: {:?}", artifact.filetype);
                        println!("  Filesize: {:?}", artifact.filesize);
                        println!("  Platform: {:?}", artifact.platform);
                        println!("  Arch: {:?}", artifact.arch);
                        println!("  Status: {:?}", artifact.status);
                        println!("  Checksum: {:?}", artifact.checksum);
                        println!("  Created: {}", artifact.created);
                    }
                    Err(e) => println!("Failed to get artifact details: {e:?}"),
                }
            }
        }
        Err(e) => {
            println!("Failed to list artifacts: {e:?}");
        }
    }

    Ok(())
}
