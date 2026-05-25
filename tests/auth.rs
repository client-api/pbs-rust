// SC-10..14 — authentication & CSRF on PBS.

mod common;

use clientapi_pbs::apis::configuration::ApiKey;
use clientapi_pbs::apis::{access_ticket_api, access_users_api, version_api, Error};
use clientapi_pbs::models::{AccessTicketCreateTicketRequest, AccessUsersCreateUsersRequest};
use common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sc_10_ticket_auth_returns_ticket_and_csrf() {
    let creds = Credentials::from_env();
    let cfg = creds.config_anonymous();

    let mut req = AccessTicketCreateTicketRequest::new(creds.user.clone());
    req.password = Some(creds.password.clone());
    let resp = access_ticket_api::access_ticket_create_ticket(&cfg, req)
        .await
        .expect("POST /access/ticket");

    let ticket = resp.data.ticket.as_deref().expect("ticket field present");
    let csrf = resp
        .data
        .csrf_prevention_token
        .as_deref()
        .expect("csrf_prevention_token field present");

    assert!(ticket.starts_with("PBS:"), "ticket prefix: {ticket}");
    assert!(!csrf.is_empty(), "csrf must be non-empty");

    let ticket_cfg = creds.config_with_ticket(ticket, csrf);
    version_api::version_get_version(&ticket_cfg)
        .await
        .expect("authenticated /version with ticket");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sc_11_ticket_auth_rejects_bad_password() {
    let creds = Credentials::from_env();
    let cfg = creds.config_anonymous();

    let mut req = AccessTicketCreateTicketRequest::new(creds.user.clone());
    req.password = Some("definitely-not-the-password".to_string());
    let err = access_ticket_api::access_ticket_create_ticket(&cfg, req)
        .await
        .expect_err("bad password must fail");
    assert_status(&err, 401);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sc_12_token_auth_returns_200() {
    let creds = Credentials::from_env();
    skip_if_pmg!(creds);

    let cfg = creds.config_with_token();
    version_api::version_get_version(&cfg)
        .await
        .expect("token auth /version");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sc_13_token_auth_rejects_malformed_token() {
    let creds = Credentials::from_env();
    skip_if_pmg!(creds);

    let mut cfg = creds.config_anonymous();
    cfg.api_key = Some(ApiKey {
        prefix: None,
        // PBS family uses `:` as the separator between tokenid and uuid.
        key: "PBSAPIToken=root@pam!bogus:00000000-0000-0000-0000-000000000000".to_string(),
    });

    let err = version_api::version_get_version(&cfg)
        .await
        .expect_err("malformed token must fail");
    assert_status(&err, 401);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sc_14_state_change_without_csrf_returns_401() {
    let creds = Credentials::from_env();

    let mut login_req = AccessTicketCreateTicketRequest::new(creds.user.clone());
    login_req.password = Some(creds.password.clone());
    let login = access_ticket_api::access_ticket_create_ticket(
        &creds.config_anonymous(),
        login_req,
    )
    .await
    .expect("login");
    let ticket = login.data.ticket.as_deref().expect("ticket");

    let cfg = creds.config_with_ticket_no_csrf(ticket);

    let _ = access_users_api::access_users_delete_users(
        &creds.config_with_token(),
        "e2e-csrf-probe@pbs",
        None,
    )
    .await;

    let req = AccessUsersCreateUsersRequest::new("e2e-csrf-probe@pbs".to_string());
    let err = access_users_api::access_users_create_users(&cfg, req)
        .await
        .expect_err("create_users without CSRF must fail");
    assert_status(&err, 401);
}

fn assert_status<T: std::fmt::Debug>(err: &Error<T>, expected: u16) {
    match err {
        Error::ResponseError(rc) => assert_eq!(
            rc.status.as_u16(),
            expected,
            "expected HTTP {expected}, got {} (body: {})",
            rc.status,
            rc.content
        ),
        other => panic!("expected ResponseError({expected}), got {other:?}"),
    }
}
