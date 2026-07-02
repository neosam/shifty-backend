# Roadmap: Shifty Backend

## Milestones

- ✅ **v1.0 Range-Based Absence Management** — Phasen 1–4 (shipped 2026-05-03) — siehe [`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md)
- ✅ **v1.1 Slot Capacity & Constraints** — Phase 5 (shipped 2026-05-04) — siehe [`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md)
- ✅ **v1.2 Frontend rest-types Konsolidierung** — Phasen 6–7 (shipped 2026-05-07) — siehe [`milestones/v1.2-ROADMAP.md`](milestones/v1.2-ROADMAP.md)
- ✅ **v1.3 Frontend Abwesenheiten + UI-Closure-Restanten** — Phasen 8–13 (closed 2026-06-22) — siehe [`milestones/v1.3-ROADMAP.md`](milestones/v1.3-ROADMAP.md)
- ✅ **v1.4 Committed Voluntary Capacity** — Phasen 14–17 (shipped 2026-06-25) — siehe [`milestones/v1.4-ROADMAP.md`](milestones/v1.4-ROADMAP.md)
- ✅ **v1.5 Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen** — Phasen 18–23 (shipped 2026-06-27) — siehe [`milestones/v1.5-ROADMAP.md`](milestones/v1.5-ROADMAP.md)
- ✅ **v1.6 Paid-Capacity-Durchsetzung & Konfiguration** — Phase 24 (shipped 2026-06-27) — siehe [`milestones/v1.6-ROADMAP.md`](milestones/v1.6-ROADMAP.md)
- ✅ **v1.7 Automatische Feiertage & Freiwilligen-Abwesenheit** — Phasen 25–26 (shipped 2026-06-29) — siehe [`milestones/v1.7-ROADMAP.md`](milestones/v1.7-ROADMAP.md)
- ✅ **v1.8 Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX)** — Phasen 27–28 (shipped 2026-06-29) — siehe [`milestones/v1.8-ROADMAP.md`](milestones/v1.8-ROADMAP.md)
- ✅ **v1.9 Schichtplan-/Urlaubs-UX-Korrekturen & Admin-Impersonation** — Phasen 29–32 (shipped 2026-06-29) — siehe [`milestones/v1.9-ROADMAP.md`](milestones/v1.9-ROADMAP.md)
- ✅ **v1.10 Feiertage — UI-Pflege & Schichtplan-Soll-Konsistenz** — Phasen 33–35 (shipped 2026-06-30) — siehe [`milestones/v1.10-ROADMAP.md`](milestones/v1.10-ROADMAP.md)
- ✅ **v1.11 Stabilisierung & UX-Politur** — Phasen 36–38 (shipped 2026-07-01) — siehe [`milestones/v1.11-ROADMAP.md`](milestones/v1.11-ROADMAP.md)
- 🚧 **v2.1 Schichtplan- & Reporting-Erweiterungen** — Phasen 39–42 (aktiv, gestartet 2026-07-01) — KW-Status & Sperre (WST), Ø-Anwesenheit flexible Stunden (AVG), Special-Days-Button-Bugfix (SDF)

## Phases

> **🚧 Aktiver Milestone: v2.1 Schichtplan- & Reporting-Erweiterungen** (gestartet 2026-07-01, autonomer Nacht-Run) — 4 Phasen (39–42), 9/9 Requirements gemappt.
>
> **Milestone-Ziel:** Zwei neue Steuerungs-/Auswertungs-Fähigkeiten für die Schichtplanung — Kalenderwochen-Status mit Sperr-Gate (WST) und eine Durchschnitts-Anwesenheits-Auswertung für flexible Stunden (AVG) — plus ein isolierter mitreitender Settings-Bugfix (SDF). Jede Phase umfasst Backend UND Frontend (GSD-Scope-Regel), außer der isolierte FE-only-Bugfix (Phase 42).
>
> **Versions-Hinweis:** `v2.1` ist das GSD-Planungs-Label (MAJOR.MINOR). Reale Release-Version via `/release-version` → `./cli-update-version.sh` (PATCH aus Git-Tags). Releases aus diesem Milestone = v2.1.0 ff.
>
> **Querschnittliche Gates (jede Phase, autonomer Run):** nach jeder neuen `query!`/`query_as!` → `cargo sqlx prepare --workspace` (in `nix develop`) + `.sqlx` committen; `cargo clippy --workspace -- -D warnings` (Pflicht-Gate, `cargo test` reicht nicht); Backend `cargo test --workspace`; Frontend `cargo build --target wasm32-unknown-unknown` + `cargo test -p shifty-dioxus`. i18n de/en/cs für alle neuen benutzersichtbaren Texte.

