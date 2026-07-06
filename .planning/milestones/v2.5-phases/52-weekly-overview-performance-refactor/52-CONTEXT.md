# Phase 52: weekly-overview-performance-refactor - Context

**Gathered:** 2026-07-05
**Status:** Ready for planning
**Mode:** discuss (Textform, 6 Gray Areas — alle „Claude's Discretion" auf User-Wunsch: „einfach schnell, Ergebnis unverändert")

<domain>
## Phase Boundary

Reine Performance-Umbau von `BookingInformationServiceImpl::get_weekly_summary`
(`service_impl/src/booking_information.rs:259`). Ziel: Endpoint
`GET /booking-information/weekly-resource-report/{year}` antwortet auf Dev-DB in
<500 ms (heute mehrere Sekunden), das Ergebnis
(`Arc<[WeeklySummary]>`) bleibt **byte-identisch** zur alten Wochen-Iteration.

**Zwei orthogonale Umbauten** in einer Phase:

1. **Chirurgisch (niedriges Risiko):** Load-once für `special_days`,
   `shiftplan_reports`, und `slot_service.get_slots` — analog zum bereits
   existierenden `all_work_details`/`all_absences`-Muster in derselben Methode
   (Zeile 291/300). ~110 sequenzielle Service-Calls fallen dadurch weg.

2. **Zentraler Service (höheres Risiko):** Neue
   `reporting_service.get_year(year, ...)`-Aggregation, die die ~55
   sequenziellen `get_week`-Calls durch **einen** Bulk-Load pro Jahr ersetzt.
   Interne Trait-Implementierung nutzt gemeinsamen Helper `assemble_weeks`,
   den `get_week` ebenfalls konsumiert (single source of truth, keine
   Duplikation, keine Perf-Regression an `/report/week/{year}/{week}`).

**Nicht-Ziele (aus REQUIREMENTS.md v2.5):** Kein Cache/ETag/Snapshot,
kein Snapshot-Schema-Bump (bleibt 12), keine Migration, kein neuer
Cargo-Dep, keine Änderung an der Verfügbarkeits-Berechnung für
Freiwillige (VFA-01 whole-week-out greift weiter), keine
`get_year`-Erweiterung anderer Services außerhalb des Weekly-Summary-
Use-Case, kein Frontend-Redesign, keine Signatur-Änderung an bestehenden
Trait-Methoden.

**Freiwilligen-Absencen-Anzeige (VAA-01..04) ist Phase 53** — Phase 52
lässt die neue Assembly so strukturiert, dass VAA-01 später eine reine
zusätzliche Feld-Belegung im DTO ist.
</domain>

<spec_lock>
## Locked Requirements (REQUIREMENTS.md v2.5)

**MUST READ vor Planning:** `.planning/REQUIREMENTS.md` §"Weekly-Overview Performance (WOP)".

- **WOP-01:** Bulk-Load für `special_days` + `shiftplan_reports` (Load-once-Muster analog `all_work_details`/`all_absences`). Ergebnis unverändert.
- **WOP-02:** Neue `reporting_service.get_year(year, ...)`-Aggregation ersetzt die ~55 `get_week`-Calls. Alle Semantik-Invarianten bleiben (Balance-Formel, CVC-06 Cap-Gating, Chain-C-Legacy-Filter unter `shortday_gate.active_from`, ShortDay-Slot-Clipping). Alte `get_week` bleibt erhalten (Signatur nicht entfernen).
- **WOP-03:** Property-/Regressions-Test in `service_impl/src/test/` beweist byte-identisches Ergebnis über Feiertage, ShortDays, Freiwilligen-Absencen, CVC-06-Cap, `shortday_gate.active_from` on/off. Diff-Toleranz 0 (bit-exakt via `f32::to_bits()`).
- **WOP-04:** End-to-End-Latenz `GET /booking-information/weekly-resource-report/{year}` < 500 ms auf Dev-DB. Messmethode + Referenz-Datensatz im PLAN.
- **WOP-05:** Alle bestehenden Tests grün, insbesondere `booking_information.rs`, `booking_information_chain_c.rs`, alle Reporting-Tests. `cargo test --workspace` + `cargo clippy --workspace -- -D warnings` grün.

**Erfolgskriterien:** Snapshot bleibt 12; `special_days`/`shiftplan_reports`-Calls sind pro Endpoint-Abruf exakt 1 (statt ~55); kein Snapshot-Bump.
</spec_lock>

<decisions>
## Implementation Decisions

### G1 — Scope des Bulk-Loads: Slot-Batching mit rein

- **D-52-01 (G1-b, Claude's Discretion):** Bulk-Load-Scope **erweitert um Slots**.
  Neben `special_days.get_by_year` (existiert schon) und neuem
  `shiftplan_report_service.extract_shiftplan_report_for_year` wird
  `slot_service.get_slots` (bereits vorhandene Methode, liefert alle Slots)
  **einmal** vor der Wochen-Schleife geladen. Der bisherige Filter
  `valid_from`/`valid_to` gegen `year/week` wird als reines In-Memory-Filter
  gegen den einmal geladenen Slot-Vektor umgesetzt — Semantik identisch zu
  `SlotDao::get_slots_for_week_all_plans`, aber ~55 DAO-Roundtrips gespart.
  **Begründung:** Der 500 ms-Zielwert (WOP-04) wäre ohne Slot-Batching evtl.
  knapp; die Erweiterung kostet keine neue Trait-Methode (`get_slots` existiert)
  und ist symmetrisch zum bestehenden Load-once-Muster.

### G2 — `reporting_service.get_year` Return-Shape

- **D-52-02 (G2-a, Claude's Discretion):** Signatur
  `async fn get_year(&self, year: u32, context: Authentication<Self::Context>, tx: Option<Self::Transaction>) -> Result<Arc<[(u8, Arc<[ShortEmployeeReport]>)]>, ServiceError>`.
  Der einzige Konsument ist eine strikt lineare Wochen-Iteration; ein Vec
  geordnet nach `calendar_week` matcht den Access-Pattern und ist billiger als
  eine HashMap. Kein neues Struct — hält den Trait minimal und vermeidet
  API-Aufblähen (Nicht-Ziel aus REQUIREMENTS.md).
- **D-52-03:** Reihenfolge im Vec ist strikt aufsteigend nach `calendar_week`
  (1..=weeks_in_year(year)). Wochen ohne Employee-Reports werden mit leerem
  inneren `Arc<[]>` geliefert (nicht ausgelassen), damit der Consumer den Index
  direkt nutzen kann.

### G3 — Spillover-Wochen (weeks_in_year + 3 → nächstes Jahr)

- **D-52-04 (G3-a, Claude's Discretion):** `get_year` nimmt genau **ein**
  `year`. `get_weekly_summary` ruft es zweimal: `get_year(year)` +
  `get_year(year+1)` und iteriert dann `1..=(weeks_in_year+3)` mit Fallthrough
  auf das nächste Jahr (wie heute). Analog werden `special_day_service.get_by_year`
  und `shiftplan_report_service.extract_shiftplan_report_for_year` für year
  und year+1 gerufen (2 Roundtrips pro Bulk statt einem, aber weiterhin
  konstant — nicht linear in Wochen). `slot_service.get_slots` liefert
  eh alle Slots (unabhängig vom Jahr) und wird nur einmal gerufen.
- **D-52-05:** Keine `get_year_range`/`extra_weeks_next_year`-Sondersignatur —
  das würde die API für einen einzigen Konsument verkomplizieren. Zwei
  separate `get_year`-Calls sind trivial und lesbar.

### G4 — Neue `_for_year`-Methoden auf Trait

- **D-52-06 (G4-a, Claude's Discretion):** Neue Trait-Methoden werden nur
  dort ergänzt, wo `_for_week` bereits existiert und keine Jahres-Variante
  da ist:
  - `SpecialDayService::get_by_year` — **existiert bereits**, wird verwendet.
    Keine Änderung.
  - `ShiftplanReportService::extract_shiftplan_report_for_year(year, ...)` —
    **wird neu ergänzt** (symmetrisch zu `_for_week`, DAO-Query auf
    `year` statt `year/week`).
  - `SlotService`: keine neue Methode — bestehendes `get_slots` liefert alle
    Slots, In-Memory-Filter im Consumer.
  - `ReportingService::get_year` — **wird neu ergänzt** (WOP-02).
- **D-52-07:** Keine `_for_range(from_date, to_date)`-Erweiterungen — die
  Symmetrie `_for_week`/`_for_year` ist im bestehenden Trait-Style etabliert
  (`special_days` hat es bereits so) und minimiert die Abweichung vom Muster.
  `extract_shiftplan_report(sp_id, from, to)` existiert schon für den
  Range-Use-Case (Reporting-Employee-Range).

### G5 — `reporting_service.get_year` Innenleben (Refactor-Strategie für Duplikation)

- **D-52-08 (G6-c, Claude's Discretion — Umnummerierung G6→G5 im finalen Doc):**
  `get_week` und `get_year` delegieren beide auf einen **gemeinsamen internen
  Helper** `assemble_weeks(weeks: &[(u32, u8)], work_details, shiftplan_reports,
  extra_hours) -> Vec<(u8, Arc<[ShortEmployeeReport]>)>` in
  `service_impl/src/reporting.rs`. Der Helper bekommt bereits vorgeladene
  Kollektionen als Slice-Referenzen und aggregiert per-Person-per-Woche daraus
  (kein DAO-Zugriff im Helper). `get_week` wird intern auf das Muster
  „lade 3 Kollektionen für 1 Woche, ruf assemble_weeks mit 1-Element-Vec auf"
  umgestellt — **keine Perf-Regression** an `/report/week/{year}/{week}`,
  weil dort ein Aufruf pro Endpoint eh nur eine Woche lädt.
- **D-52-09 (Semantik-Invarianten des Helpers, MUST-preserve):**
  - Balance-Formel (`dynamic_hours` = shiftplan-hours + extra-hours available - expected)
  - CVC-06 Cap-Gating (`cap_planned_hours_to_expected || expected_hours == 0.0`)
  - Chain-C-Legacy-Filter unter `shortday_gate.active_from` — **wird im
    Helper NICHT gelesen**, weil `get_week` heute den Toggle auch nicht liest
    (Toggle-Read passiert ausschließlich im `booking_information.get_weekly_summary`
    für's Slot-Clipping — siehe D-51-06/07).
  - ExtraHours-Kategorien-Aufsplittung (Vacation, SickLeave, Holiday,
    Unavailable, UnpaidLeave, VolunteerWork, CustomAbsence).
- **D-52-10:** `get_week` bleibt Public-Trait-Methode (WOP-02, wegen REST
  `rest/src/report.rs:148`). Der interne Helper ist private (`pub(crate)`).

### G6 — Property-/Regressions-Test-Ansatz für Byte-Identität (WOP-03)

- **D-52-11 (G5-a, Claude's Discretion):** Test-Ansatz **fest kodierte
  N-Szenarien-Tabelle** (Fixtures) in
  `service_impl/src/test/booking_information_weekly_summary_year_batch.rs`.
  Jede Zeile deckt eine Achse aus REQ-WOP-03:
  1. Baseline (keine besonderen Umstände).
  2. Feiertag in Woche N (`SpecialDayType::Holiday`).
  3. ShortDay in Woche N (`SpecialDayType::ShortDay` mit `until`).
  4. Freiwilliger mit Vacation-Absence-Period, die Woche N überlappt.
  5. CVC-06 Cap aktiv (`cap_planned_hours_to_expected=true`,
     `expected_hours>0`, actual überschreitet committed).
  6. `shortday_gate.active_from = None` (Legacy off).
  7. `shortday_gate.active_from = Some(vor Woche N)` (Gate aktiv).
  8. Kombination aus 2+3+4+5+6 (Interaktions-Test).
- **D-52-12 (Vergleich):** Byte-Identität via `f32::to_bits()` pro Feld auf
  `WeeklySummary` und alle inneren `WorkingHoursPerSalesPerson`-Felder.
  NaN-Handling: `to_bits()` unterscheidet NaN-Bitpattern nicht — falls in
  einem Fixture ein NaN auftaucht (sollte nicht), assertet der Test explizit
  `!f.is_nan()` pro Feld vor `to_bits()`.
- **D-52-13:** Kein `proptest`-Dep, kein Seeded-Random — Fixtures sind
  deterministisch, klarer Failure-Modus („Fixture 5 divergiert bei Woche 10,
  Feld `committed_voluntary_hours`"), gut debuggbar.
- **D-52-14:** Der Test läuft die **alte Implementation** (kopiert als
  `legacy_get_weekly_summary` in eine Test-Helper-Datei, wird während der
  Phase nicht committed sondern nur temporär im Test — Alternative: der Test
  ist ein **Snapshot-Test** über einen deterministisch generierten Fixture-
  Output, der als `.snap`-File committed wird und beim Refactor unverändert
  bleiben muss). **Vorzug:** Snapshot-Test (kein toter Legacy-Code im Repo).
  Wenn Snapshot-Framework fehlt: Insta wäre neuer Dep → **verboten**.
  Fallback: hand-verifizierter Vec-Vergleich gegen ein hart-kodiertes
  erwartetes Ergebnis pro Fixture.

### G7 — Frontend-Impact

- **D-52-15:** **Keine.** Phase 52 ändert nur das Backend-Innenleben. Das
  DTO `WeeklySummaryTO` bleibt bit-identisch. Kein `Dioxus.toml`-Proxy-Update.
  Kein FE-Rebuild nötig.

### G8 — Messmethode für WOP-04 (<500 ms)

- **D-52-16:** Messung im PLAN dokumentiert:
  - **Dev-DB-Baseline:** aktueller Datenbestand des Users
    (`~/programming/rust/projects/shifty/shifty-backend/dev.sqlite`).
  - **Messung:** `curl -s -o /dev/null -w "%{time_total}\n"
    http://localhost:3000/booking-information/weekly-resource-report/2026`
    5 Runs, Median.
  - **Baseline vor Umbau:** einmal messen (soll Sekunden zeigen).
  - **Zielwert:** Median < 0.500 s.
  - **Falls Zielwert nicht erreicht:** Slot-Batching-Umfang und ggf.
    Pre-Warming der SQLite-Connection-Pool im PLAN-Verify prüfen.

</decisions>

<canonical_refs>
## Canonical References (MUST READ vor Planning)

- `.planning/REQUIREMENTS.md` — WOP-01..05 (Locked Requirements)
- `.planning/notes/weekly-overview-perf-analyse.md` — Hotspot-Analyse, Query-Zählung, Risiken
- `.planning/seeds/weekly-overview-perf.md` — Umbau-Skizze, harte Korrektheits-Gates
- `.planning/research/questions.md` §Q-02 — Reproduzierbarkeit der Wochen-Invarianten in `get_year`
- `service_impl/src/booking_information.rs:259` — `get_weekly_summary` Hotspot
- `service_impl/src/booking_information.rs:494` — `get_summery_for_week` (sekundärer Konsument von `get_week`, nicht angefasst)
- `service_impl/src/reporting.rs:884` — `get_week` (bleibt öffentlich, delegiert intern auf `assemble_weeks`)
- `service/src/reporting.rs:397` — `ReportingService`-Trait (`get_year` neu)
- `service/src/shiftplan_report.rs:56` — `ShiftplanReportService`-Trait (`extract_shiftplan_report_for_year` neu)
- `service/src/special_days.rs:91` — `get_by_year` **existiert schon**
- `service/src/slot.rs:104` — `get_slots` (bestehend, wird für Load-once verwendet)
- `rest/src/report.rs:148` — externer Konsument von `reporting.get_week` (Regression verhindern)
- `service_impl/src/permission.rs` — Chain-C-Toggle-Read (nur in `booking_information`, nicht in `reporting`)
- `docs/features/F07-reporting-balance.md` + `docs/features/F07-reporting-balance_de.md` — Balance-Formel-Doku (Docs-Freshness-Gate: Trigger `reporting.rs`)
- Prior CONTEXT: `.planning/milestones/v2.4-phases/*/51-CONTEXT.md` (Kurzer-Tag-Slot-Clipping, D-51-06/07 Toggle-Read-Muster)

</canonical_refs>

<code_context>
## Reusable Assets & Patterns

### Bestehendes Load-once-Muster (Vorbild)

`service_impl/src/booking_information.rs:291-303`:

```rust
let all_work_details = self
    .employee_work_details_service
    .all(Authentication::Full, tx.clone().into())
    .await?;
let all_absences = self
    .absence_service
    .find_all(Authentication::Full, tx.clone().into())
    .await?;
```

Neue Bulk-Loads werden **direkt daneben** eingefügt, gleiche Auth
(`Authentication::Full`), gleiche `tx.clone().into()`-Konvention.

### Toggle-Read via `shortday_gate::read_active_from` (D-51-06/07)

`service_impl/src/booking_information.rs:309`. **Wird nicht angefasst.** Der
Toggle bleibt Sache von `get_weekly_summary` (Slot-Clipping), nicht von
`reporting.get_year`. Der Refactor darf diese Trennung nicht auflösen.

### Volunteer-Absence-Set pro Woche (VFA-01)

`service_impl/src/booking_information.rs:317-343`. **Wird nicht angefasst.**
`all_absences` ist schon load-once; die Per-Woche-Filterung bleibt inline.

### `shiftplan_report.extract_shiftplan_report(sp_id, from, to)` (Range-Existenz)

`service/src/shiftplan_report.rs:39-46` liefert bereits eine
`(sales_person_id, from, to)`-Variante. Die neue `_for_year` ist eine
DAO-effizientere Batch-Variante (keine Personen-Iteration), aber semantisch
äquivalent zu einem Range über das ganze Jahr für alle Personen.

### DAO-Query-Style für neue `_for_year`

`dao_impl_sqlite/src/shiftplan_report.rs` (bestehende `_for_week`-Query):
Filter `WHERE year = ? AND calendar_week = ?` wird zu `WHERE year = ?`.
`sqlx prepare` nach dem Hinzufügen: `cargo sqlx prepare --workspace` +
`.sqlx` committen (Memory-Erinnerung `reference_sqlx_prepare_after_new_query.md`).

### Service-Tier-Klassifizierung

- `ReportingService`, `ShiftplanReportService`, `SlotService`,
  `SpecialDayService`: alle bereits **Basic** (Entity-Manager). Neue
  `_for_year`-Methoden ändern die Tier-Einordnung nicht.
- `BookingInformationService`: bleibt **Business-Logic** (konsumiert
  weiterhin Basic-Services).

### Docs-Freshness-Gate (`CLAUDE.md`)

Trigger-Dateien in dieser Phase:
- `service/**/*.rs` (Trait-Signatur `get_year`, `_for_year`) → `docs/features/F07-reporting-balance.md` **prüfen**, evtl. Änderungshinweis in „Randfälle"-Sektion (die Balance-Formel bleibt aber unverändert — vermutlich nur Kommentar-Update oder gar kein Doku-Impact).
- `dao/**/*.rs` (falls Trait-Erweiterung) → passende Feature-Doku-Sektion prüfen.
- **Keine Migration** → kein `docs/architecture/03-data-model.md`-Update.
- **Kein Auth-Change** → kein `docs/architecture/04-auth.md`-Update.
- Recommend: Planner prüft am Ende, ob `F07-reporting-balance.md/de.md`
  angefasst werden muss. Wenn die Formel unverändert bleibt, reicht ein
  Hinweis „Refactor Phase 52: `get_year` als Batch-Variante ergänzt, Formel
  unverändert" in einer Änderungshistorie-Sektion — oder gar nichts,
  wenn die Doku keine Trait-Signaturen zitiert.

</code_context>

<downstream_hooks>
## Für gsd-phase-researcher

**Nicht mehr recherchieren (in CONTEXT.md entschieden):**

- Return-Shape von `get_year` (D-52-02: `Arc<[(u8, Arc<[ShortEmployeeReport]>)]>`).
- Spillover-Handling (D-52-04: zweimal `get_year`).
- Neue Trait-Methoden-Liste (D-52-06: nur `get_year` + `extract_shiftplan_report_for_year`).
- Interne Helper-Struktur (D-52-08: `assemble_weeks`).
- Test-Ansatz (D-52-11..14: Fixture-Tabelle + `f32::to_bits()`).

**Noch zu recherchieren / im PLAN zu klären:**

- Q-02.3 (CVC-06 Cap-Semantik): Ist per-Person-Cap mathematisch identisch,
  ob pro-Woche oder pro-Jahr berechnet? **Vermutlich ja** (Cap ist per
  `working_hours`-Row, die eh per-Woche gefiltert wird), aber der
  Researcher soll das im Detail durchgehen und im RESEARCH.md notieren.
- Genaue interne Struktur von `assemble_weeks`: welche
  HashMap-Indexierungen (per `(sales_person_id, calendar_week)` als Key?),
  wie viele Traversals pro Woche.
- Datenbank-Index-Check: sind
  `shiftplan_report_daily.year`,
  `working_hours.year`, `extra_hours.year` indiziert für die Year-Batch-Query?
  Wenn nicht, evtl. minimaler `CREATE INDEX` nötig (dann DOCH Migration →
  Rücksprache mit User).

## Für gsd-planner

**Task-Reihenfolge (Vorschlag):**

1. `assemble_weeks`-Helper extrahieren aus bestehendem `get_week` (reiner
   Refactor, Tests weiter grün, kein Verhaltens-Change).
2. `ReportingService::get_year` + Impl auf Helper (WOP-02).
3. `ShiftplanReportService::extract_shiftplan_report_for_year` + DAO
   (WOP-01 Teil).
4. `get_weekly_summary` umbauen: Bulk-Loads + Iteration über `get_year`-Vec
   statt per-Week (WOP-01 Rest).
5. Fixture-Test schreiben (WOP-03). **HARTES GATE** vor jedem Semantik-
   ändernden Refactor.
6. Latenz-Messung + Verify (WOP-04).
7. Docs-Freshness-Check (F07).

**Atomarität:** Der Umbau in `get_weekly_summary` (Schritt 4) wechselt das
Iterations-Muster. Fixture-Test muss **vor** Schritt 4 grün sein (mit
alter Impl als Baseline) und **nach** Schritt 4 weiter grün. Falls
Snapshot-Framework nicht existiert: Test ist als hart-kodierter
Vec-Vergleich implementiert (D-52-14 Fallback).

**Nicht-Ziele im Plan explizit ausschließen:**

- Kein VAA-Vorgriff (Phase 53).
- Kein Cache/ETag.
- Kein Snapshot-Bump.
- Keine neue Cargo-Dep.
- Keine `_for_range`-Alternativsignatur.

## Für gsd-executor

**Test-Gate:** `cargo test --workspace` + `cargo clippy --workspace -- -D
warnings` (aus `shifty-backend/` Backend-Shell — nicht aus shifty-dioxus-
Shell, Memory `reference_dioxus_clippy_not_gated.md`).

**sqlx prepare:** Nach jeder neuen `query!`/`query_as!` (mindestens für
`extract_shiftplan_report_for_year`-DAO) muss `cargo sqlx prepare
--workspace` laufen + `.sqlx`-Delta committed werden. Memory-Erinnerung:
`reference_sqlx_prepare_after_new_query.md`.

**Docs-Freshness:** Reine Refactor-Phase mit unveränderter Balance-Formel
und unveränderter Auth-Semantik — vermutlich kein Doku-Update nötig.
Executor entscheidet am Ende.

</downstream_hooks>

<deferred>
## Deferred Ideas (Noted for Later, out of scope)

- **VAA-01..04 (Phase 53):** Freiwilligen-Absencen in `sales_person_absences`
  anzeigen — baut auf der neuen Assembly auf, nicht Teil dieser Phase.
- **Slot-Filter `valid_from`/`valid_to` als DB-Index prüfen** — wenn
  In-Memory-Filter über alle Slots zu langsam wird (unwahrscheinlich, Slots
  sind wenige), separater Maintenance-Task.
- **`SlotService::get_slots_for_year`-Batch-Methode** — bewusst NICHT
  eingeführt (Trait-Aufblähen auf Verdacht ist Nicht-Ziel). Wenn zukünftige
  Konsumenten einen Jahres-Slot-Bulk brauchen, kann das später ergänzt werden.
- **HTTP-Caching / ETag** — bewusst verworfen wegen Live-Korrektheit.
- **Parallelisierung via `join_all`** — SQLite serialisiert intern,
  marginal. Nicht Teil dieser Phase.

</deferred>

<user_signals>
## User Signals (aus Discussion)

- **User: „Ich finde, du kannst alles entscheiden. Es soll einfach schnell
  sein, aber das Ergebnis sollte unverändert sein"** — grünes Licht für alle
  Recommends. Prio: (1) Latenz, (2) Byte-Identität. Alle anderen Trade-offs
  werden zu Gunsten dieser beiden aufgelöst.
- **Fat Backend, Thin Client** (feedback_fat_backend_thin_client.md) —
  bestätigt: Frontend bleibt komplett unangetastet, alle Aggregation im
  Backend.
- **Clippy-Gate** (feedback_clippy_gate.md) — muss im Executor-Plan drinstehen.
- **sqlx-prepare** (reference_sqlx_prepare_after_new_query.md) — muss im
  Executor-Plan drinstehen (neue DAO-Query!).
</user_signals>