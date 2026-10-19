use crate::client::Client;
use crate::errors::Error;
use crate::KeygenResponseData;
#[cfg(not(target_arch = "wasm32"))]
use base64::{
    alphabet,
    engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig},
    Engine,
};
#[cfg(not(target_arch = "wasm32"))]
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use sha2::{Digest, Sha512};
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use subtle::ConstantTimeEq;

/// Base64 of artifact checksums and signatures: `keygen upload` writes them unpadded
/// by default, so encode without padding and accept both when decoding.
#[cfg(not(target_arch = "wasm32"))]
const ARTIFACT_BASE64: GeneralPurpose = GeneralPurpose::new(
    &alphabet::STANDARD,
    GeneralPurposeConfig::new()
        .with_encode_padding(false)
        .with_decode_padding_mode(DecodePaddingMode::Indifferent),
);

/// Artifact status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArtifactStatus {
    Waiting,
    Uploaded,
    Failed,
    Yanked,
}

/// Artifact attributes from API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactAttributes {
    pub filename: String,
    pub filetype: Option<String>,
    pub filesize: Option<u64>,
    pub platform: Option<String>,
    pub arch: Option<String>,
    pub signature: Option<String>,
    pub checksum: Option<String>,
    pub status: ArtifactStatus,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: String,
    pub updated: String,
    #[serde(rename = "yanked")]
    pub yanked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ArtifactResponse {
    pub data: KeygenResponseData<ArtifactAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ArtifactsResponse {
    pub data: Vec<KeygenResponseData<ArtifactAttributes>>,
}

/// Request to create a new artifact
#[cfg(feature = "token")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateArtifactRequest {
    /// The filename of the artifact
    pub filename: String,
    /// Associated release ID
    pub release_id: String,
    /// Optional: File type/extension
    pub filetype: Option<String>,
    /// Optional: File size in bytes
    pub filesize: Option<u64>,
    /// Optional: Target platform (e.g., "darwin", "linux", "win32")
    pub platform: Option<String>,
    /// Optional: Target architecture (e.g., "amd64", "arm64")
    pub arch: Option<String>,
    /// Optional: Signature for verification
    pub signature: Option<String>,
    /// Optional: Checksum (e.g., SHA-256)
    pub checksum: Option<String>,
    /// Optional: Custom metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Request to update an existing artifact
#[cfg(feature = "token")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateArtifactRequest {
    pub filename: Option<String>,
    pub filetype: Option<String>,
    pub platform: Option<String>,
    pub arch: Option<String>,
    pub signature: Option<String>,
    pub checksum: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Options for listing artifacts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListArtifactsOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(rename = "page[size]", skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(rename = "page[number]", skip_serializing_if = "Option::is_none")]
    pub page_number: Option<u32>,
    /// Filter by release ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<String>,
    /// Filter by product ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
    /// Filter by channel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    /// Filter by platform
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    /// Filter by architecture
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
    /// Filter by filetype
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filetype: Option<String>,
    /// Filter by status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ArtifactStatus>,
}

/// An artifact represents a distributable file associated with a release
#[derive(Debug, Clone)]
pub struct Artifact {
    pub id: String,
    pub filename: String,
    pub filetype: Option<String>,
    pub filesize: Option<u64>,
    pub platform: Option<String>,
    pub arch: Option<String>,
    pub signature: Option<String>,
    pub checksum: Option<String>,
    pub status: ArtifactStatus,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: String,
    pub updated: String,
    pub yanked_at: Option<String>,
    pub release_id: Option<String>,
    pub product_id: Option<String>,
    pub account_id: Option<String>,
}

impl Artifact {
    pub(crate) fn from(data: KeygenResponseData<ArtifactAttributes>) -> Artifact {
        Artifact {
            id: data.id,
            filename: data.attributes.filename,
            filetype: data.attributes.filetype,
            filesize: data.attributes.filesize,
            platform: data.attributes.platform,
            arch: data.attributes.arch,
            signature: data.attributes.signature,
            checksum: data.attributes.checksum,
            status: data.attributes.status,
            metadata: data.attributes.metadata,
            created: data.attributes.created,
            updated: data.attributes.updated,
            yanked_at: data.attributes.yanked_at,
            release_id: data
                .relationships
                .release
                .as_ref()
                .and_then(|r| r.data.as_ref().map(|d| d.id.clone())),
            product_id: data
                .relationships
                .product
                .as_ref()
                .and_then(|p| p.data.as_ref().map(|d| d.id.clone())),
            account_id: data
                .relationships
                .account
                .as_ref()
                .and_then(|a| a.data.as_ref().map(|d| d.id.clone())),
        }
    }

