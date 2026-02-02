use keygen_rs::{
    artifact::Artifact,
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

    let artifact_id = env::var("KEYGEN_ARTIFACT_ID").expect("KEYGEN_ARTIFACT_ID must be set");

    match Artifact::get(&artifact_id).await {
        Ok(artifact) => {
            println!("Artifact found!");
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
        Err(e) => {
            println!("Failed to get artifact: {e:?}");
        }
    }

    Ok(())
}
