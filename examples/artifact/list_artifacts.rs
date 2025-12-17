use keygen_rs::{
    artifact::{Artifact, ListArtifactsOptions},
    config::{self, KeygenConfig},
    errors::Error,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();

    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    })?;

    let options = ListArtifactsOptions {
        limit: Some(10),
        ..Default::default()
    };

    match Artifact::list(Some(options)).await {
        Ok(artifacts) => {
            println!("Found {} artifacts:", artifacts.len());
            for artifact in artifacts {
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
        }
        Err(e) => {
            println!("Failed to list artifacts: {e:?}");
        }
    }

    Ok(())
}
