use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::entitlement::Entitlement;
use crate::license_file::LicenseFile;
use crate::to_napi_error;
use crate::token_module::Token;
use crate::user::User;

#[napi(object)]
#[derive(Clone)]
pub struct License {
    pub id: String,
    pub scheme: Option<String>,
    pub key: String,
    pub name: Option<String>,
    pub expiry: Option<String>,
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
    pub last_validated: Option<String>,
    pub last_check_out: Option<String>,
    pub last_check_in: Option<String>,
    pub next_check_in: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub policy: Option<String>,
    pub metadata: serde_json::Value,
    pub account_id: Option<String>,
    pub product_id: Option<String>,
    pub group_id: Option<String>,
    pub owner_id: Option<String>,
    pub environment_id: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
}

impl From<keygen_rs::license::License> for License {
    fn from(lic: keygen_rs::license::License) -> Self {
        License {
            id: lic.id,
            scheme: lic
                .scheme
                .as_ref()
                .and_then(|s| serde_json::to_value(s).ok())
                .and_then(|v| v.as_str().map(String::from)),
            key: lic.key,
            name: lic.name,
            expiry: lic.expiry.map(|dt| dt.to_rfc3339()),
            status: lic.status,
            uses: lic.uses,
            version: lic.version,
            floating: lic.floating,
            encrypted: lic.encrypted,
            strict: lic.strict,
            max_machines: lic.max_machines,
            max_cores: lic.max_cores,
            max_uses: lic.max_uses,
            max_processes: lic.max_processes,
            max_users: lic.max_users,
            max_memory: lic.max_memory,
            max_disk: lic.max_disk,
            protected: lic.protected,
            suspended: lic.suspended,
            require_heartbeat: lic.require_heartbeat,
            require_check_in: lic.require_check_in,
            last_validated: lic.last_validated.map(|dt| dt.to_rfc3339()),
            last_check_out: lic.last_check_out.map(|dt| dt.to_rfc3339()),
            last_check_in: lic.last_check_in.map(|dt| dt.to_rfc3339()),
            next_check_in: lic.next_check_in.map(|dt| dt.to_rfc3339()),
            permissions: lic.permissions,
            policy: lic.policy,
            metadata: serde_json::to_value(lic.metadata).unwrap_or_default(),
            account_id: lic.account_id,
            product_id: lic.product_id,
            group_id: lic.group_id,
            owner_id: lic.owner_id,
            environment_id: lic.environment_id,
            created: lic.created.map(|dt| dt.to_rfc3339()),
            updated: lic.updated.map(|dt| dt.to_rfc3339()),
        }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct LicenseCreateRequest {
    pub policy_id: String,
    pub name: Option<String>,
    pub key: Option<String>,
    pub expiry: Option<String>,
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
    pub metadata: Option<serde_json::Value>,
    pub owner_id: Option<String>,
    pub group_id: Option<String>,
}

// LicenseUpdateRequest is accepted as serde_json::Value in update_license()
// to distinguish null (clear) from undefined (keep) for clearable fields.
// The TypeScript type is specified via #[napi(ts_args_type)] on the function.

#[napi(object)]
#[derive(Clone)]
pub struct ListLicensesOptions {
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub page_cursor: Option<String>,
    pub status: Option<String>,
    pub product: Option<String>,
    pub policy: Option<String>,
    pub owner: Option<String>,
    pub user: Option<String>,
    pub group: Option<String>,
    pub machine: Option<String>,
    pub assigned: Option<bool>,
    pub unassigned: Option<bool>,
    pub activated: Option<bool>,
    pub metadata: Option<serde_json::Value>,
    pub activations: Option<NumericFilter>,
    pub expires: Option<DateWindowFilter>,
    pub expired: Option<DateWindowFilter>,
    pub activity: Option<LicenseActivityFilter>,
}

#[napi(object)]
#[derive(Clone)]
pub struct NumericFilter {
    pub eq: Option<i32>,
    pub gt: Option<i32>,
    pub gte: Option<i32>,
    pub lt: Option<i32>,
    pub lte: Option<i32>,
}

#[napi(object)]
#[derive(Clone)]
pub struct DateWindowFilter {
    pub r#in: Option<String>,
    pub on: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
}

#[napi(object)]
#[derive(Clone)]
pub struct LicenseActivityFilter {
    pub inside: Option<String>,
    pub outside: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
}

#[napi(object)]
#[derive(Clone)]
pub struct LicenseRelationshipListOptions {
    pub limit: Option<i32>,
    pub page_size: Option<i32>,
    pub page_cursor: Option<String>,
}

#[napi(object)]
#[derive(Clone)]
pub struct CreateTokenRequest {
    pub name: Option<String>,
    pub expiry: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub max_activations: Option<u32>,
    pub max_deactivations: Option<u32>,
}

#[napi(object)]
#[derive(Clone)]
pub struct LicenseValidationScope {
    pub product: Option<String>,
    pub policy: Option<String>,
    pub fingerprints: Option<Vec<String>>,
    pub fingerprint: Option<String>,
    pub components: Option<Vec<String>>,
    pub machine: Option<String>,
    pub user: Option<String>,
    pub entitlements: Option<Vec<String>>,
    pub checksum: Option<String>,
    pub version: Option<String>,
}

#[napi(object)]
#[derive(Clone)]
pub struct LicenseValidationRequest {
    pub nonce: Option<i64>,
    pub scope: LicenseValidationScope,
}

#[napi(object)]
pub struct LicenseValidationMeta {
    pub ts: String,
    pub valid: bool,
    pub detail: String,
    pub code: String,
    pub scope: LicenseValidationScope,
    pub nonce: Option<i64>,
}

#[napi(object)]
pub struct LicenseValidationResult {
    pub license: Option<License>,
    pub meta: LicenseValidationMeta,
}

#[napi(object)]
pub struct LicensePage {
    pub data: Vec<License>,
    pub meta: Option<serde_json::Value>,
    pub links: Option<serde_json::Value>,
}

#[napi(object)]
pub struct EntitlementPage {
    pub data: Vec<Entitlement>,
    pub meta: Option<serde_json::Value>,
    pub links: Option<serde_json::Value>,
}

#[napi(object)]
pub struct UserPage {
    pub data: Vec<User>,
    pub meta: Option<serde_json::Value>,
    pub links: Option<serde_json::Value>,
}

impl From<LicenseValidationScope> for keygen_rs::license::LicenseValidationScope {
    fn from(scope: LicenseValidationScope) -> Self {
        Self {
            product: scope.product,
            policy: scope.policy,
            fingerprints: scope.fingerprints,
            fingerprint: scope.fingerprint,
            components: scope.components,
            machine: scope.machine,
            user: scope.user,
            entitlements: scope.entitlements,
            checksum: scope.checksum,
            version: scope.version,
        }
    }
}

impl From<keygen_rs::license::LicenseValidationScope> for LicenseValidationScope {
    fn from(scope: keygen_rs::license::LicenseValidationScope) -> Self {
        Self {
            product: scope.product,
            policy: scope.policy,
            fingerprints: scope.fingerprints,
            fingerprint: scope.fingerprint,
            components: scope.components,
            machine: scope.machine,
            user: scope.user,
            entitlements: scope.entitlements,
            checksum: scope.checksum,
            version: scope.version,
        }
    }
}

fn make_license(id: String) -> keygen_rs::license::License {
    let mut lic = keygen_rs::license::License::from_key("");
    lic.id = id;
    lic
}

fn parse_checkout_options(
    opts: Option<serde_json::Value>,
) -> Result<keygen_rs::license::LicenseCheckoutOpts> {
    let mut checkout_opts = keygen_rs::license::LicenseCheckoutOpts::default();
    let Some(opts) = opts else {
        return Ok(checkout_opts);
    };
    let opts = opts
        .as_object()
        .ok_or_else(|| napi::Error::new(Status::InvalidArg, "opts must be an object"))?;
    if let Some(ttl) = opts.get("ttl") {
        checkout_opts.ttl = if ttl.is_null() {
            keygen_rs::license::UpdateField::Clear
        } else {
            keygen_rs::license::UpdateField::Set(ttl.as_i64().ok_or_else(|| {
                napi::Error::new(Status::InvalidArg, "ttl must be a number or null")
            })?)
        };
    }
    checkout_opts.include = opts
        .get("include")
        .cloned()
        .map(serde_json::from_value)
        .transpose()?;
    checkout_opts.encrypt = opts
        .get("encrypt")
        .map(|encrypt| {
            encrypt
                .as_bool()
                .ok_or_else(|| napi::Error::new(Status::InvalidArg, "encrypt must be a boolean"))
        })
        .transpose()?;
    checkout_opts.algorithm = opts
        .get("algorithm")
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .map_err(|error| napi::Error::new(Status::InvalidArg, error.to_string()))?;
    Ok(checkout_opts)
}

#[napi]
pub async fn validate(request: LicenseValidationRequest) -> Result<LicenseValidationResult> {
    let request = keygen_rs::license::LicenseValidationRequest {
        nonce: request.nonce,
        scope: request.scope.into(),
    };
    let result = keygen_rs::validate(&request).await.map_err(to_napi_error)?;
    Ok(LicenseValidationResult {
        license: result.license.map(License::from),
        meta: LicenseValidationMeta {
            ts: result.meta.ts.to_rfc3339(),
            valid: result.meta.valid,
            detail: result.meta.detail,
            code: result.meta.code,
            scope: result.meta.scope.into(),
            nonce: result.meta.nonce,
        },
    })
}

#[napi]
pub fn verify(scheme: String, signed_key: String) -> Result<Buffer> {
    let scheme_code: keygen_rs::license::SchemeCode =
        serde_json::from_value(serde_json::Value::String(scheme))
            .map_err(|e| napi::Error::new(Status::InvalidArg, format!("Invalid scheme: {e}")))?;

    keygen_rs::verify(scheme_code, &signed_key)
        .map(|bytes| bytes.into())
        .map_err(to_napi_error)
}

#[napi]
pub async fn create_license(request: LicenseCreateRequest) -> Result<License> {
    let mut req = keygen_rs::license::LicenseCreateRequest::new(request.policy_id);

    if let Some(name) = request.name {
        req = req.with_name(name);
    }
    if let Some(key) = request.key {
        req = req.with_key(key);
    }
    if let Some(expiry) = request.expiry {
        let dt = chrono::DateTime::parse_from_rfc3339(&expiry)
            .map_err(|e| napi::Error::new(Status::InvalidArg, format!("Invalid expiry: {e}")))?
            .with_timezone(&chrono::Utc);
        req = req.with_expiry(dt);
    }
    if let Some(max_machines) = request.max_machines {
        req = req.with_max_machines(max_machines);
    }
    if let Some(max_processes) = request.max_processes {
        req = req.with_max_processes(max_processes);
    }
    if let Some(max_users) = request.max_users {
        req = req.with_max_users(max_users);
    }
    if let Some(max_cores) = request.max_cores {
        req = req.with_max_cores(max_cores);
    }
    if let Some(max_uses) = request.max_uses {
        req = req.with_max_uses(max_uses);
    }
    if let Some(max_memory) = request.max_memory {
        req = req.with_max_memory(max_memory);
    }
    if let Some(max_disk) = request.max_disk {
        req = req.with_max_disk(max_disk);
    }
    if let Some(protected) = request.protected {
        req = req.with_protected(protected);
    }
    if let Some(suspended) = request.suspended {
        req = req.with_suspended(suspended);
    }
    if let Some(permissions) = request.permissions {
        req = req.with_permissions(permissions);
    }
    if let Some(meta) = request.metadata {
        req = req.with_metadata(crate::to_metadata(meta)?);
    }
    if let Some(owner_id) = request.owner_id {
        req = req.with_owner_id(owner_id);
    }
    if let Some(group_id) = request.group_id {
        req = req.with_group_id(group_id);
    }

    keygen_rs::license::License::create(req)
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn list_licenses(options: Option<ListLicensesOptions>) -> Result<LicensePage> {
    let opts = options
        .map(|o| -> Result<keygen_rs::license::LicenseListOptions> {
            let limit = o
                .limit
                .map(i32::try_from)
                .transpose()
                .map_err(|_| napi::Error::new(Status::InvalidArg, "limit exceeds i32"))?;
            let page_size = o
                .page_size
                .map(i32::try_from)
                .transpose()
                .map_err(|_| napi::Error::new(Status::InvalidArg, "pageSize exceeds i32"))?;
            Ok(keygen_rs::license::LicenseListOptions {
                limit,
                page_size,
                page_cursor: o.page_cursor,
                status: o.status,
                product: o.product,
                policy: o.policy,
                owner: o.owner,
                user: o.user,
                group: o.group,
                machine: o.machine,
                assigned: o.assigned,
                unassigned: o.unassigned,
                activated: o.activated,
                metadata: crate::opt_metadata(o.metadata)?,
                activations: o
                    .activations
                    .map(|filter| keygen_rs::license::NumericFilter {
                        eq: filter.eq,
                        gt: filter.gt,
                        gte: filter.gte,
                        lt: filter.lt,
                        lte: filter.lte,
                    }),
                expires: o
                    .expires
                    .map(|filter| keygen_rs::license::DateWindowFilter {
                        r#in: filter.r#in,
                        on: filter.on,
                        before: filter.before,
                        after: filter.after,
                    }),
                expired: o
                    .expired
                    .map(|filter| keygen_rs::license::DateWindowFilter {
                        r#in: filter.r#in,
                        on: filter.on,
                        before: filter.before,
                        after: filter.after,
                    }),
                activity: o
                    .activity
                    .map(|filter| keygen_rs::license::LicenseActivityFilter {
                        inside: filter.inside,
                        outside: filter.outside,
                        before: filter.before,
                        after: filter.after,
                    }),
                ..Default::default()
            })
        })
        .transpose()?;

    let page = keygen_rs::license::License::list(opts.as_ref())
        .await
        .map_err(to_napi_error)?;
    Ok(LicensePage {
        data: page.data.into_iter().map(License::from).collect(),
        meta: page.meta,
        links: page.links,
    })
}

#[napi]
pub async fn get_license(id: String) -> Result<License> {
    keygen_rs::license::License::get(&id)
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

/// Update a license. Clearable integer fields accept `number | null`:
/// - `undefined` / absent → keep unchanged
/// - `null` → clear (set to null)
/// - `number` → set to value
#[napi(
    ts_args_type = "id: string, request: { name?: string | null; expiry?: string | null; maxMachines?: number | null; maxProcesses?: number | null; maxUsers?: number | null; maxCores?: number | null; maxUses?: number | null; maxMemory?: number | null; maxDisk?: number | null; protected?: boolean; suspended?: boolean; permissions?: string[]; metadata?: any }"
)]
pub async fn update_license(id: String, request: serde_json::Value) -> Result<License> {
    let lic = make_license(id);

    let obj = request
        .as_object()
        .ok_or_else(|| napi::Error::new(Status::InvalidArg, "request must be an object"))?;

    let mut req = keygen_rs::license::LicenseUpdateRequest::new();

    if let Some(value) = obj.get("name") {
        if value.is_null() {
            req = req.clear_name();
        } else if let Some(name) = value.as_str() {
            req = req.with_name(name.to_string());
        }
    }
    if let Some(v) = obj.get("expiry") {
        if v.is_null() {
            req = req.clear_expiry();
        } else if let Some(s) = v.as_str() {
            let dt = chrono::DateTime::parse_from_rfc3339(s)
                .map_err(|e| napi::Error::new(Status::InvalidArg, format!("Invalid expiry: {e}")))?
                .with_timezone(&chrono::Utc);
            req = req.with_expiry(dt);
        }
    }

    // Clearable integer fields: null → clear, number → set, absent → keep
    macro_rules! apply_clearable {
        ($obj:expr, $req:expr, $field:literal, $set:ident, $clear:ident) => {
            if let Some(v) = $obj.get($field) {
                if v.is_null() {
                    $req = $req.$clear();
                } else {
                    let n = v.as_i64().ok_or_else(|| {
                        napi::Error::new(
                            Status::InvalidArg,
                            format!("{} must be an integer or null", $field),
                        )
                    })?;
                    let n = i32::try_from(n).map_err(|_| {
                        napi::Error::new(Status::InvalidArg, format!("{} exceeds i32", $field))
                    })?;
                    $req = $req.$set(n);
                }
            }
        };
    }

    apply_clearable!(
        obj,
        req,
        "maxMachines",
        with_max_machines,
        clear_max_machines
    );
    apply_clearable!(
        obj,
        req,
        "maxProcesses",
        with_max_processes,
        clear_max_processes
    );
    apply_clearable!(obj, req, "maxUsers", with_max_users, clear_max_users);
    apply_clearable!(obj, req, "maxCores", with_max_cores, clear_max_cores);
    apply_clearable!(obj, req, "maxUses", with_max_uses, clear_max_uses);
    if let Some(value) = obj.get("maxMemory") {
        req = if value.is_null() {
            req.clear_max_memory()
        } else if let Some(max_memory) = value.as_i64() {
            req.with_max_memory(max_memory)
        } else {
            return Err(napi::Error::new(
                Status::InvalidArg,
                "maxMemory must be an integer or null",
            ));
        };
    }
    if let Some(value) = obj.get("maxDisk") {
        req = if value.is_null() {
            req.clear_max_disk()
        } else if let Some(max_disk) = value.as_i64() {
            req.with_max_disk(max_disk)
        } else {
            return Err(napi::Error::new(
                Status::InvalidArg,
                "maxDisk must be an integer or null",
            ));
        };
    }

    if let Some(serde_json::Value::Bool(protected)) = obj.get("protected") {
        req = req.with_protected(*protected);
    }
    if let Some(serde_json::Value::Bool(suspended)) = obj.get("suspended") {
        req = req.with_suspended(*suspended);
    }
    if let Some(serde_json::Value::Array(perms)) = obj.get("permissions") {
        let permissions: Vec<String> = perms
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        req = req.with_permissions(permissions);
    }
    if let Some(meta) = obj.get("metadata") {
        if !meta.is_null() {
            req = req.with_metadata(crate::to_metadata(meta.clone())?);
        }
    }

    lic.update(req)
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn delete_license(id: String) -> Result<()> {
    let lic = make_license(id);
    lic.delete().await.map_err(to_napi_error)
}

#[napi]
pub async fn suspend_license(id: String) -> Result<License> {
    let lic = make_license(id);
    lic.suspend()
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn reinstate_license(id: String) -> Result<License> {
    let lic = make_license(id);
    lic.reinstate()
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn renew_license(id: String) -> Result<License> {
    let lic = make_license(id);
    lic.renew().await.map(License::from).map_err(to_napi_error)
}

#[napi]
pub async fn revoke_license(id: String) -> Result<()> {
    let lic = make_license(id);
    lic.revoke().await.map_err(to_napi_error)
}

#[napi]
pub async fn increment_license_usage(id: String, increment: Option<u32>) -> Result<License> {
    let lic = make_license(id);
    lic.increment_usage(increment)
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn decrement_license_usage(id: String, decrement: Option<u32>) -> Result<License> {
    let lic = make_license(id);
    lic.decrement_usage(decrement)
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn reset_license_usage(id: String) -> Result<License> {
    let lic = make_license(id);
    lic.reset_usage()
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

#[napi(
    ts_args_type = "id: string, opts?: { ttl?: number | null; include?: string[]; encrypt?: boolean; algorithm?: 'aes-256-gcm+ed25519' | 'aes-256-gcm+ecdsa-p256' | 'aes-256-gcm+rsa-pss-sha256' | 'aes-256-gcm+rsa-sha256' | 'base64+ed25519' | 'base64+ecdsa-p256' | 'base64+rsa-pss-sha256' | 'base64+rsa-sha256' }"
)]
pub async fn checkout_license(id: String, opts: Option<serde_json::Value>) -> Result<LicenseFile> {
    let lic = make_license(id);
    let checkout_opts = parse_checkout_options(opts)?;

    lic.checkout(&checkout_opts)
        .await
        .map(LicenseFile::from)
        .map_err(to_napi_error)
}

#[napi(
    ts_args_type = "id: string, opts?: { ttl?: number | null; include?: string[]; encrypt?: boolean; algorithm?: 'aes-256-gcm+ed25519' | 'aes-256-gcm+ecdsa-p256' | 'aes-256-gcm+rsa-pss-sha256' | 'aes-256-gcm+rsa-sha256' | 'base64+ed25519' | 'base64+ecdsa-p256' | 'base64+rsa-pss-sha256' | 'base64+rsa-sha256' }"
)]
pub async fn checkout_license_certificate(
    id: String,
    opts: Option<serde_json::Value>,
) -> Result<String> {
    make_license(id)
        .checkout_certificate(&parse_checkout_options(opts)?)
        .await
        .map_err(to_napi_error)
}

#[napi]
pub async fn attach_license_entitlements(id: String, entitlement_ids: Vec<String>) -> Result<()> {
    let lic = make_license(id);
    lic.attach_entitlements(&entitlement_ids)
        .await
        .map_err(to_napi_error)
}

#[napi]
pub async fn detach_license_entitlements(id: String, entitlement_ids: Vec<String>) -> Result<()> {
    let lic = make_license(id);
    lic.detach_entitlements(&entitlement_ids)
        .await
        .map_err(to_napi_error)
}

#[napi]
pub async fn list_license_entitlements(
    id: String,
    options: Option<LicenseRelationshipListOptions>,
) -> Result<EntitlementPage> {
    let options = options.map(|options| keygen_rs::license::PaginationOptions {
        limit: options.limit,
        page_size: options.page_size,
        page_cursor: options.page_cursor,
    });
    let page = make_license(id)
        .entitlements(options.as_ref())
        .await
        .map_err(to_napi_error)?;
    Ok(EntitlementPage {
        data: page.data.into_iter().map(Entitlement::from).collect(),
        meta: page.meta,
        links: page.links,
    })
}

#[napi]
pub async fn check_in_license(id: String) -> Result<License> {
    let lic = make_license(id);
    lic.check_in()
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn generate_license_token(
    id: String,
    request: Option<CreateTokenRequest>,
) -> Result<Token> {
    let lic = make_license(id);
    let req = request
        .map(
            |request| -> Result<keygen_rs::license::LicenseTokenCreateRequest> {
                Ok(keygen_rs::license::LicenseTokenCreateRequest {
                    name: request.name,
                    expiry: request.expiry,
                    permissions: request.permissions,
                    max_activations: request.max_activations,
                    max_deactivations: request.max_deactivations,
                })
            },
        )
        .transpose()?;
    lic.generate_token(req)
        .await
        .map(Token::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn attach_license_users(id: String, user_ids: Vec<String>) -> Result<()> {
    let lic = make_license(id);
    lic.attach_users(&user_ids).await.map_err(to_napi_error)
}

#[napi]
pub async fn detach_license_users(id: String, user_ids: Vec<String>) -> Result<()> {
    let lic = make_license(id);
    lic.detach_users(&user_ids).await.map_err(to_napi_error)
}

#[napi]
pub async fn list_license_users(
    id: String,
    options: Option<LicenseRelationshipListOptions>,
) -> Result<UserPage> {
    let lic = make_license(id);
    let options = options.map(|options| keygen_rs::license::PaginationOptions {
        limit: options.limit,
        page_size: options.page_size,
        page_cursor: options.page_cursor,
    });
    let page = lic.users(options.as_ref()).await.map_err(to_napi_error)?;
    Ok(UserPage {
        data: page.data.into_iter().map(User::from).collect(),
        meta: page.meta,
        links: page.links,
    })
}

#[napi]
pub async fn change_license_policy(id: String, policy_id: String) -> Result<License> {
    let lic = make_license(id);
    lic.change_policy(&policy_id)
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn change_license_owner(id: String, owner_id: String) -> Result<License> {
    let lic = make_license(id);
    lic.change_owner(&owner_id)
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn change_license_group(id: String, group_id: String) -> Result<License> {
    let lic = make_license(id);
    lic.change_group(&group_id)
        .await
        .map(License::from)
        .map_err(to_napi_error)
}
