---
phase: 61
slug: personservice-aufl-sung-von-sales-person-user
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-09-17
---

# Phase 61 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Quelle: `61-RESEARCH.md` §"Validation Architecture".

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust eingebaut) + `sqlx::migrate!` für Integrationstests gegen In-Memory-SQLite |
| **Config file** | keins — Konvention über Dateiablage in `service_impl/src/test/` |
| **Quick run command** | `cargo test --package service_impl --lib person` |
| **Full suite command** | `cargo test --workspace && cargo clippy --workspace -- -D warnings` |
| **Estimated runtime** | ~20 s quick / ~4–6 min full inkl. clippy |

> **Clippy ist ein hartes Gate** (`CLAUDE.md`): `nix build` und CI führen
> `cargo clippy -- --deny warnings` aus. `cargo test` führt clippy **nicht** mit —
> der Full-Suite-Befehl muss beide Teile enthalten.

---

## Sampling Rate

- **After every task commit:** `cargo test --package service_impl --lib person`
  (plus `cargo test --package dao_impl_sqlite`, sobald dort Tests entstehen)
- **After every plan wave:** `cargo test --workspace && cargo clippy --workspace -- -D warnings`
- **Before `/gsd-verify-work`:** Volle Suite grün **und**
  `grep -rn "sales_person_user" dao/src dao_impl_sqlite/src service_impl/src`
  liefert nur noch Migrations-Referenzkommentare (SC2)
- **Max feedback latency:** 30 s

---

## Per-Task Verification Map

Die Task-IDs werden beim Planen vergeben; die Wellen-Zuordnung unten ist die
vom Research vorgegebene Reihenfolge und **bindend**: der Byte-Identity-Vergleich
(SC1) muss in der DAO-Rewrite-Welle laufen, solange `sales_person_user` noch
existiert, und seine Alt-Query-Hälfte muss in der DROP-Welle wieder entfernt
werden, damit SC2 (grep-Sauberkeit) hält.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD | TBD | PersonDao/Service | SC5 | — | `PersonService` konsumiert keinen Domain-Service (Basic-Tier) | compile + review | `cargo build --workspace` + Durchsicht `PersonServiceDeps` | ❌ W0 | ⬜ pending |
| TBD | TBD | DAO-Rewrite | PERS-05 | T-61-SELF | Self darf die eigene Sales Person sehen und ändern | integration | `cargo test --package service_impl --lib person_self_gate::self_may_access_own_sales_person` | ❌ W0 | ⬜ pending |
| TBD | TBD | DAO-Rewrite | PERS-05 | T-61-SELF | Fremder Sales-Person-Datensatz bleibt verwehrt (kein fail-open) | integration | `cargo test --package service_impl --lib person_self_gate::foreign_sales_person_rejected` | ❌ W0 | ⬜ pending |
| TBD | TBD | DAO-Rewrite | PERS-05 | T-61-SELF | Benutzer ohne Verknüpfung bleibt verwehrt (`NULL = NULL` matcht nie) | integration | `cargo test --package service_impl --lib person_self_gate::unlinked_user_rejected` | ❌ W0 | ⬜ pending |
| TBD | TBD | DAO-Rewrite | PERS-05 / SC1 | T-61-SELF | Alt-Query und Neu-Query liefern auf derselben Fixture-DB byte-identische Pass/Fail-Matrix | integration | `cargo test --package service_impl --lib person_self_gate::byte_identity_old_vs_new_query` | ❌ W0 | ⬜ pending |
| TBD | TBD | DAO-Rewrite | SC4 / D-61-04 | T-61-MERGE | Komplementäres Verschmelzen gelingt, echte Mehrdeutigkeit wird mit 409 abgelehnt | integration | `cargo test --package service_impl --lib person_merge` | ❌ W0 | ⬜ pending |
| TBD | TBD | DAO-Rewrite | D-61-05 | — | Lösen: Sales Person behält die Person, Benutzer bekommt eine frische; keine trägerlose Person | integration | `cargo test --package service_impl --lib person_merge::split_keeps_sales_person_identity` | ❌ W0 | ⬜ pending |
| TBD | TBD | DAO-Rewrite | D-61-01 | — | Neuanlage von `user` und `sales_person` vergibt `person_id` atomar mit | integration | `cargo test --package service_impl --lib person_creation` | ❌ W0 | ⬜ pending |
| TBD | TBD | DAO-Rewrite | D-61-02 | — | „Unverknüpfte Kandidaten"-Lesepfade liefern die richtige Menge (Outer-Join, nicht `IS NULL`) | integration | `cargo test --package service_impl --lib person_candidates` | ❌ W0 | ⬜ pending |
| TBD | TBD | DROP | D-61-06 | — | Nachzieh-Backfill zieht Zeilen ohne `person_id` mit lebender `sales_person_user`-Verknüpfung nach | integration | `cargo test --package service_impl --lib person_drop::dangling_sales_person_user_row_is_backfilled_before_drop` | ❌ W0 | ⬜ pending |
| TBD | TBD | DROP | D-61-06 | — | Selbst-Assertion bricht die Migration ab, wenn nach dem Nachzug eine Verknüpfung ohne `person_id`-Gegenstück bliebe | integration | `cargo test --package service_impl --lib person_drop` | ❌ W0 | ⬜ pending |
| TBD | TBD | DROP | SC2 | — | `sales_person_user` existiert nicht mehr; Tabellenname nur noch in historischen Migrationen | integration + grep | `cargo test --package service_impl --lib person_drop::sales_person_user_table_is_gone` + `grep -rn "sales_person_user" dao/src dao_impl_sqlite/src service_impl/src` | ❌ W0 | ⬜ pending |
| TBD | TBD | Docs | SC5 | — | Service-Tier- und Auth-Doku in **beiden** Sprachen im selben Commit nachgezogen | manual | Diff-Review gegen `CLAUDE.md` §Docs-Freshness-Gate | manuell | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Kein Test-Framework fehlt — es fehlen nur die neuen Dateien selbst:

