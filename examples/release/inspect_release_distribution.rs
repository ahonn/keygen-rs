#[path = "../common/mod.rs"]
mod common;

use keygen_rs::{
    artifact::ListArtifactsOptions,
    errors::Error,
    release::{
        CreateReleaseRequest, ListReleasesOptions, Release, ReleaseChannel, ReleaseStatus,
        ReleaseUpgradeRequest,
    },
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    common::load_env();
    common::configure_admin()?;

    let product_id = common::product_id_from_env();
    let suffix = common::unique_suffix();
    let mut created_releases: Vec<Release> = Vec::new();

    let result = async {
        let releases = Release::list(Some(ListReleasesOptions {
            product: Some(product_id.clone()),
            limit: Some(5),
            ..Default::default()
        }))
        .await?;

        let (upgrade_source, artifact_release) = if releases.is_empty() {
            let patch = suffix
                .parse::<u64>()
                .map(|value| (value % 100_000) as u32)
                .unwrap_or(0);
            let base_version = format!("9.9.{patch}");
            let next_patch = patch + 1;
            let next_version = format!("9.9.{next_patch}");

            let older = Release::create(CreateReleaseRequest {
                version: base_version.clone(),
                channel: ReleaseChannel::Stable,
                product_id: product_id.clone(),
                name: Some(format!("Example Release {base_version}")),
                description: Some("Temporary example release".to_string()),
                status: Some(ReleaseStatus::Draft),
                tag: Some(format!("example-{base_version}")),
                metadata: None,
            })
            .await?
            .publish()
            .await?;

            let newer = Release::create(CreateReleaseRequest {
                version: next_version.clone(),
                channel: ReleaseChannel::Stable,
                product_id: product_id.clone(),
                name: Some(format!("Example Release {next_version}")),
                description: Some("Temporary example release".to_string()),
                status: Some(ReleaseStatus::Draft),
                tag: Some(format!("example-{next_version}")),
                metadata: None,
            })
            .await?
            .publish()
            .await?;

            println!(
                "Created temp releases for distribution checks: {} -> {}",
                older.version, newer.version
            );
            created_releases.push(newer.clone());
            created_releases.push(older.clone());

            (older, newer)
        } else {
            let release = releases
                .into_iter()
                .next()
                .expect("checked release list is non-empty");
            println!(
                "Using existing release {} ({}) on channel {:?}",
                release.version, release.id, release.channel
            );
            (release.clone(), release)
        };

        let artifacts = artifact_release
            .artifacts(Some(ListArtifactsOptions {
                limit: Some(5),
                ..Default::default()
            }))
            .await?;
        println!(
            "Artifacts attached to release {}: {}",
            artifact_release.id,
            artifacts.len()
        );

        if let Some(artifact) = artifacts.first() {
            match artifact_release.download_artifact(&artifact.id).await {
                Ok(download) => {
                    println!(
                        "Artifact redirect location for {}: {}",
                        artifact.filename, download.location
                    );
                }
                Err(err) => {
                    println!(
                        "Download redirect lookup failed for {}: {err:?}",
                        artifact.id
                    );
                }
            }
        }

        match upgrade_source
            .upgrade(Some(&ReleaseUpgradeRequest {
                product: Some(product_id),
                channel: Some(upgrade_source.channel.clone()),
                ..Default::default()
            }))
            .await
        {
            Ok(upgrade) => {
                println!("Upgrade candidate: {} ({})", upgrade.version, upgrade.id);
            }
            Err(err) => {
                println!("Upgrade lookup returned an error: {err:?}");
            }
        }

        Ok(())
    }
    .await;

    for release in &created_releases {
        if let Err(err) = release.delete().await {
            eprintln!(
                "Cleanup: failed to delete temp release {}: {err:?}",
                release.id
            );
        }
    }

    result
}
