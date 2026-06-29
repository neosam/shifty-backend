# Roadmap: Shifty Backend

## Milestones

- ✅ **v1.0 Range-Based Absence Management** — Phasen 1–4 (shipped 2026-05-03) — siehe [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 Slot Capacity & Constraints** — Phase 5 (shipped 2026-05-04) — siehe [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)
- ✅ **v1.2 Frontend rest-types Konsolidierung** — Phasen 6–7 (shipped 2026-05-07) — siehe [`milestones/v1.2-ROADMAP.md`](milestones/v1.2-ROADMAP.md)
- ✅ **v1.3 Frontend Abwesenheiten + UI-Closure-Restanten** — Phasen 8–13 (closed 2026-06-22; geliefert: 8, 8.2, 8.4, 8.5, 8.6, 9; 8.1/11 ⊘ superseded; **8.3/10/12/13 bewusst aufgegeben**) — siehe [`milestones/v1.3-ROADMAP.md`](milestones/v1.3-ROADMAP.md)
- ✅ **v1.4 Committed Voluntary Capacity** — Phasen 14–17 (shipped 2026-06-25) — siehe [`milestones/v1.4-ROADMAP.md`](milestones/v1.4-ROADMAP.md)
- ✅ **v1.5 Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen** — Phasen 18–23 (shipped 2026-06-27) — siehe [`milestones/v1.5-ROADMAP.md`](milestones/v1.5-ROADMAP.md)
- ✅ **v1.6 Paid-Capacity-Durchsetzung & Konfiguration** — Phase 24 (shipped 2026-06-27) — siehe [`milestones/v1.6-ROADMAP.md`](milestones/v1.6-ROADMAP.md)
- ✅ **v1.7 Automatische Feiertage & Freiwilligen-Abwesenheit** — Phasen 25–26 (complete & verified 2026-06-28; Milestone-Close offen)
- 🚧 **v1.8 Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX)** — Phasen 27–28 (beide executed 2026-06-29; Automatik-Gates grün, 2 Browser-Smokes als Human-UAT offen; Milestone-Close ausstehend)

## Phases

### v1.8 Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX) (Phasen 27–28) — ACTIVE 2026-06-29

**Milestone Goal:** HR-UX rund um Abwesenheiten/Urlaub: Freiwillige sind in den Abwesenheits-Selektoren auswählbar, und HR kann den berechneten Jahres-Urlaubsanspruch per Korrektur-Offset anpassen.

- [x] **Phase 27: Freiwillige in Abwesenheitsliste auswählbar (FE)** — gruppierter Personen-Selector (optgroup Angestellte/Freiwillige) in AbsenceModal + AbsenceFilterBar via gemeinsamem Helfer; `is_selectable_employee` NICHT gelockert (D-27-02: HR-Urlaubsübersicht bleibt paid-only), neue Gruppierung nutzt eigenes `!inactive`-Predicate; 2 neue i18n-Keys de/en/cs. Reines Frontend (VOL-SEL-01). **Executed 2026-06-29** — Automatik-Gates grün (677 Tests, WASM-Build), Browser-Smoke als Human-UAT offen.
- [x] **Phase 28: Urlaubsanspruch-Korrektur via Offset (BE+FE)** — signed Offset pro Person+Jahr (Delta, kein Override); HR-gekennzeichnet+inline editierbar, für User unsichtbar (API-level hiding); neue Tabelle + HR-gated CRUD + FE-Inline-Editor. Plus Off-by-one-Proration-Fix + Snapshot-Bump 11→12 (VAC-OFFSET-01). **Executed 2026-06-29** — Backend+FE-Gates grün (test --workspace + clippy -D warnings; WASM-Build + 678 FE-Tests), Browser-Smoke als Human-UAT offen.

### v1.7 Automatische Feiertage & Freiwilligen-Abwesenheit (Phasen 25–26) — COMPLETE & VERIFIED 2026-06-28

**Milestone Goal:** Feiertage werden automatisch (statt manuell pro Mitarbeiter) im Report angerechnet, und Urlaub von Freiwilligen verzerrt die Jahresansicht nicht mehr.

