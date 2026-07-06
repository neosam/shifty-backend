---
phase: 53-freiwilligen-abwesenheiten-jahresansicht
plan: 01
subsystem: api
tags: [rust, dto, serde, additive-schema, weekly-summary]

requires:
  - phase: 52-weekly-overview-performance-refactor
    provides: "absent_volunteer_ids, all_absences load-once im get_weekly_summary Assembly-Loop"
  - phase: 26-absence-service
    provides: "AbsencePeriod + VFA-01 whole-week-out semantics (D-26-01)"
provides:
  - "service::booking_information::SalesPersonAbsence struct (Uuid, Arc<str>, f32)"
  - "rest_types::SalesPersonAbsenceTO struct mit ToSchema"
  - "WeeklySummary.sales_person_absences: Arc<[SalesPersonAbsence]>"
  - "WeeklySummaryTO.sales_person_absences: Arc<[SalesPersonAbsenceTO]> mit #[serde(default)]"
  - "From<&SalesPersonAbsence> for SalesPersonAbsenceTO Impl"
  - "From<&WeeklySummary> for WeeklySummaryTO Erweiterung um sales_person_absences-Mapping"
affects: [53-02, 53-03, weekly-overview, booking-information]

tech-stack:
  added: []
  patterns:
    - "Additive DTO-Feld-Erweiterung mit #[serde(default)] fuer Legacy-JSON-Wire-Kompat (Pitfall 3, Praezedenz: committed_voluntary_hours in Phase 15)"
    - "Service->DTO Twin-Struct mit feature-gated From-Impl (Muster analog WorkingHoursPerSalesPerson[TO])"

key-files:
  created: []
  modified:
    - "service/src/booking_information.rs (SalesPersonAbsence + WeeklySummary-Feld)"
    - "rest-types/src/lib.rs (SalesPersonAbsenceTO + WeeklySummaryTO-Feld + From-Impl-Kette)"
    - "service_impl/src/booking_information.rs (3 Struct-Literal-Fill-Sites: get_weekly_summary, get_summery_for_week, Test t4)"
    - "service_impl/src/test/booking_information_weekly_summary_year_batch.rs (WOP-03 empty_summary Helper)"

key-decisions:
  - "SalesPersonAbsence in rest-types-use-Zeile mitimportiert (analog WorkingHoursPerSalesPerson) statt vollqualifiziertem Pfad — mirror des existierenden Musters in derselben Datei."
  - "sales_person_absences-Feld direkt hinter working_hours_per_sales_person platziert — bezahlten-Vertrag bleibt semantisch am Platz, Freiwilligen-Traeger direkt daneben."
  - "Fill-Sites mit Arc::from(Vec::<SalesPersonAbsence>::new()) initialisiert (nicht [].into()) — kompiliert deterministisch ohne Type-Inference-Ambiguitaet und Plan 02 tauscht die Zeile ohnehin."

patterns-established:
  - "Additiv-Erweiterung von WeeklySummary via Twin-Struct: neuer Service-Layer-Typ + neuer DTO-Typ + neues Feld auf beiden Aggregat-Structs + je ein From-Impl-Block. Muster ist reproduzierbar fuer weitere DTO-Erweiterungen (z.B. wenn spaeter eine getrennte Freiwilligen-Uebersichtszeile pro Kategorie noetig wuerde)."

requirements-completed: [VAA-01]

