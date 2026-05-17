---
type: project_charter
last_updated: 2026-05-07
last_milestone: v1.2 Frontend rest-types Konsolidierung (shipped 2026-05-07)
current_milestone: v1.3 Frontend Abwesenheiten + UI-Closure-Restanten
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

## Current Milestone: v1.3 Frontend Abwesenheiten + UI-Closure-Restanten

**Status:** Aktiv seit 2026-05-07 — gestartet via `/gsd-new-milestone v1.3`.

**Goal:** Dioxus-Frontend liefert sichtbare Abwesenheiten-Maske gegen
`/absence-period` REST-API und schließt die UI-Restanten aus v1.1/v1.2
(sichtbare Capacity-Anzeige, neue Extra-Hours-Kategorien, Settings-Felder).

**Target features:**

- **Frontend Abwesenheiten-Maske** (FUI-A-01..09) als Hauptthema: neue
  Top-Level-Route `absences` gegen `/absence-period` REST-API; HR-Sicht (alle
  Mitarbeiter, Filter) + Employee-Self-Service (eigene Liste); Form mit
  Datum-Range-Picker + Kategorie-Dropdown + Description; nicht-blockierende
  Warnings-Anzeige aus `AbsencePeriodCreateResultTO.warnings[]`.
- **Booking-Flow umstellen** auf `POST /shiftplan-edit/booking` für Reverse-
  Warnings; alter `POST /booking` bleibt für Bestands-Calls.
- **Shiftplan-Wochen-View** mit `UnavailabilityMarkerTO` farbig pro Tag pro
  Person inkl. `both`-Visual-Indication.
- **Migrations-Hinweis-UX** für alte `extra_hours`-Buttons (vor Cutover
  Soft-Migration; nach Cutover `403 ExtraHoursCategoryDeprecatedErrorTO`-
  Handling).
- **i18n** in De / En / Cs für alle neuen Strings.
- **User-facing Closure** der v1.1-/v1.2-Restanten (FUI-01..04 — sichtbares
  `current_paid_count`/`max_paid_employees`, Capacity-Editor,
  `VolunteerWork`/`UnpaidLeave` als UI-Element, `cap_planned_hours_to_expected`-
  Settings).

**Bewusst nicht in v1.3:**

- Stundenebene für Abwesenheiten (z. B. 3 h Arzttermin als Vacation) — bleibt
  out-of-scope. Halbtage (Heiligabend / Silvester) werden in Phase 8.3 über
  `day_fraction: Full \| Half` umgesetzt (FUI-A-10; Decision Log:
  `.planning/notes/halftime-absence-decision.md`).
- Genehmigungs-Workflow (Backend kennt keinen Approval-Schritt)
- Min-Paid-Capacity / Skill-Matching (SC-01, SC-02 — Backend-Themen für später)
- Admin-Cutover-UI (`/admin/cutover/*` ist getrenntes Admin-Surface; CLI-Flow
  reicht weiter)

**Quellen:**

- `seeds/abwesenheiten-frontend-milestone.md` (Trigger erfüllt; Sub-Phasen-Skizze)
- `notes/abwesenheiten-frontend-context.md` (Briefing aus Backend-Brief +
  Mockup-Walkthrough)
- `shifty-dioxus/shifty-design/project/uploads/absence-feature-frontend.md`
  (Backend-Integrations-Brief)
- `shifty-dioxus/shifty-design/project/absences.jsx` (Mockup, 729 Zeilen JSX —
  Vorlage für Dioxus-Portierung)
- `.planning/REQUIREMENTS.md` + `.planning/ROADMAP.md` für v1.3-Scope

## Active Milestones Index

Siehe `.planning/ROADMAP.md`. Geshipt:
- v1.0 Range-Based Absence Management — 2026-05-03 (Phasen 1–4)
- v1.1 Slot Capacity & Constraints — 2026-05-04 (Phase 5)
- v1.2 Frontend rest-types Konsolidierung — 2026-05-07 (Phasen 6–7)

Aktiv: v1.3 Frontend Abwesenheiten + UI-Closure-Restanten — siehe oben.

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
