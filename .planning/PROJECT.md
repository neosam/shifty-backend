---
type: project_charter
last_updated: 2026-06-25
last_milestone: v1.4 Committed Voluntary Capacity (shipped 2026-06-25, Audit passed, 10/10 CVC-Requirements)
current_milestone: v1.5 Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen
---

# Shifty — Project Charter

## Was ist Shifty

Employee shift planning + HR-Management mit zwei gekoppelten Subprojekten,
beide co-located in **diesem** Repo seit 2026-05-07:

- **`/`** (Cargo-Workspace-Root): Rust-Backend (Axum, SQLite, layered architecture).
  Authoritative source für Domain-Logik, REST-API, Persistenz, Reporting.
- **`shifty-dioxus/`** (eigener kleiner Workspace): Dioxus-Frontend (WASM).
  Konsumiert das Backend ausschließlich über REST. Aus dem Cargo-Workspace
  des Backends explizit ausgeschlossen (`exclude = ["shifty-dioxus"]`).

Geteilte Crates:

- **`rest-types`**: API-DTOs. Heute in *beiden* Subprojekten dupliziert — siehe
  [Bekannte Constraints](#bekannte-constraints).

## GSD-Scope-Regel

**Phasen umfassen Backend UND Frontend.**

Jede Phase, die ein neues TO einführt oder ein bestehendes ändert, hat per
Default Frontend-Anteil. Jeder Plan muss in seinem Header explizit benennen,
welche Pfade in beiden Subprojekten betroffen sind:

```
**Backend-Pfade:**
- `service/src/...`
- `rest/src/...`
- `rest-types/src/...`

**Frontend-Pfade:**
- `shifty-dioxus/src/...`
- `shifty-dioxus/rest-types/src/...`   (bis Konsolidierung)
```

"Frontend out of scope" ist erlaubt, **braucht aber eine begründete Notiz im
DISCUSS** (z. B. „rein interne Refactor-Phase ohne API-Wirkung", „Frontend
folgt in Folge-Phase v1.X+1, getrackt in Backlog"). Eine Phase ohne sichtbare
API-Wirkung darf still ohne Frontend-Anteil laufen.

**Konsequenz für `verify-work`:** UAT muss Frontend-Pfad mitprüfen, wenn
Frontend-Anteil im Plan stand. „Backend-Tests grün" ist nicht ausreichend
für Phasen mit Frontend-Anteil.

## Quellen-Hierarchie

| Zweck | Quelle |
|---|---|
| Backend-Konventionen | `CLAUDE.md` (Repo-Root) |
| Frontend-Konventionen | `shifty-dioxus/CLAUDE.md` (Executor lädt automatisch beim Lesen von Frontend-Files) |
| Frontend-Codebase-Map | `.planning/codebase/frontend/` (separater Subordner, weil das Frontend einen eigenständigen Subprojekt-Scope hat) |
| Backend-Codebase-Map | (`CLAUDE.md` ist detailliert genug; bewusst keine `.planning/codebase/`-Map) |
| Roadmap & Phasen | `.planning/ROADMAP.md`, `.planning/phases/` |
| Lokale Dev-Conventions | `CLAUDE.local.md` (jj-only, NixOS-Spezifika) |

## Bekannte Constraints

### `rest-types`-Drift — RESOLVED in v1.2 (2026-05-07)

**Vorher (bis v1.1):**
- **Backend**: `rest-types/` v1.13.0-dev, 2041 Zeilen — single source of truth
  für Backend
- **Frontend**: `shifty-dioxus/rest-types/` v1.0.5-dev, 1468 Zeilen —
  gedrifteter Fork

Frontend kompilierte *nicht* gegen den Backend-Stand — ein neuer Match-Arm oder
Feldname im Backend-`rest-types` schlug sich nicht automatisch im Frontend-Compile
nieder. Plan-Disziplin musste die Lücke manuell schließen.

**Nach v1.2 (gelöst):**
- Eine einzige `rest-types`-Crate im Repo (`rest-types/`) — single source of truth.
- `shifty-dioxus/Cargo.toml` deklariert `[dependencies.rest-types] path = "../rest-types" default-features = false`
  (kein WASM-inkompatibler `service`-Pull-In via Feature-Gate).
- Verzeichnis `shifty-dioxus/rest-types/` ist gelöscht.
- Alle in CONCERNS.md §1 katalogisierten 17 fehlenden TOs/Enum-Varianten und 4
  fehlenden Felder sind im Frontend referenzierbar; Match-Arme exhaustiv (rustc-
  enforced); `cargo build --target wasm32-unknown-unknown` grün.
- 466 Backend-Tests grün ohne Regression. Phase 7 (Smoke + Regression) verifiziert.

**Strukturelle Drift-Tax beseitigt:** Künftige Backend-API-Änderungen brechen
den Frontend-Compile, falls dort nicht mit-angepasst — die Plan-Disziplin-Caveat
aus v1.0/v1.1 ist nicht mehr nötig.

### Bekannte Frontend-UI-Closure-Schulden (deferred to v1.3+)

Folgendes ist **state-only / no-op-rendering** im aktuellen Frontend, weil v1.2
explizit "keine User-facing Features" als Scope hatte. Diese Punkte werden in
v1.3 (oder später) als sichtbare UI nachgezogen:

- `current_paid_count` / `max_paid_employees` werden state-only gespiegelt, aber
  noch nicht gerendert (FUI-01, FUI-02)
- `VolunteerWork` / `UnpaidLeave` Extra-Hours-Kategorien sind in Match-Armen
  durch no-op-`rsx! { "" }` abgedeckt, aber ohne sichtbares UI (FUI-03)
- `cap_planned_hours_to_expected`-Settings-UI fehlt (FUI-04)
- **Frontend-Abwesenheiten-Maske** (FUI-A-01..09): Top-Level-Maske
  "Abwesenheiten" gegen `/absence-period` REST-API mit HR-Sicht +
  Employee-Self-Service. Backend ist seit v1.0 fertig; Mockup in
  `shifty-dioxus/shifty-design/project/absences.jsx`. Briefing in
  `notes/abwesenheiten-frontend-context.md`; Seed
  `seeds/abwesenheiten-frontend-milestone.md`.

### Co-Location vollzogen (2026-05-07)

Frontend lebt jetzt als Subordner unter `shifty-dioxus/`. History wurde via
`git filter-repo --to-subdirectory-filter shifty-dioxus` umgeschrieben und in
einem Merge-Commit hier eingespielt — alle 381 Frontend-Commits mit
Original-Author/Date/Message erhalten, nur Commit-IDs sind neu. File-History
funktioniert: `jj log -r '::@' shifty-dioxus/<pfad>` zeigt die echte Frontend-
History.

Cargo-Workspace-Boundary explizit: `exclude = ["shifty-dioxus"]` plus
implizit über die explizite `members`-Liste ohne Glob. Beide Subprojekte
bauen weiter unabhängig (`cargo check --workspace` im Root für Backend;
`dx serve` aus `shifty-dioxus/` für Frontend).

Eigenes altes `../shifty-dioxus/`-Repo bleibt als Archiv liegen — keine
Aktion nötig, kein Push erforderlich.

### Versionsabgleich

Beide Subprojekte haben heute zufällig identische Versionsstände
(`1.13.0-dev`). Releases müssen Backend- und Frontend-Versionen weiterhin
bewusst synchron halten — Update via `cli-update-version.sh` (im Backend-Root)
und `shifty-dioxus/cli-update-version.sh` (im Frontend-Subordner). Eine
spätere Konsolidierung könnte das vereinheitlichen, ist aber nicht dringend.

## Current Milestone: v1.5 Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen

**Goal:** Die verbleibenden Korrektheits- und Bedienprobleme der Abwesenheits-/
Urlaubsverwaltung schließen — konsistente Carryover-Werte und ein Umwandeln-Flow,
der kaum noch Handarbeit braucht.

**Target features:**
- **A — Smart „bis"-Datum im Umwandeln-Dialog:** Beim Öffnen des
  `AbsenceConvertModal` wird das „bis"-Feld automatisch **arbeitstagbasiert**
  (Wochenenden + Feiertage übersprungen) so vorbelegt, dass der Zeitraum den
  bereits berechneten `derived_days` entspricht. HR korrigiert nur noch im
  Ausnahmefall, statt jedes Mal selbst zu rechnen.
- **B — Warn-Indikator bei stundenbasierten Einträgen:** ⚠️ am Zeilenanfang
  eines stundenbasierten Markers (`HourlyMarkerRow`) auf der Absences-Seite, weil
  ein stundenbasierter Eintrag noch **kein echter Urlaub** (Absence Period) ist
  und konvertiert werden sollte.
- **Bug 1 — Carryover-Abweichung:** Carryover-Resturlaub in der
  Vacation-Balance-/Absence-Ansicht an den Report-Service angleichen
  (`VacationBalanceService` nutzt `get_carryover(year)` → muss `year-1` sein, wie
  `ReportingService`), mit Tests abgesichert. Report-Service = Wahrheit.
- **Bug 2 — Urlaubstage = 0 nach Konvertierung:** Im Detail-Employee-Report
  (`/employees/:id`) zeigt `vacation_days` nach extra_hours→Absence-Konvertierung 0,
  obwohl die Stunden korrekt sind. Root-Cause: `hours_per_week`
  (`reporting.rs:1227`) summiert absence-derived Stunden nur in `absence_hours`,
  nicht in die per-Woche `vacation_hours`/`sick_leave_hours`/`unpaid_leave_hours`,
  aus denen die `*_days()`-Methoden rechnen. Tage-Felder müssen derived Absences
  mitzählen (ohne Doppelzählung). Betrifft `vacation_days`/`sick_leave_days`/`absence_days`.
- **C — Mitarbeiter-Jahresansicht-Lesbarkeit (`/employees/:id`):** Betrifft den
  Wochen-Graph `EmployeeWeeklyHistogram` (`component/employee_weekly_histogram.rs`)
  + aufklappbare KW-Liste/`WeekDetailPanel` in `component/employee_view.rs` (NICHT
  `weekly_overview_chart.rs` = Team-Route `/weekly_overview/`). (1) Balken-Hover zeigt
  KW + von–bis Datum; (2) wo nur die KW-Nummer steht, von–bis Datum ergänzen
  („KW XY" ⏎ „von–bis"); (3) Freiwilligen-Stunden (`volunteer_hours`) als gestapeltes
  Segment im Graph + separater Wert in der aufgeklappten KW-Liste. Datenmodell
  `WorkingHours` (`from`/`to`/`overall_hours`/`expected_hours`/`volunteer_hours`)
  hat alle Felder bereits.
- **D — Mitarbeiter-Statistik (HR-only):** Pro-SalesPerson Statistik-Ansicht, nur
  mit HR-Rolle zugänglich. Kern-Kennzahl: durchschnittlich gearbeitete Stunden pro
  Woche, mit aus dem Nenner herausgerechneten Urlaubs-/Abwesenheitszeiträumen.
  Setzt Todo `AVG-01` um; **größtes/least-defined Item → eigene `discuss-phase`**
  (Bezugszeitraum, Definition „gearbeitet", welche Abwesenheiten raus, nur flexible
  Verträge?).
- **E — UI-Polish (Tabellen-Lesbarkeit):** (1) Stunden-Tabelle unter dem Schichtplan
  (`WorkingHoursMiniOverview`, `shiftplan.rs:1140`) bekommt max-width + Zebra-Layout;
  (2) `/absences`-Tabelle: Mitarbeiter-Spalte (`1.5fr`) deutlich schmaler. Ziel:
  Zeilen auf großen Bildschirmen nicht mehr verrutschen.

**Phasen-Nummerierung:** v1.5 setzt fort → startet bei **Phase 18**.

**Bewusst NICHT in v1.5:** Bug „Vertrag landet beim falschen Mitarbeiter" ist
bereits gefixt (Signal-Mirror `current_employee_id` + Regressionstest
`FROZEN_CAPTURE` in `employee_details.rs`) — Debug-Session
`working-hours-wrong-employee` ist obsolet.

## Current State

**Aktiver Milestone: v1.5** (Planung gestartet 2026-06-25). Zuletzt geshipt:
**v1.4 Committed Voluntary Capacity** (2026-06-25, Audit `passed`,
10/10 CVC-Requirements).

<details>
<summary>✅ v1.4 Committed Voluntary Capacity — SHIPPED 2026-06-25 (Phasen 14–17)</summary>

**Geliefert (as built):** zeit-versioniertes Feld `committed_voluntary: f32` auf
`EmployeeWorkDetails` (D-01 / Variante B — nur die freiwillige Zusage obendrauf,
entkoppelt von `expected_hours`), end-to-end durch SQLite-Migration → DAO → Service
→ `rest-types` → Frontend-State → Editor. Jahresansicht-Verfügbarkeit rechnet die
Zusage **ohne Doppelzählung** als separaten `committed_voluntary_hours`-Term ein
(Zwei-Band-Dekomposition, FORMULA B) — ausschließlich in **Achse B**
(`booking_information.rs::get_weekly_summary`), NICHT in `reporting.rs` (Achse A).
Anzeige als drittes Token 🎯 „zugesagt" + drittes gestapeltes Chart-Segment;
Vertrags-Editor-Input; „alle"-Filter macht rein unbezahlte Freiwillige
(`is_paid=false`, `expected_hours=0`) sichtbar, mit explizitem `is_paid`-Gating an
jeder paid-only-Site (kein Leak in `paid_hours`/Billing/Year-Summary). i18n De/En/Cs.

**Korrektur ggü. ursprünglichem Plan:** Der ursprünglich angenommene
**Snapshot-Schema-Version-Bump entfiel** (D-01 revidiert per Phase-15-CONTEXT,
CVC-05): die Dekomposition ist Achse-B-only und berührt keinen persistierten
`BillingPeriodValueType`. Die absolute Baseline der Konstante ist mittlerweile **9**
(out-of-milestone-Bump durch Commit `adf76c9`, nicht durch v1.4).

**Validierte Requirements:** CVC-01..10 (alle 10) — siehe
`milestones/v1.4-REQUIREMENTS.md`, Audit `milestones/v1.4-MILESTONE-AUDIT.md`.

**Pending Human-UAT (deferred):** Phase 16 visuelle Chart-Farb-Lesbarkeit +
Czech-Übersetzungsqualität (nicht test-automatisierbar; STATE.md → Deferred Items).

</details>

**Bewusst nicht in v1.4 (offen für v1.5+):**

- **CVC-F-01 / CVC-F-02** — Inline-Banner „Zusage nicht erfüllt"; eigenes
  committed-Band im Chart (CVC-F-02 teilweise in Phase 16 vorgezogen).
- **AVG-01 / Durchschnittliche-Anwesenheit-Auswertung** (Todo
  `2026-06-09-auswertung-durchschnittliche-anwesenheit-flexible-stunden.md`) —
  eigene discuss-Phase, viele offene Definitionsfragen.
- **Offene v1.3-UI-Restanten** (Phase 12-Cluster) — bleiben aufgegeben.
- Genehmigungs-Workflow; Min-Paid-Capacity / Skill-Matching (SC-01, SC-02).

## Active Milestones Index

Siehe `.planning/ROADMAP.md` + `.planning/MILESTONES.md`. Geshipt:
- v1.0 Range-Based Absence Management — 2026-05-03 (Phasen 1–4)
- v1.1 Slot Capacity & Constraints — 2026-05-04 (Phase 5)
- v1.2 Frontend rest-types Konsolidierung — 2026-05-07 (Phasen 6–7)
- v1.3 Frontend Abwesenheiten + UI-Closure-Restanten — closed 2026-06-22
  (Phasen 8, 8.2, 8.4, 8.5, 8.6, 9 geliefert; 8.1/11 superseded; 8.3/10/12/13
  bewusst aufgegeben). Archiv: `milestones/v1.3-ROADMAP.md`, `milestones/v1.3-phases/`.
- v1.4 Committed Voluntary Capacity — shipped 2026-06-25 (Phasen 14–17).
  Archiv: `milestones/v1.4-ROADMAP.md`.

Aktiv: **v1.5 Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen** (ab Phase 18).

## Evolution

Dieses Dokument entwickelt sich an Phase-Übergängen und Milestone-Grenzen.

**Nach jedem Phase-Übergang** (via `/gsd:transition`):
1. Requirements invalidiert? → unter "Bewusst nicht in v1.x" mit Begründung
2. Requirements validiert? → in MILESTONES.md verlinken mit Phase-Referenz
3. Neue Requirements aufgetaucht? → "Folgemilestone-Vorschau" anpassen
4. Decisions zu loggen? → in den Constraints-Abschnitt oder `.planning/extracted-learnings/`
5. "Was ist Shifty" noch akkurat? → nachziehen wenn die Realität gedriftet ist

**Nach jeder Milestone** (via `/gsd:complete-milestone`):
1. Komplettreview aller Sektionen
2. Constraints-Audit: noch gültig?
3. Bekannte Constraints: was wurde gelöst, was bleibt
4. Backlog-Items aus STATE.md → in den Folgemilestone-Vorschau heben oder fallenlassen