- [x] **Phase 25: Feiertags-Auto-Anrechnung & Stichtag-Konfiguration** — Vollständige Feiertags-Automatik mit konfigurierbarem Aktivierungsstichtag: auto-Anrechnung in `reporting.rs` (Wirkung identisch zu manuellem ExtraHours(Holiday)), Konflikt-/Doppelzähl-Schutz, Admin-Settings-UI für den Stichtag und ggf. Snapshot-Schema-Bump (BE+FE).
- [x] **Phase 26: Freiwilligen-Abwesenheit & Cross-Navigation** — Urlaub von Freiwilligen reduziert ihre committed-Zusage 🎯 in der Jahresansicht (`booking_information.rs::get_weekly_summary`); bidirektionale Deep-Links zwischen `/absences` und Mitarbeiterreport/Jahresansicht (BE+FE).

<details>
<summary>✅ v1.6 Paid-Capacity-Durchsetzung & Konfiguration (Phase 24) — SHIPPED 2026-06-27</summary>

- [x] Phase 24: Paid-Limit konfigurierbar & rollenbasiert durchsetzen (BE+FE) (5/5 plans) — D-24-01..08 + strikt-größer-Grenzregel

Globaler hart/weich-Toggle (`paid_limit_hard_enforcement`, Default weich), pre-persist
Hard-Block (Shiftplanner-Bypass, HTTP 409), admin-gated `/settings/`-Seite, persistente
Overage-Sektion für alle Rollen, Permission-Gate-Fix HR→Shiftplanner.

Vollständige Phasen-Details, Decisions und Closeout:
[`milestones/v1.6-ROADMAP.md`](milestones/v1.6-ROADMAP.md)

</details>

<details>
<summary>✅ v1.5 Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen (Phasen 18–23) — SHIPPED 2026-06-27</summary>

- [x] Phase 18: Report-/Balance-Korrektheit (BE) (2/2 plans) — UV-04, UV-05
- [x] Phase 19: Convert-Dialog UX (FE+BE) (2/2 plans) — UV-01, UV-02
- [x] Phase 20: Absences-Indikator & Jahres-Histogramm (FE) (2/2 plans) — UV-03, YV-01/02/03
- [x] Phase 21: Tabellen-Lesbarkeit (FE) (1/1 plan) — UI-01, UI-02
- [x] Phase 22: Mitarbeiter-Statistik HR (BE+FE) (2/2 plans) — STAT-01, STAT-02
- [x] Phase 23: Frontend Slot Paid-Capacity UI (FE) (2/2 plans)

Vollständige Phasen-Details, Success-Criteria und Requirements:
[`milestones/v1.5-ROADMAP.md`](milestones/v1.5-ROADMAP.md) · [`milestones/v1.5-REQUIREMENTS.md`](milestones/v1.5-REQUIREMENTS.md)

</details>

<details>
<summary>✅ v1.4 Committed Voluntary Capacity (Phasen 14–17) — SHIPPED 2026-06-25</summary>

- [x] Phase 14: Data-model foundation (backend) (2/2 plans) — CVC-01/02/03
- [x] Phase 15: Reporting no-double-count (Achse B only, kein Snapshot-Bump) (2/2 plans) — CVC-04/05/06
- [x] Phase 16: Jahresansicht display (3/3 plans) — CVC-07/08
- [x] Phase 17: Contract editor input + „alle"-Filter / unpaid-volunteer path (4/4 plans) — CVC-09/10

Vollständige Phasen-Details, Success-Criteria und Audit:
[`milestones/v1.4-ROADMAP.md`](milestones/v1.4-ROADMAP.md) · [`milestones/v1.4-MILESTONE-AUDIT.md`](milestones/v1.4-MILESTONE-AUDIT.md)

</details>

## Phase Details

### Phase 25: Feiertags-Auto-Anrechnung & Stichtag-Konfiguration

