//! License management and validation.
//!
//! This module provides functionality for validating, verifying, and managing licenses.
//! It supports both online validation against the Keygen API and offline verification
//! of signed license keys.

use std::env;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;

use crate::certificate::CertificateFileResponse;
use crate::client::{Client, ClientOptions};
use crate::component::Component;
use crate::config::{get_config, KeygenConfig};
use crate::entitlement::{Entitlement, EntitlementsResponse};
use crate::errors::Error;
use crate::insert_optional;
use crate::keygen_client::KeygenClient;
use crate::license_file::LicenseFile;
use crate::machine::{Machine, MachineResponse, MachinesResponse};
#[cfg(feature = "token")]
use crate::token::{Token, TokenResponse};
#[cfg(feature = "token")]
use crate::user::{User, UserAttributes};
use crate::verifier::Verifier;
use crate::KeygenResponseData;
use std::sync::Arc;

/// Represents an optional field update in API requests.
///
/// This enum provides explicit semantics for nullable field updates:
/// - `Keep`: Do not include this field in the update (no change)
/// - `Clear`: Set the field to null/None
/// - `Set(T)`: Set the field to a specific value
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum UpdateField<T> {
    /// Do not update this field
    #[default]
    Keep,
    /// Clear this field (set to null)
    Clear,
    /// Set this field to a specific value
    Set(T),
}

