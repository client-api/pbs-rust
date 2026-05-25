// SC-01 — /version on PBS.

mod common;

use clientapi_pbs::apis::version_api;
use common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sc_01_version_returns_expected_shape() {
    let creds = Credentials::from_env();
    let cfg = creds.config_with_token();

    let resp = version_api::version_get_version(&cfg)
        .await
        .expect("GET /version");

    // PBS encodes the product version in the `version` field as "4.x";
    // `release` is the patchlevel ("0", "1", ...). Different convention
    // from PVE, where `release` carries the dotted version.
    assert!(
        resp.data.version.starts_with('4'),
        "expected version 4.x, got {:?}",
        resp.data.version
    );
    assert!(
        !resp.data.release.is_empty(),
        "release field must be non-empty"
    );
}
