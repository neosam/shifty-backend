---
phase: 61-personservice-aufl-sung-von-sales-person-user
plan: 03
subsystem: database
tags: [rust, sqlx, sqlite, dao, authorization, self-gate, person, sql-join, security]

# Dependency graph
requires:
  - phase: 60-person-datenmodell-backfill
    provides: "person-Tabelle, nullable person_id auf sales_person + user, zwei vollstaendige UNIQUE-Indizes, idempotenter Backfill"
  - phase: 61-01
    provides: "PersonDao/PersonService (Basic-Tier) — die person-Identitaet als Applikations-Baustein"
  - phase: 61-02
    provides: "person_id-Vergabe bei Neuanlage in beiden Pfaden — Voraussetzung dafuer, dass der person_id-Join auch neue Objekte findet"
provides:
  - "find_by_user + find_sales_person_by_user_id joinen ueber u.person_id = sp.person_id (D-61-08)"
  - "get_assigned_user + get_all_user_assignments joinen ueber u.person_id = sp.person_id (D-61-08)"
  - "service_impl::test::person_self_gate — 12 Integrationstests inkl. SC1-Matrix und D-61-09-Byte-Identity"
  - "markierte LEGACY-VERGLEICH-Region als mechanischer Abbaupunkt fuer Plan 61-05"
affects: [61-04, 61-05, 62-personenverwaltung-liste, 63-personen-detail]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Autorisierungsentscheidung ausschliesslich im SQL-JOIN statt im Rust-Options-Vergleich (NULL = NULL matcht nie, None == None waere fail-open)"
    - "Schema-Swap unter eingefrorener Spaltenliste: query_as!-Zielstruct bleibt unveraendert, weil nur die Join-Struktur wechselt"
    - "Spalten-Aliase (AS user_id, AS sales_person_id) als Kompatibilitaets-Schicht, damit der Rust-Mapping-Block unveraendert bleibt"
    - "Byte-Identity-Matrix Alt/Neu gegen EINE Fixture-DB, mit Markerzeilen als Abbau-Vertrag fuer die spaetere DROP-Welle"
    - "Alt-Query im Test bewusst als rohes sqlx::query statt query!-Makro, damit sie nicht in den .sqlx-Cache wandert und mit ihrem Block verschwinden kann"

key-files:
  created:
    - service_impl/src/test/person_self_gate.rs
  modified:
    - dao_impl_sqlite/src/sales_person.rs
    - service_impl/src/test/mod.rs
    - docs/features/F01-employee-management.md
    - docs/features/F01-employee-management_de.md
    - docs/features/F12-auth-session.md
    - docs/features/F12-auth-session_de.md
    - docs/architecture/03-data-model.md
    - docs/architecture/03-data-model_de.md

key-decisions:
  - "Die Sicherheitsentscheidung liegt vollstaendig im JOIN; kein Rust-seitiger Option<Uuid>-Vergleich (D-61-08), belegt durch null_person_id_does_not_match_null_person_id und get_assigned_user_bei_null_person_id_liefert_none"
  - "Identische Spaltenliste beim Rewrite — SalesPersonDb und TryFrom<&SalesPersonDb> bleiben unveraendert; query_as! prueft nur die selektierten Spalten, nicht die Join-Struktur"
  - "Spalten-Aliase u.name AS user_id und sp.id AS sales_person_id bewusst identisch zu vorher, damit result.user_id und Uuid::from_slice(&row.sales_person_id) unveraendert bleiben"
  - "Kein Soft-Delete-Filter nachgeruestet — verify_user_is_sales_person haengt an get_assigned_user, ein Filter wuerde einer soft-geloeschten Sales Person den Selbstzugriff entziehen (SC1-Verletzung)"
  - "Die Fixture traegt die person_id-Verknuepfung als Basis; die Alt-Haelfte legt nur der D-61-09-Test zusaetzlich an — dadurch liegt die gesamte Legacy-Abhaengigkeit in einem zusammenhaengenden, loeschbaren Block"
  - "Alt-Query im Test als rohes sqlx::query statt query!-Makro: sie darf nicht in den .sqlx-Offline-Cache und nicht zur Compile-Zeit ans Schema gebunden werden, sonst bricht sie den Build statt mit ihrem Block zu verschwinden"
  - "Zwoelfter Test soft_geloeschte_sales_person_behaelt_selbstzugriff ergaenzt (Plan nannte 11 Faelle, forderte aber mindestens 12 Tests) — siehe Deviation 4"