**Goal**: Feiertage werden automatisch und korrekt im Mitarbeiterreport angerechnet — mit identischer Wirkung zu einem manuellen ExtraHours(Holiday)-Eintrag — und ein Admin kann den Aktivierungsstichtag über eine Settings-UI setzen.
**Depends on**: Phase 24 (Settings-UI-Pattern aus v1.6; Toggle-/Konfig-Infrastruktur vorhanden)
**Requirements**: HOL-01, HOL-02, HOL-03, HCFG-01, HCFG-02, HCFG-03, HSNAP-01
**Success Criteria** (what must be TRUE):

  1. Ein Mitarbeiter mit laut Vertrag am betreffenden Wochentag arbeitendem Feiertag hat im Report denselben `holiday_hours`-Wert wie bei einem äquivalenten manuellen ExtraHours(Holiday)-Eintrag — verifiziert per Vergleichstest (HOL-01, HOL-02).
  2. Feiertage vor dem konfigurierten "aktiv ab"-Datum werden von der Automatik nicht angerechnet; bestehende manuelle Einträge und historische Snapshots bleiben davon unberührt (HCFG-01).
  3. Ein Admin kann das "aktiv ab"-Datum in der admin-gated Settings-UI setzen, ändern und nach Seitenreload wiederfinden; alle Texte sind in de/en/cs übersetzt (HCFG-02).
  4. Hat ein Feiertag bereits einen manuellen ExtraHours(Holiday)-Eintrag, erscheint er nicht doppelt im Report — Konfliktregel greift (HCFG-03).
  5. `paid_hours`, `committed_voluntary_hours` und `volunteer_hours` in der Jahresansicht sind durch die Feiertags-Automatik unverändert (HOL-03 Regressions-Guard); `CURRENT_SNAPSHOT_SCHEMA_VERSION` ist bei Bedarf auf 11 gebumpt (HSNAP-01).

**Plans**: 4/4 plans complete
**Wave 1**

- [x] 25-01-PLAN.md — Toggle `value`-Spalte Infrastruktur (Migration + DAO + Service + REST, HCFG-01/02 Backend)

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 25-02-PLAN.md — Reporting derive-on-read (3 Injektionspunkte) + main.rs-DI-Fix + Snapshot-Bump 10→11 (HOL-01/02/03, HCFG-01/03, HSNAP-01)
- [x] 25-03-PLAN.md — Frontend Settings-Datumsfeld „aktiv ab" + i18n de/en/cs (HCFG-02)

**Wave 3** *(blocked on Wave 2 completion)*

- [x] 25-04-PLAN.md — Feiertags-Auto-Anrechnung Tests (HOL-01/02/03, HCFG-01/03)

**UI hint**: yes

### Phase 26: Freiwilligen-Abwesenheit & Cross-Navigation

**Goal**: Urlaub von Freiwilligen verzerrt die Jahresansicht nicht mehr (committed-Zusage wird für Abwesenheitswochen korrekt reduziert), und Benutzer können per Deep-Link direkt zwischen Abwesenheitsansicht und Mitarbeiterreport navigieren.
**Depends on**: Phase 25 (Feiertags-Guard HOL-03 muss stabil sein, damit VFA-02-Asymmetrie sauber verifizierbar ist)
**Requirements**: VFA-01, VFA-02, NAV-01
**Success Criteria** (what must be TRUE):

  1. In der Jahresansicht zeigt ein Freiwilliger (`is_paid=false`, `committed_voluntary>0`) für Wochen mit Urlaub/Abwesenheit eine niedrigere committed-Zusage 🎯 als für Wochen ohne Abwesenheit (VFA-01).
  2. Feiertage senken die committed-Zusage eines Freiwilligen nicht — die Asymmetrie ist per Regressionstest abgesichert (VFA-02).
  3. Von der Jahresansicht/Mitarbeiterreport führt ein Link direkt zur Abwesenheitsansicht des jeweiligen Mitarbeiters (Sales: eigene Ansicht; HR: Mitarbeiter-Filter vorbelegt); alle Beschriftungen in de/en/cs (NAV-01a).
  4. Von der Abwesenheitsansicht führt ein Link pro Mitarbeiter direkt zur Jahresansicht/Mitarbeiterreport desselben Mitarbeiters; alle Beschriftungen in de/en/cs (NAV-01b).

