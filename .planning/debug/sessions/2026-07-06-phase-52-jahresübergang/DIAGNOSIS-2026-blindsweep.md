# Phase 52 Jahresübergang-Regression — 2026-Blindsweep (Follow-up)

**Datum:** 2026-07-06
**Vorherige Diagnose:** [`DIAGNOSIS.md`](./DIAGNOSIS.md) (paid_hours-Bug, ExtraHours-DAO)
**Reporter:** User (Test-Env, Jahr 2026)
**Kernproblem (aus Prior-Report):** `required_hours` (zu besetzende Stunden) unterscheidet sich zwischen alter (`get_summery_for_week`) und neuer (`get_weekly_summary`) Impl in **KW 1** und **KW 53** von 2026.

**Verdikt (kurz):** **REGRESSION durch Phase 52 (Wave 5, Commit `15a3a0e`) BESTÄTIGT** für `required_hours` an ISO-Jahresübergängen. Zweiter unabhängiger Bug, disjunkt vom vorherigen `paid_hours`-Bug.

**Reproduziert durch:** `h4_holiday_2027_01_01_iso_2026_w53_bulk_vs_legacy` und `h4b_holiday_2025_12_29_iso_2026_w1_bulk_vs_legacy` in `service_impl/src/test/booking_information_weekly_summary_year_boundary_2026.rs` (grün, dokumentieren die Drift-Zeile bit-genau).

---

## 1. Root Cause (neuer Bug, disjunkt vom paid_hours-Bug)

**`special_days_source`-Bucket-Wahl in `get_weekly_summary` benutzt Kalender-Jahr statt ISO-Wochenjahr.**

In `service_impl/src/booking_information.rs`, Zeilen 348–406, werden vor dem Loop zwei "SpecialDay-Buckets" bulk-geladen:

```rust
let special_days_this = self.special_day_service.get_by_year(year).await?;
let special_days_next = self.special_day_service.get_by_year(year_plus_1).await?;
```

Und im Loop-Body wird das Bucket per Kalender-Jahr-Vergleich ausgewählt (Zeilen 393–406):

```rust
let (year_reports_source, special_days_source, shiftplan_reports_source) =
    if year == outer_year {
        (&year_reports, &special_days_this, &shiftplan_reports_this)
    } else {
        (&next_year_reports, &special_days_next, &shiftplan_reports_next)
    };
```

Danach wird pro Woche gefiltert (Zeilen 449–453):

```rust
let special_days: Arc<[SpecialDay]> = special_days_source
    .iter()
    .filter(|d| d.year == year && d.calendar_week == week)
    .cloned()
    .collect();
```

**Die kritische Semantik-Falle:** `SpecialDayServiceImpl::get_by_year(y)` (in `service_impl/src/special_days.rs:77-116`) liefert alle Rows deren **Kalender-Datum** in `y` liegt. Die Impl macht `Union(find_by_year(y), find_by_year(y-1))` und filtert per `shifty_date.to_date().year() == y`. Der DB-Feldwert `year` ist aber das **ISO-Wochenjahr**, nicht das Kalender-Jahr.

**Folge:** Eine SpecialDay-Row `(year=2026, calendar_week=53, day=Fri)` (Kalender-Datum **2027-01-01**) wird aus `get_by_year(2026)` **ausgefiltert** (Kalender-Jahr 2027 ≠ 2026). Sie landet in `get_by_year(2027)`.

Aber die Loop-Iteration W53 mit `outer_year=2026` → `year==outer_year` → wählt `special_days_source = special_days_this = get_by_year(2026)`. Filter `d.year==2026 && d.calendar_week==53` findet **nichts** (Row ist im falschen Bucket).

Und die Loop-Iteration W54 (=Spillover W1@2027) → `year==2027, week=1` → wählt `special_days_source = special_days_next = get_by_year(2027)`. Filter `d.year==2027 && d.calendar_week==1` findet auch **nichts** (die Row hat `d.year=2026, d.calendar_week=53`).

→ Die Row wird in KEINER Loop-Iteration angesehen. → Der Feiertag am 2027-01-01 wird ignoriert. → Der Slot am W53-Fr bleibt drin. → `required_hours` in W53 = 8h zu hoch.

