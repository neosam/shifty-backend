# Phase 28: Urlaubsanspruch-Korrektur via Offset (HR, BE+FE) - Context

**Gathered:** 2026-06-29
**Status:** Ready for planning
**Mode:** Smart-Discuss (autonomous) — Seeds + ROADMAP + 2 Grey-Areas geklärt (API-Hiding, Off-by-one+Snapshot)

<domain>
## Phase Boundary

HR kann den berechneten Jahres-Urlaubsanspruch einer Person um einen **signed Offset
(Korrektur-Delta)** anheben/senken (z.B. berechnet 17 → +1 → effektiv 18). Der Offset ist
ein **Delta** (KEIN absoluter Override) → überlebt Vertragsänderungen. In der HR-Ansicht
gekennzeichnet + inline editierbar; für normale User unsichtbar (nur Effektivwert).

**Zwei Lieferungen (vom User in der Discuss zusammengelegt):**
1. **Offset-Mechanismus** (snapshot-SAFE): neue Tabelle + DAO + Service + HR-gated REST +
   Offset-Addition in `vacation_balance` + FE-Inline-Editor + API-level Hiding.
2. **Off-by-one-Begleitfix** (snapshot-RELEVANT): `vacation_days_for_year` Proration-Fehler
   (`employee_work_details.rs:173`, `ordinal()` statt `ordinal()-1`) korrigieren — fließt auch
   in `reporting.rs:803` (Billing-Snapshot `VacationDays`) → **erzwingt einen
   `CURRENT_SNAPSHOT_SCHEMA_VERSION`-Bump (11→12)** + Regressionstests.

**Requirements:** VAC-OFFSET-01.

**Liefert NICHT:**
- Kein absoluter Override (bewusst Delta, D-28-01).
- Keine Offset-Anzeige/-Edit für normale User (API-level versteckt, D-28-03).
- Keine Mehrjahres-Bulk-Edit-UI — Offset ist jahresbezogen, editiert im in der Übersicht
  gewählten Jahr (D-28-09).
</domain>

<decisions>
## Implementation Decisions

### Mechanismus & Datenmodell
- **D-28-01 (Signed Offset/Delta, nicht Override):** Pro **Person + Jahr** ein signed
  `offset_days` (i32, ±). Neue Tabelle `vacation_entitlement_offset`
  (`id`, `sales_person_id`, `year`, `offset_days` INTEGER signed, `version`, `created`,
  `deleted` — Soft-Delete-Konvention). DAO-Trait (`dao/`) + `dao_impl_sqlite` + additive
  Migration (`migrations/sqlite/`, **`sqlx migrate run`, NICHT reset**).
- **D-28-06 (Service-Tier):** Neuer **Basic Service** `VacationEntitlementOffsetService`
  (Entity-Manager: CRUD + HR-Gate, konsumiert nur DAO/Permission/Transaction). Die
  **Business-Logic** `VacationBalanceService` (konsumiert bereits `CarryoverService`)
  konsumiert zusätzlich den Offset-Service (kein Zyklus — Offset-Service kennt
  VacationBalanceService nicht). DI in `shifty_bin/src/main.rs`: erst Basic, dann Business-Logic.

### Berechnung
- **D-28-02 (Offset nach .round() addieren):** In `service_impl/src/vacation_balance.rs`
  (`entitled_days` wird bei `:186-191` als `sum(vacation_days_for_year).round()` gebildet):
  `entitled_effective = round(sum) + offset_days`. Wirkt automatisch auf `remaining_days`
  durch (`:256-257`: `entitled_days + carryover − (used+planned)`).

### Sichtbarkeit (Grey-Area, entschieden 2026-06-29)
- **D-28-03 (API-level Hiding, serverseitig):** Der Self-Endpoint ist „HR ∨ self". Der Offset
  darf NICHT in der rohen Antwort für reine Self-Caller landen. Lösung: Service liefert
  `offset_days` + `computed_entitled_days` (der Vor-Offset-Wert `round(sum)`) **nur, wenn der
  Caller HR-Privileg hat**; für reine Self-Caller sind beide `None`. `entitled_days` ist
  IMMER der Effektivwert (`round(sum)+offset`) — beide Rollen sehen die korrekte Endzahl.
  → `VacationBalanceTO` (rest-types/src/lib.rs:2042) + Domain `VacationBalance` um
  `offset_days: Option<i32>` + `computed_entitled_days: Option<f32>` erweitern; beide From-Impls
  (`:2053`, `:2068`) anpassen.

