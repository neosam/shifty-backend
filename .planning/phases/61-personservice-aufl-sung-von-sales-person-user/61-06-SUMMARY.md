---
phase: 61-personservice-aufl-sung-von-sales-person-user
plan: 06
subsystem: documentation
tags: [docs, docs-freshness-gate, auth, data-model, er-diagram, bilingual, arc42]

# Dependency graph
requires:
  - phase: 61-01
    provides: "PersonService/PersonDao (Basic-Tier) — dokumentierter Dependency-Schnitt"
  - phase: 61-02
    provides: "person_id-Vergabe bei Neuanlage in beiden Pfaden (D-61-01)"
  - phase: 61-03
    provides: "alle vier lesenden Verknuepfungs-Pfade auf JOIN u.person_id = sp.person_id (D-61-08)"
  - phase: 61-04
    provides: "Merge/Split/409-Semantik der Schreibpfade (D-61-03/04/05, P-61-B)"
  - phase: 61-05
    provides: "DROP-Migration, Teil-Migrationsketten-Harness, erster Doku-Durchgang an sieben Dateien"
provides:
  - "docs/architecture/04-auth{,_de}.md §Self-access gate — der bisher voellig undokumentierte HR-∨-self-Pfad, inkl. der Konvention 'Identitaetsvergleich gehoert ins SQL'"
  - "docs/architecture/03-data-model{,_de}.md §Die dauerhaft pruefbare Invariante — drei auditierbare SQL-Proben als Ersatz fuer die nicht mehr auswertbare Phase-60-Zaehlprobe"
  - "FK-Durchsetzungs-Beleg auf einem weiterhin existierenden Fremdschluessel (sales_person_unavailable -> sales_person), empirisch nachgefahren"
  - "PersonService-Knoten im Runtime-Diagramm mit Begruendung fuer die bewusste Kantenlosigkeit"
  - "F01 §9 Versionshistorie mit v2.7-Eintrag (Phase 60 + Phase 61) in beiden Sprachen"
affects: [62-personenverwaltung-liste, 63-personen-detail]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Doku-Nachzug am Phasenende prueft zuerst, was fruehere Plaene derselben Phase bereits korrigiert haben — ein Plan, der pauschal neu schreibt, macht korrekte Aussagen wieder kaputt"
    - "Eine Doku-Behauptung ueber Datenbankverhalten wird vor dem Schreiben gegen eine mit der VOLLEN Migrationskette gebaute Wegwerf-DB nachgefahren, nicht aus dem Plan uebernommen"
    - "Aufgeloeste Tabellen taugen nicht als Beleg-Beispiel: ein Beispiel, das auf ein verschwundenes Objekt zeigt, wird ersetzt statt mit einer Fussnote am Leben gehalten"
    - "Eine befristete Invariante (Zaehlprobe) wird nicht geloescht, sondern als historisch markiert UND durch eine dauerhaft auswertbare ersetzt — sonst laeuft der naechste Datenaudit ins Leere"

key-files:
  created: []
  modified:
    - docs/architecture/04-auth.md
    - docs/architecture/04-auth_de.md
    - docs/architecture/diagrams/service-graph-runtime.mmd
    - docs/features/F12-auth-session.md
    - docs/features/F12-auth-session_de.md
    - docs/architecture/03-data-model.md
    - docs/architecture/03-data-model_de.md
    - docs/architecture/diagrams/db-schema-er.mmd
    - docs/features/F01-employee-management.md
    - docs/features/F01-employee-management_de.md
  deleted: []
  already-correct-untouched:
    - docs/architecture/02-service-tiers.md
    - docs/architecture/02-service-tiers_de.md
    - docs/arc42/05-building-block-view.md

