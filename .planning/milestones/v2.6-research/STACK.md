# Technology Stack — v2.6 Freiwillige-Stunden-Ausgleich für gedeckelte Mitarbeiter

**Project:** shifty-backend (monorepo incl. shifty-dioxus)
**Researched:** 2026-07-06
**Scope:** ONLY new stack surface for F1–F5. Existing capabilities (Axum, layered
architecture, `ExtraHoursService`, `ReportingService`, `ToggleService`, `sqlx`,
`tokio_cron_scheduler`, Dioxus frontend) are treated as fixed baseline.

**Bottom line — NO NEW HARD DEPENDENCIES REQUIRED.** All five features can be
built by reusing the stack that already ships in `v2.5.1-dev`. This is by design:
`v2.6` is a business-logic / data-model milestone, not an infrastructure
milestone. One optional additive (dev-dep `clap` for a backfill subcommand)
is discussed below and is explicitly recommended AGAINST — a REST endpoint is
the better path.

## Recommended Stack — for what changes in v2.6

### Reuse without modification

| Concern | Reused component | Version (as pinned) | Why |
|---|---|---|---|
| Weekly cron loop (F4) | `tokio-cron-scheduler` | `0.15` (`0.15.1` in lockfile) | Already carries the `PdfExportScheduler` (v2.2/Phase 48). Same runtime, same `JobScheduler` pattern with dormant-boot + `reload_from_db` semantics, same TLS/rustls stance. No second scheduler needed. |
| Persistence (2 new tables) | `sqlx` + `sqlx::migrate!` | `0.8.2` (`0.8.6` in lockfile) | Every previous milestone that added tables (v1.0 `absence_period`, v1.1 `slot.max_paid_employees`, v1.4 `committed_voluntary`, v2.1 `week_status`, v2.2 `pdf_export_config`) added them additively via `migrations/sqlite/YYYYMMDDHHMMSS_*.sql`. Two more additive files fit that path exactly. Compile-time query checking (`SQLX_OFFLINE=true` in CI + `cargo sqlx prepare --workspace` + committed `.sqlx/`) is already the workflow. |
| Rebooking as 2 ExtraHours in one Tx (F3, F4, F5-approve) | `ExtraHoursService::create` + `TransactionDao` | current | Service-tier convention (`Option<Transaction>` → `use_transaction` → business logic → `commit`) is precisely what „−N in Kategorie A / +N in Kategorie B atomar" needs. No saga library, no outbox. Parent-batch-write + N-child-writes + Tx-commit = one flat sequence. |
| Ist-Statistik F1 aggregation | `ReportingService` year-batch pattern | current | v2.5 lifted three Chain A/B/C preloads to `_iso_year` bulk methods; the pure helpers `derive_hours_for_week_pure` + `build_derived_holiday_map_for_week_pure` already have year-scope inputs. F1 = „Σ voluntary hours ÷ contract-weeks" — one more year-scoped pure fold. |
| Soll-Berechnung F2 | `WorkingHoursService` + `committed_voluntary` field | current (v1.4 delivery) | Zeit-versioniertes Feld existiert; F2 multipliziert es mit den Vertragswochen des Jahres. Neuer Aggregate-Service, keine neuen Deps. |
| HR-alert row + modal (F5) | Dioxus + `rest-types` union path | current | Zusätzliches DTO-Feld `NegativeBalanceAlertTO`/`RebookingSuggestionTO`, additiv mit `#[serde(default)]`-Guard (Präzedenz v2.5 D-53). Kein neuer Modal-Framework, kein neuer FE-State-Store. |
| Auth-Gate HR-only (F1, F2, F5) | `PermissionService` mit `HR`-Privileg | current | Identisch zu AVG-01 (v2.1) und VAC-OFFSET-01 (v1.8). |
| API-Level hiding von HR-Feldern | Existing pattern (`Self-View` sees `None`) | current | v1.8 VAC-OFFSET-01 hat den Präzedenzfall etabliert; F1+F2 folgen dem. |
| Cutoff-Toggle (Rollout-Schutz, optional) | `ToggleService` | current | Falls F4 einen `active_from`-Stichtag braucht (analog `holiday_auto_credit_active_from` v1.7 und `shortday_slot_clipping_active_from` v2.4), ist der Blueprint fertig. Keine neue Dep. |
| Observability | `tracing` + `warn!`/`error!`/`info!` | `0.1.40` | Existing scheduler-Muster (`pdf_export_scheduler.rs`) loggt Cron-Trigger, Errors, Skip-Reasons ohne Sensitive-Data. F4/F5 folgen 1:1. |