### Off-by-one-Begleitfix (Grey-Area, entschieden 2026-06-29 — „Mitnehmen + Bump")
- **D-28-04 (Proration-Fix):** `EmployeeWorkDetails::vacation_days_for_year`
  (`service/src/employee_work_details.rs:158-176`) zieht bei unterjährigem Start
  `vacation_days * ordinal()/days_in_year` ab; für Start am 1.1. (`ordinal()==1`) wird ~0,05
  Tag zu viel abgezogen. Fix: `(ordinal()-1)` als abzuziehender Vortagesanteil (Start am 1.1.
  → 0 Abzug). Analog ggf. das `to_year`/Jahresende-Pendant prüfen. Regressionstests:
  Volljahr (1.1.–31.12. → voller `vacation_days`), Teiljahr-Start, Teiljahr-Ende.
- **D-28-05 (Snapshot-Bump 11→12 ERZWUNGEN):** `vacation_days_for_year` wird AUCH von
  `service_impl/src/reporting.rs:803` genutzt → fließt in den persistierten Billing-Snapshot
  (`BillingPeriodValueType::VacationDays`). Der Off-by-one-Fix ändert diese Computation →
  **`CURRENT_SNAPSHOT_SCHEMA_VERSION` MUSS gebumpt werden** (aktuell 11 →  12), per
  CLAUDE.md-Regel („Change the computation that produces an existing value_type"). Konstante in
  `service_impl/src/billing_period_report.rs` (Phase 26 verifiziert: `= 11`). Snapshot-Versions-
  Guard-Test anpassen. **Hinweis:** Der Offset-Mechanismus selbst ist snapshot-SAFE (berührt nur
  `vacation_balance.rs`, NICHT `reporting.rs`); der Bump wird allein durch den Off-by-one-Fix
  ausgelöst.

### HR-Gate
- **D-28-06b (HR-gated CRUD):** Setzen/Ändern/Löschen des Offsets erfordert `HR_PRIVILEGE`
  (analog bestehender HR-Gates). Lesen des Breakdowns ebenfalls HR (siehe D-28-03).

### Frontend
- **D-28-07 (Inline-Zahlenfeld in der Detail-StatBox):** Platzierung in
  `VacationEntitlementSelfBody` (`page/absences.rs:407-481`), an der „Vertragsanspruch"-StatBox
  (`VacationStatContract`, Wert = `entitled_days`). HR erreicht die Detailansicht per Klick auf
  eine Person in `VacationPerPersonList` (`forced_self`-Pfad, `:358-363`). **HR-Ansicht:**
  immer sichtbares, signed Inline-Zahlenfeld; Anzeige „berechnet {computed} + Offset [x]",
  Box-/Hero-Zahl = Effektivwert; Speichern on-blur/Enter (HR-gated). **User-Ansicht:** dieselbe
  StatBox zeigt NUR den Effektivwert (kein Feld, keine „berechnet/Offset"-Zeile).
  `VacationEntitlementSelfBody` bekommt dafür ein durchgereichtes **`is_hr`-Flag** (vorhanden in
  `VacationEntitlementCard` via `props.is_hr`).
- **D-28-08 (i18n de/en/cs):** Neue Labels („berechnet", „Offset", ggf. „Korrektur") in allen
  drei Locales.
- **D-28-09 (Jahresbezug):** Offset-Edit immer im Kontext des in der Übersicht gewählten Jahres.

### Claude's Discretion
- Exakte Migrations-Spalten/Indizes; Unique-Constraint auf (`sales_person_id`, `year`) bei
  `deleted IS NULL`.
- REST-Routenform: dediziertes `vacation_entitlement_offset`-CRUD vs. erweitern um einen
  HR-gated PUT am vacation-balance-Pfad. Empfehlung: eigener kleiner CRUD-Endpoint (sauberes
  Aggregat), GET-Breakdown via vacation_balance (D-28-03).
- Genaue Signatur, wie der Offset-Service-Read in `VacationBalanceService::get` injiziert wird
  (ein Read pro (person, year)).
- Optionaler kleiner Offset-Indikator an Personen in `VacationPerPersonList`
  (`PersonVacationCard`, `:672-727`) — editiert wird nur im Detail (optional/deferierbar).
- cs-Übersetzungen.
- Exakte Off-by-one-Korrekturform (`(ordinal-1)` vs. `month`-basiert) + ob das Jahresende
  symmetrisch korrigiert werden muss.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirement & Seed
- `.planning/ROADMAP.md` § "Phase 28" — Goal + 6 Success Criteria + Konzept-Eckpunkte.
- `.planning/phases/28-urlaubsanspruch-korrektur-offset/SEED.md` — verifizierter Status quo.
- `shifty-backend/CLAUDE.md` § "Billing Period Snapshot Schema Versioning" (Bump-Pflicht) +
  § "Service-Tier-Konventionen" (Basic vs Business-Logic).

### Backend — Berechnung
- `service_impl/src/vacation_balance.rs:185-191` — `entitled_days = sum(vacation_days_for_year).round()`
  (Offset HIER addieren, D-28-02); `:247-257` — carryover + `remaining_days`; `:262-266` — Return-Struct
  `VacationBalance` (um `offset_days`/`computed_entitled_days` erweitern, D-28-03).
- `service/src/employee_work_details.rs:158-176` — `vacation_days_for_year` (Off-by-one bei `:173`, D-28-04).
- `service_impl/src/reporting.rs:803` — ZWEITE Nutzung von `vacation_days_for_year`
  (Billing-Snapshot `VacationDays` → Bump-Auslöser, D-28-05).
- `service_impl/src/billing_period_report.rs` — `CURRENT_SNAPSHOT_SCHEMA_VERSION` (=11 → 12)
  + Versions-History-Doku (v5/v6/v7/v9 zeigen Vacation-Computation-Bumps).
- `service/src/billing_period.rs:34` — `enum BillingPeriodValueType` (`VacationDays`, `VacationHours`).

### Backend — Datenmodell/Service/REST (Vorbilder)
- `service/src/vacation_balance.rs` (Trait) + `service_impl/src/vacation_balance.rs` (Impl,
  `gen_service_impl!` mit `CarryoverService`-Dep — Muster für Offset-Dep).
- `rest/src/vacation_balance.rs:36-119` — `get_vacation_balance` (`:59`, „HR ∨ self" via
  `context.into()`), `get_team_vacation_balance` (`:92`, HR-Aggregat), ApiDoc (`:115`).
- `rest-types/src/lib.rs:2042-2080` — `VacationBalanceTO` + beide `From`-Impls (erweitern, D-28-03).
- Bestehendes Aggregat als CRUD-Vorbild (DAO-Trait + sqlite-Impl + Migration + Service + REST):
  z.B. `dao/src/special_day.rs` / `dao_impl_sqlite` / `migrations/sqlite/` / `service*/special_day.rs`
  (Soft-Delete + version + `#[utoipa::path]` + ToSchema). Planner: nächstgelegenes kleines
  Aggregat per Pattern-Mapper bestätigen.
- `shifty_bin/src/main.rs` — DI-Reihenfolge (Basic vor Business-Logic); Offset-Service vor
  VacationBalanceService konstruieren.

### Frontend
- `shifty-dioxus/src/page/absences.rs:351-481` — `VacationEntitlementCard` (`props.is_hr`),
  `VacationEntitlementSelfBody` (Inline-Editor + „berechnet/Offset"-Zeile, D-28-07),
  `forced_self`-Pfad (`:358-363`).
- `:494-588` — `VacationEntitlementHrBody` + `VacationStatContract` (die „Vertragsanspruch"-StatBox).
- `:589-727` — `VacationPerPersonList` + `PersonVacationCard` (optionaler Offset-Indikator).
- `shifty-dioxus/src/state/vacation_balance.rs` — `VacationBalance` FE-State (um offset/computed erweitern).
- `shifty-dioxus/src/api.rs` — vacation-balance Load + neuer HR-gated Offset-Save-Call.
- `shifty-dioxus/src/i18n/mod.rs` + `en.rs`/`de.rs`/`cs.rs` — neue Labels (D-28-08).
- `shifty-dioxus/src/component/form/inputs.rs` — Number/Text-Input-Muster (on-blur/Enter).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`VacationBalanceService::get(sales_person_id, year, context, tx)`** kennt den Auth-Context →
  HR-vs-self-Entscheidung für D-28-03 dort treffen.
- **`gen_service_impl!`-DI** + bestehende Carryover-Dep im VacationBalanceService = Muster für die
  Offset-Service-Dep.
- **Bestehendes kleines Soft-Delete-Aggregat** (special_day o.ä.) = 1:1-Vorbild für Tabelle+DAO+Service+REST.
- **`props.is_hr`** existiert bereits in `VacationEntitlementCard` → nur nach `SelfBody` durchreichen.

### Established Patterns
- **Snapshot-Versionierung:** Bump bei geänderter `value_type`-Computation (CLAUDE.md). Off-by-one
  ändert `VacationDays` → Bump 11→12 + Guard-Test.
- **Service-Tier:** Offset-Service = Basic; VacationBalanceService = Business-Logic (darf Offset-Service
  konsumieren, kein Zyklus).
- **Transactions:** alle Service-Methoden `Option<Transaction>`; `use_transaction`/`commit`.
- **REST:** `#[utoipa::path]` + ToSchema + `error_handler`; HR-Gate via Permission-Service.
- **Migration additiv** (`sqlx migrate run`), NICHT `reset` (DESTRUCTIVE; User-Confirmation-Pflicht).
- **i18n de/en/cs**; de.rs MUSS `Locale::De` nutzen.

### Integration Points
- `vacation_balance.rs` Offset-Read + Addition; `reporting.rs:803` Off-by-one-Fix; Snapshot-Bump;
  `VacationBalanceTO`/Domain um Felder; REST-CRUD HR-gated; FE Inline-Editor + Save + i18n.

</code_context>

<specifics>
## Specific Ideas

- **Offset-Calc-Test:** Person, berechnet 17, Offset +1 → `entitled_days==18`, `remaining_days`
  entsprechend +1; Offset −2 → 15. Reload → Offset persistiert.
- **Delta-Verhalten-Test:** Offset +1 gesetzt; Vertrag ändert berechneten Wert 17→20 → effektiv 21
  (Offset bleibt, nicht eingefroren).
- **API-Hiding-Test:** Self-Caller (kein HR) → `offset_days==None` & `computed_entitled_days==None`,
  `entitled_days`==Effektivwert; HR-Caller → beide `Some`.
- **HR-Gate-Test:** Nicht-HR-Setzen → Forbidden/Permission-Error.
- **Off-by-one-Regression:** Vertrag 1.1.–31.12. → `vacation_days_for_year == vacation_days` (kein
  Abzug); unterjähriger Start → korrekter Bruchteil. Beide vor/nach Fix gegenübergestellt.
- **Snapshot-Bump-Guard:** Test, der `CURRENT_SNAPSHOT_SCHEMA_VERSION == 12` festnagelt + Begründung.
- **FE:** HR-Detail zeigt „berechnet 17 + Offset [1]" → Box 18; User-Detail zeigt nur 18 (kein Feld).

</specifics>

<deferred>
## Deferred Ideas

- **Offset-Indikator in `VacationPerPersonList`** (kompakt) — optional; editiert wird nur im Detail.
- **Absoluter Override-Modus** — bewusst verworfen (D-28-01, Delta gewählt).
- **Bulk-/Mehrjahres-Offset-Edit** — außer Scope (jahresbezogen, D-28-09).
- **Jahresende-Symmetrie der Proration** — falls der Off-by-one nur den Jahresanfang betrifft, ist
  das Jahresende separat zu prüfen (Planner entscheidet; sonst Deferred).

### Reviewed Todos (not folded)
None.

</deferred>

---

*Phase: 28-Urlaubsanspruch-Korrektur via Offset (HR, BE+FE)*
*Context gathered: 2026-06-29*
