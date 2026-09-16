---
phase: 60
slug: person-datenmodell-backfill
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-09-16
---

# Phase 60 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Abgeleitet aus `60-RESEARCH.md § Validation Architecture`.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in Test-Harness + `tokio::test` (async) + `sqlx::migrate!`-Integrationstests gegen echte In-Memory-SQLite (`sqlite::memory:`) |
| **Config file** | keins — Cargo-Workspace-Standard; SQLx implizit über `DATABASE_URL` / `SQLX_OFFLINE` |
| **Quick run command** | `SQLX_OFFLINE=true cargo test -p service_impl person_backfill` |
| **Full suite command** | `SQLX_OFFLINE=true cargo test --workspace && cargo clippy --workspace -- -D warnings` |
| **Estimated runtime** | quick ~20–40 s · full ~3–6 min |

**Clippy ist hartes Gate** (CLAUDE.md): `cargo test` deckt Clippy NICHT ab.
`cargo clippy --workspace -- -D warnings` muss vor jedem Commit laufen.

**SQLx-Besonderheit dieser Phase:** Nach dem Anlegen der Migration muss die lokale
Dev-DB aktuell sein (`sqlx migrate run`), sonst schlagen die `query!`-Makros im
Workspace fehl. Der eingecheckte `.sqlx/`-Offline-Cache wird nur dann neu erzeugt
(`cargo sqlx prepare --workspace`), wenn ein `query!`/`query_as!`-Makro gegen
`person` oder gegen die neuen `person_id`-Spalten geschrieben wird — in Phase 60
laut D-60-10 eigentlich nicht der Fall.

---

## Sampling Rate

- **After every task commit:** `SQLX_OFFLINE=true cargo test -p service_impl person_backfill` + `cargo clippy --workspace -- -D warnings`
- **After every plan wave:** `SQLX_OFFLINE=true cargo test --workspace`
- **Before `/gsd-verify-work`:** Full suite grün + Clippy grün
- **Max feedback latency:** ~40 s (quick) / ~6 min (full)

**Zusätzliches Phase-Gate (nicht automatisierbar):** manueller `sqlx migrate run`
gegen eine **Kopie** der produktionsnahen `localdb.sqlite3`, falls verfügbar.
Begründung: D-60-01 ist als `one-way` eingestuft — einmal vergebene
Person-Identitäten binden historische `billing_period_sales_person`-Snapshots.
Ein Fixture-Lauf beweist die Mechanik, nicht die reale Datenverteilung
(CONTEXT.md `<specifics>`: es liegen **keine** Mengenannahmen vor).

---

## Kern-Invarianten (was diese Phase beweisen muss)

