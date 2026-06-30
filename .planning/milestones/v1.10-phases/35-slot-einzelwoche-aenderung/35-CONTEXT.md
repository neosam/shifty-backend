# Phase 35: Slot-Werte nur für eine Woche ändern - Context

**Gathered:** 2026-06-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Ein **Shiftplanner** kann die Werte eines Slots (`min_resources`, `max_paid_employees`,
`from`, `to`) für **genau eine Kalenderwoche** als **einmalige Ausnahme** ändern, ohne die
wiederkehrende Struktur ab dieser KW dauerhaft zu verändern. Umgesetzt als Erweiterung der
bestehenden `ShiftplanEditService::modify_slot`-„ab KW"-Mechanik um einen **dritten
Restore-Schnitt**, plus einer **UI-Wahl** im Slot-Editor zwischen „nur diese Woche" und
„ab dieser Woche".

**Achse:** Backend (Mechanik) + Frontend (Editor-Modus-Wahl).

**Requirements:** SWO-01, SWO-02, SWO-03, SWO-04.

**Harte Constraints (User, 2026-06-30):**
1. **Atomarität:** Alle Slot-Schnitte + alle Booking-Re-Points in **einer** Transaktion;
   Fehler → vollständiger Rollback, Zustand exakt wie vorher.
2. **Keine Doppelzählung:** Booking-Neuzuweisungen wasserdicht getestet — nichts doppelt
   oder verwaist in Reports/Balance.

**Liefert NICHT:**
- Kein neues „week-specific override"-Datenmodell (Ansatz A bewusst verworfen).
- Keine Änderung an `day_of_week`/`valid_from`-Semantik der Slot-Versionierung darüber hinaus.

</domain>

<decisions>
## Implementation Decisions

### Mechanik (SWO-01/SWO-02)
- **D-35-01 (Ansatz B: Split + Re-Merge, NICHT A):** „nur diese Woche" = derselbe Schnitt wie
  die heutige Dauerlösung (`modify_slot`), plus ein **drittes Segment**, das ab `KW+1` die
  Original-Werte wiederherstellt. Drei Slot-Versionen:
  - **Segment 1** (Original): `valid_to = Sonntag (KW-1)`.
  - **Segment 2** (Ausnahme): `valid_from = Montag KW`, `valid_to = Sonntag KW`, neue Werte.
  - **Segment 3** (Restore): `valid_from = Montag (KW+1)`, Original-Werte, `valid_to` =
    ursprüngliches `valid_to` (kann `None`/unbegrenzt sein).
  Ansatz A (neues Override-Datenmodell) bewusst verworfen: invasiv (Tabelle, Booking-/Report-
  Auflösung, Migration), höheres Risiko gegen die Doppelzählungs-Anforderung. B nutzt die
  bewährte, bereits atomare Maschinerie.
- **D-35-02 (UI-Wahl, Ansatz C als Schicht darüber):** Im Slot-Editor expliziter Modus-Schalter
  **„nur diese Woche" vs „ab dieser Woche"**. „ab dieser Woche" = heutiges Verhalten (2 Segmente),
  „nur diese Woche" = D-35-01 (3 Segmente). Konkrete Editor-Verdrahtung (Toggle/Radio,
  State-Feld in `SlotEdit`) = UI-Phase / Claude's Discretion.

### Booking-Re-Point (SWO-02/SWO-04)
- **D-35-03 (Buchungen aufteilen):** Heute holt `modify_slot` alle Buchungen ab `change_week`
  (`get_for_slot_id_since`) und re-pointet sie auf die neue Version. Für „nur diese Woche" werden
  diese **in zwei Gruppen geteilt**:
  - Buchungen mit `calendar_week == change_week` → Segment 2.
  - Buchungen mit `calendar_week > change_week` (bzw. Datum ≥ KW+1) → Segment 3.
  Re-Point-Mechanik unverändert (alte Buchung soft-delete + neue mit `slot_id` des Zielsegments,
  `created_by = None` → System-Stempel; Authorship überlebt in der soft-gelöschten Vorgänger-Zeile).
  **Dies ist die kritische, hart zu testende Stelle (D-35-05).**

### Atomarität (SWO-03)
- **D-35-04 (Eine Transaktion, vorhandene Klammer wiederverwenden):** `modify_slot` ist bereits
  atomar (`use_transaction` am Anfang, **ein** `commit` am Ende, alle Schritte teilen `tx`).
  Der dritte Schnitt + die aufgeteilten Re-Points bleiben **in derselben `tx`** vor dem Commit.
  Keine Zwischen-Commits. Bei jedem Fehler → kein Commit → Rollback.

