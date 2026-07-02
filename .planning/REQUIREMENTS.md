# Requirements: Shifty — v2.1 Schichtplan- & Reporting-Erweiterungen

> **Versions-Hinweis:** `v2.1` ist das interne GSD-Planungs-Label (MAJOR.MINOR). Die
> reale Release-Version vergibt der User via `/release-version` → `./cli-update-version.sh`
> (PATCH aus Git-Tags). Das v1.11-Release wird als v2.0.0 ausgeliefert; Releases aus diesem
> Milestone = v2.1.0 ff.

**Defined:** 2026-07-01
**Core Value:** Zwei neue Steuerungs-/Auswertungs-Fähigkeiten für die Schichtplanung —
Kalenderwochen mit Status/Sperre gegen unbeabsichtigte Änderungen schützen und die
durchschnittliche tatsächliche Anwesenheit flexibler Mitarbeiter sichtbar machen — plus
ein isolierter Settings-Bugfix, der beim autonomen Nacht-Run mitreitet.

**Research:** durchgeführt (2026-07-01, 4 Dimensionen + Synthese, HIGH confidence) —
`.planning/research/SUMMARY.md`. Kernergebnis: **keine neuen Dependencies**, alle Muster
bereits im Codebase erprobt; AVG-01 = reines Read-Aggregat (kein Snapshot-Bump).

**Ausgangslage (code-verifiziert durch Research):**

- **WST-01:** Muster-Vorlagen vorhanden — `week_message` (ISO-`(year, calendar_week)`-
  Composite-Key + DAO-CRUD), `extra_hours`/`special_day` (TEXT-Enum + manuelles `match` in
  `TryFrom`), `SHIFTPLANNER_PRIVILEGE`-Gate in `week_message`-Service, `is_shiftplanner`-
  Hard-Block in `shiftplan_edit`. Neuer `WeekStatusService` = **Basic-Tier**; das Sperr-Gate
  lebt im **Business-Logic-Tier** (`ShiftplanEditService`).

- **WST-Sperr-Bypass (kritisch):** `DELETE /booking/{id}` routet heute direkt über
  `BookingService::delete` (Basic-Tier) und umgeht damit alle Business-Logic-Gates → braucht
  eine neue `ShiftplanEditService::delete_booking`-Methode + REST-Re-Routing.

- **AVG-01:** Vorläufer-Formel **A-22-1** (`average_worked_hours_per_week`, `service/src/reporting.rs`)
  existiert, ist aber **nicht identisch** (schließt alle Absence-Kategorien aus, nicht nur
  Urlaub) — Nenner-/Exclusion-Regel muss in discuss-phase explizit entschieden werden.
  „Flexibel" = `EmployeeWorkDetails.is_dynamic == true`.

- **SDF-Desync:** `settings.rs:458-459` Reset-Block nach Create → Controlled-Select-Desync
  (D-25-06-Klasse); v1.11-Fix (Option 1, controlled `<select>`) wirkt in WASM nicht
  zuverlässig. Entschiedene Lösung: **Option 2 — nach Create nichts zurücksetzen.**

**i18n (querschnittlich):** Alle neuen benutzersichtbaren Texte in de/en/cs — gilt für
WST-*, AVG-* und SDF-01.

**CI-Gates (autonomer Run, querschnittlich):** Nach jeder neuen `query!`/`query_as!`
→ `cargo sqlx prepare --workspace` (in `nix develop`) + `.sqlx` committen; jedes Phasen-Gate
fährt `cargo clippy --workspace -- -D warnings` mit; WST-Datenmodell braucht KW-53-/
Jahresgrenzen-Unit-Tests (ISO-Wochen-Jahr ≠ Gregorianisches Jahr).

## v2.1 Requirements

### Kalenderwochen-Status & Sperre (WST)

- [x] **WST-01**: Schichtplaner kann den Status einer Kalenderwoche setzen/ändern —
  **None / In Planung / Geplant / Gesperrt** — persistiert pro **ISO-(Jahr, Woche)**
  (neue Tabelle + Migration; TEXT-Enum-Muster analog `special_day`). Wer den Status setzen
  darf und welche Übergänge erlaubt sind, wird in discuss-phase bestätigt (Default: Schichtplaner,
  alle Übergänge).

- [x] **WST-02**: Der KW-Status wird in der Schichtplan-Wochenansicht als **Badge** angezeigt
  (für alle Rollen sichtbar; Setzen/Ändern nur für Schichtplaner). UI als Badge + Aktions-Button
  (kein controlled `<select>`, um D-25-06-Desync zu vermeiden).

- [x] **WST-03**: In einer **Gesperrt**-Woche werden Buchungs- und Slot-Schreibaktionen für
  **Nicht-Schichtplaner** server-seitig blockiert (`ServiceError::WeekLocked` → HTTP-Code in
  discuss-phase, Default **423 Locked**; 409-Alternative geprüft); Schichtplaner behält
  Vollzugriff. Check läuft **in derselben Transaktion** wie der Write (kein TOCTOU).