patterns-established:
  - "Self-Gate-Rewrite: Spaltenliste einfrieren, nur den JOIN tauschen, Zielstruct und Mapping-Block unangetastet lassen"
  - "LEGACY-VERGLEICH-Markerblock als expliziter Abbau-Vertrag zwischen zwei Wellen derselben Phase"
  - "fail-closed-Nachweis als eigener Test: beide Seiten NULL muessen None liefern, sonst ist die Pruefung fail-open"

requirements-completed: [PERS-05]

coverage:
  - id: D1
    description: "SC1 'self darf': ein eingeloggter Mitarbeiter findet ueber die neue Implementierung genau seine eigene Sales Person"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_self_gate.rs#self_may_access_own_sales_person"
        status: pass
    human_judgment: false
  - id: D2
    description: "SC1 'fremd darf nicht' (T-61-02): ein fremder Sales-Person-Datensatz bleibt verwehrt — geprueft wird die Zuordnung namentlich, nicht nur is_some()"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_self_gate.rs#foreign_sales_person_rejected"
        status: pass
    human_judgment: false
  - id: D3
    description: "SC1 'unverknuepft darf nicht': ein Benutzer mit eigener Person, aber ohne Sales Person, bekommt None"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_self_gate.rs#unlinked_user_rejected"
        status: pass
    human_judgment: false
  - id: D4
    description: "D-61-08 / T-61-01 Kernnachweis: beidseitiges person_id = NULL matcht nicht (fail-closed) — ein Rust-Options-Vergleich waere hier fail-open. Beide Zweige des Self-Gates abgedeckt."
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_self_gate.rs#null_person_id_does_not_match_null_person_id"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_self_gate.rs#get_assigned_user_bei_null_person_id_liefert_none"
        status: pass
      - kind: other
        ref: "grep -vE '^\\s*(//|--)' dao_impl_sqlite/src/sales_person.rs | grep -cE 'sp_person(_id)?\\s*==|person_id\\s*==\\s*.*person_id' == 0"
        status: pass
    human_judgment: false
  - id: D5
    description: "Eine ausschliesslich ueber person_id bestehende Verknuepfung loest auf (RED vor dem Rewrite, GREEN danach)"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_self_gate.rs#person_id_link_without_legacy_row_resolves"
        status: pass
    human_judgment: false
  - id: D6
    description: "D-61-09 / T-61-04: Alt- und Neu-Query liefern auf EINER dual-befuellten Fixture-DB dieselbe Accept/Reject-Entscheidung, dieselbe Sales-Person-ID und denselben Benutzernamen — fuer beide Zweige des Self-Gates"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_self_gate.rs#byte_identity_old_vs_new_query"
        status: pass
    human_judgment: false
  - id: D7
    description: "T-61-03: get_all_user_assignments enthaelt exakt die ueber person_id verbundenen Paare (Mengen-, nicht Groessenvergleich); get_assigned_user liefert den richtigen Benutzer bzw. None ohne Verknuepfung"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_self_gate.rs#get_all_user_assignments_enthaelt_genau_die_verknuepften_paare"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_self_gate.rs#get_assigned_user_liefert_verknuepften_benutzer"
        status: pass
      - kind: integration
        ref: "service_impl/src/test/person_self_gate.rs#get_assigned_user_ohne_verknuepfung_liefert_none"
        status: pass
    human_judgment: false
  - id: D8
    description: "Kein neuer Soft-Delete-Filter: eine soft-geloeschte Sales Person behaelt ihren Selbstzugriff (SC1 'genau dieselben eigenen Daten wie vorher')"
    requirement: PERS-05
    verification:
      - kind: integration
        ref: "service_impl/src/test/person_self_gate.rs#soft_geloeschte_sales_person_behaelt_selbstzugriff"
        status: pass
      - kind: other
        ref: "grep -c 'deleted IS NULL' dao_impl_sqlite/src/sales_person.rs == 2, unveraendert gegenueber dem Stand vor dem Plan"
        status: pass
    human_judgment: false
  - id: D9
    description: "Alle vier lesenden Verknuepfungs-Methoden joinen ueber person_id; keine Trait-Signatur beruehrt (SC3)"
    requirement: PERS-05
    verification:
      - kind: other
        ref: "grep -c 'JOIN user u ON u.person_id = sp.person_id' dao_impl_sqlite/src/sales_person.rs == 4"
        status: pass
      - kind: other
        ref: "jj diff --from 6971ef28 -- dao/src/sales_person.rs service/src/sales_person.rs service_impl/src/sales_person.rs — leer"
        status: pass
    human_judgment: false
  - id: D10
    description: "Abbau-Vertrag fuer Plan 61-05: LEGACY-VERGLEICH-Region markiert, ausserhalb kommt der Name der abgeloesten Tabelle in der neuen Datei nicht vor"
    requirement: PERS-05
    verification:
      - kind: other
        ref: "beide Markerzeilen vorhanden; awk-Filter ausserhalb der Region liefert 0 Treffer auf den alten Tabellennamen"
        status: pass
    human_judgment: false
  - id: D11
    description: "Projekt-Gates gruen; CURRENT_SNAPSHOT_SCHEMA_VERSION unveraendert 13"
    requirement: PERS-05
    verification:
      - kind: other
        ref: "cargo test --workspace (893 Tests in service_impl) && cargo clippy --workspace --all-targets -- -D warnings — beide Exit-Code 0"
        status: pass
      - kind: manual_procedural
        ref: "timeout 45 cargo run — Server bootet auf 127.0.0.1:3000, Migrationen laufen, Cron feuert"
        status: pass
      - kind: other
        ref: "grep -n 'pub const CURRENT_SNAPSHOT_SCHEMA_VERSION' service_impl/src/billing_period_report.rs == 13"
        status: pass
    human_judgment: false

