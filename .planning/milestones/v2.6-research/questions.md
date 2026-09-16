# Research Questions

Offene Fragen, die vor der jeweiligen Plan-Phase per Code-Mapping / Recherche
beantwortet werden sollten. Format: eine Sektion pro Frage mit Kontext und Ziel.

---

## Q-01 — Kanonische Slot-Clip-Funktion für Kurzer-Tag-Feature

**Verweise:** Seed `.planning/seeds/shortday-slot-clipping.md`,
Note `.planning/notes/shortday-slot-clipping-semantics.md`

**Frage:** Wo lebt in der aktuellen Codebase die Slot-Auflösung, und wo genau
muss die Clip-Funktion konsumiert werden?

Konkrete Sub-Fragen:

1. Wo werden Slots heute pro Datum aufgelöst? Gibt es bereits einen zentralen
   Punkt (`SlotService::slots_for_date`?), oder machen das Reporting, WeekView,
   Booking-Info je selbst?
2. Wo wird `special_day` heute konsumiert? (Feiertag-Handling im Reporting
   deutet auf einen existierenden Lookup-Pfad, der wiederverwendet werden
   könnte.)
3. Genaue Signatur der aktuellen Booking-Stunden-Berechnung — welche
   Funktion(en) rechnen `slot.end - slot.start` und wo müssten sie stattdessen
   die geclippte Effektiv-Zeit nutzen?
4. Wie viele Call-Sites gibt es realistisch? Kandidaten aus dem Bauch:
   - Frontend WeekView (Slot-Rendering)
   - Reporting-Service (Balance/Ist-Stunden)
   - Booking-Information-Service (Aggregates)
   - PDF-Export (v2.3-Renderer, gerade neu, 5-Parameter-Signatur)
   - iCal-Export
   - Booking-Validation (darf man ganz-außerhalb-Slots buchen?)
5. Konsumiert das Frontend Slot-Zeiten direkt aus rest-types-DTOs, oder
   berechnet es selbst? Entscheidet mit, ob die Clip-Logik im
   Backend-Response mitgeliefert wird oder im FE dupliziert werden muss.

**Ziel:** Vor `discuss-phase` einen 1-Seiter mit
- Liste der Call-Sites (mit Datei:Zeile)
- Empfehlung für Ort der kanonischen Clip-Funktion
  (`shifty-utils`, Method auf `Slot`, oder `service_impl`-Helper)
- Skizze der Signatur

**Wann:** Sobald v2.3 archiviert ist und v2.4 (oder Nachfolge-Milestone mit
diesem Feature) startet. Als `/gsd-map-codebase`-Fokus-Pass oder als
`gsd-phase-researcher`-Agent im discuss-phase-Vorfeld.

---

## Q-02 — `reporting_service.get_year`-Aggregation korrekt reproduzierbar?

**Verweise:** Seed `.planning/seeds/weekly-overview-perf.md`,
Note `.planning/notes/weekly-overview-perf-analyse.md`

**Frage:** Kann `reporting_service` eine `get_year(year, ...)`-Aggregation
liefern, die alle bestehenden Wochen-Invarianten byte-identisch reproduziert
— oder gibt es Berechnungen, die zwingend pro-Woche isoliert bleiben müssen?

Konkrete Sub-Fragen:

1. **Balance-Formel:** Ist die Wochen-Balance rein additiv aus vor-berechneten
   Werten, oder gibt es Berechnungsschritte, die pro Woche ein
   Toggle/Special-Day-Lookup brauchen und daher schwer batchbar sind?
2. **Chain-C-Legacy-Filter unter `shortday_gate.active_from`:** Der Toggle
   wird bereits einmalig gelesen (Zeile 309 in `booking_information.rs`).
   Wird er in `reporting_service.get_week` erneut gelesen, oder ist die
   Semantik dort schon parametrisiert?
3. **CVC-06 Cap-Gating:** Pro-Person-Cap ist heute pro-Woche gerechnet. Ist
   das mathematisch dasselbe wie über das ganze Jahr batchen und dann pro
   Woche zerschneiden? (Vermutlich ja — aber verifizieren.)
4. **ShortDay-Slot-Clipping (v2.4):** Die Clip-Funktion hängt an
   `special_day.until`. Wenn wir Special-Days einmal fürs Jahr laden und
   Slots einmal fürs Jahr, ist das Clipping ebenfalls einmal-pro-Jahr
   batchbar?
5. **Andere Call-Sites von `reporting_service.get_week`:** Wenn der Trait
   erweitert wird statt ersetzt — welche Call-Sites bleiben bei `get_week`?
   REST-Handler `/report/week/{year}/{week}` z.B.

**Ziel:** Vor `discuss-phase`/`plan-phase` einen 1-Seiter mit:
- Für jede Sub-Frage: Ist die Invariante im Jahres-Batch reproduzierbar? Wo
  liegt das mathematische / semantische Risiko?
- Empfehlung: `get_week` erweitern (Trait) oder neue `get_year`-Methode
  koexistierend
- Grobe Skizze der Signatur der neuen Aggregat-Funktion + welches Return-
  Shape (Map<Week, WeekReport> oder ein anderes Aggregat)

**Wann:** Sobald der nächste Milestone startet und diese Optimierung
tatsächlich Kandidat ist. Als `gsd-phase-researcher`-Agent im
discuss-phase-Vorfeld.
