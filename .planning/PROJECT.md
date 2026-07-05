---
type: project_charter
last_updated: 2026-07-05
last_milestone: v2.4 Kurzer-Tag-Slot-Kürzung (shipped 2026-07-05, Phase 51, 6/6 Requirements)
current_milestone: (zwischen Milestones)
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

## Architektur-Prinzipien

### Fat Backend, Thin Client

**Sämtliche Business-Logik (Berechnungen, Validierung, Aggregation, Semantik-
Regeln wie Slot-Clipping am Cutoff, Balance-Berechnung, Konflikt-Detection)
lebt im Backend. Das Frontend ist reiner View-Layer und zeigt vorbereitete
DTOs an.** Wenn ein DTO nachträglich Logik verlangt, ist das ein Signal,
dass das Backend ihm die effektiv anzuzeigenden Werte hätte liefern müssen.

**Motivation:** Zweit-Client-Fähigkeit — Mobile-App, alternative Web-UI,
CLI-Client etc. sollen ohne Duplikation der Domain-Regeln angebunden werden
können. Jede Regel, die im FE lebt, müsste in jedem zukünftigen Client
wiederholt werden.

**Ausnahmen** (nicht darunter):
- Reine UI-State-Logik (Selection, Filter-Sichten, Modal-Open/Close)
- Anzeigeformatierung (Datumsformat pro Locale, Farbwahl pro Status)
- Client-Side Input-Validierung als *Convenience* zusätzlich zur
  BE-Validierung (nie stattdessen)

**Konsequenz für Phasen-Design:** In discuss-phase-Fragen wie „rechnet BE
oder FE?" ist der Default „BE liefert fertigen Wert im DTO". Abweichung
braucht expliziten User-Konsens. Etabliert in Phase 51 (Kurzer-Tag-Slot-
Kürzung, 2026-07-04), rückwirkend Grundprinzip für alle folgenden Phasen.

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

## Zuletzt geshipt: v2.4 Kurzer-Tag-Slot-Kürzung (2026-07-05)

**Geliefert (as built):** 6/6 Requirements (SHC-01..SHC-06) über eine Phase (51,
8 Pläne). An Kurzen Tagen (`special_day.ShortDay` mit Cutoff-Uhrzeit) werden
Slots, die den Cutoff überlappen, dynamisch auf `[slot.start, cutoff]` gekürzt —
in Rendering (WeekView + PDF) und Ist-Stunden-Berechnung (Reporting +
Booking-Information + Balance). Slots komplett hinter dem Cutoff verschwinden.
Soll-Stunden bleiben unverändert (Balance-Konto sammelt Minusstunden). Die
Kürzung ist view-layer / dynamisch — keine DB-Änderung, kein Snapshot-Bump.
Ein admin-konfigurierbarer Stichtag `shortday_slot_clipping_active_from`
(ISO-Datum via `ToggleService`, Präzedenz HCFG-02) schützt historische
Balance-Views: ohne Wert deaktiviert (Rollout-Default), mit Wert wirkt Kürzung
nur für `booking_date >= active_from`. Kanonische pure Value-Methode
`Slot::clip_to` auf `service::slot::Slot` (D-51-01). Vier BE-Aggregat-Ketten
(Chain A' BlockService, Chain B ShiftplanWeek/PDF, Chain C
BookingInformation, Chain D ShiftplanReport). Chain D wurde
Rust-Layer-refaktoriert (raw-row DAO + Aggregation im Service statt SUM-SQL),
das räumte einen pre-existing `/60.0`-SQL-Bug in den alten SUM-Queries mit ab.
DTO `ShiftplanSlotTO.effective_to` (Wrapper-Field, D-51-09) trägt den
geclippten Wert ans FE + PDF; `SlotTO` bleibt bidirektional roh. **Fat
Backend, Thin Client** (D-51-02): grep-verifiziert kein `clip_to`-Call im
`shifty-dioxus/src/`. Admin-Settings Card 2b analog HCFG-02-Blueprint, 6 neue
i18n-Keys de/en/cs.

