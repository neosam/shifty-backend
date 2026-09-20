---
phase: 62-personen-verzeichnis-eine-liste-statt-tabs
plan: 06
subsystem: docs
tags: [docs-freshness-gate, service-tiers, mermaid, bilingual-sync, wr-02, admin-gate]

# Dependency graph
requires:
  - phase: 62-personen-verzeichnis-eine-liste-statt-tabs
    provides: "62-01 — `PersonDirectoryDao` plus der WR-02-Fix in `users_without_sales_person`; F01-Datenmodell-Sektion bereits geschrieben"
  - phase: 62-personen-verzeichnis-eine-liste-statt-tabs
    provides: "62-03 — `PersonDirectoryService` mit Admin-Gate; F01 §4.6 bereits angelegt"
  - phase: 62-personen-verzeichnis-eine-liste-statt-tabs
    provides: "62-05 — REST-Endpoint, DI-Verdrahtung in `shifty_bin/src/main.rs`, Dev-Proxy; Tier-Doku ausdruecklich an 62-06 delegiert"
provides:
  - "F01 §3 — verallgemeinerte ON-vs-WHERE-Regel mit namentlichem WR-02-Regressionstest (EN+DE)"
  - "F01 §4.6 — bewusste Gate-Verschaerfung, Tier-Absatz, REST-Absatz, Phase-63-Abgrenzung (EN+DE)"
  - "F01 §5.4 — REST-Tabellen-Eintrag `GET /person-directory` inkl. OpenAPI-Aussage (EN+DE)"
  - "F01 §6 — Dev-Proxy-Eintrag in der Proxy-Liste (EN+DE)"
  - "02-service-tiers — `PersonDirectoryService` in der Business-Logic-Liste + Klarstellung zur Faustregel in §Rules (EN+DE)"
  - "service-graph-runtime.mmd — Knoten `PersonDirectory` im Business-Teilgraph, zwei Kommentarbloecke"
  - "EN/DE-Drift aus Phase 56 geschlossen: `VoluntaryRebookingScheduler` fehlte in der deutschen Tier-Doku"
affects: [62-07-verifikation, 63-personen-anlage]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Doku-Behauptung = Code-Behauptung: jeder genannte Symbolname wird vor dem Commit per grep gegen den Quellcode belegt"
    - "Bilinguale Struktur-Paritaet als messbares Kriterium (Ueberschriften- und Aufzaehlungszahl), nicht als Absichtserklaerung"
    - "Ein Fehlverhalten, das in der Doku als gewollt beschrieben steht, wird durch einen namentlich genannten Regressionstest geerdet"

key-files:
  created: []
  modified:
    - docs/features/F01-employee-management.md
    - docs/features/F01-employee-management_de.md
    - docs/architecture/02-service-tiers.md
    - docs/architecture/02-service-tiers_de.md
    - docs/architecture/diagrams/service-graph-runtime.mmd

key-decisions:
  - "Die REST-Oberflaeche bekommt einen eigenen Abschnitt §5.4 statt nur eines Absatzes in §4.6. §5 ist die kanonische REST-Sektion von F01; ein Leser, der dort nachschlaegt, haette den Endpoint sonst nicht gefunden — das waere Drift anderer Art. Beide Sprachfassungen bekommen die Ueberschrift, die Paritaet bleibt bei 33/33."
  - "Die Crate-Liste im F01-Kopf wird nicht nur um `person_directory`, sondern auch um `person` ergaenzt. §4.5 dokumentiert den `PersonService` seit Phase 61, ohne dass der Kopf ihn je nannte — eine unvollstaendige Liste im Dateikopf ist dieselbe Fehlinformation, die dieser Plan an anderer Stelle beseitigt."
  - "Der fehlende `VoluntaryRebookingScheduler`-Punkt in der deutschen Tier-Doku wird nachgetragen statt ignoriert. Das Akzeptanzkriterium verlangt gleiche Aufzaehlungszahl in beiden Fassungen; ohne den Nachtrag waere es nur durch Weglassen in der englischen Fassung erfuellbar gewesen — also durch Erzeugen neuer Drift."
  - "Im Laufzeit-Graph bekommt `PersonDirectory` bewusst KEINE Kante. Die Deps (Verzeichnis-DAO, Permission, Transaction) sind nach der Kopf-Konvention ausgeblendet; die Tier-Einordnung folgt der Aggregat-Eigenschaft, nicht der Kantenzahl. Der Kommentarblock sagt das ausdruecklich, damit ein spaeterer Leser die Kantenlosigkeit nicht fuer einen Fehler haelt."
  - "Kein Build und kein Testlauf. Dieser Plan aendert ausschliesslich Markdown und Mermaid; `git diff --stat` ueber beide Commits enthaelt keine einzige `.rs`-, `.sql`- oder `.toml`-Datei. Ein `cargo test` haette hier keine Aussagekraft, die ein `grep` nicht schon hat."