key-decisions:
  - "Der PersonService-Knoten im Runtime-Diagramm bekommt BEWUSST keine Kante zu PersonDao/PermissionService — der Plan verlangte sie, aber die Diagramm-Konvention im Dateikopf blendet DAOs und PermissionService ausdruecklich aus; stattdessen erklaert ein Kommentarblock die Kantenlosigkeit (siehe Deviation 1)"
  - "02-service-tiers{,_de}.md und arc42/05 blieben unveraendert — 61-01 bzw. 61-05 hatten sie bereits korrekt nachgezogen; der Plan haette hier ein Rewrite verlangt, das nur Rauschen erzeugt haette"
  - "Das FK-Beispiel wurde ersetzt statt annotiert: sales_person_unavailable.sales_person_id ist ein lebender Fremdschluessel und traegt denselben Beleg"
  - "Die Phase-60-Zaehlprobe bleibt im Text stehen (sie beschreibt eine reale Migration), wird aber als nicht mehr auswertbar markiert und um die dauerhafte Invariante ergaenzt"
  - "CURRENT_SNAPSHOT_SCHEMA_VERSION bleibt 13 — reiner Doku-Plan, kein persistierter value_type beruehrt"

patterns-established:
  - "Sicherheits-Konventionen werden als dauerhafte Regel formuliert ('Any future rework must keep the decision inside the query'), nicht als Phasen-Notiz — eine Phasen-Notiz liest die naechste Phase als erledigt und ignoriert sie"

requirements-completed: [PERS-05]

coverage:
  - id: D1
    description: "Die Service-Tier-Doku fuehrt PersonService als Basic Service mit vollstaendigem Dependency-Schnitt — in beiden Sprachen"
    requirement: PERS-05
    verification:
      - kind: other
        ref: "grep -c 'PersonService' docs/architecture/02-service-tiers{,_de}.md == 3 je Datei; Dep-Liste identisch zum gen_service_impl!-Block in service_impl/src/person.rs (PersonDao, PermissionService, ClockService, UuidService, TransactionDao)"
        status: pass
    human_judgment: false
  - id: D2
    description: "Die Auth-Doku beschreibt die Selbst-Zugriffs-Pruefung als SQL-Join ueber person_id und benennt die fail-open-Falle — in beiden Sprachen, als dauerhafte Konvention"
    requirement: PERS-05
    verification:
      - kind: other
        ref: "grep -c 'person_id' docs/architecture/04-auth{,_de}.md == 6 je Datei; Abschnitt §Self-access gate / §Selbst-Zugriffs-Gate zwischen 'Roles & privileges' und 'Session management'"
        status: pass
      - kind: other
        ref: "Query im Doku-Codeblock byte-gleich zu dao_impl_sqlite/src/sales_person.rs:446"
        status: pass
    human_judgment: false
  - id: D3
    description: "Die Datenmodell-Doku und das ER-Diagramm zeigen die aufgeloeste Verknuepfungstabelle nicht mehr als laufende Wahrheit"
    requirement: PERS-05
    verification:
      - kind: other
        ref: "grep -rni 'running truth|laufende Wahrheit' docs/ — keine Treffer mehr"
        status: pass
      - kind: other
        ref: "db-schema-er.mmd fuehrt keine Entitaet der Tabelle; PERSON ||--o| SALES_PERSON und PERSON ||--o| USER sind gezeichnet (bereits 61-05, hier nur der Kopf-Datumsstand nachgezogen)"
        status: pass
    human_judgment: false
  - id: D4
    description: "Der FK-Durchsetzungs-Beleg steht auf einem weiterhin existierenden Fremdschluessel und ist empirisch verifiziert"
    requirement: PERS-05
    verification:
      - kind: other
        ref: "Wegwerf-DB mit voller Migrationskette: PRAGMA foreign_key_list(sales_person_unavailable) listet sales_person_id -> sales_person(id); Insert mit zufaelliger sales_person_id scheitert an 'FOREIGN KEY constraint failed'"
        status: pass
    human_judgment: false
  - id: D5
    description: "Die Phase-60-Zaehlprobe ist als historisch markiert und durch eine dauerhaft auswertbare Invariante ersetzt"
    requirement: PERS-05
    verification:
      - kind: other
        ref: "grep -n 'This check is historical' / 'Diese Probe ist historisch' in 03-data-model{,_de}.md"
        status: pass
      - kind: other
        ref: "Die drei Invarianten-Queries gegen die Wegwerf-DB ausgefuehrt — alle drei liefern 0"
        status: pass
    human_judgment: false
  - id: D6
    description: "Jede geaenderte foo.md hat ihr foo_de.md im selben Commit"
    requirement: PERS-05
    verification:
      - kind: other
        ref: "jj status je Commit: 04-auth + 04-auth_de + F12 + F12_de (Commit 1); 03-data-model + _de + F01 + _de (Commit 2); die beiden .mmd sind einsprachig (Diagramme haben kein _de-Gegenstueck)"
        status: pass
    human_judgment: false
  - id: D7
    description: "Projekt-Gates gruen"
    requirement: PERS-05
    verification:
      - kind: other
        ref: "cargo test --workspace — 0 failed (u.a. 885 service_impl, 89 shifty_bin, 3x11 person-Module)"
        status: pass
      - kind: other
        ref: "cargo clippy --workspace -- -D warnings — Exit-Code 0"
        status: pass
      - kind: other
        ref: "grep -rn 'sales_person_user' dao/src dao_impl_sqlite/src service_impl/src — keine Treffer (SC2)"
        status: pass
    human_judgment: false

