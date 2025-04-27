const COMMANDS: &[&str] = &[
    "get_license",
    "is_license_valid",
    "get_license_key",
    "validate_key",
    "activate",
    "deactivate",
    "checkout_license",
    "checkout_machine",
    "reset_license",
    "get_license_metadata",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
