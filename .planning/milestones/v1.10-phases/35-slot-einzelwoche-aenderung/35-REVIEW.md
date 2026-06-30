---
phase: 35-slot-einzelwoche-aenderung
reviewed: 2026-06-30T18:48:30Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - service_impl/src/shiftplan_edit.rs
  - service_impl/src/test/shiftplan_edit.rs
  - rest/src/shiftplan_edit.rs
  - service/src/shiftplan_edit.rs
  - shifty-dioxus/src/service/slot_edit.rs
  - shifty-dioxus/src/state/slot_edit.rs
  - shifty-dioxus/src/component/slot_edit.rs
findings:
  critical: 0
  warning: 2
  info: 2
  total: 4
status: issues_found
---

# Phase 35: Code Review Report

**Reviewed:** 2026-06-30T18:48:30Z
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

Reviewed the Phase-35 `modify_slot_single_week` 3-Segment-Split (Backend, REST, Service-Trait)
plus die Frontend-Anbindung (service/state/component für den „nur diese Woche"-Modus).

Die beiden harten Nutzer-Constraints **halten**:

- **SWO-04 (kein Doppelzählen / keine Waisen):** Die Booking-Partition (`shiftplan_edit.rs:295-321`)
  läuft pro Buchung genau einmal: soft-delete der alten Row → ein einziger `create` auf
  Segment 2 (Ausnahme-KW) oder Segment 3 (danach). Das Prädikat
  `booking.year == change_year && booking.calendar_week == change_week as i32` partitioniert
  korrekt; der `u8 → i32`-Cast ist verlustfrei (KW 1-53). Buchungen vor `change_week` werden von
  `get_for_slot_id_since` gar nicht zurückgegeben und bleiben auf dem verkürzten Segment 1
  (valid_to = Sonntag KW-1), das diese Wochen weiterhin abdeckt → keine Waisen. Die alte Row wird
  per soft-delete (`deleted IS NOT NULL`) aus Reporting/Balance ausgeschlossen → kein Doppelzählen
  (gleiches Muster wie das produktiv erprobte `modify_slot`).
- **SWO-03 (Atomarität):** Genau **ein** effektiver Commit. `TransactionDaoImpl::commit`
  (`dao_impl_sqlite/src/lib.rs:334`) committet nur, wenn `Arc::into_inner` den alleinigen
  Besitzer erhält. Alle inneren Service-Calls bekommen `tx.clone()` (Refcount ≥ 2) → deren Commits
  sind No-ops. Der äußere Commit (`shiftplan_edit.rs:324`) ist der einzige reale; jeder `?`-Fehler
  kehrt davor zurück, die `tx` wird gedroppt → sqlx-Rollback. `test_msw_rollback_no_commit_on_error`
  deckt das ab.
- **Permission (D-35-06):** `check_permission("shiftplan.edit")` ist der erste Aufruf vor jeder
  Mutation (`shiftplan_edit.rs:211`), abgedeckt durch `test_msw_forbidden`.
- **FE-Routing:** `save_slot_edit` routet `single_week == true` korrekt auf
  `save_slot_single_week` (`slot_edit.rs:64-68`). Das Flag „klebt" **nicht** über Edits hinweg:
  sowohl `new_slot_edit` als auch `load_slot_edit` setzen `single_week = false` beim Öffnen
  (`slot_edit.rs:44`, `:109`).

Gefunden wurden zwei substanzielle Schwächen (eine Backend-Edge ohne symmetrischen Guard, eine
FE-Borrow-über-`.await`-Falle) sowie zwei Qualitätshinweise.

## Warnings

### WR-01: Letzte-KW-Edge eines begrenzten Slots bricht die Operation ab (fehlender Guard symmetrisch zu Segment 1)

**File:** `service_impl/src/shiftplan_edit.rs:282-292`
**Issue:**
Segment 1 hat einen Guard gegen den „erste-KW"-Fall (`valid_to < valid_from` → `delete_slot`
statt `update_slot`, Zeilen 255-264). Segment 3 hat **keinen** symmetrischen Guard. Es wird
bedingungslos erzeugt mit `valid_from = Montag KW+1` und `valid_to = original_valid_to`.

Wenn die Ausnahme-Woche die **letzte** Woche eines begrenzten Slots ist (`original_valid_to` liegt
am Ende der Ausnahme-KW oder davor), gilt `seg3_valid_from > original_valid_to`. `create_slot`
validiert genau das (`service_impl/src/slot.rs:228-230`) und liefert
`ServiceError::DateOrderWrong` → der gesamte `modify_slot_single_week`-Aufruf bricht ab und rollt
zurück. Konkret: Slot gültig bis `Some(2026-06-28)` (Sonntag KW26), Nutzer editiert exakt KW26:
- Seg1 valid_to = 2026-06-21 (ok)
- Seg2 = 2026-06-22 .. 2026-06-28 (ok, würde aber `original_valid_to` *überdehnen*, wenn dieses
  innerhalb der Ausnahme-KW läge)
- Seg3 valid_from = 2026-06-29 > valid_to = 2026-06-28 → **DateOrderWrong → Abbruch**.

Folge: Die Ausnahme-Änderung der letzten Woche eines begrenzten Slots ist **nicht durchführbar**.
Dank Single-Transaction gibt es keinen Datenverlust/keine Korruption (atomarer Rollback), aber eine
legitime Nutzeraktion schlägt fehl. Unbegrenzte Slots (`valid_to = None`, der häufige Fall) sind
nicht betroffen, weil dann auch Seg3 `None` erbt und der `if let Some`-Check nicht greift. Keiner
der 7 D-35-05-Tests deckt einen begrenzten `valid_to` ab (alle nutzen `None`), daher fällt die
Lücke nicht auf.