patterns-established:
  - "Baseline-vor-Plan beziffern: Kriterien der Form 'groesser als vorher' werden nur belastbar, wenn der Ausgangswert vor der ersten Aenderung gemessen und im SUMMARY festgehalten wird."
  - "Vorgefundene Drift im eigenen Scope wird geschlossen, nicht umschifft: wenn ein Paritaets-Kriterium wegen Altlast nicht erfuellbar ist, ist die Altlast der Fix."

# PADM-01 / PADM-06 / PADM-08 bleiben bewusst Pending. Alle sieben Plaene dieser
# Phase fuehren sie im Frontmatter, ihre Aussagen sind nutzersichtbar ("ein Admin
# sieht eine Liste"), und ein reiner Doku-Plan loest keine nutzersichtbare
# Aussage ein. Sie gehoeren an den Phasen-Abschluss nach 62-07 — Praezedenz
# 62-01 (Deviation 4), 62-03, 62-05.
requirements-completed: []
requirements-advanced: [PADM-01, PADM-06, PADM-08]

coverage:
  - id: D1
    description: "Die Doku beschreibt den Kandidaten-Lesepfad mit dem korrigierten Soft-Delete-Verhalten und verankert den Regressionstest namentlich (EN+DE)"
    requirement: "PADM-01"
    verification:
      - kind: other
        ref: "grep -c 'user_mit_nur_soft_geloeschter_sales_person_ist_kandidat' docs/features/F01-employee-management.md == 1 und _de.md == 1"
        status: pass
      - kind: other
        ref: "grep -c 'users_without_sales_person' … == 2 in beiden Fassungen; die Tabellenzeile nennt die ON-Klausel"
        status: pass
    human_judgment: false
  - id: D2
    description: "Die Doku beschreibt Aggregat-DAO, Business-Logic-Service, Admin-Gate und Endpoint"
    requirement: "PADM-01"
    verification:
      - kind: other
        ref: "grep -c 'person_directory' docs/features/F01-employee-management.md == 17, _de.md == 17 (>= 3 verlangt)"
        status: pass
      - kind: other
        ref: "grep -ci 'admin' … EN 10 -> 22, DE 9 -> 17 (beide gestiegen)"
        status: pass
    human_judgment: false
  - id: D3
    description: "Die bewusste Gate-Verschaerfung ist mit Entscheidungsdatum, Quelle und beiden HTTP-Tests dokumentiert"
    requirement: "PADM-08"
    verification:
      - kind: other
        ref: "grep -c 'hr_ohne_admin_bekommt_403' … == 1 und grep -c 'admin_bekommt_200' … == 1 in beiden Fassungen"
        status: pass
    human_judgment: false
  - id: D4
    description: "Die Tier-Doku fuehrt den neuen Business-Logic-Service inklusive Konstruktions-Reihenfolge"
    requirement: "PADM-01"
    verification:
      - kind: other
        ref: "grep -c 'PersonDirectory' docs/architecture/02-service-tiers.md == 4, _de.md == 4; grep -ci 'basic' 12 -> 15 in beiden"
        status: pass
      - kind: other
        ref: "grep -n 'let person_service' = 1278 < 'let voluntary_stats_service' = 1348 < 'let person_directory_service' = 1371 in shifty_bin/src/main.rs"
        status: pass
    human_judgment: false
  - id: D5
    description: "Der Laufzeit-Service-Graph zeigt den Knoten im Business-Teilgraph und nicht im Basic-Teilgraph"
    requirement: "PADM-01"
    verification:
      - kind: other
        ref: "awk '/subgraph Business/,/^    end/' … | grep -c 'PersonDirectory' == 1"
        status: pass
      - kind: other
        ref: "awk '/subgraph Basic/,/^    end/' … | grep -c 'PersonDirectory' == 0"
        status: pass
    human_judgment: false
  - id: D6
    description: "Englische und deutsche Fassung sind strukturell synchron"
    verification:
      - kind: other
        ref: "diff <(grep -c '^#' F01…md) <(grep -c '^#' F01…_de.md) ohne Ausgabe (33/33); dito 02-service-tiers (7/7); BL-Aufzaehlung 14/14, Basic-Aufzaehlung 11/11"
        status: pass
    human_judgment: false
  - id: D7
    description: "Jede Doku-Behauptung ist gegen den Quellcode belegt"
    verification:
      - kind: other
        ref: "23 genannte Symbole, jedes mit >= 1 Quelldatei-Treffer (Tabelle unten)"
        status: pass
    human_judgment: false

