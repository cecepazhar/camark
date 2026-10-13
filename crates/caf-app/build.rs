// Every #[tauri::command] registered in `invoke_handler!` (src/lib.rs) must be listed here too
// — Tauri v2 only auto-generates the `allow-<command>`/`deny-<command>` permissions that
// `capabilities/default.json` references for commands named in this list.
//
// A command missing from this list still compiles and still registers: it fails only at
// runtime, as `Command <name> not allowed by ACL`, and only on the code path that calls it.
// `caf-core/tests/arch.rs::tauri_command_registry_build_script_and_acl_agree` keeps this
// list, the invoke handler, and capabilities/default.json in lockstep so that can't recur.
const COMMANDS: &[&str] = &[
    // Window controls
    "window_minimize",
    "window_maximize",
    "window_close",
    "window_start_dragging",
    // Vault & Security
    "is_vault_initialized",
    "validate_vault_password",
    "lock_vault",
    "change_master_password",
    "reset_vault",
    // Multi-Profile
    "list_profiles",
    "save_profile",
    "verify_pin",
    // Sample Vertical Slice (Notes)
    "list_notes",
    "save_note",
    "delete_note",
    // Generic AI
    "get_ai_settings",
    "save_ai_settings",
    "test_ai_connection",
    "set_ai_api_key",
    "clear_ai_api_key",
    "ai_preview_context",
    "ai_chat",
    // Feedback & Crash Reporting
    "submit_feedback",
    "get_pending_crash_report",
    "dismiss_crash_report",
    // Backup & Restore
    "export_encrypted_backup",
    "import_encrypted_backup",
    // Preferences
    "get_performance_prefs",
    "set_performance_prefs",
    // CAMark Filesystem Workspace
    "workspace_list_directory",
    "workspace_read_file",
    "workspace_write_file",
    "workspace_create_file",
    "workspace_create_directory",
    "workspace_rename_file",
    "workspace_delete_file",
    "get_initial_file_path",
    // CAMark Encrypted Vault Documents
    "vault_list_documents",
    "vault_save_document",
    "vault_delete_document",
    // CAMark Standalone HTML Exporter
    "export_document_html",
    "export_document_pdf",
    "ai_copilot_action",
    "pro_status",
    "pro_server_available",
    "pro_login",
    "pro_register",
    "pro_activate_license",
    "pro_logout",
];

fn main() {
    println!("cargo:rerun-if-changed=icons/icon.ico");
    println!("cargo:rerun-if-changed=tauri.conf.json");
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .expect("gagal menjalankan tauri-build");
}
