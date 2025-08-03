use dotenv::dotenv;
use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    machine::{Machine, MachineListFilters},
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    // Set up configuration with Admin Token
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").expect("KEYGEN_API_URL must be set"),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    })?;

    // Example 1: List all machines without pagination
    match Machine::list(None).await {
        Ok(machines) => {
            println!("Found {} machines", machines.len());
        }
        Err(e) => {
            println!("Failed to list machines: {e:?}");
        }
    }

    println!("\n---\n");

    // Example 2: List machines with limit
    let filters_with_limit = MachineListFilters {
        limit: Some(25),
        ..Default::default()
    };
    match Machine::list(Some(filters_with_limit)).await {
        Ok(machines) => {
            println!("Found {} machines (limited to 25)", machines.len());
        }
        Err(e) => {
            println!("Failed to list machines: {e:?}");
        }
    }

    println!("\n---\n");

    // Example 3: List machines with pagination (page 1, 10 per page)
    let filters_with_pagination = MachineListFilters {
        page_number: Some(1),
        page_size: Some(10),
        ..Default::default()
    };
    match Machine::list(Some(filters_with_pagination)).await {
        Ok(machines) => {
            println!("Found {} machines on page 1:", machines.len());
            for machine in machines {
                println!("  ID: {}", machine.id);
                println!("  Fingerprint: {}", machine.fingerprint);
                println!("  Name: {:?}", machine.name);
                println!("  Platform: {:?}", machine.platform);
                println!("  Hostname: {:?}", machine.hostname);
                println!("  IP: {:?}", machine.ip);
                println!("  Cores: {:?}", machine.cores);
                println!("  Metadata: {:?}", machine.metadata);
                println!("  Require Heartbeat: {}", machine.require_heartbeat);
                println!("  Heartbeat Status: {}", machine.heartbeat_status);
                println!("  Created: {}", machine.created);
                println!("  Relationships:");
                println!("    Account ID: {:?}", machine.account_id);
                println!("    Environment ID: {:?}", machine.environment_id);
                println!("    Product ID: {:?}", machine.product_id);
                println!("    License ID: {:?}", machine.license_id);
                println!("    Owner ID: {:?}", machine.owner_id);
                println!("    Group ID: {:?}", machine.group_id);
                println!("  ---");
            }
        }
        Err(e) => {
            println!("Failed to list machines: {e:?}");
        }
    }

    println!("\n---\n");

    // Example 4: List machines with filters and pagination
    let filters_combined = MachineListFilters {
        platform: Some("linux".to_string()),
        page_number: Some(1),
        page_size: Some(5),
        ..Default::default()
    };
    match Machine::list(Some(filters_combined)).await {
        Ok(machines) => {
            println!("Found {} Linux machines on page 1:", machines.len());
            for machine in machines {
                println!("  - {} (Platform: {:?})", machine.id, machine.platform);
            }
        }
        Err(e) => {
            println!("Failed to list machines: {e:?}");
        }
    }

    Ok(())
}