    /// List artifacts with optional filtering and pagination
    pub async fn list(options: Option<ListArtifactsOptions>) -> Result<Vec<Artifact>, Error> {
        let client = Client::from_global_config()?;
        let response = client.get("artifacts", options.as_ref()).await?;
        let artifacts_response: ArtifactsResponse = serde_json::from_value(response.body)?;
        Ok(artifacts_response
            .data
            .into_iter()
            .map(Artifact::from)
            .collect())
    }

    /// Get an artifact by ID
    pub async fn get(id: &str) -> Result<Artifact, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("artifacts/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let artifact_response: ArtifactResponse = serde_json::from_value(response.body)?;
        Ok(Artifact::from(artifact_response.data))
    }
}

/// Parse a hex Ed25519 public key generated by `keygen genkey`.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn parse_signing_public_key(public_key: &str) -> Result<VerifyingKey, Error> {
    let bytes: [u8; 32] = hex::decode(public_key)
        .map_err(|e| Error::ArtifactPublicKeyInvalid(e.to_string()))?
        .try_into()
        .map_err(|_| Error::ArtifactPublicKeyInvalid("expected 32 bytes".to_string()))?;
    VerifyingKey::from_bytes(&bytes).map_err(|e| Error::ArtifactPublicKeyInvalid(e.to_string()))
}

/// Incrementally verifies an artifact file, as uploaded by `keygen upload`:
/// the Ed25519ph signature when a public key is given, and the SHA-512 checksum.
///
/// Feed the file with [`update`](Self::update) and call [`finish`](Self::finish);
/// the data must not be used before `finish` returns `Ok`.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct ArtifactVerifier {
    hasher: Sha512,
    checksum: Option<Vec<u8>>,
    signature: Option<(VerifyingKey, Signature, String)>,
}

#[cfg(not(target_arch = "wasm32"))]
impl ArtifactVerifier {
    /// `public_key` is the hex key from `keygen genkey`, not the account key.
    /// `context` is the product ID the artifact was signed for.
    ///
    /// With a public key the artifact must be signed ([`Error::ArtifactSignatureMissing`]);
    /// the signature covers the SHA-512 of the file, so the checksum is then optional
    /// but still checked if present. Without a public key the checksum is required
    /// ([`Error::ArtifactChecksumMissing`]).
    pub(crate) fn new(
        artifact: &Artifact,
        public_key: Option<&str>,
        context: &str,
    ) -> Result<Self, Error> {
        let signature = match public_key {
            Some(public_key) => {
                let key = parse_signing_public_key(public_key)?;
                let signature = artifact
                    .signature
                    .as_deref()
                    .ok_or(Error::ArtifactSignatureMissing)?;
                let signature = ARTIFACT_BASE64
                    .decode(signature)
                    .ok()
                    .and_then(|bytes| Signature::from_slice(&bytes).ok())
                    .ok_or_else(|| {
                        Error::ArtifactSignatureInvalid("malformed signature".to_string())
                    })?;
                Some((key, signature, context.to_string()))
            }
            None => None,
        };

        let checksum = match (artifact.checksum.as_deref(), &signature) {
            (Some(checksum), _) => Some(
                ARTIFACT_BASE64
                    .decode(checksum)
                    .map_err(|_| Error::ArtifactChecksumMismatch)?,
            ),
            (None, Some(_)) => None,
            (None, None) => return Err(Error::ArtifactChecksumMissing),
        };

        Ok(Self {
            hasher: Sha512::new(),
            checksum,
            signature,
        })
    }

    pub(crate) fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    pub(crate) fn finish(self) -> Result<(), Error> {
        if let Some(checksum) = &self.checksum {
            let digest = self.hasher.clone().finalize();
            if !bool::from(digest.as_slice().ct_eq(checksum)) {
                return Err(Error::ArtifactChecksumMismatch);
            }
        }

        if let Some((key, signature, context)) = &self.signature {
            // Ed25519ph over SHA-512 of the file, as `keygen upload` and keygen-go do.
            key.verify_prehashed(self.hasher, Some(context.as_bytes()), signature)
                .map_err(|e| Error::ArtifactSignatureInvalid(e.to_string()))?;
        }

        Ok(())
    }
}

