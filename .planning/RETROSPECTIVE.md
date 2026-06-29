# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.4 — Committed Voluntary Capacity

**Shipped:** 2026-06-25
**Phases:** 4 (14–17) | **Plans:** 11 | **Tasks:** 26

### What Was Built
- Zeit-versioniertes `committed_voluntary: f32` auf `EmployeeWorkDetails` (Variante B) end-to-end durch alle Layer (SQLite-Migration → DAO → Service → `rest-types` → Frontend-State → Vertrags-Editor).
- No-double-count-Reporting via Zwei-Band-Dekomposition (FORMULA B) ausschließlich in Achse B (`booking_information.rs`), ohne Snapshot-Schema-Bump.
- Jahresansicht mit drittem Token 🎯 „zugesagt" + drittem gestapelten Chart-Segment; i18n De/En/Cs.
- „alle"-Filter + `is_paid`-Gating: rein unbezahlte Freiwillige sichtbar/auswählbar ohne Leak in `paid_hours`/Billing/Year-Summary.

### What Worked
- **Strikt compile-dependency-geordnete Build-Order** (Backend-Foundation Phase 14 inert vor konsumierendem Reporting/Frontend) hielt jede Phase isoliert grün-baubar.
- **Per-Boundary-Threading-Verifikation:** das Feld wurde an jeder Konversions-Grenze (entity/row/struct/TO/state) per Test gepinnt — keine stillen `0.0`-Drops.
- **Achse-A-vs-Achse-B-Disziplin:** die frühe Erkenntnis „Jahresansicht hängt an `booking_information.rs`, nicht `reporting.rs`" verhinderte die Doppelzählungs-Falle (D-FORMULA-PATH).