# Metrics
duration: 30min
completed: 2026-09-17
status: complete
---

# Phase 61 Plan 06: Docs-Freshness-Gate nachgezogen Summary

**Der Selbst-Zugriffs-Pfad ist erstmals ueberhaupt in der Architektur-Doku beschrieben — inklusive der dauerhaften Konvention, dass der Identitaetsvergleich im SQL bleiben muss —, und die nicht mehr auswertbare Phase-60-Zaehlprobe ist durch eine Invariante ersetzt, die sich gegen jede laufende Datenbank auditieren laesst.**

## Performance

- **Duration:** ~30 min
- **Completed:** 2026-09-17
- **Tasks:** 2
- **Files modified:** 10 Doku-Dateien (davon 4 Sprachpaare + 2 einsprachige Diagramme)
- **Files deliberately left untouched:** 3 (bereits korrekt, siehe unten)

## Was bereits korrekt war — und deshalb nicht angefasst wurde

Der Plan war vor den Wellen 1-5 geschrieben und verlangte Arbeit, die diese
Wellen als eigene Docs-Freshness-Pflicht schon erledigt hatten. Vor jeder
Aenderung wurde der Ist-Zustand geprueft. **Unveraendert geblieben, weil
bereits richtig:**

| Datei | Wer hatte es erledigt | Befund |
| --- | --- | --- |
| `docs/architecture/02-service-tiers.md` + `_de.md` | 61-01 | `PersonService`-Bullet unter „Basic Services" vorhanden, Dep-Liste vollstaendig und wortgleich zum `gen_service_impl!`-Block, Zyklus-Begruendung und Feature-Link auf F01 §4.5 gesetzt. Nichts zu tun. |
| `docs/arc42/05-building-block-view.md` | 61-05 | Zeile „Employee" nennt `person`, die aufgeloeste Tabelle ist raus. Nichts zu tun. |
| `docs/architecture/diagrams/db-schema-er.mmd` (Entitaeten/Kanten) | 61-05 | Entitaet der Verknuepfungstabelle entfernt, `PERSON ||--o| SALES_PERSON` und `PERSON ||--o| USER` gezeichnet, Kommentar auf D-60-03/D-61-07 gezogen. Nur der **Kopf-Datumsstand** war stale (siehe unten). |
| `docs/features/F01-*` §3 Tabellen, §7 Randfaelle, ASCII-Skizze | 61-04 + 61-05 | Merge/Split/409, Race-Abschnitt und Beziehungsskizze bereits auf `person_id` umgestellt. Nur drei Detailfehler korrigiert (siehe Task 2). |
| `docs/features/F12-*` §3 Person identity | 61-02 bis 61-05 | Migrations-Liste, „einzige Wahrheit"-Aussage und Self-Gate-Beschreibung korrekt. Nur Querverweis + eine falsche FK-Aussage ergaenzt bzw. korrigiert. |

Die **substanzielle** Restarbeit war dadurch enger, aber tiefer als geplant:
in `04-auth{,_de}.md` gab es zum Selbst-Zugriffs-Gate **ueberhaupt keinen
Abschnitt**, und in `03-data-model{,_de}.md` standen zwei Aussagen, die zwar
nicht mehr „laufende Wahrheit" behaupteten, aber praktisch unbrauchbar waren
(ein Beleg-Beispiel auf einer verschwundenen Tabelle und eine Zaehlprobe, die
niemand mehr ausfuehren kann).