**Verifikation:** Phase 51 VERIFIED PASS (6/6 must-haves,
`behavior_unverified: 0`); Milestone-Audit `passed` (6/6 Requirements, 6/6
Cross-Phase Wirings, 6/6 E2E-Flows, 2 non-blocking Warnings). Backend `cargo
test --workspace` + `cargo clippy --workspace -- -D warnings` grün; FE `cargo
build --target wasm32-unknown-unknown` + FE-Clippy `-D warnings` grün.

**Bonus-Bugfixes (pre-existing, nicht in Requirements):** Filter-statt-Clip in
`shiftplan.rs` + `booking_information.rs` (ShortDay-Slots wurden ganz
ausgefiltert statt am Cutoff gekürzt); `/60.0`-SQL-Bug in alten
Chain-D-SUM-Queries (via Delete-Branch bei Rust-Layer-Refactor);
`ToggleService`-Full-Context-Bypass für internal-Aggregate-Konsumenten
(Gap-Closure via Fix-Commits `f654613`, `7f21bd4`, `1b863e8`, `5aee47e`,
`9cbe151`).

**Snapshot-Schema-Version:** bleibt 12 (grep-verifiziert; kein
`billing_period.value_type` angefasst). Migration: additiver Toggle-Seed
`20260704000001_seed-shortday-slot-clipping-toggle.sql` (`INSERT OR IGNORE`,
`enabled=0`, `value=NULL`). Keine neue Cargo-Dep.

**Closeout:** override_closeout (Milestone-Audit `passed` mit zwei non-blocking
Warnings W1 P07-SUMMARY-Doc-Drift + W2 latent `From<&SlotTO> for Slot` ohne
`effective_to`-Awareness; historische Deferred-Items acknowledged). Kein
git tag hier — SemVer-Tag via `/release-version`.

**Archiv:** `milestones/v2.4-ROADMAP.md`, `milestones/v2.4-REQUIREMENTS.md`,
`milestones/v2.4-MILESTONE-AUDIT.md`, `milestones/v2.4-phases/`.

<details>
<summary>✅ v2.3 PDF-Export — Browser-Look & Download-Button — SHIPPED 2026-07-04 (Phasen 49–50)</summary>

**Geliefert (as built):** 5/5 Requirements über 2 Phasen (49–50). Kleiner Fix-Milestone auf
dem v2.2-PDF-Export. Phase 49 (Download-Button, BE+FE, PDF-03/04/05): neuer REST-Endpoint
`GET /shiftplan/{id}/{year}/{week}/pdf` mit Auth-Gate (kein Admin-Gate) und
`WeekStatus`-Defense-in-Depth-409 (`week-not-releasable`, D-49-03); Business-Logic-Tier-Service
`PdfShiftplanService` als Assembler (ShiftplanView + SalesPerson + WeekStatus + pdf_render);
`PdfExportScheduler` refactored auf Delegation (D-49-08, single Assemble-Pfad für On-Demand +
Cron); FE-Anchor neben iCal mit pure Predicate `should_show_pdf_button(status, shiftplan_id)`
(8-case Test-Matrix, D-49-13); i18n-Key `PdfDownload` in de/en/cs. Phase 50 (Renderer neu,
PDF-01/02): kompletter Rewrite von `pdf_render.rs` mit 5-Parameter-Signatur
`render_shiftplan_week_pdf(..., render_timestamp: OffsetDateTime)` (D-50-11, pure Fn),
Hybrid-Stack-Layout (D-50-01/02), sichtbare Slot-Rahmen via `add_rect`+`PaintMode::Stroke`
(D-50-10), dynamische Sonntag-Spalte (D-50-08), Header-Timestamp „Erstellt am DD.MM.YYYY HH:MM
Uhr" (D-50-09), `resolve_render_timestamp()` mit `now_local()` + UTC-Fallback + `warn!`-Log
(D-50-12). Byte-Determinismus des Renderers bewusst gebrochen — WebDAV-Overwrite bleibt korrekt.
Kein Snapshot-Bump, keine Migration, keine neue Cargo-Dep (nur `local-offset`-Feature auf
existierendem `time`-Crate).