### Tests (SWO-04)
- **D-35-05 (Re-Point-/Keine-Doppelzählung-Tests Pflicht):** Aufbauend auf
  `service_impl/src/test/shiftplan_edit.rs`. Mindestens:
  - Werte weichen nur in der Ausnahme-KW ab; KW-1 und KW+1 zeigen Original (3-Segment-Struktur).
  - Buchung in der Ausnahme-KW landet auf Segment 2, Buchung in KW+1 auf Segment 3 —
    je **genau einmal**, keine verwaisten/doppelten Rows.
  - **Report/Balance-Konsistenz:** dieselbe Person/Buchung erscheint nicht doppelt
    (Reporting zählt nur `deleted IS NULL`).
  - Rollback-Test: Fehler mitten im Vorgang → DB exakt wie vorher (kein Teil-Schnitt persistiert).
  - Randfälle: Ausnahme-KW == erste KW des Slots (Segment 1 entfällt → `delete_slot` wie heute);
    Slot ohne `valid_to` (Segment 3 unbegrenzt); Ausnahme-KW ohne Buchungen.

### Permission (SWO-04)
- **D-35-06 (Gate `shiftplan.edit`):** Konsistent zu `modify_slot` (heute `check_permission("shiftplan.edit")`).
  FE-Gate entsprechend (das Wochenraster/Editor ist bereits `shiftplan.edit`/`shiftplanner`-Kontext).

### Claude's Discretion
- Backend-Verdrahtung: neuer Parameter `single_week: bool` an `modify_slot` **vs.** eigene
  Methode `modify_slot_single_week` (+ eigene REST-Route oder erweiterter `edit_slot`-Handler).
- Editor-UI-Layout des Modus-Schalters; State-Feld in `SlotEdit`/`SlotEditItem`.
- Exakte Datumsarithmetik der KW-Grenzen (ISO-Woche Montag/Sonntag) — Helfer aus `modify_slot`
  wiederverwenden.
- i18n-Texte (de/en/cs) für die Modus-Wahl + etwaige Hinweise.

### Folded Todos
- **`2026-06-26-einzelnen-slot-nur-fuer-eine-kw-aendern-statt-ab-kw.md`** (`area: shiftplan`) —
  kanonische Quelle dieser Phase. Skizzierte Ansätze A/B/C; Diskussion hat **B (Mechanik) + C (UI)**
  gewählt, A verworfen. Komplett in Phase 35 gefoldet.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Projektregeln
- `.planning/REQUIREMENTS.md` § "Slot-Werte nur für eine Woche ändern (SWO)" — SWO-01..04.
- `.planning/ROADMAP.md` § "Phase 35" — Goal + 4 Success Criteria.
- `.planning/todos/pending/2026-06-26-einzelnen-slot-nur-fuer-eine-kw-aendern-statt-ab-kw.md` —
  gefoldete Quelle (Problem, Ansätze A/B/C, offene Punkte).

### Mechanik-Vorbild (zu erweitern)
- `service_impl/src/shiftplan_edit.rs:51-143` — `modify_slot` (heutige „ab KW"-Logik:
  Split in 2 Segmente, Booking-Re-Point, atomar via `use_transaction`/`commit`). **Hier**
  das dritte Restore-Segment + die aufgeteilten Re-Points ergänzen.
- `service_impl/src/shiftplan_edit.rs:145+` — `remove_slot` (analoge Schnitt-/Booking-Logik
  als Referenz).
- `rest/src/shiftplan_edit.rs:22-26,43-77` — Routen `PUT /shiftplan-edit/slot/{year}/{week}`
  (`edit_slot` → `modify_slot`) + `DELETE .../{slot_id}/{year}/{week}` (`remove_slot`). Hier
  ggf. Flag/Variante für „nur diese Woche".

### Slot-Modell & Versionierung
- `service/src/slot.rs:13-24` — `Slot` (`valid_from`/`valid_to`/`version`/`shiftplan_id`).
- `service_impl/src/slot.rs:203-340` — `create_slot`/`update_slot`/`delete_slot`
  (update verbietet `day_of_week`/`from`/`to`/`valid_from`-Änderung; Overlap-Check; `shiftplanner`-Gate).
- `rest-types/src/lib.rs` § `SlotTO` — Wire-Format (`valid_from`, `valid_to`, `max_paid_employees`,
  `$version`, `shiftplan_id`).