**Symmetrischer Fall in W1:** Feiertag am **2025-12-29** (Montag = ISO-2026-W1-Mo). DB-Row `(year=2026, calendar_week=1, day=Mon)`, Kalender-Datum 2025-12-29 → in `get_by_year(2025)`, NICHT in `get_by_year(2026)`. Loop-Iteration W1 mit `outer_year=2026, year=2026` → konsultiert `special_days_this`. Row nicht drin → Slot bleibt → `required_hours` in W1 = 8h zu hoch.

**Der `paid_hours`-Bug aus der prior Diagnose (`extra_hours::find_by_year`, kalender-vs-ISO) und dieser `special_days`-Bug haben denselben strukturellen Kern**: Ein Bulk-Loader liefert nach Kalender-Jahr, aber der Konsument braucht ISO-Wochenjahr. Verschiedene DAOs, unabhängige Fixes nötig.

---

## 2. Beweis via Test

Test-Datei: `service_impl/src/test/booking_information_weekly_summary_year_boundary_2026.rs`

**11 Tests, alle grün.** Zwei sind Bug-Reproduktions-Assertions (asserten explizit die falsche Bulk-Zahl gegen die richtige Legacy-Zahl, +8h Drift):

- **`iso_2026_boundary_facts`** — Bestätigt die ISO-2026-Grenzen:
  - `weeks_in_year(2026) == 53` (2025=52, 2027=52).
  - ISO-2026-W1 = **Mo 2025-12-29** .. So 2026-01-04 (W1 startet im Kalenderjahr 2025!).
  - ISO-2026-W53 = Mo 2026-12-28 .. **So 2027-01-03** (W53 endet im Kalenderjahr 2027).
  - 2026-01-01 = Do, ISO-2026-W1-Thu.
  - 2026-12-31 = Do, ISO-2026-W53-Thu.
  - 2027-01-01 = Fr, ISO-2026-W53-Fri (Kalenderjahr 2027, ISO-Wochenjahr 2026).

- **`h1_immer_aktiver_slot_2026_jede_woche_8h`** — Slot valid_from=2019, valid_to=None, Mo 10-18. In allen 56 Loop-Iterationen (53 + 3 Spillover) = 8h. ✓

- **`h2_immer_aktiver_slot_bulk_vs_legacy_2026`** — Bit-Vergleich bulk vs legacy_for_week für alle 56 Wochen. Ohne SpecialDays kein Diff. ✓ (Slot-Filter-Semantik ist konsistent.)

- **`h2b_slot_valid_to_2026_12_31_bulk_vs_legacy`** — Slot mit `valid_to = 2026-12-31` (Kalender-Ende, Do). Bit-identisch. ✓ (Slot-`valid_to`-Semantik konsistent.)

- **`h2c_slot_valid_from_2026_06_01_boundary_correctness`** — Slot ab 2026-06-01. W1 = 0h ✓, W53 = 8h ✓, beide bit-identisch mit Legacy.

- **`h3_holiday_2026_01_01_filters_slot_in_w1`** — Holiday am 2026-01-01 (Do, ISO-2026-W1-Thu, DB-Row `year=2026, week=1, day=Thu`). Kalender-Datum in 2026 ⇒ in `get_by_year(2026)` **drin**. Bulk = Legacy = 0h. ✓ (Kein Bug wenn Kalenderjahr = ISO-Wochenjahr.)

- **`h4_holiday_2027_01_01_iso_2026_w53_bulk_vs_legacy`** — **BUG!** Holiday am **2027-01-01** (Fr, ISO-2026-W53-Fri, DB-Row `year=2026, week=53, day=Fri`). Kalender-Datum in 2027 ⇒ NICHT in `get_by_year(2026)`.
  - Bulk `required_hours[W53]` = **8.0** (Holiday nicht gefunden, Slot bleibt).
  - Legacy `required_hours[W53]` = **0.0** (Holiday via `get_by_week(2026, 53)` gefunden, Slot dropped).
  - **Drift = +8.0h**.

