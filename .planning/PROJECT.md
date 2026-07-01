---
type: project_charter
last_updated: 2026-06-30
last_milestone: v1.11 Stabilisierung & UX-Politur (shipped 2026-07-01, Phasen 36–38, 6/6 Requirements, Audit passed)
current_milestone: none (zwischen Milestones; nächste Iteration via /gsd-new-milestone)
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

## Aktueller Milestone: v1.11 Stabilisierung & UX-Politur

> **Versions-Hinweis:** `v1.11` ist das **interne GSD-Planungs-Label** (Fortführung
> der Sequenz v1.0–v1.10 für Roadmap, Phasen-Nummerierung und Archiv-Handles). Die
> **reale Release-Version** vergibt der User datumsbasiert via `./cli-update-version`
> (z. B. `2026.182.0`) zum Release-Zeitpunkt — entkoppelt vom Planungs-Label.

**Goal:** Konsolidierung nach der v1.7–v1.10-Feature-Welle — vier gemeldete Bugs abräumen
und den Frontend-Build warnungsfrei machen. **Keine neuen Fähigkeiten**, reine Stabilisierung
und UX-Politur.

**Target features (5 Items, aus dem Todo-Backlog):**
- **Special-Days-Bugfixes** (thematisch zusammen, Nachlese zu v1.10/Phase 33):
  - **Feiertag → „Kurzer Tag" wirft Fehler:** Umstellen des Special-Day-Typs auf demselben
    Datum erzeugt einen Konflikt (update-vs-insert). Umstell-Pfad muss den bestehenden
    Eintrag aktualisieren statt einen zweiten anzulegen.
    (Todo `2026-07-01-schichtplan-feiertag-auf-kurzer-tag-wirft-fehler.md`)
  - **Settings „Anlegen"-Button bleibt disabled:** Nach dem ersten angelegten Feiertag
    bleibt der Button disabled (controlled-select-Desync: `sd_type`-Reset ↛ `<select>`).
    Select controlled an Signal binden bzw. Typ nach Create beibehalten.
    (Todo `2026-06-30-settings-special-days-anlegen-button-disabled.md`)
- **Modal-UX** (thematisch zusammen):
  - **Modal schließt nach Drag:** Ein innen begonnener Text-Drag, der außerhalb losgelassen
    wird, schließt das Modal. Zentraler `dialog.rs`-Fix (mousedown-Ursprung tracken; nur
    schließen, wenn Druck UND Loslassen auf dem Backdrop) — kommt allen Modals zugute.
    (Todo `2026-06-30-modal-schliesst-bei-mouseup-ausserhalb-nach-drag.md`)
  - **Arbeitsvertrag-Modal Erklärungssätze:** Pro Feld ein kurzer Help-Text analog
    `CapPlannedHoursHelp` (Von/Bis ausgenommen); i18n de/en/cs.
    (Todo `2026-06-30-arbeitsvertrag-modal-erklaerungssatz-pro-feld.md`)
- **Build-Hygiene:**
  - **Sämtliche Frontend-Warnungen fixen:** ~45 rustc-Warnings (14 via `cargo fix`) + Abbau
    der ~198 pre-existing dioxus-Lints Richtung warnungsfreier Build. Clippy für dioxus nur
    aus der Backend-nix-Shell lauffähig (E0514). Backend bleibt `-D warnings`-sauber.
    (Todo `2026-06-30-saemtliche-warnungen-fixen.md`)

**Bewusst NICHT in v1.11 (→ Folgemilestones):**
- **v1.12 „Schichtplan- & Reporting-Erweiterungen":** KW-Status (None/Planung/Geplant/Gesperrt
  inkl. Permission-Gate für gesperrte Wochen) + Auswertung durchschnittliche Anwesenheit bei
  flexiblen Stunden. (Todos `2026-06-30-kalenderwoche-status-*`, `2026-06-09-auswertung-*`)
- **v1.13 „PDF-Export → Nextcloud/WebDAV":** Täglicher automatischer PDF-Export der
  Folgewochen-Schichtpläne per WebDAV — architektonisch eigenständig (interner Scheduler,
  PDF-Lib, WebDAV-Client, neue Deps, Secrets). (Todo `2026-06-09-taeglicher-pdf-export-*`)
- Off-theme **Backlog-Phase 999.1** (Breaking/Major Dependency-Migration) bleibt separat.