# Metrics
duration: 18min
completed: 2026-09-20
status: complete
---

# Phase 62 Plan 06: Docs-Freshness-Gate-Nachzug Summary

**Die Feature-Doku beschreibt das Personen-Verzeichnis jetzt vollstaendig bis
zum Endpoint, die WR-02-Fehlinformation ist durch einen namentlich verankerten
Regressionstest ersetzt, und die Tier-Doku fuehrt den neuen Business-Logic-
Service samt einer Klarstellung, warum eine Basic-aussehende Dependency-Liste
ihn nicht zum Basic Service macht — alles in beiden Sprachen.**

## Performance

- **Duration:** 18 min
- **Started:** 2026-09-20T08:35:00Z
- **Completed:** 2026-09-20T08:53:00Z
- **Tasks:** 2/2
- **Files modified:** 5 (0 neu, 5 geaendert) — ausschliesslich unter `docs/`

## Accomplishments

- **WR-02 ist nicht mehr nur punktuell korrigiert, sondern als Regel
  festgehalten.** 62-01 hatte die Tabellenzeile zu
  `users_without_sales_person` berichtigt; dieser Plan ergaenzt den Absatz
  darunter, der die Regel verallgemeinert (Soft-Delete-Filter auf der
  optionalen Seite eines Outer Joins gehoeren in die `ON`-Klausel, sonst
  degeneriert der Join zum INNER JOIN), den Phase-61-Befund WR-02 mit seiner
  Konsequenz benennt (der Benutzer fiel aus **beiden** Kandidatenlisten und
  war nie wieder verknuepfbar) und
  `user_mit_nur_soft_geloeschter_sales_person_ist_kandidat` als Wachhund
  namentlich verankert.
- **F01 §4.6 ist komplett.** 62-03 hatte Trait-Auszug, Fat-Backend-Begruendung,
  Auth-Gate-Tabelle und Randfaelle geliefert. Dieser Plan ergaenzt die vier
  fehlenden Bausteine: die bewusste Verhaltensaenderung (mit Datum
  2026-09-16 und Quelle), den Tier-Absatz, den REST-Absatz und die
  Phase-63-Abgrenzung.
- **F01 §5.4 neu** — der Endpoint steht jetzt auch dort, wo ein Leser
  REST-Endpoints sucht, inklusive der Aussage, dass er — anders als die
  Employee-Work-Details-Handler eine Sektion darueber — in der OpenAPI-Doku
  registriert ist.
- **F01 §6** nennt den Dev-Proxy-Eintrag mit seiner Begruendung.
- **Tier-Doku (EN+DE)** fuehrt `PersonDirectoryService` mit Deps, Gate-Aussage
  (propagieren statt redigieren — ausdruecklich anders als
  `VoluntaryStatsService`) und Konstruktions-Reihenfolge; der
  `PersonService`-Eintrag notiert seinen ersten Konsument-Kontext.
- **§Rules Nr. 3 ist praezisiert.** Ohne diesen Zusatz widerspraeche der neue
  Eintrag der eigenen Faustregel des Dokuments.
- **Laufzeit-Graph** zeigt `PersonDirectory` im Business-Teilgraph, mit einem
  Kommentarblock zur bewussten Kantenlosigkeit; der Person-Knoten-Block ist
  von „Erster Konsument folgt in Phase 62/63" auf den tatsaechlichen Ausgang
  nachgezogen.
- **Eine Altlast aus Phase 56 geschlossen:** der deutschen Tier-Doku fehlte
  der `VoluntaryRebookingScheduler`-Eintrag.

## Task Commits