# Metrics
duration: 15min
completed: 2026-09-17
status: complete
---

# Phase 61 Plan 03: Self-Gate auf `person_id`-Joins Summary

**Alle vier lesenden Verknuepfungs-Methoden entscheiden jetzt im SQL-`JOIN` ueber `u.person_id = sp.person_id` statt ueber die abgeloeste Verknuepfungstabelle — inklusive des fail-closed-Nachweises bei beidseitigem `NULL` und einer byte-identischen Vorher/Nachher-Permission-Matrix gegen eine gemeinsame Fixture-DB.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-09-17T08:16:56Z
- **Completed:** 2026-09-17T08:31:13Z
- **Tasks:** 2
- **Files modified:** 9 (1 neu, 8 geaendert) + 3 umgeschriebene `.sqlx`-Cache-Eintraege

## Accomplishments

- `find_by_user` und `find_sales_person_by_user_id` joinen `sales_person sp JOIN user u ON u.person_id = sp.person_id WHERE u.name = ?`. Die Spaltenliste ist **byte-identisch** zu vorher, deshalb bleiben `SalesPersonDb` und `impl TryFrom<&SalesPersonDb> for SalesPersonEntity` unveraendert.
- `get_assigned_user` (`SELECT u.name AS user_id … WHERE sp.id = ?`) und `get_all_user_assignments` (`SELECT sp.id AS sales_person_id, u.name AS user_id …`) nutzen dieselbe Join-Bedingung. Die Alias-Namen sind absichtlich identisch zu vorher, damit `result.user_id.into()` und `Uuid::from_slice(&row.sales_person_id)` unangetastet bleiben.
- **Die Sicherheitsentscheidung liegt vollstaendig im SQL** (D-61-08). Zwei Tests belegen das fail-closed-Verhalten: eine Sales Person und ein Benutzer mit je `person_id = NULL` finden sich **nicht**, obwohl ein Rust-seitiger `Option<Uuid>`-Vergleich hier `None == None` als `true` gewertet und fremde Lohn- und Buchungsdaten freigegeben haette.
- Die D-61-09-Matrix laeuft als **ein** Test gegen **eine** dual-befuellte Fixture-DB und deckt **beide** Zweige des Self-Gates ab (`find_sales_person_by_user_id` **und** `get_assigned_user`).
- Die Legacy-Haelfte liegt geschlossen in einer mit `LEGACY-VERGLEICH START` / `LEGACY-VERGLEICH ENDE` markierten Region. Ausserhalb kommt der Name der abgeloesten Tabelle in der neuen Datei nirgends vor — Plan 61-05 kann den Block mechanisch und vollstaendig entfernen.
- 12 Integrationstests gegen echtes In-Memory-SQLite, ohne DAO-Mock (ein Mock fuehrt kein SQL aus und koennte die eigentliche Sicherheitseigenschaft strukturell nicht pruefen).
- Keine Trait-Signatur und keine Service-Datei beruehrt (SC3): `jj diff` auf `dao/src/sales_person.rs`, `service/src/sales_person.rs` und `service_impl/src/sales_person.rs` ist leer.

