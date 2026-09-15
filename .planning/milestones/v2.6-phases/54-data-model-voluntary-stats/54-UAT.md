---
status: diagnosed
phase: 54-data-model-voluntary-stats
source: [54-VERIFICATION.md]
started: 2026-07-07T19:45:00Z
updated: 2026-07-07T20:25:00Z
---

## Current Test

[testing complete]

## Tests

### 1. HR-Roundtrip Browser-Smoke — FE-Zeile sichtbar für HR
expected: |
  Unter dem bestehenden 'Volunteer Work'-Wert erscheinen genau 3 neue Zeilen:
  'Freiwillig Ø / Woche', 'Freiwillig Soll', 'Freiwillig Delta'
  (mit positivem Delta in neutraler Farbe, negativem Delta in rot/text-warn).
why_human: |
  Browser-Rendering-Invariant; `dx serve` + Backend-Start nötig; CDP-Methode
  `get_page_text` + `find` gemäß memory `reference_dioxus_browser_verify_reports`.
result: issue
reported: "Stimmt leider komplett gar nicht. Da hat ein Freiwilliger 177 Freiwillige Sollstunden. Ich kann mir gar nicht erklären, wie das zustande kommt. Außerdem wird Freiwillig und Ehrenamt aktuell vermischt. Das ist das selbe und sollte alles Ehrenamt genannt werden."
severity: blocker

### 2. Non-HR-Roundtrip Browser-Smoke — FE-Zeile NICHT sichtbar
expected: |
  Non-HR-User sieht KEINE Freiwillig/Voluntary/Dobrovoln-Zeile im Employee-Detail-Report.
why_human: |
  Rollenseitige Sichtbarkeit nur im Browser prüfbar; Backend-Redaktion (alle Felder None) +
  Component-Guard (`rsx!{}`) greifen gemeinsam.
result: pass

### 3. cs-Locale Wortlaut Native-Check
expected: |
  Bei Locale=Cs erscheinen 'Dobrovolné prům. / týden', 'Dobrovolné plán', 'Dobrovolné rozdíl'
  (ASSUMED gemäß RESEARCH D.4 — kein nativer Tschechisch-Speaker hat das geprüft).
why_human: |
  i18n-Strings sind ASSUMED; native-Check wurde als Manual-Verify markiert
  (54-05-SUMMARY.md).
result: skipped
reason: "User: Das wird schon passen. Das brauchen wir nicht prüfen. — cs-Strings bleiben ASSUMED und werden bei Terminologie-Fix (Ehrenamt) ohnehin mit angefasst."

## Summary

total: 3
passed: 1
issues: 1
pending: 0
skipped: 1
blocked: 0

## Gaps

- truth: "Ehrenamt-Sollstunden werden Range-basiert (from_date..=to_date) berechnet — konsistent mit dem Rest der Employee-Report-Chain und Abrechnungszeitraum-tauglich"
  status: failed
  reason: "User reported: Da hat ein Freiwilliger 177 Freiwillige Sollstunden. Ich kann mir gar nicht erklären, wie das zustande kommt. Bei Nachfrage präzisiert: 'Er arbeitet 5h pro Woche freiwillig seit Mai. Erwartet hätte ich 5h pro Woche seit Mai bis zur aktuellen KW. Das sind keine 177.' Und: 'Das muss immer so sein beim Report. Alleine schon wegen dem Abrechnungszeitraum.'"
  severity: blocker
  test: 1
  root_cause: |
    `service_impl::reporting::committed_voluntary_target_for_year` iteriert
    stur `1..=weeks_in_year(year)` und summiert die Pro-Rata-Zusagen über das
    komplette ISO-Jahr — ohne `until_week`- oder `from..=to`-Cutoff.
    Ebenso ohne Range-Semantik: `voluntary_ist_total_for_year` und
    `contract_weeks_count`.
    Dadurch zeigt der Report für einen Freiwilligen mit `committed_voluntary = 5`
    seit KW 18/2026 den vollen Rest-Jahres-Sollstand (~35 Wochen × 5 = 175 h,
    plus Pro-Rata-Anteil in KW 18 = ~177 h), obwohl alle anderen
    Employee-Report-Werte bis zur aktuellen KW gerechnet werden.
    Verstärkende Design-Lücke: `VoluntaryStatsService::get_voluntary_stats`
    nimmt nur `year` (kein `until_week`, kein `from/to`), und
    `rest::report::VoluntaryStatsRequest` hat kommentiert
    `Nur 'year', keine 'until_week', weil die Aggregation ueber das gesamte
    ISO-Jahr laeuft` — diese Grundannahme ist falsch und muss auf
    Date-Range analog `ReportingService::get_report_for_employee_range`
    (`from_date: ShiftyDate, to_date: ShiftyDate`) umgestellt werden.
  artifacts:
    - path: "service_impl/src/reporting.rs"
      issue: "committed_voluntary_target_for_year / voluntary_ist_total_for_year / contract_weeks_count sind nicht range-fähig"
    - path: "service/src/voluntary_stats.rs"
      issue: "Trait-Signatur nimmt nur (sp_id, year, ctx, tx) statt (sp_id, from_date, to_date, ctx, tx)"
    - path: "service_impl/src/voluntary_stats.rs"
      issue: "Impl ruft target/ist/contract-weeks jahresweit ohne Range"
    - path: "rest/src/report.rs"
      issue: "VoluntaryStatsRequest hat kein from_date/to_date; Doc-Kommentar dokumentiert die falsche Grundannahme"
    - path: "shifty-dioxus/src/**"
      issue: "FE ruft voluntary-stats-Endpoint mit nur year; muss from_date/to_date aus dem Report-Kontext mitgeben"
  missing:
    - "Reporting pure fns range-fähig: committed_voluntary_target_in_range(wh, from_date, to_date), voluntary_ist_total_in_range(eh, from_date, to_date), contract_weeks_count_in_range(wh, from_date, to_date). Edge-Weeks: nur überlappende Tage zählen (Pro-Rata bleibt Tag-basiert)."
    - "VoluntaryStatsService::get_voluntary_stats-Signatur: (sp_id, from_date, to_date, ctx, tx) — analog get_report_for_employee_range."
    - "VoluntaryStatsRequest: { from_date, to_date } (Doc-Kommentar entfernen); FE stellt Report-Range auf voluntary-stats-Call durch."
    - "Neue Tests: 5h/Woche seit Mai → im Jahres-View (from=1.1., to=heute) zeigt Soll ≈ 5 × wochenzahl-seit-Mai; im vollen Vergangenheitsjahr (from=1.1., to=31.12.) bleibt Semantik konsistent zur alten weeks_in_year-Summe."
  debug_session: ""