**Fix:** Segment 3 — analog zum Segment-1-Guard — nur erzeugen, wenn es nicht-leer ist; Segment 2
zusätzlich gegen `original_valid_to` klammern, damit die Slot-Lebensdauer nicht überdehnt wird:
```rust
// Segment 2: bei begrenztem Original nicht über original_valid_to hinaus verlängern
let seg2_effective_valid_to = match original_valid_to {
    Some(ovt) if ovt < seg2_valid_to => ovt,
    _ => seg2_valid_to,
};
seg2.valid_to = Some(seg2_effective_valid_to);
// ... create seg2 ...

// Segment 3 nur erzeugen, wenn es eine nicht-leere Restspanne gibt
let create_seg3 = match original_valid_to {
    Some(ovt) => seg3_valid_from <= ovt,
    None => true, // unbegrenzt
};
let target_seg3_id = if create_seg3 {
    let mut seg3 = original_snapshot;
    seg3.valid_from = seg3_valid_from;
    seg3.valid_to = original_valid_to;
    seg3.id = Uuid::nil();
    seg3.version = Uuid::nil();
    Some(self.slot_service.create_slot(&seg3, Authentication::Full, tx.clone().into()).await?.id)
} else {
    None
};
```
Im Re-Point-Loop dann `target_seg3_id` defensiv behandeln (es dürften ohnehin keine Buchungen
> Ausnahme-KW existieren, wenn Seg3 leer ist; falls doch — Daten-Inkonsistenz — auf Seg2 mappen
oder hart ablehnen statt zu verwaisen). Test mit begrenztem `valid_to` (Ausnahme-KW == letzte KW)
ergänzen.

### WR-02: Write-Guard auf `SLOT_EDIT_STORE` wird über `.await` gehalten — irreführender Kommentar, Risiko `BorrowError`-Panic

**File:** `shifty-dioxus/src/service/slot_edit.rs:55-79`
**Issue:**
`let mut store = SLOT_EDIT_STORE.write();` hält den Write-Guard, und `store` wird **nach** dem
`.await` erneut benutzt (`store.visible = false` in Zeile 77). Der Guard lebt also über den
Netzwerk-`await` (`save_slot_single_week`/`save_slot`/`create_slot`) hinweg. Der Kommentar in
Zeile 58 („Read single_week before the await to avoid holding the write borrow") ist irreführend:
es wird zwar der *Wert* `single_week` in eine lokale Variable gelesen, aber der *Guard* bleibt
gehalten. Rendert die `SlotEdit`-Komponente während des `await` neu (sie liest
`SLOT_EDIT_STORE.read()` in `component/slot_edit.rs:322`), kann das in eine Borrow-Kollision
laufen (Dioxus-Signale nutzen RefCell-artige Semantik → `already borrowed`-Panic).

Das Muster ist im New-Pfad vorbestehend (`create_slot(..., store.slot.clone()).await`), wurde durch
Phase 35 aber im Edit-Pfad verfestigt.

**Fix:** Alle benötigten Werte vor dem `await` kopieren, den Guard explizit droppen und das Result
ohne gehaltenen Guard verarbeiten:
```rust
let action = {
    let store = SLOT_EDIT_STORE.read();
    (store.slot_edit_type, store.single_week, store.slot.clone(),
     store.year, store.week)
}; // guard dropped here
let (edit_type, single_week, slot, year, week) = action;
// ... await ohne gehaltenen Guard ...
let mut store = SLOT_EDIT_STORE.write(); // erst danach erneut schreiben
store.visible = false;
```

## Info

### IN-01: Erhebliche Code-Duplikation zwischen `modify_slot_single_week` und `modify_slot`

**File:** `service_impl/src/shiftplan_edit.rs:51-143` und `:199-326`
**Issue:** Version-Check, Datumsgrenzen-Berechnung, Segment-1-Verkürzung (update/delete-Guard) und
der Booking-Delete-/Re-Create-Block sind nahezu identisch dupliziert. Divergenzen (wie der in WR-01
gefundene fehlende Seg3-Guard) entstehen leichter, wenn nur ein Zweig gepflegt wird.
**Fix:** Gemeinsame Bausteine in private Helfer extrahieren (z. B. `shorten_or_delete_segment1(...)`
und `repoint_booking(booking, target_slot_id, tx)`), damit beide Pfade dieselbe geprüfte Logik
teilen.

### IN-02: `original_snapshot` dupliziert bereits vorhandenen Zustand

**File:** `service_impl/src/shiftplan_edit.rs:248-251`
**Issue:** `original_valid_to` (Zeile 248) und `original_snapshot` (Zeile 251) halten beide den
Vorzustand; `original_snapshot.valid_to == original_valid_to`. Korrekt, aber redundant —
`seg3.valid_to = original_valid_to` könnte direkt aus dem Snapshot stammen. Kein Defekt, nur
minimaler Lesbarkeitshinweis; der Snapshot-vor-Mutation-Ansatz selbst ist richtig (Pitfall 2).
**Fix:** Optional `original_valid_to` weglassen und `original_snapshot.valid_to` verwenden.

---

_Reviewed: 2026-06-30T18:48:30Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
