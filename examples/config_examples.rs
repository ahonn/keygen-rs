use keygen_rs::config::{self, KeygenConfig};

#[tokio::main]
async fn main() {
    println!("🔧 Keygen Configuration Examples");
    println!("================================\n");

    // Example 1: Client Configuration (for end users)
    #[cfg(feature = "client")]
    {
        println!("👤 Client Configuration:");
        let client_config = KeygenConfig::client(
            "your-account-id".to_string(),
            "your-product-id".to_string(),
            "your-license-key".to_string(),
            "your-public-key".to_string(),
        );
        config::set_config(client_config);
        
        println!("  ✅ Account: set");
        println!("  ✅ Product: set");
        println!("  ✅ License Key: set");
        println!("  ✅ Public Key: set");
        println!("  ✅ Max Clock Drift: {} seconds", config::get_config().max_clock_drift.unwrap_or(5));
        println!();
    }

    // Example 2: Admin Configuration (for administrators)
    #[cfg(feature = "admin")]
    {
        println!("🔧 Admin Configuration:");
        let admin_config = KeygenConfig::admin(
            "your-account-id".to_string(),
            "your-admin-token".to_string(),
        );
        config::set_config(admin_config);
        
        println!("  ✅ Account: set");
        println!("  ✅ Admin Token: set");
        println!("  ✅ API URL: {}", config::get_config().api_url);
        println!();
    }

    // Example 3: Hybrid Configuration (when both features are enabled)
    #[cfg(all(feature = "client", feature = "admin"))]
    {
        println!("🔄 Hybrid Configuration:");
        let hybrid_config = KeygenConfig::hybrid(
            "your-account-id".to_string(),
            "your-product-id".to_string(),
            Some("your-license-key".to_string()),
            Some("your-admin-token".to_string()),
        );
        config::set_config(hybrid_config);
        
        println!("  ✅ Account: set");
        println!("  ✅ Product: set");
        println!("  ✅ License Key: set");
        println!("  ✅ Admin Token: set");
        println!();
    }

    // Example 4: Manual Configuration
    println!("⚙️  Manual Configuration:");
    config::set_config(KeygenConfig::default());
    config::set_account("your-account-id");
    config::set_api_url("https://api.keygen.sh");
    
    #[cfg(feature = "client")]
    {
        config::set_product("your-product-id");
        config::set_license_key("your-license-key");
        config::set_public_key("your-public-key");
        config::set_max_clock_drift(10);
        println!("  ✅ Client fields configured");
    }
    
    #[cfg(feature = "admin")]
    {
        config::set_token("your-admin-token");
        println!("  ✅ Admin fields configured");
    }
    
    println!("  ✅ Manual configuration complete");
    println!();

    // Example 5: Environment-based Configuration
    println!("🌍 Environment-based Configuration:");
    println!("  Set these environment variables:");
    println!("  📋 Common:");
    println!("    KEYGEN_ACCOUNT=your-account-id");
    println!("    KEYGEN_API_URL=https://api.keygen.sh");
    
    #[cfg(feature = "client")]
    println!("  👤 Client:");
    #[cfg(feature = "client")]
    println!("    KEYGEN_PRODUCT=your-product-id");
    #[cfg(feature = "client")]
    println!("    KEYGEN_LICENSE_KEY=your-license-key");
    #[cfg(feature = "client")]
    println!("    KEYGEN_PUBLIC_KEY=your-public-key");
    
    #[cfg(feature = "admin")]
    println!("  🔧 Admin:");
    #[cfg(feature = "admin")]
    println!("    KEYGEN_ADMIN_TOKEN=your-admin-token");
    
    println!();
    println!("💡 Feature Flags Used:");
    #[cfg(feature = "client")]
    println!("  ✅ client - End user functionality enabled");
    #[cfg(not(feature = "client"))]
    println!("  ❌ client - End user functionality disabled");
    
    #[cfg(feature = "admin")]
    println!("  ✅ admin - Administrator functionality enabled");
    #[cfg(not(feature = "admin"))]
    println!("  ❌ admin - Administrator functionality disabled");
}