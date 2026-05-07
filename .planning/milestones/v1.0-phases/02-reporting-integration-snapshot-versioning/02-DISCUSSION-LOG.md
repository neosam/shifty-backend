# Phase 2: Reporting Integration & Snapshot Versioning - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-01
**Phase:** 02-reporting-integration-snapshot-versioning
**Areas discussed:** BUrlG §9 / Sick-overlaps-Vacation, UnpaidLeave Snapshot-Mapping, Feature-Flag-Mechanik, Locking-Test-Mechanik

---

## BUrlG §9 / Sick-overlaps-Vacation

### Q1: Auflösungs-Policy bei Vacation∩SickLeave am selben Tag

| Option | Description | Selected |
|--------|-------------|----------|
| Sick gewinnt (BUrlG §9-konform) | An Tagen mit beiden Kategorien wird ausschließlich SickLeave angerechnet; Vacation produziert 0. Entspricht BUrlG §9 — Krankheit im Urlaub bricht den Urlaub ab. | ✓ |
| Vacation gewinnt | Erste angelegte/Vacation-priorisiert; Sick wird ignoriert. Einfacher, **nicht** §9-konform. | |
| Beide zählen (doppelt) | Doppelte Anrechnung. Verstößt gegen §9; produziert Stunden-Drift gegen ExtraHours-Quelle (Phase-4-Gate würde fehlschlagen). | |

**User's choice:** Sick gewinnt (BUrlG §9-konform)
**Notes:** Klare gesetzliche Vorgabe; deterministisch implementierbar via Per-Tag-Iteration mit Prioritäts-Reihenfolge.

### Q2: Wo lebt die Cross-Category-Auflösung?

| Option | Description | Selected |
|--------|-------------|----------|
| In `AbsenceService::derive_hours_for_range` | Service nimmt alle 3 Kategorien gleichzeitig, gibt bereits konflikt-aufgelöste Tagesliste zurück. Single source of truth; Phase-4-Migration kann dieselbe Logik fahren. | ✓ |
| Im `ReportingService` | Service liefert per-Kategorie-roh; Caller orchestriert pro-Kategorie-Aufrufe. Drift-Risiko zwischen Reporting- und Phase-4-Gate-Pfad. | |
| Im Caller mit shared helper | Mittelweg: Service liefert per-Kategorie-roh; ein Helper macht die Auflösung; beide rufen ihn. Keine Duplizierung, aber AbsenceService-API bleibt simpel. | |

**User's choice:** In AbsenceService
**Notes:** Phase 4 fährt dieselbe Logik für die Migrations-Validierung — keine Duplizierung erwünscht.

### Q3: Prioritäts-Reihenfolge auch für UnpaidLeave?

| Option | Description | Selected |
|--------|-------------|----------|
| Deterministische Reihenfolge: SickLeave > Vacation > UnpaidLeave | Über alle 3 Pair-Kombinationen. Höchstpriorisierte Kategorie gewinnt pro Tag. Re-runnable für Phase-4-Gate. | ✓ |
| Nur Vacation∩SickLeave wird aufgelöst | UnpaidLeave-Kombinationen sind 'undefined behavior' — Plan-Phase entscheidet. Risiko: Phase-4-Gate kann nicht eindeutig validieren. | |
| Konflikt = Validation-Error im Service | Cross-Category-Overlap wird grundsätzlich abgelehnt (außer §9-Pair). Würde D-12 aus Phase 1 ändern. | |

**User's choice:** Deterministische Reihenfolge SickLeave > Vacation > UnpaidLeave
**Notes:** UnpaidLeave verliert immer — wenn Mitarbeiter stattdessen Urlaub nimmt, gewinnt Vacation; wenn er krank wird, gewinnt Sick.

---

## UnpaidLeave Snapshot-Mapping

### Q4: Behandlung von UnpaidLeave im Snapshot v3

| Option | Description | Selected |
|--------|-------------|----------|
| Neuer `BillingPeriodValueType::UnpaidLeave` | Eigene Enum-Variante + eigene Snapshot-Spalte. Phase-4-Gate kann pro Kategorie inkl. UnpaidLeave validieren (MIG-03). Schließt bestehende Snapshot-Lücke. | ✓ |
| UnpaidLeave bleibt nur live (kein value_type) | Persistierung nur indirekt via Balance/Overall. Phase-4-Gate weniger präzise; HR-Audit kann UnpaidLeave nicht aus Snapshot rekonstruieren. | |
| UnpaidLeave mit Vacation zusammenlegen | Verliert Differenzierung zwischen bezahltem und unbezahltem Urlaub. Kollidiert mit `EmployeeReport`-Trennung. | |

