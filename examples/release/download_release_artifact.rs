use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    release::{ArtifactDownloadOptions, Release},
};
use std::env;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Set up configuration with License Key (no admin token required).
    // With the account public key set, API responses are signature-checked,
    // so the artifact checksum used below is trusted as coming from Keygen.
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        product: env::var("KEYGEN_PRODUCT").expect("KEYGEN_PRODUCT must be set"),
        license_key: Some(env::var("KEYGEN_LICENSE_KEY").expect("KEYGEN_LICENSE_KEY must be set")),
        public_key: env::var("KEYGEN_PUBLIC_KEY").ok(),
        ..KeygenConfig::default()
    })?;

    let release_id = env::var("KEYGEN_RELEASE_ID").expect("KEYGEN_RELEASE_ID must be set");
    // Artifact filename as published for this platform, e.g. `app_linux_amd64`
    // (the naming scheme is up to the publisher, see `std::env::consts::{OS, ARCH}`)
    let filename = env::var("KEYGEN_ARTIFACT").expect("KEYGEN_ARTIFACT must be set");
    // Personal key from `keygen genkey`, not KEYGEN_PUBLIC_KEY of the account
    let signing_key = env::var("KEYGEN_SIGNING_PUBLIC_KEY").ok();
    if signing_key.is_none() {
        println!("KEYGEN_SIGNING_PUBLIC_KEY is not set: only the checksum will be verified");
    }

    let release = Release::get(&release_id).await?;
    println!("Release: {} ({})", release.version, release.id);

    let path = env::temp_dir().join(&filename);
    let options = ArtifactDownloadOptions {
        public_key: signing_key,
        on_progress: Some(Box::new(|downloaded, total| {
            match total {
                Some(total) => print!("\r  {downloaded}/{total} bytes"),
                None => print!("\r  {downloaded} bytes"),
            }
            std::io::stdout().flush().ok();
        })),
    };

    // The file appears at `path` only after verification succeeded
    let artifact = release
        .download_artifact_to(&filename, &path, Some(options))
        .await;
    println!();
    let artifact = artifact?;
    println!("Downloaded and verified {}", artifact.filename);
    println!("  Path: {}", path.display());

    // Downloaded files are not executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| Error::UnexpectedError(format!("Failed to set permissions: {e}")))?;
    }

    // Replacing the running executable is up to the application,
    // e.g. with the `self-replace` crate or the Tauri updater.
    Ok(())
}
