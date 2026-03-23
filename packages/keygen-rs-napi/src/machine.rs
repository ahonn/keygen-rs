use std::collections::HashMap;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::machine_file::MachineFile;
use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
pub struct Machine {
    pub id: String,
    pub fingerprint: String,
    pub name: Option<String>,
    pub platform: Option<String>,
    pub hostname: Option<String>,
    pub ip: Option<String>,
    pub cores: Option<i32>,
    pub metadata: Option<serde_json::Value>,
    pub require_heartbeat: bool,
    pub heartbeat_status: String,
    pub heartbeat_duration: Option<i32>,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
    pub environment_id: Option<String>,
    pub product_id: Option<String>,
    pub license_id: Option<String>,
    pub owner_id: Option<String>,
    pub group_id: Option<String>,
}

impl From<keygen_rs::machine::Machine> for Machine {
    fn from(m: keygen_rs::machine::Machine) -> Self {
        Machine {
            id: m.id,
            fingerprint: m.fingerprint,
            name: m.name,
            platform: m.platform,
            hostname: m.hostname,
            ip: m.ip,
            cores: m.cores,
            metadata: m
                .metadata
                .map(|meta| serde_json::to_value(meta).unwrap_or_default()),
            require_heartbeat: m.require_heartbeat,
            heartbeat_status: m.heartbeat_status,
            heartbeat_duration: m.heartbeat_duration,
            created: m.created.to_rfc3339(),
            updated: m.updated.to_rfc3339(),
            account_id: m.account_id,
            environment_id: m.environment_id,
            product_id: m.product_id,
            license_id: m.license_id,
            owner_id: m.owner_id,
            group_id: m.group_id,
        }
    }
}

fn make_machine(id: String) -> keygen_rs::machine::Machine {
    keygen_rs::machine::Machine {
        id,
        fingerprint: String::new(),
        name: None,
        platform: None,
        hostname: None,
        ip: None,
        cores: None,
        metadata: None,
        require_heartbeat: false,
        heartbeat_status: String::new(),
        heartbeat_duration: None,
        created: chrono::Utc::now(),
        updated: chrono::Utc::now(),
        account_id: None,
        environment_id: None,
        product_id: None,
        license_id: None,
        owner_id: None,
        group_id: None,
        config: None,
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct MachineCreateRequest {
    pub fingerprint: String,
    pub license_id: String,
    pub name: Option<String>,
    pub platform: Option<String>,
    pub hostname: Option<String>,
    pub ip: Option<String>,
    pub cores: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct MachineUpdateRequest {
    pub name: Option<String>,
    pub platform: Option<String>,
    pub hostname: Option<String>,
    pub ip: Option<String>,
    pub cores: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct MachineCheckoutOpts {
    pub ttl: Option<i64>,
    pub include: Option<Vec<String>>,
}

#[napi(object)]
#[derive(Clone)]
pub struct ListMachinesOptions {
    pub limit: Option<i32>,
    pub page_size: Option<i32>,
    pub page_number: Option<i32>,
    pub license: Option<String>,
    pub user: Option<String>,
    pub platform: Option<String>,
    pub name: Option<String>,
    pub fingerprint: Option<String>,
    pub ip: Option<String>,
    pub hostname: Option<String>,
    pub product: Option<String>,
    pub owner: Option<String>,
    pub group: Option<String>,
    pub policy: Option<String>,
    pub key: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[napi]
pub async fn create_machine(request: MachineCreateRequest) -> Result<Machine> {
    let mut metadata_map: Option<HashMap<String, serde_json::Value>> = None;
    if let Some(meta) = request.metadata {
        if let Ok(map) = serde_json::from_value::<HashMap<String, serde_json::Value>>(meta) {
            metadata_map = Some(map);
        }
    }

    let req = keygen_rs::machine::MachineCreateRequest {
        fingerprint: request.fingerprint,
        license_id: request.license_id,
        name: request.name,
        platform: request.platform,
        hostname: request.hostname,
        ip: request.ip,
        cores: request.cores,
        metadata: metadata_map,
    };

    keygen_rs::machine::Machine::create(req)
        .await
        .map(Machine::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn list_machines(options: Option<ListMachinesOptions>) -> Result<Vec<Machine>> {
    let filters = options.map(|o| {
        let metadata_map = o.metadata.and_then(|meta| {
            serde_json::from_value::<HashMap<String, serde_json::Value>>(meta).ok()
        });

        keygen_rs::machine::MachineListFilters {
            license: o.license,
            user: o.user,
            platform: o.platform,
            name: o.name,
            fingerprint: o.fingerprint,
            ip: o.ip,
            hostname: o.hostname,
            product: o.product,
            owner: o.owner,
            group: o.group,
            policy: o.policy,
            key: o.key,
            metadata: metadata_map,
            page_number: o.page_number,
            page_size: o.page_size,
            limit: o.limit,
        }
    });

    keygen_rs::machine::Machine::list(filters)
        .await
        .map(|machines| machines.into_iter().map(Machine::from).collect())
        .map_err(to_napi_error)
}

#[napi]
pub async fn get_machine(id: String) -> Result<Machine> {
    keygen_rs::machine::Machine::get(&id)
        .await
        .map(Machine::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn update_machine(id: String, request: MachineUpdateRequest) -> Result<Machine> {
    let machine = make_machine(id);

    let mut metadata_map: Option<HashMap<String, serde_json::Value>> = None;
    if let Some(meta) = request.metadata {
        if let Ok(map) = serde_json::from_value::<HashMap<String, serde_json::Value>>(meta) {
            metadata_map = Some(map);
        }
    }

    let req = keygen_rs::machine::MachineUpdateRequest {
        name: request.name,
        platform: request.platform,
        hostname: request.hostname,
        ip: request.ip,
        cores: request.cores,
        metadata: metadata_map,
    };

    machine
        .update(req)
        .await
        .map(Machine::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn deactivate_machine(id: String) -> Result<()> {
    let machine = make_machine(id);
    machine.deactivate().await.map_err(to_napi_error)
}

#[napi]
pub async fn checkout_machine(
    id: String,
    opts: Option<MachineCheckoutOpts>,
) -> Result<MachineFile> {
    let machine = make_machine(id);
    let checkout_opts = keygen_rs::machine::MachineCheckoutOpts {
        ttl: opts.as_ref().and_then(|o| o.ttl),
        include: opts.and_then(|o| o.include),
    };

    machine
        .checkout(&checkout_opts)
        .await
        .map(MachineFile::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn ping_machine(id: String) -> Result<Machine> {
    let machine = make_machine(id);
    machine
        .ping()
        .await
        .map(Machine::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn reset_machine(id: String) -> Result<Machine> {
    let machine = make_machine(id);
    machine
        .reset()
        .await
        .map(Machine::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn change_machine_owner(id: String, owner_id: String) -> Result<Machine> {
    let machine = make_machine(id);
    machine
        .change_owner(&owner_id)
        .await
        .map(Machine::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn change_machine_group(id: String, group_id: String) -> Result<Machine> {
    let machine = make_machine(id);
    machine
        .change_group(&group_id)
        .await
        .map(Machine::from)
        .map_err(to_napi_error)
}
