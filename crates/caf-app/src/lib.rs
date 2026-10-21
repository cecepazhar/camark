//! Tauri binding layer for CAMark.

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

mod commands;
pub mod exporter;
pub mod fs_workspace;
pub mod vault_docs;
mod window;

fn normalize_cli_path(arg: &str, cwd: Option<&str>) -> Option<String> {
    let raw = arg.trim().trim_matches('"').trim_matches('\'');
    if raw.is_empty() || raw.starts_with('-') {
        return None;
    }

    // Strip file:// schema if passed from desktop environment or file manager
    let decoded = if let Some(stripped) = raw.strip_prefix("file://") {
        let path_part = stripped.strip_prefix("localhost").unwrap_or(stripped);
        url_decode(path_part)
    } else if raw.contains('%') {
        let dec = url_decode(raw);
        if std::path::Path::new(&dec).exists() {
            dec
        } else {
            raw.to_string()
        }
    } else {
        raw.to_string()
    };

    let path = std::path::Path::new(&decoded);
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else if let Some(c) = cwd {
        std::path::Path::new(c).join(path)
    } else if let Ok(current) = std::env::current_dir() {
        current.join(path)
    } else {
        path.to_path_buf()
    };

    let filename = resolved
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();
    if !fs_workspace::is_supported_file(&filename) {
        return None;
    }

    if resolved.exists() {
        let canonical = resolved.canonicalize().unwrap_or(resolved);
        Some(canonical.to_string_lossy().to_string())
    } else {
        Some(resolved.to_string_lossy().to_string())
    }
}

fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut bytes = s.bytes();
    while let Some(b) = bytes.next() {
        if b == b'%' {
            let h1 = bytes.next().unwrap_or(b'0');
            let h2 = bytes.next().unwrap_or(b'0');
            let hex_str = [h1, h2];
            if let Ok(byte_val) =
                u8::from_str_radix(std::str::from_utf8(&hex_str).unwrap_or("00"), 16)
            {
                result.push(byte_val as char);
            }
        } else if b == b'+' {
            result.push(' ');
        } else {
            result.push(b as char);
        }
    }
    result
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    run_with_start(std::time::Instant::now());
}

pub fn run_with_start(start: std::time::Instant) {
    caf_core::crash::install_panic_hook();

    #[allow(clippy::expect_used, clippy::disallowed_methods)]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init());

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            use tauri::{Emitter, Manager};
            for arg in args.iter() {
                if let Some(path_str) = normalize_cli_path(arg, Some(&cwd)) {
                    let _ = app.emit("open-file", &path_str);
                    break;
                }
            }
            if let Some(w) = app.get_webview_window("main") {
                let window_clone = w.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = window_clone.unminimize();
                    let _ = window_clone.show();
                    let _ = window_clone.set_focus();
                });

                #[cfg(target_os = "linux")]
                {
                    // GNOME Shell on Wayland (Mutter) ignores direct set_focus() calls from
                    // background apps. Use wmctrl -x -a to raise/activate the WMClass "camark"
                    // window, which the window manager honors even when another app is focused.
                    let _ = std::process::Command::new("wmctrl")
                        .arg("-x")
                        .arg("-a")
                        .arg("camark")
                        .spawn();
                }
            }
        }));
    }

    for arg in std::env::args().skip(1) {
        if let Some(path_str) = normalize_cli_path(&arg, None) {
            fs_workspace::set_initial_file(path_str);
            break;
        }
    }

    builder
        .setup(move |app| {
            #[cfg(target_os = "android")]
            {
                use tauri::Manager;
                if let Ok(app_data) = app.path().app_data_dir() {
                    let _ = caf_core::paths::set_custom_data_dir(app_data);
                }
            }

            window::create_main_window(app)?;

            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            #[cfg(target_os = "linux")]
            {
                // Fix for non-Debian distros (Fedora/Arch/RHEL): tauri-plugin-updater hardcodes
                // /etc/ssl/certs/ca-certificates.crt which doesn't exist on Fedora. Sanitize it
                // so child shells/processes don't inherit a poisoned SSL_CERT_FILE.
                if let Ok(cert) = std::env::var("SSL_CERT_FILE") {
                    if !std::path::Path::new(&cert).exists() {
                        let valid_ca = [
                            "/etc/pki/ca-trust/extracted/pem/tls-ca-bundle.pem",
                            "/etc/pki/tls/cert.pem",
                            "/etc/ssl/ca-bundle.pem",
                            "/etc/ssl/cert.pem",
                        ]
                        .into_iter()
                        .find(|p| std::path::Path::new(p).exists());

                        if let Some(valid) = valid_ca {
                            unsafe {
                                std::env::set_var("SSL_CERT_FILE", valid);
                            }
                        } else {
                            unsafe {
                                std::env::remove_var("SSL_CERT_FILE");
                            }
                        }
                    }
                }
            }

            #[cfg(target_os = "linux")]
            {
                use tauri::Manager;
                if let Some(window) = app.get_webview_window("main") {
                    window
                        .with_webview(|webview| {
                            use webkit2gtk::{SettingsExt, WebViewExt};
                            let inner = webview.inner();
                            if let Some(settings) = inner.settings() {
                                let enable_gpu = caf_core::prefs::load_performance_prefs()
                                    .map(|p| p.gpu_acceleration)
                                    .unwrap_or(false);
                                settings.set_enable_webgl(enable_gpu);
                            }
                        })
                        .ok();
                }
            }

            println!("CMRKRAMEWORK_COLD_START_MS={}", start.elapsed().as_millis());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::window_minimize,
            commands::window_maximize,
            commands::window_close,
            commands::window_start_dragging,
            commands::is_vault_initialized,
            commands::validate_vault_password,
            commands::lock_vault,
            commands::change_master_password,
            commands::reset_vault,
            commands::list_profiles,
            commands::save_profile,
            commands::verify_pin,
            commands::list_notes,
            commands::save_note,
            commands::delete_note,
            commands::get_ai_settings,
            commands::save_ai_settings,
            commands::test_ai_connection,
            commands::set_ai_api_key,
            commands::clear_ai_api_key,
            commands::ai_preview_context,
            commands::ai_chat,
            commands::submit_feedback,
            commands::get_pending_crash_report,
            commands::dismiss_crash_report,
            commands::export_encrypted_backup,
            commands::import_encrypted_backup,
            commands::get_performance_prefs,
            commands::set_performance_prefs,
            commands::workspace_list_directory,
            commands::workspace_read_file,
            commands::workspace_write_file,
            commands::workspace_create_file,
            commands::workspace_create_directory,
            commands::workspace_rename_file,
            commands::workspace_delete_file,
            commands::workspace_reveal_file,
            commands::get_initial_file_path,
            commands::vault_list_documents,
            commands::vault_save_document,
            commands::vault_delete_document,
            commands::export_document_html,
            commands::export_document_pdf,
            commands::ai_copilot_action,
            commands::pro_status,
            commands::pro_server_available,
            commands::pro_login,
            commands::pro_register,
            commands::pro_activate_license,
            commands::pro_logout,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