## Task Commits

Jede Task wurde atomar committet:

1. **Task 1: Self-Gate-Lesepfade auf `person_id`-Join umschreiben und die SC1-Matrix belegen** — `d56883dd` (feat)
2. **Task 2: `get_assigned_user` und `get_all_user_assignments` auf `person_id`-Join umschreiben** — `e2fddacb` (feat)

**Plan metadata:** siehe Abschluss-Commit `docs(61-03)`.

## Files Created/Modified

**Neu:**

- `service_impl/src/test/person_self_gate.rs` — 12 Integrationstests (G-1..G-12), eigener Harness (`setup_pool`, `tx_of`, `commit`, `dao`), Muster aus `person_creation.rs`; am Dateiende die markierte LEGACY-VERGLEICH-Region.

**Geaendert:**

- `dao_impl_sqlite/src/sales_person.rs` — ausschliesslich die vier **lesenden** Verknuepfungs-Methoden. `assign_to_user` und `discard_assigned_user` (schreibend) bleiben unberuehrt; sie gehoeren 61-04/61-05.
- `service_impl/src/test/mod.rs` — Testmodul registriert.
- `docs/features/F01-employee-management{,_de}.md` — §2.1 um „Alle vier lesenden Verknuepfungs-Pfade loesen ueber `person_id` auf" und um die explizite „kein Soft-Delete-Filter"-Regel erweitert.
- `docs/features/F12-auth-session{,_de}.md` — §5: die veraltete Aussage, die Umschreibung der Selbst-Zugriffs-Pruefung sei noch offen, ist durch den Ist-Zustand ersetzt.
- `docs/architecture/03-data-model{,_de}.md` — §Abgrenzung: Nachtrag, dass die lesenden Pfade ab Phase 61 ueber `person_id` joinen und die Tabelle nur noch geschrieben wird.
- `.sqlx/` — 3 umgeschriebene Offline-Cache-Eintraege (`cargo sqlx prepare --workspace -- --all-targets`), netto keine neue oder entfallene Datei: `find_by_user` und `find_sales_person_by_user_id` teilen sich weiterhin **einen** Eintrag, weil ihre Query-Strings byte-identisch sind.

## Decisions Made