**Plans**: 3/3 plans complete

**Wave 1** *(parallel — disjoint file sets)*

- [x] 26-01-PLAN.md — Backend VFA-01: AbsenceService-DI in BookingInformationService + Wochen-Overlap-Reduktion in get_weekly_summary + pure-helper-Tests (VFA-01, VFA-02; D-26-01/02/03/04)
- [x] 26-03-PLAN.md — Frontend NAV-01: Route /absences/:employee_id + AbsencesFor-Wrapper/Preselect + 4 Ghost-Button-Cross-Links + 4 i18n-Keys de/en/cs (NAV-01; D-26-05/06)

**Wave 2** *(blocked on 26-01)*

- [x] 26-02-PLAN.md — Backend VFA-02-Regressionstest: Feiertags-vs-Abwesenheits-Asymmetrie in get_weekly_summary (full-service) + No-Snapshot-Bump-Guard (VFA-01, VFA-02; D-26-01/02/03/04)

**UI hint**: yes

## Milestone v1.8 — Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX)

### Phase 27: Freiwillige in Abwesenheitsliste auswählbar (FE)

**Goal**: Auf der Abwesenheitsseite lassen sich auch Freiwillige (`sales_person.is_paid == false`) auswählen — sowohl beim Anlegen einer Abwesenheit (AbsenceModal) als auch im HR-Personenfilter (AbsenceFilterBar). Angestellte und Freiwillige sind im selben Dropdown sichtbar getrennt (gruppierter Selector), nicht vermischt.
**Depends on**: Phase 26 (Backend-VFA für Freiwilligen-Abwesenheiten muss stabil sein)
**Requirements**: VOL-SEL-01
**Success Criteria** (what must be TRUE):

  1. Im AbsenceModal-Personen-Dropdown erscheinen aktive Freiwillige (`!inactive && !is_paid`) in einer eigenen, beschrifteten Gruppe „Freiwillige" 🎯 unterhalb der Gruppe „Angestellte" — und eine Abwesenheit kann für sie angelegt werden.
  2. Im AbsenceFilterBar-Personenfilter (HR) sind Freiwillige genauso gruppiert auswählbar; die bestehende „Alle"-Option bleibt erhalten.
  3. Inaktive Personen (`inactive`) bleiben in beiden Selektoren ausgeblendet — egal ob Angestellte oder Freiwillige.
  4. Beide Gruppen-Beschriftungen sind in allen drei Locales (de/en/cs) vorhanden (neue i18n-Keys `AbsenceGroupEmployees` / `AbsenceGroupVolunteers`).
  5. Leere Gruppen werden nicht gerendert (kein leeres `optgroup`, wenn es z.B. keine Freiwilligen gibt).

**Konzept-Eckpunkte** (entschieden, 2026-06-29):

  - **Selector-UX**: gruppierter Dropdown via native `optgroup` (Angestellte zuerst, dann Freiwillige). `SelectInput` bleibt unverändert (rendert `children` → Aufrufer übergeben `optgroup`/`option`).
  - **Geltungsbereich**: Modal UND Filter — gemeinsamer Helfer für beide Call-Sites (kein Copy-Paste).
  - **Filter-Predicate**: `is_selectable_employee` von „`is_paid && !inactive`" auf „`!inactive`" reduzieren; `is_paid` wandert in die Gruppierung. Bestehende Call-Sites prüfen, ob die Lockerung anderswo unerwünscht greift.
  - **Backend**: keine Änderung — Phase 26 (VFA) + EmployeeWorkDetails seit Phase 17 unterstützen Freiwilligen-Abwesenheiten bereits.

**Offener Punkt für die Planung**: Welche Abwesenheits-Kategorien (Urlaub / Krank / Unbezahlt) sind für Freiwillige sinnvoll? Betrifft nur das Kategorie-Dropdown, nicht den Personen-Selector.

**Plans**: 1/1 plans complete (Executed 2026-06-29; Browser-Smoke = Human-UAT offen)

Plans:

