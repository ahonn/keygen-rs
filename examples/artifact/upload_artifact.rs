use keygen_rs::{
    artifact::{Artifact, ArtifactUploadOptions, CreateArtifactRequest},
    config::{self, KeygenConfig},
    errors::Error,
};
use std::{env, io::Write, path::Path};

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
    let path = env::var("ARTIFACT_PATH").expect("ARTIFACT_PATH must be set");
    let filename = Path::new(&path)
        .file_name()
        .and_then(|name| name.to_str())
        .expect("ARTIFACT_PATH must point to a file")
        .to_string();

    // filesize and checksum are computed from the file when left empty
    let request = CreateArtifactRequest {
        filename,
        release_id,
        filetype: Path::new(&path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_string),
        filesize: None,
        platform: env::var("ARTIFACT_PLATFORM").ok(),
        arch: env::var("ARTIFACT_ARCH").ok(),
        // Not signed: downloads with a signing public key would reject it.
        // Sign with `keygen upload --signing-key` or pass the signature here.
        signature: None,
        checksum: None,
        metadata: None,
    };

    let options = ArtifactUploadOptions {
        on_progress: Some(Box::new(|uploaded, total| {
            print!("\r  {uploaded}/{total} bytes");
            std::io::stdout().flush().ok();
        })),
    };

    let result = Artifact::create_and_upload(request, &path, Some(options)).await;
    println!();
    let artifact = result?;

    println!("Artifact uploaded successfully!");
    println!("  ID: {}", artifact.id);
    println!("  Filename: {}", artifact.filename);
    println!("  Filesize: {:?}", artifact.filesize);
    println!("  Checksum: {:?}", artifact.checksum);
    println!("  Status: {:?}", artifact.status);

    Ok(())
}