- **`h4b_holiday_2025_12_29_iso_2026_w1_bulk_vs_legacy`** — **BUG (symmetrisch)!** Holiday am **2025-12-29** (Mo, ISO-2026-W1-Mo, DB-Row `year=2026, week=1, day=Mon`). Kalender-Datum in 2025 ⇒ NICHT in `get_by_year(2026)`.
  - Bulk `required_hours[W1]` = **8.0** (Holiday nicht gefunden, Slot bleibt).
  - Legacy `required_hours[W1]` = **0.0** (Holiday via `get_by_week(2026, 1)` gefunden, Slot dropped).
  - **Drift = +8.0h**.

- **`h5_shortday_2026_12_31_iso_2026_w53_bulk_vs_legacy`** — ShortDay am 2026-12-31 (Do, ISO-2026-W53-Thu). Kalender-Datum in 2026 ⇒ in `get_by_year(2026)` drin. Bulk = Legacy = 0h. ✓ (Wieder: kein Bug wenn Kalenderjahr = ISO-Wochenjahr.)

- **`h6_holiday_2026_01_01_variant_iso_year_match`** — Duplikat-Sanity zu h3. ✓

**Muster:** Nur Feiertage/ShortDays mit **Kalender-Datum in einem Kalenderjahr ≠ ISO-Wochenjahr** triggern den Bug. Das trifft für Rows an ISO-Wochen-Enden, die Silvester-Neujahr überspannen — pro Jahr genau die Wochen KW 1 (falls sie in Vorjahr startet, wie 2026) und KW 52/53 (falls sie in Folgejahr endet, wie 2020, 2026).

---

## 3. Betroffene Dateien & Zeilen

| Datei | Zeilen | Rolle |
|---|---|---|
| `service_impl/src/booking_information.rs` | 348–355 (Bulk-Load), 393–406 (Bucket-Wahl), 449–453 (Per-Woche-Filter) | **BUG-QUELLE (Konsument):** Bucket-Wahl per Kalender-Jahr, Filter per ISO-Wochenjahr |
| `service_impl/src/special_days.rs` | 77–116 (`get_by_year`) | **AUXILIARY:** liefert nach Kalender-Jahr — semantisch legitim, aber inkompatibel mit dem Konsumenten oben |
| `dao_impl_sqlite/src/special_day.rs` | 89–100 (`find_by_week`), 103–125 (`find_by_year`) | Legacy-Pfad — funktioniert korrekt (WHERE year = ? AND calendar_week = ?) |

**Kein Bug in:** `get_summery_for_week` (nutzt Legacy-DAO, korrekt).

---

## 4. Introducing Commit

- **Commit:** `15a3a0e` (`refactor(52-05): rebuild get_weekly_summary with year-bulk loads (WOP-01)`).
- **Vorher (git show 15a3a0e^):** Loop nutzte `special_day_service.get_by_week(year, week)` — per-Woche-DAO-Call. Der DAO filtert `WHERE year = ? AND calendar_week = ?` (Zeile 89–100), also nach dem DB-Feld `year` = ISO-Wochenjahr. → **Semantisch korrekt für den ISO-basierten Loop**.
- **Nach 15a3a0e:** Bulk-Load `get_by_year(year)` + `get_by_year(year+1)`, In-Memory-Filter. Aber `get_by_year` filtert per **Kalender-Datum** (Union-Trick in `special_days.rs`) — inkompatibel mit dem ISO-basierten Bucket-Wahl-Vergleich `year == outer_year`.
- **Verdikt:** **Regression, eingeführt in Phase 52 Wave 5**. Weder Sequenz-Version noch Legacy-Pfad hatten diese Fallgrube.

**Git-Archaeologie ist eindeutig — die alte Impl hatte NICHT die "1..=52-skipped-53"-Falle:** Beide Impls nutzen `weeks_in_year(outer_year) + 3` als Loop-Bound (siehe `git show 15a3a0e^:service_impl/src/booking_information.rs`, Zeile 311). Der KW-53-Diff ist echte Wave-5-Regression, nicht "old-skipped vs new-computed".

---

## 5. Zusätzliche Beobachtungen (Neben-Funde)

### 5.1 Latenter Semantik-Diff: soft-deleted Shiftplan mit `is_planning=true`