### Additive (optional) — nur EIN Kandidat

| Technology | Version | Purpose | Verdict |
|---|---|---|---|
| `clap` | `4.6.1` (derive-Feature) | Backfill-CLI-Subcommand für F4-Rollout (rückwirkende Auto-Umbuchung ab Datum X) | **NICHT empfohlen.** Heute parst `shifty_bin/src/main.rs` **keine** CLI-Args (grep-verifiziert: nur ein Kommentar-Match, kein `clap`/`env::args`/`std::env` in produktivem Code). Ein einmaliger Backfill kann als **admin-gated REST-Endpoint** (`POST /rebooking/backfill?from=YYYY-MM-DD&dry_run=true`) implementiert werden. Vorteile: (a) kein Deployment-Rebuild für einen One-Shot, (b) reuse des bestehenden Auth-Gates, (c) audit-trail via existierende Access-Logs, (d) `dry_run`-Modus trivial, (e) keine neue Crate. **Empfehlung: KEIN `clap`. Backfill = HR-Endpoint mit Dry-Run-Flag.** |

### NICHT neu einführen — explizit ausschließen

| Verlockung | Warum ablehnen |
|---|---|
| `chrono` (upgrade / weitere Nutzung) | `time = "0.3.36"` ist die kanonische Zeit-Bibliothek workspace-weit (`ShiftyDate`/`ShiftyWeek` in `shifty-utils`). `chrono` ist NUR Transitiv-Abhängigkeit von `tokio-cron` und `service_impl` (für `Local` in Legacy-`scheduler.rs`). Neue Zeit-Logik in F1/F4 muss `time` + `ShiftyDate`/`ShiftyWeek` nutzen. |
| Ein zweites Scheduler-Framework (`apalis`, `sqlx-cron`, `cron`, …) | `tokio-cron-scheduler 0.15` deckt Wochen-Cron mit 6-Feld-Ausdrücken ab (Migration `20260704000000_fix-pdf-export-cron-6-field.sql` zeigt: das ist die etablierte Konvention). Ein Zweit-Scheduler bricht das Dormant-Boot- + Reload-from-DB-Muster und verdoppelt die Test-Fläche. |
| `serde_yaml`, `toml` für Config | Cron-Ausdruck + Enable-Flag gehören in `rebooking_batch_config`-Tabelle (analog `pdf_export_config`, v2.2) — falls überhaupt konfigurierbar. Keine File-Config nötig. |
| Neuer HTTP-Client / SSE / WebSocket für F5-Alerts | F5-Banner wird durch existierenden REST-Poll (`GET /employees/:id/rebooking-suggestions` o. ä.) versorgt — analog v1.6 Overage-Warn-Sektion. Live-Push ist Overkill für HR-Sichtbarkeit. |
| „Batch-Job-Library" wie `sqlxmq`, `graphile-worker` | Batch = 1 Parent + N Children in einer SQLite-Tx. Kein Job-Queue-Substrat nötig; das ist ein reines Domänen-Aggregat, kein Distributed-Task-System. |
| Neuer FE-State-Store (`dioxus-signals`-Erweiterung o. ä.) | Rebooking-Modal ist ephemeres Modal-State — der Präzedenzfall aus v1.5/v1.8 (Convert-Dialog, VAC-OFFSET-Editor) reicht: lokales `use_signal` im Component-Scope. |
| `mockall` upgrade / Ersatz | `mockall = "0.13"` ist workspace-weit; alle neuen Services testen sich damit unverändert. |
| Snapshot-Schema-Version-Bump „vorsorglich" | Nur bumpen, wenn F1/F2/F4/F5 einen neuen persistierten `BillingPeriodValueType` einführen. **Wahrscheinlich NICHT** — die 2 neuen Tabellen sind Domain-State-Tracking, kein Billing-Snapshot-Wert. In discuss-phase pinnen. |