**Snapshot-Schema-Version:** **nicht** betroffen — reine Bugfixes + Build-Hygiene, kein
persistierter `BillingPeriodValueType`-Pfad berührt. **Kein Bump** (bleibt 12), keine Migration,
keine neuen Deps erwartet.

## Zuletzt geshipt: v1.8 Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX) (2026-06-29)

<details>
<summary>✅ v1.8 Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX) — SHIPPED 2026-06-29 (Phasen 27–28)</summary>

**Geliefert (as built):** Freiwillige (`is_paid=false`) sind in den Abwesenheits-
Selektoren auswählbar — gruppiert (native `optgroup` Angestellte/Freiwillige) in
**beiden** Call-Sites (AbsenceModal + AbsenceFilterBar) über einen gemeinsamen Helfer,
inaktive ausgeblendet, leere Gruppen ausgelassen, i18n de/en/cs (VOL-SEL-01). HR kann den
berechneten Jahres-Urlaubsanspruch per signed **Offset (Korrektur-Delta)** anpassen:
`entitled_effective = round(berechnet) + offset`, pro Person+Jahr persistiert, überlebt
Vertragsänderungen (Delta statt Override), HR-gated CRUD + immer sichtbares Inline-Editor-
Feld; für normale User unsichtbar via **API-level** Hiding (Self-View bekommt
`offset`/`computed == None`). Begleitend: Off-by-one-Proration-Fix
(`vacation_days_for_year` year-START) + Snapshot-Schema-Version-Bump 11→12
(`BillingPeriodValueType::VacationEntitlement`) (VAC-OFFSET-01).

**Validierte Requirements:** VOL-SEL-01, VAC-OFFSET-01 (2/2) — siehe
`milestones/v1.8-REQUIREMENTS.md`, Archiv `milestones/v1.8-ROADMAP.md`, Audit
`milestones/v1.8-MILESTONE-AUDIT.md` (`passed`, 100% Integration, 2/2 Flows).

**Verifikation:** beide Phasen VERIFIED inkl. **Live-HR-Browser-Smokes**
(`behavior_unverified: 0`); Backend `cargo test --workspace` + `clippy -D warnings` grün,
Frontend WASM-Build + 678 FE-Tests grün. 2 Bugs im Live-Smoke gefunden+gefixt
(`Dioxus.toml`-Dev-Proxy `/vacation-entitlement-offset`; AbsenceModal-Close).

**Closeout:** override_closeout — formaler Milestone-Audit `passed`; Carry-over Deferred
Items acknowledged (STATE.md).

</details>

<details>
<summary>✅ v1.7 Automatische Feiertage & Freiwilligen-Abwesenheit — SHIPPED 2026-06-29 (Phasen 25–26)</summary>

**Geliefert (as built):** Feiertage werden automatisch im Mitarbeiterreport angerechnet —
**derive-on-read** (Toggle-`value`-Cutoff + `SpecialDay`, keine `ExtraHours`-Rows), Wirkung
identisch zu manuellem `ExtraHours(Holiday)` (Dual-Write `holiday_hours`+`absense_hours`),
ab admin-konfigurierbarem „aktiv ab"-Stichtag (schützt Vergangenheit + verhindert
Doppelzählung). Urlaub/Abwesenheit eines Freiwilligen reduziert seine committed-Zusage 🎯
in der Jahresansicht (whole-week-out in `get_weekly_summary`); Feiertage bewusst **nicht**
(Asymmetrie, per CI-Guard gepinnt). Bidirektionale Deep-Links `/absences/:employee_id` ↔
Mitarbeiterreport. Snapshot-Bump 10→11. i18n de/en/cs.

**Validierte Requirements:** HOL-01..03, VFA-01/02, HCFG-01..03, HSNAP-01, NAV-01 (10/10) —
siehe `milestones/v1.7-REQUIREMENTS.md`, Archiv `milestones/v1.7-ROADMAP.md`.

