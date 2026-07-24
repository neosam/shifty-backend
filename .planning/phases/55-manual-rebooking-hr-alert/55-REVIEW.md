---
phase: 55-manual-rebooking-hr-alert
reviewed: 2026-07-24T00:00:00Z
depth: standard
files_reviewed: 20
files_reviewed_list:
  - dao/src/rebooking_batch.rs
  - dao_impl_sqlite/src/rebooking_batch.rs
  - service/src/rebooking_batch.rs
  - service/src/rebooking_reconciliation.rs
  - service_impl/src/rebooking_batch.rs
  - service_impl/src/rebooking_reconciliation.rs
  - service_impl/src/reporting.rs
  - rest/src/rebooking.rs
  - rest-types/src/rebooking.rs
  - service_impl/src/test/rebooking_reconciliation.rs
  - service_impl/src/test/reporting_pending_rebooking.rs
  - shifty-dioxus/src/api.rs
  - shifty-dioxus/src/loader.rs
  - shifty-dioxus/src/state/rebooking.rs
  - shifty-dioxus/src/state/employee.rs
  - shifty-dioxus/src/component/rebooking_alert_banner.rs
  - shifty-dioxus/src/component/rebooking_suggestion_modal.rs
  - shifty-dioxus/src/component/manual_rebooking_modal.rs
  - shifty-dioxus/src/page/employees.rs
  - shifty-dioxus/src/page/employee_details.rs
findings:
  critical: 2
  warning: 5
  info: 4
  total: 11
status: issues_found
---

# Phase 55: Code-Review-Bericht (Manual Rebooking + HR-Alert)

**Geprüft:** 2026-07-24
**Tiefe:** standard
**Dateien:** 20
**Status:** issues_found

## Zusammenfassung

Geprüft wurde der Phase-55-Rebooking-/HR-Alert-Stack über alle Schichten (DAO,
Service, REST, DTO, Dioxus-Frontend). Die vier Kern-Invarianten, die im
Review-Auftrag genannt wurden, halten größtenteils:

- **Race-Safe State-Transition (HR-ALERT-03):** korrekt. `update_state_conditional`
  nutzt `WHERE id=? AND state='pending' AND deleted IS NULL` mit
  `rows_affected()!=1 → DaoError::EntityAlreadyExists → 409`. Kein Read-then-write.
- **VOL-ACCT-03 Neutralität:** korrekt. Alle vier Aggregations-Sites in
  `reporting.rs` (Z. 859, 1192, 1728, 2161) filtern `source == ExtraHoursSource::Manual`,
  inkl. der internen `hours_per_week`-Aggregation (Z. 2161). Kein vergessener Site gefunden.
- **Fat Backend / Thin Client:** die FE-State-Mapper (`state/rebooking.rs`,
  `state/employee.rs`) sind reine Feld-Kopien, keine FE-Arithmetik. Gut.
- **Single Pair-write-path + Tx-Propagation:** `rebook(...)` ist der einzige
  Pair-Erzeuger; alle Cross-Service-Calls reichen `Some(tx.clone())` durch,
  commit erst am Ende. Gut.

Es bleiben aber zwei blocker-würdige Korrektheits-/Datenintegritäts-Defekte,
die weder Tests noch Clippy fangen (beide betreffen die F5-Approve-Semantik),
plus mehrere Warnungen zu Eingabe-Validierung und einer semantischen
Inkonsistenz zwischen Banner-Predicate und Suggest-Vorschau.

## Narrative Findings (AI reviewer)

## Critical Issues

### CR-01: F5-Approve verliert den Audit-Trail — kein Batch-Entry mit den Pair-ids

**File:** `service_impl/src/rebooking_reconciliation.rs:291-339`
**Issue:**
Der F3-Manual-Pfad (`rebook_manual`) schreibt nach dem Pair einen
`RebookingBatchEntryEntity` mit `extra_hours_out_id`/`extra_hours_in_id` (Z. 176-192),
damit die Audit-Kette Batch → Entry → ExtraHours vollständig ist.

Der F5-Approve-Pfad tut das **nicht**. `create_pending_suggestion`
(`service_impl/src/rebooking_batch.rs:205-257`) legt nur den Batch an (INSERT ohne
Entry). `approve` transitioniert den Batch via `update_state` und ruft danach
`rebook(...)` — aber **niemals** `create_batch_with_entries` oder ein Entry-Update.
Ergebnis: jeder per HR-Alert genehmigte Rebooking-Batch bleibt dauerhaft
**ohne Entry-Row**, d.h. die beiden erzeugten `extra_hours`-Pair-Rows sind mit
dem Batch nicht verknüpft. `RebookingBatchEntryEntity.hours/balance_before/
voluntary_actual/voluntary_committed` (die dokumentierten Audit-Snapshots aus
`dao/src/rebooking_batch.rs:58-78`) werden für den gesamten F5-Weg nie
persistiert. Das ist ein Datenverlust im Audit-Modell und macht den F5-Pfad
gegenüber F3 asymmetrisch (widerspricht der Aussage "F3 und F5-approve teilen
denselben Pair-write-path" — geteilt wird nur das ExtraHours-Pair, NICHT das
Entry-Handling).