**v2.1 Phasen-Checkliste:**

- [x] **Phase 39: KW-Status Grundlage (BE+FE)** — WST-01, WST-02, WST-05 (completed 2026-07-02)
- [ ] **Phase 40: Wochen-Sperre durchsetzen (BE+FE)** — WST-03, WST-04
- [ ] **Phase 41: Ø-Anwesenheit bei flexiblen Stunden (BE+FE)** — AVG-01, AVG-02, AVG-03
- [ ] **Phase 42: Special-Days-„Anlegen"-Button-Bugfix (FE)** — SDF-01

### Phase 39: KW-Status Grundlage (BE+FE)

**Goal**: Ein Schichtplaner kann jeder Kalenderwoche einen Status (Kein / In Planung / Geplant / Gesperrt) geben, der für alle Rollen als Badge in der Schichtplan-Wochenansicht sichtbar ist.
**Depends on**: Nichts Neues (erste v2.1-Phase; baut auf v1.11 / Phase 38 auf)
**Requirements**: WST-01, WST-02, WST-05
**Success Criteria** (what must be TRUE):

  1. Ein Schichtplaner kann in der Wochenansicht den Status der Woche über einen Aktions-Button setzen/ändern; der Status wird pro ISO-(Jahr, Woche) persistiert und bleibt nach Reload erhalten.
  2. Alle Rollen sehen den aktuellen Wochenstatus als farbkodiertes Badge im Wochen-Header; Nicht-Schichtplaner können ihn nicht ändern (nur Anzeige).
  3. Der Status wird an der ISO-Wochen-Jahresgrenze korrekt zugeordnet (KW-53-/Jahreswechsel-Tage landen in der richtigen (Jahr, Woche)-Zeile) — durch Unit-Tests belegt.
  4. Alle vier Status-Labels erscheinen lokalisiert in de/en/cs.

**Plans**: 5 plans
**Wave 1**

- [x] 39-01-PLAN.md — Migration + DAO (week_status-Tabelle, WeekStatusDao, TEXT-Diskriminant, .sqlx)

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 39-02-PLAN.md — WeekStatusService (Basic-Tier, TDD: Permission-Gate, Upsert/Soft-Delete, KW-53)

**Wave 3** *(blocked on Wave 2 completion)*

- [x] 39-03-PLAN.md — rest-types WeekStatusTO + REST-Handler/ApiDoc + DI-Wiring in main.rs

**Wave 4** *(blocked on Wave 3 completion)*

- [x] 39-04-PLAN.md — FE-Foundation: WeekStatus-Enum, i18n de/en/cs, API-Client, Fresh-Fetch-Store

**Wave 5** *(blocked on Wave 4 completion)*

- [x] 39-05-PLAN.md — FE-Komponenten: WeekStatusBadge + WeekStatusDropdown + Schichtplan-Integration

**UI hint**: yes