## Task Commits

1. **Task 1: Auth-Doku, F12-Querverweis, Runtime-Diagramm** — `024eba93` (docs)
2. **Task 2: Datenmodell-Invariante, lebendes FK-Beispiel, F01-Kohaerenz** — `1a79fc64` (docs)

## Accomplishments

### Task 1 — der Self-Gate ist jetzt dokumentiert

`docs/architecture/04-auth.md` und `_de.md` bekommen einen neuen Abschnitt
zwischen „Roles & privileges" und „Session management". Jede Aussage darin ist
am Code verifiziert:

- **HR ∨ self, und wie die beiden Haelften zusammenkommen.** Die Aufrufer
  fahren `tokio::join!(check_permission(HR), verify_user_is_sales_person(...))`
  und werten `hr.or(sp)?` aus — belegt an `service_impl/src/absence.rs:109-119`.
  Ein HR-Benutzer umgeht das Gate ueber sein Privileg; nur ein Nicht-HR-Aufrufer
  haengt tatsaechlich daran.
- **Fail-closed-Verhalten.** `verify_user_is_sales_person` liefert
  `ServiceError::Forbidden`, sobald eine der beiden Seiten `None` ist
  (`service_impl/src/sales_person.rs:402-428`).
- **Die Query im Codeblock ist byte-gleich zur Implementierung**
  (`dao_impl_sqlite/src/sales_person.rs:446`), nicht paraphrasiert.
- **Die Konvention ist als Dauerregel formuliert**, nicht als Phasen-Notiz:
  beide `person_id`-Spalten sind schema-seitig nullable, ein Rust-seitiger
  `Option<Uuid>`-Vergleich wertet `None == None` als `true` und ginge damit
  fail-open — zwei unverknuepfte Zeilen wuerden aufeinander matchen. Der Text
  schliesst mit „Any future rework of these paths must keep the decision inside
  the query." Das ist die Mitigation zu **T-61-21**: eine Phasen-Notiz liest die
  naechste Phase als erledigt, eine Konvention nicht.
- **Testverweise mit echten Namen**: `self_may_access_own_sales_person`,
  `foreign_sales_person_rejected`, `unlinked_user_rejected` als die drei
  Kernfaelle, plus `null_person_id_does_not_match_null_person_id` fuer genau
  den fail-open-Fall und
  `soft_geloeschte_sales_person_behaelt_selbstzugriff` fuer die
  Soft-Delete-Eigenschaft.

`F12{,_de}.md` bekommt den Querverweis auf diesen Abschnitt und den Verweis auf
`service_impl/src/test/person_self_gate.rs`.

Im Runtime-Diagramm erklaert ein Kommentarblock, warum der `PersonService`-Knoten
bewusst **kantenlos** ist und warum insbesondere keine Kante zu oder von
`SalesPersonService` existiert — genau deren Abwesenheit ist die
Tier-Eigenschaft (**T-61-22**).

### Task 2 — die Datenmodell-Doku wird wieder benutzbar

- **Der FK-Beleg steht jetzt auf einem lebenden Fremdschluessel.** Der Abschnitt
  „Foreign keys" belegte die aktive FK-Durchsetzung an den beiden
  FK-Deklarationen der inzwischen aufgeloesten Verknuepfungstabelle — 61-05 hatte
  das mit einer Fussnote entschaerft, aber das Beispiel zeigte weiter auf ein
  nicht mehr existierendes Objekt. Ersetzt durch
  `sales_person_unavailable.sales_person_id → sales_person(id)`
  (`migrations/sqlite/20240731043118_...sql:14`), **empirisch nachgefahren**
  gegen eine Wegwerf-DB mit voller Migrationskette:
  `PRAGMA foreign_key_list` listet den FK, ein Insert mit zufaelliger
  `sales_person_id` scheitert an `FOREIGN KEY constraint failed`.