### Cargo.toml-Konflikt-Check

Ich habe die drei relevanten Manifeste (`shifty_bin`, `service_impl`, workspace-root) und `Cargo.lock` gegen die geplanten Reuse-Punkte gelesen:

- `tokio-cron-scheduler = { version = "0.15", default-features = false }` in `service_impl/Cargo.toml` — bereits gepinnt, kein Konflikt.
- `sqlx = "0.8.2"` in `shifty_bin` + `service_impl` (dev) + `dao_impl_sqlite` — alle drei alignen; Lockfile-Auflösung `0.8.6` (patch-innerhalb-0.8). Kein Konflikt.
- `time = "0.3.36"` mit `local-offset` — reicht für F1-Aggregation (Woche-Berechnung, ISO-Kalender). Kein Feature-Bump nötig.
- `mockall = "0.13"`, `async-trait = "0.1.80"`, `thiserror = "1"`, `tracing = "0.1.40"` — unverändert.
- `printpdf`, `reqwest_dav`, `reqwest`, `tera`, `minijinja`, `wiremock` — v2.6-irrelevant, bleiben unverändert.

**Keine `Cargo.toml`-Konflikte erwartet.** F3/F4 fügen `sqlx::query!`/`query_as!`-Aufrufe für die 2 neuen Tabellen hinzu — reine Anwendungsschicht, keine Version-Bumps.

## Alternativen erwogen

| Bereich | Empfohlen | Alternative | Warum abgelehnt |
|---|---|---|---|
| F4 Cron-Trigger | `tokio-cron-scheduler` (reuse) | Neuer `apalis`/`sqlxmq`-Worker mit persistierter Queue | Wochen-Cron ist idempotent + read-heavy + write-batch — Queue-Substrat wäre Over-Engineering. Nach Crash: nächster Wochen-Trigger holt die vergessene Woche via Backfill-Endpoint nach. Präzedenzfall v2.2 PDF-Export läuft seit 2026-07-03 stabil mit demselben Muster. |
| F5 Alert-Delivery | REST-Poll auf HR-Dashboard | Server-Sent Events / WebSocket-Push | Poll-Latenz irrelevant (HR-Team schaut alle paar Stunden); Push-Infra ist neuer Angriffs-/Test-Vektor. |
| Batch-Persistenz (F4/F5) | 2 SQLite-Tabellen + Tx | Event-Sourcing über `extra_hours`-Historie | ExtraHours ist bereits das Log der Wahrheit; `rebooking_batch` = Parent-Metadaten (kind, created_by, state, source_week), `rebooking_batch_entry` = 1 Row pro SalesPerson mit Verweis auf die zwei erzeugten `extra_hours.id`s. Rebuild eines Batches = JOIN. Kein CQRS-Framework. |
| Backfill-Trigger | HR-gated REST-Endpoint mit `?dry_run=true` | `clap`-Subcommand im Binary | Kein Rebuild + Deploy für einen One-Shot, kein neues Auth-Handling. |
| Cutoff-Stichtag für F4 | `ToggleService` (falls überhaupt nötig) | Config-Datei / ENV-Var | `ToggleService` mit `active_from`-`value` ist der einzige workspace-etablierte Weg (v1.7 `holiday_auto_credit_active_from`, v2.4 `shortday_slot_clipping_active_from`). Analog D-51-07 / HCFG-02. In discuss-phase klären, ob überhaupt nötig — F4 könnte auch nur „Vorwoche == KW-1" ohne Toggle laufen und Rollout via Backfill-Endpoint handhaben. |