- truth: "Terminologie im FE ist konsistent — de: durchgängig 'Ehrenamt', en: durchgängig 'voluntary' (Freiwillig/Volunteer/Volunteer Work vermischt sich nicht mehr)"
  status: failed
  reason: "User reported: Außerdem wird Freiwillig und Ehrenamt aktuell vermischt. Das ist das selbe und sollte alles Ehrenamt genannt werden. Präzisiert: 'Im englischen soll es nur voluntary heißen.' Und: 'Ich weiß nicht, ob da jetzt zwei Konzepte erstellt wurden, was eigentlich das selbe ist.'"
  severity: major
  test: 1
  root_cause: |
    Historisch wurden im FE-i18n (shifty-dioxus/src/i18n/{de,en}.rs) zwei
    Wortpaare parallel eingeführt und nie konsolidiert:
    - de: 'Ehrenamt' (Kategorie-Label, seit früher Phase) vs 'Freiwillig …'
      (Phase 15 committed_voluntary + Phase 54 Ist/Soll/Delta-Zeilen)
    - en: 'Volunteer Work' (Kategorie-Label) vs 'Volunteer' vs 'voluntary'
      (Phase 15 + 54 Labels gemischt: 'Paid / Volunteer', 'voluntary hours', …)
    Rust-Symbole (committed_voluntary, VoluntaryStats, Key::CategoryVolunteerWork
    …) sind uneinheitlich, aber der User will nur die FE-User-facing Labels
    vereinheitlicht — Code-Symbole, DB-Spalten und docs bleiben unverändert
    (bestätigt: 'Im englischen soll es nur voluntary heißen' — Doku und
    Backend-Code sind ohnehin englisch/'voluntary'-basiert und damit konsistent).
  artifacts:
    - path: "shifty-dioxus/src/i18n/de.rs"
      issue: "'Freiwillig …' in Keys CategoryVolunteerWork ist bereits 'Ehrenamt', aber Committed / PaidVolunteer / PaidCommittedVolunteer / CommittedVoluntaryLabel / Volunteer / AbsenceGroupVolunteers / VoluntaryHoursIstPerWeek / VoluntaryHoursSoll / VoluntaryHoursDelta zeigen 'Freiwillig …'"
    - path: "shifty-dioxus/src/i18n/en.rs"
      issue: "Mix aus 'Volunteer' / 'Volunteer Work' / 'Voluntary'; User will durchgängig 'voluntary' (lowercase, mit Kapitalisierung nur am Satz-/Zeilen-Anfang)"
  missing:
    - "de: alle betroffenen Keys auf 'Ehrenamt …' umstellen (siehe Artefakt-Liste oben)"
    - "en: alle betroffenen Keys auf 'voluntary' vereinheitlichen; 'Volunteer' → 'Voluntary' und 'Volunteer Work' → 'Voluntary work'"
    - "cs: keine Änderung (Native-Check als Test 3 geskippt; 'Dobrovoln…' ist bereits einheitlich in cs.rs)"
    - "docs/features/F*.md: nicht anfassen — englische Docs verwenden schon 'voluntary'"
  debug_session: ""

---

## Round 2 — Post-Gap-Closure (Plans 54-07 + 54-08 + 54-09-Ist-Fix)

_Datum: 2026-07-10_

### Test 1 — HR-Roundtrip (Range-Cutoff + Ist-Aggregat)

- **Result:** `pass`
- **Vorher:** Ehrenamt Soll = 177h, Delta = -Soll (Ist = 0).
- **Nachher (Range-Fix):** Ehrenamt Soll ≈ Range-basiert korrekt.
- **Nachher (Ist-Fix):** Ehrenamt Ist enthält alle drei Volunteer-Quellen
  (manuelle VolunteerWork-ExtraHours + Shiftplan-Cap-Überlauf +
  no_contract-Shiftplan-Stunden) — deckungsgleich mit dem OVERALL-
  „Ehrenamt"-Wert.
- **User-Feedback:** „Jetzt sieht es richtig aus."

### Test 2 — Non-HR-Roundtrip

- **Result:** `pass`
- Ehrenamt Ø/Soll/Delta-Zeilen bleiben für Non-HR nicht sichtbar
  (Backend-Redaktion + Component-Guard).

### Test 3 — cs-Locale

- **Result:** `skipped`
- User-Feedback aus Round 1: „Das wird schon passen. Das brauchen wir
  nicht prüfen."
- Deferred-Item bleibt in Plan 54-08 (`54-08-cs-rename`).

### Terminologie-Konsistenz

- **DE:** durchgängig „Ehrenamt" — geprüft in HR-View (Ehrenamt Ø/Woche,
  Ehrenamt Soll, Ehrenamt Delta).
- **EN:** durchgängig „Voluntary" — kein „Volunteer" mehr in Labels sichtbar.