| Task | Name | Commit |
| --- | --- | --- |
| 1 | F01 — Kandidaten-Lesepfad korrigieren und das Personen-Verzeichnis dokumentieren (EN+DE) | `bbb66a34` |
| 2 | Tier-Doku und Laufzeit-Service-Graph (EN+DE) | `54f220a0` |

## Zitate, die die Akzeptanzkriterien verlangen

**Die korrigierte Kandidaten-Tabellenzeile (F01 §3, Zeile 261 EN / 262 DE):**

> `users_without_sales_person(tx)` | Login users whose person carries no
> *live* sales person — candidates for "add person". The sales-person side is
> restricted to non-deleted rows **inside the `ON` clause**
> (`AND sp.deleted IS NULL`), so a user whose only sales person was
> soft-deleted becomes a candidate again instead of dropping out of both
> candidate lists [D-62-05 / WR-02]. The `user` table itself still has no
> soft-delete column.

> `users_without_sales_person(tx)` | Login-User, deren Person keine *lebende*
> Sales Person trägt — Kandidaten für „Person hinzufügen". Die
> Sales-Person-Seite wird **in der `ON`-Klausel** auf nicht-gelöscht
> eingeschränkt (`AND sp.deleted IS NULL`), damit ein User, dessen einzige
> Sales Person soft-gelöscht wurde, wieder Kandidat ist statt aus beiden
> Kandidatenlisten zu fallen [D-62-05 / WR-02]. Die Tabelle `user` selbst hat
> weiterhin keine Soft-Delete-Spalte.

**Die Konstruktions-Reihenfolge im neuen Tier-Eintrag:**

> DI-constructed in `shifty_bin/src/main.rs` **after** the basic block, in
> particular strictly after `PersonService`, directly following
> `voluntary_stats_service`.

> DI-konstruiert in `shifty_bin/src/main.rs` **nach** dem Basic-Block,
> insbesondere strikt nach `PersonService`, direkt im Anschluss an
> `voluntary_stats_service`.

**Die Klarstellung zur Faustregel (§Rules / §Regeln Nr. 3):**

> **The rule of thumb is an indicator, not the decision.** The substantive
> question decides: does the service manage exactly *one* aggregate? A read
> aggregate that merges several domain objects into one view is Business-Logic
> even when its dependency list looks like that of a Basic service.

> **Die Faustregel ist ein Indikator, nicht die Entscheidung.** Es entscheidet
> die inhaltliche Frage: verwaltet der Service genau *ein* Aggregat? Ein
> Lese-Aggregat, das mehrere Fach-Objekte zu einer Sicht zusammenführt, ist
> Business-Logic — auch wenn seine Dependency-Liste wie die eines
> Basic-Service aussieht.

**Der `gen_service_impl!`-Block, gegen den der Tier-Eintrag abgeglichen ist**
(`sed -n '/gen_service_impl! {/,/^}/p' service_impl/src/person_directory.rs`):

```rust
gen_service_impl! {
    struct PersonDirectoryServiceImpl: PersonDirectoryService = PersonDirectoryServiceDeps {
        PersonDirectoryDao: PersonDirectoryDao<Transaction = Self::Transaction> = person_directory_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao
    }
}
```

Die Doku nennt genau diese drei Dependencies — `PersonDirectoryDao`,
`PermissionService`, `TransactionDao` — und keine vierte.

## Baselines und Messwerte

| Messung | Vorher | Nachher | Kriterium |
| --- | --- | --- | --- |
| `grep -c person_directory` F01 EN | 7 | **17** | >= 3 |
| `grep -c person_directory` F01 DE | 7 | **17** | >= 3 |
| `grep -ci admin` F01 EN | 10 | **22** | groesser |
| `grep -ci admin` F01 DE | 9 | **17** | groesser |
| `grep -c users_without_sales_person` F01 EN/DE | 1 / 1 | **2 / 2** | >= 1 |
| `grep -c user_mit_nur_soft_geloeschter…` F01 EN/DE | 0 / 0 | **1 / 1** | == 1 |
| `grep -c hr_ohne_admin_bekommt_403` F01 EN/DE | 0 / 0 | **1 / 1** | == 1 |
| `grep -c '^#'` F01 EN/DE | 32 / 32 | **33 / 33** | diff leer |
| `grep -c PersonDirectory` tiers EN/DE | 0 / 0 | **4 / 4** | >= 1 |
| `grep -ci basic` tiers EN/DE | 12 / 12 | **15 / 15** | groesser (EN) |
| `grep -c '^#'` tiers EN/DE | 7 / 7 | **7 / 7** | diff leer |
| BL-Aufzaehlungspunkte tiers EN/DE | 13 / 12 | **14 / 14** | gleich |
| Basic-Aufzaehlungspunkte tiers EN/DE | 11 / 11 | **11 / 11** | gleich |
| `grep -c PersonDirectory` Runtime-Graph | 0 | **3** | >= 1 |
| Business-Teilgraph `PersonDirectory` | 0 | **1** | == 1 |
| Basic-Teilgraph `PersonDirectory` | 0 | **0** | == 0 |

