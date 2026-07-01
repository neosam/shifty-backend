# Requirements: Shifty Backend — v1.11 Stabilisierung & UX-Politur

**Defined:** 2026-07-01
**Core Value:** Zuverlässige Schichtplan-/HR-Verwaltung — Konsolidierung nach der
v1.7–v1.10-Feature-Welle. Dieses Milestone liefert **keine neuen Fähigkeiten**, sondern
räumt gemeldete Bugs ab und macht den Frontend-Build warnungsfrei.

## v1.11 Requirements

Jedes Requirement mappt auf eine Roadmap-Phase (siehe Traceability). Herkunft: Todo-Backlog.

### Special-Days-Fixes (SDF)

Nachlese zu v1.10/Phase 33 (Special-Days-UI). Beide Bugs live vom User gemeldet 2026-06-30/07-01.

- [x] **SDF-01**: Wird im Schichtplan ein Tag von „Feiertag" auf „Kurzer Tag" (oder umgekehrt)
  umgestellt, aktualisiert der Pfad den **bestehenden** Special-Day-Eintrag für das Datum
  (update statt zweitem insert) — **keine Fehlermeldung**, der neue Typ ist danach persistiert.
  (Todo `2026-07-01-schichtplan-feiertag-auf-kurzer-tag-wirft-fehler.md`)

- [x] **SDF-02**: In der Settings-Special-Days-Karte lassen sich **mehrere Feiertage
  nacheinander** anlegen, ohne dass der „Anlegen"-Button hängen bleibt. Nach erfolgreichem
  Create ist der Button für den nächsten Eintrag sofort wieder korrekt aktiviert (kein
  Controlled-vs-Uncontrolled-Desync zwischen `sd_type`-Signal und `<select>`).
  (Todo `2026-06-30-settings-special-days-anlegen-button-disabled.md`)

### Modal-UX (MOD)

- [x] **MOD-01**: Ein innerhalb eines Modals **begonnener** Maus-Drag (Text-Selektion), der
  **außerhalb** losgelassen wird, schließt das Modal **nicht**. Nur ein echter Außerhalb-Klick
  (mousedown *und* mouseup auf dem Backdrop) schließt. Zentral in `dialog.rs`, sodass **alle**
  Modals profitieren.
  (Todo `2026-06-30-modal-schliesst-bei-mouseup-ausserhalb-nach-drag.md`)

- [x] **MOD-02**: Das Arbeitsvertrag-Modal zeigt unter jedem relevanten Feld einen kurzen
  **Erklärungssatz** (Muster `CapPlannedHoursHelp`, `text-small text-ink-muted`) — Von/Bis
  ausgenommen (selbsterklärend). Alle neuen Texte in **de/en/cs**.
  (Todo `2026-06-30-arbeitsvertrag-modal-erklaerungssatz-pro-feld.md`)

### Build-Hygiene (HYG)

- [ ] **HYG-01**: Der Frontend-Build (`shifty-dioxus`) ist **warnungsfrei** — die ~45
  rustc-Warnings (14 via `cargo fix`, Rest manuell: ungenutzte Methoden/Imports/Variablen
  entfernen oder mit begründetem `#[allow(dead_code)]` behalten) sind beseitigt.
  (Todo `2026-06-30-saemtliche-warnungen-fixen.md`)

- [ ] **HYG-02**: Das Backend bleibt weiterhin `cargo clippy --workspace -- -D warnings`-sauber
  (Regressions-Gate); der dioxus-Clippy-Lauf erfolgt bewusst aus der Backend-nix-Shell (E0514
  im dioxus-Shell). Verbleibende bewusst behaltene dioxus-Lints sind dokumentiert.
  (Todo `2026-06-30-saemtliche-warnungen-fixen.md`)

## v2 / Folgemilestones

Bewusst aus v1.11 herausgehalten (siehe PROJECT.md „Bewusst NICHT in v1.11"):

### v1.12 — Schichtplan- & Reporting-Erweiterungen

- **WST-01**: KW-Status (None / In Planung / Geplant / Gesperrt) pro (year, week), inkl.
  Permission-Gate: gesperrte Wochen nur vom Schichtplaner änderbar. Full-Stack
  (Migration+DAO+Service+REST+FE). (Todo `2026-06-30-kalenderwoche-status-*`)

- **AVG-01**: Auswertung durchschnittliche Anwesenheit bei flexiblen Stunden, Urlaub
  herausgerechnet. Viele offene Definitionsfragen → eigene discuss-Phase.
  (Todo `2026-06-09-auswertung-durchschnittliche-anwesenheit-*`)

### v1.13 — PDF-Export → Nextcloud/WebDAV

- **EXP-01**: Täglicher automatischer PDF-Export der Folgewochen-Schichtpläne per WebDAV in
  eine konfigurierbare Nextcloud-Instanz. Architektonisch eigenständig (interner Scheduler,
  PDF-Lib, WebDAV-Client, neue Deps, Secrets-Handling).
  (Todo `2026-06-09-taeglicher-pdf-export-*`)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Neue fachliche Fähigkeiten | v1.11 ist reine Stabilisierung/Politur — Features gehen nach v1.12/v1.13 |
| ShortDay/Kurztage-Automatik | Separate Future-Story (seit Phase 25 außer Scope) |
| Backlog-Phase 999.1 (Breaking/Major Dependency-Migration) | Off-theme, bleibt separat via `/gsd-plan-phase 999.1` |
| dioxus-Workspace ins CI-Clippy-Gate aufnehmen | Optional/erwägenswert nach Lint-Abbau, nicht v1.11-Pflicht (HYG-02 hält nur das Backend-Gate + dokumentiert Rest) |
| Snapshot-Schema-Version-Bump | Kein persistierter `BillingPeriodValueType`-Pfad berührt → bleibt 12 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| SDF-01 | Phase 36 | Complete |
| SDF-02 | Phase 36 | Complete |
| MOD-01 | Phase 37 | Complete |
| MOD-02 | Phase 37 | Complete |
| HYG-01 | Phase 38 | Pending |
| HYG-02 | Phase 38 | Pending |

**Coverage:**

- v1.11 requirements: 6 total
- Mapped to phases: 6
- Unmapped: 0 ✅

---
*Requirements defined: 2026-07-01*
*Last updated: 2026-07-01 after milestone v1.11 start*
