# Phase 30: Stale-Daten-Race Guard (FE) - Context

**Gathered:** 2026-06-29
**Status:** Ready for planning
**Mode:** Discuss durchlaufen — **keine User-Design-Grauzonen** (reiner Concurrency-
Korrektheits-Fix; Ansatz + Erfolgskriterien sind in ROADMAP/REQUIREMENTS bereits
vollständig gelockt). Es wurde bewusst keine AskUserQuestion gestellt: die einzige
offene Frage (Generation-Counter vs. `(year,week)`-Tupel) ist ein technisches
Implementierungsdetail (Claude's Discretion lt. Discuss-Philosophie), kein
User-facing Design-Choice.

<domain>
## Phase Boundary

Die Wochen-Summary-Karten unter dem Schichtplan zeigen beim schnellen Wochenwechsel
**immer nur** die Daten der aktuell gewählten Woche. Ein verspätet eintreffendes
Loader-Ergebnis einer anderen Woche wird **still verworfen**, statt den Store mit
Stale-Daten zu überschreiben. Reiner Frontend-Concurrency-Fix; **kein** Backend,
**kein** neuer Endpoint, bestehende API-Calls unverändert.

**In scope:** der `(year, week)`-Staleness-Guard, atomar (dieselbe Guard-Wahrheit)
über **alle drei** Wochen-Loader.
**Out of scope:** Cancellation/Abort in-flight Loads (Todo-Variante 3 — verworfen,
ROADMAP lockt den Verwerf-Ansatz); der Jahres-Loader `LoadYear`
(`load_weekly_summary_year`) ist eine andere View und nicht Teil der Wochenwechsel-Race
(Scoping-Entscheidung, siehe D-30-04).

</domain>

<decisions>
## Implementation Decisions

### Guard-Ansatz (SHP-02)
- **D-30-01:** **Staleness-Check nach dem `await`, nicht Cancellation.** Jeder
  betroffene Loader vergleicht die `(year, week)`, für die er geladen hat, nach
  Rückkehr des `await` gegen die **aktuell gewählte** `(year, week)` und schreibt
  seinen Store **nur**, wenn beide übereinstimmen; sonst wird das Ergebnis still
  verworfen (kein Store-Write, kein Error, kein Log-Spam). (ROADMAP SC1+SC3,
  Todo-Variante 1/2; Variante 3 „Cancellation" ist out of scope.)

- **D-30-02:** **Eine gemeinsame Guard-Wahrheit über alle drei Loader** (ROADMAP SC2:
  „demselben `(year, week)`-Guard atomar"). Es gibt **eine** global lesbare Quelle der
  aktuell gewählten `(year, week)`, gegen die alle drei Loader prüfen — kein
  Loader-lokaler Drift. Die drei betroffenen Pfade:
  1. `WEEKLY_SUMMARY_STORE` — `load_summary_for_week` (`service/weekly_summary.rs:37-42`,
     Coroutine `weekly_summary_service:45-49`, Action `WeeklySummaryAction::LoadWeek`).
  2. `BOOKING_CONFLICTS_STORE` — `load_booking_conflict_week`
     (`service/booking_conflict.rs:20-23`, Coroutine `booking_conflicts_service:27`,
     Action `BookingConflictAction::LoadWeek`).
  3. `reload_unavailable_days`-Closure (`page/shiftplan.rs` ~350-368) +
     Render-Guard für die Summary-Karten (`shiftplan.rs:1124-1160`).

- **D-30-03 (Render-Guard):** Zusätzlich zum Write-Guard rendert die View die
  Summary-Karten nur, wenn die Store-Daten zur aktuell gewählten Woche passen — bei
  Mismatch der bestehende Lade-/Leerzustand (das vorhandene `data_loaded`-Flag-Muster
  bleibt erhalten, keine neue UX). Stellt sicher, dass selbst ein transienter
  Mismatch nie als fremde Woche sichtbar wird (ROADMAP SC1).

### Scoping
- **D-30-04:** Der Jahres-Loader (`WeeklySummaryAction::LoadYear` /
  `load_weekly_summary_year`) ist **nicht** Teil des Guards — er bedient die
  Jahresansicht, nicht die Wochen-Summary-Karten, und ist nicht von der
  Wochenwechsel-Race betroffen. (Wenn der Planner beim Lesen feststellt, dass derselbe
  Store von beiden geteilt wird und ein Year-Load eine Week-Selektion überschreiben
  könnte, ist das als zusätzliche Konsistenz-Note zu vermerken — aber kein
  Scope-Ausbau ohne Beleg.)

### Claude's Discretion (technische Mechanik — NICHT User-Entscheidung)
- **Repräsentation der „aktuell gewählten `(year, week)`"**: neues
  `GlobalSignal<(u32, u8)>` (z.B. `SELECTED_WEEK`), synchron beim Wochenwechsel
  gesetzt **bevor** die Loads dispatcht werden, vs. Wiederverwendung bereits
  existierender globaler `week`/`year`-Signale, falls vorhanden und aus den
  Service-Coroutinen lesbar. Empfehlung: **shared global current-selection**, weil die
  Service-Coroutinen (`weekly_summary_service`, `booking_conflicts_service`) keinen
  Zugriff auf Komponenten-lokale Signale haben — der Guard braucht eine global lesbare
  Wahrheit. Generation-Counter ist eine zulässige Alternative, aber `(year, week)`-Tupel
  ist selbstdokumentierend und entspricht dem ROADMAP-Wortlaut.
- **Wo die `(year,week)` im Store mitgeführt wird** (z.B. `requested_year/week`-Felder
  in `WeeklySummaryStore`) bzw. ob der Vergleich im Loader vor dem Write oder im
  Render passiert — Planner/Researcher entscheiden anhand des realen Codes.
- **Testbarkeit:** der Vergleich („ist dieses Ergebnis noch aktuell?") sollte als reine
  Funktion/Prädikat extrahierbar sein, sodass die Guard-Logik per `cargo test` ohne
  Dioxus-Runtime/Async abgedeckt werden kann (analog Phase 29 `compute_vacation_bar`;
  Browser-/Async-Render-Tests sind laut Projekt-Memory unzuverlässig).

### Folded Todos
- **„Wochen-Summary-Karten gegen Stale-Daten bei schnellem Wochenwechsel absichern"**
  (`.planning/todos/pending/2026-06-29-wochen-summary-karten-gegen-stale-daten-bei-schnellem-wochen.md`,
  `resolves_phase: 30`). Verifizierte Ursache: `load_summary_for_week` überschreibt
  `WEEKLY_SUMMARY_STORE` bedingungslos nach `await`. Folded → durch D-30-01..04 vollständig
  abgedeckt; Todo nennt exakte Zeilen (weekly_summary.rs:37-42/45-49, shiftplan.rs:305-330/1124-1160)
  und benennt explizit, dass `BookingConflictAction::LoadWeek` + `reload_unavailable_days`
  dieselbe Race haben → genau die drei in D-30-02 gelisteten Loader.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirement & Roadmap
- `.planning/REQUIREMENTS.md` §SHP-02 — `(year,week)`-Guard, atomar über alle drei
  Loader, Write-nach-await nur bei Match + Render-Guard.
- `.planning/ROADMAP.md` §"Phase 30" — Goal + Success Criteria 1–3.

### Code (Single Source of Truth — die zu ändernden Stellen)
- `shifty-dioxus/src/service/weekly_summary.rs` — `WeeklySummaryStore`,
  `load_summary_for_week` (Z.37-42, unconditional write), `weekly_summary_service`
  (Z.45-49), `WeeklySummaryAction`.
- `shifty-dioxus/src/service/booking_conflict.rs` — `BOOKING_CONFLICTS_STORE`,
  `load_booking_conflict_week` (Z.20-23), `booking_conflicts_service`,
  `BookingConflictAction`.
- `shifty-dioxus/src/page/shiftplan.rs` — `reload_unavailable_days`-Closure (~350-368),
  Wochenwechsel-Dispatch (~305-330), Render der Summary-Karten (~1124-1160), die
  bestehenden `week`/`year`-Signale (Quelle der aktuell gewählten Woche).

### Konsistenz-Referenz (nur lesen)
- Phase 29 `compute_vacation_bar` (`shifty-dioxus/src/page/absences.rs`) als Muster für
  „reine, per `cargo test` testbare Helfer-Funktion".

</canonical_refs>

<code_context>
## Existing Code Insights

### Established Patterns
- **Dioxus-Coroutine-Service-Muster:** Stores sind `GlobalSignal<...>`; ein Service ist
  eine `async fn service(rx: UnboundedReceiver<Action>)`-Coroutine, die Actions
  sequenziell verarbeitet. Die Race entsteht NICHT durch parallele Coroutinen-Läufe,
  sondern weil die `await`-Kette zwischen mehreren `LoadWeek`-Actions intermediäre,
  veraltete Store-Writes sichtbar macht (jeder Load setzt erst `data_loaded=false`,
  schreibt dann nach dem await — ein verspäteter alter Load gewinnt zuletzt).
- **`data_loaded`-Flag:** bereits vorhanden; das Lade-/Leerzustand-Rendering soll
  unverändert wiederverwendet werden (D-30-03).
- **Service-Coroutinen sehen keine Komponenten-Signale** → der Guard braucht eine
  global lesbare aktuelle `(year, week)` (D-30-02 Claude's Discretion).

### Integration Points
- Drei Loader (s. D-30-02) + ein Render-Guard. Die „aktuell gewählte Woche" wird beim
  Wochenwechsel in `shiftplan.rs` gesetzt; dort werden auch die `LoadWeek`-Actions
  dispatcht — der synchrone Set der Guard-Wahrheit muss VOR dem Dispatch passieren.

</code_context>

<specifics>
## Specific Ideas

Repro/Akzeptanz-Szenario (aus dem Todo): schnelles Durchklicken mehrerer Wochen darf
nie kurzzeitig fremde Wochen-Daten in den Summary-Karten zeigen; ein verspätetes
Ergebnis für Woche N, das eintrifft während Woche M gewählt ist, wird verworfen.
Test-Strategie: reines Guard-Prädikat (`result_week == selected_week ?`) per `cargo test`;
das visuelle „laggt nie" ist nicht pixel-automatisierbar (manueller Smoke optional).

</specifics>

<deferred>
## Deferred Ideas

- In-flight-Load-Cancellation (Todo-Variante 3) — robuster bei sehr teuren Loads, aber
  über den Verwerf-Ansatz hinaus; nicht in Scope (ROADMAP lockt „still verworfen").

### Reviewed Todos (not folded)
- Keine weiteren — Discuss blieb im Phasen-Scope.

</deferred>

---

*Phase: 30-Stale-Daten-Race-Guard (FE)*
*Context gathered: 2026-06-29*