**Fix:** Im `approve`-Pfad nach `rebook(...)` einen Entry für den bestehenden
Batch schreiben (neuer DAO-Call, z.B. `create_entries_for_batch(batch_id, &[entry], tx)`),
oder `create_pending_suggestion` so erweitern, dass approve die Pair-ids in ein
Entry nachträgt. Konkret analog `rebook_manual`:
```rust
let (out_id, in_id) = self.rebook(
    batch.sales_person_id, RebookingDirection::VolunteerToExtra,
    hours, batch.iso_year, batch.iso_week, tx.clone(),
).await?;
let entry = RebookingBatchEntryEntity {
    id: Uuid::nil(),
    batch_id: rebooking_batch_id,          // an bestehenden Batch hängen
    sales_person_id: batch.sales_person_id,
    hours,
    balance_before: /* aus suggest-Snapshot */,
    voluntary_actual: /* … */,
    voluntary_committed: /* … */,
    extra_hours_out_id: Some(out_id),
    extra_hours_in_id: Some(in_id),
    created: time::PrimitiveDateTime::MIN,
    deleted: None,
    version: Uuid::nil(),
};
// neuer DAO-Pfad: Entry an bestehenden Batch anhängen
```

### CR-02: `approve` bucht die vom Client geschickten `hours` ohne jede Validierung

**File:** `service_impl/src/rebooking_reconciliation.rs:291-335`, `rest/src/rebooking.rs:144-160`, `rest-types/src/rebooking.rs:81-84`
**Issue:**
`approve(rebooking_batch_id, hours, …)` übernimmt `hours` direkt aus dem
`RebookingApproveRequestTO` und schreibt damit das -N/+N-Pair
(`rebook(batch.sales_person_id, VolunteerToExtra, hours, …)`, Z. 327-335).
Es gibt **keine** serverseitige Prüfung, dass
- `hours > 0` ist,
- `hours` den zum Zeitpunkt des Suggest berechneten `proposed_hours`
  (= `min(|balance|, voluntary_ist)`) nicht übersteigt.

