# Phase 36: Special-Days-Bugfixes (BE+FE) - Context

**Gathered:** 2026-07-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Zwei gemeldete Special-Days-Bugs (Nachlese zu v1.10/Phase 33) beheben:

- **SDF-01:** Umstellen eines Tages Feiertag ↔ „Kurzer Tag" auf demselben Datum
  ersetzt den bestehenden Special-Day-Eintrag statt einen zweiten anzulegen — keine
  Fehlermeldung, neuer Typ persistiert.
- **SDF-02:** In der Settings-Special-Days-Karte lassen sich mehrere Feiertage
  nacheinander anlegen, ohne dass der „Anlegen"-Button hängen bleibt.

**Keine neuen Fähigkeiten.** Kein Snapshot-Schema-Bump (bleibt 12), keine Migration,
keine neuen Deps, i18n unberührt (keine neuen Texte). Requirements-Zielverhalten ist
in REQUIREMENTS.md fixiert; diese Phase klärt nur den Implementierungsweg.

</domain>

<decisions>
## Implementation Decisions

### SDF-01 — Fix-Ort & Semantik
- **D-01:** Fix im **Backend**, nicht im Frontend. Der Service-`create`-Pfad ersetzt
  einen bestehenden Same-Date-Eintrag (gleiche `year`/`calendar_week`/`day_of_week`)
  **atomar in einer Transaktion** statt `ValidationError([Duplicate])` zurückzugeben.
  Mechanik: bestehenden Eintrag soft-deleten (bestehender `delete`-Pfad setzt
  `deleted`+`version`) und den neuen einfügen — oder DAO-`update` erweitern; beide
  laufen in **einer** Transaktion (Rollback-sicher, keine Doppelzählung/keine
  Zwischenzustände). Der bestehende Duplicate-Guard (`service_impl/src/special_days.rs:142-153`)
  wird durch die Ersetzungs-Semantik abgelöst.
- **D-02:** **Kein** Frontend delete-then-create und **kein** neuer PUT/Update-Endpoint.
  Zwei-Call-Frontend-Lösung verworfen (nicht atomar: Delete-OK/Create-Fail ließe den
  Tag leer). Der Schichtplan-Dropdown-Handler (`shiftplan.rs:846`/`:968`) bleibt
  unverändert und ruft weiter nur `create` — der Backend-Fix macht das robust.
- **D-03:** Reproduktion + exakter Statuscode/Fehlertext (Backend-Log + Netzwerk-Response)
  wird beim Umsetzen festgehalten (Requirement-Vorgabe).

### SDF-01 — Abdeckung Settings-Karte
- **D-04:** Der Backend-Fix (D-01) ist **zentral** — er deckt automatisch **beide**
  Flächen ab: Schichtplan-Wochenraster-Dropdown **und** Settings-Special-Days-Karte
  (beide rufen denselben `create`-Pfad). Kein separater Fix pro Fläche nötig.

### SDF-02 — Fix-Ansatz
- **D-05:** `SelectInput` (`shifty-dioxus/src/component/form/inputs.rs:82-102`)
  generisch **controlled** machen: optionales `value`-Prop zu `SelectInputProps`
  ergänzen und am `<select>` binden (Root-Cause-Fix, alle Dropdowns immun gegen
  Signal-vs-DOM-Desync). NICHT der Minimal-Ansatz „`sd_type` nach Create behalten".
- **D-06:** In der Settings-Karte wird das `<select>` aus dem `sd_type`-Signal
  abgeleitet (`None`→`""`, `Holiday`→`"holiday"`, `ShortDay`→`"short_day"`). Der
  Reset nach erfolgreichem Create (`settings.rs:429`, `sd_type.set(None)`) leert dann
  auch sichtbar das Dropdown → Button-Zustand und Anzeige bleiben synchron.
- **D-07:** Rückwärtskompatibilität: `value` ist ein **optionales** Prop; bestehende
  `SelectInput`-Aufrufer ohne `value` behalten das aktuelle (uncontrolled) Verhalten.
- **D-08:** D-25-06-Datepicker-Caveat mitdenken: falls das Datum-Feld nach Reset einen
  stale Signal-Wert hätte, dort den Reset/Re-Trigger ebenfalls absichern.

### Test-/Verifikations-Strategie
- **D-09:** cargo-Tests sind die **harten Gates**:
  - Backend: Test für den Switch-Pfad — Holiday→ShortDay (und umgekehrt) auf demselben
    Datum **ersetzt** den Eintrag statt `Duplicate` zu werfen; genau ein aktiver
    (nicht-deleted) Eintrag pro Datum danach.
  - Frontend: SSR-/Komponenten-Test für die Settings-Karte — mehrfaches Anlegen ohne
    Dropdown-Toggle hält den Button korrekt aktiv; `<select>`-Wert folgt dem Signal.
- **D-10:** Browser-e2e nur **optional/manuell** — wegen D-25-06 (programmatisches
  Setzen von `<input type=date>`/`<select>` triggert Dioxus-Signale nicht zuverlässig)
  wird Anzeige-/Reset-Logik per cargo-Test verifiziert, nicht per Browser-Automation.