#[cfg(feature = "token")]
impl Artifact {
    /// Create a new artifact
    ///
    /// Note: After creating an artifact, you need to upload the actual file
    /// using the upload URL provided in the response links.
    pub async fn create(request: CreateArtifactRequest) -> Result<Artifact, Error> {
        let client = Client::from_global_config()?;

        let mut attributes = serde_json::Map::new();
        attributes.insert(
            "filename".to_string(),
            serde_json::Value::String(request.filename),
        );

        if let Some(filetype) = request.filetype {
            attributes.insert("filetype".to_string(), serde_json::Value::String(filetype));
        }
        if let Some(filesize) = request.filesize {
            attributes.insert("filesize".to_string(), serde_json::json!(filesize));
        }
        if let Some(platform) = request.platform {
            attributes.insert("platform".to_string(), serde_json::Value::String(platform));
        }
        if let Some(arch) = request.arch {
            attributes.insert("arch".to_string(), serde_json::Value::String(arch));
        }
        if let Some(signature) = request.signature {
            attributes.insert(
                "signature".to_string(),
                serde_json::Value::String(signature),
            );
        }
        if let Some(checksum) = request.checksum {
            attributes.insert("checksum".to_string(), serde_json::Value::String(checksum));
        }
        if let Some(metadata) = request.metadata {
            attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }

        let body = serde_json::json!({
            "data": {
                "type": "artifacts",
                "attributes": attributes,
                "relationships": {
                    "release": {
                        "data": {
                            "type": "releases",
                            "id": request.release_id
                        }
                    }
                }
            }
        });

        let response = client.post("artifacts", Some(&body), None::<&()>).await?;
        let artifact_response: ArtifactResponse = serde_json::from_value(response.body)?;
        Ok(Artifact::from(artifact_response.data))
    }

    /// Update an existing artifact
    pub async fn update(&self, request: UpdateArtifactRequest) -> Result<Artifact, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("artifacts/{}", self.id);

        let mut attributes = serde_json::Map::new();
        if let Some(filename) = request.filename {
            attributes.insert("filename".to_string(), serde_json::Value::String(filename));
        }
        if let Some(filetype) = request.filetype {
            attributes.insert("filetype".to_string(), serde_json::Value::String(filetype));
        }
        if let Some(platform) = request.platform {
            attributes.insert("platform".to_string(), serde_json::Value::String(platform));
        }
        if let Some(arch) = request.arch {
            attributes.insert("arch".to_string(), serde_json::Value::String(arch));
        }
        if let Some(signature) = request.signature {
            attributes.insert(
                "signature".to_string(),
                serde_json::Value::String(signature),
            );
        }
        if let Some(checksum) = request.checksum {
            attributes.insert("checksum".to_string(), serde_json::Value::String(checksum));
        }
        if let Some(metadata) = request.metadata {
            attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }

        let body = serde_json::json!({
            "data": {
                "type": "artifacts",
                "attributes": attributes
            }
        });

