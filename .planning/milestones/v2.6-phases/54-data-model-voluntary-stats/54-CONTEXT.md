# Phase 54: Data-Model + Voluntary Statistics (F1 + F2) — Context

**Gathered:** 2026-07-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 54 liefert **HR-only Ist/Soll/Delta-Sichtbarkeit** für Freiwillige Stunden im
Employee-Year-Report (`/employees/:id/:year`) und die **additive Datenmodell-Basis**
für alle späteren Rebooking-Trigger (F3/F4/F5 in Phase 55+56).

**In-Scope (F1 + F2 + Data-Model):**
- 2 additive SQLite-Migrationen: `rebooking_batch` (Parent) + `rebooking_batch_entry` (Child).
- 3. Migration: Toggle-Seed `voluntary_rebooking_auto_active_from` (Default `None`).
- Marker auf `extra_hours`: neue Spalte `source TEXT NOT NULL DEFAULT 'manual'`.
- UNIQUE-Constraint `(sales_person_id, iso_year, iso_week) WHERE deleted IS NULL` auf `rebooking_batch`.
- `RebookingBatchService` (Basic-Tier, Entity-Manager, CRUD noch ohne Business-Logik).
- `VoluntaryStatsService` (BL, HR-gated, read-only) mit zwei pure fns:
  `voluntary_hours_per_contract_week(sp_id, year)` (F1-Ist) und
  `committed_voluntary_target_for_year(sp_id, year)` (F2-Soll).
- REST-Endpoint für F1+F2 (Route-Design offen für Planer — Empfehlung: neues Modul
  `rest/src/voluntary_stats.rs` oder additive Response-Felder auf existierendem
  Employee-Year-Report).
- FE-Row „Freiwillige Stunden — Ist / Soll / Δ" im Employee-Detail-Page, HR-gated
  via existierendes Role-Gate, direkt unter der bestehenden „Freiwillige Stunden"-Zeile.
- `Dioxus.toml` [[web.proxy]]-Eintrag für den neuen Endpoint
  (Memory `feedback_dioxus_proxy_for_new_backend_endpoints`).
- i18n (de/en/cs): Row-Labels „Freiwillige Stunden Ist / Soll / Delta" (Wording final
  im Planer).
- Property-Test „Rebooking-Neutralität für Read-Aggregate" (VOL-ACCT-03-CI-Guard):
  fingierte Rebooking-Pair-ExtraHours-Row (Marker gesetzt) verändert weder F1-Ist
  noch F2-Soll.
- Docs-Freshness synchron im gleichen Commit: neu `docs/features/F14-rebooking.md`
  (+ `_de.md`); Update `docs/architecture/02-service-tiers.md` +
  `diagrams/service-graph-runtime.mmd`; Update `docs/architecture/03-data-model.md`
  + `diagrams/db-schema-er.mmd`.

**Out-of-Scope (bleibt Phase 55/56):**
- Kein Schreib-Pfad auf Rebooking-Tabellen von außen (nur DAO-CRUD über
  Basic-Service — Konsumenten kommen erst in Phase 55).
- Kein `RebookingReconciliationService` (BL, orchestriert 2× ExtraHours + Batch-Entry).
- Kein `VoluntaryRebookingScheduler`, kein Cron, kein Backfill-Endpoint.
- Kein F5-Alert-Banner, kein Vorschlags-Modal.
- Kein Snapshot-Schema-Version-Bump (Entscheidung 12→13 ist REB-AUTO-05 vertagt auf
  Phase-56-discuss-phase; Straddling-Golden-Snapshot dort als Beweislast).

</domain>

<decisions>
## Implementation Decisions

### D-F1-01 — F1-Denominator-Definition
**Entscheidung:** Denominator = Anzahl ISO-Wochen im Jahr, in denen die Person eine
gültige `working_hours`-Row besitzt. **`expected_hours = 0` zählt mit** (z. B.
0-h-Verträge / Elternzeit / Sabbatical mit Kontrakt aber ohne Sollarbeitszeit).

**Warum:** User-Entscheidung, bewusst weiter als Research-Empfehlung „strikt
contract-weeks mit `expected_hours > 0`". Rationale: die Frage F1 misst „Ø freiwillige
Stunden pro Vertragswoche" — sobald ein Vertrag existiert, ist die Person „unter
Vertrag", auch wenn das Vertragsvolumen null ist. Ein 0-h-Vertragler, der 4 h
freiwillig macht, hat `1 × 4h / 1 week = 4h/Woche` und nicht `undefined`. Präzedenz-
konsistent mit v1.4 `committed_voluntary`, das ebenfalls per-Woche wirkt ohne
`expected_hours`-Vorbedingung.