Die Aufzaehlungszahlen sind mit
`awk '/^## Business-Logic Services/,/^## Rules/' … | grep -c '^- '` gemessen
(die Bereichsgrenze heisst in der deutschen Fassung `^## Regeln`); der Bereich
schliesst die beiden einleitenden Definitions-Punkte mit ein, deshalb 14 statt
12 Service-Eintraegen.

## Symbol-Verifikation gegen den Quellcode

Jeder in der Doku genannte Symbolname hat mindestens einen Treffer im
Quellcode. Gemessen mit
`grep -rl "$s" dao dao_impl_sqlite service service_impl rest rest-types shifty_bin shifty-dioxus/src | wc -l`:

| Symbol | Quelldateien |
| --- | --- |
| `PersonDirectoryDao` | 6 |
| `list_directory_entries` | 4 |
| `all_user_roles` | 4 |
| `PersonDirectoryRow` | 4 |
| `PersonDirectoryService` | 6 |
| `PersonEntry` | 10 |
| `PersonKind` | 15 |
| `PersonEntryTO` | 8 |
| `PersonKindTO` | 5 |
| `PersonDirectoryApiDoc` | 2 |
| `get_person_directory` | 3 |
| `person_directory_service` | 3 |
| `users_without_sales_person` | 4 |
| `HR_PRIVILEGE` | 34 |
| `Route::SalesPersonDetails` | 2 |
| `Route::UserDetails` | 2 |
| `user_mit_nur_soft_geloeschter_sales_person_ist_kandidat` | 1 |
| `kandidat_trotz_gesetzter_person_id` | 1 |
| `soft_geloeschte_sales_person_laesst_benutzer_sichtbar` | 1 |
| `person_ohne_traeger_erscheint_nicht` | 1 |
| `list_ohne_admin_privileg_ist_forbidden` | 1 |
| `admin_bekommt_200` | 1 |
| `hr_ohne_admin_bekommt_403` | 1 |

Zusaetzlich gegen den Code belegt:

- Die Behauptung „beide Vorkommen tragen einen Zeilenkommentar mit
  D-62-05 / WR-02": `grep -rn 'WR-02' dao_impl_sqlite/src/` trifft
  `person.rs:177` und `person_directory.rs:103,107`.
- Die Behauptung zur Konstruktions-Reihenfolge:
  `let person_service` = Zeile 1278, `let voluntary_stats_service` = 1348,
  `let person_directory_service` = 1371 in `shifty_bin/src/main.rs`.
- Die Behauptung „`PersonDirectoryApiDoc` ist Teil des `ApiDoc`-Merge":
  `rest/src/lib.rs:640` `(path = "/person-directory", api = …)` und
  `rest/src/lib.rs:706` `.nest("/person-directory", …)`.
- Die Behauptung „Dev-Proxy-Eintrag": `shifty-dioxus/Dioxus.toml:118`
  `backend = "http://localhost:3000/person-directory"`.

## Bewusst NICHT angefasste Doku — mit Begruendung