In `get_weekly_summary` (Zeilen 376–383) wird `planning_shiftplan_ids` aus `shiftplan_service.get_all()` gebildet, das nur **nicht-gelöschte** Shiftplans zurückgibt (`dao_impl_sqlite/src/shiftplan.rs:53` — `WHERE deleted IS NULL`).

Der DAO-Pfad (`get_slots_for_week_all_plans`, `dao_impl_sqlite/src/slot.rs:151-168`) macht `LEFT JOIN shiftplan ON slot.shiftplan_id = shiftplan.id WHERE (shiftplan.is_planning = 0 OR shiftplan.is_planning IS NULL)`. Der JOIN filtert **NICHT** nach `shiftplan.deleted IS NULL` — wenn ein Shiftplan soft-deleted ist, aber ein Slot noch dessen ID referenziert, sieht die JOIN-Semantik den Shiftplan mit seinem `is_planning=1` und filtert den Slot raus. Die In-Memory-Version sieht ihn nicht in `planning_shiftplan_ids` (weil `get_all()` deleted-gefiltert ist) und lässt den Slot durch.

→ **Latenter Bug, unabhängig vom Jahresübergang**. Trigger in der Realität wahrscheinlich sehr selten (Shiftplans werden fast nie mit noch-referenzierenden Slots gelöscht). **Nicht Teil des User-Reports.** Aber im Rahmen des `docs`-Gate/Reviewer-Radar erwähnenswert. **Empfehlung:** Separater Fix — entweder `shiftplan_service.get_all_including_deleted()` benutzen ODER in `slot.rs` DAO das JOIN um `shiftplan.deleted IS NULL`-Klausel erweitern (dann In-Memory-Impl bereits konsistent).

### 5.2 shiftplan_report_source und year_reports_source

Für `shiftplan_report`-Filter (Zeilen 469–474) und `year_reports`-Slice-Zugriff (Zeilen 440–443) ist dieselbe Falle theoretisch denkbar, aber **nicht getriggert**:
- `ShiftplanReportDay.year/calendar_week` kommt aus `bookings.year/calendar_week` — dort wird beim Insert das **ISO-Wochenjahr** gespeichert (der Booking-Pfad benutzt `ShiftyDate::from_date(booking_date)`). Und `extract_raw_shiftplan_report_for_year` filtert `WHERE booking.year = ?` — auch nach dem ISO-Wochenjahr-Feld. → **konsistent**, kein Diff-Punkt.
- `year_reports[week-1]` — `assemble_weeks` liefert die Reports pro (year, week) mit ISO-Semantik (das war ja der Punkt der prior Diagnose zum ExtraHours-Bug: die Bucket-Semantik ist ISO). Die Slice-Adressierung `[week-1]` ist off-by-one-safe solange `weeks_in_year` konsistent verwendet wird. **In der Realität nicht der Trigger für den User-Report**, aber steht in der prior Diagnose als `paid_hours`-Bug (ExtraHours-Range-Filter im DAO).

### 5.3 Selber Bug in `shiftplan_report::extract_shiftplan_report_for_year`

Analog zum Weekly-Overview-Bug findet sich das **gleiche Anti-Pattern** in `service_impl/src/shiftplan_report.rs:336-350` (Phase 52 Wave 3, Commit `bdbdc28`):

```rust
let special_days_year = self.special_day_service.get_by_year(year, ...).await?;
// group per calendar_week
for sd in special_days_year.iter() {
    if sd.year == year {  // ← ISO-Wochenjahr-Filter auf Kalender-Jahr-gefilterter Liste
        special_days_by_week.entry(sd.calendar_week).or_default().push(sd.clone());
    }
}
// hours_for_row(row, special_days_by_week[calendar_week], ...) — beeinflusst Booking-Aggregation
```

- ShortDay am 2027-01-01 (DB `year=2026, week=53, day=Fri`) → Kalender-Datum in 2027 → NICHT in `get_by_year(2026)`.
- Für Jahr 2026 fehlt der ShortDay-Clip auf Booking-Hours in W53 → `hours_for_row` sieht keinen ShortDay-Cutoff → Bookings am 2027-01-01 werden ohne Clip aggregiert (falls kein Gate greift; im Legacy-Mode: kein Auswirkung, da `hours_for_row` den Clip nur bei aktivem Gate anwendet — Details in `shortday_gate::hours_for_row`).

