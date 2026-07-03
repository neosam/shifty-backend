---
milestone: v2.3
milestone_name: PDF-Export — Browser-Look & Download-Button
created: 2026-07-03
---

# Requirements — v2.3: PDF-Export — Browser-Look & Download-Button

## Kontext

Der in v2.2 gelieferte Nextcloud-PDF-Export via WebDAV (Phase 48) rendert
PDFs, die praktisch unlesbar sind: ein starres mm-Absolut-Layout, keine
sichtbaren Slot-Zellen, keine Uhrzeiten je Slot, nur Sales-Person-Namen in
Zeilen und Wochentage in Spalten. Das WebDAV-Ablegen funktioniert
technisch, hilft dem Team aber nicht.

v2.3 löst zwei Probleme zusammen:

1. Die PDFs müssen visuell dem Schichtplan im Browser
   (`shifty-dioxus/src/page/shiftplan.rs`) ähneln — Slots als Zellen mit
   Uhrzeiten, Bookings mit Sales-Person-Name, so dass ein A4-Ausdruck ohne
   externe Referenz nutzbar ist. Zusätzlich soll jede Seite den
   Renderzeitpunkt tragen, damit Ausdrucke datierbar sind.
2. Auf der Schichtplan-Seite selbst gibt es einen Download-Button für die
   im UI selektierte Kalenderwoche, damit man den Export ad-hoc nutzen kann
   und nicht auf den täglichen Cron-Slot warten muss.

Der Reihenfolge-Trick: Phase 49 liefert nur den Button (nutzt den alten,
unlesbaren Renderer), Phase 50 tauscht den Renderer aus. So kann das
Rendering-Ergebnis in Phase 50 direkt per Button-Klick verifiziert werden.

## Nicht-Ziele

- **i18n der PDF-Texte** — bleibt Deutsch (Datumsformat, „Schichtplan KW"-
  Header, Timestamp-Label). Frontend-Button-Label wird i18nt.
- **Byte-Determinismus des Renderers** — fällt implizit durch PDF-02
  (Renderzeitpunkt). Der v2.2-Determinismus-Vertrag (fixe Metadata
  2000-01-01) wird in Phase 50 aufgehoben. WebDAV-Overwrite bleibt korrekt.
- **Neue Cargo-Deps** — keine erwartet. `printpdf` bleibt.
- **Migration** — keine. Snapshot-Schema-Version bleibt **12**.

## Requirements

### PDF-01 — PDF-Layout wie Browser-Wochenansicht

Das gerenderte PDF entspricht sichtbar dem Schichtplan im Browser:

- Landscape A4.
- Sieben Wochentag-Spalten (Mo–So).
- Slots als eigene Zellen/Kästen innerhalb einer Spalte, sortiert nach Slot-
  Startzeit, mit Uhrzeit-Label pro Zelle (Format „08:00 – 12:00").
- Innerhalb der Slot-Zelle stehen die Namen der gebuchten Sales-Persons.
- Kopfzeile: „Schichtplan KW {NN} ({JJJJ})" wie im Browser.
- Muss beim Blick auf ein A4-Blatt lesbar sein — Erfolgskriterium
  „Ausdruck ohne Digital-Referenz nutzbar".

**Verifikation:** Backend-Test rendert einen bekannten Beispieltag mit
mehreren Slots + Bookings, prüft dass alle relevanten Strings im PDF-
Textstream vorkommen. Zusätzlich: manueller UAT-Klick auf den Button aus
Phase 49 gegen ein reales Wochen-Fixture.

### PDF-02 — Renderzeitpunkt auf jeder Seite

Jede PDF-Seite trägt den Renderzeitpunkt sichtbar, Format „Erstellt am
DD.MM.YYYY HH:MM Uhr" (lokale Zeit des Backend-Servers, keine TZ-Angabe
im Text nötig).

**Verifikation:** Backend-Test injiziert Fixed-Timestamp über den
Renderer-Parameter, prüft dass der Renderzeitpunkt im Textstream steht.
Der Renderer nimmt den Timestamp als Argument entgegen (Pure-Funktion
bleibt testbar; Uhrzeit wird von Aufrufern injiziert).

### PDF-03 — Download-Button auf Schichtplan-Seite

Auf `shifty-dioxus/src/page/shiftplan.rs` gibt es einen Download-Button
neben dem iCal-Button. Klick lädt das PDF der **aktuell im UI selektierten
Kalenderwoche** (via `year`/`week`-Signals des Shiftplan-Views), für den
aktuell im Catalog ausgewählten Shiftplan.

Dateiname: `schichtplan-{JJJJ}-KW{NN}.pdf` (dieselbe Konvention wie beim
WebDAV-Export in v2.2, Phase 48).

**Verifikation:** Manueller UAT — Klick, Datei landet mit korrektem
Dateinamen im Browser-Download.

### PDF-04 — Sichtbarkeits-/Download-Gate auf `week_status`

Der Button ist im Frontend nur sichtbar, wenn der `WeekStatus` der aktuell
selektierten KW ∈ {`Planned`, `Locked`} liegt. Bei `Unset` (keine
persistierte Row) oder `InPlanning` wird der Button nicht gerendert (kein
disabled-Zustand, kein Tooltip).

Der Backend-Endpoint gibt bei `WeekStatus ∈ {Unset, InPlanning}` HTTP 409
zurück (Defense-in-Depth — Race-Case wenn Status zwischen Signal-Update
und Klick wechselt).

**Verifikation:** Backend-Integrationstest gegen alle vier Status-Werte
für die KW. Frontend-Test (cargo) auf reine Predikat-Fn
`should_show_pdf_button(week_status, shiftplan_id) -> bool`.

### PDF-05 — Kein Admin-Gate auf Download

Der Download ist für **jeden authentifizierten User** erreichbar
(Employee eingeschlossen). Keine Sales-Person-Filterung („eigene
Bookings only"). Das gesamte KW-PDF ist sichtbar für jeden Angemeldeten
wie im Schichtplan-Browser-View auch.

**Verifikation:** Backend-Integrationstest mit Employee-Auth-Context,
prüft 200 statt 403.

## Constraints

- **Byte-Determinismus des Renderers** (v2.2-Vertrag) wird bewusst
  gebrochen. Kein Test darf zwei-Renderings-diffen-für-Byte-Gleichheit
  mehr erwarten.
- **Renderer bleibt pure** — nimmt Timestamp als Argument, kein
  `SystemTime::now()` intern. Aufrufer (WebDAV-Scheduler + neuer
  REST-Handler) injizieren die Uhrzeit.
- **WebDAV-Scheduler wird nicht angefasst** — er nutzt automatisch den
  neuen Renderer, sobald Phase 50 mergt. Der Scheduler injiziert
  seinerseits `chrono::Utc::now()` als Renderzeitpunkt.
- **Kein Snapshot-Bump**, keine Migration, keine neue Cargo-Dep.