**User's choice:** Neuer `BillingPeriodValueType::UnpaidLeave`
**Notes:** Bestehende Snapshot-Lücke wird mit dem Phase-2-Bump 2→3 sauber geschlossen.

### Q5: v2-Snapshot-Read mit fehlender unpaid_leave-Spalte

| Option | Description | Selected |
|--------|-------------|----------|
| Fehlend = 0.0 | Reader bleibt versions-agnostic; semantisch korrekt 'unbekannt = 0' für historische Snapshots. | ✓ |
| Versions-aware Reader: bei v2 explizit `Option<f32>` | UI kann 'fehlend vs. 0' unterscheiden (Audit-Anzeige 'Spalte gab es damals nicht'). Mehr Code-Surface. | |
| Backfill auf Read aus historischen ExtraHours | Audit-treu, aber Performance-/Konsistenz-Risiko; konfligiert mit 'materialize-once'-Snapshot-Design. | |

**User's choice:** Fehlend = 0.0
**Notes:** Trivialer Read-Pfad; SC-5 ('v2-Snapshots bleiben lesbar') ist erfüllt.

---

## Feature-Flag-Mechanik

### Q6: Mechanismus für `absence.range_source_active`

| Option | Description | Selected |
|--------|-------------|----------|
| Bestehender `ToggleService` reuse | Toggle-Name `absence_range_source_active` in bestehender `toggle`-Tabelle; Phase 4 flippt via `enable_toggle`. Kein neuer Code. | |
| Neue Tabelle `feature_flag` | Separater Store für Architektur-/Migrations-Flags. Semantisch getrennt von User-Toggles. | ✓ |
| Env-Var + DB-Override (zwei-stufig) | Default via env, DB-Override schlägt env. Komplexer; Mehrwert v.a. für Dev/Test. | |

**User's choice:** Neue Tabelle `feature_flag`
**Notes:** Bewusste semantische Trennung von User-Toggles (`toggle`-Tabelle) und Architektur-Flags (`feature_flag`-Tabelle).

### Q7: Schema des `feature_flag`-Stores

| Option | Description | Selected |
|--------|-------------|----------|
| Generisches Key-Value: `feature_flag(key TEXT PK, enabled BOOLEAN, ...)` | Future-proof; neue Flags brauchen keine neue Migration. | ✓ |
| Spalte pro Flag in Singleton-Tabelle: `app_settings(absence_range_source_active BOOLEAN, ...)` | Typsicherer (SQLx-Spalten-Compile-Time-Check); jedes neue Flag = Migration. | |
| Dedizierte Tabelle nur für diesen Flag: `absence_settings(...)` | Domain-spezifisch; bei zukünftigen Flags wieder neue Tabellen. | |

**User's choice:** Generisches Key-Value
**Notes:** Future-proof; analog zum bestehenden `toggle`-Schema, aber semantisch separat.

### Q8: Service- und Permission-Surface

| Option | Description | Selected |
|--------|-------------|----------|
| Neuer `FeatureFlagService` mit eigenem Privileg `feature_flag_admin` | Eigenes Trait + Mock + DI; eigenes Privileg strenger als `toggle_admin`. REST später. | ✓ |
| Internes Read-Only Helper-Modul | Simples Helper ohne Service-Trait, kein Permission, kein REST. Kein DI-Mock; Test-Setup muss DB-direkt seeden. | |
| Service-Trait + reuse `HR_PRIVILEGE` | Eigener Service ohne neues Privileg. Architektur-Flags wären für alle HR-User flippbar — möglicherweise zu permissiv. | |

**User's choice:** Neuer `FeatureFlagService` mit eigenem Privileg `feature_flag_admin`
**Notes:** Architektur-Flags brauchen restringiertere Permission als User-Toggles.

---

## Locking-Test-Mechanik

### Klärungsphase

Der User stellte eine wichtige Klärungsfrage: *"Geht es um Tests im Sinne von QA oder um den Test, ob sich bei der Migration das Stundenkonto verändert?"*

