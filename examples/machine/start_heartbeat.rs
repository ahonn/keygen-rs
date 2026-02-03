use std::{env, sync::Arc, time::Duration};

use dotenv::dotenv;
use tokio::sync::mpsc;

use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").expect("KEYGEN_API_URL must be set"),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        product: env::var("KEYGEN_PRODUCT").expect("KEYGEN_PRODUCT must be set"),
        license_key: Some(env::var("KEYGEN_LICENSE_KEY").expect("KEYGEN_LICENSE_KEY must be set")),
        public_key: Some(env::var("KEYGEN_PUBLIC_KEY").expect("KEYGEN_PUBLIC_KEY must be set")),
        ..KeygenConfig::default()
    })?;

    let fingerprint = machine_uid::get().unwrap_or("".into());
    if let Ok(license) = keygen_rs::validate(std::slice::from_ref(&fingerprint), &[]).await {
        let machine = license.machine(&fingerprint).await?;
        // Set the interval to 30 seconds less than the heartbeat duration to ensure we don't miss a heartbeat
        let interval = Duration::from_secs(machine.heartbeat_duration.unwrap_or(600) as u64 - 30);
        let machine_arc = Arc::new(machine);

        let (tx, mut rx) = mpsc::channel(32);
        let (cancel_tx, cancel_rx) = mpsc::channel(1);
        let monitor_future = machine_arc
            .clone()
            .monitor(interval, Some(tx), Some(cancel_rx));

        let monitor_handle = tokio::spawn(async move {
            monitor_future.await;
        });

        // Spawn a task to handle message receiving and cancellation
        let message_handler = tokio::spawn(async move {
            let cancel_deadline = tokio::time::Instant::now() + Duration::from_secs(100);
            let mut cancel_interval = tokio::time::interval(Duration::from_millis(100));

            loop {
                tokio::select! {
                    Some(message) = rx.recv() => {
                        println!("Received message: {message:?}");
                    }
                    _ = cancel_interval.tick() => {
                        if tokio::time::Instant::now() >= cancel_deadline {
                            println!("Timer expired, sending cancel signal...");
                            let _ = cancel_tx.send(()).await;
                            break;
                        }
                    }
                }
            }
        });

        // Wait for both tasks to complete
        let _ = tokio::join!(message_handler, monitor_handle);
    } else {
        // License not activated, run activate_machine.rs first
        println!("License not activated");
    }
    Ok(())
}