- [ ] `dao/src/person.rs` + `dao_impl_sqlite/src/person.rs` — `PersonDao`-Trait und Impl existieren noch nicht
- [ ] `service/src/person.rs` + `service_impl/src/person.rs` — `PersonService`-Trait und Impl existieren noch nicht
- [ ] `service_impl/src/test/person_self_gate.rs` — Integrationstest-Infrastruktur für den Vorher/Nachher-Vergleich; kann die Harness-Helfer aus `service_impl/src/test/person_backfill.rs` (`setup_pool`, Fixture-Funktionen, `stored_ddl`) als Vorbild wiederverwenden
- [ ] `service_impl/src/test/person_merge.rs` — Merge-/Split-/Konflikt-Matrix
- [ ] `service_impl/src/test/person_drop.rs` — Nachzieh-Backfill, Selbst-Assertion, DROP-Nachweis
- [ ] Neue Migration `migrations/sqlite/2026091700000X_drop-sales-person-user.sql`

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Die rund 50 Aufrufstellen in `service_impl/` stehen unberührt im Diff | SC3 | Ein Test kann „der Diff fasst diese Dateien nicht an" nicht ausdrücken; das ist eine Diff-Eigenschaft, keine Laufzeit-Eigenschaft | `git diff --stat service_impl/` prüfen: außer den neuen Test- und Service-Dateien darf kein bestehender Aufrufer von `SalesPersonService` geändert sein. Zusätzlich: `service/src/sales_person.rs` und `dao/src/sales_person.rs` dürfen keine geänderten Methodensignaturen zeigen. |
| `PersonService` bleibt Basic-Tier | SC5 | Die Tier-Regel ist eine Konvention über Dependency-Typen, kein Laufzeitverhalten | Durchsicht von `PersonServiceDeps` in `service_impl/src/person.rs`: nur `PersonDao`, `PermissionService`, Clock/Uuid, `TransactionDao`. Kein anderer Domain-Service. DI-Konstruktion in `shifty_bin/src/main.rs` im Basic-Block, vor der Business-Logic-Schicht. |
| Doku in beiden Sprachen synchron | SC5 / Docs-Freshness-Gate | Sprachgleichheit ist inhaltlich, nicht mechanisch prüfbar | Für jede berührte Trigger-Datei prüfen, dass `foo.md` und `foo_de.md` denselben Stand haben: `docs/architecture/02-service-tiers.md`, `docs/architecture/diagrams/service-graph-runtime.mmd`, `docs/architecture/04-auth.md`, `docs/features/F12-auth-session.md`, `docs/architecture/03-data-model.md`, `docs/architecture/diagrams/db-schema-er.mmd`. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] Byte-Identity-Test (SC1) liegt in der DAO-Rewrite-Welle, seine Alt-Query-Hälfte wird in der DROP-Welle entfernt
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