- **Spaltenliste einfrieren, nur den Join tauschen.** `query_as!` prueft die selektierten Spalten, nicht die Join-Struktur. Dadurch war der Rewrite ein reiner Query-String-Tausch ohne jede Aenderung am Zielstruct oder am Mapping — und damit ohne Risiko fuer die 14 anderen Aufrufer.
- **Alias-Namen als Kompatibilitaets-Schicht.** `u.name AS user_id` und `sp.id AS sales_person_id` heissen bewusst genau so wie die frueheren Spalten. sqlx hat beide Felder trotz `JOIN` als non-null inferiert; ein `AS "user_id!"`-Override war nicht noetig.
- **Die Fixture ist per Default rein `person_id`-basiert.** Nur der D-61-09-Test legt die Alt-Haelfte zusaetzlich an (`seed_legacy_links`). Dadurch liegt die **gesamte** Legacy-Abhaengigkeit — Seed-Helfer, Alt-Queries und Vergleichstest — in einem zusammenhaengenden Block; 61-05 loescht ihn am Stueck, ohne dass Aufrufer draussen dangling werden.
- **Alt-Query als rohes `sqlx::query`, nicht als `query!`-Makro.** Ein Makro-Aufruf wuerde einen `.sqlx`-Cache-Eintrag erzeugen und zur Compile-Zeit ans Schema gebunden. Beides bricht den Build in dem Moment, in dem 61-05 die Tabelle entfernt — statt sauber mit dem Block zu verschwinden.
- **Kein Soft-Delete-Filter nachgeruestet.** Der Plan verlangt das ausdruecklich; `soft_geloeschte_sales_person_behaelt_selbstzugriff` zurrt die Entscheidung fest, damit sie ein spaeterer „Aufraeum-Pass" nicht versehentlich umdreht. `grep -c 'deleted IS NULL'` steht unveraendert bei `2`.
- **TDD-Zyklus ohne separaten RED-Commit.** Siehe Deviation 1.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - CLAUDE.md-Direktive] Kein separater RED-Commit im TDD-Zyklus**

- **Found during:** Task 1 und Task 2
- **Issue:** Der GSD-TDD-Ablauf sieht einen eigenen `test(...)`-Commit fuer die rote Phase vor. Ein solcher Commit haette die harten Projekt-Gates aus `CLAUDE.md` (`cargo build`/`cargo test`/`cargo clippy -D warnings` gruen bei jedem Commit) verletzt — dieselbe Lage wie in 61-01 und 61-02.
- **Fix:** RED/GREEN im Working Tree gefahren und die rote Phase **belegt**, danach je Task ein gruener `feat(...)`-Commit.
  - Task 1 RED: `self_may_access_own_sales_person`, `foreign_sales_person_rejected` und `person_id_link_without_legacy_row_resolves` FAILED — 4 passed / 3 failed.
  - Task 2 RED: `get_assigned_user_liefert_verknuepften_benutzer` (`left: None`, `right: Some("u-self")`) und `get_all_user_assignments_enthaelt_genau_die_verknuepften_paare` (`left: {}`) FAILED — 9 passed / 2 failed.
- **Files modified:** keine zusaetzlichen — betrifft nur die Commit-Granularitaet.
- **Committed in:** `d56883dd`, `e2fddacb`

**2. [Rule 3 - Blocking] `.sqlx`-Offline-Cache fuer die vier geaenderten Queries regeneriert**

- **Found during:** Task 1 (erneut Task 2)
- **Issue:** Das Repo baut ohne `.env` im sqlx-Offline-Modus. Geaenderte `query!`/`query_as!`-Strings haben einen neuen Hash und damit keinen Cache-Eintrag → `error: set DATABASE_URL to use query macros online`. Ohne Regeneration waeren Build und `nix build`/CI rot. Der Plan weist im `<verification>`-Block selbst darauf hin.
- **Fix:** Wegwerf-SQLite im Scratchpad (`sqlx migrate run`), dann `cargo sqlx prepare --workspace -- --all-targets` nach jedem Task.
- **Files modified:** `.sqlx/query-{d05fe583,e3a9df77,f9be6ad9}….json` (neu), `.sqlx/query-{b80bfc97,7b78fc49,906f6f9c}….json` (entfernt) — netto 0 Dateien, reine Rename-Operationen.
- **Verification:** `cargo build --workspace` und `cargo test --workspace` gruen ohne gesetztes `DATABASE_URL`.
- **Committed in:** `d56883dd`, `e2fddacb`