**Closeout:** override_closeout — Carry-over Deferred Items acknowledged (gemeinsam mit
v1.8 am 2026-06-29 geschlossen; Close war nach „verified 2026-06-28" liegengeblieben).

</details>

<details>
<summary>✅ v1.6 Paid-Capacity-Durchsetzung & Konfiguration — SHIPPED 2026-06-27 (Phase 24)</summary>

**Geliefert (as built):** Die Paid-Capacity-Grenze (`max_paid_employees` pro Slot/Woche)
ist von einem rein visuellen Soft-Hinweis (v1.1/Phase 5, Phase 23) zu einem **global
konfigurierbar durchsetzbaren Limit** geworden. Ein admin-schaltbarer globaler Toggle
(`paid_limit_hard_enforcement` über den bestehenden `ToggleService`, Seed-Migration,
Default = weich → keine Regression) bestimmt, ob das Buchen über das Limit hinaus hart
blockiert wird (außer für die Shiftplanner-Rolle) oder nur eine nicht-blockierende Warnung
erzeugt. Der Hard-Block läuft pre-persist im Business-Logic-Tier (`ShiftplanEditService`
mit frisch gelesenem Toggle vor `booking_service.create`), liefert einen unterscheidbaren
`ServiceError::PaidLimitExceeded` (HTTP **409**, nicht 403) + lokalisierte Inline-Meldung.
Persistente Overage-Warn-Sektion über dem Wochenplan für **alle Rollen**. Permission-Gate
des Buchungspfads korrigiert von `HR ∨ self` auf `Shiftplanner ∨ self` (D-24-04). i18n De/En/Cs.

**Verifikation:** 7/7 must-haves verified (24-VERIFICATION.md); Human-UAT 3/4 PASS;
2 Bugs während UAT gefunden+gefixt (Dev-Proxy-Allowlist `/toggle`; Overage-Count
für Nicht-HR-Rollen). Backend `cargo test`/`clippy -D warnings` grün, Frontend WASM-Build grün.

**Closeout:** override_closeout — kein formaler Milestone-Audit (Phasen-Verifikation +
UAT genügten, analog v1.5). Ein Human-UAT-Item bewusst deferred (Inline-Block-Platzierung,
nicht-blockierend; Backend-409-Logik unit-getestet).

**Archiv:** `milestones/v1.6-ROADMAP.md`.

</details>

<details>
<summary>✅ v1.5 Mitarbeiter-Sicht & Urlaubsverwaltung — SHIPPED 2026-06-27 (Phasen 18–23)</summary>

**Geliefert (as built):** Carryover-Resturlaub stimmt zwischen Vacation-Balance und
Report-Service überein (`year-1`-Quelle gepinnt + Mock-Lock); `vacation_days` bleibt
nach extra_hours→Absence-Konvertierung korrekt (derived Absences in per-Woche-
Kategorien gemergt, Single Source `by_week`, kein Double-Count → Snapshot-Bump 9→10).
Convert-Dialog belegt das bis-Datum arbeitstagbasiert vor + erkennt den exakten
1-Wochen-Fall. Mitarbeiter-Jahresansicht: KW+Datum-Hover/-Labels + gestapelte
Freiwilligen-Stunden. HR-only Ø-Stunden/Woche-Statistik pro Person (urlaubsbereinigt,
Regel A-22-1). UI-Polish (max-width + Zebra, schmalere Mitarbeiter-Spalte). Mitgeliefert:
Slot-Paid-Capacity-Frontend (Editor + Overage-Warnfarbe) inkl. `modify_slot`-Bugfix.

**Validierte Requirements:** UV-01..05, YV-01..03, STAT-01/02, UI-01/02 (12/12) —
siehe `milestones/v1.5-REQUIREMENTS.md`, Archiv `milestones/v1.5-ROADMAP.md`.

**Closeout:** override_closeout — `carryover-absence-vs-report` code-gefixt, nur
`awaiting_human_verify`; historischer Quick-Task-/Todo-Ballast deferred (STATE.md).

</details>

**Bewusst NICHT in v1.5:** Bug „Vertrag landet beim falschen Mitarbeiter" ist bereits
gefixt (Signal-Mirror `current_employee_id` + Regressionstest `FROZEN_CAPTURE`) —
Debug-Session `working-hours-wrong-employee` obsolet.

## Current State

**v1.11 shipped 2026-07-01** (Phasen 36–38, 6 Pläne, 6/6 Requirements, Audit `passed`,
override_closeout; autonomer Run). Geliefert: SDF-01/02 Special-Days-Bugfixes (atomarer
in-place Special-Day-Replace Feiertag↔Kurzer-Tag statt HTTP-422-Duplicate; controlled
`SelectInput` + Settings-Card-3-Bindung → „Anlegen"-Button re-enabled), MOD-01/02 Modal-UX
(zentrale drag-sichere `BackdropPress`-Backdrop-Logik für alle Dialoge + pro-Feld-Help-Texte
im Arbeitsvertrag-Modal, 6 neue `*Help`-Keys de/en/cs), HYG-01/02 Frontend-Build-Hygiene
(`shifty-dioxus` `cargo build` warnungsfrei; Backend-Clippy-Gate grün). Keine neuen
Fähigkeiten, kein Snapshot-Bump (bleibt 12), keine Migration, keine neuen Deps. **Zwischen
Milestones** — nächste Iteration via `/gsd-new-milestone`. Details:
`milestones/v1.11-ROADMAP.md` / `-REQUIREMENTS.md` / `-MILESTONE-AUDIT.md`.

Zuvor: **v1.10 Feiertage — UI-Pflege & Schichtplan-Soll-Konsistenz** (shipped 2026-06-30,
Phasen 33–35, 8 Pläne, 12/12 Requirements, Audit `passed`). Special Days shiftplanner-gated
über zwei UI-Flächen pflegbar + Feiertags-Soll im Schichtplan + Slot-Werte für genau eine KW.
Details: `milestones/v1.10-ROADMAP.md` / `-REQUIREMENTS.md` / `v1.10-MILESTONE-AUDIT.md`.

Zuvor geshipt + archiviert: **v1.9 Schichtplan-/Urlaubs-UX-Korrekturen &
Admin-Impersonation** (Phasen 29–32, 6 Pläne, 7/7 Requirements, override_closeout, Audit
`passed`; autonomer Run). Geliefert: VAC-01 Urlaubsbalken-Konsistenz, SHP-02 Stale-Daten-
Race-Guard, SHP-01 proaktive Abwesenheits-Markierung, IMP-01..04 Admin-Impersonation-FE +
zentrale Audit-Schicht. Kein Snapshot-Bump, keine Migration, keine neuen Deps. Details:
`milestones/v1.9-ROADMAP.md` / `-REQUIREMENTS.md` / `-MILESTONE-AUDIT.md`.
Davor: **v1.8 HR-UX** (Phasen 27–28) und **v1.7 Feiertage/VFA** (Phasen 25–26), beide
2026-06-29 geschlossen; **v1.6 Paid-Capacity** (2026-06-27).

**Snapshot-Schema-Version: aktuell 12** (v1.7 Bump 10→11 Holiday-Computation; v1.8 Bump
11→12 `VacationEntitlement`-Computation; v1.9/v1.10/v1.11 kein Bump — für v1.11 verifiziert).

**Nächste Milestones (aus dem 3er-Split, laut Roadmap):** v1.12 Schichtplan-/Reporting-
Erweiterungen (WST-01 KW-Status, AVG-01 Durchschnitts-Anwesenheit) und v1.13 PDF-Export →
Nextcloud/WebDAV (EXP-01). Start via `/gsd-new-milestone`. Die off-theme **Backlog-Phase
999.1 (Breaking/Major Dependency-Migration)** bleibt separat verfügbar via
`/gsd-plan-phase 999.1`.

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
- v1.5 Mitarbeiter-Sicht & Urlaubsverwaltung — shipped 2026-06-27 (Phasen 18–23).
  Archiv: `milestones/v1.5-ROADMAP.md`, `milestones/v1.5-REQUIREMENTS.md`.
- v1.6 Paid-Capacity-Durchsetzung & Konfiguration — shipped 2026-06-27 (Phase 24).
  Archiv: `milestones/v1.6-ROADMAP.md`.
- v1.7 Automatische Feiertage & Freiwilligen-Abwesenheit — shipped 2026-06-29 (Phasen 25–26).
  Archiv: `milestones/v1.7-ROADMAP.md`, `milestones/v1.7-REQUIREMENTS.md`.
- v1.8 Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX) — shipped 2026-06-29 (Phasen 27–28).
  Archiv: `milestones/v1.8-ROADMAP.md`, `milestones/v1.8-REQUIREMENTS.md`, `milestones/v1.8-MILESTONE-AUDIT.md`.

- v1.9 Schichtplan-/Urlaubs-UX-Korrekturen & Admin-Impersonation — shipped 2026-06-29 (Phasen 29–32).
  Archiv: `milestones/v1.9-ROADMAP.md`, `milestones/v1.9-REQUIREMENTS.md`, `milestones/v1.9-MILESTONE-AUDIT.md`.
- v1.10 Feiertage — UI-Pflege & Schichtplan-Soll-Konsistenz — shipped 2026-06-30 (Phasen 33–35).
  Archiv: `milestones/v1.10-ROADMAP.md`, `milestones/v1.10-REQUIREMENTS.md`, `v1.10-MILESTONE-AUDIT.md`.

Zwischen Milestones — nächste Iteration via `/gsd-new-milestone`.

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
