//! Example: list cluster nodes.
//!
//! Run with:
//!
//! ```sh
//! PBS_HOST=https://pbs.example.com:8007 \
//! PBS_TOKEN='root@pam!auto:...' \
//! cargo run --example list_nodes
//! ```

use clientapi_pbs::apis::configuration::Configuration;
use clientapi_pbs::apis::nodes_api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = Configuration::new();
    cfg.base_path = format!(
        "{}/api2/json",
        std::env::var("PBS_HOST").unwrap_or_else(|_| "https://localhost:8007".into()),
    );
    cfg.bearer_access_token = std::env::var("PBS_TOKEN").ok();

    let resp = nodes_api::nodes_get_nodes(&cfg).await?;
    // Non-PVE products: the upstream apidoc.js declares this endpoint
    // `returns: { type: null }`, so the generator emits `data` as an
    // untyped `serde_json::Value` (or similar). Print whatever came back.
    println!("Response: {:?}", resp);
    Ok(())
}