        let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
        let artifact_response: ArtifactResponse = serde_json::from_value(response.body)?;
        Ok(Artifact::from(artifact_response.data))
    }

    /// Delete an artifact
    pub async fn delete(&self) -> Result<(), Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("artifacts/{}", self.id);
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }

    /// Yank an artifact (make it unavailable for download)
    pub async fn yank(&self) -> Result<Artifact, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("artifacts/{}/actions/yank", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let artifact_response: ArtifactResponse = serde_json::from_value(response.body)?;
        Ok(Artifact::from(artifact_response.data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        KeygenRelationship, KeygenRelationshipData, KeygenRelationships, KeygenResponseData,
    };

    #[test]
    fn test_artifact_from_response_data() {
        let artifact_data = KeygenResponseData {
            id: "test-artifact-id".to_string(),
            r#type: "artifacts".to_string(),
            attributes: ArtifactAttributes {
                filename: "app-1.0.0.dmg".to_string(),
                filetype: Some("dmg".to_string()),
                filesize: Some(10485760),
                platform: Some("darwin".to_string()),
                arch: Some("amd64".to_string()),
                signature: None,
                checksum: Some("abc123".to_string()),
                status: ArtifactStatus::Uploaded,
                metadata: Some(HashMap::new()),
                created: "2023-01-01T00:00:00Z".to_string(),
                updated: "2023-01-01T00:00:00Z".to_string(),
                yanked_at: None,
            },
            relationships: KeygenRelationships {
                release: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "releases".to_string(),
                        id: "test-release-id".to_string(),
                    }),
                    links: None,
                }),
                account: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "accounts".to_string(),
                        id: "test-account-id".to_string(),
                    }),
                    links: None,
                }),
                ..Default::default()
            },
        };

        let artifact = Artifact::from(artifact_data);

        assert_eq!(artifact.id, "test-artifact-id");
        assert_eq!(artifact.filename, "app-1.0.0.dmg");
        assert_eq!(artifact.filetype, Some("dmg".to_string()));
        assert_eq!(artifact.filesize, Some(10485760));
        assert_eq!(artifact.platform, Some("darwin".to_string()));
        assert_eq!(artifact.arch, Some("amd64".to_string()));
        assert_eq!(artifact.status, ArtifactStatus::Uploaded);
        assert_eq!(artifact.release_id, Some("test-release-id".to_string()));
        assert_eq!(artifact.account_id, Some("test-account-id".to_string()));
    }

    #[test]
    fn test_artifact_status_serialization() {
        assert_eq!(
            serde_json::to_string(&ArtifactStatus::Waiting).unwrap(),
            "\"WAITING\""
        );
        assert_eq!(
            serde_json::to_string(&ArtifactStatus::Uploaded).unwrap(),
            "\"UPLOADED\""
        );
        assert_eq!(
            serde_json::to_string(&ArtifactStatus::Failed).unwrap(),
            "\"FAILED\""
        );
        assert_eq!(
            serde_json::to_string(&ArtifactStatus::Yanked).unwrap(),
            "\"YANKED\""
        );
    }

    // Reproduce with calculateChecksum/calculateSignature from keygen-cli cmd/upload.go
    // (commit 3fa3a56), curve25519-voi v0.0.0-20220328075252-7dd334e3daae:
    // ed25519.NewKeyFromSeed(0x01..=0x20), SHA-512 of PAYLOAD, Ed25519ph with
    // Context = CONTEXT, base64.RawStdEncoding. Same vectors as in tests/release_api.rs.
    const PAYLOAD: &[u8] = b"hello keygen";
    const CONTEXT: &str = "test-product-id";
    const PUBLIC_KEY: &str = "79b5562e8fe654f94078b112e8a98ba7901f853ae695bed7e0e3910bad049664";
    const CHECKSUM: &str =
        "dnEi9Lf9kMb2ntwoiG5vSE5tXvJrGo0rLYTMBMcwLP5/TnwwbLgfzp8RuVIRTawUEiuwblnd6laUR0Zzt1BytA";
    const SIGNATURE: &str =
        "YSpVi2Y333gwAyaSGZC67UDdslK8WjbH/QZ1DINDRMseUsNYd6ML++nPUgZqE+q3H1xaQ9y1OdegGH7jzFkPDg";

    fn artifact(checksum: Option<&str>, signature: Option<&str>) -> Artifact {
        Artifact {
            id: "artifact-1".to_string(),
            filename: "app".to_string(),
            filetype: None,
            filesize: Some(PAYLOAD.len() as u64),
            platform: None,
            arch: None,
            signature: signature.map(str::to_string),
            checksum: checksum.map(str::to_string),
            status: ArtifactStatus::Uploaded,
            metadata: None,
            created: "2024-01-01T00:00:00Z".to_string(),
            updated: "2024-01-01T00:00:00Z".to_string(),
            yanked_at: None,
            release_id: None,
            product_id: None,
            account_id: None,
        }
    }

    fn verify(
        artifact: &Artifact,
        data: &[u8],
        public_key: Option<&str>,
        context: &str,
    ) -> Result<(), Error> {
        let mut verifier = ArtifactVerifier::new(artifact, public_key, context)?;
        verifier.update(data);
        verifier.finish()
    }

    #[test]
    fn test_verifier_accepts_go_signed_artifact() {
        verify(
            &artifact(Some(CHECKSUM), Some(SIGNATURE)),
            PAYLOAD,
            Some(PUBLIC_KEY),
            CONTEXT,
        )
        .expect("artifact signed by keygen upload should verify");
    }

    #[test]
    fn test_verifier_accepts_chunked_input_and_padded_base64() {
        let artifact = artifact(
            Some(&format!("{CHECKSUM}==")),
            Some(&format!("{SIGNATURE}==")),
        );
        let mut verifier = ArtifactVerifier::new(&artifact, Some(PUBLIC_KEY), CONTEXT).unwrap();
        for chunk in PAYLOAD.chunks(5) {
            verifier.update(chunk);
        }
        verifier.finish().expect("chunked input should verify");
    }

    #[test]
    fn test_verifier_rejects_checksum_mismatch() {
        let err = verify(
            &artifact(Some(CHECKSUM), None),
            b"hello keygen!",
            None,
            CONTEXT,
        )
        .expect_err("modified payload should fail");
        assert!(matches!(err, Error::ArtifactChecksumMismatch));

        // SHA-256 of PAYLOAD, as `keygen upload --checksum-algorithm sha-256` writes it
        let err = verify(
            &artifact(Some("D9awDuPGxPqXIvc1yl4wSrtl7ge2Epri0MgpMf3lds8"), None),
            PAYLOAD,
            None,
            CONTEXT,
        )
        .expect_err("checksum of another algorithm should fail");
        assert!(matches!(err, Error::ArtifactChecksumMismatch));
    }

    #[test]
    fn test_verifier_rejects_invalid_signature() {
        let other_key = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
        let cases = [(PUBLIC_KEY, "other-product-id"), (other_key, CONTEXT)];
        for (public_key, context) in cases {
            let err = verify(
                &artifact(Some(CHECKSUM), Some(SIGNATURE)),
                PAYLOAD,
                Some(public_key),
                context,
            )
            .expect_err("wrong key or context should fail");
            assert!(matches!(err, Error::ArtifactSignatureInvalid(_)));
        }
    }

    #[test]
    fn test_verifier_requires_checksum_without_public_key() {
        let err = verify(&artifact(None, Some(SIGNATURE)), PAYLOAD, None, CONTEXT).err();
        assert!(matches!(err, Some(Error::ArtifactChecksumMissing)));
    }

    #[test]
    fn test_verifier_requires_signature_with_public_key() {
        for checksum in [Some(CHECKSUM), None] {
            let err = verify(
                &artifact(checksum, None),
                PAYLOAD,
                Some(PUBLIC_KEY),
                CONTEXT,
            )
            .err();
            assert!(matches!(err, Some(Error::ArtifactSignatureMissing)));
        }
    }

    #[test]
    fn test_verifier_signature_alone_is_enough() {
        verify(
            &artifact(None, Some(SIGNATURE)),
            PAYLOAD,
            Some(PUBLIC_KEY),
            CONTEXT,
        )
        .expect("valid signature should verify without a checksum");

        let err = verify(
            &artifact(None, Some(SIGNATURE)),
            b"hello keygen!",
            Some(PUBLIC_KEY),
            CONTEXT,
        )
        .expect_err("modified payload should fail");
        assert!(matches!(err, Error::ArtifactSignatureInvalid(_)));
    }

    #[test]
    fn test_verifier_checks_present_checksum_even_with_valid_signature() {
        let wrong_checksum = ARTIFACT_BASE64.encode(Sha512::digest(b"something else"));
        let err = verify(
            &artifact(Some(&wrong_checksum), Some(SIGNATURE)),
            PAYLOAD,
            Some(PUBLIC_KEY),
            CONTEXT,
        )
        .expect_err("inconsistent checksum should fail");
        assert!(matches!(err, Error::ArtifactChecksumMismatch));
    }

    #[test]
    fn test_verifier_rejects_malformed_public_key() {
        for public_key in ["not-hex", "79b5562e"] {
            let err = verify(
                &artifact(Some(CHECKSUM), Some(SIGNATURE)),
                PAYLOAD,
                Some(public_key),
                CONTEXT,
            )
            .err();
            assert!(matches!(err, Some(Error::ArtifactPublicKeyInvalid(_))));
        }
    }

    #[test]
    fn test_verifier_rejects_malformed_signature_and_checksum() {
        let short_signature = ARTIFACT_BASE64.encode([0u8; 32]);
        for signature in ["not-base64!", short_signature.as_str()] {
            let err = verify(
                &artifact(Some(CHECKSUM), Some(signature)),
                PAYLOAD,
                Some(PUBLIC_KEY),
                CONTEXT,
            )
            .err();
            assert!(matches!(err, Some(Error::ArtifactSignatureInvalid(_))));
        }

        let err = verify(&artifact(Some("not-base64!"), None), PAYLOAD, None, CONTEXT).err();
        assert!(matches!(err, Some(Error::ArtifactChecksumMismatch)));
    }
}
