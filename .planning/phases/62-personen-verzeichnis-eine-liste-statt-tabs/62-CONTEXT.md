# Phase 62: Personen-Verzeichnis — eine Liste statt Tabs - Context

**Gathered:** 2026-09-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Diese Phase ersetzt die heutige Zwei-Tab-Ansicht unter `/user_management/` durch
**eine** durchsuchbare Liste aller Einträge — verknüpfte, reine Benutzerkonten und
reine Verkäufer nebeneinander, jeder mit sichtbarer Ausprägung. Gefüllt wird sie
aus **einem** Aggregat-Request gegen einen neuen Business-Logic-Lese-Service
(Arbeitstitel `PersonDirectoryService`) über ein neues, zweckgebautes
`PersonEntryTO`. Der Anzeigename kommt fertig aus dem Backend.

**Backend + Frontend.** Backend: neuer Business-Logic-Service (Tier-Regel: er
konsumiert Basic-Services/DAOs, liegt in der DI **nach** dem Basic-Block),
`rest-types` (`PersonEntryTO`), ein Aggregat-`GET`, DI in
`shifty_bin/src/main.rs`. Frontend: `page/user_management.rs` (Tab-Leiste raus),
`service/user_management.rs` (neuer Loader), `Dioxus.toml`-Proxy-Eintrag im
**selben** Commit, i18n de/en/cs.

**Rein lesend plus Erhalt des Bestehenden.** Die Detailansicht und alle
Ausprägungs-Mutationen (Anlegen mit zuschaltbaren Teilen, Verknüpfen per
Kandidaten-Picker, ausprägungs-spezifisches Löschen) sind **Phase 63**. Diese
Phase legt keine neue Detail-Route an.

**Requirements:** PADM-01, PADM-06, PADM-08

</domain>

<decisions>
## Implementation Decisions

### DTO-Form und Lesequelle

- **D-62-01:** **`PersonEntryTO` trägt ein explizites `kind`-Enum** (verknüpft /
  nur Benutzer / nur Verkäufer), **nicht** nur `Option`-gefüllte Felder, aus denen
  das Frontend die Ausprägung ableitet. Success Criterion 1 verlangt eine sichtbare
  Ausprägung pro Zeile; eine FE-seitige Ableitung aus zwei `Option`s wäre genau die
  Fallback-Regel, die Success Criterion 3 und das Fat-Backend-Prinzip verbieten.
  Damit löst diese Phase SUMMARY-offene-Frage 8.
  — **Reversibility:** reversible — additive Enum-Variante in einem neuen, noch
  von nichts konsumierten TO.

- **D-62-02:** **Die Liste wird aus dem Outer-Join `sales_person` ↔ `person` ↔
  `user` abgeleitet, nicht direkt aus `person`.** Roadmap-Discuss-Point bestätigt.
  Folge: verwaiste `person`-Zeilen ohne Träger sind in der Liste unsichtbar,
  unabhängig davon, wie D-63-01 ausgeht (PITFALLS Nr. 5). Nach **D-61-05** kann
  der Fall ohnehin nicht mehr durch Lösen entstehen — jede Person behält nach
  jeder Operation mindestens einen Träger. Die Join-Richtung ist dieselbe wie in
  den Phase-61-Kandidatenpfaden (`dao_impl_sqlite/src/person.rs:155-189`).

