//! Example: open a terminal session against a QEMU VM.
//!
//! Run with:
//!
//! ```sh
//! PBS_HOST=https://pbs.example.com:8007 \
//! PBS_TOKEN='root@pam!auto=...' \
//! PBS_NODE=orca PBS_VMID=100 \
//! cargo run --example terminal --features extras
//! ```

use std::env;
use std::time::Duration;

use clientapi_pbs::apis::configuration::Configuration;
use clientapi_pbs::websocket::{connect_terminal, TerminalTarget};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = Configuration::new();
    cfg.base_path = format!(
        "{}/api2/json",
        env::var("PBS_HOST").unwrap_or_else(|_| "https://localhost:8007".into()),
    );
    cfg.bearer_access_token = env::var("PBS_TOKEN").ok();

    let node = env::var("PBS_NODE").unwrap_or_else(|_| "pbs1".into());
    let vmid: i32 = env::var("PBS_VMID")
        .unwrap_or_else(|_| "100".into())
        .parse()?;

    println!("Opening terminal on {}:qemu/{}...", node, vmid);
    let mut session = connect_terminal(&cfg, TerminalTarget::Qemu { node, vmid }).await?;

    session.resize(120, 32).await?;
    session.send("uname -a\n").await?;

    // Read with a 5 s overall timeout.
    let _ = tokio::time::timeout(Duration::from_secs(5), async {
        while let Ok(Some(msg)) = session.recv().await {
            print!("{msg}");
        }
    })
    .await;

    session.close().await?;
    Ok(())
}
