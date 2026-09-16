---
phase: 60-person-datenmodell-backfill
plan: 03
subsystem: documentation
tags: [docs, docs-freshness-gate, data-model, er-diagram, bilingual, person, identity]

# Dependency graph
requires:
  - phase: 60-person-datenmodell-backfill
    plan: 01
    provides: "Migration 20260916000000_create-person-backfill.sql — der dokumentierte Spaltensatz, die Index-Namen und der Backfill-Mechanismus"
  - phase: 60-person-datenmodell-backfill
    plan: 02
    provides: "16 Invariantentests — jede Doku-Aussage dieses Plans ist durch einen davon oder durch 60-01 gedeckt"
provides:
  - "Docs-Freshness-Gate fuer `migrations/sqlite/*.sql` in Phase 60 geschlossen: alle drei Trigger-Ziele aktualisiert, jeweils EN und DE"
  - "Schriftliche Begruendung inkl. one-way-Vermerk fuer den vollstaendigen statt partiellen Unique-Index (D-60-01/D-60-02)"
  - "Schriftliche Festlegung der dauerhaften Optionalitaet beider `person_id`-Spalten und der drei gleichwertigen Auspraegungen (D-60-03)"
  - "Aufloesung des `[To verify]`-Platzhalters in der Foreign-Keys-Konvention: FK-Durchsetzung ist aus, Deklarationen sind dekorativ (D-60-06)"
  - "ER-Diagramm mit Entitaet PERSON, beiden Beziehungen und fortgeschriebenem Stichtag"
affects: [61-person-dao-selbstzugriff, 62-personenverwaltung, 63-person-auspraegungen]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Phasenbezogener Nachtrag in `03-data-model.md` im Aufbau des Phase-54-Abschnitts (Ueberschrift mit Phase + Milestone, dann Unterabschnitte pro Tabelle/Spalte/Thema)"
    - "Doku-Aussagen werden an die absichernden Testnamen gekoppelt (`person_table_has_no_deleted_column`, `*_index_is_not_partial`), damit ein spaeterer Leser sieht, was die Aussage haelt"

key-files:
  created: []
  modified:
    - docs/architecture/03-data-model.md
    - docs/architecture/03-data-model_de.md
    - docs/architecture/diagrams/db-schema-er.mmd
    - docs/features/F01-employee-management.md
    - docs/features/F01-employee-management_de.md
    - docs/features/F12-auth-session.md
    - docs/features/F12-auth-session_de.md

key-decisions:
  - "Der `[To verify]`-Platzhalter unter `Schema conventions → Foreign keys` wurde aufgeloest statt nur referenziert: der Plan verlangt eine Doku, die keine Garantie behauptet, die der Code nicht haelt (T-60-13). Ein `[To verify]` an genau der Stelle, auf die der neue Abschnitt verweist, waere ein offener Widerspruch gewesen. Belegt durch `grep -rc 'PRAGMA foreign_keys' --include='*.rs' --include='*.sql' .` = 0 Treffer."
  - "Das ASCII-Beziehungsdiagramm in F01 wurde um zwei `person`-Zeilen ueber dem bestehenden Diagramm erweitert statt das bestehende Diagramm umzuzeichnen — die Elternbeziehung ist so ablesbar, ohne die etablierte `sales_person`-Zentrierung zu zerstoeren."
  - "Der Kommentar zur fehlenden `deleted`-Spalte im ER-Diagramm steht als `%%`-Block VOR der `PERSON`-Entitaet, nicht als Spaltenkommentar: der Kopfkommentar des Diagramms nennt die Audit-Spalten als implizit vorhanden, die Ausnahme muss deshalb auf Blockebene stehen, um gefunden zu werden."
  - "In F12 bekam die Person-Identitaet einen eigenen `###`-Unterabschnitt im Relationships-Block statt einer Tabellen-Fussnote — die vier Aussagen (PK bleibt, HR-Benutzer dauerhaft erlaubt, sales_person_user bleibt, Durchsetzung via Index) brauchen Prosa, keine Zelle."

patterns-established:
  - "Zweisprachiger Doku-Commit: EN- und DE-Fassung derselben Datei werden in einem einzigen `git add` gestaged; `git diff --name-only HEAD~1 HEAD` ist die Gegenprobe."

