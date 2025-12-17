use keygen_rs::{
    artifact::{Artifact, CreateArtifactRequest},
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

    let release_id = env::var("KEYGEN_RELEASE_ID").expect("KEYGEN_RELEASE_ID must be set");

    let request = CreateArtifactRequest {
        filename: "my-app-1.0.0-darwin-amd64.dmg".to_string(),
        release_id,
        filetype: Some("dmg".to_string()),
        filesize: Some(10485760), // 10 MB
        platform: Some("darwin".to_string()),
        arch: Some("amd64".to_string()),
        signature: None,
        checksum: Some("abc123def456...".to_string()),
        metadata: None,
    };

    match Artifact::create(request).await {
        Ok(artifact) => {
            println!("Artifact created successfully!");
            println!("  ID: {}", artifact.id);
            println!("  Filename: {}", artifact.filename);
            println!("  Filetype: {:?}", artifact.filetype);
            println!("  Filesize: {:?}", artifact.filesize);
            println!("  Platform: {:?}", artifact.platform);
            println!("  Arch: {:?}", artifact.arch);
            println!("  Status: {:?}", artifact.status);
            println!("  Created: {}", artifact.created);
        }
        Err(e) => {
            println!("Failed to create artifact: {e:?}");
        }
    }

    Ok(())
}