- [x] 27-01-PLAN.md — Pure `grouped_selectable` + `PersonGroup` + RSX-Helfer `grouped_person_options`; beide Call-Sites (Modal + FilterBar) umgestellt; 2 i18n-Keys de/en/cs; 5 neue Pure-Function-Tests (D-27-01..06)

### Phase 28: Urlaubsanspruch-Korrektur via Offset (HR, BE+FE)

**Goal**: HR kann den berechneten Jahres-Urlaubsanspruch einer Person um einen signed **Offset (Korrektur-Delta)** anheben/senken, um Rundungs-/Proration-Differenzen auszugleichen (z.B. Shifty rechnet 17 → HR setzt +1 → angezeigt 18). Der Offset wandert bei Vertragsänderungen mit (Delta, kein absoluter Override). In der HR-Ansicht ist die Korrektur gekennzeichnet und editierbar; für normale User ist sie unsichtbar (nur die finale Zahl).
**Depends on**: Phase 26 (vacation_balance / EmployeeWorkDetails stabil); unabhängig von Phase 27
**Requirements**: VAC-OFFSET-01
**Success Criteria** (what must be TRUE):

  1. Für eine Person mit gesetztem Offset gilt in der HR-Urlaubsübersicht `entitled_effective = round(berechneter Anspruch) + offset` 🎯; das wirkt automatisch auf `remaining_days` durch (Anspruch + Carryover − Verbraucht − Geplant).
  2. Der Offset ist **signed** (positiv wie negativ setzbar) und pro **Person + Jahr** persistiert; nach Seitenreload bleibt er erhalten.
  3. Ändert sich später der Vertrag, bleibt der Offset bestehen und wird auf den neu berechneten Anspruch angewandt (Delta-Verhalten, nicht eingefroren).
  4. In der HR-Personen-Detailansicht zeigt die „Vertragsanspruch"-StatBox den Effektivwert plus ein **immer sichtbares, signed Inline-Offset-Zahlenfeld** mit Beschriftung „berechnet {n} + Offset [x]"; Änderung speichert HR-gated (on-blur/Enter).
  5. In der User-Eigenansicht erscheint in derselben StatBox **nur der Effektivwert** — kein „berechnet/Offset", kein Eingabefeld.
  6. Das Setzen/Ändern des Offsets ist HR-gated (`HR_PRIVILEGE`); neue Texte in de/en/cs.

**Konzept-Eckpunkte** (entschieden, 2026-06-29):

  - **Mechanismus**: Offset/Delta, NICHT absoluter Override (User-Entscheidung) — überlebt Vertragsänderungen.
  - **Datenmodell**: neue kleine Tabelle (z.B. `vacation_entitlement_offset`: `sales_person_id`, `year`, `offset_days` signed, `version`, `created`, `deleted`).
  - **Backend**: Offset in `vacation_balance`-Berechnung addieren (nach `.round()` bei `service_impl/src/vacation_balance.rs:191`); HR-gated CRUD-Endpoint.
  - **Frontend-Platzierung**: in der **Personen-Detailansicht** (`VacationEntitlementSelfBody`, im HR-Kontext via `forced_self`), an der **„Vertragsanspruch"-StatBox** (`VacationStatContract` = `entitled_days`). HR erreicht sie per Klick auf eine Person in `VacationPerPersonList`.
  - **Edit-Control (entschieden)**: **Inline-Zahlenfeld** — in der HR-Detailansicht immer sichtbar, signed Offset, Anzeige „berechnet 17 + Offset [1]" → die große/Box-Zahl zeigt den Effektivwert 18. Speichern on-blur/Enter (HR-gated). Self-Body bekommt dafür ein `is_hr`-Flag durchgereicht.
  - **User-Ansicht**: dieselbe StatBox zeigt **nur den Effektivwert**, kein „berechnet/Offset", kein Feld.
  - **Optional**: kleiner Indikator an Personen mit Offset in der kompakten `VacationPerPersonList` (editiert wird aber nur im Detail).