**Offene Entscheidungen (discuss-phase 39):** Wer den Status setzen darf + welche Status-Übergänge erlaubt sind (Default: Schichtplaner, alle Übergänge). UI-Muster Badge + Aktions-Button (kein controlled `<select>`, um D-25-06-Desync zu vermeiden). None-Variante NICHT `None` nennen (Clippy/`Option`-Shadowing → z.B. `Unset`/`Open`).
**Scope (BE+FE):** neue `week_status`-Tabelle + Migration (TEXT-Enum analog `special_day`, ISO-(year, week)-Composite-Key analog `week_message`, partial UNIQUE `WHERE deleted IS NULL`); `WeekStatusService` (Basic-Tier: nur DAO/Permission/Transaction); Status-CRUD-REST (`#[utoipa::path]`, `ToSchema`-DTO); DI-Wiring in `main.rs` (Basic-Tier vor Business-Logic); Frontend Status-Badge + Set-Button (nur Schichtplaner), Status-Reload vom Server nach jeder Änderung. ISO-Jahr immer aus `to_iso_week_date().0` ableiten (nie `date.year()`).

### Phase 40: Wochen-Sperre durchsetzen (BE+FE)

**Goal**: In einer Gesperrt-Woche sind Buchungs- und Slot-Schreibaktionen für Nicht-Schichtplaner auf allen Schreibpfaden server-seitig blockiert; Schichtplaner behalten Vollzugriff.
**Depends on**: Phase 39 (Status-Datenmodell + `WeekStatusService`)
**Requirements**: WST-03, WST-04
**Success Criteria** (what must be TRUE):

  1. Versucht ein Nicht-Schichtplaner in einer Gesperrt-Woche eine Buchung/Slot-Änderung, wird sie abgelehnt (`ServiceError::WeekLocked` → HTTP 423 Default) mit lokalisierter Rückmeldung; das Frontend zeigt die Woche read-only + nicht-blockierendes Inline-Banner bei 423.
  2. Ein Schichtplaner kann in derselben Gesperrt-Woche weiterhin alle Schreibaktionen ausführen.
  3. Die Sperre greift auf allen sechs Schreibpfaden ohne Bypass (`book_slot_with_conflict_check`, `modify_slot`, `modify_slot_single_week`, `remove_slot`, `copy_week_with_conflict_check`, neu `delete_booking` inkl. Re-Routing von `DELETE /booking/{id}`) — belegt durch Test-Matrix 6 Pfade × {gesperrt, offen}.
  4. Der Sperr-Check läuft in derselben Transaktion wie der Write (kein TOCTOU) — durch Test/Review belegt.

**Plans**: 4 plans
**Wave 1**

- [x] 40-01-PLAN.md — Backend contract + DI scaffold (WeekLocked+423, delete_booking trait, WeekStatusService dep, pass-through gate) [W1]
- [x] 40-02-PLAN.md — Frontend: +/- Buttons ausblenden in Locked-Woche + i18n WeekLockedError [W1]

**Wave 2** *(blocked on Wave 1 completion)*

- [ ] 40-03-PLAN.md — TDD Lock-Enforcement (assert_week_not_locked blockt 6 Pfade; 6×2+TOCTOU+delete-Reihenfolge) [W2]

**Wave 3** *(blocked on Wave 2 completion)*

- [ ] 40-04-PLAN.md — REST Re-Routing DELETE /booking → delete_booking (WST-04) + OpenAPI 423 [W3]

**UI hint**: yes

**Offene Entscheidungen (discuss-phase 40):** HTTP-Code für Locked-Write (Default **423 Locked**; 409-Alternative geprüft — Konsistenz mit `PaidLimitExceeded`-409-Präzedenz abwägen).
**Scope (BE+FE):** geteilter `assert_week_not_locked(year, week, context, tx)`-Helper, aufgerufen am Kopf aller sechs Schreibmethoden im Business-Logic-Tier (`ShiftplanEditService`); **neue** `ShiftplanEditService::delete_booking`-Methode + Re-Routing des `DELETE /booking/{id}`-Handlers weg von `BookingService::delete` (schließt den einzigen echten Nicht-Schichtplaner-Bypass); `ServiceError::WeekLocked { year, week }` → HTTP-Code in `rest/src/lib.rs` (+ OpenAPI-Annotation); Frontend read-only-Woche + Inline-423-Banner; i18n de/en/cs der Write-Block-Meldung.

