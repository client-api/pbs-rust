// PBS lifecycle gates aren't currently used (PBS doesn't host VMs/CTs).
// The PMG-skip macro is kept for symmetry with the PVE suite so a future
// scenario port can use it.

#[macro_export]
macro_rules! skip_if_pmg {
    ($creds:expr) => {
        if !$creds.token_auth_supported() {
            eprintln!("SKIP: token auth unsupported on this product (PMG sentinel)");
            return;
        }
    };
}

#[macro_export]
macro_rules! skip_if_no_network {
    () => {
        if std::env::var("PROXMOX_NO_NETWORK").as_deref() == Ok("1") {
            eprintln!("SKIP: PROXMOX_NO_NETWORK=1 (air-gapped runner)");
            return;
        }
    };
}