Antwort: Es ist ein **QA-Test (Build-Zeit)**, nicht der Phase-4-Cutover-Gate. Der Locking-Test schützt die Snapshot-Versions-Disziplin: bei jeder stillen Logik-Änderung im Reporting-Pfad bricht `cargo test`, sodass der Entwickler bewusst entscheiden muss, ob Bump nötig ist oder ein Bug vorliegt.

Der User bestätigte das Verständnis und wählte dann:

### Q9: Locking-Test-Mechanik

| Option | Description | Selected |
|--------|-------------|----------|
| Pin-Map mit konkreten erwarteten Werten | Deterministische Fixture; pro `BillingPeriodValueType` ein `assert_eq!` auf erwarteten Wert. Bei Drift bricht `cargo test`. Kein neues Tooling. | |
| Insta-Snapshot (Yaml-Datei im VCS) | Neue Dependency `insta`; Yaml-Diff in CI; `cargo insta accept` zwingt bewusste Bestätigung. | |
| Hybrid: Pin-Map + Compiler-Exhaustive-Match | Zwei Tests: Pin-Map deckt Berechnungs-Drift; `match`-Exhaustivität deckt Schema-Surface-Drift (neue Enum-Variante zwingt zur Test-Anpassung). | ✓ |

**User's choice:** Hybrid: Pin-Map + Compiler-Exhaustive-Match
**Notes:** Maximaler Schutz; Berechnungs-Drift UND Schema-Surface-Drift werden separat erkannt.

### Q10: Pin-Map-Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Alle 12 `BillingPeriodValueType`-Varianten | Universeller Drift-Schutz; auch unrelated Refactors brechen den Test. Intended pain. | ✓ |
| Nur durch Phase 2 berührte: VacationHours, SickLeave, UnpaidLeave, Balance, Overall, VacationDays | Schmaler Pin; weniger Wartung. Risiko: zukünftige Drift in anderen value_types wird nicht erkannt. | |
| Nur die direkt betroffenen: VacationHours, SickLeave, UnpaidLeave | Minimalpin; minimaler Wartungsschmerz. Kein Schutz für Balance/Overall. | |

**User's choice:** Alle 12 Varianten
**Notes:** Der Locking-Test soll universeller Snapshot-Drift-Schutz sein, nicht nur Phase-2-spezifisch.

---

## Claude's Discretion

Folgende Punkte wurden Plan-Phase überlassen (siehe CONTEXT.md `<decisions>` → "Claude's Discretion"):

- **C-Phase2-01:** `derive_hours_for_range`-Return-Type-Detail (Vorschlag: `BTreeMap<Date, ResolvedAbsence>`).
- **C-Phase2-02:** Konkrete Wochenend-Logik der Feiertags-0-Auflösung (Pragma: "Vertragsstunden des Tages = 0 ⇒ Absence-Stunden des Tages = 0").
- **C-Phase2-03:** `FeatureFlagDao`-Surface-Breite (schmal vs. voll).
- **C-Phase2-04:** Konkrete Pin-Map-Fixture-Werte (Vorschlag in `<specifics>`).
- **C-Phase2-05:** DI-Reihenfolge für `FeatureFlagService` in `main.rs`.
- **C-Phase2-06:** Naming des Toggle-Keys (`absence_range_source_active` als snake_case-Storage-Key).

## Deferred Ideas

- REST-Endpoints für `FeatureFlagService` — später bei Bedarf.
- `feature_flag`-Audit-Trail (wer hat wann geflippt) — bei Bedarf später.
- Insta-Snapshot-Tooling — bewusst nicht eingeführt; falls Snapshot-Surface stark wächst, in Future-Phase nachziehen.
- Phase-4-Cutover-Gate (MIG-02/MIG-03) — anderer Mechanismus, Phase 4. Phase 2 stellt die Logik-Surface bereit.
- Atomares Feature-Flag-Flippen in derselben Tx wie MIG-01/MIG-04 — Phase 4.
- Carryover-Refresh nach Flag-Flip — Phase 4 (MIG-04).
- `derive_hours_for_range` für andere Konsumenten als Reporting (z.B. Phase 3 Booking-Konflikt) — Phase 3 nutzt `find_overlapping`-Style-API; kein Konflikt.