| # | Invariante | Warum sie zählt |
|---|-----------|-----------------|
| I-1 | Nach der Migration hat **jede** `sales_person`-Zeile — **einschließlich soft-gelöschter** — und jede `user`-Zeile ein nicht-leeres `person_id` | PERS-01 / SC1; der `WHERE deleted IS NULL`-Reflex ist die dokumentierte Falle (PITFALLS §2) |
| I-2 | Jedes Paar aus `sales_person_user` zeigt auf **dieselbe** `person.id` | PERS-01 / SC1; das ist der eigentliche Zweck des Backfills |
| I-3 | Zählprobe: `COUNT(person) == COUNT(sales_person) + COUNT(user) − COUNT(sales_person_user)` | SC1-Gegenprobe; läuft laut D-60-07 zusätzlich als **harte Assertion in der Migration selbst** |
| I-4 | Zweiter Lauf des Backfill-Abschnitts ändert weder `COUNT(person)` noch eine einzige Zuordnung (Vorher/Nachher-Diff, nicht nur „läuft durch") | PERS-02 / SC2; D-60-08 |
| I-5 | Eine soft-gelöschte Sales Person **mit** `billing_period_sales_person`-Zeile hat nach der Migration ein `person_id` | PERS-03 / SC3; dedizierte Fixture, weil genau dieser Fall sonst stillschweigend herausfällt |
| I-6 | Ein direkter SQL-Schreibversuch, zwei `sales_person`- (oder zwei `user`-) Zeilen auf dieselbe `person.id` zu setzen, wird **von der Datenbank** abgelehnt | PERS-04 / SC4; D-60-05 + D-60-06 — Index, nicht FK, nicht Applikationsprüfung |
| I-7 | Der eindeutige Index auf `sales_person.person_id` ist **vollständig**, nicht partiell — kein `WHERE deleted IS NULL` | D-60-01 (`one-way`); weicht bewusst vom dominanten Repo-Muster ab und braucht Schutz vor einem späteren „Aufräum"-Pass |
| I-8 | `person_id` bleibt auf **beiden** Seiten nullable; HR-/Admin-User ohne Sales Person und Freiwillige ohne Login bleiben anlegbar; **nirgends** wird automatisch beides angelegt | D-60-03; dauerhaft zugesagte Ausprägung, kein Übergangszustand |
| I-9 | `sales_person.id` und `user.name` sind bitgleich wie vorher; bestehende Abrechnungs-Snapshots und Schichtplan-Daten lösen unverändert auf | PERS-06 / SC5 |
| I-10 | `CURRENT_SNAPSHOT_SCHEMA_VERSION` steht unverändert auf **13** | CONTEXT.md „Gesperrt"; keine der vier Bump-Klauseln aus CLAUDE.md greift — **nicht „vorsichtshalber" bumpen** |
| I-11 | `sales_person_user` existiert nach der Phase unverändert weiter | Scope-Abgrenzung; die Tabelle ist die laufende Wahrheit, gegen die geprüft wird (Auflösung = Phase 61) |
| I-12 | `person` hat **keine** `deleted`-Spalte | D-60-04; eine ungenutzte Soft-Delete-Spalte lädt zu halb implementierter Semantik ein |

---

## Per-Task Verification Map

*Task-IDs werden beim Planen vergeben; die Zeilen unten binden Requirement → Invariante → Test → Kommando.
Der Executor trägt Task-IDs und Status nach.*

| Task ID | Plan | Wave | Requirement | Invariante | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------|-------------------|-------------|--------|
| TBD | 01 | 1 | PERS-01 | I-12 | schema | `person_backfill::person_table_has_no_deleted_column` (PRAGMA `table_info`) | ❌ neu | ⬜ pending |
| TBD | 01 | 1 | PERS-04 | I-7 | schema | `person_backfill::sales_person_index_is_not_partial` (`sqlite_master`-SQL enthält **kein** `WHERE`) | ❌ neu | ⬜ pending |
| TBD | 01 | 1 | PERS-01 | I-1, I-2 | integration | `SQLX_OFFLINE=true cargo test -p service_impl person_backfill::full_identity_coverage` | ❌ neu | ⬜ pending |
| TBD | 01 | 1 | PERS-01 | I-3 | integration | `person_backfill::count_identity_holds` (Zählprobe nach `sqlx::migrate!`) | ❌ neu | ⬜ pending |
| TBD | 01 | 1 | PERS-01 | I-3 | integration (negativ) | `person_backfill::migration_aborts_on_count_mismatch` — Selbst-Assertion aus D-60-07 muss bei manipulierter Fixture **fehlschlagen** | ❌ neu | ⬜ pending |
| TBD | 01 | 1 | PERS-03 | I-5 | integration + Fixture | `SQLX_OFFLINE=true cargo test -p service_impl person_backfill::soft_deleted_sales_person_gets_person_id` | ❌ neu | ⬜ pending |
| TBD | 01 | 1 | PERS-01 | I-8 | integration + Fixture | `person_backfill::hr_user_without_sales_person` und `person_backfill::volunteer_without_user` — je eigene Person, **kein** automatisches Anlegen der Gegenseite | ❌ neu | ⬜ pending |
| TBD | 01 | 1 | PERS-02 | I-4 | integration (Apply-twice via `sqlx::raw_sql`) | `SQLX_OFFLINE=true cargo test -p service_impl person_backfill::apply_twice_is_noop` | ❌ neu | ⬜ pending |
| TBD | 01 | 1 | PERS-04 | I-6 | integration (DB-Level-Ablehnung) | `SQLX_OFFLINE=true cargo test -p service_impl person_backfill::duplicate_person_id_rejected_by_db` (beide Seiten: `sales_person` **und** `user`) | ❌ neu | ⬜ pending |
| TBD | 01 | 1 | PERS-06 | I-9 | regression | `SQLX_OFFLINE=true cargo test --workspace` — bestehende Suite bleibt grün | ✅ | ⬜ pending |
| TBD | 01 | 1 | PERS-06 | I-10 | grep-Lock | `grep -n "CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13" service_impl/src/billing_period_report.rs` | ✅ (Check neu) | ⬜ pending |
| TBD | 01 | 1 | PERS-06 | I-11 | schema | `grep -L "DROP TABLE.*sales_person_user" migrations/sqlite/*.sql` bzw. `sqlite_master`-Assertion im Testmodul | ✅ | ⬜ pending |
| TBD | 02 | 2 | PERS-01 … PERS-06 | I-1 … I-12 | docs-gate | `docs/architecture/03-data-model.md` + `03-data-model_de.md` + `diagrams/db-schema-er.mmd` + passende `docs/features/F*.md` (EN **und** DE) im selben Commit aktualisiert | ✅ (Dateien existieren) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Sampling-Kontinuität:** Keine drei aufeinanderfolgenden Tasks ohne automatisierte
Verifikation — jede Zeile oben trägt ein Kommando oder eine grep-/PRAGMA-Assertion.

---

## Wave 0 Requirements

Diese Phase hat echte Wave-0-Lücken — das Testmodul existiert noch nicht:

- [ ] `migrations/sqlite/<ts>_create-person-backfill.sql` — die Migration selbst
      (Dateiname + Zeitstempel sind Planer-Ermessen, CONTEXT.md „Claude's Discretion")
- [ ] `service_impl/src/test/person_backfill.rs` — neues Testmodul; deckt alle
      Testfälle der Verification Map ab. Kein bestehendes File deckt das ab.
- [ ] Registrierung des Moduls dort, wo `rebooking_batch_state` registriert ist
      (`service_impl/src/test/mod.rs` — Muster übernehmen)
- [ ] Fixture-Helfer für die drei Ausprägungen, roh per SQL nach dem Muster von
      `service_impl/src/test/absence_conversion.rs`:
      (a) soft-gelöschte Sales Person **mit** `billing_period_sales_person`-Zeile,
      (b) HR-/Admin-`user` ohne Sales Person,
      (c) Freiwilliger `sales_person` ohne `user`,
      (d) verknüpftes Paar über `sales_person_user`

**Kein Framework-Install nötig** — `sqlx`, `tokio` und `uuid` sind bereits
Dev-Dependencies; Präzedenz für den Integrationstest-Aufbau ist
`service_impl/src/test/rebooking_batch_state.rs`.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Migration gegen realistischen Datenbestand | PERS-01, PERS-03 | Produktions-Datenverteilung ist unbekannt (CONTEXT.md: keine Zahlen zu HR-Accounts, Freiwilligen, soft-gelöschten Sales Persons); Fixtures beweisen die Mechanik, nicht die Menge | Kopie der `localdb.sqlite3` ziehen → `sqlx migrate run` → Zählprobe aus I-3 manuell per `sqlite3` gegenprüfen → Migration darf **nicht** abbrechen |
| Rollback-Pfad | — | Kein Down-Migration-Konzept; der Betreiber zieht laut CONTEXT.md ein Datei-Backup vor dem Deploy (D-60-09: **keine** Backup-Tabelle) | Vor Deploy: SQLite-Datei sichern. Rollback = Datei zurückspielen, nicht Down-Migration |

*Kein Frontend-Anteil in dieser Phase (ROADMAP: „bewusst kein Anteil — reine Schema-
und Backfill-Phase ohne API-Wirkung"), daher keine UI-Verifikation.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (Testmodul + Migration + Fixtures)
- [ ] No watch-mode flags
- [ ] Feedback latency < 40 s (quick) / < 6 min (full)
- [ ] `cargo clippy --workspace -- -D warnings` in jedem Task-Gate enthalten
- [ ] Docs-Freshness-Gate erfüllt (EN **und** DE im selben Commit)
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