### Phase 41: Ø-Anwesenheit bei flexiblen Stunden (BE+FE)

**Goal**: HR kann die durchschnittliche tatsächliche Anwesenheit flexibler Mitarbeiter über einen Zeitraum einsehen, wobei Urlaub aus dem Nenner herausgerechnet ist.
**Depends on**: Nichts Hartes (fachlich unabhängig von WST; nach Phase 40 sequenziert, da WST höheres Regressionsrisiko trägt und zuerst stabil sein soll)
**Requirements**: AVG-01, AVG-02, AVG-03
**Success Criteria** (what must be TRUE):

  1. HR kann pro flexiblem Mitarbeiter (`EmployeeWorkDetails.is_dynamic == true`) die durchschnittliche tatsächliche Anwesenheit über einen Zeitraum einsehen; Urlaub ist aus dem Nenner herausgerechnet.
  2. Nicht-flexible Mitarbeiter erscheinen nicht in der Auswertung (server-seitiger `is_dynamic`-Filter); Nicht-HR-Rollen haben keinen Zugriff.
  3. Die Auswertung ist im Frontend als Report-/Auswertungs-Sicht sichtbar inkl. Leerzustand; Labels/Tooltips in de/en/cs.
  4. Die Auswertung ist ein reines Read-Aggregat — kein Snapshot-Bump, keine neue Persistenz, kein neuer `BillingPeriodValueType` (`CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 12) — grep-/test-verifiziert.

**Plans**: TBD
**UI hint**: yes

**Offene Entscheidungen (discuss-phase 41, D-AVG-01..08):** Bezugsgröße (Woche/Monat/Abrechnungsperiode); Zähler (geleistete Stunden vs. Anwesenheitstage); exaktes Exclusion-Set (nur Urlaub vs. auch Krankheit/unbezahlt/Feiertag — **A-22-1 schließt ALLE Absence-Kategorien aus und ist NICHT identisch**); Mitarbeiter-Scope (`is_dynamic == true` bestätigen); Anzeige-Ort (Abrechnungsperioden-Report vs. eigenständige Sicht); Mindest-Datenschwelle; No-Persist-Bestätigung.
**Scope (BE+FE):** neue Read-Aggregat-Methode im `ReportingService` (Business-Logic-Tier) — A-22-1 NICHT blind wiederverwenden, ggf. eigene Funktion (A-22-1 selbst nie ändern); HR-gated REST-Endpoint (`#[utoipa::path]`); Frontend-Report-Sicht; i18n de/en/cs. Kein neuer `BillingPeriodValueType`, keine Migration.

### Phase 42: Special-Days-„Anlegen"-Button-Bugfix (FE)

**Goal**: Nach dem Anlegen eines Special-Day bleibt der „Anlegen"-Button aktiv; mehrfaches Anlegen hintereinander ist ohne Dropdown-Toggle möglich.
**Depends on**: Nichts (isoliert, FE-only, niedrigstes Risiko — bewusst zuletzt platziert)
**Requirements**: SDF-01
**Success Criteria** (what must be TRUE):

  1. Nach erfolgreichem Special-Day-Anlegen bleibt der „Anlegen"-Button aktiv und Typ/Datum stehen unverändert (Option 2 — nach Create nichts zurücksetzen).
  2. Ein User kann mehrere Special-Days hintereinander anlegen, ohne das Dropdown neu zu togglen.
  3. Ein SSR-/Komponenten-Test deckt das mehrfache Anlegen ab (Formulardaten bleiben erhalten).

**Plans**: TBD
**UI hint**: yes

**Scope (FE-only):** Reset-Block `settings.rs:458-459` (und etwaigen Zeit-Reset) entfernen — umgeht den Controlled-Select-Desync (D-25-06-Klasse) komplett. Kein Backend-Anteil (begründete „Backend out of scope"-Notiz: reiner FE-State-Fix ohne API-Wirkung; SDF-Desync ist ein isolierter Settings-Bug, kein neues/geändertes TO).

<details>
<summary>✅ v1.11 Stabilisierung & UX-Politur (Phasen 36–38) — SHIPPED 2026-07-01</summary>

- [x] Phase 36: Special-Days-Bugfixes (BE+FE) (2/2 plans) — SDF-01, SDF-02
- [x] Phase 37: Modal-UX-Politur (FE) (2/2 plans) — MOD-01, MOD-02
- [x] Phase 38: Frontend-Build-Hygiene (2/2 plans) — HYG-01, HYG-02

Konsolidierung nach der v1.7–v1.10-Feature-Welle: vier gemeldete Bugs abgeräumt und der
Frontend-Build warnungsfrei gemacht. SDF-01 atomarer in-place Special-Day-Replace (Feiertag↔
Kurzer-Tag ohne Fehler, HTTP 422→success); SDF-02 controlled SelectInput; MOD-01 zentrale
drag-sichere Backdrop-Schließ-Logik (`BackdropPress`) inkl. absence_convert_modal; MOD-02
pro-Feld-Help-Texte im Arbeitsvertrag-Modal (de/en/cs); HYG-01 `shifty-dioxus` cargo-build-
warnungsfrei; HYG-02 Backend-Clippy-Gate grün. Kein Snapshot-Bump (bleibt 12), keine Migration,
keine neuen Deps. Audit `passed` (6/6 Requirements, Integration clean, 4/4 Flows).

Vollständige Phasen-Details, Decisions und Closeout:
[`milestones/v1.11-ROADMAP.md`](milestones/v1.11-ROADMAP.md) · [`milestones/v1.11-REQUIREMENTS.md`](milestones/v1.11-REQUIREMENTS.md) · [`milestones/v1.11-MILESTONE-AUDIT.md`](milestones/v1.11-MILESTONE-AUDIT.md)

</details>

<details>
<summary>✅ v1.10 Feiertage — UI-Pflege & Schichtplan-Soll-Konsistenz (Phasen 33–35) — SHIPPED 2026-06-30</summary>

- [x] Phase 33: Special-Days-UI in den Einstellungen (FE) (4/4 plans) — SPD-01..04
- [x] Phase 34: Feiertags-Soll im Schichtplan (BE) (1/1 plan) — HSP-01..04
- [x] Phase 35: Slot-Werte nur für eine Woche ändern (BE+FE) (3/3 plans) — SWO-01..04

Special Days (Holiday/ShortDay) shiftplanner-gated über die UI pflegbar auf zwei Flächen
(Schichtplan-Wochenraster Per-Tag-Dropdown + Settings-Kalenderdatum-Picker + Jahres-Liste)
gegen die bestehende REST-CRUD + neuen `for-year`-Read-Endpoint. Automatisch angerechneter
Feiertag reduziert das angezeigte Soll in der Schichtplan-Wochentabelle (`get_week` 4.
Injektionspunkt via `build_derived_holiday_map`), konsistent zum Stundenkonto, Kapazitätsbänder
unangetastet (D-25-08). Slot-Werte für genau eine KW als einmalige Ausnahme via 3-Segment-
Split+Re-Merge (atomar, Buchungs-Re-Point ohne Doppelzählung) + UI-Wahl „nur diese Woche"/
„ab dieser Woche". Kein Snapshot-Bump (bleibt 12), keine Migration, keine neuen Deps. Audit
`passed` (12/12 Requirements, Integration clean, 2/2 E2E-Flows).

Vollständige Phasen-Details, Decisions und Closeout:
[`milestones/v1.10-ROADMAP.md`](milestones/v1.10-ROADMAP.md) · [`milestones/v1.10-REQUIREMENTS.md`](milestones/v1.10-REQUIREMENTS.md) · [`v1.10-MILESTONE-AUDIT.md`](v1.10-MILESTONE-AUDIT.md)

</details>

<details>
<summary>✅ v1.9 Schichtplan-/Urlaubs-UX-Korrekturen & Admin-Impersonation (Phasen 29–32) — SHIPPED 2026-06-29</summary>

- [x] Phase 29: Urlaubs-Balken-Konsistenz (FE) (1/1 plan) — VAC-01
- [x] Phase 30: Stale-Daten-Race Guard (FE) (1/1 plan) — SHP-02
- [x] Phase 31: Abwesenheit → Nicht-Verfügbar-Markierung im Schichtplan (FE) (1/1 plan) — SHP-01
- [x] Phase 32: Admin-Impersonation Frontend + Audit-Schicht (FE+BE) (3/3 plans) — IMP-01..04

Drei Schichtplan-/Urlaubs-UX-Fixes + vollwertige Admin-Impersonation mit Audit der echten
Admin-Identität. Urlaubsbalken `(used+planned)/total` (Überzug als Farb-Signal); geteilter
`(year,week)`-Staleness-Guard über alle Summary-Loader; proaktive „Nicht Verfügbar"-Markierung
eigener/ausgewählter Absence-Tage (kategorie-treu zur `BookingOnAbsenceDay`-Warnung, null Drift);
Impersonation-FE (nicht-schließbarer Banner, reload-persistent, Users-Tab-Einstieg) + zentrale
Audit-Middleware (`RealUser`) + Store-Teardown — ohne `Authentication<Context>`-Signatur-Change,
ohne Snapshot-Bump, ohne Migration. Audit `passed` (7/7 Requirements, 4/4 Integration + E2E).

Vollständige Phasen-Details, Decisions und Closeout:
[`milestones/v1.9-ROADMAP.md`](milestones/v1.9-ROADMAP.md) · [`milestones/v1.9-REQUIREMENTS.md`](milestones/v1.9-REQUIREMENTS.md) · [`milestones/v1.9-MILESTONE-AUDIT.md`](milestones/v1.9-MILESTONE-AUDIT.md)

</details>

<details>
<summary>✅ v1.8 Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX) (Phasen 27–28) — SHIPPED 2026-06-29</summary>

- [x] Phase 27: Freiwillige in Abwesenheitsliste auswählbar (FE) (1/1 plan) — VOL-SEL-01
- [x] Phase 28: Urlaubsanspruch-Korrektur via Offset (BE+FE) (4/4 plans) — VAC-OFFSET-01

Gruppierter Personen-Selector (optgroup Angestellte/Freiwillige) in AbsenceModal +
AbsenceFilterBar via gemeinsamem Helfer; signed Urlaubsanspruch-Offset pro Person+Jahr
(Delta, kein Override), HR-gekennzeichnet+editierbar, für User unsichtbar (API-level
Hiding) + Off-by-one-Proration-Fix + Snapshot-Bump 11→12. Audit `passed`.

Vollständige Phasen-Details, Decisions und Closeout:
[`milestones/v1.8-ROADMAP.md`](milestones/v1.8-ROADMAP.md) · [`milestones/v1.8-REQUIREMENTS.md`](milestones/v1.8-REQUIREMENTS.md) · [`milestones/v1.8-MILESTONE-AUDIT.md`](milestones/v1.8-MILESTONE-AUDIT.md)

</details>

<details>
<summary>✅ v1.7 Automatische Feiertage & Freiwilligen-Abwesenheit (Phasen 25–26) — SHIPPED 2026-06-29</summary>

- [x] Phase 25: Feiertags-Auto-Anrechnung & Stichtag-Konfiguration (BE+FE) (4/4 plans) — HOL-01..03, HCFG-01..03, HSNAP-01
- [x] Phase 26: Freiwilligen-Abwesenheit & Cross-Navigation (BE+FE) (3/3 plans) — VFA-01/02, NAV-01

Feiertage werden automatisch (derive-on-read, identisch zu manuellem ExtraHours(Holiday))
ab konfigurierbarem Stichtag angerechnet; Urlaub von Freiwilligen reduziert die committed-
Zusage in der Jahresansicht (Feiertage bewusst nicht — Asymmetrie); bidirektionale
Deep-Links /absences ↔ Report. Snapshot-Bump 10→11.

Vollständige Phasen-Details, Success-Criteria und Requirements:
[`milestones/v1.7-ROADMAP.md`](milestones/v1.7-ROADMAP.md) · [`milestones/v1.7-REQUIREMENTS.md`](milestones/v1.7-REQUIREMENTS.md)

</details>

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
| 24 — Paid-Limit konfigurierbar & rollenbasiert (BE+FE) | v1.6 | 5/5 | Complete | 2026-06-27 |
| 25 — Feiertags-Auto-Anrechnung & Stichtag-Konfiguration (BE+FE) | v1.7 | 4/4 | Complete | 2026-06-28 |
| 26 — Freiwilligen-Abwesenheit & Cross-Navigation (BE+FE) | v1.7 | 3/3 | Complete | 2026-06-28 |
| 27 — Freiwillige in Abwesenheitsliste auswählbar (FE) | v1.8 | 1/1 | Complete | 2026-06-29 |
| 28 — Urlaubsanspruch-Korrektur via Offset (BE+FE) | v1.8 | 4/4 | Complete | 2026-06-29 |
| 29 — Urlaubs-Balken-Konsistenz (FE) | v1.9 | 1/1 | Complete    | 2026-06-29 |
| 30 — Stale-Daten-Race Guard (FE) | v1.9 | 1/1 | Complete    | 2026-06-29 |
| 31 — Abwesenheit → Nicht-Verfügbar-Markierung im Schichtplan (FE) | v1.9 | 1/1 | Complete    | 2026-06-29 |
| 32 — Admin-Impersonation Frontend + Audit-Schicht (FE+BE) | v1.9 | 3/3 | Complete    | 2026-06-29 |
| 33 — Special-Days-UI in den Einstellungen (FE) | v1.10 | 4/4 | Complete   | 2026-06-30 |
| 34 — Feiertags-Soll im Schichtplan (BE) | v1.10 | 1/1 | Complete    | 2026-06-30 |
| 35 — Slot-Werte nur für eine Woche ändern (BE+FE) | v1.10 | 3/3 | Complete    | 2026-06-30 |
| 36 — Special-Days-Bugfixes (BE+FE) | v1.11 | 2/2 | Complete    | 2026-07-01 |
| 37 — Modal-UX-Politur (FE) | v1.11 | 2/2 | Complete    | 2026-07-01 |
| 38 — Frontend-Build-Hygiene | v1.11 | 2/2 | Complete    | 2026-07-01 |
| 39 — KW-Status Grundlage (BE+FE) | v2.1 | 5/5 | Complete    | 2026-07-02 |
| 40 — Wochen-Sperre durchsetzen (BE+FE) | v2.1 | 2/4 | In Progress|  |
| 41 — Ø-Anwesenheit bei flexiblen Stunden (BE+FE) | v2.1 | 0/TBD | Not started | - |
| 42 — Special-Days-„Anlegen"-Button-Bugfix (FE) | v2.1 | 0/TBD | Not started | - |

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
  **Plans:** 5/5 plans complete

*Last updated: 2026-07-01 — **v2.1 Milestone-Roadmap erstellt** (Phasen 39–42, 9/9 Requirements gemappt: WST-01/02/05→39, WST-03/04→40, AVG-01/02/03→41, SDF-01→42). v1.0–v1.11 archiviert/collapsed unverändert; Backlog 999.1 unverändert. Nächster Schritt: `/gsd-discuss-phase 39`.*
