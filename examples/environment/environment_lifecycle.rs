use keygen_rs::{
    config::{self, KeygenConfig},
    environment::{
        CreateEnvironmentRequest, CreateEnvironmentTokenRequest, Environment, IsolationStrategy,
        ListEnvironmentsOptions, UpdateEnvironmentRequest,
    },
    errors::Error,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();

    // Set up configuration with Admin Token
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    })?;

    println!("🚀 Starting Environment API Lifecycle Demo");
    println!("==========================================\n");

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Step 1: Create a new environment
    println!("1️⃣  Creating a new environment...");
    let create_request = CreateEnvironmentRequest {
        name: format!("Demo Environment {timestamp}"),
        code: format!("demo-{timestamp}"),
        isolation_strategy: Some(IsolationStrategy::Isolated),
    };

    let environment = match Environment::create(create_request).await {
        Ok(env) => {
            println!("   ✅ Environment created: {} ({})", env.name, env.code);
            env
        }
        Err(e) => {
            println!("   ❌ Failed to create environment: {e:?}");
            return Ok(());
        }
    };

    // Step 2: Get the environment by ID
    println!("\n2️⃣  Retrieving environment by ID...");
    match Environment::get(&environment.id).await {
        Ok(retrieved_env) => {
            println!("   ✅ Retrieved environment: {}", retrieved_env.name);
            println!("      - Isolation: {:?}", retrieved_env.isolation_strategy);
        }
        Err(e) => {
            println!("   ❌ Failed to retrieve environment: {e:?}");
        }
    }

    // Step 3: List environments with pagination
    println!("\n3️⃣  Listing environments with pagination...");
    let list_options = ListEnvironmentsOptions {
        limit: Some(5),
        page_size: Some(3),
        page_number: Some(1),
    };

    match Environment::list(Some(list_options)).await {
        Ok(result) => {
            println!("   ✅ Found {} environments", result.environments.len());
            for env in &result.environments[..std::cmp::min(3, result.environments.len())] {
                println!("      - {} ({})", env.name, env.code);
            }
        }
        Err(e) => {
            println!("   ❌ Failed to list environments: {e:?}");
        }
    }

    // Step 4: Generate environment token
    println!("\n4️⃣  Generating environment token...");
    let token_request = CreateEnvironmentTokenRequest {
        name: Some("Demo Token".to_string()),
        expiry: None,
        permissions: Some(vec![
            "environment.read".to_string(),
            "license.read".to_string(),
        ]),
    };

    let token = match environment.generate_token(Some(token_request)).await {
        Ok(token) => {
            println!("   ✅ Token generated: {} chars", token.token.len());
            println!("      - Permissions: {:?}", token.permissions);
            Some(token)
        }
        Err(e) => {
            println!("   ❌ Failed to generate token: {e:?}");
            None
        }
    };

    // Step 5: Update environment
    println!("\n5️⃣  Updating environment name...");
    let update_request = UpdateEnvironmentRequest {
        name: Some(format!("{} (Updated)", environment.name)),
        code: None,
    };

    let updated_environment = match environment.update(update_request).await {
        Ok(updated_env) => {
            println!("   ✅ Environment updated: {}", updated_env.name);
            updated_env
        }
        Err(e) => {
            println!("   ❌ Failed to update environment: {e:?}");
            environment
        }
    };

    // Step 6: Show final environment details
    println!("\n6️⃣  Final environment details:");
    println!("   ID: {}", updated_environment.id);
    println!("   Name: {}", updated_environment.name);
    println!("   Code: {}", updated_environment.code);
    println!("   Isolation: {:?}", updated_environment.isolation_strategy);
    println!("   Created: {}", updated_environment.created);
    println!("   Updated: {}", updated_environment.updated);

    if let Some(token) = token {
        println!("\n🔐 Generated Token Details:");
        println!("   Token ID: {}", token.id);
        println!("   Token Name: {:?}", token.name);
        println!("   Environment ID: {}", token.environment_id);
    }

    // Optional: Clean up (uncomment to delete the created environment)
    /*
    println!("\n7️⃣  Cleaning up - deleting environment...");
    match updated_environment.delete().await {
        Ok(()) => {
            println!("   ✅ Environment deleted successfully");
        }
        Err(e) => {
            println!("   ❌ Failed to delete environment: {e:?}");
        }
    }
    */

    println!("\n✨ Environment API Lifecycle Demo completed!");
    println!(
        "   Environment '{}' is ready to use.",
        updated_environment.code
    );

    Ok(())
}