requirements-completed: [PERS-01, PERS-04, PERS-06]

coverage:
  - id: D1
    description: "Docs-Freshness-Gate fuer `migrations/sqlite/*.sql` erfuellt — alle drei Trigger-Ziele aus der CLAUDE.md-Tabelle aktualisiert, jeweils EN und DE"
    requirement: "PERS-01"
    verification:
      - kind: other
        ref: "git diff --name-only HEAD~2 HEAD -> 7 Dateien: 03-data-model{,_de}.md, db-schema-er.mmd, F01{,_de}.md, F12{,_de}.md"
        status: pass
    human_judgment: false
  - id: D2
    description: "`person` ist mit genau vier Spalten und ohne Soft-Delete-Spalte dokumentiert (D-60-04)"
    requirement: "PERS-01"
    verification:
      - kind: other
        ref: "Spaltentabelle in 03-data-model{,_de}.md deckt sich mit PRAGMA table_info(person) aus Test person_table_has_only_identity_and_audit_columns"
        status: pass
    human_judgment: false
  - id: D3
    description: "Beide Unique-Indizes sind als vollstaendig, mit Begruendung und one-way-Vermerk dokumentiert (D-60-01, D-60-02)"
    requirement: "PERS-04"
    verification:
      - kind: other
        ref: "grep -c 'sales_person_person_id_unique_idx' 03-data-model.md / _de.md -> je >= 1; Abschnitt nennt 'one-way' bzw. 'Diese Entscheidung ist one-way'"
        status: pass
    human_judgment: false
  - id: D4
    description: "Die 1:1-Durchsetzung ist als DB-Index dokumentiert, nicht als Fremdschluessel; die abgeschaltete FK-Durchsetzung ist benannt (D-60-05, D-60-06)"
    requirement: "PERS-04"
    verification:
      - kind: other
        ref: "Abschnitt 'Enforcement: index, not foreign key' + aufgeloeste Foreign-Keys-Konvention; grep -rc 'PRAGMA foreign_keys' --include='*.rs' --include='*.sql' . = 0"
        status: pass
    human_judgment: false
  - id: D5
    description: "Dauerhafte Optionalitaet und die drei gleichwertigen Auspraegungen sind in Architektur- UND Feature-Doku festgehalten (D-60-03)"
    requirement: "PERS-01"
    verification:
      - kind: other
        ref: "03-data-model{,_de}.md Abschnitt person_id; F01 Randfall 'Sales Person ohne Login'; F12 Abschnitt 'Person identity' / 'Person-Identitaet'"
        status: pass
    human_judgment: false
  - id: D6
    description: "`sales_person_user` ist als in Phase 60 bestehend dokumentiert, Aufloesung in Phase 61"
    requirement: "PERS-06"
    verification:
      - kind: other
        ref: "grep -rc 'DROP TABLE sales_person_user' migrations/sqlite/ = 0 Treffer; F01-Tabellenzeile + ER-Kommentar + Abgrenzungs-Abschnitt nennen Phase 61"
        status: pass
    human_judgment: false
  - id: D7
    description: "Snapshot-Vertrag unveraendert dokumentiert: Version bleibt 13, keine der vier Bump-Klauseln greift (PERS-06)"
    requirement: "PERS-06"
    verification:
      - kind: other
        ref: "grep -c 'CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13' service_impl/src/billing_period_report.rs = 1 (Datei nicht im Diff); Abschnitt 'Snapshot contract' / 'Snapshot-Vertrag'"
        status: pass
    human_judgment: false
  - id: D8
    description: "ER-Diagramm zeigt PERSON, `person_id` an beiden Traegern, beide Beziehungen mit Kardinalitaet 0..1 und einen aktuellen Stichtag"
    requirement: "PERS-01"
    verification:
      - kind: other
        ref: "grep -c '^    PERSON {' = 1; 'PERSON ||--o| SALES_PERSON :' = 1; 'PERSON ||--o| USER' = 1; '2026-07-04' = 0"
        status: pass
    human_judgment: false
  - id: D9
    description: "Das Mermaid-Diagramm rendert weiterhin (Syntax unbeschaedigt)"
    requirement: "PERS-01"
    verification:
      - kind: manual_procedural
        ref: "Visuelle Pruefung der Syntax gegen die bestehenden Bloecke desselben Files; kein Mermaid-Renderer in der Devshell verfuegbar"
        status: pass
    human_judgment: true
    rationale: "Weder `mmdc` noch ein anderer Mermaid-Renderer ist in der Nix-Devshell installiert. Die Aenderung nutzt ausschliesslich Konstrukte, die im selben File bereits vorkommen (Entity-Block mit typisierten Spalten, Spaltenkommentar in Anfuehrungszeichen, `||--o|`-Beziehung mit Label, `%%`-Kommentarzeilen zwischen Beziehungen). Ein echtes Render-Gate bleibt eine Sichtpruefung im Browser."