## Installation — was tatsächlich neu geschrieben wird

```bash
# Migrations (additiv, keine bestehende Zeile verändern):
# migrations/sqlite/20260707000000_create-rebooking-batch.sql
# migrations/sqlite/20260707000001_create-rebooking-batch-entry.sql
# (evtl. 20260707000002_seed-auto-rebooking-active-from-toggle.sql, wenn ToggleService-Gate gewünscht)

# Nach jedem neuen sqlx::query!/query_as! in DAO-Impls:
cd shifty-backend
cargo sqlx prepare --workspace   # committet .sqlx/*.json — CI läuft SQLX_OFFLINE=true

# Keine Cargo-Änderungen erwartet.
```

## Integration mit bestehenden Services — konkrete Andockpunkte

| Neue Kapazität | Andockt an | Wie |
|---|---|---|
| F1 Statistik-Aggregation | `ReportingService` (existing) oder neuer thin **business-logic** `VoluntaryStatisticsService` | Konsumiert `ExtraHoursService` + `WorkingHoursService` + optional `SpecialDayService::get_by_iso_year` (wenn v2.5-Follow-up SDF-03 vorher ausgeführt wird — sonst reicht year-batch aus v2.5). |
| F2 Soll-Aggregat | Neuer basic/business-logic Service (Klassifizierung in discuss-phase) | Liest `WorkingHours::committed_voluntary` × Vertragswochen; write-free. |
| F3 Manuelle Umbuchung | Neuer **business-logic** `RebookingService` | Konsumiert `ExtraHoursService::create` (2×) + `TransactionDao` + `PermissionService`. Schreibt `rebooking_batch` (kind=`manual`) + `rebooking_batch_entry` mit Verweis auf die beiden erzeugten `extra_hours.id`s. |
| F4 Automatischer Cron | Neuer `AutoRebookingScheduler` (analog `PdfExportSchedulerImpl`) | Reuse `JobScheduler`-Muster mit dormant-boot + `reload_from_db`; per Tick: iterate SalesPersonen mit `cap=true`, berechne `Ist > Soll + committed_voluntary`, delegiere Umbuchung an `RebookingService::rebook(..., kind=auto_cron)`. |
| F5 HR-Alert-Feed | Erweiterung Employee-Overview-Endpoint + neuer `RebookingSuggestionService` | Business-Logic-Tier. Signals-Trigger: negatives `balance_hours` UND `cap=true`. Vorschlags-Rows in `rebooking_batch(kind=hr_suggestion, state=pending)`; approve/reject mutiert `state`; approve führt zusätzlich Rebooking aus (denselben Pfad wie F3). |
| Backfill-CLI (F4-Rollout) | Neuer REST `POST /rebooking/backfill` (nicht `clap`) | HR-Admin-gated, `?dry_run=true` + `?from=YYYY-MM-DD` — schreibt Batches mit `kind=auto_cron_backfill`. |

**Service-Tier-Klassifizierung (per CLAUDE.md-Konvention):**

- `RebookingService` → **business-logic** (konsumiert `ExtraHoursService`, kombiniert mehrere Aggregate).
- `AutoRebookingScheduler` → **business-logic** (konsumiert `RebookingService` + `WorkingHoursService` + `ReportingService`).
- `VoluntaryStatisticsService` → **business-logic** (Read-Aggregat).
- `RebookingSuggestionService` → **business-logic** (kombiniert Balance + Rebooking).
- KEIN Basic-Service für die 2 neuen Tabellen nötig, wenn die DAO direkt aus `RebookingService` konsumiert wird — beide Tabellen sind ein einziges Aggregat mit einer Wurzel (`rebooking_batch.id`).

