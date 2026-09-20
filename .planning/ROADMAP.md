# Roadmap: Shifty Backend

## Milestones

- 🚧 **v2.7** — Phasen 60–64 (planning 2026-09-16) — Personenverwaltung & Planungs-Soll (this file)
- ✅ **v2.6** — Phasen 54–57 (shipped 2026-07-31, deployed) — Freiwillige-Stunden-Ausgleich für gedeckelte Mitarbeiter ([`milestones/v2.6-ROADMAP.md`](milestones/v2.6-ROADMAP.md))
- ✅ **v2.5** — Phasen 52–53 (shipped 2026-07-06) — Weekly-Overview Performance & Freiwilligen-Abwesenheiten ([`milestones/v2.5-ROADMAP.md`](milestones/v2.5-ROADMAP.md))
- ✅ **v2.4** — Phase 51 (shipped 2026-07-05) — Kurzer-Tag-Slot-Kürzung ([`milestones/v2.4-ROADMAP.md`](milestones/v2.4-ROADMAP.md))
- ✅ **v2.3** — Phasen 49–50 (shipped 2026-07-04) — PDF-Export: Browser-Look & Download-Button ([`milestones/v2.3-ROADMAP.md`](milestones/v2.3-ROADMAP.md))
- ✅ **v2.2** — Phasen 43–48 (shipped 2026-07-03) — Aufräumen, WebDAV-Export & Wochentag-Muster ([`milestones/v2.2-ROADMAP.md`](milestones/v2.2-ROADMAP.md))
- ✅ **v2.1** — Phasen 39–42 (shipped 2026-07-02) — Schichtplan- & Reporting-Erweiterungen ([`milestones/v2.1-ROADMAP.md`](milestones/v2.1-ROADMAP.md))
- ✅ **v1.11** — Phasen 36–38 (shipped 2026-07-01) — Stabilisierung & UX-Politur ([`milestones/v1.11-ROADMAP.md`](milestones/v1.11-ROADMAP.md))
- ✅ **v1.10** — Phasen 33–35 (shipped 2026-06-30) — Feiertage — UI-Pflege & Schichtplan-Soll-Konsistenz ([`milestones/v1.10-ROADMAP.md`](milestones/v1.10-ROADMAP.md))
- ✅ **v1.9** — Phasen 29–32 (shipped 2026-06-29) — Schichtplan-/Urlaubs-UX-Korrekturen & Admin-Impersonation ([`milestones/v1.9-ROADMAP.md`](milestones/v1.9-ROADMAP.md))
- ✅ **v1.8** — Phasen 27–28 (shipped 2026-06-29) — Freiwilligen-Auswahl & Urlaubsanspruch-Korrektur (HR-UX) ([`milestones/v1.8-ROADMAP.md`](milestones/v1.8-ROADMAP.md))
- ✅ **v1.7** — Phasen 25–26 (shipped 2026-06-29) — Automatische Feiertage & Freiwilligen-Abwesenheit ([`milestones/v1.7-ROADMAP.md`](milestones/v1.7-ROADMAP.md))
- ✅ **v1.6** — Phase 24 (shipped 2026-06-27) — Paid-Capacity-Durchsetzung & Konfiguration ([`milestones/v1.6-ROADMAP.md`](milestones/v1.6-ROADMAP.md))
- ✅ **v1.5** — Phasen 18–23 (shipped 2026-06-27) — Mitarbeiter-Sicht & Urlaubsverwaltung — Korrekturen & Auswertungen ([`milestones/v1.5-ROADMAP.md`](milestones/v1.5-ROADMAP.md))
- ✅ **v1.4** — Phasen 14–17 (shipped 2026-06-25) — Committed Voluntary Capacity ([`milestones/v1.4-ROADMAP.md`](milestones/v1.4-ROADMAP.md))
- ✅ **v1.3** — Phasen 8–13 (closed 2026-06-22) — Frontend Abwesenheiten + UI-Closure-Restanten ([`milestones/v1.3-ROADMAP.md`](milestones/v1.3-ROADMAP.md))
- ✅ **v1.2** — Phasen 6–7 (shipped 2026-05-07) — Frontend rest-types Konsolidierung ([`milestones/v1.2-ROADMAP.md`](milestones/v1.2-ROADMAP.md))
- ✅ **v1.1** — Phase 5 (shipped 2026-05-04) — Slot Capacity & Constraints ([`milestones/v1.1-ROADMAP.md`](milestones/v1.1-ROADMAP.md))
- ✅ **v1.0** — Phasen 1–4 (shipped 2026-05-03) — Range-Based Absence Management ([`milestones/v1.0-ROADMAP.md`](milestones/v1.0-ROADMAP.md))

Vollständiger historischer Index: [`MILESTONES.md`](MILESTONES.md).

**Aktiver Milestone: v2.7 — Personenverwaltung & Planungs-Soll** (Phasen 60–64, Roadmap-Sektion direkt unter dieser Liste; 58 und 59 bleiben reserviert).

**Kern-Scope dieses Milestones** (User-Festlegung 2026-09-15):
Benutzerverwaltung und Sales-Person-Verwaltung in der UI zusammenlegen — heute
zerfällt dieselbe Person in drei Detailseiten (`/user_details`,
`/sales_person_details`, `/employees/:id`), was Nutzer verwirrt. Dafür bekommt
der Eintrag ein **eigenes Fachobjekt `Person`** als Aggregat über User und
SalesPerson (Entscheidung 2026-09-15) — also eine Data-Model-first-Phase mit
Migration und Backfill, nicht nur UI. Ist-Zustand, Blast Radius und offene
Modellierungsfragen:
[`seeds/personenverwaltung-zusammenlegen.md`](seeds/personenverwaltung-zusammenlegen.md).

**Zweiter Scope-Punkt** (User-Festlegung 2026-09-15): **Sollstunden bei
dynamischen Verträgen** fehlen in der Übersicht unterhalb des Schichtplans.
`weight_for_week` setzt `expected_hours` bei `is_dynamic` hart auf 0, damit das
Soll die Balance nicht beeinflusst — für die Planung soll es aber angezeigt und
vom Backend geliefert werden. Lösungsrichtung und die Falle beim naheliegenden
Fix: [`todos/pending/2026-09-15-sollstunden-fehlen-bei-dynamischen-vertraegen-unter-schichtplan.md`](todos/pending/2026-09-15-sollstunden-fehlen-bei-dynamischen-vertraegen-unter-schichtplan.md).

## Milestone v2.7 — Personenverwaltung & Planungs-Soll (Active)