**Verifikation:** beide Phasen VERIFIED passed; Verifier PASSED 14/14 in Phase 50; Human UAT
D-50-17 bestätigt 2026-07-04 (visueller Layout-Check via Phase-49-Button gegen reale Woche).
781 Tests grün workspace-wide; `cargo clippy --workspace -- -D warnings` grün.

**Post-Ship-Hotfix (v2.3.1):** `fix(pdf-export): tolerate per-week ValidationError + fix
cron seed to 6-field` (Commit `754f94f`). WebDAV-Scheduler ignoriert einzelne Wochen mit
`ValidationError` statt komplett zu fallen; Cron-Seed-Format korrigiert.

**Design-Deviations post-UAT:** `+ N weitere` Overflow-Marker (D-50-03/04) und
`(freiwillig)`-Suffix (D-50-06/07) auf User-Wunsch entfernt (Boxen wachsen mit, Namen
komma-separiert, paid/unpaid irrelevant im PDF); Layout-Feintuning für Row-Alignment und
~2 zusätzliche Slots pro Spalte.

**Closeout:** override_closeout — kein formaler Milestone-Audit (2 Phasen, beide verifiziert,
Präzedenz v1.2/v1.3/v1.5/v1.6/v1.7). Kein git tag hier — Release-Tag via `/release-version`.

**Archiv:** `milestones/v2.3-ROADMAP.md`, `milestones/v2.3-REQUIREMENTS.md`.

</details>

<details>
<summary>✅ v2.1 Schichtplan- & Reporting-Erweiterungen — SHIPPED 2026-07-02 (Phasen 39–42)</summary>

**Geliefert (as built):** WST-01/02/05 KW-Status (neue `week_status`-Tabelle, shiftplanner-gated
CRUD, farbkodiertes Badge). WST-03/04 Wochen-Sperre TOCTOU-sicher in allen 6 Schreibpfaden (HTTP
423, Shiftplanner-Bypass, neue `delete_booking`-Methode + REST-Re-Routing, FE read-only + 423-Banner).
AVG-01/02/03 Ø-Anwesenheit flexibler Mitarbeiter (pure fn `average_hours_per_attendance_day`,
HR-gated). SDF-01 Special-Days-Anlegen-Button-Fix. Migration in Phase 39 (`week_status`-Tabelle).
Snapshot bleibt 12. Audit `passed` (9/9 Requirements). Details: `milestones/v2.1-ROADMAP.md`.

</details>

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

## Current Milestone

**Zwischen Milestones.** Nächste Iteration via `/gsd-new-milestone`.

Die off-theme **Backlog-Phase 999.1 (Breaking/Major Dependency-Migration)** bleibt
separat via `/gsd-plan-phase 999.1`.

## Current State