Der Kommentar in `rest-types/src/rebooking.rs:78-84` ("Der Client übergibt die
im Suggest-Response gelieferten `proposed_hours` unverändert zurück")
beschreibt eine reine Client-Konvention — sie ist **nicht** erzwungen. Ein
HR-Nutzer (autorisiert, aber die Autorität endet nicht bei der Rolle) kann mit
einem manipulierten/fehlerhaften Body beliebige `hours` genehmigen, z.B. mehr
Stunden umbuchen als der Mitarbeiter an freiwilligen Ist-Stunden hat, oder
negative Stunden (was das Vorzeichen der Pair-Rows invertiert). Da der
Suggest→Approve-Zwischenschritt genau existiert, um den Betrag backend-fest zu
machen, unterläuft das Vertrauen auf den Client-Wert das Fat-Backend-Prinzip.

**Fix:** `approve` sollte `hours` serverseitig neu ableiten oder validieren.
Bevorzugt: `proposed_hours` im `approve` selbst neu berechnen (Report + Stats
für die Batch-Woche, wie in `suggest`) und `hours` verwerfen — oder mindestens
`if hours <= 0.0 { return Err(ServiceError::ValidationError(...)) }` plus einen
Deckel gegen `proposed_hours`. Damit wird der genehmigte Betrag unabhängig vom
Client korrekt.

## Warnings

### WR-01: Banner-Predicate (Jahres-kumulativ) und Suggest-Vorschau (Wochen-scoped) rechnen auf verschiedenen Datenbasen

**File:** `service_impl/src/reporting.rs:1543-1547` vs. `service_impl/src/rebooking_reconciliation.rs:246-274`
**Issue:**
Der Alert-Flag feuert auf Basis des **Jahres-kumulativen** Report-Akkumulators:
`alert_predicate(balance_hours, weekly_hours.volunteer_hours, weekly_hours.cap_active)`,
wobei `balance_hours` (Z. 1536) `overall - expected + previous_year_carryover`
über den gesamten Range bis `until_week` ist und `weekly_hours.volunteer_hours`
das Jahres-Aggregat.

`suggest` (Z. 246-274) berechnet IST/DANN dagegen für **eine einzelne
ISO-Woche** (Mo..=So von `iso_week`): `get_report_for_employee_range(from_date=Mo,
to_date=So)` und `get_voluntary_stats` über dieselbe Woche. Balance und
`voluntary_ist` in der Modal-Vorschau sind damit potenziell völlig andere
Zahlen als die, die den Banner ausgelöst haben. Konsequenzen:
- Der Banner kann feuern (Jahres-Balance −8h, Jahres-Volunteer 5h), während
  die Wochen-Vorschau `voluntary_ist = 0` und damit `proposed_hours = 0`
  zeigt → HR sieht einen Alert, aber "0.00h Vorschlag".
- Approve/Reject reagieren auf den Wochen-Slot, nicht auf das kumulative Defizit.

Das ist keine Panik, aber eine inhaltliche Inkonsistenz, die HR verwirrt und
die Fachlogik der Umbuchung fragwürdig macht (buche ich das Wochen- oder das
Jahres-Defizit aus?). Bitte fachlich klären und beide Sites auf denselben Range
ziehen.

**Fix:** Entweder den Banner-Predicate ebenfalls wochen-scoped berechnen, oder
`suggest` über denselben kumulativen Range wie der Report rechnen lassen. Die
gewählte Semantik in einem Test festnageln.

### WR-02: `rebook_manual` validiert `hours` nicht serverseitig (nur FE-Guard)

**File:** `service_impl/src/rebooking_reconciliation.rs:139-196`
**Issue:**
Analog CR-02, aber für F3: `rebook_manual` akzeptiert beliebige `hours`. Die
einzige Prüfung `a > 0.0` liegt im Modal (`manual_rebooking_modal.rs:117`),
also rein clientseitig. Ein direkter API-Aufruf mit `hours = 0` oder negativ
erzeugt ein Pair mit `0.0`/invertiertem Vorzeichen und einen Approved-Batch.
Da F3 keinen Suggest-Zwischenschritt hat, ist die Wahrscheinlichkeit eines
Fehleingriffs sogar höher.

**Fix:** In `rebook_manual` früh validieren: `if !(hours > 0.0) { return
Err(ServiceError::ValidationError("hours must be > 0".into())) }` (bzw. die im
Projekt übliche Validation-Variante). Test ergänzen.

### WR-03: `iso_week`/`iso_year` werden nirgends gegen gültige Bereiche geprüft

**File:** `service_impl/src/rebooking_reconciliation.rs:69-74`, `rest-types/src/rebooking.rs:59-75`
**Issue:**
`iso_week: u8` / `iso_year: u32` kommen ungeprüft aus dem Request. Bei
ungültigen Werten (z.B. `iso_week = 60`) schlägt `ShiftyDate::new(...)` fehl und
wird auf `ServiceError::InternalError` (Z. 70-72, 246-249) gemappt → HTTP 500
statt 422. Ungültige Eingaben sind Client-Fehler, kein interner Fehler.
Zusätzlich kann ein `create_pending_suggestion`-Claim für eine unmögliche Woche
in der DB landen, weil der Claim-INSERT vor der `week_anchor`-Berechnung läuft
(im `approve`/`suggest`-Fluss wird `week_anchor` erst später gebraucht).

**Fix:** Eingangs-Validierung der `(iso_year, iso_week)`-Kombination (1..=53,
plausibles Jahr) mit Mapping auf `ValidationError` (422). `InternalError` nur
für echte Invarianten reservieren.

### WR-04: `approve` schreibt `approved`-Zeitstempel, aber `approved_by = None` — Audit ohne Urheber

**File:** `service_impl/src/rebooking_reconciliation.rs:315-323`, `service_impl/src/rebooking_batch.rs:280-327`
**Issue:**
`approve` ruft `update_state(id, Approved, approved_by=None, …)`. Der
Service-Impl setzt zwar `approved = clock.now()` (gut), aber `approved_by`
bleibt `None`, obwohl das Feld (`RebookingBatchEntity.approved_by`,
`dao/src/rebooking_batch.rs:53`) genau für den Genehmiger-Audit existiert. Der
`Authentication<Context>` des HR-Nutzers ist im Handler verfügbar, wird aber
nicht durchgereicht. Damit lässt sich im Nachhinein nicht feststellen, wer
einen Vorschlag genehmigt hat — ein echtes Manko für einen HR-Genehmigungs-Flow.

**Fix:** Den anfragenden Principal (User-id/Name aus `context`) durch `approve`
→ `update_state` als `approved_by: Some(...)` durchreichen.

### WR-05: `suggest` committet die Transaktion vor der DANN-Berechnung — Claim ohne Preview persistiert

**File:** `service_impl/src/rebooking_reconciliation.rs:276-289`
**Issue:**
`suggest` ruft `self.transaction_dao.commit(tx).await?` (Z. 276) und berechnet
`proposed`/`balance_after`/`voluntary_ist_after` **danach** (Z. 280-288). Der
Claim (pending-Batch-INSERT) ist damit bereits fest committet. Das ist für die
DANN-Werte selbst unkritisch (reine Arithmetik auf schon geladenen Werten),
aber es bedeutet: schlägt zwischen Commit und Response irgendetwas fehl, bleibt
ein pending-Batch geclaimt, ohne dass der Client je die id gesehen hat. Beim
nächsten `suggest` ist der Pfad zwar idempotent (find_pending liefert ihn), aber
falls der Predicate inzwischen nicht mehr gilt, bleibt ein „verwaister"
pending-Batch stehen, der den Slot dauerhaft blockiert (späterer Manual-Rebooking
läuft in 409). Kein Datenverlust, aber ein Slot-Leak-Risiko.

**Fix:** DANN-Arithmetik ist deterministisch — sie könnte vor dem Commit
erfolgen (Reihenfolge tauschen), sodass ein Fehler den Claim mit-rollt. Optional
einen Reaper/Cleanup für verwaiste pending-Batches vorsehen.

## Info

### IN-01: `create` (Basic) hat trotz UNIQUE-Index einen Read-then-write-Pre-Check

**File:** `service_impl/src/rebooking_batch.rs:104-117`
**Issue:**
`create` macht `find_by_sales_person_year_week` + `EntityAlreadyExists`-Return
vor dem INSERT. Der Doc-Kommentar begründet das (nie Panik für den Aufrufer),
aber es ist ein Read-then-write, dessen Race-Fenster nur durch den DB-UNIQUE-Index
geschlossen wird — anders als `create_pending_suggestion`, das bewusst rein auf
den Index setzt (Pitfall 6). Die INSERT-Fehlerbehandlung fängt hier eine echte
UNIQUE-Violation **nicht** als `EntityAlreadyExists` ab (kein `is_unique_violation`),
sondern würde sie als generischen `DatabaseQueryError` (500) durchreichen. Bei
paralleler Ausführung außerhalb einer SERIALIZABLE-Tx bekäme der Verlierer also
500 statt 409.
**Fix:** `create_batch_with_entries` denselben `is_unique_violation`-Mapping-Pfad
geben wie `create_pending_suggestion`, dann ist der Pre-Check nur noch Komfort.

### IN-02: `unwrap()` auf Response-Builder in den REST-Handlern

**File:** `rest/src/rebooking.rs:83, 120-124, 156, 185`
**Issue:**
`Response::builder()...unwrap()` und `serde_json::to_string(&to).unwrap()`. Für
statische Header/Bodies praktisch nicht auslösbar, entspricht dem bestehenden
Projektmuster — aber es sind Panik-Pfade im Request-Handler. Konsistent mit dem
Rest der Codebase; nur zur Kenntnis.
**Fix:** Optional auf den zentralen Response-Konstruktor/`?`-Pfad umstellen.

### IN-03: `create_pending_suggestion` ignoriert `find_pending`-Existenz und verlässt sich auf UNIQUE

**File:** `service_impl/src/rebooking_reconciliation.rs:213-242`
**Issue:**
`suggest` prüft zwar `find_pending_for_sales_person` und ruft
`create_pending_suggestion` nur bei `None`. Zwischen dem `find_pending` und dem
Claim-INSERT liegt ein (kleines) Race-Fenster; korrekt abgefangen durch den
UNIQUE-Index → `EntityAlreadyExists`. Nur der Vollständigkeit halber: der
idempotente „gib dieselbe id zurück"-Zweig greift dann NICHT (der zweite
Aufrufer bekommt 409 statt der id des Gewinners). Für den erwarteten Single-HR-
Klick-Flow unkritisch.
**Fix:** Falls echte Nebenläufigkeit erwartet wird: bei `EntityAlreadyExists`
aus dem Claim ein erneutes `find_pending` versuchen und dessen id zurückgeben.

### IN-04: `week_until`-Ableitung im Frontend fällt bei Nicht-aktuellem Jahr auf 52 zurück

**File:** `shifty-dioxus/src/page/employees.rs:34-39`
**Issue:**
`week_until = if year == current { current_week } else { 52 }`. Jahre mit
ISO-Woche 53 (z.B. 2026) verlieren beim Betrachten als „Vorjahr" die 53. Woche.
`year` ist hier fix auf das aktuelle Jahr gesetzt, daher aktuell nicht
auslösbar, aber ein latenter Off-by-one bei künftiger Jahres-Auswahl.
**Fix:** ISO-Wochenzahl des Zieljahres berechnen statt Konstante 52.

---

_Reviewed: 2026-07-24_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