| Datei | Begruendung |
| --- | --- |
| `docs/architecture/03-data-model.md` (+ `_de`) | Keine Migration in dieser Phase; das Schema ist unveraendert (CONTEXT.md, gesperrt). Der Docs-Gate-Trigger `migrations/sqlite/*.sql` hat nicht gefeuert. |
| `docs/architecture/diagrams/db-schema-er.mmd` | Dito — keine Tabelle, keine Spalte, kein Index hat sich geaendert. |
| `docs/architecture/04-auth.md` (+ `_de`) | `service_impl/src/permission.rs` wurde nicht geaendert; es gibt kein neues Privileg und keine neue Rolle. Das Verzeichnis nutzt das bestehende Literal `"admin"`. Die Verschaerfung betrifft ausschliesslich den neuen Endpoint und steht darum in F01 §4.6. |
| `docs/features/F12-auth-session.md` (+ `_de`) | Dito — kein neues Auth-Konzept, nur eine neue Anwendung eines bestehenden Gates. |
| `docs/features/F08-billing-period.md`, `docs/domain/billing-period.md` | `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **13**; keine der vier Bump-Klauseln aus CLAUDE.md greift (CONTEXT.md, gesperrt). Keine Version-Tabellen-Zeile ergaenzt. |
| `docs/features/F07-reporting-balance.md`, `docs/domain/time-accounting.md` | `service_impl/src/reporting.rs` ist unveraendert; keine Balance-Formel und keine Kategorie-Semantik beruehrt. |
| `docs/architecture/diagrams/service-graph-traits.mmd` | Die Trait-Deklarations-Sicht zeigt ubiquitaere Deps ebenfalls nicht; der neue Service fordert keinen Domain-Service, haette dort also denselben kantenlosen Knoten. Der Docs-Gate-Trigger nennt ausdruecklich nur den **Runtime**-Graphen. **[Zu pruefen]** beim naechsten Doku-Sweep, ob die Trait-Sicht generell nachgezogen werden soll — sie fuehrt auch `RebookingBatch`/`Person` nicht. |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — Doku-Drift] `VoluntaryRebookingScheduler` fehlte in der deutschen Tier-Doku**

- **Found during:** Task 2
- **Issue:** Die englische Business-Logic-Liste fuehrt seit Phase 56 einen
  `VoluntaryRebookingScheduler`-Eintrag, die deutsche nicht — 13 gegen 12
  Aufzaehlungspunkte. Das Akzeptanzkriterium dieses Tasks verlangt aber
  ausdruecklich, dass „die Zahl der Aufzaehlungspunkte in der
  Business-Logic-Liste in beiden gleich" ist. Mit nur je einem neuen Punkt
  waere das Kriterium gerissen (14 gegen 13). Es liesse sich sonst nur
  erfuellen, indem man den englischen Eintrag entfernt — also durch Erzeugen
  neuer Drift statt Beseitigung alter.
- **Fix:** Den fehlenden Eintrag in der deutschen Fassung nachgetragen,
  inhaltlich deckungsgleich zur englischen Fassung (Cron-Ausdruck, Deps,
  Boot-Reihenfolge nach `pdf_export_scheduler.start()`, Verweis auf F14 §9).
  Nach dem Nachtrag: 14 / 14.
- **Files modified:** `docs/architecture/02-service-tiers_de.md`
- **Commit:** `54f220a0`

**2. [Rule 2 — Projektvorgabe CLAUDE.md] Crate-Liste im F01-Kopf um `person` erweitert**

- **Found during:** Task 1
- **Issue:** Der Plan verlangt „Ergaenze die Crate-Liste im Dateikopf um die
  neuen Module". Beim Ergaenzen fiel auf, dass der Kopf auch `person` nie
  genannt hat, obwohl §4.5 den `PersonService` seit Phase 61 dokumentiert und
  §3 den `PersonDao`. Eine unvollstaendige Modulliste im Dateikopf ist
  dieselbe Art von Fehlinformation, die dieser Plan an anderer Stelle
  beseitigt.
- **Fix:** `person` **und** `person_directory` in allen fuenf Crate-Zeilen
  ergaenzt (`service`, `service_impl`, `dao`, `dao_impl_sqlite`, `rest` — bei
  `rest` nur `person_directory`, denn ein `rest::person` existiert nicht).
  Gegen den Code belegt: `rest/src/person.rs` gibt es nicht, `ls rest/src`
  fuehrt `person_directory.rs`.
- **Files modified:** beide F01-Fassungen
- **Commit:** `bbb66a34`

### Bewusste Scope-Erweiterung (dokumentiert)

**F01 §5.4 als eigener Abschnitt statt nur als Absatz in §4.6.** Der Plan
listet den REST-Teil als Aufzaehlungspunkt der neuen §4.6-Struktur
(„REST-Absatz: Methode, Pfad, Antwort-Typ, OpenAPI-Registrierung …"). Dieser
Absatz **ist** in §4.6 geschrieben. Zusaetzlich bekommt §5 — die kanonische
REST-Endpoint-Sektion von F01 mit ihrer Methode/Pfad/DTO/Fehler-Tabelle —
einen Unterabschnitt §5.4. Begruendung: §5 listet heute jeden REST-Endpoint
des Clusters; ein Leser, der dort nachschlaegt, haette `/person-directory`
sonst nicht gefunden, und §5 waere ab diesem Commit nachweislich
unvollstaendig. Beide Sprachfassungen bekommen den Abschnitt, die
Ueberschriften-Paritaet bleibt erfuellt (33 / 33). Der §4.6-Absatz verweist
mit „Details in §5.4" dorthin, damit keine zweite Wahrheit entsteht.

### Acceptance-Criterion-Abweichungen

Keine. Alle Kriterien beider Tasks sind wie geschrieben erfuellt; die
Zaehlwerte stehen in der Baseline-Tabelle oben. Zwei Praezisierungen zum
Verstaendnis, keine Substitutionen:

- Das Kriterium „`grep -ci 'basic' docs/architecture/02-service-tiers.md` ist
  groesser als vor dem Plan" nennt nur die englische Fassung. Gemessen wurde
  beides: 12 → 15 in **beiden** Fassungen.
- Die Aufzaehlungszaehlung der Business-Logic-Liste braucht in der deutschen
  Fassung `^## Regeln` als Bereichsende statt `^## Rules`. Das ist keine
  Korrektur eines kaputten Kommandos, sondern die uebersetzte
  Ueberschrift — im SUMMARY beziffert, wie das Kriterium verlangt.

