use std::{
    sync::{mpsc, Arc},
    thread,
    time::Duration,
};

use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").expect("KEYGEN_API_URL must be set"),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        product: env::var("KEYGEN_PRODUCT").expect("KEYGEN_PRODUCT must be set"),
        license_key: Some(env::var("KEYGEN_LICENSE_KEY").expect("KEYGEN_LICENSE_KEY must be set")),
        public_key: Some(env::var("KEYGEN_PUBLIC_KEY").expect("KEYGEN_PUBLIC_KEY must be set")),
        ..KeygenConfig::default()
    });
    let fingerprint = machine_uid::get().unwrap_or("".into());
    if let Ok(license) = keygen_rs::validate(&[fingerprint.clone()], &[]).await {
        let machine = license.machine(&fingerprint).await?;
        let interval = Duration::from_secs(machine.heartbeat_duration.unwrap_or(570) as u64);

        let machine_arc = Arc::new(machine);

        let (tx, rx) = mpsc::channel();

        let monitor_future = machine_arc.clone().monitor(interval, Some(tx));

        tokio::spawn(async move {
            monitor_future.await;
        });

        loop {
            // Keep this thread alive and monitor for received errors
            thread::sleep(Duration::from_secs(10));
            println!("{}", rx.recv().unwrap());
        }
    }

    Ok(())
}