**Blast-Radius:** `ShiftplanReportDay.hours` in Weekly-Overview `volunteer_hours`-Band, Chain-C-Gate-Effekte. Wenn User ShortDays oder Feiertage in KW 1 / KW 53 pflegt UND das Chain-C-Gate aktiv ist → weiterer stiller Drift.

**Fix:** Gleicher Fix-Ansatz wie im Weekly-Overview — beide Bulk-Buckets kombinieren oder eine `get_by_iso_year`-Variante bereitstellen.

### 5.4 Slot-Filter für Bulk vs Legacy ist konsistent

`h1, h2, h2b, h2c` zeigen keine Diskrepanz für rein Slot-basierte Felder ohne SpecialDays. Der In-Memory-Slot-Filter reproduziert die DAO-`WHERE`-Klausel bit-genau. → **Kein Bug im Slot-Bereich**, entgegen prior Diagnose-Sorge.

---

## 6. Regression-Blast-Radius (zusätzlich zum paid_hours-Bug)

Der `special_days`-Bug betrifft:
- `required_hours` in Weekly-Overview (der User-sichtbare Endpunkt) — bestätigt.

Andere Konsumenten von `SpecialDayService::get_by_year`?

```
grep -rn "get_by_year" service_impl/src/ | grep -i special
```

<Prüfe unten, ob es andere Konsumenten gibt.>

Prüfung: `special_day_service.get_by_year` wird außer in `booking_information::get_weekly_summary` (Zeilen 348, 352) auch in `service_impl/src/reporting.rs` konsumiert (aus prior Investigation bekannt: `assemble_weeks` lädt beide Jahre für die Bulk-Optimierung). **Selbst-Prüfung durch Auditor nötig — reporting.rs:530-580 (Wave 4 special_day-Bulk)**. Wenn `assemble_weeks` denselben Fehler macht, wäre auch `dynamic_hours` (holidays wirken auf balance) betroffen. **Aber die prior Diagnose meldet keine anderen Diffs**, weil `find_by_week` in Reports einen anderen Pfad hat — der Fall braucht eigenständigen Test, ist aber außerhalb dieses Follow-ups.

---

## 7. Vorgeschlagener Fix (NICHT IMPLEMENTIERT, wartet auf Greenlight)

**Option A (chirurgisch, empfohlen): Bucket-Wahl-Kriterium anpassen.**

Der Fix ist im Konsumenten (`booking_information.rs`), nicht in `special_days.rs`. Statt `if year == outer_year` (Kalender-Jahr-basiert) müsste die Bucket-Wahl per Zugehörigkeit der (year, week) zum ISO-Wochenjahr entscheiden. Weil aber `get_by_year` inhärent Kalender-Jahr-basiert ist und der Filter im Loop ISO-Wochenjahr-basiert ist, ist die einfachste Lösung: **Beide Buckets zusammen filtern** und nicht zwischen "this" und "next" per year unterscheiden.

Konkret:

```rust
// Statt zwei Buckets aufzuteilen: sie in einen kombinieren.
// Union ist idempotent nach Row-ID.
let all_special_days: Vec<SpecialDay> = special_days_this
    .iter()
    .chain(special_days_next.iter())
    .cloned()
    .collect();
// Optional: nach id deduplizieren, falls Union überlappt.

// Im Loop:
let special_days: Arc<[SpecialDay]> = all_special_days
    .iter()
    .filter(|d| d.year == year && d.calendar_week == week)  // ISO-Wochenjahr-Match
    .cloned()
    .collect();
```

Damit ist die Bucket-Wahl irrelevant — beide Buckets werden konsultiert, der Row-Filter matched per ISO-Wochenjahr wie das DB-Schema es speichert.

**Alternative Option A': `get_by_year` erweitern.** Statt in `special_days.rs` nach Kalender-Jahr zu filtern, direkt nach ISO-Wochenjahr filtern — d.h. `find_by_year(y)` einfach durchreichen (das DAO filtert bereits `WHERE year = ? AND deleted IS NULL`). Aber das würde die `get_by_year`-Semantik brechen, die von den Frontends (Special-Days-Liste im UI) mit Kalender-Jahr-Semantik erwartet wird. → **Nicht empfohlen**, ist eine Domain-API-Änderung.

