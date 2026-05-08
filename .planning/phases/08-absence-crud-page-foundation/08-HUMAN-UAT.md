---
status: partial
phase: 08-absence-crud-page-foundation
source: [08-06-PLAN.md, 08-06-SUMMARY.md]
started: 2026-05-08T15:00:00Z
updated: 2026-05-08T16:00:00Z
blocked_by: phase-9-cutover-migration-ui
---

## Current Test

**Blocked.** Awaiting Phase 9 (Cutover-Migration-UI) so der int-UAT auf migrierten Daten laufen kann. Aktuell hängt der Cutover an Drift-Patterns, die die Auto-Heuristik (Plan 08-09) nicht abdeckt; manuelle Resolution braucht die Phase-9-UI.

## Tests

### HR-User-Flow (20 Schritte)

**Setup:** Backend (Port 3000) + Tailwind-Watch + Dioxus-Frontend (Port 8080) + Cutover bereits committed (feature_flag `absence_range_source_active = true`).

**Login:** Mock-/OIDC-User MIT `hr`-Privileg.

#### 1. Login HR
expected: Top-Bar zeigt User-Avatar + Logout
result: [pending]

#### 2. Top-Bar prüfen
expected: Menü-Eintrag "Abwesenheiten" sichtbar (De) bzw. "Absences"/"Nepřítomnosti" (En/Cs) — bei HR unter "Verwaltung"-Submenu (Plan 08-07)
result: [pending]

#### 3. Click "Abwesenheiten"
expected: URL = `/absences/`, Page rendert
result: [pending]

#### 4. VacationEntitlementCard (HR-Variante)
expected: Header "Urlaubsanspruch Team · {N} Personen", Stat-Boxen Vertrag/Übertrag/Genommen/Beantragt/Verbleibend, **VacationPerPersonList**-Section mit Avatar+Name+Resturlaub pro Mitarbeiter, sortiert nach `remaining` aufsteigend
result: [pending]

#### 5. AbsenceFilterBar (HR)
expected: Drei Filter sichtbar: Person-Dropdown (HR-only), Kategorie-Pills, Status-Dropdown
result: [pending]

#### 6. AbsenceList (HR)
expected: 5-Spalten-Layout: Person+Description / Range+Days / CategoryBadge / StatusPill / Warnings; alle Mitarbeiter-Einträge sichtbar
result: [pending]

#### 7. Click "Neue Abwesenheit"
expected: Modal öffnet zentriert, Width ~520px
result: [pending]

#### 8. Modal-Form prüfen
expected: Mitarbeiter-Dropdown editierbar, Kategorie Vacation/SickLeave/UnpaidLeave wählbar, Von/Bis als native `<input type="date">`, Description Textarea
result: [pending]

#### 9. Cross-Field-Validation: Bis < Von
expected: Field-Error "Enddatum liegt vor Startdatum" am Bis-Feld; Save-Btn DISABLED. Nach Korrektur: Save-Btn ENABLED
result: [pending]

#### 10. Form mit gültigen Daten ausfüllen
expected: Form valide (Mitarbeiter, Vacation, Von=heute+1, Bis=heute+5, Description="UAT-Test")
result: [pending]

#### 11. Click "Anlegen"
expected: 201 → Modal schließt (oder bei Booking-Überlappung: WarningList rendert, Submit-Label wechselt zu "Verstanden" → click → Modal schließt)
result: [pending]

#### 12. AbsenceList Refresh
expected: Refreshed automatisch via ABSENCE_REFRESH-Bump → neuer Eintrag sichtbar
result: [pending]

#### 13. VacationEntitlementCard Refresh
expected: Team-Stats neu berechnet (Beantragt aktualisiert)
result: [pending]

#### 14. Click neuer Listen-Eintrag
expected: Modal öffnet im Edit-Mode mit prefilled Form
result: [pending]

#### 15. Description ändern, "Speichern"
expected: 200 → Modal schließt → Liste reflektiert Änderung
result: [pending]

#### 16. Eintrag → Edit-Modal → Delete-Btn (Danger, links unten)
expected: DeleteConfirmDialog öffnet
result: [pending]

#### 17. DeleteConfirmDialog prüfen
expected: Center-Dialog Width ~360, Title "Abwesenheit löschen?", Body Soft-Delete-Hinweis, Buttons Cancel (Ghost) + Löschen (Danger). NICHT `window.confirm`
result: [pending]

#### 18. Click "Löschen"
expected: 204 → beide Modals schließen → Eintrag aus Liste verschwunden
result: [pending]

#### 19. Self-Overlap-Test (D-11)
expected: "Neue Abwesenheit" mit gleichem Mitarbeiter + Range-Überlappung → Save → 422 → SelfOverlapBanner inline (linker `border-bad`, `bg-bad-soft`, Header "Selbst-Überlappung"). Modal bleibt OFFEN, Form-State erhalten
result: [pending]

