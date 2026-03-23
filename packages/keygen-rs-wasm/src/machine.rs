use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::machine_file::MachineFile;
use crate::to_js_error;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[wasm_bindgen(js_name = "createMachine")]
pub async fn create_machine(request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        fingerprint: String,
        license_id: String,
        name: Option<String>,
        platform: Option<String>,
        hostname: Option<String>,
        ip: Option<String>,
        cores: Option<i32>,
        metadata: Option<serde_json::Value>,
    }
    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;

    let mut metadata_map: Option<HashMap<String, serde_json::Value>> = None;
    if let Some(meta) = req.metadata {
        if let Ok(map) = serde_json::from_value::<HashMap<String, serde_json::Value>>(meta) {
            metadata_map = Some(map);
        }
    }

    let r = keygen_rs::machine::MachineCreateRequest {
        fingerprint: req.fingerprint,
        license_id: req.license_id,
        name: req.name,
        platform: req.platform,
        hostname: req.hostname,
        ip: req.ip,
        cores: req.cores,
        metadata: metadata_map,
    };

    let machine = keygen_rs::machine::Machine::create(r)
        .await
        .map(Machine::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&machine).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "listMachines")]
pub async fn list_machines(options: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct Opts {
        limit: Option<i32>,
        page_size: Option<i32>,
        page_number: Option<i32>,
        license: Option<String>,
        user: Option<String>,
        platform: Option<String>,
        name: Option<String>,
        fingerprint: Option<String>,
        ip: Option<String>,
        hostname: Option<String>,
        product: Option<String>,
        owner: Option<String>,
        group: Option<String>,
        metadata: Option<serde_json::Value>,
    }
    let opts: Option<Opts> = if options.is_undefined() || options.is_null() {
        None
    } else {
        Some(serde_wasm_bindgen::from_value(options).map_err(|e| JsError::new(&e.to_string()))?)
    };

    let filters = opts.map(|o| {
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
            metadata: metadata_map,
            page_number: o.page_number,
            page_size: o.page_size,
            limit: o.limit,
        }
    });

    let machines: Vec<Machine> = keygen_rs::machine::Machine::list(filters)
        .await
        .map(|list| list.into_iter().map(Machine::from).collect())
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&machines).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "getMachine")]
pub async fn get_machine(id: String) -> Result<JsValue, JsError> {
    let machine = keygen_rs::machine::Machine::get(&id)
        .await
        .map(Machine::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&machine).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "updateMachine")]
pub async fn update_machine(id: String, request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        name: Option<String>,
        platform: Option<String>,
        hostname: Option<String>,
        ip: Option<String>,
        cores: Option<i32>,
        metadata: Option<serde_json::Value>,
    }
    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;
    let m = make_machine(id);

    let mut metadata_map: Option<HashMap<String, serde_json::Value>> = None;
    if let Some(meta) = req.metadata {
        if let Ok(map) = serde_json::from_value::<HashMap<String, serde_json::Value>>(meta) {
            metadata_map = Some(map);
        }
    }

    let r = keygen_rs::machine::MachineUpdateRequest {
        name: req.name,
        platform: req.platform,
        hostname: req.hostname,
        ip: req.ip,
        cores: req.cores,
        metadata: metadata_map,
    };

    let machine = m.update(r).await.map(Machine::from).map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&machine).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "deactivateMachine")]
pub async fn deactivate_machine(id: String) -> Result<(), JsError> {
    let machine = make_machine(id);
    machine.deactivate().await.map_err(to_js_error)
}

#[wasm_bindgen(js_name = "checkoutMachine")]
pub async fn checkout_machine(id: String, opts: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct Opts {
        ttl: Option<i64>,
        include: Option<Vec<String>>,
    }
    let opts: Option<Opts> = if opts.is_undefined() || opts.is_null() {
        None
    } else {
        Some(serde_wasm_bindgen::from_value(opts).map_err(|e| JsError::new(&e.to_string()))?)
    };
    let machine = make_machine(id);
    let checkout_opts = keygen_rs::machine::MachineCheckoutOpts {
        ttl: opts.as_ref().and_then(|o| o.ttl),
        include: opts.and_then(|o| o.include),
    };
    let mf = machine
        .checkout(&checkout_opts)
        .await
        .map(MachineFile::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&mf).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "pingMachine")]
pub async fn ping_machine(id: String) -> Result<JsValue, JsError> {
    let machine = make_machine(id);
    let m = machine
        .ping()
        .await
        .map(Machine::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&m).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "resetMachine")]
pub async fn reset_machine(id: String) -> Result<JsValue, JsError> {
    let machine = make_machine(id);
    let m = machine
        .reset()
        .await
        .map(Machine::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&m).map_err(|e| JsError::new(&e.to_string()))
}
