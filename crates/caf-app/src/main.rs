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
    caf_app_lib::run_with_start(start);
}
