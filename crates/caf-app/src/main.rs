// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented
)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::todo,
        clippy::unimplemented
    )
)]

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    // Ignore SIGPIPE so that a closed stdout/stderr (GUI launch) does not kill the
    // process or cause the std-panics on print failures that produce crash dumps.
    #[cfg(unix)]
    unsafe {
        unsafe extern "C" {
            fn signal(signum: i32, handler: usize) -> usize;
        }
        // SIG_IGN = 1
        let _ = signal(13 /* SIGPIPE */, 1 /* SIG_IGN */);
    }

    let start = std::time::Instant::now();
    #[cfg(target_os = "linux")]
    repair_ssl_cert_file();
    caf_app_lib::run_with_start(start);
}

/// Some launchers (AppImage runtimes, sandboxes) export an `SSL_CERT_FILE` that does not exist
/// on this system, which breaks every HTTPS call (license activation, updater, AI). Point it at
/// the system CA bundle, or drop it so the TLS stack uses its defaults.
///
/// Runs first thing in `main`, before Tauri or Tokio start any thread. It used to run inside
/// the Tauri setup hook, where other threads already existed and could read the environment
/// while it changed.
#[cfg(target_os = "linux")]
fn repair_ssl_cert_file() {
    let Ok(cert) = std::env::var("SSL_CERT_FILE") else {
        return;
    };
    if std::path::Path::new(&cert).exists() {
        return;
    }
    let system_bundle = [
        "/etc/pki/ca-trust/extracted/pem/tls-ca-bundle.pem",
        "/etc/pki/tls/cert.pem",
        "/etc/ssl/ca-bundle.pem",
        "/etc/ssl/cert.pem",
    ]
    .into_iter()
    .find(|p| std::path::Path::new(p).exists());

    // SAFETY: called at the top of `main`, before any other thread exists.
    #[allow(clippy::disallowed_methods)]
    unsafe {
        match system_bundle {
            Some(path) => std::env::set_var("SSL_CERT_FILE", path),
            None => std::env::remove_var("SSL_CERT_FILE"),
        }
    }
}
