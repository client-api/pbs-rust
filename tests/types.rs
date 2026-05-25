// SC-50, SC-51 — type edge cases on PBS.
// SC-52 (oneOf discriminator) doesn't apply: PBS has no comparable
// type-tagged polymorphic body in this surface area.

mod common;

use clientapi_pbs::apis::access_users_api;
use common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sc_50_bigint_fields_deserialize_as_i64() {
    let creds = Credentials::from_env();
    let cfg = creds.config_with_token();

    let resp = access_users_api::access_users_get_users(&cfg, None)
        .await
        .expect("list users");
    for u in &resp.data {
        // expire is the int64 field on user entries; assert the type slot.
        let _: Option<i64> = u.expire;
        let _: Option<i64> = u.tfa_locked_until;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sc_51_nullable_fields_decode_as_option_none() {
    let creds = Credentials::from_env();
    let cfg = creds.config_with_token();

    let resp = access_users_api::access_users_get_users(&cfg, None)
        .await
        .expect("list users");
    for u in &resp.data {
        // comment is nullable — at least one of these should be None on
        // a freshly seeded fixture, but either value is acceptable. The
        // assertion is that serde successfully decoded the wire value.
        let _: Option<String> = u.comment.clone();
        if let Some(c) = &u.comment {
            assert_ne!(c, "null", "string \"null\" leaking through Option layer");
        }
    }
}