## Verification Results

| Gate | Kommando | Ergebnis |
| --- | --- | --- |
| Task 1 automated | `test $(grep -c "person_directory" F01…md) -ge 3 && test $(grep -c … _de.md) -ge 3` | `OK` (17 / 17) |
| Task 2 automated | `grep -lci PersonDirectory 02-service-tiers.md _de.md service-graph-runtime.mmd` | alle drei gelistet |
| Regressionstest verankert | `grep -c 'user_mit_nur_soft_geloeschter_sales_person_ist_kandidat' F01 EN/DE` | 1 / 1 |
| HTTP-Gate-Tests verankert | `grep -c 'hr_ohne_admin_bekommt_403'` / `'admin_bekommt_200'` EN/DE | 1 / 1 je Test |
| Ueberschriften-Paritaet F01 | `diff <(grep -c "^#" EN) <(grep -c "^#" DE)` | keine Ausgabe (33 / 33) |
| Ueberschriften-Paritaet Tiers | `diff <(grep -c "^#" EN) <(grep -c "^#" DE)` | keine Ausgabe (7 / 7) |
| Knoten im Business-Teilgraph | `awk '/subgraph Business/,/^    end/' … \| grep -c 'PersonDirectory'` | 1 |
| Knoten NICHT im Basic-Teilgraph | `awk '/subgraph Basic/,/^    end/' … \| grep -c 'PersonDirectory'` | 0 |
| Dependency-Abgleich | `sed -n '/gen_service_impl! {/,/^}/p' service_impl/src/person_directory.rs` | drei Deps, deckungsgleich mit der Doku |
| Symbol-Verifikation | 23 Symbole, `grep -rl` ueber alle Crates | jedes >= 1 Treffer |
| Keine Quellcode-Aenderung | `git show --name-only --format= bbb66a34 54f220a0` | ausschliesslich `docs/`-Dateien |
| Keine Loeschungen | `git diff --diff-filter=D --name-only` ueber beide Commits | leer |

**Kein Build- und kein Testlauf.** Dieser Plan aendert ausschliesslich Markdown
und Mermaid; es gibt keine `.rs`-, `.sql`- oder `.toml`-Aenderung, die `cargo
build`, `cargo test` oder `cargo clippy` beeinflussen koennte. Die
CLAUDE.md-Regel „immer cargo build/test/run ausfuehren" bezieht sich auf
Feature-Implementierungen; hier waere der Lauf ein Nachweis ueber unveraenderten
Code. Die Verifikation dieses Plans ist stattdessen die Symbol-Tabelle oben:
sie prueft genau das, was bei einem Doku-Plan schiefgehen kann — eine
Behauptung ueber Code, den es so nicht gibt.

## Threat Mitigations

