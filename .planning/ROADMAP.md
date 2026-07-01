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
- 🚧 **v1.11 Stabilisierung & UX-Politur** — Phasen 36–38 (gestartet 2026-07-01) — 6 Requirements (SDF-01/02, MOD-01/02, HYG-01/02)

## Phases

### 🚧 v1.11 Stabilisierung & UX-Politur (Phasen 36–38) — AKTIV

Konsolidierung nach der v1.7–v1.10-Feature-Welle: vier gemeldete Bugs abräumen und den
Frontend-Build warnungsfrei machen. **Keine neuen Fähigkeiten.** Kein Snapshot-Bump (bleibt 12),
keine Migration, keine neuen Deps. Herkunft: Todo-Backlog. Aufgeteilt aus einem 8-Item-Wunsch
in 3 Meilensteine (v1.11 Stabilisierung · v1.12 Schichtplan/Reporting · v1.13 PDF-Export).

- [x] **Phase 36: Special-Days-Bugfixes (BE+FE)** — SDF-01, SDF-02 (completed 2026-07-01)
  - **SDF-01**: Umstellen eines Tages Feiertag ↔ „Kurzer Tag" aktualisiert den bestehenden
    Special-Day-Eintrag (update statt zweitem insert) — keine Fehlermeldung, neuer Typ
    persistiert. Update-vs-insert-Pfad prüfen (ggf. Backend), Reproduktion + Statuscode erfassen.

  - **SDF-02**: Settings „Anlegen"-Button bleibt nach erstem Feiertag aktiv — `<select>`
    controlled an `sd_type`-Signal binden bzw. Typ nach Create beibehalten; D-25-06-Datepicker-
    Reset mitdenken. SSR-/Komponenten-Test gegen Re-Regression.

  - **Success:** Beide Bugs reproduziert-und-behoben, Regressionstests grün, Backend-Roundtrip
    (create- vs. edit-Pfad) verifiziert. i18n unberührt (keine neuen Texte).

- [ ] **Phase 37: Modal-UX-Politur (FE)** — MOD-01, MOD-02
  - **MOD-01**: Zentraler `dialog.rs`-Fix — Backdrop schließt nur, wenn mousedown UND mouseup
    auf dem Backdrop selbst (mousedown-Ursprung als Flag tracken). Kommt allen Modals zugute.
    Strukturell über Predikat-/Handler-Logik testen (Maus-Drag ist D-25-06-Klasse, schwer
    browser-automatisierbar).

  - **MOD-02**: Arbeitsvertrag-Modal — pro Feld ein Help-Text analog `CapPlannedHoursHelp`
    (`text-small text-ink-muted`), Von/Bis ausgenommen. Neue `*Help`-Keys in de/en/cs.
    SSR-Test: Help-Texte werden unter den Feldern gerendert.

  - **Success:** Drag-innen→mouseup-außen lässt Modal offen (Handler-Logik-Test); alle
    Vertragsfelder (außer Von/Bis) tragen Erklärungssätze in allen drei Locales.

- [ ] **Phase 38: Frontend-Build-Hygiene** — HYG-01, HYG-02
  - **HYG-01**: `shifty-dioxus` warnungsfrei — ~45 rustc-Warnings (14 via `cargo fix`, Rest
    manuell: ungenutzte Methoden/Imports/Variablen entfernen oder begründetes
    `#[allow(dead_code)]`, z. B. `has_sunday_slots` `state/shiftplan.rs:315`).

  - **HYG-02**: Backend bleibt `cargo clippy --workspace -- -D warnings` grün (Regressions-Gate);
    dioxus-Clippy aus der Backend-nix-Shell (E0514 im dioxus-Shell); verbleibende bewusst
    behaltene Lints dokumentiert. (dioxus ins CI-Clippy-Gate aufnehmen ist optional → out of scope.)

  - **Success:** `cargo build` (dioxus) ohne Warnungen; `cargo clippy --workspace -- -D warnings`
    (Backend) grün; FE `cargo test -p shifty-dioxus` + WASM-Build grün.

**Reihenfolge:** 36 → 37 → 38 (Bugs zuerst, Cleanup zuletzt; fachlich unabhängig, parallel
planbar). Nächster Schritt: `/gsd-plan-phase 36` (oder `/gsd-discuss-phase 36`).

### Phase 36: Special-Days-Bugfixes (BE+FE)

**Goal**: Die beiden live gemeldeten Special-Days-Bugs sind reproduziert und behoben — das Umstellen eines Tages Feiertag ↔ „Kurzer Tag" aktualisiert den bestehenden Eintrag (update statt zweitem insert) ohne Fehlermeldung, und der Settings-„Anlegen"-Button bleibt für aufeinanderfolgende Einträge korrekt aktiv. Backend-Roundtrip (create- vs. edit-Pfad) verifiziert, keine neuen i18n-Texte.
**Depends on**: Nothing (Nachlese zu v1.10/Phase 33; fachlich unabhängig von Phasen 37/38)
**Requirements**: SDF-01, SDF-02
**Success Criteria** (what must be TRUE):

  1. Wird im Schichtplan ein Tag von „Feiertag" auf „Kurzer Tag" (oder umgekehrt) umgestellt, aktualisiert der Pfad den bestehenden Special-Day-Eintrag für das Datum (update statt zweitem insert) — keine Fehlermeldung, der neue Typ ist danach persistiert. (SDF-01)
  2. Der Update-vs-insert-Pfad (ggf. Backend) ist geprüft, Reproduktion + Statuscode erfasst und der Backend-Roundtrip create- vs. edit-Pfad verifiziert. (SDF-01)
  3. In der Settings-Special-Days-Karte lassen sich mehrere Feiertage nacheinander anlegen; nach erfolgreichem Create ist der „Anlegen"-Button sofort wieder korrekt aktiviert (kein Controlled-vs-Uncontrolled-Desync zwischen `sd_type`-Signal und `<select>`; D-25-06-Datepicker-Reset mitgedacht). (SDF-02)
  4. Ein SSR-/Komponenten-Test sichert SDF-02 gegen Re-Regression ab; Regressionstests grün. (SDF-02)
  5. i18n unberührt — keine neuen benutzersichtbaren Texte. (SDF-01, SDF-02)

