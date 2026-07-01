---
created: 2026-05-05T19:18:37.838Z
title: Booking-Log Service liefert sporadisch 500 ohne Logs
area: api
files:
  - rest/src/booking_log.rs
  - service/src/booking_log.rs
  - service_impl/src/booking_log.rs
  - dao/src/booking_log.rs
  - dao_impl_sqlite/src/booking_log.rs:42-49
---

## Problem

Der `booking-log` REST-Service antwortet teilweise mit HTTP 500. In den
Logs taucht zum Zeitpunkt des Fehlers nichts Aussagekräftiges auf —
weder ein `tracing::error!` noch ein Stack-Trace.

**Reproduktions-URL:** `GET /booking-log/2026/19`
(Route in `rest/src/booking_log.rs:16`: `/{year}/{week}`,
Path-Params `Path<(u32, u8)>` → keine Zahl-Parsing-Fehler bei `2026/19`,
also kein Path-Extractor-Problem.)

**User-Verdacht:** "Datumsformat macht oft Probleme."

**Konkreter Verdacht aus Code-Inspektion (`dao_impl_sqlite/src/booking_log.rs:42-49`):**

```rust
time_from: time::Time::parse(time_from_str, &Iso8601::DEFAULT)?,
time_to:   time::Time::parse(time_to_str,   &Iso8601::DEFAULT)?,
created:   PrimitiveDateTime::parse(created_str, &Iso8601::DEFAULT)?,
deleted:   db.deleted.as_ref().map(|d| PrimitiveDateTime::parse(d, &Iso8601::DEFAULT)).transpose()?,
```

Diese vier Aufrufe konvertieren `Option<String>` aus der View
`bookings_view` zu `time::Time` / `time::PrimitiveDateTime` mit
`Iso8601::DEFAULT`. Wenn die DB für *einzelne* Einträge in Woche 19
Strings liefert, die nicht ISO-8601-DEFAULT-konform sind (z. B.
`"2026-04-15 10:30:00"` mit Space statt `T`, oder `"08:00"` vs.
`"08:00:00"`, oder fehlende Bruchteilsekunden), failen genau diese
Zeilen. Der Fehler wandert via `?` als `DaoError` hoch, wird in
`ServiceError` gemappt, in `error_handler` zu HTTP 500 — aber
offenbar **ohne aussagekräftiges Log**.

Sekundärer Verdacht: `bookings_view` selbst — wer schreibt die
Datums-Strings? Wenn unterschiedliche Code-Pfade unterschiedliche
Formate produzieren (alt vs. neu), erklärt das das *sporadische*
Auftreten.

## Solution

**Schritt 1 — Logging schärfen (5 min, nicht-invasiv):**

In `dao_impl_sqlite/src/booking_log.rs:42-49` jedes `parse(…)?` durch
ein `parse(…).map_err(|e| { tracing::error!(…); e })?` ersetzen, das
den fehlerhaften String + Spaltenname loggt:

```rust
time_from: time::Time::parse(time_from_str, &Iso8601::DEFAULT)
    .map_err(|e| {
        tracing::error!("booking_log: failed to parse time_from {time_from_str:?}: {e}");
        e
    })?,
```

Analog für `time_to`, `created`, `deleted`. Außerdem im
`error_handler` (REST-Schicht) sicherstellen, dass jeder 500-Pfad mit
`tracing::error!("{:?}", err)` inkl. Fehler-Chain geloggt wird —
gegenchecken und ggf. ergänzen.

**Schritt 2 — Reproduzieren:**

DB direkt abfragen für `year=2026 AND calendar_week=19`:

```sql
SELECT name, time_from, time_to, created, deleted
FROM bookings_view
WHERE year = 2026 AND calendar_week = 19;
```

Auf abweichende Format-Varianten in den vier Datums-Spalten prüfen.

**Schritt 3 — Fix:**

Je nach Befund:
- **DB-Inhalt fehlerhaft** (Migrations-Altlast): Daten via Migration
  normalisieren ODER Parser tolerant machen (mehrere Format-Beschreibungen
  durchprobieren).
- **Schreiber inkonsistent**: Schreibseite vereinheitlichen, sodass
  `bookings_view` durchgängig ein einheitliches Format liefert
  (z. B. immer `Iso8601::DEFAULT` oder immer SQL-Standard mit Space).
- **Test:** Service-Tier-Test in `service_impl/src/test/` mit
  In-Memory-SQLite, der einen Eintrag mit "schwierigem" Format einfügt
  und sicherstellt, dass `get_booking_logs_for_week` ihn entweder
  korrekt parst ODER einen sauberen, getypten Fehler zurückgibt
  (kein 500-ohne-Log).

**Nicht akzeptabel als Fix:** Fehler stillschweigend schlucken oder
fehlerhafte Einträge stillschweigend überspringen — das Problem muss
sichtbar bleiben.