- **Die Phase-60-Zaehlprobe ist als historisch markiert** — mit der Begruendung,
  *warum* sie nicht mehr auswertbar ist (ihr letzter Term zaehlt Zeilen einer
  aufgeloesten Tabelle) und mit dem Hinweis, dass der Phase-60-Nachweis in
  `shifty_bin/src/person_link_migration_test.rs` ueber den gefilterten
  `Migrator` erhalten bleibt. Das ist die Mitigation zu **T-61-23**.
- **Neuer Abschnitt „Die dauerhaft pruefbare Invariante (ab Phase 61)"** mit drei
  auditierbaren SQL-Proben (`sales_person` ohne `person_id` = 0, `user` ohne
  `person_id` = 0, `person` ohne Traeger = 0). Alle drei wurden gegen die
  migrierte Wegwerf-DB ausgefuehrt und liefern `0`. In Worten: jede
  `sales_person`- und jede `user`-Zeile traegt genau ein `person_id`, und jede
  `person`-Zeile hat mindestens einen Traeger.
- **Die Schema-/Code-Asymmetrie ist benannt:** `person_id` bleibt nullable und
  ohne FK-Deklaration (SQLite kann ohne Table-Rebuild nicht auf `NOT NULL`
  verschaerfen, und ein Rebuild von `sales_person` muesste jede referenzierende
  Tabelle neu verdrahten). Die „nie NULL"-Haelfte traegt seit Phase 61 der Code
  (D-61-01) — mit dem ausdruecklichen Hinweis, dass ein dritter Anlage-Pfad das
  von Hand einhalten muss, weil die Datenbank das Versaeumnis nicht abfaengt.
- **F01** wurde an drei Stellen korrigiert (siehe Deviations 2 und 3) und um den
  fehlenden **v2.7-Eintrag** in §9 Versionshistorie ergaenzt (Phase 60:
  `person` + Backfill; Phase 61: Aufloesung, Merge/Split/409, `PersonService`).
  Der v0.1-Eintrag bleibt als historische Aussage stehen.

## Decisions Made

- **Kein Rewrite dessen, was schon stimmt.** Der Plan verlangte Arbeit an
  `02-service-tiers{,_de}.md` und `arc42/05`, die 61-01 bzw. 61-05 bereits
  korrekt erledigt hatten. Ein weiterer Durchgang haette bestenfalls nichts
  geaendert und schlimmstenfalls praezise Formulierungen verwaessert. Der
  Befund steht oben in der Tabelle, damit die Auslassung nachvollziehbar ist
  und nicht als Vergessen gelesen wird.
- **Das FK-Beispiel ersetzen statt annotieren.** 61-05 hatte die Stelle mit
  „**The table itself is gone since Phase 61**" entschaerft — richtig, aber ein
  Leser, der die Beobachtung nachvollziehen will, findet die Tabelle nicht.
  Ein lebender FK traegt denselben Beleg und ist reproduzierbar.
- **Die Zaehlprobe nicht loeschen.** Sie beschreibt eine reale
  Migrations-Assertion, die weiterhin in der Datei steht und bei einem
  Alt-Deploy noch laeuft. Geloescht waere die Migration unerklaert; unmarkiert
  waere sie eine Falle. Markiert plus Ersatz-Invariante ist die einzige Variante,
  die beides loest.
