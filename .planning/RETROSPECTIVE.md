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

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Key Change |
|-----------|--------|------------|
| v1.4 | 4 | Erster Milestone mit dediziertem Pre-Close-Integration-Check + formalem Milestone-Audit vor dem Abschluss; no-bump-Justification-Pattern etabliert. |
| v1.5 | 6 | Backend-rechnet-vor-Pattern für sonst-nur-im-Browser-prüfbare Logik; override_closeout für code-fertige, human-unverifizierte Debug-Sessions; Close ohne formalen Milestone-Audit (Phasen-UAT genügte). |

### Cumulative Quality

| Milestone | Tests (Backend / Frontend) | Snapshot-Schema-Version |
|-----------|----------------------------|-------------------------|
| v1.4 | service_impl 451 + rest-types 3 / 628 | 9 (unverändert durch v1.4) |
| v1.5 | workspace grün (+ Regressionstests UV-04/UV-05/A-22-1) / WASM-Build grün | 10 (Bump 9→10 in Phase 18, `vacation_days`-Computation) |

### Top Lessons (Verified Across Milestones)

1. Achse-A-vs-Achse-B-Trennung (Reporting-Persistenz vs. Jahresansicht-Read-Pfad) ist die wiederkehrende Doppelzählungs-Falle in Shifty — jede Kapazitäts-/Stunden-Änderung muss explizit benennen, welche Achse sie berührt.
2. End-to-end-Feld-Threading braucht Per-Boundary-Tests; Round-Trip-Tests mit fraktionalen Werten (2.5) fangen stille `0.0`-Drops zuverlässig.