# Metrics
duration: 6min
completed: 2026-09-16
status: complete
---

# Phase 60 Plan 03: Docs-Freshness-Gate schliessen Summary

**Sieben Doku-Dateien (drei Trigger-Ziele, jeweils EN und DE) beschreiben jetzt die `person`-Elterntabelle, beide dauerhaft optionalen `person_id`-Spalten, beide vollstaendigen Unique-Indizes mit one-way-Vermerk und den unveraendert bei 13 bleibenden Snapshot-Vertrag — reine Doku-Phase, kein Rust-Code angefasst.**

## Performance

- **Duration:** 6 min
- **Started:** 2026-09-16T07:07:57Z
- **Completed:** 2026-09-16T07:13:29Z
- **Tasks:** 2 (beide `type="auto"`, kein Checkpoint)
- **Files modified:** 7 (0 neu)

## Accomplishments

- **`docs/architecture/03-data-model.md` + `_de.md`** tragen einen neuen Abschnitt „Phase 60 (v2.7) — Person identity parent table" / „Person-Identitaets-Elterntabelle" im Aufbau des bestehenden Phase-54-Nachtrags: Spaltentabelle von `person` (vier Spalten, ausdruecklich ohne `deleted` und ohne Namensspalten), dauerhaft nullable `person_id` als fachliche Zusage statt Uebergangszustand, beide Unique-Indizes als vollstaendig mit Begruendung und **one-way**-Vermerk, Durchsetzung ueber Index statt Fremdschluessel, Backfill-Mechanik inklusive Selbst-Assertion und `update_process`-Herkunftsmarker, Abgrenzung zu Phase 61 und der unveraenderte Snapshot-Vertrag.
- **Zwei Bestandsstellen derselben Dateien mitgezogen:** die Aggregat-Liste bekommt `Person` als Identitaets-Klammer (ohne die `Employee`-Zeile zu ersetzen), die Migrations-Historie eine `2026-09`-Zeile mit dem Dateinamen.
- **`docs/architecture/diagrams/db-schema-er.mmd`** zeigt die Entitaet `PERSON` (mit `%%`-Warnblock, dass hier die Audit-Spalten-Konvention des Kopfkommentars bewusst nicht gilt), `blob person_id "nullable, UK"` in den Bloecken `SALES_PERSON` und `USER`, die beiden Beziehungen `PERSON ||--o| SALES_PERSON : "identifies"` und `PERSON ||--o| USER : "identifies"` sowie einen Kommentar an den `SALES_PERSON_USER`-Beziehungen, dass die Verknuepfungstabelle bis Phase 61 die laufende Wahrheit bleibt. Der Stichtag im Kopfkommentar steht jetzt auf `2026-09-16`.
- **`docs/features/F01-employee-management.md` + `_de.md`** fuehren `person` als eigene Tabellenzeile, nennen `person_id` in der `sales_person`-Zeile, weisen an `sales_person_user` auf das Nebeneinander bis Phase 61 hin, listen die neue Migration und ergaenzen zwei Randfaelle: die dauerhaft behaltene Identitaet nach Soft-Delete (zurueckkehrender Mitarbeiter = zunaechst zwei Personen, Zusammenfuehren kein v2.7-Scope) und die Sales Person ohne Login als gleichwertige Auspraegung.
- **`docs/features/F12-auth-session.md` + `_de.md`** nennen `user.person_id` in der Tabellenzeile samt Verweis auf die vollstaendige Beschreibung, listen die neue Migration und bekommen einen eigenen Unterabschnitt „Person identity" / „Person-Identitaet": `user.name` bleibt Primaerschluessel und die Auth-Schicht unberuehrt, HR-/Admin-Benutzer ohne Sales Person sind dauerhaft erlaubt und es wird nirgends automatisch eine Sales Person dazu angelegt, `sales_person_user` bleibt bestehen, und die Umschreibung der Selbst-Zugriffs-Pruefung auf `person_id`-Joins ist Phase-61-Scope.