### What Was Inefficient
- **Snapshot-Versions-Annahme drehte mitten im Milestone** (D-01: erst „Bump 7→8" geplant, dann revidiert auf no-bump). Hinterließ Doc-Drift in Summaries + einem Code-Kommentar (beim Close gefixt).
- **Out-of-milestone-Commits** (`adf76c9` Snapshot-Bump 8→9, zwei Reverts zu Short-Employee-Report/Ehrenamt) liefen parallel zum Milestone und machten die „Version bleibt 7"-Aussage absolut falsch, obwohl semantisch korrekt.
- **MILESTONES.md-Auto-Entry** der GSD-CLI erzeugte Junk-Accomplishments („One-liner:", „Files:") aus fehlerhafter Summary-Frontmatter-Extraktion — manuell nachbearbeitet.

### Patterns Established
- **No-bump-Justification als Audit-Trail:** wenn eine Reporting-Änderung bewusst KEINEN Snapshot-Bump auslöst, wird die Begründung explizit in REQUIREMENTS/Phase-CONTEXT + einem Regressionstest (`snapshot_schema_version_…`) gepinnt.
- **Human-UAT-Verifikation für nicht-automatisierbare Pixel/Sprach-Checks:** Phase 17 live im Browser bestätigt; Phase 16 als `human_needed` markiert und beim Close bewusst deferred.

### Key Lessons
1. Wenn eine zentrale Versions-/Schema-Annahme während des Milestones kippt, sofort ALLE Zitate (Summaries, Kommentare, REQUIREMENTS) nachziehen — sonst entsteht Doc-Drift, die ein späterer Audit als „real bug?" flaggt.
2. Parallele out-of-milestone-Commits (hier: Snapshot-Bump + Reverts) gegen die laufende Milestone-Baseline prüfen, bevor man absolute Werte dokumentiert — relative Aussagen („v1.4 bumpt nicht") überleben, absolute („bleibt 7") nicht.
3. CLI-generierte MILESTONES.md-Einträge nach `milestone.complete` immer auf Junk-Accomplishments aus kaputter Frontmatter-Extraktion prüfen.

### Cost Observations
- Model mix: Planner opus, Executor sonnet (GSD-Config).
- Notable: Audit + Integration-Check via dediziertem `gsd-integration-checker`-Subagent deckte die Doc-Drift auf, die in den Phase-Verifikationen einzeln nicht sichtbar war.

---

## Milestone: v1.5 — Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen

**Shipped:** 2026-06-27
**Phases:** 6 (18–23) | **Plans:** 11

### What Was Built
- Carryover-Konsistenz: Vacation-Balance liest dieselbe `year-1`-Quelle wie der Report-Service (gepinnt + per Mock-Matcher gegen Reversion verriegelt).
- `vacation_days`-Korrektheit nach extra_hours→Absence-Konvertierung: derived Absences in per-Woche-Kategorien gemergt, Single Source `by_week`, kein Double-Count → Snapshot-Bump 9→10.
- Convert-Dialog UX: arbeitstagbasiertes bis-Datum + Erkennung des exakten 1-Wochen-Falls (Backend rechnet vor, FE wired).
- Mitarbeiter-Jahresansicht: KW+Datum-Hover/-Labels + gestapelte `volunteer_hours`; HR-only Ø-Stunden/Woche-Statistik (Regel A-22-1); UI-Polish (max-width + Zebra, schmalere Mitarbeiter-Spalte).
- Slot-Paid-Capacity-Frontend (Editor + Overage-Warnfarbe) inkl. `modify_slot`-Bugfix mit Regressionstest.

### What Worked
- **Backend-rechnet-vor-Pattern:** für Convert-Vorschläge (`suggested_end`/`is_full_week`) und HR-Statistik (A-22-1) lebt die Logik testbar im Backend, das Frontend wired nur — vermeidet untestbare WASM-Rechenlogik (vgl. Memory „Dioxus Browser-Test: Report-Werte").
- **Berechnungsregel als gepinnter CONTEXT-Anker** (A-22-1 in `22-CONTEXT.md`) hielt die „was zählt als gearbeitet / welche Wochen raus"-Definition über Plan-Grenzen stabil.
- **Snapshot-Bump diszipliniert** (9→10) genau dort, wo sich eine persistierte Computation (`vacation_days`) ändert — gemäß CLAUDE.md-Pflicht.

### What Was Inefficient
- **Bug erst im UAT gefunden:** `modify_slot` ließ `max_paid_employees` fallen (create-Pfad ≠ edit-Pfad) — erst beim Browser-UAT der Phase 23 entdeckt, nicht im Plan. Lesson in Memory „Backend-Roundtrip e2e prüfen".
- **STATE.md-Header-Drift:** SDK `phase.complete` zog bei Dezimal-/Folge-Phasen Header-Checkbox + Progress nicht nach — manuelles Nachziehen nötig (bekanntes Muster).
- **Close verzögert:** v1.5 wurde erst nach Start von v1.6/Phase 24 archiviert → ROADMAP musste v1.5 aus einem bereits-kollabierten `<details>` extrahieren statt aus der aktiven Sektion.

### Patterns Established
- **Convert-/Statistik-Vorberechnung im Backend** als Default für alles, was sonst nur im Browser verifizierbar wäre.
- **Override-Closeout bei code-fertigen, nur human-unverifizierten Debug-Sessions:** `carryover-absence-vs-report` mit Code-Fix + Tests grün wird als `awaiting_human_verify` bewusst beim Close acknowledged, nicht als Blocker behandelt.

### Key Lessons
1. Bei Feldern, die über mehrere Schreibpfade laufen (create vs. modify), jeden Pfad einzeln testen — ein grüner create-Test deckt den edit-Drop nicht ab.
2. Logik, deren Ergebnis nur im WASM-Frontend sichtbar wäre, ins Backend ziehen und dort unit-testen; das Frontend nur als dünner Wiring-Layer.
3. Milestones zeitnah zum letzten Phase-Abschluss schließen — ein verspäteter Close gegen eine schon-aktive Folge-Milestone macht die ROADMAP-Reorganisation aufwändiger.

### Cost Observations
- Model mix: Planner opus, Executor sonnet (GSD-Config).
- Notable: 6 Phasen / 11 Pläne, +5616/−154 LOC über 47 Dateien in ~2 Tagen; größter Einzelaufwand war die HR-Statistik-Phase (22) wegen der A-22-1-Definitionsfragen.

---

## Milestone: v1.6 — Paid-Capacity-Durchsetzung & Konfiguration

**Shipped:** 2026-06-27
**Phases:** 1 (24) | **Plans:** 5 (2 Waves)

### What Was Built
- Globaler hart/weich-Toggle (`paid_limit_hard_enforcement`) über den bestehenden `ToggleService` (Seed-Migration, Default weich → keine Regression) — bewusst NICHT `feature_flag`.
- Pre-Persist-Hard-Block in `book_slot_with_conflict_check` (`ShiftplanEditService` mit `ToggleService`-Dep, Toggle frisch pro Buchung gelesen): Nicht-Shiftplanner über Limit → `ServiceError::PaidLimitExceeded` (HTTP 409) vor `booking_service.create`; Shiftplanner-Bypass; nur bezahlte zählen; strikt-größer-Grenzregel.
- Permission-Gate des Buchungspfads korrigiert: `HR ∨ self` → `Shiftplanner ∨ self` (D-24-04).
- Admin-gated `/settings/`-Seite mit genau einem Toggle (`aria-pressed`, Inline-Feedback) + Toggle-REST-Client; persistente Overage-Warn-Sektion über dem Wochenplan für alle Rollen; i18n De/En/Cs.

### What Worked
- **Bestehende Infrastruktur wiederverwendet statt neue gebaut:** `ToggleService` (statt neuer feature_flag-Mechanik) und der v1.1-`Warning::PaidEmployeeLimitExceeded`-Pfad als Basis hielten die Phase auf 5 Pläne / einen Tag.
- **Service-Tier-Disziplin zahlte sich aus:** der Pre-Persist-Block lebt korrekt im Business-Logic-Tier (`ShiftplanEditService`), nicht im Basic-Tier-`BookingService` — DI-Order Basic-vor-Business in `main.rs` blieb deterministisch (CLAUDE.md-Konvention).
- **Distinkter Error-Status (409 statt 403)** machte das Frontend-Inline-Handling sauber adressierbar und vermeidet Verwechslung mit echter Permission-Verweigerung.

### What Was Inefficient
- **Zwei Bugs erst im Browser-UAT gefunden** (nicht im Plan/Test): (a) `/toggle` fehlte in der Dev-Proxy-Allowlist (`Dioxus.toml`) → Toggle im Dev funktionslos; (b) `current_paid_count` wurde aus dem HR-gegateten `is_paid` berechnet → Overage-Sektion für Nicht-HR-Rollen unsichtbar (D-24-03 verletzt). Beide klassisch „grüner Unit-Test deckt den e2e-/Rollen-Pfad nicht ab" (vgl. Memory „Backend-Roundtrip e2e prüfen").
- **STATE.md-Frontmatter-Drift:** beim Close stand STATE.md noch auf `executing/69%` (Phase 24 „execution started"), während ROADMAP/Commit längst Complete waren — SDK-Auto-State zog den Phasen-Abschluss nicht nach (bekanntes Muster, manuell korrigiert).

### Patterns Established
- **Globale Admin-Konfiguration über `ToggleService`** (nicht `feature_flag`): feature_flag = Rollout-Gating, Toggle = bleibende Admin-Einstellung. Saubere Trennung für künftige Schalter.
- **Rollen-sichtbarer Aggregat-Count ohne Identitäts-Leak:** `current_paid_count` aus un-gegatetem `get_all_paid(Authentication::Full)` zählen, per-Booking `is_paid` weiterhin gegated — der *Count* ist für alle Rollen sichtbar, *wer* bezahlt ist nicht.

### Key Lessons
1. Neue Backend-Pfad-Familien (`/toggle`) brauchen denselben Tag den passenden Dev-Proxy-Eintrag in `Dioxus.toml` — sonst ist das Feature im Dev still funktionslos, obwohl alle Tests grün sind.
2. Wenn ein Aggregat-Wert (`current_paid_count`) aus rollen-gegateten Quelldaten abgeleitet wird, explizit prüfen, ob die Anforderung „für alle Rollen sichtbar" mit dem Gating kollidiert — sonst verschwindet die UI für Nicht-privilegierte Rollen.
3. STATE.md-Frontmatter nach dem letzten Phasen-Abschluss aktiv verifizieren, bevor man `complete-milestone` fährt — der Auto-State ist nicht verlässlich auf 100%.

### Cost Observations
- Model mix: Planner opus, Executor sonnet (GSD-Config).
- Notable: kompakter Ein-Phasen-Milestone (53 Dateien, +6855/−106, ein `feat(24)`-Commit) an einem Tag; der Aufwand lag im UAT-Bug-Fixing, nicht im Plan.

---

## Milestone: v1.7 — Automatische Feiertage & Freiwilligen-Abwesenheit

**Shipped:** 2026-06-29 (Phasen complete & verified 2026-06-28)
**Phases:** 2 (25–26) | **Plans:** 7

### What Was Built
- Feiertags-Auto-Anrechnung **derive-on-read** (`build_derived_holiday_map` aus Toggle-`value`-Cutoff + `SpecialDay`), Wirkung identisch zu manuellem `ExtraHours(Holiday)`; Dual-Write `holiday_hours`+`absense_hours`; Snapshot-Bump 10→11.
- Konfigurierbarer „aktiv ab"-Stichtag über die `ToggleService`-`value`-Spalte (nullable `TEXT`, value-Presence treibt `enabled`) + admin-gated Settings-Date-Input, i18n de/en/cs.
- VFA-01 whole-week-out in `get_weekly_summary`: Abwesenheit eines Freiwilligen reduziert seine committed-Zusage 🎯 (beide Bänder); Feiertags-vs-Abwesenheits-Asymmetrie als CI-Guard gepinnt.
- Bidirektionale Deep-Links `/absences/:employee_id` ↔ Mitarbeiterreport (GlobalSignal-Preselect + 4 Ghost-Button-Cross-Links), i18n de/en/cs.

### What Worked
- **v1.6-Infrastruktur wiederverwendet:** die `ToggleService`-/Settings-Seite aus v1.6 trug Stichtag-Konfiguration + Date-Input fast unverändert — Phase 25 blieb dadurch kompakt.
- **derive-on-read statt materialize** hielt die Feiertags-Automatik reversibel und schrieb keine Bestands-Rows — der Stichtag schützt die Vergangenheit ohne Migration.
- **Asymmetrie als executable Guard** (`vfa02_holiday_vs_absence_asymmetry` + `phase26_vfa_no_snapshot_bump`): die bewusste Holiday-≠-Absence-Regel ist gegen versehentliche Kopplung CI-verriegelt.

### What Was Inefficient
- **Milestone-Close blieb liegen:** v1.7 wurde am 2026-06-28 complete & verified, aber der Close (Archiv/Tag/MILESTONES) wurde erst beim v1.8-Close 2026-06-29 nachgeholt — die `REQUIREMENTS.md` blieb in der Zwischenzeit auf v1.7 eingefroren und driftete gegen v1.8.
- **REQUIREMENTS.md-Body-Checkboxen** (HCFG-02/HSNAP-01/NAV-01) wurden nach Ausführung nicht abgehakt, obwohl verifiziert — reine Doc-Drift, beim Close korrigiert.

### Patterns Established
- **Toggle-`value`-Spalte für skalare Admin-Konfiguration:** boolean-`enabled` + optionaler `value TEXT` im selben Toggle-Record; value-Presence = aktiviert. Erweitert das v1.6-Toggle-Pattern um konfigurierbare Werte (z.B. ISO-Datum) ohne neue Tabelle.
- **whole-week-out statt pro-rata** für Abwesenheits-Reduktion der committed-Zusage: jede Überlappung in [Mo,So] nullt die Woche (category-agnostisch) — bewusst grob, deterministisch testbar.

### Key Lessons
1. Milestone-Close zeitnah nach „complete & verified" fahren — sonst friert die aktive `REQUIREMENTS.md` den alten Stand ein und driftet gegen den nächsten Milestone (hier: zwei offene Closes gleichzeitig).
2. derive-on-read mit Cutoff ist das saubere Mittel für „ab Stichtag, Vergangenheit unberührt" — keine Bestands-Migration, voll reproduzierbare historische Snapshots.

### Cost Observations
- Model mix: Planner opus, Executor sonnet (GSD-Config), autonom über beide Phasen.
- Notable: Snapshot-Bump 10→11 sauber an die Holiday-Computation gekoppelt; No-Bump-Guard für die VFA-Phase verhinderte versehentliche Drift.

---

## Milestone: v1.8 — Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX)

**Shipped:** 2026-06-29 (beide Phasen VERIFIED inkl. Live-HR-Browser-Smokes)
**Phases:** 2 (27–28) | **Plans:** 5

### What Was Built
- Gruppierter Personen-Selector (native `optgroup` Angestellte/Freiwillige) in **beiden** Call-Sites (AbsenceModal + AbsenceFilterBar) über einen gemeinsamen Pure-Helfer + RSX-Passthrough; inaktive ausgeblendet, leere Gruppen ausgelassen; i18n de/en/cs.
- Signed Urlaubsanspruch-**Offset** pro Person+Jahr (eigene Tabelle + Basic HR-gated Service + HR-gated REST CRUD): `entitled_effective = round(berechnet) + offset`, Delta überlebt Vertragsänderungen, fließt in `remaining_days`.
- **API-level Hiding:** `offset_days`/`computed_entitled_days` nur `Some` für HR; Self-View bekommt `None` und re-derived nichts. FE-Inline-Editor „berechnet {n} + Offset [x]" (signed, on-blur/Enter, year-scoped), User-Seite effective-only.
- Off-by-one-Proration-Fix (`vacation_days_for_year` year-START) + Snapshot-Bump 11→12 (`BillingPeriodValueType::VacationEntitlement`).

### What Worked
- **Service-Tier-Disziplin:** der Offset-Service ist Basic (nur DAO/Permission/Clock/Uuid/Transaction), `VacationBalanceService` (Business-Logic) konsumiert ihn — kein Cycle, DI-Order Basic-vor-Business deterministisch.
- **API-level statt UI-only Hiding:** die HR-only-Breakdown-Felder werden serverseitig auf `None` gesetzt (`is_hr` vor `hr.or(sp)?` gecaptured) — der Self-Pfad leakt den Offset gar nicht erst, sauberer als clientseitiges Ausblenden.
- **Ein Helfer für zwei Call-Sites** (D-27-01/03): Pure-Grouping + dünner RSX-Passthrough verhinderten Copy-Paste zwischen Modal und Filter; `is_selectable_employee` bewusst NICHT gelockert (D-27-02) hielt die HR-Urlaubsübersicht paid-only.

### What Was Inefficient
- **Zwei Bugs erst im Live-Smoke gefunden** (nicht im Plan/Test): (a) `/vacation-entitlement-offset` fehlte im `Dioxus.toml`-Dev-Proxy → FE-Save lief auf HTTP 405; (b) AbsenceModal schloss nach sauberem (warnungsfreiem) Create/Update nicht. Exakt das wiederkehrende „grüner Unit-Test deckt den e2e-/Proxy-Pfad nicht ab"-Muster (vgl. v1.6, Memory „Backend-Roundtrip e2e prüfen").
- **REQUIREMENTS.md-Drift:** v1.8 lief ganz ohne eigene aktive REQUIREMENTS.md — VOL-SEL-01/VAC-OFFSET-01 existierten nur in der ROADMAP; der Audit musste 3-Quellen-Cross-Reference statt Traceability-Tabelle nutzen. Beim Close in `v1.8-REQUIREMENTS.md` nachgetragen.

### Patterns Established
- **API-level field hiding für rollen-sensitive Breakdowns:** `Option`-Felder im TO, serverseitig nur für die privilegierte Rolle `Some`; der Client leitet nichts ab. Robuster gegen Leaks als UI-Gating.
- **Offset/Delta statt absolutem Override** für korrigierbare berechnete Werte: die Korrektur überlebt Neuberechnungen der Basis (hier Vertragsänderungen) — Integer-Korrektur **nach** `.round()`, nicht in die Summe.

### Key Lessons
1. Neue Backend-Pfad-Familien brauchen **am selben Tag** den `Dioxus.toml`-Dev-Proxy-Eintrag — zum dritten Mal in Folge (v1.6 `/toggle`, jetzt `/vacation-entitlement-offset`) im Live-Smoke aufgeschlagen; Kandidat für eine Plan-Checkliste „neuer REST-Pfad → Proxy-Eintrag".
2. Ein Milestone ohne eigene REQUIREMENTS.md ist verifizierbar (Decision-/ROADMAP-getrieben + Audit), erzeugt aber Doc-Schulden — entweder bewusst Decision-getrieben fahren (wie v1.6) oder REQUIREMENTS.md am Milestone-Start anlegen.
3. Pflicht-Snapshot-Bump greift auch bei reinen Korrektheits-Fixes: der Off-by-one änderte `VacationEntitlement`-Computation → Bump 11→12 zwingend (CLAUDE.md-Regel korrekt befolgt).

### Cost Observations
- Model mix: Planner opus, Executor sonnet (GSD-Config), autonom über beide Phasen + Live-HR-Browser-Smokes.
- Notable: kompakte 5-Plan-/Zwei-Phasen-Lieferung an einem Tag inkl. formalem Milestone-Audit (`passed`); der Aufwand lag im Live-Smoke-Bugfixing, nicht im Plan.

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Key Change |
|-----------|--------|------------|
| v1.4 | 4 | Erster Milestone mit dediziertem Pre-Close-Integration-Check + formalem Milestone-Audit vor dem Abschluss; no-bump-Justification-Pattern etabliert. |
| v1.5 | 6 | Backend-rechnet-vor-Pattern für sonst-nur-im-Browser-prüfbare Logik; override_closeout für code-fertige, human-unverifizierte Debug-Sessions; Close ohne formalen Milestone-Audit (Phasen-UAT genügte). |
| v1.6 | 1 | Kompakter Ein-Phasen-Milestone auf bestehender Infrastruktur (`ToggleService`); override_closeout mit deferred Human-UAT-Item; Decision-getrieben (D-24-XX) statt formaler REQ-IDs (keine REQUIREMENTS.md). |
| v1.7 | 2 | derive-on-read + Cutoff-Stichtag-Pattern; Asymmetrie als executable CI-Guard; Milestone-Close verzögert (gemeinsam mit v1.8 nachgeholt) → REQUIREMENTS.md-Drift. |
| v1.8 | 2 | Formaler Milestone-Audit (`passed`) trotz fehlender aktiver REQUIREMENTS.md (3-Quellen-Cross-Reference); API-level Hiding + Offset/Delta-Pattern; Live-HR-Browser-Smokes als Verifikation. |

### Cumulative Quality

| Milestone | Tests (Backend / Frontend) | Snapshot-Schema-Version |
|-----------|----------------------------|-------------------------|
| v1.4 | service_impl 451 + rest-types 3 / 628 | 9 (unverändert durch v1.4) |
| v1.5 | workspace grün (+ Regressionstests UV-04/UV-05/A-22-1) / WASM-Build grün | 10 (Bump 9→10 in Phase 18, `vacation_days`-Computation) |
| v1.6 | workspace grün (+ 4 Hard-Block-Tests + `test_current_paid_count_correct_for_non_hr_caller`) / WASM-Build grün | 10 (unverändert durch v1.6 — kein persistierter `value_type` berührt) |
| v1.7 | workspace grün (+ Holiday-Acceptance-Tests + VFA-01/02-Guards) / WASM-Build grün | 11 (Bump 10→11 in Phase 25, Holiday-Computation/Input-Set) |
| v1.8 | workspace grün (+ Offset/Balance/Off-by-one/Snapshot-Guard) / WASM-Build + 678 FE-Tests grün | 12 (Bump 11→12 in Phase 28, `VacationEntitlement`-Computation) |

### Top Lessons (Verified Across Milestones)

1. Achse-A-vs-Achse-B-Trennung (Reporting-Persistenz vs. Jahresansicht-Read-Pfad) ist die wiederkehrende Doppelzählungs-Falle in Shifty — jede Kapazitäts-/Stunden-Änderung muss explizit benennen, welche Achse sie berührt.
2. End-to-end-Feld-Threading braucht Per-Boundary-Tests; Round-Trip-Tests mit fraktionalen Werten (2.5) fangen stille `0.0`-Drops zuverlässig.
3. **Neue REST-Pfad-Familien brauchen am selben Tag den `Dioxus.toml`-Dev-Proxy-Eintrag** — in v1.6 (`/toggle`) und v1.8 (`/vacation-entitlement-offset`) je erst im Live-Smoke (HTTP 405/funktionslos) aufgeschlagen, obwohl alle Tests grün waren. Backend-Pfad ≠ im Dev erreichbar.
4. **Snapshot-Bump-Pflicht gilt auch für Korrektheits-Fixes:** sobald die Computation eines persistierten `value_type` sich ändert (v1.8 Off-by-one → `VacationEntitlement`), ist der Bump zwingend — unabhängig davon, ob es ein „Feature" oder ein „Bugfix" ist.
