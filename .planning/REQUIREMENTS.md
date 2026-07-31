---
type: requirements
milestone: v2.6
milestone_name: Freiwillige-Stunden-Ausgleich für gedeckelte Mitarbeiter
last_updated: 2026-07-31
---

# v2.6 Requirements — Freiwillige-Stunden-Ausgleich für gedeckelte Mitarbeiter

**Milestone-Goal:** Gedeckelte Mitarbeiter (`cap_planned_hours_to_expected=true`),
die freiwillig mitarbeiten, sollen ihre Freiwilligen-Stunden automatisch oder
halbautomatisch als Ausgleich gegen ein negatives Stundenkonto einsetzen können.
HR sieht Soll/Ist-Konto Freiwilligkeit, wird proaktiv über Ausgleichsbedarf
informiert, und die Software erledigt die Doppelbuchung (heute manuell:
`−N VolunteerWork` / `+N ExtraWork`).

**Research:** `.planning/research/SUMMARY.md` (Konvergenz aus 4 parallelen
Researcher-Outputs mit 2 explizit offenen Discuss-Decisions: Snapshot-Bump
12→13 + Phase-Dekomposition-Wahl).

**Snapshot-Schema-Version:** aktuell 12. Der Bump 12→13 ist in **Phase 57** per
D-57-10 entschieden — Phase 56 hatte den „Nein"-Zweig gepinnt, dessen Beweislast
(byte-identischer Straddling-Snapshot) mit dem Filter-Fix entfallen ist
(Nachtrag an REB-AUTO-05).

**Scope:** Backend + Frontend voll enthalten. Fat Backend, Thin Client
durchgängig — alle Berechnungen (F1-Ist, F2-Soll, F4-Excess, F5-DANN-Werte)
im Backend.

---

## v2.6 Requirements

### Voluntary-Statistik (VOL-STAT — F1)

- [x] **VOL-STAT-01**: HR sieht im Mitarbeiter-Jahresreport (`/employees/:id`)
      pro ausgewähltem Jahr die Ø freiwillig geleisteten Stunden pro
      Vertragswoche als eigene Zeile neben der bestehenden AVG-01 Ø-Anwesenheits-
      Statistik. Zähler = Σ ExtraHours-Rows in Kategorie `VolunteerWork` im Jahr
      (ohne Rebooking-Marker); Nenner = Wochen mit gültigem `working_hours`-Eintrag
      der Person (exakte Denominator-Definition D-F1-01 in Phase-54-discuss-phase
      zu pinnen: strikt "contract-weeks in Jahr" vs. AVG-01-A-22-1 absence-adjusted).

- [x] **VOL-STAT-02**: Statistik ist HR-only. Für Nicht-HR-Rollen ist das Feld
      im DTO `Option<f32> = None` (API-Level-Redaction, Präzedenz VAC-OFFSET-01
      v1.8). Kein Redact im Frontend.

### Voluntary-Stundenkonto (VOL-ACCT — F2)

- [x] **VOL-ACCT-01**: HR sieht im Mitarbeiter-Jahresreport unter der bestehenden
      Zeile „Freiwillige Stunden" zusätzlich Soll- und Delta-Wert:
      Soll = Σ (`committed_voluntary(week) × Wochen-in-Kraft`) im Jahr;
      Ist = VOL-STAT-01-Zähler; Delta = Ist − Soll.

- [x] **VOL-ACCT-02**: Soll- und Delta-Anzeige sind HR-only (`Option<f32> = None`
      für Non-HR-DTOs, analog VOL-STAT-02). Der bestehende „Freiwillige Stunden"-
      Ist-Wert bleibt für alle Rollen sichtbar wie heute.

- [x] **VOL-ACCT-03**: Rebooking-Pair-ExtraHours (F3/F4/F5) verändern das
      **VOL-ACCT-01-Soll nicht**. Das Soll stammt aus `committed_voluntary` im
      Arbeitsvertrag und liest überhaupt keine `extra_hours` — eine Umbuchung darf
      die Freiwilligen-*Statistik* also nicht doppelt zählen. Property-Test:
      Rebooking über eine Woche → `VoluntaryStats::soll_total` unverändert.
      **Ausdrücklich NICHT Teil dieses Requirements sind die Balance und das
      VOL-STAT-01-Ist:** deren Veränderung durch eine Umbuchung ist gewollt und
      wird in VOL-ACCT-04 zugesagt.

      > **Historie:** Die Fassung bis Phase 56 forderte zusätzlich die Neutralität
      > von Ist und Balance. Wörtlich umgesetzt in Phase 55 (D-55-A1,
      > `source == Manual`-Filter an 4 Aggregations-Sites) machte das den gesamten
      > Übertrag wirkungslos: die Balance blieb nach einer Umbuchung unverändert
      > (empirisch `−53 → −53`), das Minus wanderte per Carryover ins Folgejahr.
      > Gemeint war der Schutz der **Freiwilligen-Statistik** gegen Doppelzählung,
      > nicht das Einfrieren der Balance. Neu gefasst in Phase 57 (D-57-09); die
      > Ausgleichswirkung liefert VOL-ACCT-04, den Deckel im Auto-Cron
      > REB-AUTO-06. Analyse: ROADMAP.md §Phase 57.

