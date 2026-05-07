# Phase 03 — Deferred Items

Außerhalb des Scopes der jeweiligen Plans entdeckt, aber pre-existing
und nicht durch die Plan-Aktionen verursacht (per `<deviation_rules>`
Scope-Boundary).

## Pre-existing: `cargo test -p dao` und `cargo test -p dao_impl_sqlite` standalone fehlschlagen

**Entdeckt während:** Plan 03-02 Task 2 Verify-Step (`cargo test -p dao_impl_sqlite`).

**Symptom:**
- `cargo test -p dao` schlägt mit `error[E0599]: no function or associated item
  named 'new_v4' found for struct 'Uuid'` in `dao/src/billing_period.rs:153`
  fehl.
- `cargo test -p dao_impl_sqlite` schlägt analog mit 9 `new_v4`-Errors in
  `billing_period.rs`, `text_template.rs`, `billing_period_sales_person.rs`
  fehl.

**Ursache:**
- `dao/Cargo.toml` und `dao_impl_sqlite/Cargo.toml` deklarieren `uuid = "1.8"`
  ohne das `v4`-Feature.
- Standalone-Test (`cargo test -p <crate>`) sieht nur die Crate-eigenen
  Cargo-Features; `Uuid::new_v4()` ist daher nicht verfügbar.
- Im Workspace-Build (`cargo build --workspace`, `cargo test --workspace`)
  wird `v4` durch Feature-Unification von anderen Workspace-Members (z.B.
  `service_impl`, `shifty_bin`) transitiv aktiviert — daher ist der Workspace-
  Build grün und der Drift fällt nur bei Standalone-Tests auf.

**Verifikation pre-existing:**
- Vor Plan 03-02 Task 2 (jj Change `572d6737`, davor `c9dd09b4`): identisches
  Verhalten. Cargo.toml unverändert seit Phase-1.
- `git log -- dao/Cargo.toml dao_impl_sqlite/Cargo.toml` zeigt nur Versions-
  Bumps (`1.10.1`, `1.11.0`, `1.12.0-dev`) — `v4`-Feature wurde nie ergänzt.

**Warum nicht hier gefixt:**
- Liegt außerhalb des Plan-03-02-Scopes (Wave-1 Domain-Surface, kein
  Cargo-Tooling-Cleanup).
- Fix wäre `features = ["v4"]` zu beiden Cargo.tomls — eine eigene Hygiene-
  Aktion, die in einem dedizierten Plan oder in Phase-4-Tooling-Cleanup
  liegen sollte.

**Praktische Konsequenz für Plan 03-02:**
- Der Plan-Verify-Step `cargo test -p dao_impl_sqlite GRÜN` ist in der
  vorliegenden Form nicht erreichbar (pre-existing Drift).
- Stattdessen: `cargo test --workspace` zeigt das wahre Bild — service_impl
  321 passed / 0 failed / 6 ignored (Plan-01-Stubs unverändert), shifty_bin
  20 passed / 8 failed (siehe nächster Eintrag, pre-existing Phase-1-
  Migrations-Lücke).
- Plan 03-02 hat das Test-Bild **nicht verändert** (kein Test ist neu
  fehlgeschlagen).

**Empfehlung:**
- Phase-4-Hygiene oder dedizierter Cleanup-Plan: `v4` in `dao/Cargo.toml` und
  `dao_impl_sqlite/Cargo.toml` ergänzen.

## Pre-existing: 8 absence_period-Integration-Tests fehlschlagen (Phase-1-Migrations-Lücke)

**Entdeckt während:** ursprünglich Plan 02-01, dokumentiert in
`.planning/phases/02-reporting-integration-snapshot-versioning/deferred-items.md`,
weiter beobachtet in Plan 03-01 SUMMARY und Plan 03-02 Verify-Step.

**Symptom:**
- `shifty_bin/src/integration_test/absence_period.rs` (8 Tests) scheitern alle
  mit `SqliteError { code: 1, message: "no such table: absence_period" }` auf
  der lokalen Dev-DB.

**Ursache:**
- Lokale `localdb.sqlite3` enthält Migrations-Drift (zwei Migrationen-Files
  fehlen aus dem Source-Tree: `20260428101456_add-logical-id-to-extra-hours`
  und `20260501162017_create-absence-period`). `sqlx setup`/`sqlx migrate run`
  bricht ab.

**Status:**
- Pre-existing seit Phase 1; nicht durch Plan 03-02 ausgelöst.
- Workspace-Compilation und `cargo test -p service_impl` sind grün.
- Tracking-Eintrag in Phase-2-deferred-items.md, STATE.md "Carry-Over für
  Phase 4" und Plan-01-Issues-Encountered.

**Empfehlung:** Phase-4-Migration nachreichen (Migrations-File-Wiederherstellung
aus Phase-1-Worktree-Branch oder neue Migration mit `<TS>_create-absence-period.sql`).