**v2.4 shipped 2026-07-05** (Phase 51, 8 Pläne, 6/6 Requirements SHC-01..SHC-06,
override_closeout, Audit `passed`). Geliefert: dynamische View-Layer-Kürzung
an ShortDays über die kanonische pure Value-Methode `Slot::clip_to` auf
`service::slot::Slot`; vier BE-Aggregat-Ketten (Chain A' BlockService, Chain B
ShiftplanWeek/PDF, Chain C BookingInformation, Chain D ShiftplanReport
Rust-Layer-Refactor) gaten am `shortday_gate::should_clip`;
Admin-konfigurierbarer Stichtag `shortday_slot_clipping_active_from` (ISO-Datum
via `ToggleService`, Präzedenz HCFG-02) schützt historische Balance-Views; DTO
`ShiftplanSlotTO.effective_to` (Wrapper, D-51-09) trägt geclippten Wert ans FE +
PDF, `SlotTO` bleibt bidirektional roh; FE-Loader collapst `effective_to` in
`state::Slot.to`, damit WeekView + PDF-Renderer automatisch geclippte Werte
sehen (Fat Backend, Thin Client — grep-verifiziert kein `clip_to`-Call im FE);
Admin-Settings Card 2b analog HCFG-02-Blueprint + 6 neue i18n-Keys de/en/cs.
Chain-D-Rust-Layer-Refactor räumte pre-existing `/60.0`-SQL-Bug in alten
SUM-Queries ab; Filter-statt-Clip-Bug in `shiftplan.rs` + `booking_information.rs`
gefixt; `ToggleService`-Full-Context-Bypass für internal-Aggregate-Konsumenten
nachträglich als Gap-Closure gefixt (`f654613`, `7f21bd4`, `1b863e8`, `5aee47e`,
`9cbe151`). Kein Snapshot-Bump (bleibt 12), keine neue Cargo-Dep, nur additive
Toggle-Seed-Migration. Details: `milestones/v2.4-ROADMAP.md` / `-REQUIREMENTS.md`
/ `-MILESTONE-AUDIT.md`. **Zwischen Milestones** — nächste Iteration via
`/gsd-new-milestone`.

Zuvor: **v2.3 PDF-Export — Browser-Look & Download-Button** (shipped 2026-07-04,
Phasen 49–50, 5/5 Requirements PDF-01..PDF-05, override_closeout, Post-Ship-Hotfix
v2.3.1); **v2.2 Aufräumen, WebDAV-Export & Wochentag-Muster** (shipped 2026-07-03,
Phasen 43–48, 16/16 Requirements, Audit `passed`); **v2.1 Schichtplan- &
Reporting-Erweiterungen** (2026-07-02, Phasen 39–42); **v1.11 Stabilisierung &
UX-Politur** (2026-07-01); **v1.10 Feiertage** (2026-06-30); **v1.9
Admin-Impersonation** (2026-06-29); **v1.8 HR-UX** (2026-06-29); **v1.7
Feiertage/VFA** (2026-06-29); **v1.6 Paid-Capacity** (2026-06-27) — alle
archiviert (siehe MILESTONES.md).

**Snapshot-Schema-Version: aktuell 12** (v1.7 Bump 10→11; v1.8 Bump 11→12;
v1.9–v2.4 kein Bump — v2.4 rein additive Live-Berechnung, kein
`BillingPeriodValueType` angefasst).

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
- v1.11 Stabilisierung & UX-Politur — shipped 2026-07-01 (Phasen 36–38).
  Archiv: `milestones/v1.11-ROADMAP.md`, `milestones/v1.11-REQUIREMENTS.md`, `milestones/v1.11-MILESTONE-AUDIT.md`.
- v2.1 Schichtplan- & Reporting-Erweiterungen — shipped 2026-07-02 (Phasen 39–42).
  Archiv: `milestones/v2.1-ROADMAP.md`, `milestones/v2.1-REQUIREMENTS.md`, `milestones/v2.1-MILESTONE-AUDIT.md`.
- v2.2 Aufräumen, WebDAV-Export & Wochentag-Muster — shipped 2026-07-03 (Phasen 43–48).
  Archiv: `milestones/v2.2-ROADMAP.md`, `milestones/v2.2-REQUIREMENTS.md`, `milestones/v2.2-MILESTONE-AUDIT.md`.
- v2.3 PDF-Export: Browser-Look & Download-Button — shipped 2026-07-04 (Phasen 49–50).
  Archiv: `milestones/v2.3-ROADMAP.md`, `milestones/v2.3-REQUIREMENTS.md`.
- v2.4 Kurzer-Tag-Slot-Kürzung — shipped 2026-07-05 (Phase 51).
  Archiv: `milestones/v2.4-ROADMAP.md`, `milestones/v2.4-REQUIREMENTS.md`, `milestones/v2.4-MILESTONE-AUDIT.md`, `milestones/v2.4-phases/`.

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
5. **Docs-Freshness-Gate** (siehe unten) — hartes Gate, blockiert Milestone-Close bei Drift.