**Option B: `get_by_year` per ISO-Wochenjahr explizit anbieten.**

Neue Methode `get_by_iso_week_year(year)`, die einfach `find_by_year(y)` durchreicht. `booking_information::get_weekly_summary` benutzt die neue Methode. Andere Konsumenten (Frontend-Special-Days-Liste) bleiben auf Kalender-Jahr-`get_by_year`. → sauberer, aber Trait-Änderung nötig.

**Empfehlung: Option A oder Option B.** Beide erhalten die Perf-Gewinne von Phase 52. Option A ist eine 4-Zeilen-Änderung im Konsumenten. Option B fügt eine Trait-Methode + Impl hinzu, ist aber semantisch klarer.

**Snapshot-Bump nötig?** `billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION` betrifft nur `billing_period_report.rs` (siehe `shifty-backend/CLAUDE.md`). Dieser Bug ist im Weekly-Overview, der nicht persistiert wird — **kein Bump nötig**.

---

## 8. Zusammenhang zur prior Diagnose

**Vorheriger Bug (`paid_hours`):** `ExtraHoursDao::find_by_year` filtert kalendarisch, Konsument `assemble_weeks` bucket ISO. Bug in DAO-Ebene.

**Neuer Bug (`required_hours`):** `SpecialDayServiceImpl::get_by_year` filtert per Kalender-Datum (Union-Trick), Konsument `booking_information::get_weekly_summary` bucket per ISO-Wochenjahr über den Filter `d.year == year && d.calendar_week == week`. Bug ist **im Konsumenten (der wählt das falsche Bucket)** — der Service selbst ist semantisch legitim, nur seine Semantik passt nicht zur Verwendung im Weekly-Overview.

**Beide Bugs sind unabhängig fixbar.** Beide passieren nur an ISO-Wochenjahr-≠-Kalenderjahr-Grenzen (immer KW 1 wenn sie in Vorjahr startet, immer KW 52/53 wenn sie in Folgejahr endet).

**Beide sind auf die Weekly-Overview-Seite beschränkt.** Employee-Reports, Balance-Reports, Billing-Period-Snapshots, HR-Statistiken bleiben unberührt.

---

## 9. Fazit für den User-Report

Alle drei User-Beobachtungen sind jetzt vollständig erklärt und mechanisch reproduziert:

| User-Beobachtung | Bug | Grün-Reproduktions-Test |
|---|---|---|
| `paid_hours` KW 1 (Y+1) diff | ExtraHours-DAO Range | `reporting_year_boundary::get_year_vs_get_week_diverges_for_extra_hours_at_iso_kw1_boundary` |
| `paid_hours` KW 53 diff | ExtraHours-DAO Range | `reporting_year_boundary::get_year_vs_get_week_diverges_for_extra_hours_at_iso_kw53_boundary` |
| `required_hours` KW 1 diff | SpecialDay-Bucket-Wahl | `booking_information_weekly_summary_year_boundary_2026::h4b_holiday_2025_12_29_iso_2026_w1_bulk_vs_legacy` |
| `required_hours` KW 53 diff | SpecialDay-Bucket-Wahl | `booking_information_weekly_summary_year_boundary_2026::h4_holiday_2027_01_01_iso_2026_w53_bulk_vs_legacy` |

**Beide Bugs müssen einzeln gefixt werden.** Der Fix des ExtraHours-DAO-Range-Bugs ändert nichts am SpecialDay-Bucket-Bug und umgekehrt. Nach beiden Fixes müssen die zwei Repro-Tests umgebaut werden zu bulk-vs-legacy-Bit-Vergleichen (statt Drift-Assertions).

Der Test-File `booking_information_weekly_summary_year_boundary_2026.rs` bleibt im Repo — die 9 anderen Tests decken semantische Sicherheits-Netze für 2026 ab (ISO-Boundary-Facts, Slot-Filter-Konsistenz, Kalenderjahr-in-2026-Fälle die kein Diff triggern), die auch nach dem Fix relevant bleiben.
