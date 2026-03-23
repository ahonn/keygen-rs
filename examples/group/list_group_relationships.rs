#[path = "../common/mod.rs"]
mod common;

use keygen_rs::{
    errors::Error,
    group::{CreateGroupRequest, Group},
    license::{License, LicenseCreateRequest, PaginationOptions},
    machine::{Machine, MachineCreateRequest},
    user::{self, CreateUserRequest, UserRole},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    common::load_env();
    common::configure_admin()?;

    let policy_id = common::required_env("KEYGEN_POLICY_ID");
    let suffix = common::unique_suffix();
    let mut created_group: Option<Group> = None;
    let mut created_license: Option<License> = None;
    let mut created_machine: Option<Machine> = None;
    let mut created_user_id: Option<String> = None;

    let result = async {
        let group = match Group::list(None).await?.into_iter().next() {
            Some(group) => {
                println!("Using existing group: {} ({})", group.name, group.id);
                group
            }
            None => {
                let group = Group::create(CreateGroupRequest {
                    name: format!("example-group-relationships-{suffix}"),
                    max_users: Some(5),
                    max_licenses: Some(5),
                    max_machines: Some(5),
                    metadata: None,
                })
                .await?;
                println!("Created temp group: {} ({})", group.name, group.id);
                created_group = Some(group.clone());

                let user = user::create(CreateUserRequest {
                    email: format!("group-example-{suffix}@example.com"),
                    first_name: Some("Group".to_string()),
                    last_name: Some("Example".to_string()),
                    role: Some(UserRole::User),
                    permissions: None,
                    metadata: None,
                })
                .await?;
                created_user_id = Some(user.id.clone());
                println!("Created temp user: {}", user.id);

                let user = user::change_group(&user.id, &group.id).await?;
                println!("Assigned temp user {} to temp group", user.id);

                let license = License::create(
                    LicenseCreateRequest::new(policy_id.clone())
                        .with_name(format!("Example Group License {suffix}"))
                        .with_group_id(group.id.clone()),
                )
                .await?;
                created_license = Some(license.clone());
                println!("Created temp license: {}", license.id);

                let machine = Machine::create(MachineCreateRequest {
                    fingerprint: format!("example-group-machine-{suffix}"),
                    name: Some(format!("Example Group Machine {suffix}")),
                    platform: Some("linux/x86_64".to_string()),
                    hostname: Some(format!("group-host-{suffix}")),
                    ip: None,
                    cores: Some(2),
                    metadata: None,
                    license_id: license.id.clone(),
                })
                .await?;
                let machine = machine.change_group(&group.id).await?;
                created_machine = Some(machine.clone());
                println!("Created and grouped temp machine: {}", machine.id);

                group
            }
        };

        let page = PaginationOptions {
            limit: Some(5),
            page_number: None,
            page_size: None,
        };

        let owners = group.owners(Some(&page)).await?;
        let users = group.users(Some(&page)).await?;
        let licenses = group.licenses(Some(&page)).await?;
        let machines = group.machines(Some(&page)).await?;

        println!("Group: {} ({})", group.name, group.id);
        println!("  Owners: {}", owners.len());
        println!("  Users: {}", users.len());
        println!("  Licenses: {}", licenses.len());
        println!("  Machines: {}", machines.len());

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
        if let Err(err) = user::delete(user_id).await {
            eprintln!("Cleanup: failed to delete temp user {}: {err:?}", user_id);
        }
    }
    if let Some(group) = created_group.as_ref() {
        if let Err(err) = group.delete().await {
            eprintln!("Cleanup: failed to delete temp group {}: {err:?}", group.id);
        }
    }

    result
}