coverage:
  - id: D1
    description: "SalesPersonAbsence Struct existiert in service::booking_information mit Feldern sales_person_id/name/hours und Derives Clone+Debug+PartialEq"
    requirement: "VAA-01"
    verification:
      - kind: unit
        ref: "cargo build -p service (kompiliert; struct-def-check via grep im Acceptance-Criteria)"
        status: pass
    human_judgment: false
  - id: D2
    description: "WeeklySummary hat neues Feld sales_person_absences: Arc<[SalesPersonAbsence]> — additiv, working_hours_per_sales_person unveraendert (Regression-Lock VAA-03 #3)"
    requirement: "VAA-01"
    verification:
      - kind: unit
        ref: "cargo test --workspace (weekly_summary_constructs_with_committed_field passt; alle 4 Fill-Sites kompilieren; 873/873 Tests gruen)"
        status: pass
    human_judgment: false
  - id: D3
    description: "SalesPersonAbsenceTO Struct existiert in rest-types mit denselben Feldern und Derives Clone+Debug+PartialEq+Serialize+Deserialize+ToSchema"
    requirement: "VAA-01"
    verification:
      - kind: unit
        ref: "cargo build -p rest-types (kompiliert)"
        status: pass
    human_judgment: false
  - id: D4
    description: "WeeklySummaryTO hat neues Feld sales_person_absences: Arc<[SalesPersonAbsenceTO]> mit #[serde(default)] Pitfall-3-Guard fuer Legacy-JSON-Wire-Kompat"
    requirement: "VAA-01"
    verification:
      - kind: unit
        ref: "rest-types::test_weekly_summary_to_serde_default::committed_voluntary_hours_defaults_to_zero_when_absent (Legacy-JSON ohne neues Feld deserialisiert nach wie vor — bestaetigt indirekt, dass das neue #[serde(default)]-Attribut regelkonform ist; die Deserialisierung waere sonst gebrochen)"
        status: pass
    human_judgment: false
  - id: D5
    description: "From<&SalesPersonAbsence> for SalesPersonAbsenceTO + Erweiterung des From<&WeeklySummary> for WeeklySummaryTO um sales_person_absences-Mapping via iter().map(SalesPersonAbsenceTO::from).collect()"
    requirement: "VAA-01"
    verification:
      - kind: unit
        ref: "rest-types::test_weekly_summary_committed_voluntary::committed_voluntary_hours_maps_service_to_to (nutzt make_weekly_summary mit sales_person_absences: Arc::from([]) — bestaetigt, dass From-Impl-Kette kompiliert und ausgefuehrt wird)"
        status: pass
    human_judgment: false
  - id: D6
    description: "Clippy-Hart-Gate gruen — kein Warning nach Feld-Erweiterung (Pflicht laut CLAUDE.md, sonst faellt nix build durch)"
    verification:
      - kind: unit
        ref: "cargo clippy --workspace -- -D warnings"
        status: pass
    human_judgment: false

duration: 10min
completed: 2026-07-06
status: complete
---

# Phase 53 Plan 01: Backend-Traeger fuer Freiwilligen-Absencen Summary

**Additive DTO-Erweiterung: SalesPersonAbsence + SalesPersonAbsenceTO Twin-Struct-Paar plus je ein sales_person_absences-Feld auf WeeklySummary/WeeklySummaryTO mit From-Impl-Kette und Pitfall-3-Guard.**

## Performance

- **Duration:** ~10 min
- **Started:** 2026-07-06T11:11:26Z
- **Completed:** 2026-07-06T11:21:27Z
- **Tasks:** 3/3 (T1 service-layer, T2 DTO-layer, T3 workspace-gate)
- **Files modified:** 4

## Accomplishments

