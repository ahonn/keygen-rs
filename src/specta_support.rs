//! Type-level shims used by the `specta` feature.
//!
//! specta inlines `serde_json::Value`, whose `Array` variant recurses
//! (`Value -> Vec<Value> -> Value`), so any type with a `metadata` field fails to
//! export. [`KeygenJsonValue`] is a *named* equivalent — a named type may refer to
//! itself — using [`f64`] for numbers, since specta-typescript rejects `i64`/`u64`.
//! The prefix avoids colliding with the `JsonValue` older specta exports natively.
//!
//! Substitution is type-level only: fields stay `HashMap<String, serde_json::Value>`
//! and the wire format is unchanged.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An arbitrary JSON value, standing in for [`serde_json::Value`].
// Kept short: specta copies doc comments into generated bindings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(untagged)]
pub enum KeygenJsonValue {
    // `()` renders as JSON `null`; a unit variant would render as the string "Null".
    Null(()),
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<KeygenJsonValue>),
    Object(HashMap<String, KeygenJsonValue>),
}

/// The `metadata` shape on most Keygen resources.
pub type JsonMetadata = HashMap<String, KeygenJsonValue>;

#[cfg(test)]
mod tests {
    //! Covers the whole `specta` surface, not just this module. A derive can
    //! compile and still fail to export, so these run a real export.

    use super::{JsonMetadata, KeygenJsonValue};
    use crate::component::Component;
    use crate::entitlement::Entitlement;
    use crate::group::Group;
    use crate::license::{License, LicenseStatus, SchemeCode};
    use crate::license_file::{IncludedResources, LicenseFile, LicenseFileDataset};
    use crate::machine::{HeartbeatStatus, Machine};
    use crate::machine_file::{MachineFile, MachineFileDataset};
    use crate::service::{PingResponse, ServiceInfo};
    use specta::Types;
    use specta_typescript::Typescript;

    /// Every type the `specta` feature is expected to cover.
    fn end_user_types() -> Types {
        let mut t = Types::default();

        t.register_mut::<Machine>();
        t.register_mut::<HeartbeatStatus>();

        t.register_mut::<License>();
        t.register_mut::<LicenseStatus>();
        t.register_mut::<SchemeCode>();

        t.register_mut::<LicenseFile>();
        t.register_mut::<LicenseFileDataset>();
        t.register_mut::<IncludedResources>();
        t.register_mut::<MachineFile>();
        t.register_mut::<MachineFileDataset>();

        t.register_mut::<Entitlement>();
        t.register_mut::<Component>();
        t.register_mut::<Group>();

        t.register_mut::<PingResponse>();
        t.register_mut::<ServiceInfo>();

        t.register_mut::<KeygenJsonValue>();
        t.register_mut::<JsonMetadata>();

        t
    }

    fn export() -> String {
        Typescript::default()
            .export(&end_user_types(), specta_serde::Format)
            .expect("the end-user surface should export to TypeScript")
    }

    #[test]
    fn end_user_surface_exports_to_typescript() {
        let ts = export();

        // One assertion per registered type, so a dropped derive names itself.
        for ty in [
            "Machine",
            "HeartbeatStatus",
            "License",
            "LicenseStatus",
            "SchemeCode",
            "LicenseFile",
            "LicenseFileDataset",
            "IncludedResources",
            "MachineFile",
            "MachineFileDataset",
            "Entitlement",
            "Component",
            "Group",
            "PingResponse",
            "ServiceInfo",
            "KeygenJsonValue",
        ] {
            assert!(
                ts.contains(&format!("export type {ty} =")),
                "{ty} is missing from the exported bindings:\n{ts}"
            );
        }
    }

    /// `KeygenConfig` does not implement `Type` — its fields are `#[cfg]`-gated —
    /// so the `#[serde(skip)]` on `config` is what keeps it out of the graph.
    #[test]
    fn skipped_config_field_is_absent_from_the_bindings() {
        let ts = export();
        assert!(
            !ts.contains("KeygenConfig") && !ts.contains("config"),
            "the skipped config field leaked into the bindings:\n{ts}"
        );
    }

    /// Guards the `#[specta(type = ...)]` override on every `metadata` field;
    /// without it the export fails outright.
    #[test]
    fn metadata_fields_use_the_named_json_shim() {
        let ts = export();

        assert!(
            ts.contains("export type KeygenJsonValue = null | boolean | number"),
            "KeygenJsonValue should render as a JSON union, not a tagged enum:\n{ts}"
        );
        assert!(
            ts.contains("KeygenJsonValue[]"),
            "KeygenJsonValue should stay recursive through a named reference:\n{ts}"
        );

        let metadata_lines: Vec<&str> = ts
            .lines()
            .filter(|l| l.trim_start().starts_with("metadata:"))
            .collect();
        assert_eq!(
            metadata_lines.len(),
            5,
            "expected metadata on Machine, License, Component, Entitlement and Group, got: {metadata_lines:?}"
        );
        for line in metadata_lines {
            assert!(
                line.contains("KeygenJsonValue"),
                "a metadata field bypassed the KeygenJsonValue override: {line}"
            );
        }
    }

    /// If a rename is dropped the frontend reads fields that are never populated.
    #[test]
    fn machine_serde_renames_reach_the_bindings() {
        let ts = export();
        for field in ["requireHeartbeat", "heartbeatStatus", "heartbeatDuration"] {
            assert!(
                ts.contains(field),
                "{field} missing from bindings — serde rename was dropped:\n{ts}"
            );
        }
        assert!(
            !ts.contains("require_heartbeat"),
            "snake_case leaked past the serde rename:\n{ts}"
        );
    }
}