- **D-62-03:** **Das TO trägt Anzeigename und Username als zwei getrennte
  Felder** (`display_name` + optionaler `username`), plus die Verkäufer-Attribute,
  die die heutige Liste zeigt (`sales_person_id`, `background_color`, `is_paid`,
  `inactive`) und die Rollen. `display_name` ist der Verkäufername, wo vorhanden,
  sonst der Username — **die Ableitung passiert im Backend** (PADM-06,
  Success Criterion 3; grep-verifiziert keine Fallback-Regel im FE, Präzedenz
  v2.4 `clip_to`-Lock). Bei einem verknüpften Eintrag sind beide Felder gefüllt und
  **beide werden in der Liste angezeigt** — Verkäufername als Zeilen-Titel, Username
  in einer eigenen Spalte. Bei einem reinen Benutzerkonto trägt `username` denselben
  Wert wie `display_name`; das Frontend rendert das stumpf und unterdrückt nichts.
  (User-Entscheidung 2026-09-19: „Vielleicht sollte irgendwie beides angezeigt
  werden.")

### Umfang der Liste

- **D-62-04:** **Die Liste zeigt aktive **und** inaktive Verkäufer**, inaktive
  weiter ausgegraut mit Inaktiv-Badge (heutiges Verhalten,
  `page/user_management.rs:271-276, 313-319`). **Soft-gelöschte Verkäufer erscheinen
  nicht** — `WHERE sp.deleted IS NULL` bleibt, wie in
  `sales_persons_without_user` (`dao_impl_sqlite/src/person.rs:163`).

- **D-62-05:** **WR-02 aus dem Phase-61-Code-Review wird in dieser Phase
  mitgefixt.** `users_without_sales_person` (`dao_impl_sqlite/src/person.rs:173-189`)
  filtert heute nicht auf `sp.deleted IS NULL`; ein Benutzer, dessen einzige Sales
  Person soft-gelöscht ist, fällt dadurch aus **beiden** Kandidatenlisten und ließe
  sich über den Phase-63-Picker nie wieder verknüpfen. Der Fix ist der vom Review
  vorgeschlagene: Filter in die **`ON`-Klausel**, nicht in `WHERE` (sonst wird der
  LEFT JOIN zum INNER JOIN), plus Test
  `user_mit_nur_soft_geloeschter_sales_person_ist_kandidat`, plus Korrektur des
  irreführenden Doc-Kommentars (`dao/src/person.rs:65-66`) und der daraus
  übernommenen Formulierung in `docs/features/F01-employee-management.md` (EN+DE).
  Begründung für den Ort: derselbe Join entsteht in dieser Phase ohnehin, und die
  beiden Kandidatenpfade werden damit symmetrisch. Dieselbe Soft-Delete-Semantik
  gilt für das Aggregat aus D-62-04 — eine soft-gelöschte Sales Person bindet ihren
  Benutzer nirgends mehr.
  — **Reversibility:** reversible — reine Query-Änderung mit Test.

### Aktionen und Navigation

- **D-62-06:** **Die heutigen Listen-Aktionen bleiben in Phase 62 vollständig
  erhalten** und wandern in die vereinte Liste: „Neuen Benutzer anlegen" (Dialog),
  Benutzer löschen (🗑 + Bestätigungs-Dialog), „Act as this person"
  (Impersonation, D-32-07/IMP-01) und die Rollen-Spalte. Anlegen, Löschen und
  Impersonation sind nur für Einträge sichtbar, die eine **Benutzer-Ausprägung**
  haben. Grund: sonst entstünde zwischen Phase 62 und 63 eine Funktionslücke in
  einer Seite, die bis dahin produktiv ist. Phase 63 ersetzt diese Aktionen durch
  die ausprägungs-bewussten Varianten (PADM-02/03/04/07).

- **D-62-07:** **Phase 62 legt keine neue Detail-Route an.** Eine Zeile verlinkt
  weiter auf die bestehenden Detailseiten: verknüpfter Eintrag und reiner Verkäufer
  → `Route::SalesPersonDetails`, reines Benutzerkonto → `Route::UserDetails`. Beide
  Seiten bleiben in dieser Phase unangetastet. Phase 63 ersetzt das durch **eine**
  Detail-Route. Die Alternative (leere `/person/{id}`-Hülle schon in 62) wurde
  verworfen — sie kostet eine Route, die 63 sofort wieder umbaut.

### Gating

- **D-62-08:** **Admin-Gate an beiden Enden, beides getestet** (Success Criterion 4).
  Backend: der Aggregat-Endpoint prüft `"admin"` — dasselbe Privileg, das
  `PermissionServiceImpl::get_all_users` (`service_impl/src/permission.rs:162`) heute
  schon verlangt, **nicht** `HR_PRIVILEGE`. Frontend: Component-Level-Guard nach dem
  Muster `page/settings.rs:556-570` (`AUTH.read().auth_info…has_privilege("admin")`,
  frühes `return` mit „Not authorized."), weil die Nav-Ausblendung
  (`component/top_bar.rs:55`) einen direkten URL-Aufruf nicht abfängt.
  **HR-Nutzer ohne Admin-Rolle verlieren dabei bewusst den Zugang zur
  Sales-Person-Verwaltung** — REQUIREMENTS.md §„Bewusste Änderung", vom User am
  2026-09-16 explizit entschieden.
  — **Reversibility:** reversible — Gate-Konstante an zwei Stellen.

### Gesperrt — in dieser Phase nicht neu aufmachen

- `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **13**. Keine der vier Bump-Klauseln aus
  `CLAUDE.md` greift: kein neuer persistierter `value_type`, keine geänderte
  Computation, kein geändertes Input-Set. **Nicht „vorsichtshalber" bumpen.**
- **Keine Migration in dieser Phase.** Das Schema steht seit Phase 60/61; diese
  Phase liest nur.
- **Fat Backend, Thin Client.** Der Anzeigename und die Ausprägung kommen fertig
  aus dem Backend. Grep-Nachweis, dass im FE keine Namens-Fallback-Regel existiert,
  ist Pflicht-Artefakt (Präzedenz v2.4 `clip_to`-Lock, v2.5 D-53-05/06).
- **Ein Aggregat-Request statt `LoadAllUsers` + `LoadAllSalesPersons`** — und ohne
  die heutigen N+1-Nachladepfade (`load_all_sales_person_user_links`,
  `load_all_user_sales_person_links`, `load_all_user_roles` in
  `service/user_management.rs:174-225` feuern je **einen Request pro Zeile**).
- **`Dioxus.toml`-Proxy-Eintrag für den neuen Routen-Präfix im selben Commit**,
  verifiziert per `dx serve`-Smoke, nicht nur per Unit-Test (Roadmap-Constraint 6 —
  dieses Repo hat den Bug dreimal geshipt: v1.6 `/toggle`, v1.8
  `/vacation-entitlement-offset`, v1.10 `/special-days`).
- **Tier-Regel:** Das Lese-Aggregat ist **Business-Logic-Tier** (es kombiniert
  mehrere Aggregate), `PersonService` bleibt Basic-Tier. DI-Konstruktion in
  `shifty_bin/src/main.rs` strikt basic-vor-business-logic, keine
  `OnceLock`-/Forward-Decl-Tricks (`CLAUDE.md` §Service-Tier-Konventionen,
  Roadmap-Constraint 5).
- **`sales_person.id` und `user.name` werden nicht umgeschlüsselt**;
  `/employees/:id` (HR-Report) bleibt eine getrennte Sicht (PADM-F-01);
  abschnittsweises Rollen-Gating ist PADM-F-02, nicht v2.7.
- **`SalesPersonTO` / `UserTO` werden nicht wiederverwendet** — `PersonEntryTO` ist
  klein und zweckgebaut, Präzedenz `SalesPersonAbsenceTO` (v2.5).

### Claude's Discretion

- **Routen-Präfix und Endpoint-Zuschnitt** des Aggregat-`GET` (z. B. `/person` vs.
  `/person-directory`) — mit der Auflage, dass der Präfix im selben Commit in
  `Dioxus.toml` steht.
- **Genauer Name und Modul-Ablage** des Business-Logic-Services (Arbeitstitel
  `PersonDirectoryService`) sowie die Frage, ob das Aggregat eine neue DAO-Methode
  bekommt oder bestehende kombiniert.
- **Serde-Form des `kind`-Enums** (Tag-Name, Varianten-Schreibweise) und die
  Feldnamen im `PersonEntryTO`.
- **Sortierung der Liste** (Vorschlag: case-insensitive nach `display_name`,
  analog v2.5 VAA-Union-Merge) und ob die Ausprägung zusätzlich als Filter neben
  der Suche angeboten wird.
- **Ob die Suche im Backend oder im Frontend filtert.** Bei dieser Nutzerzahl ist
  FE-seitiges Filtern über die geladene Liste vertretbar — es ist reine
  UI-State-Logik und fällt damit unter die ausdrückliche Ausnahme im
  Fat-Backend-Prinzip. `matches_search` (`page/user_management.rs:32`) ist
  wiederverwendbar; Suchkriterium sind `display_name` **und** `username`
  (Success Criterion 2).
- **Ob die Rollen-Spalte im Aggregat mitkommt oder weiter separat lädt** — mit der
  Auflage, dass Success Criterion 5 („ein Aggregat-Request") erfüllt bleibt, also
  kein Per-Zeile-Nachladen zurückkommt.
- **i18n-Key-Namen** für Ausprägungs-Badge und Spaltenköpfe; bestehende Keys
  (`ColumnType`, `ColumnLinkedUser`, `ColumnRoles`, `Unlinked`,
  `SearchPlaceholder`, `Paid`, `Volunteer`, `Inactive`) werden wiederverwendet, wo
  sie passen. Alle drei Sprachen de/en/cs synchron.
- **Verbleib von `UserManagementTabBar` / `UserMgmtTab`** — löschen oder stehen
  lassen, falls andere Aufrufer existieren.
- **Wellen-Schnitt, Plan-Anzahl und Testablage.**

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Gesperrte Entscheidungen & Scope
- `.planning/ROADMAP.md` §„Milestone v2.7" → „Gesperrte Entscheidungen",
  „Harte Constraints aus der Research" (Punkte **5** Tier-Regel und **6**
  Dev-Proxy betreffen Phase 62), „Backend-/Frontend-Abdeckung pro Phase",
  „Docs-Freshness-Gate" (Zeile Phase 62/63), „Phase Details → Phase 62"
  (Success Criteria, D-62-01/02).
- `.planning/REQUIREMENTS.md` — PADM-01, PADM-06, PADM-08; §„Bewusste Änderung
  gegenüber heute" (HR verliert die Sales-Person-Verwaltung); §„Out of Scope";
  PADM-F-01/F-02 als Future.
- `.planning/seeds/personenverwaltung-zusammenlegen.md` — Variante A, die drei
  Ausprägungen, „Namen bleiben wo sie sind", die No-Bump-Begründung.
- `.planning/PROJECT.md` §„Fat Backend, Thin Client", §„GSD-Scope-Regel"
  (Plan-Header muss Backend- und Frontend-Pfade explizit nennen).

### Vorgänger-Entscheidungen, die hier tragen
- `.planning/phases/61-personservice-aufl-sung-von-sales-person-user/61-CONTEXT.md`
  — **D-61-01** (jede Sales Person und jeder Benutzer trägt genau eine Person),
  **D-61-02** („unverknüpft" ist nicht `person_id IS NULL`, sondern Outer-Join),
  **D-61-05** (keine verwaisten `person`-Zeilen durch Lösen — nimmt dieser Phase
  die Ghost-Eintrag-Frage ab), **D-61-04** (409-Semantik, relevant für Phase 63).
- `.planning/phases/60-person-datenmodell-backfill/60-CONTEXT.md` — **D-60-03**
  (drei gleichwertige Ausprägungen), **D-60-04** (`person` trägt nur Identität +
  Audit, keine Namen).

### Fallen mit direktem Phase-62-Bezug
- `.planning/research/PITFALLS.md` §Pitfall 5 — verwaiste/umgehängte
  `Person`-Zeilen; begründet D-62-02.
- `.planning/research/PITFALLS.md` §„Looks Done But Isn't" — Fat-Backend-Verstöße,
  die Tests nicht fangen.
- `.planning/research/ARCHITECTURE.md` — Build-Order, Tier-Klassifikation,
  Blast Radius mit `file:line`.
- `.planning/phases/61-personservice-aufl-sung-von-sales-person-user/61-REVIEW.md`
  §WR-02 — enthält den fertigen SQL-Fix und die Testbenennung für D-62-05.

### Bestehender Code, der umgebaut oder gelesen wird
- `shifty-dioxus/src/page/user_management.rs` — die zu ersetzende Seite:
  Tab-State (`:47`), zwei Suchfelder (`:48-49`), `matches_search` (`:32`),
  `SalesPersonTabContent` (`:220-350`), `UsersTabContent` (`:364-494`),
  `AddUserDialog`/`DeleteUserConfirmDialog` (`:91-114`), SSR-Testharness
  (`:506-560`).
- `shifty-dioxus/src/service/user_management.rs` — `UserManagementStore`
  (`:46-79`) und die drei N+1-Loader `load_all_sales_person_user_links` (`:174`),
  `load_all_user_sales_person_links` (`:190`), `load_all_user_roles` (`:209`).
- `shifty-dioxus/src/page/settings.rs:556-570` — Muster für den
  Component-Level-Admin-Guard (D-62-08).
- `shifty-dioxus/src/component/top_bar.rs:55` — Nav-Ausblendung `has("admin")`,
  bereits korrekt; der Guard ist die Defense-in-Depth dazu.
- `shifty-dioxus/Dioxus.toml:45-116` — die `[[web.proxy]]`-Liste, in die der neue
  Präfix gehört.
- `dao_impl_sqlite/src/person.rs:145-189` — `sales_persons_without_user` (mit
  Soft-Delete-Filter) und `users_without_sales_person` (ohne — der WR-02-Fix).
- `dao/src/person.rs:58-70` — `PersonDao`-Trait und der zu korrigierende
  Doc-Kommentar.
- `service/src/person.rs` — `PersonService`-Trait (Basic-Tier, bleibt es).
- `service_impl/src/permission.rs:158-170` — `get_all_users` mit
  `check_permission("admin")`, das Gate-Vorbild für D-62-08.
- `shifty_bin/src/main.rs` — DI-Verdrahtung; der neue Business-Logic-Service kommt
  **nach** dem Basic-Block.

### Konventionen & Doku-Gate
- `CLAUDE.md` §„Service-Tier-Konventionen: Basic vs. Business-Logic Services",
  §„REST API" (utoipa-Annotation Pflicht, `ToSchema` auf dem TO),
  §„Billing Period Snapshot Schema Versioning" (kein Bump), §„Docs-Freshness-Gate",
  §Clippy-Hard-Gate.
- `shifty-dioxus/CLAUDE.md` — Frontend-Konventionen (wird beim Lesen von
  Frontend-Files automatisch geladen).
- `.planning/codebase/frontend/` — `ARCHITECTURE.md`, `CONVENTIONS.md`,
  `TESTING.md` (SSR-Rendertests als Nachweisform).
- Docs-Trigger dieser Phase (EN **und** DE im selben Commit): neue/erweiterte
  `docs/features/F*.md` zur Personenverwaltung wegen `service/**` + `dao/**`;
  zusätzlich `docs/features/F01-employee-management.md` (+ `_de`) wegen der
  WR-02-Korrektur aus D-62-05.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`matches_search`** (`shifty-dioxus/src/page/user_management.rs:32`) — pure
  Filter-Funktion mit eigener Testabdeckung (`:565+`). Wird für die vereinte Suche
  über `display_name` **und** `username` wiederverwendet statt neu geschrieben.
- **SSR-Rendertest-Harness** (`page/user_management.rs:506-560`, `render_with` /
  `render_users_with`) — rendert eine Tab-Komponente mit injiziertem Store zu
  einem String. Direktes Zuhause für die Listentests (Ausprägungs-Badge sichtbar,
  Benutzerkonto ohne Verkäufernamen ist beschriftet, Suche findet über beide
  Namen).
- **Guard-Muster** `page/settings.rs:556-570` — frühes `return` mit
  „Not authorized.", identisch nutzbar; die Kommentarzeile dort nennt
  `billing_periods`, `shiftplan`, `absences` als weitere Präzedenzfälle.
- **`PersonChip`** und die Badge-Klassen (`page/user_management.rs:310-319`) —
  fertige Darstellung für Paid/Volunteer/Inactive, erweiterbar um die Ausprägung.
- **`SalesPersonAbsenceTO`** (v2.5) — Präzedenz für ein kleines, zweckgebautes
  Listen-DTO statt Wiederverwendung eines Entity-TO.
- **`dao_impl_sqlite/src/person.rs:145-189`** — die beiden Phase-61-Outer-Joins
  über `person_id`; der Aggregat-Join ist strukturell dieselbe Form, nur ohne
  `WHERE`-Ausschluss.

### Established Patterns
- **`sqlx::query_as!` mit Compile-Time-Checking** und expliziten
  `AS "name!: Type"`-Annotationen — die lokale Dev-DB muss migriert sein.
- **`gen_service_impl!`** für DI; Services nehmen `Option<Transaction>` und
  committen selbst.
- **Soft-Delete-Filter `WHERE deleted IS NULL`** ist Konvention — für Joins gehört
  er in die `ON`-Klausel, sonst degeneriert der LEFT JOIN (D-62-05).
- **Zentrales Error-Mapping** über den `error_handler`-Wrapper statt
  Handler-lokalem Status-Mapping (`rest/src/rebooking.rs:14-18`).
- **i18n-Keys in allen drei Sprachen** (`shifty-dioxus/src/i18n/{de,en,cs}.rs`) —
  es existiert ein Test, der die Key-Mengen gegen die deutsche Referenz vergleicht.
- **Fat-Backend-Nachweis per grep** ist in diesem Projekt die etablierte
  Beweisform (v2.4 `clip_to`, v2.5 D-53-05/06).

### Integration Points
- `shifty_bin/src/main.rs` — DI: neuer Business-Logic-Service nach dem Basic-Block;
  Trigger für `docs/architecture/02-service-tiers.md` (+ `_de`) **nur**, falls die
  Tier-Struktur sichtbar wächst.
- `rest/src/lib.rs` (Router-Zusammenbau) + `ApiDoc`-Struct — der neue Endpoint
  braucht Route **und** OpenAPI-Eintrag.
- `shifty-dioxus/Dioxus.toml` — Dev-Proxy; ohne Eintrag liefert `dx serve` einen
  404/405 auf den neuen Präfix, und kein Unit-Test fängt das.
- `shifty-dioxus/src/loader.rs` + `api.rs` — hier hängen die bestehenden
  Lade-Funktionen; der Aggregat-Loader kommt dazu, die drei N+1-Loader fallen an
  der Listen-Seite weg (ob sie ganz verschwinden, hängt daran, ob
  `user_details.rs` / `sales_person_details.rs` sie noch brauchen).

</code_context>

<specifics>
## Specific Ideas

- **Der sichtbare Maßstab ist „keine Tab-Leiste mehr".** Success Criterion 1 ist im
  Browser in einer Sekunde prüfbar; die Phase ist genau dann erfüllt, wenn ein
  Admin eine Liste sieht, in der ein reines Benutzerkonto, ein reiner Verkäufer und
  ein verknüpfter Eintrag nebeneinanderstehen und man jedem ansieht, was er ist.
- **„Beides anzeigen" ist ausdrücklich gewünscht** (User, 2026-09-19): bei einem
  verknüpften Eintrag sollen Verkäufername und Username beide sichtbar sein, nicht
  nur der eine. Die heutige Information aus **beiden** Tabs (verknüpfter Benutzer,
  verknüpfte Sales Person, Rollen, Paid/Volunteer, Inaktiv) soll in der vereinten
  Zeile erhalten bleiben — die Liste ist eine Zusammenführung, kein Verlust.
- **Die Phase darf keine Funktionslücke reißen.** Die Seite ist produktiv; alles,
  was heute in den Tabs geht (Benutzer anlegen, löschen, Impersonation), muss nach
  dem Umbau weiter gehen (D-62-06). Phase 63 löst das dann ausprägungs-bewusst ab.
- **Der N+1-Abbau ist Nebeneffekt, nicht Ziel.** Heute feuert die Seite bei N
  Verkäufern und M Benutzern 2 + N + 2M Requests. Der Aggregat-Request macht daraus
  einen — Success Criterion 5 misst aber die Request-Zahl, nicht die Latenz; kein
  Performance-Versprechen in dieser Phase.
- **Der Dev-Proxy ist der teuerste vergessbare Handgriff.** Dreimal in diesem Repo
  geshipt. Der Nachweis ist ein `dx serve`-Smoke gegen den neuen Präfix, nicht ein
  grüner Unit-Test.

</specifics>

<deferred>
## Deferred Ideas

- **Eine gemeinsame Detail-Route `/person/{id}`** samt ausprägungs-bewussten
  Abschnitten — Phase 63 (PADM-05). In Phase 62 verlinken die Zeilen bewusst noch
  auf die bestehenden Detailseiten (D-62-07).
- **Anlegen mit zuschaltbaren Ausprägungen, Nachschalten einer fehlenden
  Ausprägung, Kandidaten-Picker zum Verknüpfen, ausprägungs-spezifische
  Lösch-Bestätigung** — Phase 63 (PADM-02/03/04/07). In Phase 62 bleiben die
  heutigen Benutzer-Aktionen unverändert stehen.
- **Abschnittsweises Rollen-Gating** (HR bekommt Teile der Personenverwaltung
  zurück) — PADM-F-02, Future. Phase 62 gated die Seite als Ganzes auf Admin.
- **HR-Report `/employees/:id` in die Personen-Ansicht einfalten** — PADM-F-01,
  Future.
- **Echte Personenstammdaten** (E-Mail, Telefon, Geburtsdatum) an `person` —
  PERS-F-01, Future. `display_name` bleibt bis dahin abgeleitet, nicht gespeichert.
- **Zusammenführen zweier Personen im UI** — Merge-Mechanik existiert seit D-61-03,
  eine Bedienoberfläche dafür ist kein v2.7-Scope.
- **Backend-seitige Suche / Pagination** — bei dieser Nutzerzahl unnötig; FE-Filter
  über die geladene Liste genügt (siehe Claude's Discretion).
- **`WR-01`, `WR-03`, `WR-07` und `IN-01..IN-07`** aus `61-REVIEW.md` bleiben im
  Backlog; nur **WR-02** wird hier mitgenommen (D-62-05).

### Reviewed Todos (not folded)

Aus `todo.match-phase 62` — alle Treffer sind Keyword-Zufall über „admin",
„frontend", „user", keiner betrifft das Personen-Verzeichnis:

- **„Cutover-UI als Admin-Feature"**
  (`.planning/todos/pending/2026-05-08-cutover-ui-admin-feature.md`) — eigenes
  Admin-Frontend für `/admin/cutover/*`, ohne Bezug zur Personenverwaltung.
- **„Urlaub beantragen mit Genehmigungs-Workflow"**
  (`.planning/todos/pending/2026-07-01-urlaub-beantragen-mit-genehmigung.md`) —
  Abwesenheits-Domäne.
- **„admin-Rolle automatisch alle Privilegien"**
  (`.planning/todos/pending/2026-05-08-admin-rolle-bekommt-alle-privilegien.md`) —
  betrifft das Rollen-/Privilegien-Modell selbst; würde das Gate aus D-62-08
  verändern und gehört deshalb bewusst getrennt.
- **„Warnung bei Eintrag außerhalb der Vertragszeiten"**, **„list_user_invitations
  stilles leeres Array"**, **„silentRenewIframe in index.html"** — unverbundene
  Altlasten aus Mai 2026.

</deferred>

---

*Phase: 62-Personen-Verzeichnis — eine Liste statt Tabs*
*Context gathered: 2026-09-19*