**Offene Punkte für die Planung**:

  1. „Für User unsichtbar" — UI-only oder API-level? (Self-Endpoint „HR ∨ self" liefert den Offset sonst in der rohen Antwort mit. Empfehlung: API-seitig im Self-Pfad weglassen, sauberer als nur UI-Ausblenden.)
  2. Off-by-one in der Proration (`employee_work_details.rs:173`, `ordinal()` statt `ordinal()-1`) als Begleit-Fix mitnehmen oder bewusst draußen lassen?
  3. Snapshot-Bump prüfen: Urlaub ist vermutlich kein billing `value_type` → kein Bump; bei Planung verifizieren.

**Plans**: 4/4 plans complete (Executed 2026-06-29; Browser-Smoke = Human-UAT offen)

Plans:

- [x] 28-01-PLAN.md — Backend data layer: additive migration + `vacation_entitlement_offset` table + DAO trait/sqlite impl + Basic HR-gated `VacationEntitlementOffsetService` + CRUD/HR-gate tests (D-28-01, D-28-06, D-28-06b) [Wave 1]
- [x] 28-02-PLAN.md — VacationBalance integration: offset-after-`.round()`, API-level hiding (HR-only breakdown), `VacationBalanceTO` + domain fields, HR-gated REST CRUD endpoint, DI wiring, offset/delta/hiding tests (D-28-02, D-28-03, D-28-06b, D-28-09) [Wave 2]
- [x] 28-03-PLAN.md — Off-by-one proration fix (`vacation_days_for_year`, year-start only) + snapshot bump 11→12 (VacationEntitlement) + guard/regression tests (D-28-04, D-28-05) [Wave 1]
- [x] 28-04-PLAN.md — Frontend inline signed offset editor in HR person-detail StatBox (“berechnet {n} + Offset [x]”), user-side effective-only, i18n de/en/cs, FE state/api/service plumbing (D-28-07, D-28-03, D-28-08, D-28-09) [Wave 3]

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1 — Absence Domain Foundation | v1.0 | 5/5 | Complete | 2026-05-01 |
| 2 — Reporting Integration & Snapshot Versioning | v1.0 | 4/4 | Complete | 2026-05-02 |
| 3 — Booking & Shift-Plan Konflikt-Integration | v1.0 | 6/6 | Complete | 2026-05-02 |
| 4 — Migration & Cutover | v1.0 | 8/8 | Complete | 2026-05-03 |
| 5 — Slot Paid Capacity Warning | v1.1 | 6/6 | Complete | 2026-05-04 |
| 6 — rest-types Unification & Frontend Compile-Through | v1.2 | 5/5 | Complete | 2026-05-07 |
| 7 — Runtime Smoke & Regression Safety | v1.2 | 1/1 | Complete | 2026-05-07 |
| 8–13 — v1.3 (siehe milestones/v1.3-ROADMAP.md) | v1.3 | — | Closed | 2026-06-22 |
| 14 — Data-model foundation (backend) | v1.4 | 2/2 | Complete | 2026-06-23 |
| 15 — Reporting no-double-count (KEIN Snapshot-Bump) | v1.4 | 2/2 | Complete | 2026-06-24 |
| 16 — Jahresansicht display | v1.4 | 3/3 | Complete | 2026-06-24 |
| 17 — Contract editor input + „alle"-Filter / unpaid-volunteer | v1.4 | 4/4 | Complete | 2026-06-24 |
| 18 — Report-/Balance-Korrektheit (BE) | v1.5 | 2/2 | Complete | 2026-06-26 |
| 19 — Convert-Dialog UX (FE+BE) | v1.5 | 2/2 | Complete | 2026-06-26 |
| 20 — Absences-Indikator & Jahres-Histogramm (FE) | v1.5 | 2/2 | Complete | 2026-06-26 |
| 21 — Tabellen-Lesbarkeit (FE) | v1.5 | 1/1 | Complete | 2026-06-26 |
| 22 — Mitarbeiter-Statistik HR (BE+FE) | v1.5 | 2/2 | Complete | 2026-06-26 |
| 23 — Frontend: Slot Paid-Capacity UI (FE) | v1.5 | 2/2 | Complete | 2026-06-27 |
| 24 — Paid-Limit konfigurierbar & rollenbasiert (BE+FE) | v1.6 | 5/5 | Complete   | 2026-06-27 |
| 25 — Feiertags-Auto-Anrechnung & Stichtag-Konfiguration (BE+FE) | v1.7 | 4/4 | Complete   | 2026-06-28 |
| 26 — Freiwilligen-Abwesenheit & Cross-Navigation (BE+FE) | v1.7 | 3/3 | Complete   | 2026-06-28 |
| 27 — Freiwillige in Abwesenheitsliste auswählbar (FE) | v1.8 | 1/1 | Executed ⚠ smoke | 2026-06-29 |
| 28 — Urlaubsanspruch-Korrektur via Offset (BE+FE) | v1.8 | 4/4 | Executed ⚠ smoke | 2026-06-29 |