- **D-11:** Standard-Gates dieser Phase: `cargo build`, `cargo test`,
  `cargo clippy --workspace -- -D warnings` (Backend), plus `cargo test -p shifty-dioxus`
  + WASM-Build (`cargo build --target wasm32-unknown-unknown` aus `shifty-dioxus/`).

### Claude's Discretion
- Exakte DAO-Mechanik für D-01 (soft-delete-then-insert vs. `update` erweitern) —
  Planner/Executor wählt die kleinere saubere Variante; beide erfüllen „update statt
  zweitem insert" und bleiben atomar.
- Konkreter Typ des `value`-Props (`ImStr`/`String`/`Option<..>`) am `SelectInput`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Roadmap
- `.planning/REQUIREMENTS.md` §Special-Days-Fixes (SDF) — SDF-01/SDF-02 Zielverhalten (locked)
- `.planning/ROADMAP.md` §Phase 36 — Scope + Success-Kriterien
- `.planning/todos/2026-07-01-schichtplan-feiertag-auf-kurzer-tag-wirft-fehler.md` — SDF-01 Ursprung/Repro
- `.planning/todos/2026-06-30-settings-special-days-anlegen-button-disabled.md` — SDF-02 Ursprung/Repro

### Backend — Special Day (SDF-01)
- `service/src/special_days.rs:83-106` — Service-Trait (get_by_week/get_by_year/create/delete; **kein** update)
- `service_impl/src/special_days.rs:85-192` — `create` (Duplicate-Guard `:142-153`) + `delete` (soft-delete `:165-192`)
- `dao/src/special_day.rs:29-38` — DAO-Trait
- `dao_impl_sqlite/src/special_day.rs:127-189` — `create` (plain INSERT, kein ON CONFLICT) + `update` (nur deleted/version); `find_by_week` filtert `deleted IS NULL`
- `rest/src/special_day.rs:17-29,115-165` — Routes (POST `/`, DELETE `/{id}`; **kein** PUT)
- `migrations/sqlite/20241020064536_add-special-day-table.sql` — **kein** UNIQUE-Constraint auf Datum (nur `id` PK)

### Frontend — Schichtplan-Dropdown (SDF-01) & Settings-Karte (SDF-02)
- `shifty-dioxus/src/page/shiftplan.rs:794-981` — Per-Tag-Dropdown (Holiday `:846`, ShortDay `:968`, Delete `:882-909`); `existing_id`/`existing_sd` bereits berechnet
- `shifty-dioxus/src/api.rs:1020-1040` — `create_special_day` (POST), `delete_special_day` (DELETE); **kein** update
- `shifty-dioxus/src/page/settings.rs:332,357-359,380-383,417-431,611-624,649` — `sd_type`-Signal, Enable-Prädikat, Create-Guard, Reset, `<select>`-onchange, Button-disabled
- `shifty-dioxus/src/component/form/inputs.rs:50,63-102` — `TextInput` (controlled `value:`) vs. `SelectInput` (uncontrolled — hier `value`-Prop ergänzen)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Bestehender `delete`-Pfad (`service_impl/src/special_days.rs:165-192`, soft-delete) —
  Baustein für die atomare Ersetzung in D-01.
- `TextInput` mit `value:`-Binding (`inputs.rs:50`) — Vorlage für das controlled-Pattern,
  das `SelectInput` bekommen soll (D-05).
- Schichtplan-Dropdown hat `existing_id`/`existing_sd` bereits zur Hand — nach Backend-Fix
  ohne FE-Änderung nutzbar.

### Established Patterns
- Alle Special-Day-Queries filtern `deleted IS NULL` → soft-deletete Zeilen zählen nicht
  zum Duplicate-Check; ermöglicht soft-delete-then-insert ohne DB-Constraint-Konflikt.
- Service-Methoden nehmen `Option<Transaction>` → atomare Ersetzung in einer Transaktion
  (D-01) ist idiomatisch.

### Integration Points
- SDF-01: Änderung liegt vollständig in `service_impl/src/special_days.rs::create`
  (+ ggf. DAO). REST/FE-Contract bleibt gleich (POST `/`), keine API-Signaturänderung.
- SDF-02: Änderung in shared `inputs.rs::SelectInput` (+ Aufruf in `settings.rs`);
  rückwärtskompatibel über optionales `value`-Prop (D-07).

</code_context>

<specifics>
## Specific Ideas

- Zielverhalten „update statt zweitem insert — keine Fehlermeldung, neuer Typ
  persistiert" (SDF-01) und „Button nach Create sofort wieder aktiv, kein
  Dropdown-Toggle-Workaround nötig" (SDF-02) sind wörtlich aus den Requirements
  übernommen und dürfen nicht aufgeweicht werden.

</specifics>

<deferred>
## Deferred Ideas

None — Diskussion blieb innerhalb des Phasen-Scopes (SDF-01/SDF-02). Modal-UX
(MOD-01/02) → Phase 37, Build-Hygiene (HYG-01/02) → Phase 38, laut Roadmap.

</deferred>

---

*Phase: 36-special-days-bugfixes*
*Context gathered: 2026-07-01*
