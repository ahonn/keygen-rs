#[path = "../common/mod.rs"]
mod common;

use keygen_rs::{
    errors::Error,
    group::{CreateGroupRequest, Group},
    license::{License, LicenseCreateRequest},
    machine::{Machine, MachineCreateRequest, MachineListFilters},
    user::{CreateUserRequest, User, UserRole},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    common::load_env();
    common::configure_admin()?;

    let policy_id = common::required_env("KEYGEN_POLICY_ID");
    let suffix = common::unique_suffix();

    let mut created_group: Option<Group> = None;
    let mut created_user_id: Option<String> = None;
    let mut created_license: Option<License> = None;
    let mut created_machine: Option<Machine> = None;

    let result = async {
        let group = Group::create(CreateGroupRequest {
            name: format!("example-machine-group-{suffix}"),
            max_users: Some(5),
            max_licenses: Some(5),
            max_machines: Some(5),
            metadata: None,
        })
        .await?;
        created_group = Some(group.clone());
        println!("Created temp group: {}", group.id);

        let user = User::create(CreateUserRequest {
            email: format!("machine-example-{suffix}@example.com"),
            first_name: Some("Machine".to_string()),
            last_name: Some("Example".to_string()),
            role: Some(UserRole::User),
            permissions: None,
            metadata: None,
        })
        .await?;
        created_user_id = Some(user.id.clone());
        println!("Created temp user: {}", user.id);

        let license = License::create(
            LicenseCreateRequest::new(policy_id.clone())
                .with_name(format!("Example Machine License {suffix}")),
        )
        .await?;
        created_license = Some(license.clone());
        println!("Created temp license: {} ({})", license.id, license.key);

        license.attach_users(std::slice::from_ref(&user.id)).await?;
        println!("Attached temp user {} to temp license", user.id);

        let machine = Machine::create(MachineCreateRequest {
            fingerprint: format!("example-machine-{suffix}"),
            name: Some(format!("Example Machine {suffix}")),
            platform: Some("linux/x86_64".to_string()),
            hostname: Some(format!("example-host-{suffix}")),
            ip: None,
            cores: Some(4),
            metadata: None,
            license_id: license.id.clone(),
        })
        .await?;
        created_machine = Some(machine.clone());
        println!("Created temp machine: {}", machine.id);

        let machine = machine.change_owner(&user.id).await?;
        println!("Changed machine owner to {}", user.id);

        let machine = machine.change_group(&group.id).await?;
        println!("Changed machine group to {}", group.id);
        created_machine = Some(machine.clone());

        let filtered = Machine::list(Some(MachineListFilters {
            policy: Some(policy_id),
            key: Some(license.key.clone()),
            limit: Some(10),
            ..Default::default()
        }))
        .await?;
        println!(
            "Filtered machines by policy + license key: {}",
            filtered.len()
        );

        Ok(())
    }
    .await;

    if let Some(machine) = created_machine.as_ref() {
        if let Err(err) = machine.deactivate().await {
            eprintln!(
                "Cleanup: failed to deactivate temp machine {}: {err:?}",
                machine.id
            );
        }
    }
    if let Some(license) = created_license.as_ref() {
        if let Err(err) = license.delete().await {
            eprintln!(
                "Cleanup: failed to delete temp license {}: {err:?}",
                license.id
            );
        }
    }
    if let Some(user_id) = created_user_id.as_ref() {
        if let Ok(user) = User::get(user_id).await {
            if let Err(err) = user.delete().await {
                eprintln!("Cleanup: failed to delete temp user {}: {err:?}", user_id);
            }
        }
    }
    if let Some(group) = created_group.as_ref() {
        if let Err(err) = group.delete().await {
            eprintln!("Cleanup: failed to delete temp group {}: {err:?}", group.id);
        }
    }

    result
}