**3. [Rule 2 - CLAUDE.md-Direktive] Docs-Freshness-Gate bedient — und aktiv veraltete Aussagen korrigiert**

- **Found during:** Task 1 und Task 2
- **Issue:** Der Plan erwaehnt keine Doku-Arbeit. `docs/features/F12-auth-session{,_de}.md` behaupteten woertlich, die Umschreibung der Selbst-Zugriffs-Pruefung auf `person_id`-Joins sei „ebenfalls Phase-61-Scope" und die alte Tabelle sei „weiterhin die laufende Wahrheit der 1:1-Kopplung". Genau das hat dieser Plan umgesetzt — die Saetze waren ab Task 1 sachlich falsch. `docs/architecture/03-data-model{,_de}.md` trug dieselbe Aussage.
- **Fix:** F01 §2.1 um die Ist-Beschreibung aller vier Lesepfade plus die „kein Soft-Delete-Filter"-Regel erweitert; F12 §5 und 03-data-model §Abgrenzung auf den Ist-Zustand gezogen (gelesen wird ueber `person_id`, geschrieben wird die Tabelle bis zur Aufloesungs-Migration weiter). Jede Aenderung in `foo.md` **und** `foo_de.md`, jeweils im selben Commit wie der ausloesende Code-Diff.
- **Files modified:** `docs/features/F01-employee-management{,_de}.md`, `docs/features/F12-auth-session{,_de}.md`, `docs/architecture/03-data-model{,_de}.md`
- **Committed in:** `d56883dd`, `e2fddacb`

**4. [Rule 2 - Missing critical coverage] Zwoelfter Test ergaenzt: Soft-Delete darf den Selbstzugriff nicht entziehen**

- **Found during:** Task 2 (Acceptance-Pruefung)
- **Issue:** Die `<behavior>`-Bloecke nennen zusammen 11 Testfaelle (7 in Task 1, 4 neue in Task 2 — die Byte-Identity-Erweiterung gehoert laut Plan ausdruecklich **in** den bestehenden Test). Das Acceptance-Kriterium verlangt aber „mindestens 12 Tests". Statt die Zaehlung durch Aufspalten des D-61-09-Tests zu erzwingen (was dem Plan widersprochen haette), wurde ein inhaltlich fehlender Fall ergaenzt.
- **Fix:** `soft_geloeschte_sales_person_behaelt_selbstzugriff` zurrt die vom Plan explizit getroffene Entscheidung „kein Soft-Delete-Filter nachruesten" als Test fest: eine soft-geloeschte Sales Person bleibt ueber `find_sales_person_by_user_id` auffindbar und `get_assigned_user` liefert weiterhin ihren Login. Ohne diesen Test waere die Entscheidung nur ein Kommentar gewesen — und der naechste Aufraeum-Pass haette den Filter „repariert" und damit SC1 verletzt. Die Fixture wurde um `s_geloescht` / `U_GELOESCHT` erweitert; `get_all_user_assignments_enthaelt_genau_die_verknuepften_paare` erwartet das Paar jetzt ausdruecklich mit.
- **Files modified:** `service_impl/src/test/person_self_gate.rs`, `docs/features/F01-employee-management{,_de}.md`
- **Verification:** `cargo test --package service_impl --lib person_self_gate` — 12 passed.
- **Committed in:** `e2fddacb`

---