**Explizit NICHT:** AVG-01-A-22-1-Semantik („contract-weeks minus komplett-Absence")
— das wäre für die direkt nebenliegende Ø-Anwesenheits-Row inkonsistent, aber F1
misst eine andere Größe (freiwillig geleistet, nicht anwesend). Absence-Adjust wäre
Doppel-Korrektur.

### D-F2-01 — Mid-Week-Vertragswechsel-Semantik für VOL-ACCT-01-Soll
**Entscheidung:** **A pro-rata.** Wenn `committed_voluntary` mitten in einer
ISO-Woche wechselt, zählt die Woche anteilig zu Tagen unter jedem Kontrakt.
Formel-Skizze: `week_committed = Σ (kontrakt_i.committed_voluntary × tage_i / 7)`
über alle innerhalb der Woche gültigen `working_hours`-Rows.

**Warum:** User-Entscheidung gegen Research-Empfehlung (die „reuse
`WorkingHoursService::get_working_hours_for_week`-Semantik" ≈ latest-active
vorschlug). Rationale: F2-Soll ist eine kumulative Konten-Zusage, keine
Punkt-Messung. Pro-rata ist ehrlicher — ein Kontraktwechsel Mittwoch mit halbierter
Voluntary-Zusage soll ab Mittwoch anteilig wirken, nicht rückwirkend für die ganze
Woche „umgeschrieben" werden.

**Konsequenz für Planer:** ein neuer Aggregator `committed_voluntary_prorata_for_week`
oder direkt inline in `committed_voluntary_target_for_year` — die existierende
`get_working_hours_for_week` liefert die pro-Wochen-Contract-Slots, aber die
anteilige Aggregation muss neu. Ein pure-fn-Helper mit Tag-Set-basierter Verteilung
ist der wahrscheinliche Weg (`ShiftyDate`-Iteration Mo–So, `working_hours_for_day`
lookup, aufsummieren). Grand-Aggregat: Σ über alle 52/53 ISO-Wochen des Jahres.

**Randfall Jahresübergang:** ISO-Woche 53/Woche 1-Straddle behält die `iso_year`-
Semantik von v2.5 `_iso_year`-Helper — der Nenner in D-F1-01 nutzt dieselben
ISO-Wochen wie der Soll-Nenner in D-F2-01 (kongruent).

### D-54-DM-01 — UNIQUE-Constraint-Shape auf `rebooking_batch`
**Entscheidung:** **(i)** `UNIQUE (sales_person_id, iso_year, iso_week) WHERE
deleted IS NULL` — globale Wochen-Sperre über **alle** Kinds (`manual`,
`hr_suggestion`, `auto_cron`, `auto_cron_backfill`).

**Warum:** Deckt sich mit Research-Empfehlung + Pitfall 4 (kombinierter Idempotenz-
+ TOCTOU- + Doppel-Zählungs-Guard). Ein `hr_suggestion(state=pending)` beansprucht
die Wochen-Slot sofort → Claim-on-Suggest-Strategie (HR-ALERT-04) fällt aus dem
Constraint direkt raus, ohne eigene State-Machine. Cron-Restart / Backfill über
bereits verarbeitete Woche → `INSERT ... ON CONFLICT DO NOTHING`, no-op.

**Randfall Reject:** wenn ein `hr_suggestion` `state=rejected` wird, muss die
Wochen-Slot wieder frei — Roadmap-Klärung Phase 55 (`state='rejected'` gilt als
soft-blocking? oder mit `WHERE state <> 'rejected'` verfeinern?). **Für Phase 54
irrelevant** — dieser Trade-off wird beim ersten Rebooking-Trigger-Schreiber
(Phase 55) beobachtbar.

### D-54-DM-02 — Marker-Approach auf `extra_hours`
**Entscheidung:** **(x) STRING-Enum.** Neue Spalte `source TEXT NOT NULL DEFAULT 'manual'`
auf `extra_hours`. Werte: `'manual' | 'rebooking'`.

**Warum:** User-Entscheidung, simpel + kein NULL-JOIN in jeder Balance-/F1-/F2-Chain.
Filter-Semantik: alle Aggregat-Reader (F1, F2, `reporting.rs::balance`,
`booking_information::get_weekly_summary`) filtern `WHERE source = 'manual'` (oder
`source != 'rebooking'` — Planer entscheidet, semantisch äquivalent). Alte Rows
bekommen per DEFAULT `'manual'` beim Migrations-Schema-Add und im ALTER TABLE
Backfill (Präzedenz v2.5 additive Spalten).

**Konsequenz:** kein Reverse-Lookup „welche ExtraHours gehört zu welchem Batch"
ohne extra Query — akzeptabel für v2.6, weil Batch-→-ExtraHours-Referenz bereits
via `rebooking_batch_entry.extra_hours_out_id / _in_id` (BLOB-FK) gehalten wird.

**Migration-Detail:** SQLite-Konvention `ALTER TABLE extra_hours ADD COLUMN
source TEXT NOT NULL DEFAULT 'manual'` — additiv, sqlx-prepare-Gate danach
(Memory `reference_sqlx_prepare_after_new_query`).

### Locked (Prior-Context, nicht neu diskutiert)

- **Fat Backend / Thin Client:** F1-Ist, F2-Soll, F2-Delta werden im BE berechnet;
  FE rendert nur DTO-Felder, keine Arithmetik (Memory `feedback_fat_backend_thin_client`).
- **HR-Only via API-Level-Redaction:** DTO-Felder für Non-HR = `Option<f32> = None`
  (Präzedenz VAC-OFFSET-01 v1.8, VOL-STAT-02 / VOL-ACCT-02). Kein Frontend-Redact.
- **Snapshot-Version bleibt 12** in Phase 54. Bump 12→13 ist REB-AUTO-05 → auf
  Phase-56-discuss-phase vertagt.
- **Rebooking-Neutralität als Property-Test:** VOL-ACCT-03 wird als CI-Property-Test
  in Phase 54 implementiert (Rebooking-Marker-Row lässt F1-Ist und F2-Soll invariant).
- **Toggle-Seed `voluntary_rebooking_auto_active_from`** (Default `None`) wird in
  Phase 54 gesät, Wirkung aber erst Phase 56 (Cron-Guard). Präzedenz v2.4 SHC-04.
- **Kind-Diskriminator** auf `rebooking_batch`: `TEXT NOT NULL` mit Werten
  `manual | hr_suggestion | auto_cron | auto_cron_backfill` — Phase 54 setzt das
  Schema, Verwendung erst Phase 55/56.
- **Service-Tier-Trennung:** `RebookingBatchService` ist **Basic** (nur DAO +
  Permission + Transaction als Deps); `VoluntaryStatsService` ist **BL** (liest
  `ExtraHoursService`, `WorkingHoursService`, `SalesPersonService`, `PermissionService`).
  Konvention `service_tier_convention` in CLAUDE.md.
- **Docs-Freshness im gleichen Commit** wie Code-Diff (Memory
  `feedback_docs_always_current_no_followup`). Kein deferred_item.

### Claude's Discretion
- **REST-Route-Design für F1/F2:** entweder additive Response-Felder auf existierendem
  Employee-Year-Report-Endpoint ODER neuer dedizierter Endpoint
  `GET /reporting/employee/{y}/{spid}/voluntary-stats`. Planer entscheidet basierend
  auf DTO-Größe + Cache-Semantik. Beide Muster sind präzedent (v1.8 Employee-Report
  additiv vs. v2.1 AVG-01 separater Endpoint).
- **i18n-Wording final:** „Freiwillige Stunden Ist / Soll / Δ" vs. „Ist-Ø
  freiwillig pro Woche / Zugesagt / Konto" — Planer/UI-Phase präzisiert; Domain-
  User (HR) versteht beide Varianten.
- **FE-Row-Layout:** eine Zeile mit 3 Werten (Ist / Soll / Delta) oder drei
  separate Zeilen — Planer entscheidet visuell, MEMORY-präzedent v2.1 AVG-01
  „drei Zellen in einer Zeile".
- **Toggle-Seed-Migration:** eigene Migration-Datei oder inline in der
  `rebooking_batch`-Migration — Planer entscheidet nach v2.4-SHC-04-Pattern.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone-Kern
- `.planning/REQUIREMENTS.md` — 17 REQ-IDs (VOL-STAT, VOL-ACCT, REB-MANUAL, REB-AUTO,
  HR-ALERT); Phase 54 adressiert VOL-STAT-01/02 + VOL-ACCT-01/02/03.
- `.planning/ROADMAP.md` §Phase 54 — Success Criteria + Präzedenzen + Backend/Frontend-
  Pfade + Discuss-Points.
- `.planning/research/SUMMARY.md` — Executive Summary + Divergenzen; §"Divergenz 2"
  klärt Snapshot-Bump-Vertagung.
- `.planning/research/FEATURES.md` — F1/F2/F3/F4/F5 Feature-Detail.
- `.planning/research/ARCHITECTURE.md` — Service-Graph + Table-Schemas + DTO-Schemas.
- `.planning/research/STACK.md` — Cargo-Deps Zero-Add-Bestätigung.
- `.planning/research/PITFALLS.md` — 17 durchnummerierte Pitfalls; Phase 54 addressiert
  1 (Doppel-Zählung), 8 (HR-Only DTO-Redaction), 9 (shared pure-fn), 10
  (F2-Soll reuses `get_working_hours_for_week`), 13 (Docs-Freshness).

### Projekt-Konventionen (harte Gates)
- `shifty-backend/CLAUDE.md` §"Service-Tier-Konventionen" — Basic vs. BL.
- `shifty-backend/CLAUDE.md` §"Billing Period Snapshot Schema Versioning" —
  Bump-Kontrakt (in Phase 54 nicht getriggert — Bestätigung dokumentiert).
- `shifty-backend/CLAUDE.md` §"Docs-Freshness-Gate" — Trigger-Datei-Tabelle;
  Migrations + service/*.rs-Traits + REST triggern `docs/features/F14-rebooking.md`
  + `docs/architecture/03-data-model.md` + `02-service-tiers.md`.
- `CLAUDE.md` (root) — Fat-Backend-Prinzip; jj-VCS-Konvention.

### Repo-Präzedenzen
- `service_impl/src/reporting.rs` — Ziel-Standort für pure fn
  `committed_voluntary_target_for_year` (+ ggf. Pro-Rata-Helper).
- `service_impl/src/extra_hours.rs` — Beispiel für `Authentication::Full` internal-caller
  pattern (MEMORY `reference_toggle_service_full_context_bypass`).
- `service_impl/src/booking_information.rs` — BL-liest-BL-Präzedenz.
- `service/src/extra_hours.rs` — `ExtraHoursCategory::VolunteerWork` +
  `::ExtraWork` (Pair-Ziele in Phase 55, hier nur für Marker-Filter).
- `service_impl/src/billing_period_report.rs::CURRENT_SNAPSHOT_SCHEMA_VERSION` —
  aktueller Wert 12 (unverändert in Phase 54).
- Migrations-Präzedenz: `migrations/sqlite/20260704000001_seed-shortday-slot-clipping-toggle.sql`
  (Toggle-Seed neben Data-Model-Migration, v2.4 SHC-04).
- Report-DTO-Präzedenz: v2.5 `EmployeeReportTO`-Additive mit `#[serde(default)]`.
- HR-Only-DTO-Redaction-Präzedenz: v1.8 VAC-OFFSET-01 (`Option<f32> = None`).

### Frontend-Konventionen (aus Memory)
- `shifty-dioxus/Dioxus.toml` — neuer `[[web.proxy]]` für F1/F2-Endpoint
  (MEMORY `feedback_dioxus_proxy_for_new_backend_endpoints`).
- dx-CLI-Version bleibt 0.6.x (MEMORY `project_frontend_dx_version_pin`).
- Screenshots via html2canvas — nicht CDP (MEMORY `reference_dioxus_screenshots_html2canvas`).
- Dioxus-Input-Verify: Reports via `get_page_text` + `find`, nicht Screenshot
  (MEMORY `reference_dioxus_browser_verify_reports`).

### Docs-Freshness-Ziele (neu/update in Phase 54)
- `docs/features/F14-rebooking.md` **(NEU)** + `F14-rebooking_de.md` **(NEU)** —
  Rebooking-Domäne inkl. F1/F2/F3/F4/F5-Übersicht (F3/F4/F5 initial „shipped in
  Phase 55/56"), Marker-Filter-Regel, Batch-Struktur.
- `docs/architecture/02-service-tiers.md` + `diagrams/service-graph-runtime.mmd` —
  neu: `RebookingBatchService` (Basic), `VoluntaryStatsService` (BL).
- `docs/architecture/03-data-model.md` + `diagrams/db-schema-er.mmd` — neu:
  `rebooking_batch`, `rebooking_batch_entry`; Spalte `source` auf `extra_hours`;
  Toggle-Row-Seed dokumentieren.
- `docs/features/F07-reporting-balance.md` + `_de.md` — Balance-Chain sieht
  Rebooking-Marker-Filter (in Phase 54 sichtbar für Reader, aber kein Schreiber —
  narrativ „Filter respektiert, Schreiber ab Phase 55").
- `docs/features/F08-billing-period.md` + `_de.md` — narrative Note zu
  ExtraWork-aus-Rebooking (Schreiber ab Phase 55). Kein Snapshot-Bump in Phase 54.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`ExtraHoursService`** — nur Read-Zugriff in Phase 54 (Marker-Filter). Kein Create-Pfad.
- **`WorkingHoursService::get_working_hours_for_week`** — Basis für D-F2-01 pro-rata.
  Rückgabe ist bereits per-Woche, aber der Wechsel-Detail-Grain kommt aus
  `working_hours`-Row-Liste. Planer prüft, ob ein neuer Aggregator nötig ist oder
  ob bestehender Helper Tage-Info liefert.
- **`ToggleService`** — für Seed `voluntary_rebooking_auto_active_from`. Kein
  Consumer in Phase 54, nur Seed. Präzedenz v2.4 SHC-04 mit exakt derselben
  Seed-Migration-Struktur.
- **`ReportingService`** — bleibt bestehen. `voluntary_hours` als Ist-Ø kommt in den
  Employee-Year-Report; Planer entscheidet, ob F1-Ist direkt via
  `VoluntaryStatsService` in `EmployeeReport` gemergt wird oder als separater DTO.
- **`PermissionService`** — HR-Gate auf `VoluntaryStatsService`-Methoden (Präzedenz
  AVG-01 v2.1 mit `check_permission`).
- **`ShiftyDate` / `ShiftyWeek` + `_iso_year`-Helper** aus v2.5 WOP-Follow-up-#3 —
  ISO-Wochen-Arithmetik für Nenner-Wochen-Iteration. Kein `chrono`.
- **`TransactionDao`** + `use_transaction` — für Migrations-Auswirkung und
  für Property-Test-Setup (Test schreibt Marker-Row in Tx-Umgebung).

### Established Patterns
- **`gen_service_impl!`-DI-Graph** — `RebookingBatchService` neu; DI-Wiring in
  `shifty_bin/src/main.rs` unter Basic-Services vor BL-Services (Service-Tier-
  Convention).
- **HR-Only DTO Redaction:** `Option<f32> = None` (VAC-OFFSET-01 v1.8, VOL-STAT-02).
  Response-Serialisierung guarded via `#[serde(default)]` (v2.5 Additive-DTO).
- **API-Level-Redaction, nicht FE:** `PermissionService::has_privilege(Hr)`-Check
  im Service, DTO-Feld `None` bei fehlender Berechtigung. FE zeigt Row schlicht
  nicht, wenn Feld `None` (Nullable-Guard).
- **Soft-Delete-Konvention:** `WHERE deleted IS NULL` auf beiden neuen Tabellen +
  Filter in DAO-Queries. Optimistic-Lock `version BLOB` + `update_process TEXT`.
- **Additive Migration mit `DEFAULT`:** `ALTER TABLE extra_hours ADD COLUMN source
  TEXT NOT NULL DEFAULT 'manual'` — Bestehende Rows bekommen `'manual'` automatisch,
  keine separate Backfill-Migration nötig.
- **Property-Test-Framework:** existierende Property-Tests unter
  `service_impl/src/test/` (v2.5 VAA-04 als Vorlage für „Balance-Neutralität").

### Integration Points
- **`shifty_bin/src/main.rs`** — 2 neue Service-Konstruktionen: erst
  `RebookingBatchService` (Basic), dann `VoluntaryStatsService` (BL, konsumiert
  Basic + `ExtraHoursService` + `WorkingHoursService`). Beide Wire-Ups vor allen
  BL-Consumern (keine derzeit — Phase 55 wird konsumieren).
- **`rest/src/`** — entweder additive Response auf `reporting.rs` (bestehender
  Employee-Year-Report-Handler) ODER neues Modul `voluntary_stats.rs` mit dediziertem
  Endpoint. `#[utoipa::path]` obligatorisch.
- **`rest-types/src/employee_report.rs`** — `EmployeeReportTO`-Additive-Felder
  `voluntary_hours_per_contract_week`, `committed_voluntary_target`,
  `voluntary_balance` (alle `Option<f32>`, HR-Only). Alternativ neues `VoluntaryStatsTO`.
- **`shifty-dioxus/src/state/employee_report.rs`** — Feld-Ergänzung + thin
  `From<&…TO>`-Mapper, keine FE-Arithmetik.
- **`shifty-dioxus/src/component/`** — neue Komponente
  `voluntary_stats_row.rs` unter Freiwilligen-Sektion in `employee_details.rs`.
- **`shifty-dioxus/Dioxus.toml`** — `[[web.proxy]]` für neuen Endpoint (falls
  eigener Endpoint gewählt).
- **`shifty-dioxus/i18n/{de,en,cs}/*.ftl`** — 3–5 neue Row-Labels.
- **`migrations/sqlite/`** — 2–3 neue Files:
  `YYYYMMDDHHMMSS_create_rebooking_batch.sql`,
  `YYYYMMDDHHMMSS_add_source_column_to_extra_hours.sql`,
  `YYYYMMDDHHMMSS_seed_voluntary_rebooking_auto_active_from.sql`. Genaue Aufteilung
  entscheidet Planer. `cargo sqlx prepare --workspace` nach jeder query!-Ergänzung
  (Memory `reference_sqlx_prepare_after_new_query`).

</code_context>

<specifics>
## Specific Ideas

- **D-F1-01 explizit:** „Wochen mit Arbeitsvertrag, `expected_hours` darf 0 sein"
  — Kernaussage des Users. Nicht auf `expected_hours > 0` verengen.
- **D-F2-01 explizit:** pro-rata — anteilig je nach Tagen unter jedem Kontrakt.
  Bewusste Abweichung von der Research-Empfehlung „reuse latest-active-Semantik".
  Planer muss einen neuen Aggregator bauen; existierender `get_working_hours_for_week`
  liefert die Contract-Slots, aber die anteilige Tages-Verteilung ist neu.
- **D-54-DM-01 explizit:** globale Wochen-Sperre über alle Kinds — Claim-on-Suggest
  fällt direkt aus dem Constraint raus.
- **D-54-DM-02 explizit:** STRING-Enum `source TEXT NOT NULL DEFAULT 'manual'` —
  keine FK, kein NULL-JOIN, einfacher Filter.

</specifics>

<deferred>
## Deferred Ideas

- **Snapshot-Schema-Version-Bump 12→13** — Entscheidung REB-AUTO-05, in
  Phase-56-discuss-phase. Beweislast beim „Nein"-Zweig = Straddling-Golden-Snapshot.
- **F5-Reject-Wochen-Slot-Freigabe** — wenn `hr_suggestion(state=rejected)` die
  UNIQUE-Wochen-Slot blockiert oder freigibt. Beobachtbar in Phase 55 beim ersten
  Rebooking-Trigger-Schreiber.
- **F5-Stale-Vorschlag-Strategie** (Fingerprint vs. Claim-on-Suggest) — Research
  empfiehlt Claim-on-Suggest (Slot direkt beanspruchen via UNIQUE). Aber Feindetails
  sind Phase-55/56-Territorium.
- **F4-Cron-Cadence + Uhrzeit** — Research schlägt Montag 03:00 vor; Konflikt mit
  `PdfExportScheduler` in Phase-56-discuss-phase klären.
- **UI: Voluntary-Konto-Historie / Batch-Timeline** — kein FE-Sichtbarkeit auf
  Batch-Objekte in Phase 54 (nur F1-Ist / F2-Soll / F2-Delta). Batch-Historie wäre
  v2.7+-Erweiterung.
- **Employee-Self-Service-View des Freiwilligen-Kontos** — bleibt HR-only in v2.6,
  defer v2.7+.
- **Multi-Role-Approval-Workflow / Notifications / Undo** — alle explizit defer
  v2.7+ (FEATURES.md-Konsens).

</deferred>

---

*Phase: 54-data-model-voluntary-stats*
*Context gathered: 2026-07-06*