- [x] **WST-04**: Die Sperre greift auf **allen** Schreibpfaden **ohne Bypass** —
  `book_slot_with_conflict_check`, `modify_slot`, `modify_slot_single_week`, `remove_slot`,
  `copy_week_with_conflict_check` und **neu** `delete_booking` (inkl. Re-Routing von
  `DELETE /booking/{id}` über den Business-Logic-Tier). Geteilter `assert_week_not_locked`-Helper;
  Test-Matrix 6 Pfade × {gesperrt, offen}.

- [x] **WST-05**: i18n de/en/cs für alle vier Status-Labels und die Sperr-Rückmeldung.

### Durchschnittliche Anwesenheit bei flexiblen Stunden (AVG)

- [ ] **AVG-01**: HR kann pro **flexiblem** Mitarbeiter (`is_dynamic == true`) die
  **durchschnittliche tatsächliche Anwesenheit** über einen Zeitraum einsehen, wobei **Urlaub
  aus dem Nenner** herausgerechnet wird. Exakte Bezugsgröße (Woche/Monat/Abrechnungsperiode),
  Zähler (geleistete Stunden vs. Anwesenheitstage) und das genaue Exclusion-Set (nur Urlaub vs.
  auch Krankheit/unbezahlt/Feiertag) werden in discuss-phase entschieden (D-AVG-01..08). Die
  bestehende A-22-1-Formel wird **nicht** blind wiederverwendet.

- [x] **AVG-02**: Die Auswertung ist im Frontend sichtbar (Report-/Auswertungs-Sicht). Reines
  **Read-Aggregat** im `ReportingService` (Business-Logic-Tier) — **kein Snapshot-Bump**, keine
  neue Persistenz, kein neuer `BillingPeriodValueType`.

- [ ] **AVG-03**: i18n de/en/cs für die neue Auswertungs-Sicht (Labels, Tooltips, Leerzustand).

### Special-Days-Settings-Bugfix (SDF)

- [ ] **SDF-01**: Nach erfolgreichem Special-Day-Anlegen in den Einstellungen bleibt der
  **„Anlegen"-Button aktiv** — mehrfaches Anlegen hintereinander ohne Dropdown-Toggle möglich.
  Umsetzung: Reset-Block `settings.rs:458-459` (und etwaiger Zeit-Reset) entfernen — nach Create
  **nichts** zurücksetzen (Option 2), sodass Typ/Datum stehen bleiben. SSR-/Komponenten-Test:
  mehrfaches Anlegen ohne Toggle, Formulardaten bleiben erhalten.

## v2 Requirements (deferred → Folgemilestones)

### PDF-Export (EXP) → v2.2

- **EXP-01**: Täglicher automatischer PDF-Export der Folgewochen-Schichtpläne per WebDAV nach
  Nextcloud (interner Scheduler, PDF-Lib, WebDAV-Client, Secrets) — architektonisch eigenständig.

### Erweiterungen (deferred)

- **WST-06**: Bulk-KW-Status (mehrere Wochen auf einmal sperren/setzen).
- **WST-07**: Benachrichtigung bei Statuswechsel auf „Geplant" (Publish-Notification).
- **AVG-04**: AVG-Trend über mehrere Abrechnungsperioden hinweg.
- **AVG-05**: Konfigurierbare Absence-Exclusion-Kategorien für die Ø-Anwesenheit.

## Out of Scope

| Feature | Reason |
|---------|--------|
| PDF-Export → Nextcloud/WebDAV | Eigenständige Architektur (Scheduler, PDF, WebDAV, Deps, Secrets) → v2.2 |
| AVG-01 Snapshot-Persistenz / neuer `BillingPeriodValueType` | Read-only-Aggregat genügt; Persistenz erzwänge Snapshot-Bump ohne Mehrwert |
| Bulk-/Multi-Wochen-Statusaktionen | Nicht nötig für Kern-Use-Case; potenzielles v2 (WST-06) |
| Publish-Notification bei Statuswechsel | Kein Notification-Kanal im Scope; potenzielles v2 (WST-07) |
| Konfigurierbare AVG-Exclusion-Kategorien | v2.1 fixiert eine Regel in discuss-phase; Konfigurierbarkeit = v2 (AVG-05) |

## Traceability

Befüllt bei der Roadmap-Erstellung (2026-07-01) — jede Requirement → genau eine Phase.

| Requirement | Phase | Status |
|-------------|-------|--------|
| WST-01 | Phase 39 | Complete |
| WST-02 | Phase 39 | Complete |
| WST-05 | Phase 39 | Complete |
| WST-03 | Phase 40 | Complete |
| WST-04 | Phase 40 | Complete |
| AVG-01 | Phase 41 | Pending |
| AVG-02 | Phase 41 | Complete |
| AVG-03 | Phase 41 | Pending |
| SDF-01 | Phase 42 | Pending |

**Coverage:**

- v2.1 requirements: 9 total
- Mapped to phases: 9 (Phase 39: WST-01/02/05 · Phase 40: WST-03/04 · Phase 41: AVG-01/02/03 · Phase 42: SDF-01)
- Unmapped: 0 ✓

---
*Requirements defined: 2026-07-01*
*Last updated: 2026-07-01 — Traceability befüllt bei Roadmap-Erstellung (Phasen 39–42), Coverage 9/9 gemappt ✓*