| Threat ID | Disposition | Umsetzung |
| --- | --- | --- |
| T-62-15 | mitigate | Die Aussage zum Kandidaten-Lesepfad ist in beiden Fassungen korrigiert (schon durch 62-01) **und** um die verallgemeinerte ON-vs-WHERE-Regel plus den namentlich genannten Regressionstest `user_mit_nur_soft_geloeschter_sales_person_ist_kandidat` ergaenzt. Ein Reviewer, der den Fix zurueckdrehen wollte, faende jetzt drei sich widersprechende Stellen: die Doku, den Zeilenkommentar in `dao_impl_sqlite/src/person.rs:177` und den roten Test. |
| T-62-16 | mitigate | Die Gate-Verschaerfung steht in F01 §4.6 unter eigener Ueberschrift, mit Entscheidungsdatum (2026-09-16), Quelle (`.planning/REQUIREMENTS.md` §„Bewusste Änderung gegenüber heute"), der genauen Konsequenz (HR ohne Admin verliert die Sales-Person-Verwaltung), der Abgrenzung zu PADM-F-02 und beiden HTTP-Tests. Ein Support-Fall „HR sieht die Seite nicht mehr" ist damit ohne Code-Archaeologie rekonstruierbar. |
| T-62-17 | accept | `docs/` ist repo-intern; alle genannten Symbol- und Pfadnamen stehen ohnehin im Quellcode. Keine Credentials, keine Beispieldaten mit echten Namen. |
| T-62-SC | accept | Kein Vektor: beide Commits enthalten ausschliesslich `.md`- und `.mmd`-Dateien, keine `Cargo.toml`, kein `package.json`, kein Installationskommando. |

## Known Stubs

Keine. Dieser Plan liefert keinen Code und keine UI. Ein `[Zu pruefen]`-Marker
wurde bewusst gesetzt (Trait-Service-Graph, siehe Tabelle „bewusst NICHT
angefasst") — das ist die in F01 etablierte Konvention fuer eine offene
Recherchefrage, kein Stub.

## Threat Flags

Keine. Der Plan legt keine neue Oberflaeche an: kein Endpoint, kein Auth-Pfad,
kein Dateizugriff, keine Schema-Aenderung.

## Notes for Future Phases

- **62-07 muss PADM-01 / PADM-06 / PADM-08 abhaken, nicht dieser Plan.** Alle
  sieben Plaene der Phase fuehren sie; ihre Aussagen sind nutzersichtbar und
  erst nach der UAT eingeloest. Praezedenz: 62-01 Deviation 4.
- **Phase 63 muss F01 an vier Stellen nachziehen**, sobald die gemeinsame
  Detail-Route steht: §4.6 „Abgrenzung" (der Verweis auf
  `Route::SalesPersonDetails` / `Route::UserDetails` wird dann falsch), §5.4
  (neue Endpoints), §6 Proxy-Liste und die Tier-Doku, falls das Verzeichnis
  einen Schreibpfad bekommt — ein Schreibpfad wuerde die Gate-Aussage
  („`list` ist admin-gegated") von einer Zeile zu einer Tabelle machen.
- **Die Klarstellung in §Rules Nr. 3 ist jetzt die kanonische Antwort** auf
  „drei Deps, kein Domain-Service, trotzdem Business-Logic". Wer den naechsten
  solchen Service baut, sollte dort landen statt die Diskussion neu zu
  fuehren. `CLAUDE.md` §Service-Tier-Konventionen traegt die Faustregel noch
  ohne diese Praezisierung — **[Zu pruefen]**, ob sie beim naechsten
  CLAUDE.md-Sweep nachgezogen wird; die Datei ist normativ, das Doku-Kapitel
  erlaeuternd.
- **Der deutsche Trait-Graph und `service-graph-traits.mmd` haengen
  hinterher.** Dieser Plan hat die Runtime-Sicht aktuell gehalten; die
  Trait-Sicht fuehrt weder `RebookingBatch` noch `Person` noch
  `PersonDirectory`. Kein Docs-Gate-Trigger verlangt sie heute, aber die
  Luecke waechst.

## Self-Check: PASSED

Dateien (alle fuenf existieren und sind geaendert):

- FOUND: `docs/features/F01-employee-management.md`
- FOUND: `docs/features/F01-employee-management_de.md`
- FOUND: `docs/architecture/02-service-tiers.md`
- FOUND: `docs/architecture/02-service-tiers_de.md`
- FOUND: `docs/architecture/diagrams/service-graph-runtime.mmd`

Commits:

- FOUND: `bbb66a34`
- FOUND: `54f220a0`