## Backlog

Ungeplante / off-theme Arbeit, die NICHT zum aktiven Milestone gehört. Vor Ausführung
in einen Milestone promoten oder per `/gsd-plan-phase 999.1` direkt planen.

- [ ] **Phase 999.1: Breaking/Major Dependency-Migration** (Backend + Frontend, Maintenance) — Alle direkten Deps mit verfügbaren Major-Releases über beide Cargo-Workspaces (Backend-Root + `shifty-dioxus/`, 9 Member-Crates) auf den neuen Major heben (Cargo.toml-Constraint-Edits + Code-/API-Migration). **Off-theme zu v1.6** (Paid-Capacity) → bewusst Backlog.

  **Goal:** Reproduzierbares Breaking-Update-Tooling etabliert und alle tragbaren Major-Bumps migriert, mit grünen Gates über beide Workspaces — ohne die heiklen Pins (dioxus 0.6.x) ungefragt anzufassen.

  **Context:** Quick-Task `260627-vgo` hat die **semver-kompatible** Baseline bereits geliefert (nur Cargo.lock, alle Gates grün). Offen ist NUR der Breaking/Major-Teil, der dort eskaliert wurde, weil die gepinnte **stable cargo 1.95.0** kein `cargo update --breaking` kann (nightly-only) und weder `cargo-edit` (`cargo upgrade`) noch `cargo-outdated` noch `+nightly` verfügbar sind.

  **Scope / grobe Wave-Struktur:**

  - Task 1 — Toolchain-Enabler: nightly-Toolchain bzw. `cargo-edit`/`cargo-outdated` ins `flake.nix` aufnehmen, sodass `cargo update --breaking` oder `cargo upgrade --incompatible` reproduzierbar laufen.
  - Task 2 — Major-Bump-Inventar: welche direkten Deps, welcher Sprung, Changelog-/Breaking-Risiko (beide Workspaces).
  - Task 3 — iterativ pro Major migrieren mit Gates: Backend `cargo build` + `cargo clippy --workspace -- -D warnings` + `cargo test --workspace`; Frontend `cargo build --target wasm32-unknown-unknown` (nix-shell -p openssl pkg-config lld) + `cargo test`.

  **Constraints:**

  - **dioxus-Major** (0.6.x-Pin) NUR mit expliziter User-Freigabe — dx-CLI-0.7-Inkompatibilität dokumentiert (App startet nicht + Design gestrippt).
  - `flake.lock` Nix-Inputs sind NICHT Teil dieser Phase (separater Maintenance-Job).
  - jj-Repo: User committet manuell, keine git-Fallbacks.

  **Depends on:** Quick-Task `260627-vgo` (compatible baseline) ✅
  **Plans:** noch nicht geplant — `/gsd-plan-phase 999.1`

*Last updated: 2026-06-29 — **Milestone v1.8** (Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur, HR-UX): Phase 27 (VOL-SEL-01, gruppierter Selector in Modal + Filter, FE) und Phase 28 (VAC-OFFSET-01, signed Urlaubsanspruch-Offset pro Person+Jahr, HR-gekennzeichnet/User-unsichtbar, BE+FE) hinzugefügt. v1.7 (Phasen 25–26) bleibt complete/verified. Nächster Schritt: `/gsd-plan-phase 27` bzw. `/gsd-plan-phase 28`.*