- [x] **VOL-ACCT-04**: Eine Umbuchung über `h` Stunden wirkt auf der Leseseite
      tatsächlich: nach dem Übertrag ist die Balance (bezahltes Stundenkonto) um
      `h` **höher** und das Ehrenamt-Ist (VOL-STAT-01) um `h` **niedriger**. Gilt
      für alle drei Schreib-Trigger (F3 manuell, F4 Auto-Cron, F5 HR-Approve) und
      rückwirkend für bereits bestehende Rebooking-Rows — es gibt keinen
      read-seitigen Stichtag-Guard (D-57-03). Akzeptanzbedingung ist das
      Referenz-Szenario D-57-02: Vertrag 5 h bezahlt + 5 h freiwillig pro Woche,
      Woche 1 = 3 h, Woche 2 = 17 h. Vor dem Übertrag verbucht Shifty 8 h bezahlt
      / 12 h freiwillig bei Balance −2; nach dem Übertrag von 2 h sind es
      10 h / 10 h bei Balance 0.

### Manuelle Umbuchung (REB-MANUAL — F3)

- [x] **REB-MANUAL-01**: HR kann im Mitarbeiter-Jahresreport eine Umbuchung
      Freiwillig ↔ Bezahlt anlegen (1-Klick-Aktion). Ergebnis: zwei atomar
      geschriebene ExtraHours-Rows (`−N` in Quell-Kategorie, `+N` in
      Ziel-Kategorie) plus ein `rebooking_batch(kind=manual)` mit einem
      `rebooking_batch_entry`. Alles in einer Transaktion; Rollback bei Fehler.

- [x] **REB-MANUAL-02**: Beide Richtungen werden unterstützt:
      `VolunteerWork → ExtraWork` (Standard-Fall Ausgleich) und
      `ExtraWork → VolunteerWork` (Korrektur überzahlter Zusagen).

- [x] **REB-MANUAL-03**: Frontend zeigt einen Umbuchungs-Modal mit Vorschau
      (Menge, Richtung, Woche); User bestätigt oder bricht ab. Kein Undo
      nach Bestätigung (Undo defer to v2.7+).

### Automatische Wochen-Umbuchung (REB-AUTO — F4)

- [x] **REB-AUTO-01**: Ein Wochen-Cron-Job (`VoluntaryRebookingScheduler`
      analog `PdfExportSchedulerImpl` v2.2) verarbeitet automatisch die
      Vorwoche (letzte abgeschlossene ISO-Woche via v2.5 `_iso_year`-Helper).
      Für jeden `cap_planned_hours_to_expected=true`-SalesPerson mit
      `Ist_der_Vorwoche > Soll_der_Vorwoche + committed_voluntary_der_Vorwoche`
      wird der Excess automatisch als Rebooking-Pair-ExtraHours geschrieben
      (`rebooking_batch(kind=auto_cron)`).

- [x] **REB-AUTO-02**: Ein admin-gesteuerter Stichtag-Toggle
      `voluntary_rebooking_auto_active_from` (analog HCFG-02 v1.7, SHC-04 v2.4)
      schützt historische Balance-Views. Bei `None` = deaktiviert (Default-
      Rollout, keine Regression); mit Datum = wirkt nur für ISO-Wochen ≥
      `active_from`. Legacy-Semantik pro Konsumkette rekonstruiert (Balance,
      F1-Ist, F2-Soll/Nutz, F4-Self-Guard).

- [x] **REB-AUTO-03**: Cron ist idempotent — UNIQUE-Constraint
      `(sales_person_id, iso_year, iso_week)` auf `rebooking_batch` verhindert
      Doppelverarbeitung bei Neustart/Backfill; `INSERT ... ON CONFLICT DO
      NOTHING`-Semantik. Zweiter Lauf über bereits verarbeitete Woche = no-op.

- [x] **REB-AUTO-04**: HR-Admin kann via REST-Endpoint einen einmaligen
      Backfill-Lauf über historische Wochen starten:
      `POST /admin/rebooking/backfill?from=YYYY-Www&dry_run={true|false}`.
      `dry_run=true` zeigt geplante Änderungen ohne DB-Write. Backfill schreibt
      `rebooking_batch(kind=auto_cron_backfill)`. HR-Admin-gated. Kein CLI-Tool.