### Booking-Kopplung (Doppelzählungs-Risiko)
- `service/src/booking.rs:16,85-90` — `Booking.slot_id`, `get_for_slot_id_since`.
- `service_impl/src/test/shiftplan_edit.rs` — bestehende Tests (Basis für D-35-05).
- `service_impl/src/booking_information.rs` / `service_impl/src/reporting.rs` — Report-/Balance-
  Aggregation (zählt nur `deleted IS NULL`) — gegen Doppelzählung verifizieren.

### Frontend Slot-Editor
- `shifty-dioxus/src/state/slot_edit.rs` — `SlotEditItem`/`SlotEdit` (`new_valid_from`,
  `valid_from`/`valid_to`); hier Modus-Feld ergänzen.
- `shifty-dioxus/src/service/slot_edit.rs` — `SlotEditAction`/`save_slot_edit`/`delete_slot_edit`;
  hier den „nur diese Woche"-Pfad anbinden.
- `shifty-dioxus/src/component/slot_edit.rs` — Editor-Dialog (Modus-Schalter).
- `shifty-dioxus/src/api.rs:180-195` — `delete_slot_from` / `post_slot` (API-Muster);
  ggf. neue/erweiterte Edit-API für „nur diese Woche".

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`modify_slot` ist bereits atomar UND macht Split + Booking-Re-Point** — die „nur diese
  Woche"-Logik ist eine reine Erweiterung (drittes Segment + Re-Point-Aufteilung), kein Neubau.
- **Booking-Re-Point-Pattern** (delete + create mit neuem `slot_id`, `created_by=None`) steht
  und ist erprobt — nur die Ziel-Segment-Zuordnung kommt hinzu.
- **ISO-Wochen-Datumsarithmetik** (`from_iso_week_date(..., Monday)`, `- Duration::days(1)`)
  direkt aus `modify_slot` übernehmbar für die KW+1-Grenze.

### Established Patterns
- **Transaktions-Pattern** (`use_transaction(tx)` → Schritte mit `tx.clone()` → ein `commit`)
  ist die Atomaritäts-Klammer — strikt einhalten (D-35-04).
- **`shiftplan.edit`-Gate** für ShiftplanEditService-Mutationen (nicht `shiftplanner`).
- **Soft-Delete `WHERE deleted IS NULL`** — Reports ignorieren soft-gelöschte Vorgänger-Buchungen
  (Schutz gegen Doppelzählung, in Tests verifizieren).
- **Service-Tier:** `ShiftplanEditService` ist Business-Logic-Tier (konsumiert SlotService +
  BookingService) — die Erweiterung bleibt dort.

### Integration Points
- **FE Editor → Backend:** Modus „nur diese Woche" → erweiterter/neuer `modify_slot`-Aufruf
  (Flag oder eigene Methode/Route, D-35-Discretion).
- **Booking-Aufteilung:** `get_for_slot_id_since(change_week)` liefert ab-KW-Buchungen; in der
  Schleife nach `calendar_week == change_week` (→ Seg 2) vs `> change_week` (→ Seg 3) trennen.

</code_context>

<specifics>
## Specific Ideas

- **Verhalten bei späterer Dauer-Änderung:** Eine bereits gesetzte Einzelwochen-Ausnahme bleibt
  als eigenes Segment bestehen, wenn später „ab einer KW dahinter" geändert wird (sie wird nicht
  automatisch eingesammelt) — bewusst akzeptiert.
- **Versionswachstum:** Häufige Einzelwochen-Ausnahmen erzeugen viele Slot-Versionen — akzeptiert
  (kein Cleanup in dieser Phase).
- **Rollback-Test als First-Class-Test** (D-35-05): bewusst ein Fehler mitten im Vorgang → DB
  unverändert.

</specifics>

<deferred>
## Deferred Ideas

- **Ansatz A (week-specific override Datenmodell)** — verworfen zugunsten B; falls künftig viele
  Einzelwochen-Overrides/Cleanup nötig werden, eigenständige Re-Evaluierung.
- **Eingesammeltes Mergen/Cleanup** redundanter Slot-Versionen — Future-Idee, nicht in v1.10.

### Reviewed Todos (not folded)
- Keine weiteren — Phase blieb im Slot-Einzelwochen-Scope.

</deferred>

---

*Phase: 35-slot-einzelwoche-aenderung*
*Context gathered: 2026-06-30*
</content>