- **`CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 13.** Reiner Doku-Plan, keine der
  vier Bump-Klauseln aus `CLAUDE.md` greift.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Kein `PersonService → PersonDao`/`PermissionService`-Kantenpaar im Runtime-Diagramm**

- **Found during:** Task 1
- **Issue:** Der Plan verlangt woertlich „einen Knoten `PersonService` … mit
  Kante zu `PersonDao` und zu `PermissionService`, analog zum
  `RebookingBatchService`-Knoten". Beides ist im Diagramm nicht moeglich:
  `PersonDao` ist **kein Knoten** (der Dateikopf blendet DAOs ausdruecklich aus:
  „Nur Domain-Service-Kanten sind gezeichnet"), und der
  `RebookingBatchService`-Knoten — vom Plan als Vorbild benannt — hat aus genau
  diesem Grund selbst **keine** solche Kante. Eine Kante zu `PermissionService`
  zu ziehen waere zusaetzlich inkonsistent, weil dann rund 25 weitere
  Basic-Service-Knoten dieselbe Kante brauchten und der Graph unlesbar wuerde —
  wovor der Dateikopf warnt.
- **Fix:** Der Knoten (bereits von 61-01 angelegt) bleibt kantenlos und bekommt
  stattdessen einen Kommentarblock nach dem Muster des
  `RebookingBatchService`-Kommentars: er nennt die fuenf Deps, erklaert die
  Ausblendung und haelt ausdruecklich fest, dass die Abwesenheit einer Kante zu
  `SalesPersonService` die Tier-Eigenschaft ist. Die Plan-Absicht (T-61-22:
  Phase 62/63 soll `PersonService` keinen Domain-Service unterschieben) ist
  damit staerker bedient als durch die woertliche Umsetzung.
- **Files modified:** `docs/architecture/diagrams/service-graph-runtime.mmd`
- **Verification:** Akzeptanzkriterium „keine Kante zwischen `PersonService` und
  `SalesPersonService`" erfuellt (`grep` liefert nichts); Knoten vorhanden.
- **Committed in:** `024eba93`

**2. [Rule 1 - Bug] Blockquote zerbrach die Tabellenliste in F01 §3**

- **Found during:** Task 2
- **Issue:** 61-05 hatte den Aufloesungs-Hinweis als Blockquote **zwischen**
  zwei Zeilen der Markdown-Tabelle eingefuegt. Markdown beendet eine Tabelle an
  der ersten Nicht-Tabellenzeile: die drei folgenden Zeilen
  (`employee_work_details`, `sales_person_unavailable`,
  `sales_person_shiftplan`) rendern seither als Rohtext mit Pipes, nicht als
  Tabellenzeilen. Betraf **beide** Sprachfassungen.
- **Fix:** Der Blockquote steht jetzt **nach** der vollstaendigen Tabelle.
  Inhalt unveraendert.
- **Files modified:** `docs/features/F01-employee-management{,_de}.md`
- **Verification:** Tabelle hat wieder fuenf durchgehende Zeilen; der
  Blockquote folgt als eigener Block.
- **Committed in:** `1a79fc64`

**3. [Rule 1 - Bug] Zwei Behauptungen, die seit 61-05 bzw. seit Plan 60-04 falsch waren**

- **Found during:** Task 1 und Task 2
- **Issue:** (a) `F01{,_de}.md` §2.1 behauptete, `person_self_gate` fahre „eine
  byte-identische Vorher/Nachher-Permission-Matrix gegen **eine** Fixture-DB
  [D-61-09]". 61-05 hat genau diese Matrix abgebaut (die „Vorher"-Haelfte ist
  nach dem `DROP` nicht mehr herstellbar) — die Aussage verwies auf Testcode,
  der nicht mehr existiert. (b) `F12{,_de}.md` §3 behauptete, „die
  FK-Durchsetzung ist in diesem Repo ausgeschaltet [D-60-06]". Genau diese
  Aussage hat `03-data-model` schon in Plan 60-04 als falsch widerrufen (sqlx
  setzt `PRAGMA foreign_keys = ON` pro Verbindung); F12 war nie nachgezogen
  worden. Zwei Architektur-Dokumente widersprachen sich damit direkt.
- **Fix:** (a) Der Satz beschreibt die Matrix jetzt als befristetes, in Phase 61
  abgebautes Artefakt und nennt die 11 verbliebenen Dauer-Regressionstests.
  (b) Die FK-Aussage ist korrigiert: `user.person_id` traegt keine
  `FOREIGN KEY`-Deklaration (deshalb der Index als Durchsetzung), aber die
  FK-Durchsetzung als solche ist aktiv; D-60-06 bleibt als Scope-Entscheidung
  gueltig, seine Praemisse ueber den Ist-Zustand war falsch. Mit Querverweis auf
  `03-data-model` §Foreign keys.
- **Files modified:** `docs/features/F01-employee-management{,_de}.md`,
  `docs/features/F12-auth-session{,_de}.md`
- **Verification:** 11 `#[tokio::test]` in `person_self_gate.rs` gezaehlt;
  FK-Aussage empirisch nachgefahren (siehe D4).