**Goal:** Die Verwaltung einer Person passiert an **einem** Ort — Benutzer- und
Sales-Person-Verwaltung werden zu einer Personen-Sicht zusammengeführt, getragen
von einem neuen Fachobjekt `Person`. Zusätzlich zeigt die Übersicht unterhalb des
Schichtplans auch bei dynamischen Verträgen ein Stundenziel für die Planung.

**Requirements-Quelle:** `.planning/REQUIREMENTS.md` (18 REQ-IDs in 3 Kategorien:
PERS-01..06, PADM-01..08, PSOLL-01..04).

**Research-Quelle:** `.planning/research/SUMMARY.md` (Konvergenz aus vier
Researcher-Outputs), `research/ARCHITECTURE.md` (Build-Order, Tier-Klassifikation,
Blast Radius mit file:line), `research/PITFALLS.md` (10 Pitfalls; die dort
verwendeten Phasennummern 60–63 sind eine Hypothese, keine Entscheidung).

**Seed (gesperrte Entscheidungen, nicht neu aufmachen):**
[`seeds/personenverwaltung-zusammenlegen.md`](seeds/personenverwaltung-zusammenlegen.md).

### Phasennummerierung

Der Milestone startet bei **Phase 60**. Die Nummern **58 und 59 bleiben
reserviert** für die vorbereiteten, bewusst nicht promoteten Phasen (siehe
„Vorbereitete Phasen" weiter unten) — sie werden in v2.7 nicht vergeben.

### Phase-Dekomposition-Entscheidung

Die Research schlägt vier Phasen vor (SUMMARY §„Implications for Roadmap",
provisorisch 60–63). Diese Roadmap schneidet **fünf**, mit genau einer Abweichung:
der UI-Block wird in **Liste (62)** und **Detail + Ausprägungs-Aktionen (63)**
geteilt.

1. **Data-Model-first ist gesetzt** (PROJECT.md-Constraint, von drei Researchern
   unabhängig bestätigt): `person`-Tabelle, nullable `person_id`-FKs, Backfill und
   die wiederhergestellten UNIQUE-Constraints landen in Phase 60, **bevor**
   irgendetwas durch sie liest oder schreibt. `sales_person_user` bleibt in Phase
   60 bewusst stehen und wird erst in Phase 61 aufgelöst — die Tabelle ist bis
   dahin die laufende Wahrheit, gegen die der Backfill geprüft werden kann.

2. **Phase 61 bleibt von Phase 60 getrennt**, obwohl sie nur ein Requirement trägt
   (PERS-05). Sie enthält die einzige sicherheitsrelevante Umschreibung des
   Milestones (`find_sales_person_by_user_id`, siehe Constraints unten) und braucht
   einen eigenen, engen Plan mit Vorher/Nachher-Permission-Matrix, statt im
   Migrations-Rauschen mitzulaufen. Präzedenz gegen das Zusammenlegen: v2.6
   Phase 57 — ein Requirement, das planintern „mitlief", kostete eine ganze
   Nachzieh-Phase.

3. **Der UI-Block wird geteilt.** Zusammen wären Aggregat-Service + zwei DTO-
   Familien + mehrere REST-Endpoints + Listen-Umbau + Detail-Komponente +
   Anlege-/Verknüpfungs-Masken + i18n de/en/cs eine Phase von 10+ Plänen; die
   größten bisherigen Phasen dieses Projekts lagen bei 9 (Phase 54). Der Schnitt
   verläuft entlang Lesen/Schreiben: Phase 62 liefert die Liste (ein
   Aggregat-Read, sofort UAT-bar — „keine Tabs mehr"), Phase 63 die Detailansicht
   und alle Ausprägungs-Mutationen.

4. **PSOLL bekommt eine eigene Phase (64)** — Begründung steht in der
   Phase-Detail-Sektion unter „Depends on".

Jede Phase bleibt ein UAT-anchorbarer Slice: 60 und 61 gegen Datenbank- und
Permission-Beobachtungen, 62–64 gegen sichtbares Verhalten im Browser.

### Backend-/Frontend-Abdeckung pro Phase (GSD-Scope-Regel)

| Phase | Backend | Frontend | Begründung bei Frontend = nein |
|---|---|---|---|
| 60 | ja | **nein** | Reine Schema- + Backfill-Phase ohne API-Wirkung — kein TO geändert, kein Endpoint neu, keine sichtbare Verhaltensänderung. |
| 61 | ja | **nein** | Reine Backend-Phase — `PersonService`/`PersonDao` neu, Verknüpfungspfade auf `person_id` umgestellt, `sales_person_user` aufgelöst. Kein TO geändert, kein Endpoint neu; `POST /{id}/user` dokumentiert lediglich zusätzlich 409. |
| 62 | ja | ja | — |
| 63 | ja | ja | — |
| 64 | ja | ja | — |

### Gesperrte Entscheidungen (in keiner Phase neu aufmachen)

- **Kein Snapshot-Schema-Version-Bump — für beide Features.**
  `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **13**. `person` ist rein administrativ
  (kein neuer persistierter `value_type`, keine geänderte Computation, kein
  geändertes Input-Set, `billing_period_sales_person.sales_person_id` unverändert);
  das Planungs-Soll ist ein reines Anzeigefeld aus bereits lebender Logik. Die
  Begründung ist im Seed und in PROJECT.md schriftlich festgehalten. **Keine Phase
  bumpt „vorsichtshalber".**

- **`sales_person.id` und `user.name` werden nicht umgeschlüsselt** — zwölf Tabellen
  referenzieren die Sales-Person-UUID, darunter write-once-Abrechnungs-Snapshots;
  `user.name` trägt die Auth-Schicht. `Person` kommt obendrauf, nicht an deren Stelle.

- **Namen bleiben, wo sie sind** — `person` trägt zunächst nur Identität und
  Audit-Spalten. Echte Personenstammdaten sind PERS-F-01 (Future).

- **Abschnittsweises Rollen-Gating ist nicht v2.7-Scope.** Die Personenverwaltung
  ist als Ganzes admin-gated (PADM-08); das von der Research als „unscoped work"
  markierte Per-Sektion-Rollen-Gating steht in REQUIREMENTS.md bewusst als
  **PADM-F-02** hinten an. Die Sektions-Sichtbarkeit in Phase 63 richtet sich
  ausschließlich nach der **Ausprägung**, nicht nach der Rolle. Das reduziert den
  von der Research befürchteten Umfang des UI-Blocks erheblich.

- **`/employees/:id` (HR-Report) bleibt eine getrennte Sicht** — PADM-F-01.

### Harte Constraints aus der Research (mit file:line, verbatim in die Pläne)

Jeder dieser Punkte erzeugt bei Verletzung einen Defekt, den die Tests **nicht**
von selbst fangen würden:

1. **`find_sales_person_by_user_id` ist eine Autorisierungsprüfung, kein CRUD**
   (`service_impl/src/sales_person.rs:356-401`; DAO
   `dao_impl_sqlite/src/sales_person.rs:226-243`). Sie trägt das
   Self-vs-HR-Gate auf Sales-Person-Mutationen. Die Umschreibung auf `person_id`
   gehört in **dieselbe Phase wie die Auflösung der Verknüpfungstabelle (61)**,
   mit eigener Vorher/Nachher-Matrix. Vergleich im SQL-Join, nie zwei separat
   geladene `Option<Uuid>` in Rust — sonst fail-open über `NULL == NULL`.

2. **`user` hat keine Soft-Delete-Spalte, `sales_person` schon.** Ein Backfill mit
   dem gewohnten `WHERE deleted IS NULL` überspringt soft-gelöschte Sales Persons,
   die noch write-once-`billing_period_sales_person`-Zeilen besitzen (Phase 60,
   PERS-03).

3. **`booking_information.rs:644` und `:818` lesen bewusst das genullte
   `expected_hours` in `available_hours`** — der v2.5-Regression-Lock
   („`WorkingHoursPerSalesPerson` (Bezahlten-Vertrag) unangetastet"). Das
   Planungs-Soll darf dort **nicht** substituiert werden (Phase 64, PSOLL-04).

4. **`ShortEmployeeReport` wird an ZWEI unabhängigen Stellen konstruiert** —
   `assemble_weeks` (`service_impl/src/reporting.rs:996-1150`) und
   `get_reports_for_all_employees` (`:1318-1664`). Beide müssen in **derselben**
   Phase gefüllt werden, sonst ist das Feld in einer Sicht gesetzt und in der
   anderen leer. Strukturell dieselbe Falle wie der Phase-57-Bug (Phase 64,
   PSOLL-03).

5. **`PersonService` ist Basic-Tier** — nur DAOs + `PermissionService` +
   Clock/Uuid + `TransactionDao`, **kein** anderer Domain-Service (CLAUDE.md
   Tier-Regel). Sonst entsteht ein DI-Zyklus über `PermissionService`, das jeder
   Service braucht. Das zusammengeführte Lese-Aggregat für Liste und Detail
   gehört eine Stufe höher als Business-Logic-Service. DI-Konstruktion in
   `shifty_bin/src/main.rs` strikt basic-vor-business-logic.

6. **Eine neue REST-Routen-Familie braucht ihren `Dioxus.toml`-Dev-Proxy-Eintrag
   in derselben Phase** — dieses Repo hat genau diesen Bug dreimal geshipt
   (v1.6 `/toggle`, v1.8 `/vacation-entitlement-offset`, v1.10
   `/special-days`-Variante). Betrifft Phase 62 (erste `/person`-Route).

7. **Die 1:1-Garantie aus `sales_person_user` (`UNIQUE(sales_person_id)` +
   `UNIQUE(user_id)`, `migrations/sqlite/20240506114107_add_sales_person.sql:14-22`)
   verschwindet mit der Tabelle** und muss als `UNIQUE(person_id)` auf **beiden**
   neuen FK-Spalten wiederhergestellt werden — mit DB-Level-Ablehnungstest, nicht
   nur Applikationsprüfung (Phase 60, PERS-04).

### Docs-Freshness-Gate

Dieser Milestone berührt ungewöhnlich viele Trigger-Dateien gleichzeitig. Der
Trigger-Diff-Check läuft deshalb **nach jeder Phase**, nicht erst beim
Milestone-Close (PITFALLS Nr. 9). Beide Sprachen synchron, gleicher Commit wie der
Code-Diff (MEMORY `feedback_docs_always_current_no_followup`).

| Phase | Trigger-Datei | → Docs (EN **und** DE) |
|---|---|---|
| 60 | `migrations/sqlite/*.sql` | `docs/architecture/03-data-model.md` + `diagrams/db-schema-er.mmd` + passende `docs/features/F*.md` |
| 61 | `shifty_bin/src/main.rs`, `dao/**`, `service/**`, Self-Gate-Pfad | `docs/architecture/02-service-tiers.md` + `diagrams/service-graph-runtime.mmd`; `docs/architecture/04-auth.md` + `docs/features/F12-auth-session.md` |
| 62 / 63 | `service/**`, `dao/**` (Aggregat + Verknüpfung) | neue/erweiterte `docs/features/F*.md` zur Personenverwaltung |
| 64 | `service_impl/src/reporting.rs` | `docs/features/F07-reporting-balance.md` + `docs/domain/time-accounting.md` |

### Phases

- [x] **Phase 60: Person-Datenmodell + Backfill** — `person`-Tabelle, nullable
      `person_id`-FKs mit wiederhergestelltem UNIQUE, idempotenter Backfill inkl.
      soft-gelöschter Sales Persons (Backend-only)

- [x] **Phase 61: PersonService + Auflösung von `sales_person_user`** — Basic-Tier (completed 2026-09-17)
      `PersonDao`/`PersonService`, sechs DAO-Methoden auf `person_id`-Joins
      umgeschrieben inkl. der Selbst-Zugriffs-Prüfung, Verknüpfen/Lösen mit
      Konflikt-Ablehnung (Backend-only)

- [ ] **Phase 62: Personen-Verzeichnis — eine Liste statt Tabs** — Business-Logic-
      Lese-Aggregat + `PersonEntryTO` + ein Aggregat-Endpoint + admin-gated
      Listenansicht unter `/user_management/` mit Ausprägungs-Badge und Suche

- [ ] **Phase 63: Personen-Detailansicht + Ausprägungs-Aktionen** — eine Komponente
      für alle drei Ausprägungen, Anlegen mit zuschaltbaren Teilen, „Login/Person
      hinzufügen", Verknüpfen per Kandidaten-Auswahl, ausprägungs-spezifische
      Lösch-Bestätigung

- [ ] **Phase 64: Planungs-Soll für dynamische Verträge** — `planning_target_hours`
      an beiden `ShortEmployeeReport`-Konstruktionsstellen, Anzeige unterhalb des
      Schichtplans, Balance-/`available_hours`-Pfade nachweislich unverändert

### Phase Details

#### Phase 60: Person-Datenmodell + Backfill

**Goal:** Jede Sales Person und jeder Benutzer hat nach der Migration genau eine
Person-Identität; wer heute über `sales_person_user` verknüpft ist, teilt sich
danach dieselbe Person — und die Datenbank erzwingt diese 1:1-Garantie weiterhin
selbst, ohne die alte Verknüpfungstabelle.

**Depends on:** Nothing (erste Phase des Milestones, sitzt auf der geshipten
v2.6-Baseline).

**Requirements:** PERS-01, PERS-02, PERS-03, PERS-04, PERS-06

**Success Criteria** (what must be TRUE):

  1. Nach `sqlx migrate run` trägt jede Zeile in `sales_person` — **einschließlich
     der soft-gelöschten** — und jede Zeile in `user` ein nicht-leeres `person_id`;
     jedes Paar aus `sales_person_user` zeigt auf **dieselbe** `person.id`.
     Gegenprobe per Zählung: `COUNT(person) == COUNT(sales_person) + COUNT(user) −
     COUNT(sales_person_user)`.

  2. Ein zweiter Lauf des Backfills gegen dieselbe Fixture-DB ändert weder die
     Zeilenzahl in `person` noch eine einzige Zuordnung (Apply-twice-Test mit
     Vorher/Nachher-Vergleich, nicht nur „läuft ohne Fehler durch").

  3. Eine soft-gelöschte Sales Person, die eine `billing_period_sales_person`-Zeile
     besitzt, hat nach der Migration ein `person_id` — als Fixture-Test, weil genau
     dieser Fall der gewohnten `WHERE deleted IS NULL`-Reflexhandlung zum Opfer
     fällt.

  4. Ein direkter SQL-Schreibversuch, der zwei `sales_person`-Zeilen (oder zwei
     `user`-Zeilen) auf dieselbe `person.id` zeigen lässt, wird **von der Datenbank**
     abgelehnt — nicht von einer Applikationsprüfung.

  5. Bestehende Abrechnungs-Snapshots und Schichtplan-Daten lösen unverändert auf:
     `sales_person.id` und `user.name` sind bitgleich wie vorher (Diff-verifiziert),
     die komplette Backend-Testsuite bleibt grün, und
     `CURRENT_SNAPSHOT_SCHEMA_VERSION` steht unverändert auf **13**.

**Scope-Abgrenzung:** `sales_person_user` wird in dieser Phase **nicht** gelöscht.
Die Tabelle bleibt die laufende Wahrheit, gegen die der Backfill geprüft wird; die
Auflösung ist Phase 61.

**Frontend:** bewusst kein Anteil — reine Schema- und Backfill-Phase ohne
API-Wirkung (GSD-Scope-Regel, begründete Ausnahme).

**Research-Flag:** keine tiefere Recherche nötig — jede Technik hat eine direkte
Präzedenz in diesem Repo (nullable `ALTER TABLE ADD COLUMN`, SQL-only-Backfill,
`randomblob(16)`).

**Plans:** 3 plans

Plans:
**Wave 1**

- [x] 60-01-PLAN.md — Migration `20260916000000_create-person-backfill.sql`: `person`-Tabelle, beide nullable `person_id`-Spalten, beide vollständigen Unique-Indizes, idempotenter Backfill inkl. soft-gelöschter Sales Persons, Selbst-Assertion (TDD: RED→GREEN; Entscheidungs-Gate für D-60-01 one-way)

**Wave 2** *(blocked on Wave 1 completion)*

- [x] 60-02-PLAN.md — DB-Level-Ablehnungstests für die 1:1-Garantie plus Schema- und Erhaltungs-Locks (Index vollständig, `person`-Spaltensatz, `sales_person_user` unangetastet, Legacy-Schlüssel bitgleich, Snapshot-Version 13)
- [x] 60-03-PLAN.md — Docs-Freshness-Gate: `03-data-model.md` + `_de.md`, `diagrams/db-schema-er.mmd`, `F01-employee-management` und `F12-auth-session` je EN + DE

---

#### Phase 61: PersonService + Auflösung von `sales_person_user`

**Goal:** Alle Lese- und Schreibpfade, die heute über `sales_person_user` laufen,
gehen über `person_id` — sichtbar daran, dass ein Mitarbeiter unverändert genau
seine eigenen Daten sieht und bucht, ein fremder Datensatz ihm weiterhin verwehrt
bleibt, und die alte Verknüpfungstabelle danach verschwinden kann.

**Depends on:** Phase 60 (Spalten, Backfill und UNIQUE-Constraints müssen stehen,
bevor etwas durch sie liest oder schreibt).

**Requirements:** PERS-05

**Success Criteria** (what must be TRUE):

  1. Ein eingeloggter Mitarbeiter sieht und bucht nach der Umstellung genau
     dieselben eigenen Daten wie vorher. Belegt durch eine Vorher/Nachher-Matrix mit
     drei Fällen: „self darf", „fremder Sales-Person-Datensatz darf nicht",
     „Benutzer ohne Verknüpfung darf nicht" — jeweils vor und nach dem Schema-Swap.

  2. `sales_person_user` existiert nicht mehr; ein `grep` über `dao_impl_sqlite/`,
     `service_impl/` und `dao/` findet den Tabellennamen nur noch in den
     historischen Migrationen.

  3. Die öffentlichen Signaturen von `SalesPersonService` (`set_user`,
     `get_assigned_user`, `get_all_user_assignments`, `get_sales_person_for_user`,
     `get_sales_person_current_user`, `verify_user_is_sales_person`) sind
     unverändert und die rund 50 Aufrufstellen in `service_impl/` stehen unberührt
     im Diff — die Auflösung bleibt eine Zwei-Schichten-Änderung.

  4. Ein Verknüpfungsversuch, bei dem Sales Person und Benutzer bereits auf zwei
     **verschiedene** Personen zeigen, wird mit einem unterscheidbaren
     Konflikt-Fehler (HTTP 409) abgelehnt, statt still umzuhängen; Verknüpfen zweier
     noch unverknüpfter Objekte legt genau eine gemeinsame Person an.

  5. `PersonService` ist Basic-Tier (nur `PersonDao` + Permission + Clock/Uuid +
     Transaction, kein Domain-Service als Dependency) und wird in
     `shifty_bin/src/main.rs` im Basic-Block konstruiert; `cargo clippy --workspace
     -- -D warnings` ist grün und Service-Tier-Doku plus Auth-Doku sind im selben
     Commit in **beiden** Sprachen nachgezogen.

**Frontend:** bewusst kein Anteil — der Service-/REST-Vertrag bleibt per
Konstruktion eingefroren, das Frontend verhält sich unverändert (GSD-Scope-Regel,
begründete Ausnahme). Die „unverknüpfte Kandidaten"-Lesepfade, die Phase 63 für den
Picker braucht, entstehen hier als Backend-Vorarbeit ohne UI.

**Research-Flag:** `assign_to_user` / `discard_assigned_user` sind echte neue
Geschäftslogik (Person auflösen-oder-anlegen, mehrdeutigen Merge ablehnen) ohne
1:1-Präzedenz im Repo — eigene Test-Matrix beim Planen entwerfen, nicht „Query
portieren".

**Plans:** 10/10 plans complete

Plans:
**Wave 1**

- [x] 61-01-PLAN.md — Basic-Tier `PersonDao`/`PersonDaoImpl`/`PersonService`/`PersonServiceImpl`, die beiden „unverknüpfte Kandidaten"-Anti-Join-Lesepfade (D-61-02), DI-Konstruktion im Basic-Block plus `RestStateDef`-Accessor (SC5)

**Wave 2** *(blocked on Wave 1)*

- [x] 61-02-PLAN.md — `person_id` bei jeder Neuanlage: `SalesPersonDaoImpl::create` auf derselben `tx`, `PermissionDaoImpl::create_user` mit interner Transaktion (D-61-01, Atomaritäts-Nachweis)

**Wave 3** *(blocked on Wave 2)*

- [x] 61-03-PLAN.md — Self-Gate-Rewrite der vier Lesepfade auf `person_id`-Joins (D-61-08) plus die byte-identische Vorher/Nachher-Permission-Matrix gegen eine gemeinsame Fixture-DB (D-61-09, SC1)

**Wave 4** *(blocked on Wave 3)*

- [x] 61-04-PLAN.md — Verknüpfen als Merge mit Konflikt-Ablehnung (D-61-03/D-61-04), Lösen als Split mit frischer Person (D-61-05), 409-Übersetzung in `set_user` und OpenAPI-Ergänzung (SC4)

**Wave 5** *(blocked on Wave 4)*

- [x] 61-05-PLAN.md — DROP-Migration `20260917000000_drop-sales-person-user.sql` mit Nachzieh-Backfill und Selbst-Assertion (D-61-06/D-61-07), Verlagerung der Migrations-Regressionstests nach `shifty_bin`, Abbau der Alt-Query-Hälfte, `cargo sqlx prepare` (SC2)

**Wave 6** *(blocked on Wave 5)*

- [x] 61-06-PLAN.md — Docs-Freshness-Gate: `02-service-tiers` + `diagrams/service-graph-runtime.mmd`, `04-auth`, `F12-auth-session`, `03-data-model` + `diagrams/db-schema-er.mmd`, `F01-employee-management`, `arc42/05-building-block-view` — je EN + DE

**Wave 7** *(Gap-Closure, blocked on Wave 6)*

- [x] 61-07-PLAN.md — CR-01 + WR-05: Nachzieh-Backfill der DROP-Migration konvergiert statt zu divergieren (Entkopplung + Merge + Träger-Aufräumung nach D-61-03/D-61-05), Selbst-Assertion bekommt die beiden globalen Deckungsterme aus Phase 60, der Test, der den Abbruch als Sollverhalten zementiert, wird umgedreht

**Wave 8** *(blocked on Wave 7)*

- [x] 61-08-PLAN.md — CR-02: `BasicDaoImpl::clear_all` löscht nur noch trägerlose `person`-Zeilen, zwei Regressionstests für die Invariante, `.sqlx`-Cache und Datenmodell-Doku im selben Commit

**Wave 9** *(blocked on Wave 8)*

- [x] 61-09-PLAN.md — WR-04: `set_user` übersetzt den Konflikt aus `discard_assigned_user` genauso wie den aus `assign_to_user` (409 statt 500 auf `DELETE /sales-person/{id}/user`), plus OpenAPI-Ergänzung

**Wave 10** *(blocked on Wave 9)*

- [x] 61-10-PLAN.md — WR-06: `AUTHENTICATION.md` beschreibt nicht länger die aufgelöste 1:1-Verknüpfungstabelle als aktuelles Schema; `user`/`person` auf dem Stand der Migrationskette, Verweis auf die kanonische zweisprachige Doku

**Aufgeschoben (in `61-07-PLAN.md` §„Deferral Record" dokumentiert):** WR-02 →
Phase 62; WR-01, WR-03, WR-07 und IN-01 bis IN-07 → Backlog.

---

#### Phase 62: Personen-Verzeichnis — eine Liste statt Tabs

**Goal:** Ein Admin öffnet `/user_management/` und sieht **eine** durchsuchbare
Liste aller Einträge mit erkennbarer Ausprägung — verknüpfte, reine Benutzer und
reine Verkäufer nebeneinander, jeder mit einem fertig vom Backend gelieferten Namen,
kein Tab-Wechsel mehr.

**Depends on:** Phase 61 (Person-Verknüpfung als Lesequelle, `PersonService` als
Basic-Tier-Baustein für das Aggregat).

**Requirements:** PADM-01, PADM-06, PADM-08

**Success Criteria** (what must be TRUE):

  1. `/user_management/` zeigt eine einzige Liste ohne Tab-Leiste; verknüpfte
     Einträge, reine Benutzerkonten und reine Verkäufer stehen nebeneinander, jeder
     mit sichtbarer Ausprägung (Badge oder Spalte).

  2. Die Suche findet einen Eintrag über den Namen, den er **hat** — Verkäufername,
     Username oder beides — ohne dass der Nutzer wissen muss, welcher Objekttyp
     dahintersteht.

  3. Jeder Eintrag ist beschriftet, auch ein reines Benutzerkonto ohne
     Verkäufernamen; der Anzeigename kommt fertig aus dem Backend —
     grep-verifiziert keine Fallback-Regel im Frontend (Fat Backend, Thin Client;
     Präzedenz v2.4 `clip_to`-Lock).

  4. Ein Nutzer ohne Admin-Rolle erreicht die Seite nicht (Route-Guard) **und**
     bekommt vom Aggregat-Endpoint keine Daten (Backend-Gate) — beides getestet,
     nicht nur eines. HR-Nutzer ohne Admin-Rolle verlieren dabei bewusst den Zugang
     zur Sales-Person-Verwaltung (REQUIREMENTS.md §„Bewusste Änderung").

  5. Die Liste wird aus **einem** Aggregat-Request gefüllt (nicht mehr
     `LoadAllUsers` + `LoadAllSalesPersons`), und der neue Routen-Präfix steht im
     **selben Commit** in `shifty-dioxus/Dioxus.toml` — verifiziert per `dx serve`-
     Smoke, nicht nur per Unit-Test.

**Backend-Pfade (erwartet):** neuer Business-Logic-Service (Arbeitstitel
`PersonDirectoryService`), `rest-types` (`PersonEntryTO`, klein und zweckgebaut wie
`SalesPersonAbsenceTO`, keine Wiederverwendung von `SalesPersonTO`/`UserTO`),
`rest/` (ein Aggregat-`GET`), `shifty_bin/src/main.rs` (DI nach den Basic-Services).

**Frontend-Pfade (erwartet):** `shifty-dioxus/src/page/user_management.rs`
(Tab-Leiste raus), `service/user_management.rs` (Store bedient bereits beide
Entitäten — kein Neuschnitt nötig), `shifty-dioxus/Dioxus.toml`, i18n de/en/cs.

**Discuss-Points (in der discuss-phase zu pinnen):**

- **D-62-01:** Braucht das Listen-DTO ein explizites `kind`-Enum, oder sind
  `Option`-gefüllte Felder (`sales_person_id` / `username`) Signal genug?
  (SUMMARY offene Frage 8 — kleine DTO-Form-Entscheidung, kein Blocker.)

- **D-62-02:** Die Liste wird aus dem Outer-Join `sales_person`/`user` abgeleitet,
  **nicht** direkt aus `person` — dann sind verwaiste `person`-Zeilen unsichtbar,
  unabhängig davon, wie D-63-01 ausgeht (PITFALLS Nr. 5). Bestätigen oder
  begründet verwerfen.

**UI hint**: yes

**Plans:** 7 plans

Plans:

**Wave 1**

- [x] 62-01-PLAN.md — DAO-Schicht: WR-02-Soft-Delete-Fix in der `ON`-Klausel (D-62-05) plus neues `PersonDirectoryDao` mit Drei-Wege-Outer-Join und Rollen-Bulk-Query
- [x] 62-02-PLAN.md — Transport-DTO `PersonEntryTO`/`PersonKindTO` (D-62-01/D-62-03) und sechs neue i18n-Keys in de/en/cs

**Wave 2** *(blocked on Wave 1)*

- [x] 62-03-PLAN.md — Business-Logic-Service `PersonDirectoryService`: Admin-Gate (D-62-08), Ausprägungs-Klassifikation, `display_name`-Ableitung, Sortierung, TO-Mapping
- [ ] 62-04-PLAN.md — Frontend: Aggregat-Datenpfad statt N+1, `PersonKindBadge`, eine Liste statt Tabs, Component-Level-Admin-Guard, Tab-Leiste entfernt

**Wave 3** *(blocked on Wave 2)*

- [ ] 62-05-PLAN.md — REST-Endpoint `GET /person-directory`, OpenAPI, DI nach dem Basic-Block, `Dioxus.toml`-Proxy im selben Commit, Integrationstests Admin→200 / HR-ohne-Admin→403

**Wave 4** *(blocked on Wave 3)*

- [ ] 62-06-PLAN.md — Docs-Freshness-Gate: `F01-employee-management` und `02-service-tiers` je EN + DE, `diagrams/service-graph-runtime.mmd`
- [ ] 62-07-PLAN.md — Phasen-Verifikation: `dx serve`-Proxy-Smoke, Ein-Request-Nachweis, Fat-Backend-Grep-Artefakt, visueller Checkpoint

---

#### Phase 63: Personen-Detailansicht + Ausprägungs-Aktionen

**Goal:** Ein Admin verwaltet eine Person vollständig an einem Ort — anlegen,
fehlende Ausprägung zuschalten, mit einem bestehenden Objekt verknüpfen, entfernen
— ohne zwischen Seiten zu springen und ohne raten zu müssen, was ein Löschen
mitnimmt.

**Depends on:** Phase 62 (Liste als Einstieg, Aggregat-Service als Basis), Phase 61
(Verknüpfen/Lösen inkl. Konflikt-Ablehnung und Kandidaten-Lesepfad).

**Requirements:** PADM-02, PADM-03, PADM-04, PADM-05, PADM-07

**Success Criteria** (what must be TRUE):

  1. Eine Detailansicht zeigt für einen verknüpften Eintrag Stammdaten +
     Schichtplan-Zuordnung **und** Rollen + Einladungen; bei einem reinen
     Benutzerkonto fehlen die Verkäufer-Abschnitte, bei einem reinen Verkäufer die
     Benutzer-Abschnitte — dieselbe Komponente, keine zweite Route, keine leeren
     Platzhalter-Abschnitte.

  2. Beim Anlegen erfasst der Admin **einen** Eintrag und schaltet dabei zu, ob er
     ein Benutzerkonto und/oder einen Verkäufer-Datensatz bekommt; nach dem Speichern
     existieren genau die gewählten Objekte, verknüpft über dieselbe Person.

  3. Bei einem Eintrag mit fehlender Ausprägung legt „Login hinzufügen" bzw.
     „Person hinzufügen" das fehlende Objekt an und verknüpft es — der Eintrag
     wechselt sichtbar seine Ausprägung, ohne dass der Admin die Seite verlässt.

  4. Zwei bereits getrennt bestehende Objekte lassen sich verknüpfen, indem der
     Admin aus einer Liste der **noch unverknüpften** Kandidaten auswählt (kein
     Eintippen eines Benutzernamens); bereits verknüpfte Ziele erscheinen dort nicht
     und werden serverseitig mit 409 abgelehnt, falls sie doch angefragt werden.

  5. Vor dem Entfernen einer Ausprägung oder dem Löschen eines Eintrags nennt die
     Bestätigung konkret, was verschwindet und was bestehen bleibt — und
     unterscheidet dabei die Verkäufer-Seite (Schichten, Stunden,
     Abrechnungs-Historie) von der Benutzer-Seite (Login, Rollen, Einladungen).

**Rollen-Gating:** ausschließlich die Seiten-Ebene aus PADM-08 (Admin). Sektions-
Sichtbarkeit richtet sich nach der **Ausprägung**, nicht nach der Rolle —
abschnittsweises Rollen-Gating ist bewusst PADM-F-02 (Future), siehe „Gesperrte
Entscheidungen".

**Discuss-Points (in der discuss-phase zu pinnen):**

- **D-63-01:** Was passiert mit der `person`-Zeile, wenn die letzte Ausprägung
  entfernt wird — mitlöschen, liegen lassen, oder in einer Admin-Sicht zeigen?
  (SUMMARY offene Frage 4; PITFALLS empfiehlt „letztes Lösen löscht die Person"
  als einfachste Variante, nennt es aber ausdrücklich eine Produkt-Entscheidung.)

- **D-63-02:** Wie stark trennt die Maske „neu anlegen" von „bestehendes Objekt
  verknüpfen"? Zwei getrennt beschriftete Aktionen statt eines mehrdeutigen Feldes
  (FEATURES-Empfehlung, Keycloak-Präzedenz).

**Verifikations-Hinweis:** Die Ausprägungs-abhängige Sektions-Sichtbarkeit ist
DOM-Rendering-Logik, die sich in Dioxus/WASM headless schlecht prüfen lässt
(PITFALLS Nr. 10, dieselbe Klasse wie die zwei `human_needed`-Overrides aus v2.6).
Entscheidungslogik in reine, unit-getestete Helfer ziehen **und** einen
Human-UAT-Durchgang von vornherein einplanen, statt ihn beim Close zu entdecken.

**UI hint**: yes

**Plans:** TBD

---

#### Phase 64: Planungs-Soll für dynamische Verträge

**Goal:** Die Übersicht unterhalb des Schichtplans zeigt auch bei dynamischen
Verträgen ein Stundenziel für die Planung — fertig berechnet im Backend, ohne dass
sich am Stundenkonto irgendetwas bewegt.

**Depends on:** **Nichts.** Architektonisch unabhängig von den Phasen 60–63 — die
Phase berührt `service_impl/src/reporting.rs`, `rest-types` und
`shifty-dioxus/src/.../working_hours_mini_overview.rs`, keine Person-/User-Tabelle,
keinen Person-Service. Sie ist jederzeit vorziehbar.

**Warum trotzdem ans Ende sequenziert** (bewusste Entscheidung, keine Abhängigkeit):

- Die Person-Kette trägt das irreversible Risiko dieses Milestones (Migration und
  Backfill gegen Live-Daten). Sie bekommt deshalb den frühesten Start und die
  längste Liegezeit zwischen Ausführung und Milestone-Close.

- Ein Fehler im Planungs-Soll ist anzeigeseitig und rückbaubar; ein Fehler im
  Backfill ist es nicht. Das riskantere Stück zuerst.

- Die Phase ist klein und in sich geschlossen (wenige Dateien, keine Migration) —
  sie kann Terminsdruck am Ende absorbieren, ohne etwas halbfertig zu hinterlassen.

- Sie darf sich **keine** Phase mit der Person-Arbeit teilen: ihre eigene Falle
  (zwei unabhängige `ShortEmployeeReport`-Konstruktionsstellen) braucht eine
  eigene Test-Matrix und würde in einer gemischten Phase untergehen.

**Requirements:** PSOLL-01, PSOLL-02, PSOLL-03, PSOLL-04

**Success Criteria** (what must be TRUE):

  1. In der Übersicht unterhalb des Schichtplans steht für einen Mitarbeiter mit
     dynamischem Vertrag eine Sollstunden-Zahl > 0 statt der heutigen 0; für einen
     Mitarbeiter mit festem Vertrag steht dieselbe Zahl wie heute.

  2. Dieselbe Person zeigt in der Wochen-Übersicht
     (`GET /report/week/{year}/{calendar_week}`) und im Gesamt-Report
     (`GET /report?year&until_week`) dasselbe Planungs-Soll — **beide**
     `ShortEmployeeReport`-Konstruktionsstellen (`assemble_weeks` und
     `get_reports_for_all_employees`) werden in dieser Phase gefüllt und durch
     denselben Test gegeneinander gehalten.

  3. Das Planungs-Soll kommt fertig aus dem Backend; im Frontend existiert keine
     Fallback- oder Ableitungsregel (grep-verifiziert). Der Wert ist über dieselbe
     Feiertags-/Abwesenheits-Reduktion berechnet wie `expected_hours`, **nicht** aus
     `dynamic_hours` abgeleitet (der naheliegende Fix, den die Bug-Analyse bereits
     als falsch belegt hat).

  4. Für jede bestehende Nicht-dynamisch-Fixture der Reporting-Testsuite gilt
     `planning_target_hours == expected_hours` (Byte-Identität) — für bestehende
     Mitarbeiter verschiebt sich damit nachweislich nichts.

  5. `available_hours` in `WorkingHoursPerSalesPerson` liest weiterhin
     `expected_hours` (bei dynamischen Verträgen genullt) — Regressions-Lock an
     **beiden** Stellen (`booking_information.rs:644` und `:818`). Balance-,
     Carryover- und Billing-Pfade liefern für dieselben Ausgangsdaten unveränderte
     Werte; `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **13**.

**Discuss-Points (in der discuss-phase zu pinnen):**

- **D-64-01:** Welche der Folge-Konsumenten werden auf das Planungs-Soll
  umgestellt — `hours_text_class`, `progress_bar_class`, `total_target` in
  `working_hours_mini_overview.rs`? Das ist laut Origin-Todo **eine bewusste
  Entscheidung pro Stelle**, kein pauschales Suchen-und-Ersetzen.

- **D-64-02:** Soll `get_year` das Feld auf seinen `ShortEmployeeReport`-Zeilen
  ebenfalls führen, oder bleibt die Jahresansicht außen vor (rendert die Spalte
  heute nicht)? (SUMMARY offene Frage 10.)

- **D-64-03:** Braucht die Anzeige einen visuellen Hinweis, dass diese Zahl reines
  Planungsziel ist und die Balance nicht bewegt — jetzt, wo bilanzrelevantes Soll
  und Planungs-Soll erstmals nebeneinander sichtbar werden? (PITFALLS §UX.)

**Research-Flag:** Vor dem Planen die Reduktions-Pipeline in `assemble_weeks`
(`:1080-1101`) und in `get_reports_for_all_employees` (`:1340-1420`) **zeilenweise**
gegeneinander lesen — die beiden Schleifen sind unabhängig geschrieben und könnten
seit dem WOP-01-Refactor auseinandergelaufen sein (ARCHITECTURE flaggt das selbst
als unsicher).

**UI hint**: yes

**Plans:** TBD

### Progress Table

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 60. Person-Datenmodell + Backfill | 0/? | Not started | — |
| 61. PersonService + Auflösung `sales_person_user` | 10/10 | Complete | 2026-09-17 |
| 62. Personen-Verzeichnis — eine Liste statt Tabs | 3/7 | In Progress | — |
| 63. Personen-Detailansicht + Ausprägungs-Aktionen | 0/? | Not started | — |
| 64. Planungs-Soll für dynamische Verträge | 0/? | Not started | — |

**Requirements-Coverage: 18/18 gemappt**, keine Waisen, keine Doppelung.

| Phase | Requirements | Anzahl |
|---|---|---|
| 60 | PERS-01, PERS-02, PERS-03, PERS-04, PERS-06 | 5 |
| 61 | PERS-05 | 1 |
| 62 | PADM-01, PADM-06, PADM-08 | 3 |
| 63 | PADM-02, PADM-03, PADM-04, PADM-05, PADM-07 | 5 |
| 64 | PSOLL-01, PSOLL-02, PSOLL-03, PSOLL-04 | 4 |

## Phasen

<details>
<summary>✅ v2.6 Freiwillige-Stunden-Ausgleich für gedeckelte Mitarbeiter (Phasen 54–57) — SHIPPED 2026-07-31, deployed</summary>

- [x] Phase 54: Data-Model + Voluntary Statistics (F1 + F2) — 9/9 Pläne
- [x] Phase 54.5: Voluntary-Soll Absence-Aware-Angleichung an VFA-01 (Hotfix v2.6.1, INSERTED) — 2/2 Pläne
- [x] Phase 55: Manuelle Umbuchung + HR-Alert-Modal (F3 + F5) — 6/6 Pläne
- [x] Phase 56: Wochen-Cron + Rollout-Backfill (F4) — 6/6 Pläne
- [x] Phase 57: F14-Rebooking-Wirksamkeit — Requirements-Korrektur + Balance-Ausgleich (INSERTED) — 6/6 Pläne

Detail-Archiv: [`milestones/v2.6-ROADMAP.md`](milestones/v2.6-ROADMAP.md), [`milestones/v2.6-phases/`](milestones/v2.6-phases/), Audit [`milestones/v2.6-MILESTONE-AUDIT.md`](milestones/v2.6-MILESTONE-AUDIT.md).

</details>

Ältere Milestones: siehe die jeweiligen `milestones/v*-ROADMAP.md` (oben verlinkt).

## Vorbereitete Phasen (noch keinem Milestone zugeordnet)

Diese Phasen sind ausgearbeitet, gehören aber zu keinem aktiven Milestone. Beim
nächsten `/gsd-new-milestone` entweder promoten oder bewusst liegen lassen.

- [ ] **Phase 58: Optionale Stundenverteilung pro Werktag** (Feature, Backend + Frontend) — sieben optionale `REAL`-Spalten auf der bereits versionierten `employee_work_details`-Vertragszeile; der bestehende Vertragswechsel-Mechanismus liefert „gültig ab KW" gratis. Betrifft beide Ableitungen der Tagesstunde (`hours_per_day`, `holiday_hours`) und damit Holiday-Derivation und Absence-Zählung. **Zieht Snapshot-Bump 13→14 nach sich.** Enabler für die Vertragszeit-Warnung. Entwurf: `.hermes/plans/2026-08-24_feature-phase-optionale-stundenverteilung-pro-werktag-snapshot-bump-13-14.md`.

- [ ] **Phase 59: E2E-Browser-Test-Infrastruktur (Playwright)** (Maintenance, off-theme) — kein Milestone-Goal, kein REQUIREMENTS-Eintrag; Abnahmemessstab ist der Scope aus dem Origin-Todo. Deckt genau die Invarianten-Klasse ab, die in v2.6 zweimal als `human_needed` stehenblieb (DOM-Rendering), plus die wiederkehrende Dev-Proxy-Bug-Klasse (fehlender `[[web.proxy]]`-Eintrag → 405). Plan liegt fertig: `.planning/phases/59-e2e-browser-testinfra/59-PLAN.md`.

## Backlog

Ungeplante / off-theme Arbeit, die NICHT zum aktiven Milestone gehört. Vor Ausführung
in einen Milestone promoten oder per `/gsd-plan-phase 999.1` direkt planen.

- [ ] **Phase 999.1: Breaking/Major Dependency-Migration** (Backend + Frontend, Maintenance) — Alle direkten Deps mit verfügbaren Major-Releases über beide Cargo-Workspaces (Backend-Root + `shifty-dioxus/`, 9 Member-Crates) auf den neuen Major heben (Cargo.toml-Constraint-Edits + Code-/API-Migration).

  **Goal:** Reproduzierbares Breaking-Update-Tooling etabliert und alle tragbaren Major-Bumps migriert, mit grünen Gates über beide Workspaces — ohne die heiklen Pins (dioxus 0.6.x) ungefragt anzufassen.

  **Context:** Quick-Task `260627-vgo` hat die **semver-kompatible** Baseline bereits geliefert (nur Cargo.lock, alle Gates grün). Offen ist NUR der Breaking/Major-Teil, der dort eskaliert wurde, weil die gepinnte **stable cargo 1.95.0** kein `cargo update --breaking` kann (nightly-only) und weder `cargo-edit` (`cargo upgrade`) noch `cargo-outdated` noch `+nightly` verfügbar sind.

  **Scope / grobe Wave-Struktur:**

  - Task 1 — Toolchain-Enabler: nightly-Toolchain bzw. `cargo-edit`/`cargo-outdated` ins `flake.nix` aufnehmen, sodass `cargo update --breaking` oder `cargo upgrade --incompatible` reproduzierbar laufen.
  - Task 2 — Major-Bump-Inventar: welche direkten Deps, welcher Sprung, Changelog-/Breaking-Risiko (beide Workspaces).
  - Task 3 — iterativ pro Major migrieren mit Gates: Backend `cargo build` + `cargo clippy --workspace -- -D warnings` + `cargo test --workspace`; Frontend `cargo build --target wasm32-unknown-unknown` (nix-shell -p openssl pkg-config lld) + `cargo test`.

  **Constraints:**

  - **dioxus-Major** (0.6.x-Pin) NUR mit expliziter User-Freigabe — dx-CLI-0.7-Inkompatibilität dokumentiert (App startet nicht + Design gestrippt).
  - `flake.lock` Nix-Inputs sind NICHT Teil dieser Phase (separater Maintenance-Job).
  - jj-Repo: User committet manuell, keine git-Fallbacks.

  **Depends on:** Quick-Task `260627-vgo` (compatible baseline) ✅
  **Plans:** 1/1 plans complete

*Last updated: 2026-09-16 — Milestone v2.7 angelegt: Phasen 60–64, 18/18 Requirements gemappt (5 → P60, 1 → P61, 3 → P62, 5 → P63, 4 → P64). Phasen 58 + 59 bleiben reserviert und unangetastet; Backlog 999.1 unverändert.*