#### 20. Range korrigieren, Save
expected: 201 → Modal schließt
result: [pending]

---

### Employee-User-Flow (15 Schritte)

**Setup:** Backend + Frontend laufen weiter aus HR-Flow oder Restart.

**Login:** Mock-/OIDC-User OHNE `hr`-Privileg, MIT `sales_person_id` zugewiesen.

#### 21. Logout HR + Login Employee
expected: Auth-Switch erfolgreich
result: [pending]

#### 22. Top-Bar (Employee)
expected: Menü-Eintrag "Abwesenheiten" sichtbar (D-10: für ALLE eingeloggten User), bei Non-HR Top-Level (nicht im Verwaltung-Submenu)
result: [pending]

#### 23. Click "Abwesenheiten"
expected: Page rendert auf `/absences/`
result: [pending]

#### 24. VacationEntitlementCard (Self-Variante)
expected: 2-Spalten-Hero-Layout, Hero-Spalte mit `text-display font-mono text-good` Hero-Zahl `{remaining}/{entitled}`, Sublabel "Tage verbleibend", Card-Title "Dein Urlaubskonto", Stats Vertrag/Übertrag/Genommen/Beantragt/Verbleibend. KEINE VacationPerPersonList-Section
result: [pending]

#### 25. AbsenceFilterBar (Employee)
expected: KEIN Person-Dropdown (Employee-Variante); nur Kategorie + Status + "Vergangene anzeigen"
result: [pending]

#### 26. AbsenceList (Employee)
expected: Nur Einträge des eingeloggten Employee
result: [pending]

#### 27. Click "Neue Abwesenheit"
expected: Modal öffnet
result: [pending]

#### 28. Modal-Form (Employee)
expected: Mitarbeiter-Dropdown DISABLED, vorgefüllt mit eingeloggtem Employee (D-09 `lock_person = true`); Kategorie/Range/Description editierbar
result: [pending]

#### 29. Anlegen analog HR-Flow Schritt 10-12
expected: 201 → Modal schließt → Liste refreshed → VacationCard refreshed
result: [pending]

#### 30. Edit eigener Eintrag
expected: 200 → Modal schließt
result: [pending]

#### 31. Delete eigener Eintrag via Confirm-Dialog
expected: 204 → Liste refreshed
result: [pending]

#### 32. Forbidden-Test (Defense-in-Depth, T-8-AUTH-01 + T-8-IDOR-01)
expected: in Browser-DevTools → Console: `fetch('/absence-period/by-sales-person/{ANDERE-UUID}')` mit fremder Person → Backend liefert **403** (NICHT 200). UI hat fremde IDs nicht exposed; dieser Smoke-Test bestätigt das Backend-Gate
result: [pending]

#### 33. Locale-Switch En
expected: Page-Title "Absences", Kategorie-Pills "Vacation"/"Sick leave"/"Unpaid leave"
result: [pending]

#### 34. Locale-Switch Cs
expected: Page-Title "Nepřítomnosti", Kategorie-Pills "Dovolená"/"Nemoc"/"Neplacené"
result: [pending]

#### 35. Locale-Switch zurück De
expected: "Abwesenheiten", "Urlaub"/"Krankheit"/"Unbezahlt"
result: [pending]

## Summary

total: 35
passed: 0
issues: 0
pending: 35
skipped: 0
blocked: 35

## Gaps

### gap-1
status: failed
phase: 08
title: Cutover blockiert int-UAT — Auto-Heuristik deckt nicht alle realen Buchungs-Patterns ab
detail: |
  Plan 08-09 erkennt ganze Wochenpauschalen (1× expected_hours pro ISO-Woche), aber
  die int-Daten enthalten Patterns die nicht greifen:
  (a) Pre-Check matched nicht für Lila/Anina/Karin u.a. trotz scheinbar passendem Pattern
      (Vertragsdaten-Edge-case, Diagnose offen)
  (b) Teil-Wochen-Pauschalen (n × hours_per_day für n < workdays_per_week) sind nicht
      abgedeckt — User-Konvention "3 von 4 Tagen Urlaub als 15h-Eintrag"
  (c) Feiertag-Inkonsistenz zwischen Pre-Check (ignoriert Feiertage) und
      derive_hours_for_range (zieht sie ab) → false-positive Cluster mit Drift
      (z.B. Sonja Vac 2026: 80h legacy, 76.67h derived)
  (d) Echte Datenprobleme (z.B. Karin P. SickLeave 2026 mit "contract_not_active_at_date")
      brauchen manuelle Korrektur
resolution: phase-9-cutover-migration-ui
debug_session: null