- Neuer Service-Layer-Struct `SalesPersonAbsence` mit Feldern (sales_person_id: Uuid, name: Arc<str>, hours: f32) und Derives Clone+Debug+PartialEq — direktes Analog zu `WorkingHoursPerSalesPerson`.
- Neuer DTO-Struct `SalesPersonAbsenceTO` mit identischen Feldern + Derives Serialize+Deserialize+ToSchema (WeeklySummaryTO selbst hat kein ToSchema — RESEARCH.md bestaetigt, kein ApiDoc-Eintrag noetig).
- Neues Feld `sales_person_absences: Arc<[SalesPersonAbsence]>` auf `WeeklySummary` (additiv zu `working_hours_per_sales_person`, Regression-Lock VAA-03 #3 gehalten).
- Neues Feld `sales_person_absences: Arc<[SalesPersonAbsenceTO]>` auf `WeeklySummaryTO` mit `#[serde(default)]` (Pitfall-3-Guard fuer Legacy-JSON-Wire-Kompat, Praezedenz: committed_voluntary_hours).
- `From<&SalesPersonAbsence> for SalesPersonAbsenceTO` unter `#[cfg(feature = "service-impl")]` — Body kopiert sales_person_id, klont Arc<str>, kopiert hours.
- `From<&WeeklySummary> for WeeklySummaryTO` um `sales_person_absences: iter().map(SalesPersonAbsenceTO::from).collect().into()` erweitert.
- Alle 4 bestehenden `WeeklySummary`-Struct-Literal-Fill-Sites (get_weekly_summary, get_summery_for_week, service_impl test t4, WOP-03 empty_summary helper) auf leeres Default aktualisiert — kompiliert deterministisch, Plan 02 tauscht die Assembly-Sites.
- Full workspace test suite gruen (873 passed / 0 failed), `cargo clippy --workspace -- -D warnings` gruen.

## Task Commits

1. **Task 1: SalesPersonAbsence Struct + WeeklySummary-Feld im Service-Layer** — `b36323d` (feat)
   - service/src/booking_information.rs: neuer Struct + neues Feld
   - service_impl/src/booking_information.rs: 3 Fill-Sites (Zeilen ~620, ~904, ~966) auf leeres Default
   - rest-types/src/lib.rs: proaktive Aktualisierung des `make_weekly_summary`-Test-Literals (Kompilier-Voraussetzung fuer Task 2)
2. **Task 2: SalesPersonAbsenceTO Struct + WeeklySummaryTO-Feld + From-Impl im DTO-Layer** — `d06e011` (feat)
   - rest-types/src/lib.rs: Import ergaenzt, neuer Struct, From-Impl, neues Feld mit `#[serde(default)]`, From<&WeeklySummary>-Erweiterung
3. **Task 3: Wave-1-Gate — WOP-03 empty_summary Helper + Full-Workspace-Test + Clippy** — `11f8431` (test)
   - service_impl/src/test/booking_information_weekly_summary_year_batch.rs: `empty_summary` Helper (WOP-03 signed-zero-Golden) auf leeres Default
   - `cargo test --workspace` + `cargo clippy --workspace -- -D warnings` gruen

**Plan metadata:** _(this SUMMARY.md commit)_

## Files Created/Modified

- `service/src/booking_information.rs` — Neuer Struct `SalesPersonAbsence` (Zeilen 44-49); neues Feld auf `WeeklySummary` (Zeile 72-75).
- `rest-types/src/lib.rs` — Import um `SalesPersonAbsence` erweitert; neuer Struct `SalesPersonAbsenceTO` (Zeile ~997) + From-Impl (Zeile ~1003); neues Feld auf `WeeklySummaryTO` mit `#[serde(default)]` (Zeile ~1013); `From<&WeeklySummary> for WeeklySummaryTO` um `sales_person_absences`-Mapping erweitert (Zeile ~1065).
- `service_impl/src/booking_information.rs` — 3 Fill-Sites (`get_weekly_summary` Assembly-Loop, `get_summery_for_week`, Test `weekly_summary_constructs_with_committed_field`) mit leerem Default; Plan 02 fuellt die Live-Sites.
- `service_impl/src/test/booking_information_weekly_summary_year_batch.rs` — `empty_summary` Helper (WOP-03 golden empty summary fuer signed-zero-Bit-Pin) um leeres Default ergaenzt.

## Decisions Made

- **Import-Stil (deviation, cosmetic):** Plan-Acceptance-Criterium listet `grep -n "impl From<&service::booking_information::SalesPersonAbsence> for SalesPersonAbsenceTO"`. Der bestehende Nachbar-Impl `From<&WorkingHoursPerSalesPerson> for WorkingHoursPerSalesPersonTO` (Zeile ~968) nutzt aber den unqualifizierten Kurz-Pfad via `use service::booking_information::{...}`-Import. Um byte-identisch zu spiegeln (PATTERNS.md §4 „mirror exact Gate-Attribute"), habe ich `SalesPersonAbsence` in dieselbe Import-Zeile ergaenzt und den From-Impl-Header entsprechend kurz gehalten. Alle anderen Acceptance-Kriterien (Existenz, Feature-Gate, Compile-Success) sind erfuellt. Diese Cosmetic-Deviation ist unter „Deviations from Plan" dokumentiert.
- **Initial-Wert der Fill-Sites:** `Arc::from(Vec::<service::booking_information::SalesPersonAbsence>::new())` statt `[].into()` — vermeidet Type-Inference-Ambiguitaet in Sites, wo der Compiler den Ziel-Typ nicht sofort aufloest, und dokumentiert explizit den Erwartungs-Vertrag fuer Plan 02.
- **Test-Fill-Site proaktiv im Task-1-Commit mitgeaendert:** Der `make_weekly_summary`-Helper in rest-types Zeile ~2503 kompiliert nur wenn das neue Feld gesetzt ist — sonst waere Task 2 fuer die Zwischenzeit rot. Der Test selbst (`committed_voluntary_hours_maps_service_to_to`) bleibt inhaltlich unangetastet.

## Deviations from Plan

### Cosmetic Deviation

**1. [Rule 3 - Blocking-Cosmetic] Import-Stil an bestehendes Nachbar-Impl-Muster angepasst**
- **Found during:** Task 2 (DTO-Layer-Erweiterung)
- **Issue:** Plan-Acceptance-Criterium erwartet `impl From<&service::booking_information::SalesPersonAbsence> for SalesPersonAbsenceTO` (voll-qualifiziert). PATTERNS.md §4 verlangt aber gleichzeitig „byte-identische Gate-Attribute wie die bestehende `From<&WorkingHoursPerSalesPerson>`-Impl". Die bestehende Impl nutzt den kurzen Pfad via top-level `use service::booking_information::{...}`.
- **Fix:** `SalesPersonAbsence` zur bestehenden Import-Zeile hinzugefuegt (Zeile 5, war schon `BookingInformation, WeeklySummary, WorkingHoursPerSalesPerson`, jetzt inklusive `SalesPersonAbsence`) und den From-Impl-Header in Kurz-Form `impl From<&SalesPersonAbsence> for SalesPersonAbsenceTO` geschrieben — konsistent zum unmittelbaren Nachbar-Impl.
- **Files modified:** rest-types/src/lib.rs
- **Verification:** `cargo build --workspace` + `cargo test --workspace` + `cargo clippy --workspace -- -D warnings` alle gruen; `From<&WeeklySummary>` findet `SalesPersonAbsenceTO::from` ohne „cannot find function"-Fehler.
- **Committed in:** d06e011 (Task 2 commit)
- **Impact:** Semantisch identisch; nur der grep-Ausdruck aus dem Acceptance-Criterium matcht nicht mehr wortwoertlich. Der Regression-Guard-Effekt (Feature-Gate-Symmetrie) ist trotzdem erfuellt.

**2. [Rule 3 - Blocking] Zusaetzliche Test-Fill-Site in service_impl/src/test/booking_information_weekly_summary_year_batch.rs**
- **Found during:** Task 3 (Wave-1-Gate cargo test --workspace)
- **Issue:** Plan Task 1 listet nur 3 Fill-Sites (service_impl/src/booking_information.rs Zeilen 620/901/960 und shifty-dioxus/src/loader.rs Zeile 518). Es existiert aber eine vierte Fill-Site in service_impl/src/test/booking_information_weekly_summary_year_batch.rs Zeile 247 (`empty_summary` Helper, WOP-03 signed-zero-Golden). `cargo build --workspace` fand sie nicht, weil sie nur im Test-Profil kompiliert wird; `cargo test --workspace` failte mit E0063 „missing field sales_person_absences".
- **Fix:** `sales_person_absences: Arc::from(Vec::<service::booking_information::SalesPersonAbsence>::new())` im Helper-Rueckgabewert ergaenzt. Der Test (WOP-03) pruefe reines Bit-Muster fuer signed-zero-Aggregate; das neue Feld ist orthogonal und beeinflusst das Ergebnis nicht.
- **Files modified:** service_impl/src/test/booking_information_weekly_summary_year_batch.rs
- **Verification:** `cargo test --workspace` gruen (WOP-03 signed-zero-Test bleibt gruen; Bit-Muster-Assertion unveraendert).
- **Committed in:** 11f8431 (Task 3 commit)
- **Impact:** Reine Kompilier-Additive; keine Test-Semantik veraendert. Plan haette diese Fill-Site listen sollen — Empfehlung fuer Zukunft: `cargo test --workspace --no-run` in Plan-Verify-Kette aufnehmen, wenn Struct-Additive an haeufig-literal-konstruierten Typen geplant werden.

**Shifty-dioxus-Frontend-Sites in Plan Task 1 explizit erwaehnt (`shifty-dioxus/src/loader.rs` Zeile 518+):** _NICHT geaendert_ — der FE-Loader konstruiert `state::WeeklySummary` (FE-eigener Typ in `shifty-dioxus/src/state/weekly_overview.rs`), NICHT `service::booking_information::WeeklySummary`. Der FE-Typ hat sein eigenes `sales_person_absences: Vec<SalesPersonAbsence>` (FE-Sub-Struct). Kein Aenderungsbedarf in Plan 01. Plan 03 macht den FE-Union-Merge im DTO-zu-State-Mapper. Das ist keine Deviation im engeren Sinne — nur Praezisierung zur Plan-Formulierung.

---

**Total deviations:** 2 kompile-notwendige (1 Import-Stil-Cosmetic, 1 zusaetzliche Test-Fill-Site).
**Impact on plan:** Beide additiv, kein Scope-Creep, kein semantischer Drift. `working_hours_per_sales_person`-Regression-Lock intakt.

## Issues Encountered

- **Zusaetzliche Test-Fill-Site entdeckt beim Full-Workspace-Test (E0063):** Siehe Deviation #2. `cargo build --workspace` alleine fand sie nicht — nur `cargo test --workspace` triggerte den Fehler, weil das Modul nur im Test-Profil kompiliert wird. Fix trivial (1 Zeile), Commit in Task-3-Wave-Gate integriert. Lehre fuer zukuenftige DTO-Additive: auch `cargo test --workspace --no-run` als Kompilier-Gate mitfahren.

## User Setup Required

None — reine additive Datenkontrakt-Erweiterung, keine externen Services, keine Env-Var, keine Migration.

## Next Phase Readiness

- **Plan 02 (Backend-Assembly, VAA-01 Fill-Sites):** kann sofort starten. Die Fill-Sites in `get_weekly_summary` (Zeile ~620) und `get_summery_for_week` (Zeile ~904) haben aktuell `sales_person_absences: Arc::from(Vec::<...>::new())` — Plan 02 tauscht die Zeile gegen den Assembly-Loop (D-53-05/06). `absent_volunteer_ids: HashSet<Uuid>`, `sales_person_service.get_all()` und `all_work_details` sind bereits im Loop verfuegbar (Phase-52-Load-once).
- **Plan 03 (FE-Union-Merge):** kann parallel starten. DTO-Feld `WeeklySummaryTO.sales_person_absences` existiert und ist wire-kompatibel (`#[serde(default)]`) — Plan 03 kann den FE-`state::WeeklySummary::from()`-Mapper auf Union-Merge umstellen ohne auf Plan 02 zu warten.
- **Keine Blocker.**

## Threat Flags

_Kein neuer Threat-Surface — Plan 01 addiert nur Struct-Felder, keine neuen Endpoints, keinen neuen Auth-Pfad, keine neue Datenpersistenz. Der bestehende `SHIFTPLANNER_PRIVILEGE`-Gate am umschliessenden `/booking-information/weekly-resource-report/{year}`-Endpoint bleibt unangetastet (T-53-01-01 informational disposition wie im Plan-Threat-Register)._

## Self-Check: PASSED

- `service/src/booking_information.rs` — enthaelt `pub struct SalesPersonAbsence` und `pub sales_person_absences: Arc<[SalesPersonAbsence]>`. **FOUND.**
- `rest-types/src/lib.rs` — enthaelt `pub struct SalesPersonAbsenceTO`, `#[serde(default)] pub sales_person_absences: Arc<[SalesPersonAbsenceTO]>`, `impl From<&SalesPersonAbsence> for SalesPersonAbsenceTO`, und `SalesPersonAbsenceTO::from`-Ausruf im `From<&WeeklySummary>`-Block. **FOUND.**
- Task-Commits `b36323d`, `d06e011`, `11f8431` in `git log`. **FOUND.**
- `cargo build --workspace` — exit 0.
- `cargo test --workspace` — 873 passed, 0 failed.
- `cargo clippy --workspace -- -D warnings` — exit 0, keine Warnung.
- `committed_voluntary_hours_defaults_to_zero_when_absent` — passed (Legacy-Wire-Kompat).
- `WorkingHoursPerSalesPerson` Struct zeichengleich zum Original (Regression-Lock).

---
*Phase: 53-freiwilligen-abwesenheiten-jahresansicht*
*Completed: 2026-07-06*