## Docs-Freshness-Gate

Die technische Doku unter `docs/` ist **verbindliche Projekt-Referenz** und
wird bei jedem Milestone-Close **hart geprüft**. Ein Milestone kann nicht
geschlossen werden, wenn Docs-Drift gegen die Trigger-Dateien besteht.

### Was ist Drift?

Eine der folgenden Backend-Dateien wurde zwischen letztem Milestone-Close und
jetzt geändert, aber die zugehörige Docs-Sektion **nicht**:

| Trigger-Datei | → Docs-Sektion |
| --- | --- |
| `shifty_bin/src/main.rs` | `docs/architecture/02-service-tiers.md` + `diagrams/service-graph-runtime.mmd` |
| `service/**/*.rs` (Trait-Änderungen) | passende `docs/features/F*.md` |
| `dao/**/*.rs` (Trait-Änderungen) | passende `docs/features/F*.md` |
| `migrations/sqlite/*.sql` | `docs/architecture/03-data-model.md` + `diagrams/db-schema-er.mmd` + passende `docs/features/F*.md` |
| `service_impl/src/permission.rs` | `docs/architecture/04-auth.md` + `docs/features/F12-auth-session.md` |
| `service_impl/src/billing_period_report.rs` | `docs/features/F08-billing-period.md` + `docs/domain/billing-period.md` |
| `service_impl/src/reporting.rs` | `docs/features/F07-reporting-balance.md` + `docs/domain/time-accounting.md` |

Beide Sprach-Varianten (`.md` und `_de.md`) müssen synchron sein — Änderung nur
in einer Sprache = Drift.

### Prüfvorgang bei Milestone-Close

Ausführbar als Teil von `/gsd-audit-milestone` oder manuell durch die/den
Milestone-Verantwortliche/n:

1. **Trigger-Diff sammeln:**
   ```bash
   git diff --name-only <last-milestone-tag>..HEAD -- \
     shifty_bin/src/main.rs \
     'service/**/*.rs' \
     'dao/**/*.rs' \
     'migrations/sqlite/*.sql' \
     service_impl/src/permission.rs \
     service_impl/src/billing_period_report.rs \
     service_impl/src/reporting.rs
   ```
2. **Docs-Diff sammeln:**
   ```bash
   git diff --name-only <last-milestone-tag>..HEAD -- 'docs/**/*.md' 'docs/**/*.mmd'
   ```
3. **Mapping-Check:** Für jede berührte Trigger-Datei prüfen, ob die
   zugehörige Docs-Sektion (Tabelle oben) im Docs-Diff enthalten ist.
4. **`/gsd-docs-update` optional** — verifiziert Doku-Behauptungen gegen
   die Codebase; flagt inhaltliche Drift, die die reine Datei-Zuordnung
   nicht fängt.

### Ergebnis

- **Kein Drift:** Milestone-Close geht durch.
- **Drift gefunden:** Milestone-Close **blockiert**. Zwei Wege raus:
  1. Docs im selben Milestone nachziehen (bevorzugt).
  2. Explizite Begründung im MILESTONE-AUDIT.md („Trigger-Datei X wurde
     geändert, aber nur additiv/keine Konvention betroffen, Doku bleibt
     korrekt") — mit Reviewer-Ack.

### Warum das hart ist

Docs-Drift kumuliert schleichend. Nach zwei Milestones ohne Check ist die
Doku unbrauchbar. Der harte Gate zwingt die Disziplin an genau dem Punkt,
an dem alle Kontext-Fenster ohnehin auf den Milestone gerichtet sind.

Historischer Bezug: Root-CLAUDE.md und `shifty-backend/CLAUDE.md` enthalten
schon eine ähnliche Regel für den Snapshot-Version-Bump-Vertrag —
Docs-Freshness folgt derselben Logik: eine kritische, sonst leicht
übersehbare Kopplung zwischen Code-Änderung und Konvention.