- **Committed in:** `024eba93` (F12), `1a79fc64` (F01)

**4. [Rule 1 - Bug] Stale Datumsstand im ER-Diagramm-Kopf**

- **Found during:** Task 2
- **Issue:** Der Kopf sagte „STATUS-QUO nach allen Migrationen bis 2026-09-16".
  Die Drag-Migration `20260917000000_drop-sales-person-user.sql` ist vom
  17.09.; 61-05 hatte den Diagramm-**Inhalt** korrekt nachgezogen, den
  Datumsstand aber nicht. Ein Leser haette den Inhalt fuer veraltet halten und
  die geloeschte Entitaet „zurueckergaenzen" koennen.
- **Fix:** Datum auf 2026-09-17 und die letzte Migration namentlich genannt.
- **Files modified:** `docs/architecture/diagrams/db-schema-er.mmd`
- **Verification:** Letzte Migration in `migrations/sqlite/` ist
  `20260917000000_drop-sales-person-user.sql`.
- **Committed in:** `1a79fc64`

**5. [Rule 1 - Bug] „Scope boundary" las sich als Gegenwartsaussage**

- **Found during:** Task 2
- **Issue:** Der Bullet begann mit „The former 1:1 link table **stayed** through
  Phase 60. It was the running truth…". Grammatisch Vergangenheit, aber die
  Fett-Auszeichnung lag auf „stayed", und das must-have des Plans ist
  ausdruecklich „kein `docs/`-Dokument behauptet mehr, die Tabelle … sei die
  laufende Wahrheit". Ein ueberfliegender Leser haette das Gegenteil mitgenommen.
- **Fix:** Umgestellt auf „**The 1:1 link table no longer exists.** During
  Phase 60 it was still there …" plus dem expliziten Satz, dass der folgende
  Absatz Historie ist und keine Beschreibung des aktuellen Schemas. Beide
  Sprachen.
- **Files modified:** `docs/architecture/03-data-model{,_de}.md`
- **Verification:**
  `grep -rni 'running truth|laufende Wahrheit' docs/` — keine Treffer.
- **Committed in:** `1a79fc64`

---

**Total deviations:** 5 auto-fixed (1 blocking, 4 Doku-Wahrheits-Bugs)
**Impact on plan:** Kein Scope Creep, kein Rule-4-Fall. Deviation 1 ist eine
Plan-Anweisung, die der Diagramm-Konvention widerspricht und deren Absicht
anders besser erfuellt wird; die uebrigen vier sind falsche oder irrefuehrende
Aussagen, die beim Verifizieren des Ist-Zustands aufgefallen sind — genau die
Klasse Drift, die dieser Plan beseitigen soll.

## TDD Gate Compliance

Nicht anwendbar: reiner Doku-Plan, keine Task traegt `tdd="true"`, es wurde
keine Zeile Rust geaendert. Die Projekt-Gates (`cargo test --workspace`,
`cargo clippy --workspace -- -D warnings`) wurden trotzdem gefahren, weil die
Akzeptanzkriterien von Task 2 sie ausdruecklich verlangen — beide gruen.

## Issues Encountered

- **`cargo` nicht auf dem PATH.** Alle Gates liefen ueber
  `nix develop --command bash -c '…'` (liefert `cargo`, `clippy`, `sqlx-cli`,
  `sqlite3`).
- **Der Plan war aelter als die Wellen, die er dokumentieren soll.** Er verlangt
  Aenderungen an Stellen, die 61-01 bis 61-05 als eigene
  Docs-Freshness-Pflicht bereits korrigiert hatten. Der Umgang damit ist oben
  in der Tabelle „Was bereits korrekt war" festgehalten — jede Auslassung ist
  begruendet, nicht uebersehen.
- **Der Plan nennt Zeilennummern, die inzwischen verschoben sind.** Die
  `<read_first>`-Angaben (z. B. `F01:80-90, 185-205`) treffen den aktuellen
  Stand nicht mehr, weil 61-04 und 61-05 Absaetze ergaenzt haben. Navigation
  lief deshalb ueber `grep` auf Ueberschriften statt ueber Zeilennummern.
