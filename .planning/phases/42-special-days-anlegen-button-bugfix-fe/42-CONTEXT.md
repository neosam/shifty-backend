# Phase 42: Special-Days-„Anlegen"-Button-Bugfix (FE) - Context

**Gathered:** 2026-07-02
**Status:** Ready for planning

<domain>
## Phase Boundary

Reiner Frontend-State-Fix im Special-Day-Anlegeformular der Einstellungen
(`shifty-dioxus/src/page/settings.rs`). Bug: Nach erfolgreichem Anlegen setzt
der Success-Handler die Formularfelder zurück (`sd_date_str`/`sd_type`/`sd_time_str`
→ leer/`None`), wodurch `sd_form_valid` `false` wird und der „Anlegen"-Button
ausgraut. Ziel (Option 2, roadmap-gelockt): nach Create **nichts** an den
Formularfeldern zurücksetzen, sodass Typ/Datum stehen bleiben und der Button
aktiv bleibt → mehrfaches Anlegen hintereinander ohne Dropdown-Toggle.

**Kein Backend-Anteil** (begründete „Backend out of scope"-Notiz): reiner
FE-State-Fix ohne API-Wirkung; kein neues/geändertes TO. SDF-Desync ist ein
isolierter Settings-Bug. Kein Snapshot-Bump, keine Migration, keine neuen Deps.

</domain>

<decisions>
## Implementation Decisions

### Reset-Verhalten nach Create
- **D-42-01:** Option 2 — nach erfolgreichem Create werden die drei Formular-Feld-Resets
  entfernt: `sd_date_str.set(String::new())`, `sd_type.set(None)`,
  `sd_time_str.set(String::new())` (heute `settings.rs:456–462`; Roadmap nannte
  „458-459", Zeilen leicht verschoben). Typ/Datum/Zeit bleiben stehen → `sd_form_valid`
  bleibt `true` → „Anlegen"-Button bleibt aktiv.
- **D-42-02:** `sd_year.set(iso_year)` (WR-04, Jahresgrenzen-Sichtbarkeit) und
  `sd_resource.restart()` (Liste neu laden) sind **keine** Formular-Resets und
  **bleiben erhalten**. „Nichts zurücksetzen" gilt nur für die drei Feld-Resets.

### Duplikat-Handling (WR-02-Regression)
- **D-42-03:** Ein sofortiger Zweitklick mit unveränderten Feldern (exaktes Duplikat)
  wird **akzeptiert** — keine zusätzliche Button-Sperre. Begründung: das Backend
  prüft/behandelt Duplikate seit Phase 36 als atomaren in-place-Replace (HTTP 422→success),
  ein Resubmit ist damit idempotent/harmlos. Der bestehende Inline-Hinweis
  `sd_is_duplicate` (via `is_duplicate_special_day`) bleibt informativ sichtbar,
  wird aber **nicht** an `button.disabled` gekoppelt. Dies kehrt bewusst die
  Phase-36-WR-02-Reset-Entscheidung um.

### Erfolgs-Feedback
- **D-42-04:** `sd_save_result` bleibt unverändert — die „Gespeichert"-Meldung
  bleibt bis zum nächsten „Anlegen"-Klick sichtbar (kein Clear beim Feld-Edit).
  `sd_save_result.set(None)` am Kopf von `on_add_special_day` räumt es beim
  nächsten Submit ohnehin auf.

### Test-Strategie (SC #3: SSR-/Komponenten-Test)
- **D-42-05:** Harter Test (Pflicht): Das Button-/Validitäts-Prädikat
  (`date non-empty && type is Some && (type≠ShortDay || time non-empty)`, heute inline
  `settings.rs:387–389`) in eine reine Funktion extrahieren (analog zu bestehenden
  Helpern `sd_type_to_select_value`, `is_duplicate_special_day`) und unit-testen —
  inkl. „bleibt `true` für gefülltes Formular nach Create". Ergänzend eine reine
  Funktion, die die Post-Create-Retention-Policy modelliert, sodass „mehrfaches
  Anlegen, Formulardaten bleiben erhalten" ohne Browser-Flakiness prüfbar ist.
- **D-42-06:** Best-effort (sekundär): VirtualDom-/SSR-Render-Test, der prüft, dass
  der „Anlegen"-Button nach dem Mount nicht `disabled` ist — **nur falls** die
  Komponente ohne Live-Backend/Config mountbar ist. Researcher/Planner klärt die
  Machbarkeit (Resource-/Config-Deps); wenn nicht sinnvoll mountbar, wird der
  SSR-Weg mit begründeter Skip-Notiz dokumentiert und der Pure-Unit-Test aus
  D-42-05 ist die alleinige Absicherung.

### Claude's Discretion
- Test-Granularität und die Frage Pure-Unit-only vs. Pure-Unit + SSR sind an
  Claude delegiert (D-42-05/06): Pure-Unit ist das harte Gate, SSR ist best-effort
  je nach Mountbarkeit.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase-Definition & Requirement
- `.planning/ROADMAP.md` §Phase 42 — Goal, Success Criteria, Scope (FE-only, Reset-Block entfernen)
- `.planning/REQUIREMENTS.md` §SDF-01 — Requirement-Text (Button bleibt aktiv, mehrfaches Anlegen, SSR-Test)

### Zu ändernder Code
- `shifty-dioxus/src/page/settings.rs` — Special-Day-Formular; Success-Handler
  `~456–462` (Reset-Block), Validitäts-Prädikat `~387–389`, Button `~681–686`,
  bestehende Helper + Tests `~64–180`
- `shifty-dioxus/CLAUDE.md` — Frontend-Konventionen (Controlled-Select D-06/D-08, i18n)

### Vorläufer-Kontext (Phase 36, gleicher Bereich)
- `.planning/milestones/v1.11-ROADMAP.md` — Phase 36 SDF-01/02 (in-place Replace 422→success,
  controlled SelectInput; erklärt die WR-02/WR-04-Kommentare im aktuellen Code)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `is_duplicate_special_day(parts, &sd_list)` (`settings.rs:83`) + `sd_is_duplicate`-Signal
  (`settings.rs:392`): erkennt exakte Duplikate, rendert Inline-Hinweis. Bleibt informativ (D-42-03).
- `sd_type_to_select_value` (`settings.rs:71`) + zugehörige Unit-Tests: Vorbild für den
  neu zu extrahierenden reinen Validitäts-Helper (D-42-05).
- `Btn { variant, disabled, on_click }` (`component/atoms/btn.rs`): der „Anlegen"-Button,
  `disabled: !sd_form_valid || *sd_saving.read()`.

### Established Patterns
- Controlled `<select>` via `sd_type_to_select_value` (D-06/D-08) vermeidet Signal↔DOM-Desync;
  darf durch den Fix NICHT reingebrochen werden (Feld bleibt einfach gefüllt statt geleert).
- Tests in `settings.rs` sind reine `#[cfg(test)]`-Unit-Tests über pure Helper — kein
  VirtualDom-Harness vorhanden (bestätigt D-42-05 als Default-Pfad).

### Integration Points
- Nur der Success-Arm von `spawn(async { create_special_day(...) })` im `on_add_special_day`-Closure
  (`settings.rs:447–470`) wird berührt. `create_special_day`-API und TOs bleiben unverändert.

</code_context>

<specifics>
## Specific Ideas

Häufiger User-Flow als Prüfstein: Tag A anlegen → Datum auf Tag B ändern → anlegen,
ohne das Typ-Dropdown neu zu togglen. Nach dem Anlegen von A ist A kurz ein „Duplikat"
(Inline-Hinweis erscheint), was harmlos ist, weil der User als Nächstes das Datum ändert.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope (isolierter FE-Bugfix).

</deferred>

---

*Phase: 42-special-days-anlegen-button-bugfix-fe*
*Context gathered: 2026-07-02*