- [x] **REB-AUTO-05**: Snapshot-Schema-Version-Bump-Entscheidung
      (12 → 13 oder unverändert 12) wird in Phase-56-discuss-phase gefällt.
      Kriterium: gilt F4-schreibende Rebooking-ExtraHours-Rows als „Input-Set-
      Change" im Sinne CLAUDE.md-Klausel? Beweislast beim „Nein"-Zweig =
      Straddling-Golden-Snapshot über pre-/post-/traversierende `active_from`-
      Periode, byte-identisch.

      > **Nachtrag (Phase 57, D-57-10):** Die in Phase 56 gefällte Entscheidung
      > lautete „kein Bump, Version bleibt 12" und stützte sich als Beweislast auf
      > einen byte-identischen Straddling-Golden-Snapshot (D-01-BEWEISLAST). Diese
      > Beweislage ist mit dem Filter-Fix aus Phase 57 entfallen: ein
      > Rebooking-Paar ist nicht mehr snapshot-neutral, es verschiebt Balance,
      > ExtraWork und Volunteer um `±h`. Nach der CLAUDE.md-Klausel „Change the
      > input set the computation reads from" ist der Bump damit Pflicht — Phase 57
      > revidiert den Ausgang auf 13 (D-57-10) und invertiert die Golden-Fixture
      > entsprechend (D-57-11). Dies ist ausschließlich eine Historien-Notiz:
      > REB-AUTO-05 bleibt inhaltlich und in der Traceability bei Phase 56.

- [x] **REB-AUTO-06**: Der Auto-Cron (F4) überträgt höchstens so viel, wie das
      bezahlte Stundenkonto aufnehmen kann. Formel:
      `min( max(ist − committed_voluntary, 0), max(−balance, 0) )`. Die Balance
      steigt durch die Automatik **nie über 0**; bei Balance ≥ 0 wird trotz
      Freiwilligen-Überschuss nichts gebucht. Balance-Quelle ist die kumulierte
      Jahres-Balance bis einschließlich der verarbeiteten Woche inklusive
      Vorjahres-Carryover (D-57-07), nicht die Balance zum Laufzeitpunkt. Der
      Deckel gilt für den Schreib-Pfad `run_for_week` **und** für den
      Dry-Run-Preview-Wert `BackfillPreviewItem.excess`. Die HR-Vorschlagsformel
      `proposed_hours = min(|balance|, ist)` (F5) ist ausdrücklich ausgenommen und
      bleibt **unverändert** — dass sie die Freiwilligen-Zusage unterschreiten
      darf, ist der Sinn des HR-Vorschlags (D-57-01). Gegenprobe: Woche 2 mit 20 h
      statt 17 h ergibt einen Wochen-Überschuss von 10; der Übertrag bleibt
      trotzdem 2 und die Balance landet auf 0.

### HR-Alert + Vorschlags-Modal (HR-ALERT — F5)

- [x] **HR-ALERT-01**: In der Employee-Overview (`/employees/`) zeigt eine
      dauerhafte Warnzeile pro betroffenem SalesPerson. Predicate:
      `cap_planned_hours_to_expected = true` AND `balance < 0` AND
      `voluntary_ist > 0`. Backend-berechnet, im DTO als
      `has_pending_rebooking: bool` + `pending_rebooking_id: Option<Uuid>`
      (Fat Backend — FE macht keine Rechnung).

- [x] **HR-ALERT-02**: Klick auf die Warnzeile öffnet ein Vorschlags-Modal mit
      IST- und DANN-Spalten für: Stundenkonto (Balance), Freiwillige Ist (VOL-
      STAT-01), Freiwilliges Soll (VOL-ACCT-01-Soll), Freiwilliges Delta
      (VOL-ACCT-01-Delta). Alle DANN-Werte backend-computed
      (`RebookingSuggestionTO`).

- [x] **HR-ALERT-03**: HR bestätigt oder lehnt ab. Bei Approve: Rebooking wie
      REB-MANUAL-01 plus `rebooking_batch(kind=hr_suggestion, state=approved)`.
      Bei Reject: `state=rejected` persistiert (bleibt sichtbar-vermerkt).
      State-conditional UPDATE (`WHERE state='pending'`, affected-rows == 1) für
      Concurrency (Pitfall 12).

- [x] **HR-ALERT-04**: Persistiertes `rebooking_batch(kind=hr_suggestion)`
      teilt die UNIQUE-Slot mit F4-`auto_cron` — Claim-on-Suggest-Strategie
      (Pitfall 7): sobald HR einen Vorschlag hat (`state=pending`), skipt F4-
      Cron diese Woche für diese Person. Vermeidet Stale-Vorschläge nach Cron-
      Race.

---

## Future Requirements (v2.7+)

Aus der v2.6-Anti-Feature-Liste (FEATURES.md), explizit vertagt:

- **VOL-SELF-01** — Employee-Self-Service-View des Freiwilligen-Kontos
  (bleibt HR-only in v2.6; Self-View bräuchte eigenen UX-Diskurs).

- **REB-NOTIF-01** — Notifications (Email/iCal/Push) auf F4-Completion oder
  F5-Alert; Shifty hat heute keine Notification-Infrastruktur.

- **REB-APPROVAL-01** — Multi-Role-Approval-Workflow auf F5-Batches
  (single-step approve/reject in v2.6 genügt).

- **REB-UNDO-01** — Undo/Rollback applied Batches (Schema unterstützt
  spätere Extension via stable `batch.id`).

- **REB-HISTORY-01** — UI zur Visualisierung der Rebooking-Batch-Historie
  pro SalesPerson (Audit-Trail in DB reicht für v2.6).

- **REB-ALERT-EXPANSION-01** — Alerts für nicht-gedeckelte Mitarbeiter mit
  Voluntary-Überschuss (nicht Zielgruppe).

## Out of Scope

Explizit ausgeschlossen mit Begründung:

- **CLI-Backfill** — Backfill läuft als HR-gated REST-Endpoint (REB-AUTO-04),
  nicht als `clap`-CLI. Grund: `shifty_bin` parst heute 0 CLI-Args; REST
  wiederverwendet Auth + Audit-Trail ohne Rebuild-Deploy-Zyklus.

- **SSE/WebSocket für Live-Alerts** — F5-Alerts kommen via existierendem
  REST-Poll-Muster. Grund: keine bestehende SSE/WS-Infrastruktur, Alert-
  Frequenz niedrig.

- **Neuer FE-State-Store** — Bestehende Loader/State-Konvention reicht.
- **Neuer `BillingPeriodValueType`** — Rebooking schreibt existierende
  Kategorien (`VolunteerWork` / `ExtraWork`), kein neuer persistierter
  value_type. (Snapshot-Version-Bump-Frage siehe REB-AUTO-05.)

- **Migration alter ExtraHours zu Batches** — historische ExtraHours (vor
  v2.6) bleiben unangetastet; Backfill erzeugt neue Batches ex-post ohne
  Bestandsdaten-Umzug.

## Traceability

19 v1-Requirements → 4 Phasen (ARCHITECTURE-C 3-Phasen-Baseline aus Research
SUMMARY, dokumentiert in ROADMAP.md; Phase 57 als Korrektur-Phase ergänzt).
Coverage: 19/19.

| Requirement    | Kategorie   | Phase    | Status  |
|----------------|-------------|----------|---------|
| VOL-STAT-01    | F1 Statistik | Phase 54 | Pending |
| VOL-STAT-02    | F1 Statistik | Phase 54 | Pending |
| VOL-ACCT-01    | F2 Konto    | Phase 54 | Pending |
| VOL-ACCT-02    | F2 Konto    | Phase 54 | Pending |
| VOL-ACCT-03    | F2 Konto    | Phase 57 | Pending |
| VOL-ACCT-04    | F2 Konto    | Phase 57 | Pending |
| REB-MANUAL-01  | F3 Manuell  | Phase 55 | Pending |
| REB-MANUAL-02  | F3 Manuell  | Phase 55 | Pending |
| REB-MANUAL-03  | F3 Manuell  | Phase 55 | Pending |
| HR-ALERT-01    | F5 Alert    | Phase 55 | Pending |
| HR-ALERT-02    | F5 Alert    | Phase 55 | Pending |
| HR-ALERT-03    | F5 Alert    | Phase 55 | Pending |
| HR-ALERT-04    | F5 Alert    | Phase 55 | Pending |
| REB-AUTO-01    | F4 Cron     | Phase 56 | Pending |
| REB-AUTO-02    | F4 Cron     | Phase 56 | Pending |
| REB-AUTO-03    | F4 Cron     | Phase 56 | Pending |
| REB-AUTO-04    | F4 Cron     | Phase 56 | Pending |
| REB-AUTO-05    | F4 Cron     | Phase 56 | Pending |
| REB-AUTO-06    | F4 Cron     | Phase 57 | Pending |

**Per-Phase Coverage:**

- Phase 54 (Data-Model + F1 + F2): 4 Requirements
  (VOL-STAT-01, VOL-STAT-02, VOL-ACCT-01, VOL-ACCT-02)

- Phase 55 (F3 + F5 Manuelle Umbuchung + HR-Alert-Modal): 7 Requirements
  (REB-MANUAL-01/02/03, HR-ALERT-01/02/03/04)

- Phase 56 (F4 Wochen-Cron + Backfill): 5 Requirements
  (REB-AUTO-01/02/03/04/05)

- Phase 57 (Rebooking-Wirksamkeit): 3 Requirements
  (VOL-ACCT-03, VOL-ACCT-04, REB-AUTO-06)

**Total:** 19/19 mapped, no orphans, no duplicates.