- **Kein `rustfmt` in der Dev-Shell** — hier ohne Belang, es wurde keine
  Rust-Datei angefasst.

## Known Stubs

Keine. Dieser Plan erzeugt keinen Code und laesst keine unverdrahteten Pfade
zurueck.

## Threat Flags

Keine neue Sicherheitsflaeche. Kein Endpunkt, kein Auth-Pfad, kein Dateizugriff
und keine Schema-Aenderung kommt hinzu. Die drei `mitigate`-Dispositionen des
`<threat_model>` sind bedient:

| Threat | Mitigation im Ergebnis |
| --- | --- |
| T-61-21 (EoP, Auth-Doku) | Die „Join statt Options-Vergleich"-Regel steht in beiden Sprachen als **dauerhafte Konvention** mit ausformulierter fail-open-Konsequenz und dem Satz „Any future rework of these paths must keep the decision inside the query". |
| T-61-22 (Spoofing, Tier-Doku) | Dep-Liste wortgetreu aus `service_impl/src/person.rs`; die Abwesenheit einer `PersonService`↔`SalesPersonService`-Kante ist im Diagramm zusaetzlich **kommentiert**, nicht nur faktisch gegeben. |
| T-61-23 (Repudiation, Datenmodell) | Zaehlprobe als historisch markiert **und** durch drei auditierbare Queries ersetzt, die gegen eine real migrierte DB ausgefuehrt wurden. |

## User Setup Required

None. Reiner Doku-Plan.

Das bestehende **Deploy-Gate aus Phase 60/61** bleibt unveraendert in Kraft
(Datei-Backup der Produktions-SQLite vor dem Deploy, CR-02-Pre-Flight-Query
**vor** dem Deploy) — siehe `61-05-SUMMARY.md` §User Setup Required.

## Next Phase Readiness

- **Phase 61 ist damit abgeschlossen.** Success Criterion 5 ist vollstaendig:
  `PersonService` steht im Basic-Tier, clippy ist gruen, und die Doku ist in
  beiden Sprachen nachgezogen.
- **Phase 62/63** finden eine Doku vor, die drei Fragen ohne Code-Lektuere
  beantwortet: wo `PersonService` in der DI-Reihenfolge steht und warum er
  keinen Domain-Service konsumieren darf, wie der Selbst-Zugriff entschieden
  wird und warum diese Entscheidung nicht nach Rust wandern darf, und welche
  Invariante ein Datenaudit gegen `person`/`sales_person`/`user` pruefen kann.
- **Offen (unveraendert, kein Blocker):** das CR-02-Deploy-Gate aus Phase 60.

## Self-Check: PASSED

- Alle zehn geaenderten Dateien existieren und tragen die behaupteten
  Abschnitte:
  - `docs/architecture/04-auth.md:95` — `## Self-access gate for sales-person data`
  - `docs/architecture/04-auth_de.md:95` — `## Selbst-Zugriffs-Gate auf Sales-Person-Daten`
  - `docs/architecture/03-data-model.md:320` / `_de.md:337` — „This check is
    historical" / „Diese Probe ist historisch"
- `grep -c 'person_id' docs/architecture/04-auth{,_de}.md` == `6` je Datei.
- `grep -c 'PersonService' docs/architecture/02-service-tiers{,_de}.md` == `3`
  je Datei (unveraendert, bereits von 61-01).
- Keine Kante zwischen `PersonService` und `SalesPersonService` im
  Runtime-Diagramm.
- `grep -rni 'running truth|laufende Wahrheit' docs/` — keine Treffer.
- `grep -rn 'sales_person_user' dao/src dao_impl_sqlite/src service_impl/src` —
  keine Treffer (SC2 haelt).
- Beide Commits sind im Repo auffindbar: `024eba93`, `1a79fc64`.
- `cargo test --workspace` — 0 failed; `cargo clippy --workspace -- -D warnings`
  — Exit-Code 0.
- `CURRENT_SNAPSHOT_SCHEMA_VERSION` steht weiterhin auf `13`.

---
*Phase: 61-personservice-aufl-sung-von-sales-person-user*
*Completed: 2026-09-17*
