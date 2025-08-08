use std::{
    env,
    sync::{mpsc, Arc},
    thread,
    time::Duration,
};

use dotenv::dotenv;

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

        let (tx, rx) = mpsc::channel();
        let (cancel_tx, cancel_rx) = mpsc::channel();
        let monitor_future = machine_arc
            .clone()
            .monitor(interval, Some(tx), Some(cancel_rx));

        let monitor_futures = tokio::spawn(async move {
            monitor_future.await;
        });

        // Spawn a new thread to handle message receiving and cancellation
        let message_handler = thread::spawn(move || {
            // Set a timer to cancel the monitor after 100 seconds
            let cancel_timer = std::time::Instant::now() + Duration::from_secs(100);

            loop {
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(message) => println!("Received message: {message:?}"),
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if std::time::Instant::now() >= cancel_timer {
                            println!("Timer expired, sending cancel signal...");
                            cancel_tx.send(()).unwrap();
                            break;
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        println!("Channel disconnected, exiting...");
                        break;
                    }
                }
            }
        });

        // Wait for the message handler to complete
        message_handler.join().unwrap();
        monitor_futures.await.unwrap();
    } else {
        // License not activated, run activate_machine.rs first
        println!("License not activated");
    }
    Ok(())
}
