---
created: 2026-06-29T14:08:01.509Z
title: Wochen-Summary-Karten gegen Stale-Daten bei schnellem Wochenwechsel absichern
area: shiftplan
files:
  - shifty-dioxus/src/service/weekly_summary.rs:37-42
  - shifty-dioxus/src/service/weekly_summary.rs:45-49
  - shifty-dioxus/src/page/shiftplan.rs:305-330
  - shifty-dioxus/src/page/shiftplan.rs:1124-1160
---

## Problem

Die Tabelle/Karten unterhalb vom Schichtplan, die die verfügbaren Stunden pro
Wochentag anzeigen, laggen hinterher: wechselt man schnell zwischen Wochen, zeigen
sie kurzzeitig die Daten einer anderen (nicht mehr ausgewählten) Woche an. Das
sieht nach einer Race Condition aus und sollte nicht passieren — angezeigt werden
darf immer nur das, was zur aktuell gewählten Woche gehört.

**Ursache (verifiziert):** Die Karten werden aus `WEEKLY_SUMMARY_STORE` gespeist.
Beim Wochenwechsel feuert `shiftplan.rs` ein `WeeklySummaryAction::LoadWeek(year, week)`
(z.B. `src/page/shiftplan.rs:305-330`). Die Coroutine `weekly_summary_service`
(`src/service/weekly_summary.rs:45-49`) ruft `load_summary_for_week` auf, das nach
dem `await` den globalen Store **bedingungslos** überschreibt
(`src/service/weekly_summary.rs:37-42`):

```rust
async fn load_summary_for_week(year: u32, week: u8) -> Result<(), ShiftyError> {
    (*WEEKLY_SUMMARY_STORE.write()).data_loaded = false;
    let weekly_summary = loader::load_summary_for_week(CONFIG.read().clone(), year, week).await?;
    (*WEEKLY_SUMMARY_STORE.write()).weekly_summary = Rc::new([weekly_summary]);
    (*WEEKLY_SUMMARY_STORE.write()).data_loaded = true;
    ...
}
```

Der Store kennt die zugehörige `(year, week)` nicht und prüft beim Schreiben nicht,
ob das Ergebnis noch zur aktuell ausgewählten Woche passt. Bei schnellem Wechsel
werden mehrere Loads angestoßen; verspätete/zwischenzeitliche Antworten überschreiben
den Store mit veralteten Daten → die Karten "laggen" sichtbar hinterher, bis der
letzte Load gewinnt.

## Solution

Sicherstellen, dass nur das Ergebnis der aktuell gewählten Woche in den Store/UI
gelangt. Optionen (TBD, eine wählen):

1. **Selektions-Token / Generation-Counter:** Beim Wochenwechsel einen Counter
   hochzählen; jeder Load merkt sich seinen Wert und schreibt nur dann in den Store,
   wenn er beim Zurückkommen noch aktuell ist (sonst Ergebnis verwerfen).
2. **`(year, week)` im Store mitführen:** `load_summary_for_week` schreibt
   `requested_year/week` mit; die View rendert die Karten nur, wenn Store-`(year,week)`
   == aktuell gewählte `(year, week)` (`week`/`year`-Signale in `shiftplan.rs`).
   Mismatch → Lade-/Leerzustand statt Stale-Daten.
3. **In-flight Load abbrechen:** beim Wochenwechsel den vorherigen Load canceln,
   bevor ein neuer startet.

Variante 1 oder 2 ist am robustesten (Dioxus-Coroutine verarbeitet Actions zwar
sequenziell, aber die await-Kette macht intermediäre Stale-Writes sichtbar). Gleiche
Race kann auch `BookingConflictAction::LoadWeek` und `unavailable_days`
(`reload_unavailable_days`, `src/page/shiftplan.rs:350-368`) betreffen — beim Fix
mitprüfen, ob dieselbe Guard-Logik dort nötig ist.