## Task Commits

1. **Task 1: Architektur-Datenmodell und ER-Diagramm (EN + DE)** — `61e3be7a` (docs)
2. **Task 2: Feature-Dokus F01 und F12 (EN + DE)** — `08502ba8` (docs)

## Files Created/Modified

- `docs/architecture/03-data-model.md` / `_de.md` — neuer Phase-60-Abschnitt (~110 Zeilen je Sprache), Aggregat-Bullet, Migrations-Historie-Zeile, aufgeloeste Foreign-Keys-Konvention.
- `docs/architecture/diagrams/db-schema-er.mmd` — Entitaet `PERSON` + Warnkommentar, je eine `person_id`-Zeile in `SALES_PERSON` und `USER`, zwei Beziehungszeilen + Erlaeuterungskommentar, Kommentar an den `SALES_PERSON_USER`-Beziehungen, Stichtag `2026-07-04` → `2026-09-16`.
- `docs/features/F01-employee-management.md` / `_de.md` — `person`-Tabellenzeile, `person_id` an `sales_person`, erweiterte `sales_person_user`-Zeile, Migrations-Eintrag, erweitertes Beziehungsdiagramm + Vorspann, zwei neue Randfaelle.
- `docs/features/F12-auth-session.md` / `_de.md` — `person_id` in der `user`-Zeile, Migrations-Eintrag, neuer Unterabschnitt „Person identity" / „Person-Identitaet".

## Decisions Made

- **`[To verify]` in der Foreign-Keys-Konvention aufgeloest** (siehe Deviation 1). Der neue Abschnitt verweist fuer „die Garantie laeuft ueber den Index, nicht ueber Fremdschluessel" ausdruecklich auf diese Konvention; sie durfte dort nicht weiter offenlassen, ob FK-Durchsetzung aktiv ist.
- **ER-Ausnahmekommentar auf Blockebene.** Der Kopfkommentar des Diagramms erklaert die Audit-Spalten (inklusive `deleted`) als implizit vorhanden. Fuer `PERSON` trifft das nicht zu — der Hinweis steht deshalb als `%%`-Block direkt ueber der Entitaet, wo ein Leser ihn findet, bevor er die Konvention anwendet.
- **F01-Beziehungsdiagramm additiv erweitert**, nicht umgezeichnet: zwei `person`-Zeilen oberhalb des bestehenden ASCII-Diagramms im selben Codeblock. Ein Umbau haette die gewachsene `sales_person`-Zentrierung unleserlich gemacht, ohne fachlich mehr zu sagen.
- **Keine Aussage ueber Loeschen oder Zusammenfuehren von Personen** — beide Randfaelle sind ausdruecklich als Nicht-v2.7-Scope formuliert, weil kein Test aus 60-01/60-02 ein solches Verhalten deckt.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] `[To verify]`-Platzhalter in der Foreign-Keys-Konvention aufgeloest**
- **Found during:** Task 1
- **Issue:** Der Plan verlangt in Task 1 einen „Verweis auf die bestehende Notiz dazu in `Schema conventions → Foreign keys`". Diese Notiz stand in beiden Sprachfassungen auf „Ob Shifty das global setzt: **[Zu prüfen]** im Startup-Pfad." Ein Verweis auf eine offene Frage haette den neuen Abschnitt („die Durchsetzung ist aus, deshalb laeuft die Garantie ueber den Index") direkt widersprochen — genau der Fall, den Threat T-60-13 („Doku behauptet eine Garantie, die der Code nicht haelt") verhindern soll.
- **Fix:** Die Notiz haelt jetzt den verifizierten Stand fest: Shifty setzt `PRAGMA foreign_keys` nirgends, die `FOREIGN KEY`-Deklarationen sind dekorative Dokumentation, kein durchgesetztes Constraint [D-60-06]; wer eine echte Garantie braucht, formuliert sie als `UNIQUE`-Index oder `CHECK`.
- **Files modified:** `docs/architecture/03-data-model.md`, `docs/architecture/03-data-model_de.md`
- **Verification:** `grep -rc 'PRAGMA foreign_keys' --include='*.rs' --include='*.sql' .` → 0 Treffer (Repo-Zustand unveraendert; die Prohibition beschraenkt sich ausdruecklich auf `*.rs`/`*.sql`, die Nennung in `.md` faellt nicht darunter).
- **Committed in:** `61e3be7a`