**Total deviations:** 4 auto-fixed (2 CLAUDE.md-Direktiven, 1 blocking, 1 fehlende kritische Abdeckung)
**Impact on plan:** Kein Scope Creep, kein Rule-4-Fall. Drei Abweichungen sind harte Projekt-Gates, die vierte schliesst eine Luecke zwischen `<behavior>` (11 Faelle) und `<acceptance_criteria>` (mindestens 12 Tests) mit einem inhaltlich begruendeten Fall statt mit einer Aufspaltung.

## Issues Encountered

- **`cargo` nicht auf dem PATH.** Alle Gates liefen ueber `nix develop --command bash -c '…'` (liefert `cargo`, `clippy`, `sqlx-cli`, `sqlite3`) — wie in 61-01 und 61-02.
- **`cargo fmt` existiert im Devshell nicht** (`error: no such command: 'fmt'`). Es gibt entsprechend kein Formatier-Gate; `cargo clippy --workspace --all-targets -- -D warnings` ist gruen.
- **Der `cargo run`-Smoke ist fuer diesen Plan inhaltsleer.** Der Server bootet sauber (`Running server at 127.0.0.1:3000`, Migrationen, Cron), aber die Dev-DB `localdb.sqlite3` enthaelt `0` Sales Persons, `2` Benutzer und `0` Zeilen im abgeloesten Datenmodell — ein Live-Vergleich Alt/Neu hat dort schlicht keine Datenbasis. Die Beweislast traegt vollstaendig `person_self_gate` gegen In-Memory-SQLite. Eine Sicherungskopie der Dev-DB liegt im Scratchpad; die Datei ist gitignored und wurde strukturell nicht veraendert.
- **Der Connect-String hat weiterhin kein `mode=rwc`** (`shifty_bin/src/main.rs:1690`). Bestandsverhalten aus 61-02, kein Regress dieses Plans.

## User Setup Required

None — keine externe Service-Konfiguration noetig. Die Phase installiert kein Package (Threat T-61-SC, `accept`).

## Next Phase Readiness

- **Plan 61-04** (Merge/Split) kann starten: alle vier Lesepfade sind auf `person_id` umgestellt, die schreibenden Methoden (`assign_to_user`, `discard_assigned_user`) sind bewusst unberuehrt geblieben und gehoeren dorthin.
- **Plan 61-05** (DROP-Migration) findet in `service_impl/src/test/person_self_gate.rs` genau einen zusammenhaengenden Block zwischen `// ── LEGACY-VERGLEICH START (Plan 61-05 entfernt diesen Block) ──` und `// ── LEGACY-VERGLEICH ENDE ──`. Loeschen dieses Blocks entfernt `seed_legacy_links`, `alt_find_sales_person_id_by_user_id`, `alt_get_assigned_user` und `byte_identity_old_vs_new_query` vollstaendig; die uebrigen 11 Tests laufen unveraendert weiter. **Wellen-Zwang eingehalten:** dieser Plan lief vor 61-05, solange die Alt-Haelfte noch herstellbar war.
- **Offen (unveraendert, kein Blocker):** Die abgeloeste Tabelle besteht weiter und wird noch **geschrieben**; ihre Aufloesung ist 61-05. `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **13**.

## Self-Check: PASSED

- `service_impl/src/test/person_self_gate.rs` existiert.
- Commits `d56883dd` und `e2fddacb` sind im Repo auffindbar (`jj log`).
- `cargo test --workspace` gruen (893 Tests in `service_impl`), `cargo clippy --workspace --all-targets -- -D warnings` gruen.
- `grep -c 'JOIN user u ON u.person_id = sp.person_id' dao_impl_sqlite/src/sales_person.rs` == `4`.
- SC3-Diff auf `dao/src/sales_person.rs`, `service/src/sales_person.rs`, `service_impl/src/sales_person.rs` leer.

---
*Phase: 61-personservice-aufl-sung-von-sales-person-user*
*Completed: 2026-09-17*
