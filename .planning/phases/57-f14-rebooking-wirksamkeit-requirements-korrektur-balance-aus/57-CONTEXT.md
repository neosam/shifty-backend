# Phase 57: F14-Rebooking-Wirksamkeit — Requirements-Korrektur + Balance-Ausgleich - Context

**Gathered:** 2026-07-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Der F14-Übertrag freiwillig→bezahlt wird auf der **Leseseite** wirksam: nach einer
Umbuchung ist die Balance um den umgebuchten Betrag höher und das Ehrenamt-Ist um
denselben Betrag niedriger. Die Schreibseite (Datenmodell, F3 manuell, F4 Cron,
F5 HR-Alert) existiert vollständig aus den Phasen 54–56 und wird nur an einer
Stelle korrigiert (Balance-Deckel im Auto-Cron).

**Reihenfolge ist bindend: Spec vor Code.** Ohne Requirements-Korrektur
reproduziert die nächste Phase den Fehler aus der Spec heraus.

**In Scope:**

1. VOL-ACCT-03 neu fassen + neue REQs für Ausgleichswirkung und Balance-Deckel
2. `source`-Filter an den 4 Report-Aggregaten site-spezifisch korrigieren
3. Balance-Deckel in `run_for_week` / `week_excess`
4. Neutralitäts-Tests invertieren (von „bit-identisch" zu „Balance +h, Ehrenamt −h")
5. Snapshot-Schema-Bump 12→13 + Straddling-Fixture invertieren
6. Docs-Freshness-Gate: F14 + F07 + F08, je `.md` und `_de.md`

**Out of Scope:**

- Änderung der HR-Vorschlagsformel `proposed_hours = min(|balance|, ist)` — bleibt wie sie ist
- Entkopplung von D-55-07 (Banner-Erlöschen) — vom User explizit verworfen, siehe D-57-04
- Undo/Rollback von Batches (REB-UNDO-01, v2.7+)
- Bereinigung der Alt-Inkonsistenz zwischen `assemble_weeks` und `hours_per_week`
  bezüglich VolunteerWork-in-Balance (siehe RISK-01 — nur nicht verschlimmern)

</domain>

<decisions>
## Implementation Decisions

### Fachliches Sollverhalten (User-bestätigt, 2026-07-31)