impl<T: Serialize> UpdateField<T> {
    /// Applies this update field to a JSON map if it should be included
    pub fn apply_to(&self, map: &mut serde_json::Map<String, Value>, key: &str) {
        match self {
            UpdateField::Keep => {}
            UpdateField::Clear => {
                map.insert(key.to_string(), Value::Null);
            }
            UpdateField::Set(value) => {
                map.insert(key.to_string(), json!(value));
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SchemeCode {
    #[serde(rename = "ED25519_SIGN")]
    Ed25519Sign,
    #[serde(rename = "ECDSA_P256_SIGN")]
    EcdsaP256Sign,
    #[serde(rename = "RSA_2048_PKCS1_PSS_SIGN_V2")]
    Rsa2048Pkcs1PssSignV2,
    #[serde(rename = "RSA_2048_PKCS1_SIGN_V2")]
    Rsa2048Pkcs1SignV2,
    #[serde(rename = "RSA_2048_PKCS1_ENCRYPT")]
    Rsa2048Pkcs1Encrypt,
    #[serde(rename = "RSA_2048_JWT_RS256")]
    Rsa2048JwtRs256,
    #[serde(rename = "LEGACY_ENCRYPT")]
    LegacyEncrypt,
    #[serde(rename = "RSA_2048_PKCS1_PSS_SIGN")]
    Rsa2048Pkcs1PssSign, // Deprecated
    #[serde(rename = "RSA_2048_PKCS1_SIGN")]
    Rsa2048Pkcs1Sign, // Deprecated
}

/// License status as returned by the Keygen API
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LicenseStatus {
    Active,
    Inactive,
    Expiring,
    Expired,
    Suspended,
    Banned,
}

impl LicenseStatus {
    /// Parses a LicenseStatus from a string, returning None for unknown values
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "ACTIVE" => Some(Self::Active),
            "INACTIVE" => Some(Self::Inactive),
            "EXPIRING" => Some(Self::Expiring),
            "EXPIRED" => Some(Self::Expired),
            "SUSPENDED" => Some(Self::Suspended),
            "BANNED" => Some(Self::Banned),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LicenseResponse<M> {
    pub meta: Option<M>,
    pub data: KeygenResponseData<LicenseAttributes>,
}

#[cfg(feature = "token")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LicenseUsersResponse {
    pub data: Vec<KeygenResponseData<UserAttributes>>,
    pub meta: Option<Value>,
    pub links: Option<Value>,
}

#[cfg(feature = "token")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LicenseAttachmentAttributes {
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
}

#[cfg(feature = "token")]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LicenseAttachmentResource {
    id: String,
    #[serde(rename = "type")]
    resource_type: String,
    attributes: LicenseAttachmentAttributes,
    #[serde(default)]
    relationships: Value,
    #[serde(default)]
    links: Value,
}

#[cfg(feature = "token")]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LicenseAttachmentsResponse {
    data: Vec<LicenseAttachmentResource>,
}

#[cfg(feature = "token")]
fn attachment_relationship_id(relationships: &Value, name: &str) -> Option<String> {
    relationships
        .get(name)?
        .get("data")?
        .get("id")?
        .as_str()
        .map(str::to_owned)
}

#[cfg(feature = "token")]
fn attachment_related_link(links: &Value) -> Option<String> {
    links.get("related")?.as_str().map(str::to_owned)
}

/// A relationship resource created when attaching a user to a license.
#[cfg(feature = "token")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicenseUserAttachment {
    pub id: String,
    pub resource_type: String,
    pub account_id: Option<String>,
    pub license_id: Option<String>,
    pub user_id: Option<String>,
    pub related: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[cfg(feature = "token")]
impl From<LicenseAttachmentResource> for LicenseUserAttachment {
    fn from(resource: LicenseAttachmentResource) -> Self {
        Self {
            id: resource.id,
            resource_type: resource.resource_type,
            account_id: attachment_relationship_id(&resource.relationships, "account"),
            license_id: attachment_relationship_id(&resource.relationships, "license"),
            user_id: attachment_relationship_id(&resource.relationships, "user"),
            related: attachment_related_link(&resource.links),
            created: resource.attributes.created,
            updated: resource.attributes.updated,
        }
    }
}

/// A relationship resource created when attaching an entitlement to a license.
#[cfg(feature = "token")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicenseEntitlementAttachment {
    pub id: String,
    pub resource_type: String,
    pub account_id: Option<String>,
    pub license_id: Option<String>,
    pub entitlement_id: Option<String>,
    pub related: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[cfg(feature = "token")]
impl From<LicenseAttachmentResource> for LicenseEntitlementAttachment {
    fn from(resource: LicenseAttachmentResource) -> Self {
        Self {
            id: resource.id,
            resource_type: resource.resource_type,
            account_id: attachment_relationship_id(&resource.relationships, "account"),
            license_id: attachment_relationship_id(&resource.relationships, "license"),
            entitlement_id: attachment_relationship_id(&resource.relationships, "entitlement"),
            related: attachment_related_link(&resource.links),
            created: resource.attributes.created,
            updated: resource.attributes.updated,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseValidationMeta {
    pub ts: DateTime<Utc>,
    pub valid: bool,
    pub detail: String,
    pub code: String,
    pub scope: LicenseValidationScope,
    #[serde(default)]
    pub nonce: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LicenseValidationCode {
    Valid,
    NotFound,
    Suspended,
    Expired,
    Overdue,
    HeartbeatDead,
    HeartbeatNotStarted,
    FingerprintScopeRequired,
    FingerprintScopeEmpty,
    FingerprintScopeMismatch,
    ComponentsScopeRequired,
    ComponentsScopeMismatch,
    MachineScopeRequired,
    MachineScopeMismatch,
    NoMachine,
    NoMachines,
    UserScopeRequired,
    UserScopeMismatch,
    ProductScopeRequired,
    ProductScopeMismatch,
    PolicyScopeRequired,
    PolicyScopeMismatch,
    EntitlementsScopeEmpty,
    EntitlementsMissing,
    ChecksumScopeRequired,
    ChecksumScopeMismatch,
    VersionScopeRequired,
    VersionScopeMismatch,
    TooManyMachines,
    TooManyCores,
    TooManyProcesses,
    TooManyUsers,
    TooMuchMemory,
    TooMuchDisk,
    Unknown,
}

impl LicenseValidationCode {
    pub fn parse(code: &str) -> Self {
        match code {
            "VALID" => Self::Valid,
            "NOT_FOUND" => Self::NotFound,
            "SUSPENDED" => Self::Suspended,
            "EXPIRED" => Self::Expired,
            "OVERDUE" => Self::Overdue,
            "HEARTBEAT_DEAD" => Self::HeartbeatDead,
            "HEARTBEAT_NOT_STARTED" => Self::HeartbeatNotStarted,
            "FINGERPRINT_SCOPE_REQUIRED" => Self::FingerprintScopeRequired,
            "FINGERPRINT_SCOPE_EMPTY" => Self::FingerprintScopeEmpty,
            "FINGERPRINT_SCOPE_MISMATCH" => Self::FingerprintScopeMismatch,
            "COMPONENTS_SCOPE_REQUIRED" => Self::ComponentsScopeRequired,
            "COMPONENTS_SCOPE_MISMATCH" => Self::ComponentsScopeMismatch,
            "MACHINE_SCOPE_REQUIRED" => Self::MachineScopeRequired,
            "MACHINE_SCOPE_MISMATCH" => Self::MachineScopeMismatch,
            "NO_MACHINE" => Self::NoMachine,
            "NO_MACHINES" => Self::NoMachines,
            "USER_SCOPE_REQUIRED" => Self::UserScopeRequired,
            "USER_SCOPE_MISMATCH" => Self::UserScopeMismatch,
            "PRODUCT_SCOPE_REQUIRED" => Self::ProductScopeRequired,
            "PRODUCT_SCOPE_MISMATCH" => Self::ProductScopeMismatch,
            "POLICY_SCOPE_REQUIRED" => Self::PolicyScopeRequired,
            "POLICY_SCOPE_MISMATCH" => Self::PolicyScopeMismatch,
            "ENTITLEMENTS_SCOPE_EMPTY" => Self::EntitlementsScopeEmpty,
            "ENTITLEMENTS_MISSING" => Self::EntitlementsMissing,
            "CHECKSUM_SCOPE_REQUIRED" => Self::ChecksumScopeRequired,
            "CHECKSUM_SCOPE_MISMATCH" => Self::ChecksumScopeMismatch,
            "VERSION_SCOPE_REQUIRED" => Self::VersionScopeRequired,
            "VERSION_SCOPE_MISMATCH" => Self::VersionScopeMismatch,
            "TOO_MANY_MACHINES" => Self::TooManyMachines,
            "TOO_MANY_CORES" => Self::TooManyCores,
            "TOO_MANY_PROCESSES" => Self::TooManyProcesses,
            "TOO_MANY_USERS" => Self::TooManyUsers,
            "TOO_MUCH_MEMORY" => Self::TooMuchMemory,
            "TOO_MUCH_DISK" => Self::TooMuchDisk,
            _ => Self::Unknown,
        }
    }
}

impl LicenseValidationMeta {
    pub fn code_kind(&self) -> LicenseValidationCode {
        LicenseValidationCode::parse(&self.code)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseValidationScope {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprints: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entitlements: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LicenseValidationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<i64>,
    pub scope: LicenseValidationScope,
}

impl LicenseValidationRequest {
    pub fn new(scope: LicenseValidationScope) -> Self {
        Self { nonce: None, scope }
    }

    pub fn for_fingerprint(fingerprint: String) -> Self {
        Self::new(LicenseValidationScope {
            fingerprint: Some(fingerprint),
            ..Default::default()
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseValidationResult {
    pub license: Option<License>,
    pub meta: LicenseValidationMeta,
}

impl LicenseValidationResult {
    pub fn into_license(self) -> Result<License, Error> {
        if !self.meta.valid {
            return Err(Error::LicenseKeyInvalid {
                code: self.meta.code,
                detail: self.meta.detail,
            });
        }
        self.license.ok_or(Error::LicenseKeyInvalid {
            code: self.meta.code,
            detail: self.meta.detail,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LicenseValidationResponse {
    pub meta: LicenseValidationMeta,
    pub data: Option<KeygenResponseData<LicenseAttributes>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct LicenseAttributes {
    pub key: String,
    pub name: Option<String>,
    pub expiry: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub uses: Option<i32>,
    pub protected: Option<bool>,
    pub version: Option<String>,
    pub suspended: Option<bool>,
    pub floating: Option<bool>,
    pub encrypted: Option<bool>,
    pub scheme: Option<String>,
    pub strict: Option<bool>,
    #[serde(rename = "maxMachines")]
    pub max_machines: Option<i32>,
    #[serde(rename = "maxCores")]
    pub max_cores: Option<i32>,
    #[serde(rename = "maxUses")]
    pub max_uses: Option<i32>,
    #[serde(rename = "maxProcesses")]
    pub max_processes: Option<i32>,
    #[serde(rename = "maxUsers")]
    pub max_users: Option<i32>,
    #[serde(rename = "maxMemory")]
    pub max_memory: Option<i64>,
    #[serde(rename = "maxDisk")]
    pub max_disk: Option<i64>,
    #[serde(rename = "requireHeartbeat")]
    pub require_heartbeat: Option<bool>,
    #[serde(rename = "requireCheckIn")]
    pub require_check_in: Option<bool>,
    #[serde(rename = "lastValidated")]
    pub last_validated: Option<DateTime<Utc>>,
    #[serde(rename = "lastCheckOut")]
    pub last_check_out: Option<DateTime<Utc>>,
    #[serde(rename = "lastCheckIn")]
    pub last_check_in: Option<DateTime<Utc>>,
    #[serde(rename = "nextCheckIn")]
    pub next_check_in: Option<DateTime<Utc>>,
    pub permissions: Option<Vec<String>>,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    #[serde(default)]
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub id: String,
    #[serde(skip_serializing)]
    pub scheme: Option<SchemeCode>,
    pub key: String,
    pub name: Option<String>,
    pub expiry: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub uses: Option<i32>,
    pub version: Option<String>,
    pub floating: Option<bool>,
    pub encrypted: Option<bool>,
    pub strict: Option<bool>,
    pub max_machines: Option<i32>,
    pub max_cores: Option<i32>,
    pub max_uses: Option<i32>,
    pub max_processes: Option<i32>,
    pub max_users: Option<i32>,
    pub max_memory: Option<i64>,
    pub max_disk: Option<i64>,
    pub protected: Option<bool>,
    pub suspended: Option<bool>,
    pub require_heartbeat: Option<bool>,
    pub require_check_in: Option<bool>,
    pub last_validated: Option<DateTime<Utc>>,
    pub last_check_out: Option<DateTime<Utc>>,
    pub last_check_in: Option<DateTime<Utc>>,
    pub next_check_in: Option<DateTime<Utc>>,
    pub permissions: Option<Vec<String>>,
    pub policy: Option<String>,
    pub metadata: HashMap<String, Value>,
    pub account_id: Option<String>,
    pub product_id: Option<String>,
    pub group_id: Option<String>,
    pub owner_id: Option<String>,
    pub environment_id: Option<String>,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub config: Option<Arc<KeygenConfig>>,
    #[serde(skip)]
    client: Option<Arc<Client>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseFileAlgorithm {
    #[serde(rename = "aes-256-gcm+ed25519")]
    Aes256GcmEd25519,
    #[serde(rename = "aes-256-gcm+ecdsa-p256")]
    Aes256GcmEcdsaP256,
    #[serde(rename = "aes-256-gcm+rsa-pss-sha256")]
    Aes256GcmRsaPssSha256,
    #[serde(rename = "aes-256-gcm+rsa-sha256")]
    Aes256GcmRsaSha256,
    #[serde(rename = "base64+ed25519")]
    Base64Ed25519,
    #[serde(rename = "base64+ecdsa-p256")]
    Base64EcdsaP256,
    #[serde(rename = "base64+rsa-pss-sha256")]
    Base64RsaPssSha256,
    #[serde(rename = "base64+rsa-sha256")]
    Base64RsaSha256,
}

impl LicenseFileAlgorithm {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Aes256GcmEd25519 => "aes-256-gcm+ed25519",
            Self::Aes256GcmEcdsaP256 => "aes-256-gcm+ecdsa-p256",
            Self::Aes256GcmRsaPssSha256 => "aes-256-gcm+rsa-pss-sha256",
            Self::Aes256GcmRsaSha256 => "aes-256-gcm+rsa-sha256",
            Self::Base64Ed25519 => "base64+ed25519",
            Self::Base64EcdsaP256 => "base64+ecdsa-p256",
            Self::Base64RsaPssSha256 => "base64+rsa-pss-sha256",
            Self::Base64RsaSha256 => "base64+rsa-sha256",
        }
    }

    pub const fn is_encrypted(self) -> bool {
        matches!(
            self,
            Self::Aes256GcmEd25519
                | Self::Aes256GcmEcdsaP256
                | Self::Aes256GcmRsaPssSha256
                | Self::Aes256GcmRsaSha256
        )
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "aes-256-gcm+ed25519" => Some(Self::Aes256GcmEd25519),
            "aes-256-gcm+ecdsa-p256" => Some(Self::Aes256GcmEcdsaP256),
            "aes-256-gcm+rsa-pss-sha256" => Some(Self::Aes256GcmRsaPssSha256),
            "aes-256-gcm+rsa-sha256" => Some(Self::Aes256GcmRsaSha256),
            "base64+ed25519" => Some(Self::Base64Ed25519),
            "base64+ecdsa-p256" => Some(Self::Base64EcdsaP256),
            "base64+rsa-pss-sha256" => Some(Self::Base64RsaPssSha256),
            "base64+rsa-sha256" => Some(Self::Base64RsaSha256),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LicenseCheckoutOpts {
    pub ttl: UpdateField<i64>,
    pub include: Option<Vec<String>>,
    pub encrypt: Option<bool>,
    pub algorithm: Option<LicenseFileAlgorithm>,
}

impl LicenseCheckoutOpts {
    /// Create new checkout options with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create checkout options with TTL
    pub fn with_ttl(ttl: i64) -> Self {
        Self {
            ttl: UpdateField::Set(ttl),
            ..Self::default()
        }
    }

    pub fn perpetual() -> Self {
        Self {
            ttl: UpdateField::Clear,
            ..Self::default()
        }
    }

    /// Create checkout options with specific relationships to include
    pub fn with_include(include: Vec<String>) -> Self {
        Self {
            include: Some(include),
            ..Self::default()
        }
    }

    pub fn with_encrypt(mut self, encrypt: bool) -> Self {
        self.encrypt = Some(encrypt);
        self
    }

    pub fn with_algorithm(mut self, algorithm: LicenseFileAlgorithm) -> Self {
        self.algorithm = Some(algorithm);
        self
    }

    fn to_query(&self) -> Result<Value, Error> {
        if self.encrypt.is_some() && self.algorithm.is_some() {
            return Err(Error::UnexpectedError(
                "checkout encrypt and algorithm are mutually exclusive".to_string(),
            ));
        }

        let mut query = json!({});
        match &self.ttl {
            UpdateField::Keep => {}
            UpdateField::Clear => query["ttl"] = json!("null"),
            UpdateField::Set(ttl) => query["ttl"] = json!(ttl),
        }
        if let Some(include) = &self.include {
            query["include"] = json!(include.join(","));
        }
        if let Some(encrypt) = self.encrypt {
            query["encrypt"] = json!(encrypt);
        }
        if let Some(algorithm) = &self.algorithm {
            query["algorithm"] = serde_json::to_value(algorithm)?;
        }
        Ok(query)
    }
}

#[derive(Debug, Default, Serialize)]
pub struct PaginationOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(rename = "page[size]", skip_serializing_if = "Option::is_none")]
    pub page_size: Option<i32>,
    #[serde(rename = "page[cursor]", skip_serializing_if = "Option::is_none")]
    pub page_cursor: Option<String>,
}

fn cursor_pagination_query(
    limit: Option<i32>,
    page_size: Option<i32>,
    page_cursor: Option<&str>,
    default_limit: Option<i32>,
) -> Value {
    let mut query = json!({});
    if let Some(limit) = limit.or(default_limit) {
        query["limit"] = json!(limit);
    }
    if page_size.is_some() || page_cursor.is_some() {
        query["page[cursor]"] = json!(page_cursor.unwrap_or_default());
    }
    if let Some(page_size) = page_size {
        query["page[size]"] = json!(page_size);
    }
    query
}

#[derive(Debug, Clone, Default)]
pub struct NumericFilter {
    pub eq: Option<i32>,
    pub gt: Option<i32>,
    pub gte: Option<i32>,
    pub lt: Option<i32>,
    pub lte: Option<i32>,
}

#[derive(Debug, Clone, Default)]
pub struct DateWindowFilter {
    pub r#in: Option<String>,
    pub on: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct LicenseActivityFilter {
    pub inside: Option<String>,
    pub outside: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
}

/// Filters and cursor pagination for listing licenses.
#[derive(Debug, Default)]
pub struct LicenseListOptions {
    pub limit: Option<i32>, // Number of resources to return (1-100, default 10)
    pub page_size: Option<i32>, // Number of resources per page (1-100)
    pub page_cursor: Option<String>,

    // Common filters
    pub status: Option<String>,  // "ACTIVE", "EXPIRED", "SUSPENDED", etc.
    pub product: Option<String>, // Product ID
    pub policy: Option<String>,  // Policy ID
    pub owner: Option<String>,   // Owner ID or email
    pub user: Option<String>,    // User ID or email
    pub group: Option<String>,
    pub machine: Option<String>,
    pub assigned: Option<bool>,
    pub unassigned: Option<bool>,
    pub activated: Option<bool>,
    pub metadata: Option<HashMap<String, Value>>,
    pub activations: Option<NumericFilter>,
    pub expires: Option<DateWindowFilter>,
    pub expired: Option<DateWindowFilter>,
    pub activity: Option<LicenseActivityFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePage<T> {
    pub data: Vec<T>,
    pub meta: Option<Value>,
    pub links: Option<Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PageLinks {
    #[serde(rename = "self", default, skip_serializing_if = "Option::is_none")]
    self_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    next: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    previous: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    first: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last: Option<String>,
    #[serde(flatten)]
    extensions: Map<String, Value>,
}

impl PageLinks {
    pub fn self_url(&self) -> Option<&str> {
        self.self_url.as_deref()
    }

    pub fn next_url(&self) -> Option<&str> {
        self.next.as_deref()
    }

    pub fn previous_url(&self) -> Option<&str> {
        self.previous.as_deref()
    }

    pub fn first_url(&self) -> Option<&str> {
        self.first.as_deref()
    }

    pub fn last_url(&self) -> Option<&str> {
        self.last.as_deref()
    }

    pub fn extensions(&self) -> &Map<String, Value> {
        &self.extensions
    }

    pub fn has_next_page(&self) -> bool {
        self.next.is_some()
    }

    pub fn next_cursor(&self) -> Option<String> {
        let query = self.next.as_deref()?.split_once('?')?.1;
        url::form_urlencoded::parse(query.as_bytes())
            .find_map(|(key, value)| (key == "page[cursor]").then(|| value.into_owned()))
    }
}

impl<T> ResourcePage<T> {
    pub fn typed_links(&self) -> Result<Option<PageLinks>, serde_json::Error> {
        self.links.clone().map(serde_json::from_value).transpose()
    }

    pub fn next_cursor(&self) -> Option<String> {
        self.typed_links().ok().flatten()?.next_cursor()
    }
}

pub type LicensePage = ResourcePage<License>;

/// Attributes and relationships accepted when creating a license.
#[derive(Debug, Default)]
pub struct LicenseCreateRequest {
    // Required relationship
    pub policy_id: String,

    // Optional attributes
    pub name: Option<String>,
    pub key: Option<String>,
    pub expiry: Option<DateTime<Utc>>,
    pub max_machines: Option<i32>,
    pub max_processes: Option<i32>,
    pub max_users: Option<i32>,
    pub max_cores: Option<i32>,
    pub max_uses: Option<i32>,
    pub max_memory: Option<i64>,
    pub max_disk: Option<i64>,
    pub protected: Option<bool>,
    pub suspended: Option<bool>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, Value>>,

    // Optional relationships
    pub owner_id: Option<String>, // User ID
    pub group_id: Option<String>, // Group ID
}

/// Mutable license attributes.
#[derive(Debug, Default)]
pub struct LicenseUpdateRequest {
    // All optional attributes that can be updated
    pub name: UpdateField<String>,
    pub expiry: UpdateField<DateTime<Utc>>,
    pub max_machines: UpdateField<i32>,
    pub max_processes: UpdateField<i32>,
    pub max_users: UpdateField<i32>,
    pub max_cores: UpdateField<i32>,
    pub max_uses: UpdateField<i32>,
    pub max_memory: UpdateField<i64>,
    pub max_disk: UpdateField<i64>,
    pub protected: Option<bool>,
    pub suspended: Option<bool>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, Value>>,
}

#[cfg(feature = "token")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseTokenCreateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_activations: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_deactivations: Option<u32>,
}

impl LicenseCreateRequest {
    /// Create a new license creation request with the required policy ID
    pub fn new(policy_id: String) -> Self {
        Self {
            policy_id,
            ..Default::default()
        }
    }

    /// Set the license name
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Set a custom license key
    pub fn with_key(mut self, key: String) -> Self {
        self.key = Some(key);
        self
    }

    /// Set the expiry date
    pub fn with_expiry(mut self, expiry: DateTime<Utc>) -> Self {
        self.expiry = Some(expiry);
        self
    }

    /// Set the maximum number of machines
    pub fn with_max_machines(mut self, max_machines: i32) -> Self {
        self.max_machines = Some(max_machines);
        self
    }

    /// Set the maximum number of processes
    pub fn with_max_processes(mut self, max_processes: i32) -> Self {
        self.max_processes = Some(max_processes);
        self
    }

    /// Set the maximum number of users
    pub fn with_max_users(mut self, max_users: i32) -> Self {
        self.max_users = Some(max_users);
        self
    }

    /// Set the maximum number of cores
    pub fn with_max_cores(mut self, max_cores: i32) -> Self {
        self.max_cores = Some(max_cores);
        self
    }

    /// Set the maximum number of uses
    pub fn with_max_uses(mut self, max_uses: i32) -> Self {
        self.max_uses = Some(max_uses);
        self
    }

    pub fn with_max_memory(mut self, max_memory: i64) -> Self {
        self.max_memory = Some(max_memory);
        self
    }

    pub fn with_max_disk(mut self, max_disk: i64) -> Self {
        self.max_disk = Some(max_disk);
        self
    }

    /// Set the protected flag
    pub fn with_protected(mut self, protected: bool) -> Self {
        self.protected = Some(protected);
        self
    }

    /// Set the suspended flag
    pub fn with_suspended(mut self, suspended: bool) -> Self {
        self.suspended = Some(suspended);
        self
    }

    /// Set the permissions array
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = Some(permissions);
        self
    }

    /// Set the metadata
    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set the owner (user) ID
    pub fn with_owner_id(mut self, owner_id: String) -> Self {
        self.owner_id = Some(owner_id);
        self
    }

    /// Set the group ID
    pub fn with_group_id(mut self, group_id: String) -> Self {
        self.group_id = Some(group_id);
        self
    }

    /// Convert this request to attributes and relationships JSON maps for the API
    pub fn to_json_body(self) -> Value {
        let mut attributes = serde_json::Map::new();
        let mut relationships = serde_json::Map::new();

        // Build attributes — all types here (String, i32, DateTime, bool, Vec, HashMap)
        // are infallibly serializable, so unwrap is safe.
        let _ = insert_optional(&mut attributes, "name", self.name);
        let _ = insert_optional(&mut attributes, "key", self.key);
        let _ = insert_optional(&mut attributes, "expiry", self.expiry);
        let _ = insert_optional(&mut attributes, "maxMachines", self.max_machines);
        let _ = insert_optional(&mut attributes, "maxProcesses", self.max_processes);
        let _ = insert_optional(&mut attributes, "maxUsers", self.max_users);
        let _ = insert_optional(&mut attributes, "maxCores", self.max_cores);
        let _ = insert_optional(&mut attributes, "maxUses", self.max_uses);
        let _ = insert_optional(&mut attributes, "maxMemory", self.max_memory);
        let _ = insert_optional(&mut attributes, "maxDisk", self.max_disk);
        let _ = insert_optional(&mut attributes, "protected", self.protected);
        let _ = insert_optional(&mut attributes, "suspended", self.suspended);
        let _ = insert_optional(&mut attributes, "permissions", self.permissions);
        let _ = insert_optional(&mut attributes, "metadata", self.metadata);

        // Build relationships - policy is required
        relationships.insert(
            "policy".to_string(),
            json!({
                "data": {
                    "type": "policies",
                    "id": self.policy_id
                }
            }),
        );

        if let Some(owner_id) = self.owner_id {
            relationships.insert(
                "owner".to_string(),
                json!({
                    "data": {
                        "type": "users",
                        "id": owner_id
                    }
                }),
            );
        }

        if let Some(group_id) = self.group_id {
            relationships.insert(
                "group".to_string(),
                json!({
                    "data": {
                        "type": "groups",
                        "id": group_id
                    }
                }),
            );
        }

        json!({
            "data": {
                "type": "licenses",
                "attributes": attributes,
                "relationships": relationships
            }
        })
    }
}

impl LicenseUpdateRequest {
    /// Create a new empty license update request
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the license name
    pub fn with_name(mut self, name: String) -> Self {
        self.name = UpdateField::Set(name);
        self
    }

    pub fn clear_name(mut self) -> Self {
        self.name = UpdateField::Clear;
        self
    }

    /// Set the expiry date
    pub fn with_expiry(mut self, expiry: DateTime<Utc>) -> Self {
        self.expiry = UpdateField::Set(expiry);
        self
    }

    pub fn clear_expiry(mut self) -> Self {
        self.expiry = UpdateField::Clear;
        self
    }

    /// Set the maximum number of machines
    pub fn with_max_machines(mut self, max_machines: i32) -> Self {
        self.max_machines = UpdateField::Set(max_machines);
        self
    }

    /// Clear the maximum number of machines (set to null)
    pub fn clear_max_machines(mut self) -> Self {
        self.max_machines = UpdateField::Clear;
        self
    }

    /// Set the maximum number of processes
    pub fn with_max_processes(mut self, max_processes: i32) -> Self {
        self.max_processes = UpdateField::Set(max_processes);
        self
    }

    /// Clear the maximum number of processes (set to null)
    pub fn clear_max_processes(mut self) -> Self {
        self.max_processes = UpdateField::Clear;
        self
    }

    /// Set the maximum number of users
    pub fn with_max_users(mut self, max_users: i32) -> Self {
        self.max_users = UpdateField::Set(max_users);
        self
    }

    /// Clear the maximum number of users (set to null)
    pub fn clear_max_users(mut self) -> Self {
        self.max_users = UpdateField::Clear;
        self
    }

    /// Set the maximum number of cores
    pub fn with_max_cores(mut self, max_cores: i32) -> Self {
        self.max_cores = UpdateField::Set(max_cores);
        self
    }

    /// Clear the maximum number of cores (set to null)
    pub fn clear_max_cores(mut self) -> Self {
        self.max_cores = UpdateField::Clear;
        self
    }

    /// Set the maximum number of uses
    pub fn with_max_uses(mut self, max_uses: i32) -> Self {
        self.max_uses = UpdateField::Set(max_uses);
        self
    }

    /// Clear the maximum number of uses (set to null)
    pub fn clear_max_uses(mut self) -> Self {
        self.max_uses = UpdateField::Clear;
        self
    }

    pub fn with_max_memory(mut self, max_memory: i64) -> Self {
        self.max_memory = UpdateField::Set(max_memory);
        self
    }

    pub fn clear_max_memory(mut self) -> Self {
        self.max_memory = UpdateField::Clear;
        self
    }

    pub fn with_max_disk(mut self, max_disk: i64) -> Self {
        self.max_disk = UpdateField::Set(max_disk);
        self
    }

    pub fn clear_max_disk(mut self) -> Self {
        self.max_disk = UpdateField::Clear;
        self
    }

    /// Set the protected flag
    pub fn with_protected(mut self, protected: bool) -> Self {
        self.protected = Some(protected);
        self
    }

    /// Set the suspended flag
    pub fn with_suspended(mut self, suspended: bool) -> Self {
        self.suspended = Some(suspended);
        self
    }

    /// Set the permissions array
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = Some(permissions);
        self
    }

    /// Set the metadata
    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Convert this request to complete JSON body for the API
    pub fn to_json_body(self) -> Value {
        let mut attributes = serde_json::Map::new();

        self.name.apply_to(&mut attributes, "name");
        self.expiry.apply_to(&mut attributes, "expiry");
        self.max_machines.apply_to(&mut attributes, "maxMachines");
        self.max_processes.apply_to(&mut attributes, "maxProcesses");
        self.max_users.apply_to(&mut attributes, "maxUsers");
        self.max_cores.apply_to(&mut attributes, "maxCores");
        self.max_uses.apply_to(&mut attributes, "maxUses");
        self.max_memory.apply_to(&mut attributes, "maxMemory");
        self.max_disk.apply_to(&mut attributes, "maxDisk");
        if let Some(protected) = self.protected {
            attributes.insert("protected".to_string(), json!(protected));
        }
        if let Some(suspended) = self.suspended {
            attributes.insert("suspended".to_string(), json!(suspended));
        }
        if let Some(permissions) = self.permissions {
            attributes.insert("permissions".to_string(), json!(permissions));
        }
        if let Some(metadata) = self.metadata {
            attributes.insert("metadata".to_string(), json!(metadata));
        }

        json!({
            "data": {
                "type": "licenses",
                "attributes": attributes
            }
        })
    }
}

impl License {
    pub(crate) fn from(data: KeygenResponseData<LicenseAttributes>) -> License {
        let scheme = data
            .attributes
            .scheme
            .as_ref()
            .and_then(|value| serde_json::from_value(Value::String(value.clone())).ok());
        License {
            id: data.id,
            scheme,
            key: data.attributes.key,
            name: data.attributes.name,
            expiry: data.attributes.expiry,
            status: data.attributes.status,
            uses: data.attributes.uses,
            version: data.attributes.version,
            floating: data.attributes.floating,
            encrypted: data.attributes.encrypted,
            strict: data.attributes.strict,
            max_machines: data.attributes.max_machines,
            max_cores: data.attributes.max_cores,
            max_uses: data.attributes.max_uses,
            max_processes: data.attributes.max_processes,
            max_users: data.attributes.max_users,
            max_memory: data.attributes.max_memory,
            max_disk: data.attributes.max_disk,
            protected: data.attributes.protected,
            suspended: data.attributes.suspended,
            require_heartbeat: data.attributes.require_heartbeat,
            require_check_in: data.attributes.require_check_in,
            last_validated: data.attributes.last_validated,
            last_check_out: data.attributes.last_check_out,
            last_check_in: data.attributes.last_check_in,
            next_check_in: data.attributes.next_check_in,
            permissions: data.attributes.permissions,
            policy: data.relationships.policy_id(),
            metadata: data.attributes.metadata,
            account_id: data.relationships.account_id(),
            product_id: data.relationships.product_id(),
            group_id: data.relationships.group_id(),
            owner_id: data.relationships.owner_id(),
            environment_id: data.relationships.environment_id(),
            created: data.attributes.created,
            updated: data.attributes.updated,
            config: None,
            client: None,
        }
    }

    pub(crate) fn from_signed_key(scheme: SchemeCode, signed_key: &str) -> License {
        License {
            id: String::new(),
            scheme: Some(scheme),
            key: signed_key.to_string(),
            name: None,
            expiry: None,
            status: None,
            uses: None,
            version: None,
            floating: None,
            encrypted: None,
            strict: None,
            max_machines: None,
            max_cores: None,
            max_uses: None,
            max_processes: None,
            max_users: None,
            max_memory: None,
            max_disk: None,
            protected: None,
            suspended: None,
            require_heartbeat: None,
            require_check_in: None,
            last_validated: None,
            last_check_out: None,
            last_check_in: None,
            next_check_in: None,
            permissions: None,
            policy: None,
            metadata: HashMap::new(),
            account_id: None,
            product_id: None,
            group_id: None,
            owner_id: None,
            environment_id: None,
            created: None,
            updated: None,
            config: None,
            client: None,
        }
    }

    /// Creates a new License with just the key
    pub fn from_key(key: &str) -> Self {
        License {
            id: String::new(),
            scheme: None,
            key: key.to_string(),
            name: None,
            expiry: None,
            status: None,
            uses: None,
            version: None,
            floating: None,
            encrypted: None,
            strict: None,
            max_machines: None,
            max_cores: None,
            max_uses: None,
            max_processes: None,
            max_users: None,
            max_memory: None,
            max_disk: None,
            protected: None,
            suspended: None,
            require_heartbeat: None,
            require_check_in: None,
            last_validated: None,
            last_check_out: None,
            last_check_in: None,
            next_check_in: None,
            permissions: None,
            policy: None,
            metadata: HashMap::new(),
            account_id: None,
            product_id: None,
            group_id: None,
            owner_id: None,
            environment_id: None,
            created: None,
            updated: None,
            config: None,
            client: None,
        }
    }

    pub fn from_id(id: &str) -> Self {
        let mut license = Self::from_key("");
        license.id = id.to_string();
        license
    }

    /// Associates a configuration with this License
    pub fn with_config(mut self, config: KeygenConfig) -> Self {
        self.config = Some(Arc::new(config));
        self
    }

    pub(crate) fn with_client(mut self, client: Arc<Client>, config: Arc<KeygenConfig>) -> Self {
        self.client = Some(client);
        self.config = Some(config);
        self
    }

    fn inherit_client(&self, license: License) -> License {
        match (&self.client, &self.config) {
            (Some(client), Some(config)) => {
                license.with_client(Arc::clone(client), Arc::clone(config))
            }
            (None, Some(config)) => license.with_config(config.as_ref().clone()),
            _ => license,
        }
    }

    /// Gets a client for this license, using the associated config or global config
    fn get_client(&self) -> Result<Client, Error> {
        if let Some(client) = &self.client {
            return Ok(client.as_ref().clone());
        }
        let config = if let Some(ref cfg) = self.config {
            cfg.as_ref().clone()
        } else {
            get_config()?
        };
        Client::new(ClientOptions::from(config))
    }

    pub async fn validate(
        &self,
        request: &LicenseValidationRequest,
    ) -> Result<LicenseValidationResult, Error> {
        let client = self.get_client()?;
        let params = json!({ "meta": request });

        let response = client
            .post(
                &format!("licenses/{}/actions/validate", self.id),
                Some(&params),
                None::<&()>,
            )
            .await?;
        let validation: LicenseValidationResponse = serde_json::from_value(response.body)?;
        Ok(LicenseValidationResult {
            license: validation
                .data
                .map(|data| self.inherit_client(License::from(data))),
            meta: validation.meta,
        })
    }

    pub async fn validate_key(
        &self,
        request: &LicenseValidationRequest,
    ) -> Result<LicenseValidationResult, Error> {
        let client = self.get_client()?;
        let mut meta = serde_json::to_value(request)?;
        meta["key"] = json!(self.key);
        let params = json!({ "meta": meta });

        let response = client
            .post("licenses/actions/validate-key", Some(&params), None::<&()>)
            .await?;
        let validation: LicenseValidationResponse = serde_json::from_value(response.body)?;
        Ok(LicenseValidationResult {
            license: validation
                .data
                .map(|data| self.inherit_client(License::from(data))),
            meta: validation.meta,
        })
    }

    #[must_use = "verification result should be checked"]
    pub fn verify(&self) -> Result<Vec<u8>, Error> {
        if self.scheme.is_none() {
            return Err(Error::LicenseNotSigned);
        }
        let config = if let Some(ref cfg) = self.config {
            cfg.as_ref().clone()
        } else {
            get_config()?
        };
        if let Some(public_key) = &config.public_key {
            let verifier = Verifier::new(public_key.clone());
            verifier.verify_license(self)
        } else {
            Err(Error::PublicKeyMissing)
        }
    }

    pub async fn activate(
        &self,
        fingerprint: &str,
        components: &[Component],
    ) -> Result<Machine, Error> {
        #[cfg(not(target_arch = "wasm32"))]
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| String::from("unknown"));
        #[cfg(target_arch = "wasm32")]
        let hostname = String::from("wasm");

        let config = if let Some(ref cfg) = self.config {
            cfg.as_ref()
        } else {
            &get_config()?
        };
        let platform = config
            .platform
            .clone()
            .or_else(|| Some(format!("{}/{}", env::consts::OS, env::consts::ARCH)));

        #[cfg(not(target_arch = "wasm32"))]
        let cores = num_cpus::get();
        #[cfg(target_arch = "wasm32")]
        let cores = 0usize;

        let mut params = json!({
          "data": {
            "type": "machines",
            "attributes": {
              "fingerprint": fingerprint,
              "cores": cores,
              "hostname": hostname,
              "platform": platform,
            },
            "relationships": {
              "license": {
                "data": {
                  "type": "licenses",
                  "id": self.id
                }
              },
            }
          }
        });
        if !components.is_empty() {
            params["data"]["relationships"]["components"] = json!({
                "data": components
                    .iter()
                    .map(|comp| json!({
                        "type": "components",
                        "attributes": {
                            "fingerprint": comp.fingerprint,
                            "name": comp.name
                        }
                    }))
                    .collect::<Vec<serde_json::Value>>()
            });
        }

        let client = self.get_client()?;
        let response = client.post("machines", Some(&params), None::<&()>).await?;
        let machine_response: MachineResponse = serde_json::from_value(response.body)?;
        let machine = Machine::from(machine_response.data).with_config(config.clone());
        Ok(machine)
    }

    pub async fn deactivate(&self, id: &str) -> Result<(), Error> {
        let client = self.get_client()?;
        let _response = client
            .delete::<(), serde_json::Value>(&format!("machines/{id}"), None::<&()>)
            .await?;
        Ok(())
    }

    pub async fn machine(&self, id: &str) -> Result<Machine, Error> {
        let client = self.get_client()?;
        let response = client.get(&format!("machines/{id}"), None::<&()>).await?;
        let machine_response: MachineResponse = serde_json::from_value(response.body)?;
        let machine = Machine::from(machine_response.data).with_config(
            self.config
                .as_ref()
                .ok_or(Error::MissingConfiguration)?
                .as_ref()
                .clone(),
        );
        Ok(machine)
    }

    pub async fn machines(
        &self,
        options: Option<&PaginationOptions>,
    ) -> Result<ResourcePage<Machine>, Error> {
        let query = options.map_or_else(
            || cursor_pagination_query(None, None, None, Some(100)),
            |options| {
                cursor_pagination_query(
                    options.limit,
                    options.page_size,
                    options.page_cursor.as_deref(),
                    Some(100),
                )
            },
        );

        let client = self.get_client()?;
        let response = client
            .get(&format!("licenses/{}/machines", self.id), Some(&query))
            .await?;
        let machines_response: MachinesResponse = serde_json::from_value(response.body)?;
        let config = self
            .config
            .as_ref()
            .ok_or(Error::MissingConfiguration)?
            .as_ref()
            .clone();
        let machines = machines_response
            .data
            .iter()
            .map(|d| Machine::from(d.clone()).with_config(config.clone()))
            .collect();
        Ok(ResourcePage {
            data: machines,
            meta: machines_response.meta,
            links: machines_response.links,
        })
    }

    pub async fn entitlements(
        &self,
        options: Option<&PaginationOptions>,
    ) -> Result<ResourcePage<Entitlement>, Error> {
        let query = options.map_or_else(
            || cursor_pagination_query(None, None, None, Some(100)),
            |options| {
                cursor_pagination_query(
                    options.limit,
                    options.page_size,
                    options.page_cursor.as_deref(),
                    Some(100),
                )
            },
        );

        let client = self.get_client()?;
        let response = client
            .get(&format!("licenses/{}/entitlements", self.id), Some(&query))
            .await?;
        let entitlements_response: EntitlementsResponse = serde_json::from_value(response.body)?;
        let entitlements = entitlements_response
            .data
            .iter()
            .map(|d| Entitlement::from(d.clone()))
            .collect();
        Ok(ResourcePage {
            data: entitlements,
            meta: entitlements_response.meta,
            links: entitlements_response.links,
        })
    }

    pub async fn checkout(&self, options: &LicenseCheckoutOpts) -> Result<LicenseFile, Error> {
        let query = options.to_query()?;

        let client = self.get_client()?;
        let response = client
            .post(
                &format!("licenses/{}/actions/check-out", self.id),
                None::<&()>,
                Some(&query),
            )
            .await?;
        let license_file_response: CertificateFileResponse = serde_json::from_value(response.body)?;
        let license_file = LicenseFile::from(license_file_response.data);
        Ok(license_file)
    }

    pub async fn checkout_certificate(
        &self,
        options: &LicenseCheckoutOpts,
    ) -> Result<String, Error> {
        let query = options.to_query()?;
        let client = self.get_client()?;
        let response = client
            .get_text_with_params(
                &format!("licenses/{}/actions/check-out", self.id),
                Some(&query),
            )
            .await?;
        Ok(response.body)
    }

    /// Check a license back in, invalidating any offline license file.
    pub async fn check_in(&self) -> Result<License, Error> {
        let client = self.get_client()?;
        let endpoint = format!("licenses/{}/actions/check-in", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(self.inherit_client(License::from(license_response.data)))
    }

    /// Create a license.
    #[cfg(feature = "token")]
    pub async fn create(request: LicenseCreateRequest) -> Result<License, Error> {
        let config = Arc::new(get_config()?);
        let client = Arc::new(Client::new(ClientOptions::from(config.as_ref().clone()))?);
        Self::create_with_client(client, config, request).await
    }

    #[cfg(feature = "token")]
    pub(crate) async fn create_with_client(
        client: Arc<Client>,
        config: Arc<KeygenConfig>,
        request: LicenseCreateRequest,
    ) -> Result<License, Error> {
        let body = request.to_json_body();
        let response = client.post("licenses", Some(&body), None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(License::from(license_response.data).with_client(client, config))
    }

    /// List all licenses with optional filtering
    #[cfg(feature = "token")]
    pub async fn list(options: Option<&LicenseListOptions>) -> Result<LicensePage, Error> {
        let config = Arc::new(get_config()?);
        let client = Arc::new(Client::new(ClientOptions::from(config.as_ref().clone()))?);
        Self::list_with_client(client, config, options).await
    }

    #[cfg(feature = "token")]
    pub(crate) async fn list_with_client(
        client: Arc<Client>,
        config: Arc<KeygenConfig>,
        options: Option<&LicenseListOptions>,
    ) -> Result<LicensePage, Error> {
        let mut query = options.map_or_else(
            || cursor_pagination_query(None, None, None, None),
            |options| {
                cursor_pagination_query(
                    options.limit,
                    options.page_size,
                    options.page_cursor.as_deref(),
                    None,
                )
            },
        );

        if let Some(opts) = options {
            // Simple filters
            if let Some(ref status) = opts.status {
                query["status"] = json!(status);
            }
            if let Some(ref product) = opts.product {
                query["product"] = json!(product);
            }
            if let Some(ref policy) = opts.policy {
                query["policy"] = json!(policy);
            }
            if let Some(ref owner) = opts.owner {
                query["owner"] = json!(owner);
            }
            if let Some(ref user) = opts.user {
                query["user"] = json!(user);
            }
            if let Some(ref group) = opts.group {
                query["group"] = json!(group);
            }
            if let Some(ref machine) = opts.machine {
                query["machine"] = json!(machine);
            }
            if let Some(assigned) = opts.assigned {
                query["assigned"] = json!(assigned);
            }
            if let Some(unassigned) = opts.unassigned {
                query["unassigned"] = json!(unassigned);
            }
            if let Some(activated) = opts.activated {
                query["activated"] = json!(activated);
            }
            if let Some(ref metadata) = opts.metadata {
                for (key, value) in metadata {
                    if let Some(obj) = query.as_object_mut() {
                        obj.insert(format!("metadata[{key}]"), value.clone());
                    }
                }
            }
            if let Some(ref activations) = opts.activations {
                if let Some(eq) = activations.eq {
                    query["activations[eq]"] = json!(eq);
                }
                if let Some(gt) = activations.gt {
                    query["activations[gt]"] = json!(gt);
                }
                if let Some(gte) = activations.gte {
                    query["activations[gte]"] = json!(gte);
                }
                if let Some(lt) = activations.lt {
                    query["activations[lt]"] = json!(lt);
                }
                if let Some(lte) = activations.lte {
                    query["activations[lte]"] = json!(lte);
                }
            }
            if let Some(ref expires) = opts.expires {
                if let Some(value) = &expires.r#in {
                    query["expires[in]"] = json!(value);
                }
                if let Some(value) = &expires.on {
                    query["expires[on]"] = json!(value);
                }
                if let Some(value) = &expires.before {
                    query["expires[before]"] = json!(value);
                }
                if let Some(value) = &expires.after {
                    query["expires[after]"] = json!(value);
                }
            }
            if let Some(ref expired) = opts.expired {
                if let Some(value) = &expired.r#in {
                    query["expired[in]"] = json!(value);
                }
                if let Some(value) = &expired.on {
                    query["expired[on]"] = json!(value);
                }
                if let Some(value) = &expired.before {
                    query["expired[before]"] = json!(value);
                }
                if let Some(value) = &expired.after {
                    query["expired[after]"] = json!(value);
                }
            }
            if let Some(ref activity) = opts.activity {
                if let Some(value) = &activity.inside {
                    query["activity[inside]"] = json!(value);
                }
                if let Some(value) = &activity.outside {
                    query["activity[outside]"] = json!(value);
                }
                if let Some(value) = &activity.before {
                    query["activity[before]"] = json!(value);
                }
                if let Some(value) = &activity.after {
                    query["activity[after]"] = json!(value);
                }
            }
        }

        let response = client.get("licenses", Some(&query)).await?;

        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct LicensesResponse {
            pub data: Vec<KeygenResponseData<LicenseAttributes>>,
            pub meta: Option<Value>,
            pub links: Option<Value>,
        }

        let licenses_response: LicensesResponse = serde_json::from_value(response.body)?;
        Ok(ResourcePage {
            data: licenses_response
                .data
                .into_iter()
                .map(|data| {
                    License::from(data).with_client(Arc::clone(&client), Arc::clone(&config))
                })
                .collect(),
            meta: licenses_response.meta,
            links: licenses_response.links,
        })
    }

    /// Get a license by ID
    #[cfg(feature = "token")]
    pub async fn get(id: &str) -> Result<License, Error> {
        let config = Arc::new(get_config()?);
        let client = Arc::new(Client::new(ClientOptions::from(config.as_ref().clone()))?);
        Self::get_with_client(client, config, id).await
    }

    #[cfg(feature = "token")]
    pub(crate) async fn get_with_client(
        client: Arc<Client>,
        config: Arc<KeygenConfig>,
        id: &str,
    ) -> Result<License, Error> {
        let endpoint = format!("licenses/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(License::from(license_response.data).with_client(client, config))
    }

    /// Update a license
    #[cfg(feature = "token")]
    pub async fn update(&self, request: LicenseUpdateRequest) -> Result<License, Error> {
        let client = self.get_client()?;
        let endpoint = format!("licenses/{}", self.id);
        let body = request.to_json_body();
        let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(self.inherit_client(License::from(license_response.data)))
    }

    /// Delete a license
    #[cfg(feature = "token")]
    pub async fn delete(&self) -> Result<(), Error> {
        let client = self.get_client()?;
        let endpoint = format!("licenses/{}", self.id);
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }

    /// Suspend a license
    #[cfg(feature = "token")]
    pub async fn suspend(&self) -> Result<License, Error> {
        let client = self.get_client()?;
        let endpoint = format!("licenses/{}/actions/suspend", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(self.inherit_client(License::from(license_response.data)))
    }

    /// Reinstate a suspended license
    #[cfg(feature = "token")]
    pub async fn reinstate(&self) -> Result<License, Error> {
        let client = self.get_client()?;
        let endpoint = format!("licenses/{}/actions/reinstate", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(self.inherit_client(License::from(license_response.data)))
    }

    /// Renew a license
    #[cfg(feature = "token")]
    pub async fn renew(&self) -> Result<License, Error> {
        let client = self.get_client()?;
        let endpoint = format!("licenses/{}/actions/renew", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(self.inherit_client(License::from(license_response.data)))
    }

    /// Revoke a license
    #[cfg(feature = "token")]
    pub async fn revoke(&self) -> Result<(), Error> {
        let client = self.get_client()?;
        let endpoint = format!("licenses/{}/actions/revoke", self.id);
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }

    pub async fn increment_usage(&self, increment: Option<u32>) -> Result<License, Error> {
        let client = self.get_client()?;
        let endpoint = format!("licenses/{}/actions/increment-usage", self.id);
        let body = increment.map(|value| json!({ "meta": { "increment": value } }));
        let response = client.post(&endpoint, body.as_ref(), None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(self.inherit_client(License::from(license_response.data)))
    }

    #[cfg(feature = "token")]
    pub async fn decrement_usage(&self, decrement: Option<u32>) -> Result<License, Error> {
        let client = self.get_client()?;
        let endpoint = format!("licenses/{}/actions/decrement-usage", self.id);
        let body = decrement.map(|value| json!({ "meta": { "decrement": value } }));
        let response = client.post(&endpoint, body.as_ref(), None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(self.inherit_client(License::from(license_response.data)))
    }

    /// Reset the license's usage count to zero (Admin only)
    ///
    /// This is an administrative operation typically used at the start
    /// of a new billing period or for license resets.
    #[cfg(feature = "token")]
    pub async fn reset_usage(&self) -> Result<License, Error> {
        let client = self.get_client()?;
        let endpoint = format!("licenses/{}/actions/reset-usage", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(self.inherit_client(License::from(license_response.data)))
    }

    /// Attach entitlements to a license
    #[cfg(feature = "token")]
    pub async fn attach_entitlements(&self, entitlement_ids: &[String]) -> Result<(), Error> {
        self.attach_entitlements_with_response(entitlement_ids)
            .await?;
        Ok(())
    }

    /// Attach entitlements and return the created relationship resources.
    #[cfg(feature = "token")]
    pub async fn attach_entitlements_with_response(
        &self,
        entitlement_ids: &[String],
    ) -> Result<Vec<LicenseEntitlementAttachment>, Error> {
        let client = self.get_client()?;
        let endpoint = format!("licenses/{}/entitlements", self.id);

        let data: Vec<Value> = entitlement_ids
            .iter()
            .map(|id| {
                json!({
                    "type": "entitlements",
                    "id": id
                })
            })
            .collect();

        let body = json!({
            "data": data
        });

        let response = client
            .post::<Value, Value, ()>(&endpoint, Some(&body), None::<&()>)
            .await?;
        if response.body.is_null() {
            return Ok(Vec::new());
        }
        let attachments: LicenseAttachmentsResponse = serde_json::from_value(response.body)?;
        Ok(attachments
            .data
            .into_iter()
            .map(LicenseEntitlementAttachment::from)
            .collect())
    }

    /// Detach entitlements from a license
    #[cfg(feature = "token")]
    pub async fn detach_entitlements(&self, entitlement_ids: &[String]) -> Result<(), Error> {
        let client = self.get_client()?;
        let endpoint = format!("licenses/{}/entitlements", self.id);

        let data: Vec<Value> = entitlement_ids
            .iter()
            .map(|id| {
                json!({
                    "type": "entitlements",
                    "id": id
                })
            })
            .collect();

        let body = json!({
            "data": data
        });

        client
            .delete::<Value, Value>(&endpoint, Some(&body))
            .await?;
        Ok(())
    }

    /// Generate a token scoped to this license.
    #[cfg(feature = "token")]
    pub async fn generate_token(
        &self,
        request: Option<LicenseTokenCreateRequest>,
    ) -> Result<Token, Error> {
        let client = self.get_client()?;
        let endpoint = format!("licenses/{}/tokens", self.id);
        let attributes = serde_json::to_value(request.unwrap_or_default())?;
        let body = json!({
            "data": {
                "type": "tokens",
                "attributes": attributes
            }
        });
        let response = client.post(&endpoint, Some(&body), None::<&()>).await?;
        let token_response: TokenResponse = serde_json::from_value(response.body)?;
        Ok(Token::from(token_response.data))
    }

    /// Attach users to this license.
    #[cfg(feature = "token")]
    pub async fn attach_users(&self, user_ids: &[String]) -> Result<(), Error> {
        self.attach_users_with_response(user_ids).await?;
        Ok(())
    }

    /// Attach users and return the created relationship resources.
    #[cfg(feature = "token")]
    pub async fn attach_users_with_response(
        &self,
        user_ids: &[String],
    ) -> Result<Vec<LicenseUserAttachment>, Error> {
        let client = self.get_client()?;
        let endpoint = format!("licenses/{}/users", self.id);
        let data: Vec<Value> = user_ids
            .iter()
            .map(|id| {
                json!({
                    "type": "users",
                    "id": id
                })
            })
            .collect();
        let body = json!({ "data": data });
        let response = client
            .post::<Value, Value, ()>(&endpoint, Some(&body), None::<&()>)
            .await?;
        if response.body.is_null() {
            return Ok(Vec::new());
        }
        let attachments: LicenseAttachmentsResponse = serde_json::from_value(response.body)?;
        Ok(attachments
            .data
            .into_iter()
            .map(LicenseUserAttachment::from)
            .collect())
    }

    /// Detach users from this license.
    #[cfg(feature = "token")]
    pub async fn detach_users(&self, user_ids: &[String]) -> Result<(), Error> {
        let client = self.get_client()?;
        let endpoint = format!("licenses/{}/users", self.id);
        let data: Vec<Value> = user_ids
            .iter()
            .map(|id| {
                json!({
                    "type": "users",
                    "id": id
                })
            })
            .collect();
        let body = json!({ "data": data });
        client
            .delete::<Value, Value>(&endpoint, Some(&body))
            .await?;
        Ok(())
    }

    /// List users attached to this license.
    #[cfg(feature = "token")]
    pub async fn users(
        &self,
        options: Option<&PaginationOptions>,
    ) -> Result<ResourcePage<User>, Error> {
        let client = self.get_client()?;
        let endpoint = format!("licenses/{}/users", self.id);
        let query = options.map(|options| {
            cursor_pagination_query(
                options.limit,
                options.page_size,
                options.page_cursor.as_deref(),
                None,
            )
        });
        let response = client.get(&endpoint, query.as_ref()).await?;
        let users_response: LicenseUsersResponse = serde_json::from_value(response.body)?;
        Ok(ResourcePage {
            data: users_response.data.into_iter().map(User::from).collect(),
            meta: users_response.meta,
            links: users_response.links,
        })
    }

    /// Change the policy associated with this license.
    #[cfg(feature = "token")]
    pub async fn change_policy(&self, policy_id: &str) -> Result<License, Error> {
        let client = self.get_client()?;
        let endpoint = format!("licenses/{}/policy", self.id);
        let body = json!({
            "data": {
                "type": "policies",
                "id": policy_id
            }
        });
        let response = client.put(&endpoint, Some(&body), None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(self.inherit_client(License::from(license_response.data)))
    }

    /// Change the owner associated with this license.
    #[cfg(feature = "token")]
    pub async fn change_owner(&self, owner_id: &str) -> Result<License, Error> {
        let client = self.get_client()?;
        let endpoint = format!("licenses/{}/owner", self.id);
        let body = json!({
            "data": {
                "type": "users",
                "id": owner_id
            }
        });
        let response = client.put(&endpoint, Some(&body), None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(self.inherit_client(License::from(license_response.data)))
    }

    /// Change the group associated with this license.
    #[cfg(feature = "token")]
    pub async fn change_group(&self, group_id: &str) -> Result<License, Error> {
        let client = self.get_client()?;
        let endpoint = format!("licenses/{}/group", self.id);
        let body = json!({
            "data": {
                "type": "groups",
                "id": group_id
            }
        });
        let response = client.put(&endpoint, Some(&body), None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(self.inherit_client(License::from(license_response.data)))
    }
}

pub struct LicenseService<'a> {
    client: &'a KeygenClient,
}

impl<'a> LicenseService<'a> {
    pub(crate) fn new(client: &'a KeygenClient) -> Self {
        Self { client }
    }

    pub fn resource(&self, id: &str) -> License {
        License::from_id(id).with_client(self.client.transport_arc(), self.client.config_arc())
    }

    pub fn from_key(&self, key: &str) -> License {
        License::from_key(key).with_client(self.client.transport_arc(), self.client.config_arc())
    }

    pub async fn validate(
        &self,
        id: &str,
        request: &LicenseValidationRequest,
    ) -> Result<LicenseValidationResult, Error> {
        self.resource(id).validate(request).await
    }

    pub async fn validate_key(
        &self,
        key: &str,
        request: &LicenseValidationRequest,
    ) -> Result<LicenseValidationResult, Error> {
        self.from_key(key).validate_key(request).await
    }

    #[must_use = "verification result should be checked"]
    pub fn verify(&self, scheme: SchemeCode, signed_key: &str) -> Result<Vec<u8>, Error> {
        License::from_signed_key(scheme, signed_key)
            .with_client(self.client.transport_arc(), self.client.config_arc())
            .verify()
    }

    pub async fn activate(
        &self,
        id: &str,
        fingerprint: &str,
        components: &[Component],
    ) -> Result<Machine, Error> {
        self.resource(id).activate(fingerprint, components).await
    }

    pub async fn deactivate_machine(
        &self,
        license_id: &str,
        machine_id: &str,
    ) -> Result<(), Error> {
        self.resource(license_id).deactivate(machine_id).await
    }

    pub async fn machine(&self, license_id: &str, machine_id: &str) -> Result<Machine, Error> {
        self.resource(license_id).machine(machine_id).await
    }

    pub async fn machines(
        &self,
        id: &str,
        options: Option<&PaginationOptions>,
    ) -> Result<ResourcePage<Machine>, Error> {
        self.resource(id).machines(options).await
    }

    pub async fn entitlements(
        &self,
        id: &str,
        options: Option<&PaginationOptions>,
    ) -> Result<ResourcePage<Entitlement>, Error> {
        self.resource(id).entitlements(options).await
    }

    pub async fn checkout(
        &self,
        id: &str,
        options: &LicenseCheckoutOpts,
    ) -> Result<LicenseFile, Error> {
        self.resource(id).checkout(options).await
    }

    pub async fn checkout_certificate(
        &self,
        id: &str,
        options: &LicenseCheckoutOpts,
    ) -> Result<String, Error> {
        self.resource(id).checkout_certificate(options).await
    }

    pub async fn check_in(&self, id: &str) -> Result<License, Error> {
        self.resource(id).check_in().await
    }

    pub async fn increment_usage(
        &self,
        id: &str,
        increment: Option<u32>,
    ) -> Result<License, Error> {
        self.resource(id).increment_usage(increment).await
    }

    #[cfg(feature = "token")]
    pub async fn create(&self, request: LicenseCreateRequest) -> Result<License, Error> {
        License::create_with_client(
            self.client.transport_arc(),
            self.client.config_arc(),
            request,
        )
        .await
    }

    #[cfg(feature = "token")]
    pub async fn list(&self, options: Option<&LicenseListOptions>) -> Result<LicensePage, Error> {
        License::list_with_client(
            self.client.transport_arc(),
            self.client.config_arc(),
            options,
        )
        .await
    }

    #[cfg(feature = "token")]
    pub async fn get(&self, id: &str) -> Result<License, Error> {
        License::get_with_client(self.client.transport_arc(), self.client.config_arc(), id).await
    }

    #[cfg(feature = "token")]
    pub async fn update(&self, id: &str, request: LicenseUpdateRequest) -> Result<License, Error> {
        self.resource(id).update(request).await
    }

    #[cfg(feature = "token")]
    pub async fn delete(&self, id: &str) -> Result<(), Error> {
        self.resource(id).delete().await
    }

    #[cfg(feature = "token")]
    pub async fn suspend(&self, id: &str) -> Result<License, Error> {
        self.resource(id).suspend().await
    }

    #[cfg(feature = "token")]
    pub async fn reinstate(&self, id: &str) -> Result<License, Error> {
        self.resource(id).reinstate().await
    }

    #[cfg(feature = "token")]
    pub async fn renew(&self, id: &str) -> Result<License, Error> {
        self.resource(id).renew().await
    }

    #[cfg(feature = "token")]
    pub async fn revoke(&self, id: &str) -> Result<(), Error> {
        self.resource(id).revoke().await
    }

    #[cfg(feature = "token")]
    pub async fn decrement_usage(
        &self,
        id: &str,
        decrement: Option<u32>,
    ) -> Result<License, Error> {
        self.resource(id).decrement_usage(decrement).await
    }

    #[cfg(feature = "token")]
    pub async fn reset_usage(&self, id: &str) -> Result<License, Error> {
        self.resource(id).reset_usage().await
    }

    #[cfg(feature = "token")]
    pub async fn attach_entitlements(
        &self,
        id: &str,
        entitlement_ids: &[String],
    ) -> Result<(), Error> {
        self.resource(id).attach_entitlements(entitlement_ids).await
    }

    #[cfg(feature = "token")]
    pub async fn attach_entitlements_with_response(
        &self,
        id: &str,
        entitlement_ids: &[String],
    ) -> Result<Vec<LicenseEntitlementAttachment>, Error> {
        self.resource(id)
            .attach_entitlements_with_response(entitlement_ids)
            .await
    }

    #[cfg(feature = "token")]
    pub async fn detach_entitlements(
        &self,
        id: &str,
        entitlement_ids: &[String],
    ) -> Result<(), Error> {
        self.resource(id).detach_entitlements(entitlement_ids).await
    }

    #[cfg(feature = "token")]
    pub async fn generate_token(
        &self,
        id: &str,
        request: Option<LicenseTokenCreateRequest>,
    ) -> Result<Token, Error> {
        self.resource(id).generate_token(request).await
    }

    #[cfg(feature = "token")]
    pub async fn attach_users(&self, id: &str, user_ids: &[String]) -> Result<(), Error> {
        self.resource(id).attach_users(user_ids).await
    }

    #[cfg(feature = "token")]
    pub async fn attach_users_with_response(
        &self,
        id: &str,
        user_ids: &[String],
    ) -> Result<Vec<LicenseUserAttachment>, Error> {
        self.resource(id).attach_users_with_response(user_ids).await
    }

    #[cfg(feature = "token")]
    pub async fn detach_users(&self, id: &str, user_ids: &[String]) -> Result<(), Error> {
        self.resource(id).detach_users(user_ids).await
    }

    #[cfg(feature = "token")]
    pub async fn users(
        &self,
        id: &str,
        options: Option<&PaginationOptions>,
    ) -> Result<ResourcePage<User>, Error> {
        self.resource(id).users(options).await
    }

    #[cfg(feature = "token")]
    pub async fn change_policy(&self, id: &str, policy_id: &str) -> Result<License, Error> {
        self.resource(id).change_policy(policy_id).await
    }

    #[cfg(feature = "token")]
    pub async fn change_owner(&self, id: &str, owner_id: &str) -> Result<License, Error> {
        self.resource(id).change_owner(owner_id).await
    }

    #[cfg(feature = "token")]
    pub async fn change_group(&self, id: &str, group_id: &str) -> Result<License, Error> {
        self.resource(id).change_group(group_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{reset_config, set_config, KeygenConfig};
    #[cfg(feature = "token")]
    use chrono::TimeZone;
    use mockito::{mock, server_url};
    use serde_json::json;

    fn create_test_license() -> License {
        License {
            id: "test_license_id".to_string(),
            scheme: None,
            name: Some("Test License".to_string()),
            key: "TEST-LICENSE-KEY".to_string(),
            expiry: None,
            status: None,
            uses: None,
            max_machines: None,
            max_cores: None,
            max_uses: None,
            max_processes: None,
            max_users: None,
            protected: None,
            suspended: None,
            permissions: None,
            policy: None,
            metadata: HashMap::new(),
            account_id: None,
            product_id: None,
            group_id: None,
            owner_id: None,
            config: None,
            client: None,
            ..License::from_key("TEST-LICENSE-KEY")
        }
    }

    fn validation_request() -> LicenseValidationRequest {
        LicenseValidationRequest {
            nonce: Some(1),
            scope: LicenseValidationScope {
                product: Some("test_product".to_string()),
                fingerprint: Some("test_fingerprint".to_string()),
                components: Some(vec!["comp1".to_string(), "comp2".to_string()]),
                ..Default::default()
            },
        }
    }

    fn get_mock_body() -> String {
        json!({
            "meta": {
                "ts": "2021-01-01T00:00:00Z",
                "valid": true,
                "detail": "is valid",
                "code": "VALID",
                "scope": {
                    "fingerprint": "test_fingerprint",
                    "components": ["comp1", "comp2"],
                    "product": "test_product"
                }
            },
            "data": {
                "id": "test_license_id",
                "type": "licenses",
                "attributes": {
                    "name": "Test License",
                    "key": "TEST-LICENSE-KEY",
                    "expiry": null,
                    "status": "valid",
                    "metadata": {
                        "customer_name": "Test Customer",
                        "customer_email": "test@example.com",
                        "is_premium": true
                    }
                },
                "relationships": {
                    "policy": {
                        "data": {
                            "type": "policies",
                            "id": "11314277-0f31-4a77-9366-0299e9f52123"
                        }
                    }
                }
            }
        })
        .to_string()
    }

    #[tokio::test]
    async fn test_validate() {
        let license = create_test_license();
        let _m = mock("POST", "/v1/licenses/test_license_id/actions/validate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(get_mock_body())
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            ..Default::default()
        })
        .unwrap();

        let result = license.validate(&validation_request()).await;
        assert!(result.is_ok());
        let _ = reset_config();
    }

    #[tokio::test]
    async fn test_validate_key() {
        let license = create_test_license();
        let _m = mock("POST", "/v1/licenses/actions/validate-key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(get_mock_body())
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            license_key: Some("TEST-LICENSE-KEY".to_string()),
            ..Default::default()
        });

        let result = license.validate_key(&validation_request()).await;
        assert!(result.is_ok());
        let _ = reset_config();
    }

    #[tokio::test]
    async fn test_validate_key_supports_complete_scope_and_missing_license() {
        let request = LicenseValidationRequest {
            nonce: Some(42),
            scope: LicenseValidationScope {
                product: Some("product-1".into()),
                policy: Some("policy-1".into()),
                fingerprints: Some(vec!["fp-a".into(), "fp-b".into()]),
                fingerprint: Some("fp-primary".into()),
                components: Some(vec!["component-1".into()]),
                machine: Some("machine-1".into()),
                user: Some("user-1".into()),
                entitlements: Some(vec!["ENTITLEMENT".into()]),
                checksum: Some("checksum".into()),
                version: Some("1.2.3".into()),
            },
        };
        let _mock = mock("POST", "/v1/licenses/actions/validate-key")
            .match_body(mockito::Matcher::Json(json!({
                "meta": {
                    "key": "TEST-LICENSE-KEY",
                    "nonce": 42,
                    "scope": {
                        "product": "product-1",
                        "policy": "policy-1",
                        "fingerprints": ["fp-a", "fp-b"],
                        "fingerprint": "fp-primary",
                        "components": ["component-1"],
                        "machine": "machine-1",
                        "user": "user-1",
                        "entitlements": ["ENTITLEMENT"],
                        "checksum": "checksum",
                        "version": "1.2.3"
                    }
                }
            })))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "meta": {
                        "ts": "2021-01-01T00:00:00Z",
                        "valid": false,
                        "detail": "license not found",
                        "code": "NOT_FOUND",
                        "scope": {}
                    },
                    "data": null
                })
                .to_string(),
            )
            .create();
        let config = KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            ..Default::default()
        };

        let result = create_test_license()
            .with_config(config)
            .validate_key(&request)
            .await
            .unwrap();

        assert!(!result.meta.valid);
        assert_eq!(result.meta.code, "NOT_FOUND");
        assert!(result.license.is_none());
    }

    #[test]
    fn test_verify() {
        let mut license = create_test_license();

        license.scheme = Some(SchemeCode::Ed25519Sign);
        let result = license.verify();
        assert!(matches!(result, Err(Error::PublicKeyMissing)));

        license.scheme = None;
        let result = license.verify();
        assert!(matches!(result, Err(Error::LicenseNotSigned)));
    }

    #[tokio::test]
    async fn test_validate_with_metadata() {
        let license = create_test_license();
        let _m = mock("POST", "/v1/licenses/test_license_id/actions/validate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(get_mock_body())
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            ..Default::default()
        })
        .unwrap();

        let result = license.validate(&validation_request()).await;

        assert!(result.is_ok());
        let validated_license = result.unwrap().license.unwrap();

        // Verify metadata fields
        assert!(validated_license.metadata.contains_key("customer_name"));
        assert_eq!(
            validated_license
                .metadata
                .get("customer_name")
                .unwrap()
                .as_str()
                .unwrap(),
            "Test Customer"
        );

        assert!(validated_license.metadata.contains_key("customer_email"));
        assert_eq!(
            validated_license
                .metadata
                .get("customer_email")
                .unwrap()
                .as_str()
                .unwrap(),
            "test@example.com"
        );

        assert!(validated_license.metadata.contains_key("is_premium"));
        assert!(validated_license
            .metadata
            .get("is_premium")
            .unwrap()
            .as_bool()
            .unwrap());

        let _ = reset_config();
    }

    #[tokio::test]
    async fn test_pagination_options() {
        let _m = mock("GET", "/v1/licenses/test_license_id/machines")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("limit".into(), "50".into()),
                mockito::Matcher::UrlEncoded("page[cursor]".into(), "cursor-2".into()),
                mockito::Matcher::UrlEncoded("page[size]".into(), "10".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": [],
                    "meta": {
                        "page": {
                            "cursor": "cursor-2",
                            "next": "cursor-3"
                        }
                    },
                    "links": {
                        "next": "/v1/licenses/test_license_id/machines?page[cursor]=cursor-3"
                    }
                })
                .to_string(),
            )
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            ..Default::default()
        })
        .unwrap();

        let config = KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            ..Default::default()
        };
        let license = create_test_license().with_config(config);

        let pagination_options = PaginationOptions {
            limit: Some(50),
            page_size: Some(10),
            page_cursor: Some("cursor-2".to_string()),
        };

        let page = license.machines(Some(&pagination_options)).await.unwrap();
        assert!(page.data.is_empty());
        assert_eq!(page.meta.unwrap()["page"]["next"], "cursor-3");
        assert!(page.links.unwrap()["next"]
            .as_str()
            .unwrap()
            .contains("cursor-3"));
        let _ = reset_config();
    }

    #[tokio::test]
    async fn test_validation_errors() {
        let license = create_test_license();

        // Test expired license
        let _m = mock("POST", "/v1/licenses/test_license_id/actions/validate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "meta": {
                        "ts": "2021-01-01T00:00:00Z",
                        "valid": false,
                        "detail": "license expired",
                        "code": "EXPIRED",
                        "scope": {
                            "fingerprint": "test_fingerprint"
                        }
                    },
                    "data": {
                        "id": "test_license_id",
                        "type": "licenses",
                        "attributes": {
                            "key": "TEST-LICENSE-KEY",
                            "name": "Test License",
                            "expiry": null,
                            "status": "expired",
                            "uses": null,
                            "maxMachines": null,
                            "maxCores": null,
                            "maxUses": null,
                            "maxProcesses": null,
                            "protected": null,
                            "suspended": null,
                            "metadata": {}
                        },
                        "relationships": {
                            "policy": {
                                "data": {
                                    "type": "policies",
                                    "id": "policy_123"
                                }
                            }
                        }
                    }
                })
                .to_string(),
            )
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            ..Default::default()
        })
        .unwrap();

        let result = license.validate(&validation_request()).await.unwrap();
        assert!(!result.meta.valid);
        assert_eq!(result.meta.code, "EXPIRED");
        assert!(matches!(
            result.into_license(),
            Err(Error::LicenseKeyInvalid { code, .. }) if code == "EXPIRED"
        ));
        let _ = reset_config();
    }

    #[tokio::test]
    async fn test_validation_with_empty_scope() {
        let license = create_test_license();
        let _m = mock("POST", "/v1/licenses/test_license_id/actions/validate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(get_mock_body())
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            ..Default::default()
        })
        .unwrap();

        let result = license.validate(&LicenseValidationRequest::default()).await;
        assert!(result.is_ok());
        let _ = reset_config();
    }

    #[tokio::test]
    async fn test_license_with_all_attributes() {
        let license = create_test_license();
        let _m = mock("POST", "/v1/licenses/test_license_id/actions/validate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "meta": {
                        "ts": "2021-01-01T00:00:00Z",
                        "valid": true,
                        "detail": "is valid",
                        "code": "VALID",
                        "scope": {
                            "fingerprint": "test_fingerprint"
                        }
                    },
                    "data": {
                        "id": "test_license_id",
                        "type": "licenses",
                        "attributes": {
                            "key": "TEST-LICENSE-KEY",
                            "name": "Test License",
                            "expiry": "2025-12-31T23:59:59Z",
                            "status": "active",
                            "uses": 5,
                            "version": "1.2.3",
                            "floating": true,
                            "encrypted": false,
                            "scheme": "ED25519_SIGN",
                            "strict": true,
                            "maxMachines": 10,
                            "maxCores": 20,
                            "maxUses": 100,
                            "maxProcesses": 5,
                            "maxUsers": 3,
                            "maxMemory": 8589934592_i64,
                            "maxDisk": 53687091200_i64,
                            "requireHeartbeat": true,
                            "requireCheckIn": true,
                            "lastValidated": "2025-01-01T00:00:00Z",
                            "lastCheckOut": "2025-01-02T00:00:00Z",
                            "lastCheckIn": "2025-01-03T00:00:00Z",
                            "nextCheckIn": "2025-01-04T00:00:00Z",
                            "created": "2024-01-01T00:00:00Z",
                            "updated": "2025-01-04T00:00:00Z",
                            "protected": true,
                            "suspended": false,
                            "metadata": {
                                "tier": "premium",
                                "features": ["feature_a", "feature_b"]
                            }
                        },
                        "relationships": {
                            "environment": {
                                "data": { "type": "environments", "id": "environment-1" }
                            },
                            "policy": {
                                "data": {
                                    "type": "policies",
                                    "id": "policy_123"
                                }
                            }
                        }
                    }
                })
                .to_string(),
            )
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            ..Default::default()
        })
        .unwrap();

        let result = license.validate(&validation_request()).await;
        assert!(result.is_ok());

        let validated_license = result.unwrap().license.unwrap();
        assert_eq!(validated_license.uses, Some(5));
        assert_eq!(validated_license.max_machines, Some(10));
        assert_eq!(validated_license.max_cores, Some(20));
        assert_eq!(validated_license.max_uses, Some(100));
        assert_eq!(validated_license.max_processes, Some(5));
        assert_eq!(validated_license.max_users, Some(3));
        assert_eq!(validated_license.max_memory, Some(8_589_934_592));
        assert_eq!(validated_license.max_disk, Some(53_687_091_200));
        assert_eq!(validated_license.version.as_deref(), Some("1.2.3"));
        assert_eq!(validated_license.scheme, Some(SchemeCode::Ed25519Sign));
        assert_eq!(
            validated_license.environment_id.as_deref(),
            Some("environment-1")
        );
        assert_eq!(validated_license.require_heartbeat, Some(true));
        assert_eq!(validated_license.require_check_in, Some(true));
        assert!(validated_license.protected == Some(true));
        assert_eq!(validated_license.suspended, Some(false));
        assert!(validated_license.metadata.contains_key("tier"));
        assert_eq!(
            validated_license
                .metadata
                .get("tier")
                .unwrap()
                .as_str()
                .unwrap(),
            "premium"
        );

        let _ = reset_config();
    }

    #[tokio::test]
    async fn test_machine_activation_errors() {
        let license = create_test_license();
        let _m = mock("POST", "/v1/machines")
            .with_status(422)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "errors": [{
                        "title": "Unprocessable Entity",
                        "detail": "License has reached machine limit",
                        "code": "MACHINE_LIMIT_EXCEEDED"
                    }]
                })
                .to_string(),
            )
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            ..Default::default()
        })
        .unwrap();

        let result = license.activate("test_fingerprint", &[]).await;
        assert!(result.is_err());
        let _ = reset_config();
    }

    #[test]
    fn test_license_relationships() {
        use crate::{
            KeygenRelationship, KeygenRelationshipData, KeygenRelationships, KeygenResponseData,
        };

        // Test that all relationship IDs are properly extracted
        let license_data = KeygenResponseData {
            id: "test-license-id".to_string(),
            r#type: "licenses".to_string(),
            attributes: LicenseAttributes {
                key: "TEST-LICENSE-KEY".to_string(),
                name: Some("Test License".to_string()),
                expiry: None,
                status: Some("active".to_string()),
                uses: Some(5),
                max_machines: Some(10),
                max_cores: Some(20),
                max_uses: Some(100),
                max_processes: Some(5),
                max_users: None,
                protected: Some(true),
                suspended: Some(false),
                permissions: None,
                metadata: HashMap::new(),
                ..Default::default()
            },
            relationships: KeygenRelationships {
                policy: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "policies".to_string(),
                        id: "test-policy-id".to_string(),
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
                product: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "products".to_string(),
                        id: "test-product-id".to_string(),
                    }),
                    links: None,
                }),
                group: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "groups".to_string(),
                        id: "test-group-id".to_string(),
                    }),
                    links: None,
                }),
                owner: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "users".to_string(),
                        id: "test-owner-id".to_string(),
                    }),
                    links: None,
                }),
                users: None,
                machines: None,
                environment: None,
                license: None,
                release: None,
                other: HashMap::new(),
            },
        };

        let license = License::from(license_data);

        assert_eq!(license.policy, Some("test-policy-id".to_string()));
        assert_eq!(license.account_id, Some("test-account-id".to_string()));
        assert_eq!(license.product_id, Some("test-product-id".to_string()));
        assert_eq!(license.group_id, Some("test-group-id".to_string()));
        assert_eq!(license.owner_id, Some("test-owner-id".to_string()));
        assert_eq!(license.id, "test-license-id");
        assert_eq!(license.key, "TEST-LICENSE-KEY");
    }

    #[test]
    fn test_license_without_relationships() {
        use crate::{KeygenRelationships, KeygenResponseData};

        // Test that all relationship IDs are None when no relationships exist
        let license_data = KeygenResponseData {
            id: "test-license-id".to_string(),
            r#type: "licenses".to_string(),
            attributes: LicenseAttributes {
                key: "TEST-LICENSE-KEY".to_string(),
                name: Some("Test License".to_string()),
                expiry: None,
                status: Some("active".to_string()),
                uses: None,
                max_machines: None,
                max_cores: None,
                max_uses: None,
                max_processes: None,
                max_users: None,
                protected: None,
                suspended: None,
                permissions: None,
                metadata: HashMap::new(),
                ..Default::default()
            },
            relationships: KeygenRelationships {
                policy: None,
                account: None,
                product: None,
                group: None,
                owner: None,
                users: None,
                machines: None,
                environment: None,
                license: None,
                release: None,
                other: HashMap::new(),
            },
        };

        let license = License::from(license_data);

        assert_eq!(license.policy, None);
        assert_eq!(license.account_id, None);
        assert_eq!(license.product_id, None);
        assert_eq!(license.group_id, None);
        assert_eq!(license.owner_id, None);
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_create_license_basic() {
        let _m = mock("POST", "/v1/licenses")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": "license-123",
                        "type": "licenses",
                        "attributes": {
                            "key": "LICENSE-KEY-123",
                            "name": "Test License",
                            "expiry": null,
                            "status": "active",
                            "uses": null,
                            "maxMachines": 5,
                            "maxCores": null,
                            "maxUses": null,
                            "maxProcesses": null,
                            "protected": null,
                            "suspended": false,
                            "metadata": {
                                "tier": "premium"
                            }
                        },
                        "relationships": {
                            "policy": {
                                "data": {
                                    "type": "policies",
                                    "id": "policy-123"
                                }
                            },
                            "owner": {
                                "data": {
                                    "type": "users",
                                    "id": "user-123"
                                }
                            }
                        }
                    }
                })
                .to_string(),
            )
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let mut metadata = HashMap::new();
        metadata.insert("tier".to_string(), json!("premium"));

        let request = LicenseCreateRequest::new("policy-123".to_string())
            .with_name("Test License".to_string())
            .with_max_machines(5)
            .with_owner_id("user-123".to_string())
            .with_metadata(metadata);

        let result = License::create(request).await;

        assert!(result.is_ok());
        let license = result.unwrap();
        assert_eq!(license.id, "license-123");
        assert_eq!(license.key, "LICENSE-KEY-123");
        assert_eq!(license.name, Some("Test License".to_string()));
        assert_eq!(license.max_machines, Some(5));
        assert_eq!(license.owner_id, Some("user-123".to_string()));
        assert!(license.metadata.contains_key("tier"));

        let _ = reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_create_license_with_custom_key() {
        let _m = mock("POST", "/v1/licenses")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": "license-456",
                        "type": "licenses",
                        "attributes": {
                            "key": "CUSTOM-LICENSE-KEY",
                            "name": "Custom License",
                            "expiry": "2025-12-31T23:59:59Z",
                            "status": "active",
                            "uses": null,
                            "maxMachines": 10,
                            "maxCores": null,
                            "maxUses": null,
                            "maxProcesses": null,
                            "protected": null,
                            "suspended": false,
                            "metadata": {}
                        },
                        "relationships": {
                            "policy": {
                                "data": {
                                    "type": "policies",
                                    "id": "policy-456"
                                }
                            },
                            "group": {
                                "data": {
                                    "type": "groups",
                                    "id": "group-456"
                                }
                            }
                        }
                    }
                })
                .to_string(),
            )
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let expiry = Utc.from_utc_datetime(
            &chrono::NaiveDate::from_ymd_opt(2025, 12, 31)
                .unwrap()
                .and_hms_opt(23, 59, 59)
                .unwrap(),
        );

        let request = LicenseCreateRequest::new("policy-456".to_string())
            .with_name("Custom License".to_string())
            .with_key("CUSTOM-LICENSE-KEY".to_string())
            .with_expiry(expiry)
            .with_max_machines(10)
            .with_group_id("group-456".to_string());

        let result = License::create(request).await;

        assert!(result.is_ok());
        let license = result.unwrap();
        assert_eq!(license.id, "license-456");
        assert_eq!(license.key, "CUSTOM-LICENSE-KEY");
        assert_eq!(license.name, Some("Custom License".to_string()));
        assert_eq!(license.max_machines, Some(10));
        assert_eq!(license.group_id, Some("group-456".to_string()));
        assert!(license.owner_id.is_none());

        let _ = reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_create_license_minimal() {
        let _m = mock("POST", "/v1/licenses")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": "license-789",
                        "type": "licenses",
                        "attributes": {
                            "key": "AUTO-GENERATED-KEY",
                            "name": null,
                            "expiry": null,
                            "status": "active",
                            "uses": null,
                            "maxMachines": null,
                            "maxCores": null,
                            "maxUses": null,
                            "maxProcesses": null,
                            "protected": null,
                            "suspended": false,
                            "metadata": {}
                        },
                        "relationships": {
                            "policy": {
                                "data": {
                                    "type": "policies",
                                    "id": "policy-789"
                                }
                            }
                        }
                    }
                })
                .to_string(),
            )
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let request = LicenseCreateRequest::new("policy-789".to_string());
        let result = License::create(request).await;

        assert!(result.is_ok());
        let license = result.unwrap();
        assert_eq!(license.id, "license-789");
        assert_eq!(license.key, "AUTO-GENERATED-KEY");
        assert!(license.name.is_none());
        assert!(license.max_machines.is_none());
        assert!(license.owner_id.is_none());
        assert!(license.group_id.is_none());

        let _ = reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_create_license_error() {
        let _m = mock("POST", "/v1/licenses")
            .with_status(422)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "errors": [
                        {
                            "title": "Unprocessable Entity",
                            "detail": "Policy is required",
                            "code": "MISSING_POLICY"
                        }
                    ]
                })
                .to_string(),
            )
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let request = LicenseCreateRequest::new("invalid-policy".to_string());
        let result = License::create(request).await;

        assert!(result.is_err());
        let _ = reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_create_license_with_all_parameters() {
        let _m = mock("POST", "/v1/licenses")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": "license-comprehensive",
                        "type": "licenses",
                        "attributes": {
                            "key": "COMPREHENSIVE-LICENSE-KEY",
                            "name": "Comprehensive License",
                            "expiry": "2025-12-31T23:59:59Z",
                            "status": "active",
                            "uses": null,
                            "maxMachines": 10,
                            "maxProcesses": 5,
                            "maxUsers": 3,
                            "maxCores": 8,
                            "maxUses": 100,
                            "protected": true,
                            "suspended": false,
                            "permissions": ["activate", "deactivate", "read"],
                            "metadata": {
                                "tier": "enterprise",
                                "features": ["advanced", "premium"]
                            }
                        },
                        "relationships": {
                            "policy": {
                                "data": {
                                    "type": "policies",
                                    "id": "policy-comprehensive"
                                }
                            },
                            "owner": {
                                "data": {
                                    "type": "users",
                                    "id": "user-comprehensive"
                                }
                            },
                            "group": {
                                "data": {
                                    "type": "groups",
                                    "id": "group-comprehensive"
                                }
                            }
                        }
                    }
                })
                .to_string(),
            )
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let expiry = Utc.from_utc_datetime(
            &chrono::NaiveDate::from_ymd_opt(2025, 12, 31)
                .unwrap()
                .and_hms_opt(23, 59, 59)
                .unwrap(),
        );
        let mut metadata = HashMap::new();
        metadata.insert("tier".to_string(), json!("enterprise"));
        metadata.insert("features".to_string(), json!(["advanced", "premium"]));

        let request = LicenseCreateRequest::new("policy-comprehensive".to_string())
            .with_name("Comprehensive License".to_string())
            .with_key("COMPREHENSIVE-LICENSE-KEY".to_string())
            .with_expiry(expiry)
            .with_max_machines(10)
            .with_max_processes(5)
            .with_max_users(3)
            .with_max_cores(8)
            .with_max_uses(100)
            .with_protected(true)
            .with_suspended(false)
            .with_permissions(vec![
                "activate".to_string(),
                "deactivate".to_string(),
                "read".to_string(),
            ])
            .with_metadata(metadata)
            .with_owner_id("user-comprehensive".to_string())
            .with_group_id("group-comprehensive".to_string());

        let result = License::create(request).await;

        assert!(result.is_ok());
        let license = result.unwrap();
        assert_eq!(license.id, "license-comprehensive");
        assert_eq!(license.key, "COMPREHENSIVE-LICENSE-KEY");
        assert_eq!(license.name, Some("Comprehensive License".to_string()));
        assert_eq!(license.max_machines, Some(10));
        assert_eq!(license.max_processes, Some(5));
        assert_eq!(license.max_cores, Some(8));
        assert_eq!(license.max_uses, Some(100));
        assert!(license.protected == Some(true));
        assert_eq!(license.suspended, Some(false));
        assert_eq!(license.owner_id, Some("user-comprehensive".to_string()));
        assert_eq!(license.group_id, Some("group-comprehensive".to_string()));
        assert!(license.metadata.contains_key("tier"));
        assert_eq!(
            license.metadata.get("tier").unwrap().as_str().unwrap(),
            "enterprise"
        );

        let _ = reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_update_license_comprehensive() {
        let license = create_test_license();
        let _m = mock("PATCH", "/v1/licenses/test_license_id")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": "test_license_id",
                        "type": "licenses",
                        "attributes": {
                            "key": "TEST-LICENSE-KEY",
                            "name": "Updated License Name",
                            "expiry": "2025-12-31T23:59:59Z",
                            "status": "active",
                            "uses": null,
                            "maxMachines": 20,
                            "maxProcesses": 10,
                            "maxUsers": 5,
                            "maxCores": 16,
                            "maxUses": 200,
                            "protected": true,
                            "suspended": false,
                            "permissions": ["read", "write", "activate"],
                            "metadata": {
                                "tier": "enterprise",
                                "updated": true
                            }
                        },
                        "relationships": {
                            "policy": {
                                "data": {
                                    "type": "policies",
                                    "id": "policy-123"
                                }
                            }
                        }
                    }
                })
                .to_string(),
            )
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let expiry = Utc.from_utc_datetime(
            &chrono::NaiveDate::from_ymd_opt(2025, 12, 31)
                .unwrap()
                .and_hms_opt(23, 59, 59)
                .unwrap(),
        );
        let mut metadata = HashMap::new();
        metadata.insert("tier".to_string(), json!("enterprise"));
        metadata.insert("updated".to_string(), json!(true));

        let request = LicenseUpdateRequest::new()
            .with_name("Updated License Name".to_string())
            .with_expiry(expiry)
            .with_max_machines(20)
            .with_max_processes(10)
            .with_max_users(5)
            .with_max_cores(16)
            .with_max_uses(200)
            .with_protected(true)
            .with_suspended(false)
            .with_permissions(vec![
                "read".to_string(),
                "write".to_string(),
                "activate".to_string(),
            ])
            .with_metadata(metadata);

        let result = license.update(request).await;

        assert!(result.is_ok());
        let updated_license = result.unwrap();
        assert_eq!(updated_license.id, "test_license_id");
        assert_eq!(
            updated_license.name,
            Some("Updated License Name".to_string())
        );
        assert_eq!(updated_license.max_machines, Some(20));
        assert_eq!(updated_license.max_processes, Some(10));
        assert_eq!(updated_license.max_users, Some(5));
        assert_eq!(updated_license.max_cores, Some(16));
        assert_eq!(updated_license.max_uses, Some(200));
        assert!(updated_license.protected == Some(true));
        assert_eq!(updated_license.suspended, Some(false));
        assert!(updated_license.metadata.contains_key("tier"));
        assert_eq!(
            updated_license
                .metadata
                .get("tier")
                .unwrap()
                .as_str()
                .unwrap(),
            "enterprise"
        );

        let _ = reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_update_license_clear_limits() {
        let license = create_test_license();
        let _m = mock("PATCH", "/v1/licenses/test_license_id")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": "test_license_id",
                        "type": "licenses",
                        "attributes": {
                            "key": "TEST-LICENSE-KEY",
                            "name": "Test License",
                            "expiry": null,
                            "status": "active",
                            "uses": null,
                            "maxMachines": null,
                            "maxProcesses": null,
                            "maxUsers": null,
                            "maxCores": null,
                            "maxUses": null,
                            "protected": null,
                            "suspended": false,
                            "metadata": {}
                        },
                        "relationships": {
                            "policy": {
                                "data": {
                                    "type": "policies",
                                    "id": "policy-123"
                                }
                            }
                        }
                    }
                })
                .to_string(),
            )
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        // Test clearing limits (setting them to null)
        let request = LicenseUpdateRequest::new()
            .clear_max_machines()
            .clear_max_processes()
            .clear_max_users()
            .clear_max_cores()
            .clear_max_uses();

        let result = license.update(request).await;

        assert!(result.is_ok());
        let updated_license = result.unwrap();
        assert_eq!(updated_license.max_machines, None);
        assert_eq!(updated_license.max_processes, None);
        assert_eq!(updated_license.max_users, None);
        assert_eq!(updated_license.max_cores, None);
        assert_eq!(updated_license.max_uses, None);

        let _ = reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_update_license_basic() {
        let license = create_test_license();
        let _m = mock("PATCH", "/v1/licenses/test_license_id")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": "test_license_id",
                        "type": "licenses",
                        "attributes": {
                            "key": "TEST-LICENSE-KEY",
                            "name": "Updated Name",
                            "expiry": "2025-06-30T23:59:59Z",
                            "status": "active",
                            "uses": null,
                            "maxMachines": null,
                            "maxCores": null,
                            "maxUses": null,
                            "maxProcesses": null,
                            "protected": null,
                            "suspended": false,
                            "metadata": {
                                "updated": true
                            }
                        },
                        "relationships": {
                            "policy": {
                                "data": {
                                    "type": "policies",
                                    "id": "policy-123"
                                }
                            }
                        }
                    }
                })
                .to_string(),
            )
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let expiry = Utc.from_utc_datetime(
            &chrono::NaiveDate::from_ymd_opt(2025, 6, 30)
                .unwrap()
                .and_hms_opt(23, 59, 59)
                .unwrap(),
        );
        let mut metadata = HashMap::new();
        metadata.insert("updated".to_string(), json!(true));

        let request = LicenseUpdateRequest::new()
            .with_name("Updated Name".to_string())
            .with_expiry(expiry)
            .with_metadata(metadata);

        let result = license.update(request).await;

        assert!(result.is_ok());
        let updated_license = result.unwrap();
        assert_eq!(updated_license.id, "test_license_id");
        assert_eq!(updated_license.name, Some("Updated Name".to_string()));
        assert!(updated_license.metadata.contains_key("updated"));

        let _ = reset_config();
    }

    #[test]
    fn test_license_update_request_builder() {
        let mut metadata = HashMap::new();
        metadata.insert("tier".to_string(), json!("premium"));

        let request = LicenseUpdateRequest::new()
            .with_name("Test License".to_string())
            .with_max_machines(10)
            .with_max_memory(8_589_934_592)
            .clear_max_disk()
            .with_protected(true)
            .with_metadata(metadata.clone());

        assert_eq!(request.name, UpdateField::Set("Test License".to_string()));
        assert!(matches!(request.max_machines, UpdateField::Set(10)));
        assert!(request.protected == Some(true));
        assert_eq!(request.metadata, Some(metadata));

        // Test clearing a limit
        let request_with_clear = LicenseUpdateRequest::new()
            .with_max_machines(5)
            .clear_max_machines();

        assert!(matches!(
            request_with_clear.max_machines,
            UpdateField::Clear
        ));

        // Test JSON body conversion
        let body = request.to_json_body();
        let data = body.get("data").unwrap();
        let attributes = data.get("attributes").unwrap();
        assert_eq!(
            attributes.get("name").unwrap().as_str().unwrap(),
            "Test License"
        );
        assert_eq!(attributes.get("maxMachines").unwrap().as_i64().unwrap(), 10);
        assert!(attributes.get("protected").unwrap().as_bool().unwrap());
        assert!(attributes.get("metadata").is_some());
        assert_eq!(attributes["maxMemory"], json!(8_589_934_592_i64));
        assert!(attributes["maxDisk"].is_null());
    }

    #[test]
    fn test_license_create_supports_memory_and_disk_limits() {
        let body = LicenseCreateRequest::new("policy-1".to_string())
            .with_max_memory(8_589_934_592)
            .with_max_disk(53_687_091_200)
            .to_json_body();

        assert_eq!(
            body["data"]["attributes"]["maxMemory"],
            json!(8_589_934_592_i64)
        );
        assert_eq!(
            body["data"]["attributes"]["maxDisk"],
            json!(53_687_091_200_i64)
        );
    }

    #[test]
    fn test_checkout_query_supports_current_api_options() {
        let options = LicenseCheckoutOpts {
            ttl: UpdateField::Clear,
            include: Some(vec!["entitlements".into(), "product".into()]),
            encrypt: None,
            algorithm: Some(LicenseFileAlgorithm::Base64Ed25519),
        };

        assert_eq!(
            options.to_query().unwrap(),
            json!({
                "ttl": "null",
                "include": "entitlements,product",
                "algorithm": "base64+ed25519"
            })
        );

        let invalid = LicenseCheckoutOpts {
            encrypt: Some(true),
            algorithm: Some(LicenseFileAlgorithm::Aes256GcmEd25519),
            ..Default::default()
        };
        assert!(invalid.to_query().is_err());
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_license_list_cursor_pagination() {
        let _m = mock("GET", "/v1/licenses")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("page[cursor]".into(), "cursor-2".into()),
                mockito::Matcher::UrlEncoded("page[size]".into(), "15".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": [
                        {
                            "id": "license-1",
                            "type": "licenses",
                            "attributes": {
                                "key": "TEST-LICENSE-1",
                                "name": "Test License 1",
                                "expiry": null,
                                "status": "active",
                                "uses": null,
                                "maxMachines": null,
                                "maxCores": null,
                                "maxUses": null,
                                "maxProcesses": null,
                                "protected": null,
                                "suspended": false,
                                "metadata": {}
                            },
                            "relationships": {
                                "policy": {
                                    "data": {
                                        "type": "policies",
                                        "id": "policy-123"
                                    }
                                }
                            }
                        }
                    ],
                    "meta": { "count": 1 },
                    "links": {
                        "self": "https://api.keygen.sh/v1/licenses?page[cursor]=cursor-2",
                        "next": "https://api.keygen.sh/v1/licenses?page[cursor]=cursor-3"
                    }
                })
                .to_string(),
            )
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let options = LicenseListOptions {
            page_cursor: Some("cursor-2".to_string()),
            page_size: Some(15),
            ..Default::default()
        };

        let result = License::list(Some(&options)).await;
        assert!(result.is_ok());
        let licenses = result.unwrap();
        assert_eq!(licenses.data.len(), 1);
        assert_eq!(licenses.data[0].id, "license-1");
        assert_eq!(licenses.meta.as_ref().unwrap()["count"], 1);
        assert!(licenses.links.as_ref().unwrap()["next"]
            .as_str()
            .unwrap()
            .contains("cursor-3"));

        let _ = reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_license_list_initial_cursor_pagination() {
        let _m = mock("GET", "/v1/licenses")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("page[cursor]".into(), String::new()),
                mockito::Matcher::UrlEncoded("page[size]".into(), "15".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({ "data": [] }).to_string())
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });
        let options = LicenseListOptions {
            page_size: Some(15),
            ..Default::default()
        };

        let result = License::list(Some(&options)).await;

        assert!(result.is_ok());
        let _ = reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_license_list_pagination_with_limit_only() {
        let _m = mock("GET", "/v1/licenses")
            .match_query(mockito::Matcher::UrlEncoded("limit".into(), "5".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": []
                })
                .to_string(),
            )
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let options = LicenseListOptions {
            limit: Some(5),
            ..Default::default()
        };

        let result = License::list(Some(&options)).await;
        assert!(result.is_ok());

        let _ = reset_config();
    }

    #[tokio::test]
    async fn test_pagination_options_with_new_parameters() {
        let _m = mock("GET", "/v1/licenses/test_license_id/machines")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("page[cursor]".into(), "cursor-3".into()),
                mockito::Matcher::UrlEncoded("page[size]".into(), "25".into()),
                mockito::Matcher::UrlEncoded("limit".into(), "50".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": []
                })
                .to_string(),
            )
            .create();

        let config = KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            ..Default::default()
        };
        let _ = set_config(config.clone());

        let license = create_test_license().with_config(config);
        let pagination_options = PaginationOptions {
            limit: Some(50),
            page_size: Some(25),
            page_cursor: Some("cursor-3".to_string()),
        };

        let result = license.machines(Some(&pagination_options)).await;
        assert!(result.is_ok());
        let _ = reset_config();
    }

    #[tokio::test]
    async fn test_pagination_options_start_with_an_empty_cursor() {
        let _m = mock("GET", "/v1/licenses/test_license_id/machines")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("page[cursor]".into(), String::new()),
                mockito::Matcher::UrlEncoded("page[size]".into(), "25".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({ "data": [] }).to_string())
            .create();

        let config = KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            ..Default::default()
        };
        let _ = set_config(config.clone());
        let license = create_test_license().with_config(config);
        let pagination_options = PaginationOptions {
            page_size: Some(25),
            ..Default::default()
        };

        let result = license.machines(Some(&pagination_options)).await;

        assert!(result.is_ok());
        let _ = reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_attach_entitlements() {
        let license = create_test_license();
        let _m = mock("POST", "/v1/licenses/test_license_id/entitlements")
            .with_status(204)
            .with_header("content-type", "application/json")
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let entitlement_ids = vec!["entitlement-1".to_string(), "entitlement-2".to_string()];

        let result = license.attach_entitlements(&entitlement_ids).await;
        assert!(result.is_ok());
        let _ = reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_detach_entitlements() {
        let license = create_test_license();
        let _m = mock("DELETE", "/v1/licenses/test_license_id/entitlements")
            .with_status(204)
            .with_header("content-type", "application/json")
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let entitlement_ids = vec!["entitlement-1".to_string(), "entitlement-2".to_string()];

        let result = license.detach_entitlements(&entitlement_ids).await;
        assert!(result.is_ok());
        let _ = reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_attach_entitlements_empty_list() {
        let license = create_test_license();
        let _m = mock("POST", "/v1/licenses/test_license_id/entitlements")
            .with_status(204)
            .with_header("content-type", "application/json")
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let entitlement_ids: Vec<String> = vec![];
        let result = license.attach_entitlements(&entitlement_ids).await;
        assert!(result.is_ok());
        let _ = reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_detach_entitlements_empty_list() {
        let license = create_test_license();
        let _m = mock("DELETE", "/v1/licenses/test_license_id/entitlements")
            .with_status(204)
            .with_header("content-type", "application/json")
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let entitlement_ids: Vec<String> = vec![];
        let result = license.detach_entitlements(&entitlement_ids).await;
        assert!(result.is_ok());
        let _ = reset_config();
    }

    #[tokio::test]
    async fn test_increment_usage() {
        let license = create_test_license();
        let _m = mock(
            "POST",
            "/v1/licenses/test_license_id/actions/increment-usage",
        )
        .match_body(mockito::Matcher::Json(json!({
            "meta": { "increment": 3 }
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "data": {
                    "id": "test_license_id",
                    "type": "licenses",
                    "attributes": {
                        "key": "TEST-LICENSE-KEY",
                        "name": "Test License",
                        "expiry": null,
                        "status": "active",
                        "uses": 6, // Incremented from 5 to 6
                        "maxMachines": null,
                        "maxCores": null,
                        "maxUses": 100,
                        "maxProcesses": null,
                        "maxUsers": null,
                        "protected": null,
                        "suspended": false,
                        "permissions": null,
                        "metadata": {}
                    },
                    "relationships": {
                        "policy": {
                            "data": {
                                "type": "policies",
                                "id": "policy-123"
                            }
                        }
                    }
                }
            })
            .to_string(),
        )
        .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            license_key: Some("TEST-LICENSE-KEY".to_string()),
            ..Default::default()
        });

        let result = license.increment_usage(Some(3)).await;
        assert!(result.is_ok());
        let updated_license = result.unwrap();
        assert_eq!(updated_license.uses, Some(6));
        assert_eq!(updated_license.id, "test_license_id");
        let _ = reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_decrement_usage() {
        let license = create_test_license();
        let _m = mock(
            "POST",
            "/v1/licenses/test_license_id/actions/decrement-usage",
        )
        .match_body(mockito::Matcher::Json(json!({
            "meta": { "decrement": 2 }
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "data": {
                    "id": "test_license_id",
                    "type": "licenses",
                    "attributes": {
                        "key": "TEST-LICENSE-KEY",
                        "name": "Test License",
                        "expiry": null,
                        "status": "active",
                        "uses": 4, // Decremented from 5 to 4
                        "maxMachines": null,
                        "maxCores": null,
                        "maxUses": 100,
                        "maxProcesses": null,
                        "maxUsers": null,
                        "protected": null,
                        "suspended": false,
                        "permissions": null,
                        "metadata": {}
                    },
                    "relationships": {
                        "policy": {
                            "data": {
                                "type": "policies",
                                "id": "policy-123"
                            }
                        }
                    }
                }
            })
            .to_string(),
        )
        .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let result = license.decrement_usage(Some(2)).await;
        assert!(result.is_ok());
        let updated_license = result.unwrap();
        assert_eq!(updated_license.uses, Some(4));
        assert_eq!(updated_license.id, "test_license_id");
        let _ = reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_reset_usage() {
        let license = create_test_license();
        let _m = mock("POST", "/v1/licenses/test_license_id/actions/reset-usage")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": "test_license_id",
                        "type": "licenses",
                        "attributes": {
                            "key": "TEST-LICENSE-KEY",
                            "name": "Test License",
                            "expiry": null,
                            "status": "active",
                            "uses": 0, // Reset to 0
                            "maxMachines": null,
                            "maxCores": null,
                            "maxUses": 100,
                            "maxProcesses": null,
                            "maxUsers": null,
                            "protected": null,
                            "suspended": false,
                            "permissions": null,
                            "metadata": {}
                        },
                        "relationships": {
                            "policy": {
                                "data": {
                                    "type": "policies",
                                    "id": "policy-123"
                                }
                            }
                        }
                    }
                })
                .to_string(),
            )
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let result = license.reset_usage().await;
        assert!(result.is_ok());
        let updated_license = result.unwrap();
        assert_eq!(updated_license.uses, Some(0));
        assert_eq!(updated_license.id, "test_license_id");
        let _ = reset_config();
    }

    #[test]
    fn test_scheme_code_serialization() {
        assert_eq!(
            serde_json::to_string(&SchemeCode::Ed25519Sign).unwrap(),
            "\"ED25519_SIGN\""
        );
        assert_eq!(
            serde_json::to_string(&SchemeCode::EcdsaP256Sign).unwrap(),
            "\"ECDSA_P256_SIGN\""
        );
        assert_eq!(
            serde_json::to_string(&SchemeCode::Rsa2048Pkcs1PssSignV2).unwrap(),
            "\"RSA_2048_PKCS1_PSS_SIGN_V2\""
        );
        assert_eq!(
            serde_json::to_string(&SchemeCode::LegacyEncrypt).unwrap(),
            "\"LEGACY_ENCRYPT\""
        );
    }

    #[test]
    fn test_scheme_code_deserialization() {
        assert_eq!(
            serde_json::from_str::<SchemeCode>("\"ED25519_SIGN\"").unwrap(),
            SchemeCode::Ed25519Sign
        );
        assert_eq!(
            serde_json::from_str::<SchemeCode>("\"ECDSA_P256_SIGN\"").unwrap(),
            SchemeCode::EcdsaP256Sign
        );
        assert_eq!(
            serde_json::from_str::<SchemeCode>("\"RSA_2048_PKCS1_PSS_SIGN_V2\"").unwrap(),
            SchemeCode::Rsa2048Pkcs1PssSignV2
        );
        assert_eq!(
            serde_json::from_str::<SchemeCode>("\"LEGACY_ENCRYPT\"").unwrap(),
            SchemeCode::LegacyEncrypt
        );
    }
}