**Plans**: 2 plans

Plans:

- [x] 36-01-PLAN.md — SDF-01 backend fix: create replaces an existing same-date special_day (atomic in-place update) instead of throwing Duplicate
- [x] 36-02-PLAN.md — SDF-02 frontend fix: controlled SelectInput `value` prop + Settings Card-3 sd_type binding so the Anlegen button re-enables after each create

### Phase 37: Modal-UX-Politur (FE)

**Goal**: Ein zentraler `dialog.rs`-Fix verhindert, dass ein innerhalb eines Modals begonnener und außerhalb losgelassener Maus-Drag das Modal schließt (kommt allen Modals zugute), und das Arbeitsvertrag-Modal trägt pro Feld (außer Von/Bis) einen Erklärungssatz in allen drei Locales.
**Depends on**: Nothing (rein Frontend; fachlich unabhängig von Phasen 36/38)
**Requirements**: MOD-01, MOD-02
**Success Criteria** (what must be TRUE):

  1. Ein innerhalb eines Modals begonnener Maus-Drag (Text-Selektion), der außerhalb losgelassen wird, schließt das Modal nicht — nur ein echter Außerhalb-Klick (mousedown UND mouseup auf dem Backdrop) schließt. (MOD-01)
  2. Der Fix ist zentral in `dialog.rs` umgesetzt (mousedown-Ursprung als Flag getrackt), sodass alle Modals profitieren; die Handler-/Prädikat-Logik ist strukturell getestet (Drag-innen→mouseup-außen lässt das Modal offen). (MOD-01)
  3. Das Arbeitsvertrag-Modal zeigt unter jedem relevanten Feld einen kurzen Erklärungssatz (Muster `CapPlannedHoursHelp`, `text-small text-ink-muted`), Von/Bis ausgenommen. (MOD-02)
  4. Die neuen `*Help`-Keys existieren in de/en/cs; ein SSR-Test bestätigt, dass die Help-Texte unter den Feldern gerendert werden. (MOD-02)

**Plans**: 2 plans (0/2 complete)

Plans:

- [ ] 37-01-PLAN.md — MOD-01: zentraler drag-safe Backdrop-Close in `dialog.rs` (Signal-Flag) + `absence_convert_modal.rs`-Duplikat inline mitgefixt (TDD)
- [ ] 37-02-PLAN.md — MOD-02: sechs `*Help`-i18n-Keys (de/en/cs) + Sibling-Help-Spans pro Feld im Arbeitsvertrag-Modal (Von/Bis ausgenommen)

**Cross-cutting constraints:**

- Frontend gates stay green: cargo test -p shifty-dioxus and cargo build --target wasm32-unknown-unknown pass; backend cargo clippy --workspace -- -D warnings stays green as a regression guard. (D-11)

### Phase 38: Frontend-Build-Hygiene

**Goal**: Der Frontend-Build (`shifty-dioxus`) ist warnungsfrei (~45–50 rustc-Warnings beseitigt), das Backend bleibt `cargo clippy --workspace -- -D warnings`-sauber (Regressions-Gate), und verbleibende bewusst behaltene dioxus-Lints sind dokumentiert. Keine Verhaltensänderung.
**Depends on**: Nothing (reiner Cleanup; bewusst zuletzt in der Reihenfolge)
**Requirements**: HYG-01, HYG-02
**Success Criteria** (what must be TRUE):

  1. `cargo build` (shifty-dioxus) läuft ohne Warnungen — die ~45–50 rustc-Warnings sind beseitigt (14 via `cargo fix`, Rest manuell: ungenutzte Methoden/Imports/Variablen entfernt oder mit begründetem `#[allow(dead_code)]` behalten). (HYG-01)
  2. `cargo clippy --workspace -- -D warnings` (Backend) bleibt grün als Regressions-Gate; der dioxus-Clippy-Lauf erfolgt bewusst aus der Backend-nix-Shell (E0514 im dioxus-Shell). (HYG-02)
  3. Verbleibende bewusst behaltene dioxus-Lints sind dokumentiert. (HYG-02)
  4. Frontend-Gates grün: `cargo test -p shifty-dioxus` und der WASM-Build (`cargo build --target wasm32-unknown-unknown`). (HYG-01, HYG-02)

**Plans**: 0/0 plans complete

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
  **Plans:** 2/2 plans complete

*Last updated: 2026-06-30 — **v1.10 geshipt + archiviert** (Phasen 33–35, 8 Pläne, 12/12 Requirements, Audit `passed`). Phase-Details nach [`milestones/v1.10-ROADMAP.md`](milestones/v1.10-ROADMAP.md) ausgelagert + im Milestones-Block collapsed; Progress-Zeilen 33/35 bleiben Complete (2026-06-30). Backlog 999.1 unverändert.*