- **D-57-01 — Zwei Untergrenzen, zwei Zuständigkeiten.** Der Übertrag zerfällt in
  einen unstrittigen und einen strittigen Teil:

  - **Automatik (Wochen-Cron, ohne HR):** gleicht die bezahlte Balance aus, aber nur
    so weit, wie der Mitarbeiter seine **Freiwilligen-Zusage noch erfüllt**. Formel:
    `min( max(ist − committed, 0), max(−balance, 0) )`. Die Balance steigt durch die
    Automatik **nie über 0**.
  - **HR-Vorschlag (F5, manuell bestätigt):** darf die Zusage **unterschreiten** —
    genau darin liegt der Sinn des Features. Untergrenze ist hier, dass die
    **freiwilligen Stunden im Jahr nicht unter 0** fallen („man kann nicht insgesamt
    −3 Stunden freiwillig arbeiten"). Formel `min(|balance|, ist)` erfüllt das
    bereits und bleibt **unverändert**.

  Wörtlich vom User: „Die paid-balance soll automatisch 1× die Woche ausgeglichen
  werden, solange die volunteer-balance nicht unter 0 geht. Sie darf bis 0 gehen.
  Alles darüber soll manuell in der UI an HR vorgeschlagen werden, aber bei den
  Vorschlägen sollen die insgesamten freiwilligen Stunden nicht unter 0 gehen."

  „volunteer-balance" = Delta `Ist − committed_voluntary`. „insgesamte freiwillige
  Stunden" = Jahres-Ist (VOL-STAT-01).

- **D-57-02 — Referenz-Szenario für alle Tests.** Vertrag 5 h bezahlt + 5 h
  freiwillig/Woche, 2 Wochen. Woche 1: 3 h, Woche 2: 17 h → 20 h gearbeitet bei
  20 h zugesagt. Shifty verbucht heute 8 h bezahlt / 12 h freiwillig, Balance −2.
  Nach korrektem Übertrag von 2 h: 10 h / 10 h, Balance 0.
  **Gegenprobe Übererfüllung:** Woche 2 mit 20 h → 23 h gearbeitet, Überschuss 5,
  Übertrag trotzdem nur **2** (Balance-Deckel), Balance 0, 3 h bleiben freiwillig.

### Rückwirkung auf Bestandsdaten

- **D-57-03 — Die Leseänderung wirkt sofort und rückwirkend.** Kein read-seitiger
  Stichtag-Guard. Alle bestehenden Rebooking-Rows aus Phase 55/56 werden mit der
  Filterkorrektur wirksam.

  **Warum unkritisch:** Alle Rebooking-Rows stammen aus 2026 (F3 ab 2026-07-24,
  F4 ab 2026-07-26), es liegt **keine Jahresgrenze** dazwischen. `carryover_hours`
  wird erst zum Jahreswechsel aus `balance_hours` geschrieben
  (`shiftplan_edit.rs:384`) — es existiert also kein stale Carryover, der nachgezogen
  werden müsste.

  **Konsequenz für den Planner:** Falls die Verifikation doch Rebooking-Rows aus
  ≤2025 findet (nicht erwartet), ist das ein Blocker und muss eskaliert werden, nicht
  stillschweigend gebucht.

### HR-Alert-Banner (D-55-07 bleibt)

- **D-57-04 — Der Banner erlischt weiterhin bei Approve UND bei Reject.** Sobald HR
  entschieden hat, gibt es für diese ISO-Woche keine Frage mehr. Der belegte
  UNIQUE-Slot bleibt zweiter Erlöschungsgrund. **D-55-07 wird NICHT entkoppelt** —
  damit ist Scope-Punkt 2 der ROADMAP bewusst gestrichen.

  **Begründung (User):** „Solange bis HR auf annehmen oder ablehnen klickt. Dann hat
  HR entschieden und es gibt keine Frage mehr in der Woche."

  **Konsequenz:** Der Banner ist ein Arbeitswerkzeug für HR, **kein Wächter** gegen
  ein erneutes Versagen des Übertrags. Diese Rolle übernimmt stattdessen D-57-08
  (invertierte Tests): sie schlagen fehl, sobald sich die Balance nach einer
  Umbuchung nicht bewegt. Das war der eigentliche Zweck des ROADMAP-Punkts und ist
  damit strukturell — nicht per Sichtprüfung — abgedeckt.

  Die nächste Alert-Chance für dieselbe Person entsteht in der nächsten ISO-Woche.

### Produktionsstand

- **D-57-05 — Der Auto-Cron war nie aktiv.** `voluntary_rebooking_auto_active_from`
  wurde vom User nie gesetzt, `POST /admin/rebooking/backfill` nie mit
  `dry_run=false` gelaufen.

  **Konsequenz:** Es existieren **keine** `auto_cron`- oder `auto_cron_backfill`-Rows,
  die ohne Balance-Deckel geschrieben wurden. **Keine Bestandsdaten-Korrektur, keine
  Repair-Migration in dieser Phase.** Nur manuelle F3-Rows und ggf. F5-Approvals
  können existieren; deren Beträge wurden mit `min(|balance|, ist)` berechnet und sind
  korrekt.

  Der Planner MUSS diesen Stand vor der Umsetzung gegen die Ziel-DB verifizieren
  (Query auf `rebooking_batch.kind`), nicht blind annehmen.

### Filter-Neufassung (Claude-Entscheidung, gegen Code verifiziert)

- **D-57-06 — Site-spezifisch, NICHT pauschal streichen.** Die vier Filter-Sites
  leiten die Balance unterschiedlich her — pauschales Entfernen wäre an einer Stelle
  falsch:

  | Site | Balance-Quelle | Verhalten ohne `source`-Filter |
  |---|---|---|
  | `reporting.rs:1192` (All-Employees-Report, Summe `:1404`) | `as_report_type() == WorkingHours` | ✅ korrekt: VolunteerWork fällt per Kategorie raus, nur `+h ExtraWork` zählt |
  | `reporting.rs:1728` (`overall_extra_work_hours`) | `as_report_type() == WorkingHours` | ✅ korrekt |
  | `reporting.rs:2161` (`hours_per_week`) | Kategorie-basiert (belegt durch Test `volunteer_extra_hours_excluded_from_balance_and_overall`, `:2800`) | ✅ korrekt |
  | `reporting.rs:859` (`assemble_weeks`/Wochenübersicht, Summe `:876`) | `availability() == Available` — und `VolunteerWork.availability() == Available` (`extra_hours.rs:122`) | ❌ **falsch**: das `−h VolunteerWork`-Bein zöge die Balance wieder herunter, das Paar wäre netto 0 |

  **Entscheidung:** An `:1192`, `:1728`, `:2161` den `source == Manual`-Filter
  **ersatzlos streichen** — die Kategorie-Semantik liefert das gewünschte Verhalten
  von selbst. An `:859` gezielt nur die Rebooking-`VolunteerWork`-Rows aus der
  `extra_working_hours`-Summe (`:876`) ausnehmen; in `manual_volunteer_hours`
  (`:932`) müssen sie **drin bleiben**, damit das Ehrenamt-Ist sinkt.

  `ExtraHoursSource` bleibt als Provenienz-Marker erhalten (UI-Kennzeichnung,
  spätere Undo-Fähigkeit), verliert aber seine pauschale Aggregations-Wirkung.

- **RISK-01 (an Researcher/Planner):** Zwischen `assemble_weeks` (`:876`,
  `availability()`) und `hours_per_week` / All-Employees (`as_report_type()`)
  besteht eine **Alt-Inkonsistenz**: gewöhnliche VolunteerWork-Stunden zählen in der
  Wochenübersicht in `overall_hours`/`balance_hours`, im Jahresreport aber nicht.
  Das ist **vorbestehend und nicht Scope dieser Phase**. Phase 57 darf das Verhalten
  für Nicht-Rebooking-VolunteerWork-Rows **nicht verändern** — der Eingriff an `:876`
  muss auf `source == Rebooking` beschränkt bleiben. Wenn der Researcher zu dem
  Schluss kommt, dass die Inkonsistenz den Fix unmöglich sauber macht: eskalieren,
  nicht nebenbei mitfixen.

### Balance-Deckel im Auto-Cron (Claude-Entscheidung)

- **D-57-07 — Balance zum Ende der verarbeiteten Woche.** `run_for_week`
  (`rebooking_reconciliation.rs:618`) liest die Balance bisher bewusst nicht
  (Kommentar `:611-614`); das kippt. Der Deckel braucht die Balance **year-to-date
  bis einschließlich der verarbeiteten Woche**, inklusive Vorjahres-Carryover — nicht
  die Balance zum Laufzeitpunkt.

  **Warum:** Beim Backfill über mehrere historische Wochen würde die Laufzeit-Balance
  denselben Spielraum mehrfach vergeben. Chronologische Korrektheit schlägt
  Performance; die Automatik ist noch nicht aktiv (D-57-05), also gibt es keinen
  Bestandsdruck.

  `week_excess` wird um den Balance-Parameter erweitert:
  `min( max(ist − (soll + committed), 0), max(−balance, 0) )`, weiterhin mit dem
  bestehenden NaN/Inf-Guard (`0.0` bei nicht-endlichen Eingaben).

  **Optimierung erlaubt, nicht gefordert:** Innerhalb eines Backfill-Laufs darf die
  Balance einmal gelesen und pro gebuchter Woche inkrementell fortgeschrieben werden,
  statt pro (SalesPerson, Woche) einen vollen Report zu rechnen. Nur wenn der Planner
  die Laufzeit als Problem belegt — sonst der einfache Weg.

### Tests

- **D-57-08 — Neutralitäts-Tests invertieren, nicht löschen.**
  `service_impl/src/test/rebooking_neutrality.rs` (3 `#[tokio::test]` +
  `assert_bit_eq`) behauptet heute Bit-Identität als Erfolgskriterium und weist ein
  Fehlschlagen im Kommentar ausdrücklich als „echte Entdeckung" zurück — die Tests
  zementieren den Fehler.

  Neue Zusagen:
  - `balance_after == balance_before + h`
  - `voluntary_ist_after == voluntary_ist_before − h`
  - `voluntary_soll` (VOL-ACCT-01-Soll) **bleibt unverändert** — das Soll kommt aus
    `committed_voluntary` im Vertrag, nicht aus `extra_hours`. Dieser eine Test
    bleibt inhaltlich als Neutralitäts-Test bestehen.

  Dateiname darf bleiben, die Datei-Doku muss aber neu erklären, was „Neutralität"
  jetzt noch heißt (nur noch Soll-Schutz). `assert_bit_eq` wird für die beiden
  invertierten Fälle durch eine Delta-Assertion mit f32-Toleranz ersetzt.

  **Zusätzlich:** Ein Test für den Balance-Deckel entlang D-57-02 (Gegenprobe
  Übererfüllung: 23 h gearbeitet, Überschuss 5, Übertrag 2).

### Requirements-Struktur (Claude-Entscheidung)

- **D-57-09 — Drei REQ-Änderungen, IDs in bestehenden Kategorien.**

  - **VOL-ACCT-03 (Neufassung):** schützt nur noch die **Freiwilligen-Statistik**
    gegen Doppelzählung: Rebooking-Paare verändern **VOL-ACCT-01-Soll nicht**. Der
    Zusatz „(balance-neutral)" und der Anspruch auf unverändertes Ist entfallen
    ersatzlos. Der ⚠️-Fehlervermerk in REQUIREMENTS.md wird durch einen kurzen
    Historien-Hinweis ersetzt (nicht gelöscht — die Fehlerspur bleibt auffindbar).
  - **VOL-ACCT-04 (neu):** die Ausgleichswirkung. Nach einer Umbuchung über `h` ist
    die Balance um `h` höher und das Ehrenamt-Ist um `h` niedriger. Akzeptanz =
    Referenz-Szenario D-57-02.
  - **REB-AUTO-06 (neu):** der Balance-Deckel im Auto-Cron. Eigenes REQ, weil es
    eine eigenständige, testbare Zusage ist (Balance steigt durch die Automatik nie
    über 0) und weil die Traceability sonst den ROADMAP-Scope-Punkt 4 nicht abdeckt.

  Traceability-Tabelle in REQUIREMENTS.md um die zwei neuen Zeilen (→ Phase 57)
  ergänzen, VOL-ACCT-03 auf Phase 57 umhängen. Coverage danach 19/19.

### Snapshot-Schema (Claude-Entscheidung)

- **D-57-10 — Bump `CURRENT_SNAPSHOT_SCHEMA_VERSION` 12 → 13**
  (`billing_period_report.rs:117`). D-01 aus Phase 56 („bleibt bei 12") stützte sich
  ausdrücklich auf die Balance-Neutralität des Rebookings — die Grundlage entfällt.
  Nach CLAUDE.md ist das ein „Input-Set-Change": eine frische Neuberechnung würde für
  dieselbe Periode anders ausfallen als ein v12-Snapshot.

- **D-57-11 — Straddling-Golden-Fixture invertieren, nicht löschen.** Die Fixture aus
  D-01-BEWEISLAST fordert heute Byte-Identität über `active_from` hinweg. Sie muss
  jetzt das Gegenteil beweisen: eine Billing-Period, die eine Rebooking-Woche
  enthält, verschiebt sich um genau `±h`. Perioden **ohne** Rebooking-Woche bleiben
  byte-identisch — dieser Teil der Fixture bleibt unverändert und ist die
  Regressions-Absicherung.

- **D-57-12 — Bestehende v12-Snapshot-Rows bleiben unangetastet.** Kein Recompute,
  keine Migration der Daten. Validatoren vergleichen laut CLAUDE.md-Konvention nur
  innerhalb derselben `snapshot_schema_version`; genau dafür existiert das
  Versionsfeld. Der Bump ist die vollständige Antwort.

### Claude's Discretion

Der User hat die technischen Detailentscheidungen ausdrücklich delegiert
(„Den Rest entscheide ich selbst"). Delegiert und oben festgelegt: D-57-06
(Filter-Sites), D-57-07 (Balance-Quelle im Cron), D-57-08 (Test-Zuschnitt),
D-57-09 (REQ-IDs), D-57-10/11/12 (Snapshot). Der Planner darf diese Entscheidungen
revidieren, wenn die Recherche sie widerlegt — dann aber explizit mit Begründung im
PLAN.md, nicht stillschweigend.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase-Analyse und Requirements
- `.planning/ROADMAP.md` §"Phase 57" — Befund, Scope-Reihenfolge, betroffene Pfade,
  Referenz-Szenario. **Abweichung:** Scope-Punkt 2 (D-55-07 entkoppeln) ist per
  D-57-04 gestrichen.
- `.planning/REQUIREMENTS.md` §"Voluntary-Stundenkonto (VOL-ACCT — F2)" — VOL-ACCT-03
  inkl. ⚠️-Fehlervermerk; §"Traceability" — Tabelle, die erweitert wird.
- `.planning/phases/55-manual-rebooking-hr-alert/55-CONTEXT.md` — D-55-07
  (Banner-Erlöschen, bleibt), D-55-A1 (der eingebaute Filter), §constraints
  (UNIQUE-Slot-Semantik, Claim-on-Suggest).
- `.planning/phases/56-wochen-cron-rollout-backfill-f4/56-CONTEXT.md` — D-01 +
  D-01-BEWEISLAST (Snapshot-Entscheidung, die per D-57-10 kippt), D-02
  (Claim-on-Suggest), D-04 (Locked-Woche).

### Projekt-Konventionen (hart)
- `CLAUDE.md` §"Billing Period Snapshot Schema Versioning" — Bump-Kriterien; belegt
  D-57-10 („Change the input set the computation reads from").
- `CLAUDE.md` §"Docs-Freshness-Gate" — Trigger-Tabelle. Diese Phase triggert
  `reporting.rs` → F07, `billing_period_report.rs` → F08, `service/**` → F14.
- `CLAUDE.md` §"Key Development Notes" — Clippy ist hartes Gate:
  `cargo clippy --workspace -- -D warnings` in jedem Test-Gate, wird von
  `cargo test` NICHT abgedeckt.

### Feature-Doku (Docs-Freshness-Gate — beide Sprachen synchron halten)
- `docs/features/F14-rebooking.md` + `docs/features/F14-rebooking_de.md` — beschreibt
  aktuell das fehlerhafte Verhalten und begründet den Filter mit „Doppelzählung";
  muss auf das Sollverhalten D-57-01 umgeschrieben werden.
- `docs/features/F07-reporting-balance.md` + `_de.md` — Balance-Formel und
  Kategorie-Semantik ändern sich.
- `docs/features/F08-billing-period.md` + `_de.md` — Version-Tabelle + Bump-Historie
  um 13 ergänzen.
- `docs/domain/time-accounting.md` — Kategorie-Semantik.
- `docs/domain/billing-period.md` — Snapshot-Rationale.

### Code (Primärquellen, gegen den Ist-Stand verifiziert 2026-07-31)
- `service_impl/src/reporting.rs:859` / `:876` — Wochenübersicht, `availability()`-Summe (Sonderfall, siehe D-57-06)
- `service_impl/src/reporting.rs:1192` / `:1404` — All-Employees-Report, `as_report_type()`-Summe
- `service_impl/src/reporting.rs:1728` — `overall_extra_work_hours`
- `service_impl/src/reporting.rs:2161` — `hours_per_week`
- `service_impl/src/reporting.rs:395` — `alert_predicate`; `:405` — `proposed_hours` (**bleibt unverändert**); `:1543` — Aufruf
- `service_impl/src/reporting.rs:2800` — Test `volunteer_extra_hours_excluded_from_balance_and_overall` (belegt die Kategorie-Semantik in `hours_per_week`)
- `service/src/extra_hours.rs:91` — `as_report_type()`; `:114` — `availability()` (VolunteerWork = `Available`, Ursache des Sonderfalls); `:60` — `ExtraHoursSource`
- `service_impl/src/rebooking_reconciliation.rs:63` — `week_excess`; `:618` — Aufruf mit `soll = 0.0`; `:611-614` — Kommentar „Balance wird bewusst nicht gelesen" (kippt)
- `service_impl/src/billing_period_report.rs:117` — `CURRENT_SNAPSHOT_SCHEMA_VERSION`
- `service_impl/src/test/rebooking_neutrality.rs` — 3 Tests + `assert_bit_eq`
- `service_impl/src/test/rebooking_reconciliation.rs` — Cron-Tests
- `service_impl/src/shiftplan_edit.rs:384` — `new_carryover_hours = balance_hours` (Carryover-Kopplung, relevant für D-57-03)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`week_excess` ist bereits eine reine, testbare Funktion** mit NaN/Inf-Guard —
  der Balance-Deckel ist eine Parameter-Erweiterung, keine Umstrukturierung.
- **`alert_predicate` / `proposed_hours`** (`reporting.rs:395/405`) sind ebenfalls
  rein und getestet. Beide bleiben unverändert; `proposed_hours = min(|balance|, ist)`
  erfüllt die HR-Untergrenze aus D-57-01 bereits.
- **Straddling-Golden-Fixture-Muster** aus v2.5 Phase 52 (WOP-03, 8 Fixtures) — die
  Infrastruktur steht, nur die Zusage wird invertiert.
- **`rebook()`** (`rebooking_reconciliation.rs:107`) ist der einzige Pair-Write-Path
  (Pitfall 15). Diese Phase fasst ihn nicht an.

### Established Patterns
- **Kategorie-Semantik trägt die Fachlogik, nicht der `source`-Marker.**
  `as_report_type()` entscheidet über Balance-Relevanz, `availability()` über
  Anwesenheit. `VolunteerWork` = `Documented` + `Available` — diese Kombination ist
  der ganze Grund, warum `:876` ein Sonderfall ist.
- **Transaktionsklammer beim Aufrufer** (`rebook()` committet nicht selbst) — beim
  Ergänzen des Balance-Reads in `run_for_week` beibehalten.
- **Snapshot-Versionierung als Vergleichs-Gate**, nicht als Datenmigration
  (CLAUDE.md) — stützt D-57-12.

### Integration Points
- `run_for_week` braucht neu eine Balance-Quelle → Dependency auf den
  `ReportingService` im `RebookingReconciliationService`. **Service-Tier prüfen:**
  `RebookingReconciliationService` ist bereits Business-Logic (konsumiert andere
  Domain-Services), `ReportingService` ebenfalls — laut CLAUDE.md
  §"Service-Tier-Konventionen" zulässig, **sofern kein Zyklus entsteht**. Der Planner
  MUSS prüfen, ob `ReportingService` seinerseits auf Rebooking zeigt, und ggf. die
  DI-Reihenfolge in `shifty_bin/src/main.rs` anpassen (+ `docs/architecture/`-Update
  laut Docs-Freshness-Gate).
- Die vier Filter-Sites liegen alle in `reporting.rs` — ein Plan-Task, aber vier
  unterschiedliche Begründungen (D-57-06-Tabelle).

</code_context>

<specifics>
## Specific Ideas

- **Der User denkt in Konten, nicht in Filtern.** Formulierung im UI und in der Doku
  entsprechend: „bezahltes Stundenkonto ausgleichen", „Freiwilligen-Zusage erfüllen",
  „Minus-Ehrenamt gibt es nicht".
- **Die Warnzeile ist ein Arbeitswerkzeug, kein Monitor.** HR klickt einmal pro Woche
  und Person, dann ist Ruhe. Kein Design, das HR dieselbe Entscheidung mehrfach
  vorlegt.
- **Die Fehlersichtbarkeit gehört in die Tests, nicht ins UI.** Der Grund, warum
  dieser Bug ein ganzes Milestone überlebt hat, war ein Test, der Bit-Identität
  belohnt hat — nicht eine fehlende Warnung.

</specifics>

<deferred>
## Deferred Ideas

- **Alt-Inkonsistenz `assemble_weeks` vs. `hours_per_week`** (RISK-01): VolunteerWork
  zählt in der Wochenübersicht in die Balance, im Jahresreport nicht. Vorbestehend,
  eigener Fix, eigene Phase. Hier nur „nicht verschlimmern".
- **Undo/Rollback von Rebooking-Batches** — REB-UNDO-01, v2.7+. Das Schema trägt es
  bereits.
- **Repair-Werkzeug für zu große Auto-Cron-Paare** — durch D-57-05 gegenstandslos
  (Cron nie aktiv). Falls die Automatik jemals ohne Deckel produktiv lief, wäre das
  eine eigene Phase.

### Reviewed Todos (not folded)
Der `todo.match-phase 57`-Lauf lieferte drei Treffer, alle als
Stoppwort-Fehlalarme identifiziert („bei, der, wird, ein, eine") — keiner berührt
F14 oder das Reporting:
- `2026-05-05-warnung-eintrag-ausserhalb-vertragszeiten.md` — Shiftplan-Validierung
- `2026-05-08-cutover-ui-admin-feature.md` — Frontend/Cutover
- `2026-07-01-urlaub-beantragen-mit-genehmigung.md` — Absence-Workflow

</deferred>

---

*Phase: 57-f14-rebooking-wirksamkeit-requirements-korrektur-balance-aus*
*Context gathered: 2026-07-31*