**2. [Rule 3 - Blocking] `cargo` nicht auf dem Default-PATH**
- **Found during:** Task 1 (Verifikationslauf)
- **Issue:** `cargo: command not found` — die Rust-Toolchain liegt in der Nix-Devshell (identisch zu Deviation 3 in 60-01).
- **Fix:** Alle Gates ueber `nix develop --command bash -c '...'`. Keine Repo-Aenderung.
- **Files modified:** keine
- **Committed in:** —

### Nicht erfuellbares Acceptance Criterion (Plan-Fehler, kein Deviation-Fall)

Task 1 fordert `grep -c 'PERSON {' docs/architecture/diagrams/db-schema-er.mmd` == 1. Das ist unerfuellbar: `SALES_PERSON {` und `BILLING_PERIOD_SALES_PERSON {` enthalten den gesuchten Teilstring und existierten bereits vor dieser Aenderung (der Wert war also schon vorher 3, jetzt 4). Gemeint ist offensichtlich „genau eine `PERSON`-Entitaet". Gepruefte Ersatzform: `grep -c '^    PERSON {' …` → **1**. Alle uebrigen Acceptance Criteria wurden woertlich gepruefet.

---

**Total deviations:** 2 auto-fixed (1 missing critical, 1 blocking)
**Impact on plan:** Keine inhaltliche Abweichung vom Liefergegenstand. Deviation 1 erweitert den Plan um genau eine Zeile Bestands-Doku, ohne die die neue Aussage widerspruechlich waere.

## Issues Encountered

- **`gsd-tools query roadmap.update-plan-progress 60` hat die falsche ROADMAP-Tabelle ueberschrieben — zum dritten Mal in dieser Phase.** Siehe Abschnitt „Bekannter Werkzeugdefekt" unten.
- **Kein Mermaid-Renderer in der Devshell.** Die Syntaxpruefung des ER-Diagramms ist eine Sichtpruefung gegen die bestehenden Konstrukte desselben Files (Coverage-Item D9, `human_judgment: true`).
- **Keine offenen Probleme.**

## Bekannter Werkzeugdefekt — dritte Reproduktion bestaetigt

`node .claude/gsd-core/bin/gsd-tools.cjs query roadmap.update-plan-progress 60` hat erneut Zeile 110 von `.planning/ROADMAP.md` getroffen — die Phase-60-Zeile der **Frontend-Relevanz**-Tabelle statt der Fortschrittstabelle. Vor dem Lauf verbatim gesichert:

```
| 60 | ja | **nein** | Reine Schema- + Backfill-Phase ohne API-Wirkung — kein TO geändert, kein Endpoint neu, keine sichtbare Verhaltensänderung. |
```

Nach dem Lauf stand dort wortwoertlich `| 60 | 3/3 | Complete   | 2026-09-16 |`. Die Originalzeile wurde wortgleich wiederhergestellt, sodass `git diff .planning/ROADMAP.md` nur noch die korrekte Checkbox-Aenderung fuer `60-03-PLAN.md` zeigt; der korrekte Teil des Updates (Plan-Checkbox fuer `60-03-PLAN.md`) bleibt erhalten. **Der Defekt ist damit dreimal in Folge reproduziert** (60-01, 60-02, 60-03) — der Handler matcht offenbar die erste Markdown-Tabelle, die eine Zeile mit `| 60 |` enthaelt, statt die Fortschrittstabelle zu adressieren. Fuer kuenftige Phasen mit einer Frontend-Relevanz-Tabelle ist die Sicherung-und-Wiederherstellung vor jedem `roadmap.update-plan-progress`-Lauf Pflicht, solange der Handler nicht korrigiert ist.

## Prohibitions — Verifikationsstand

