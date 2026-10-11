# CAMark PRD Audit — Notes/prd.md (FR-1..19) + prd-amendments.md vs Implementasi

> Audit: 2026-10-11 · Base: `/home/cecepazhar/Product/camark` · Metode: statis

## Hasil: 15/19 FR (79%) · Design Compliance: PASS

### Gap

| FR | Gap | Detail | Severity |
|---|---|---|---|
| FR-16 | Scheduler TIDAK ADA | Tidak ada module scheduler/cron di `caf-core`. Backup terjadwal & reminder (mandatory backup FR-9) tidak bisa jalan otomatis. | P0 |
| FR-17 | Data import/export TIDAK ADA | Tidak ada importer/exporter (CSV/XLSX/JSON per entity) di `caf-core`. Mandatory di PRD §4 & FR-17. | P0 |
| FR-15 | GCC adapter parsial | `billing/` (client, gate, token) ada; adapter non-billing tidak ada. | P1 |

### Amendments — 100% dihormati
- D-1 `[window]` di app.toml ✅
- D-4 Pro scope client-only (ProTeamPanel + sync endpoint ditunda) ✅ — konsisten `pro.rs` client licensing
- D-5 No legacy vault migration (`VaultError::LegacyFormat`) ✅

### Bukti Implementasi (key)
- `app.toml`: CAFramework generator, name=camark, bundle=com.fathforce.camark, version 0.1.3, cyan, [modules] 7 flag, [roles] owner/partner/member/child, [[menus]] workspace.
- Core: `vault.rs`, `rbac.rs`, `profiles.rs`, `ai/`, `backup.rs`, `billing/`, `feedback.rs`, `crash.rs`, `http.rs`, `pro.rs`, `sync.rs`.
- Frontend routes: workspace, vault, settings, copilot + DualSplitViewport, GlobalSplitControls, FileTreeNode, TabGroupStrip (editor Markdown multi-pane).
- Generator: `caf-xtask` `NewApp` (new-app), `caf-cli` (env/errors/vault/profile/notes), Android gen + APK.
- i18n en+id ✅.

### Rekomendasi
1. P0: Implementasi scheduler di `caf-core` (FR-16) — banyak app anak inherit.
2. P0: Implementasi data import/export (FR-17) — gate DoD v1.1.
3. P1: Dokumentasi batasan GCC adapter (billing only) di PRD.