## Beobachtbarkeit / Scheduler-Pattern zum Wiederverwenden

Der `PdfExportSchedulerImpl` liefert das komplette Blueprint (F4 sollte dies 1:1 spiegeln):

1. `Arc<Mutex<Option<JobScheduler>>>` — lazy-init in `start()`, tolerant gegenüber DB-Config-Fehlern beim Boot (dormant-Modus + `warn!`-Log, kein Boot-Fail).
2. `reload_from_db()` — separate Methode, die per Config-Änderung (PUT-Endpoint) getriggert wird, alten Job removed, neuen Job registriert.
3. `Job::new_async(cron_schedule.as_str(), move |_uuid, _lock| { … })` — 6-Feld-Cron-Ausdruck (siehe Migration `20260704000000_fix-pdf-export-cron-6-field.sql`).
4. Per-Iteration: `Authentication::Full` für interne Aggregate (siehe `ToggleService`-Full-Context-Bypass in MEMORY-Referenz `reference_toggle_service_full_context_bypass.md`).
5. Fehler in einem einzelnen Item werden per `record_error` + `return Ok(())` **skip-per-item** behandelt, nicht batch-fail (v2.3.1-Präzedenz Commit `754f94f`).
6. Kein sensibles Feld loggen (Token-Leak-Guard) — F4 sollte SalesPerson-IDs loggen, aber keine Namen.

**Neuer Test-Modus, den F4 mitbringen sollte** (nicht als Dep, als Konvention): `RebookingService::compute_dry_run(week) -> DryRunReport` — dieselbe Berechnungsfunktion ohne DAO-Write, wiederverwendet vom Backfill-Endpoint UND F5-Modal-„DANN"-Zahl. Präzedenzfall: `PdfExportScheduler` hat kein Dry-Run, aber das `absence_service::compute_conflict`-Muster (v1.0) hat es — Ratschlag: von dort spiegeln.

## Sources

- Repo-internal: `service_impl/src/pdf_export_scheduler.rs` (Zeilen 1–260, Blueprint für F4), `service_impl/src/scheduler.rs` (Legacy `tokio-cron`, NICHT als Vorlage), `service_impl/src/pdf_export_config.rs` (Config-Reload-Muster), `service_impl/Cargo.toml` (pinning-Präzedenz), `Cargo.lock` (transitive Auflösungen `tokio-cron-scheduler 0.15.1`, `sqlx 0.8.6`, `reqwest_dav 0.3.3`, `printpdf 0.7.0`).
- Repo-internal: `CLAUDE.md` (Service-Tier-Konvention, `gen_service_impl!`, sqlx-prepare-Gate, Docs-Freshness-Gate, Snapshot-Schema-Version-Bump-Vertrag).
- Repo-internal: `.planning/PROJECT.md` §„Current Milestone: v2.6" (F1–F5-Definition), §v2.5-Shipped (Fat-Backend-Präzedenz und Chain-A/B/C-year-scope), §v2.2-Shipped (WebDAV-Scheduler-Muster).
- Repo-internal: `migrations/sqlite/*.sql` (Migration-Namenskonvention, additive-Präzedenz — 25 Einträge, alle additiv).
- Web: [tokio-cron-scheduler 0.15.1 — crates.io](https://crates.io/crates/tokio-cron-scheduler) — bereits gepinnt, latest confirmed.
- Web: [clap 4.6.1 — Docs.rs](https://docs.rs/crate/clap/latest) — nur als NICHT-empfohlene Alternative dokumentiert.

**Confidence:** HIGH — alle Empfehlungen stützen sich auf im Repo bereits gelieferte Präzedenzfälle (v1.0, v1.4, v1.7, v1.8, v2.2, v2.4, v2.5) statt auf externe Web-Behauptungen. Der einzige Web-Fakt (Version-Bestätigung `tokio-cron-scheduler 0.15.1` + `clap 4.6.1`) ist crates.io-verified.