| Prohibition | Pruefung | Ergebnis |
|---|---|---|
| Keine einseitige Sprachfassung | `git diff --name-only HEAD~2 HEAD` | 7 Dateien, jede `foo.md` mit ihrer `foo_de.md` (das `.mmd` ist sprachneutral und existiert nur einmal) |
| Snapshot-Version bleibt 13 | `grep -c 'CURRENT_SNAPSHOT_SCHEMA_VERSION: u32 = 13' service_impl/src/billing_period_report.rs` | 1 (Datei nicht im Diff); Doku nennt 13 als unveraenderten Stand |
| `person` ohne Namens- und Soft-Delete-Spalte dokumentiert | Spaltentabelle in beiden Sprachfassungen | exakt `id`, `created`, `update_process`, `update_version` |
| `sales_person_user` nicht als entfernt beschrieben | `grep -rc 'DROP TABLE sales_person_user' migrations/sqlite/` | 0 Treffer; Doku verweist fuer die Aufloesung auf Phase 61 |
| Keine Empfehlung, FK-Durchsetzung einzuschalten | `grep -rc 'PRAGMA foreign_keys' --include='*.rs' --include='*.sql' .` | 0 Treffer; Doku haelt nur fest, dass sie aus ist |
| Kein Install-Task (T-60-SC) | Diff enthaelt ausschliesslich `.md`/`.mmd` | bestaetigt |

## Verification

**Dieser Plan ist docs-only — es wurde keine einzige Rust-, SQL- oder Konfigurationsdatei angefasst.** Die Cargo-Gates liefen trotzdem als Regressionsnachweis, wie vom Plan gefordert:

- `SQLX_OFFLINE=true cargo test --workspace` → exit 0, alle Suites gruen
- `cargo clippy --workspace -- -D warnings` → exit 0
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0
- `git diff --name-only HEAD~2 HEAD` → 7 Doku-Dateien, jede Sprachfassung mit ihrem Gegenstueck
- `grep -c '^    PERSON {' docs/architecture/diagrams/db-schema-er.mmd` → 1
- `grep -c 'PERSON ||--o| SALES_PERSON :' …` → 1 · `grep -c 'PERSON ||--o| USER' …` → 1
- `grep -c '2026-07-04' docs/architecture/diagrams/db-schema-er.mmd` → 0
- `grep -c 'sales_person_person_id_unique_idx'` → EN 3, DE 3 (Abschnittsueberschrift, Fliesstext, Feature-Doku)
- `grep -c '20260916000000_create-person-backfill'` → 2 je Architektur-Datei, 1 je Feature-Datei
- `grep -c 'person_id'` → F01 EN/DE je 19, F12 EN/DE je 9

## User Setup Required

None.

## Next Phase Readiness

- **Phase 60 ist inhaltlich abgeschlossen.** Alle drei Plaene sind ausgefuehrt; das Docs-Freshness-Gate fuer die Trigger-Datei `migrations/sqlite/*.sql` ist geschlossen.
- **Offenes Phasen-Gate aus 60-01 (unveraendert):** der Migrationslauf gegen eine Kopie der echten Produktionsdatei bleibt manuell (`60-VALIDATION.md`, Manual-Only). Die lokale Dev-DB traegt keine realistische Verteilung.
- **Fuer Phase 61:** die Doku benennt an drei Stellen explizit, was dort ansteht — Vergabe von `person_id` beim Anlegen (D-60-10), Umschreibung von `find_sales_person_by_user_id` auf `person_id`-Joins inklusive Vorher/Nachher-Permission-Matrix, und die Aufloesung von `sales_person_user`. Wer Phase 61 plant, findet die Abgrenzung in `03-data-model.md` § „Scope boundary" und in F12 § „Person identity".
- **Warnung fuer kuenftige Doku-Passes:** der one-way-Vermerk zu den vollstaendigen Unique-Indizes steht bewusst in beiden Sprachfassungen UND im Kopfkommentar der Migration. Er ist Vertragsbestandteil, keine Redundanz.

---
*Phase: 60-person-datenmodell-backfill*
*Completed: 2026-09-16*

## Self-Check: PASSED

Alle sieben in dieser SUMMARY genannten Doku-Dateien existieren auf der Platte; beide Task-Commits (`61e3be7a`, `08502ba8`) sind in der Git-Historie auffindbar.
