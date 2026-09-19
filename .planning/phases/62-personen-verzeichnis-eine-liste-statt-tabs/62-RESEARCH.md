# Phase 62: Personen-Verzeichnis — eine Liste statt Tabs - Research

**Researched:** 2026-09-19
**Domain:** Rust-Backend-Aggregat-Service (Axum/SQLx/utoipa) + Dioxus-Frontend-Listenseite
**Confidence:** HIGH

## Summary

Diese Phase ist überwiegend ein **Umbau bestehender, gut verstandener Bausteine**,
kein Neuland. Phase 61 hat bereits alle Fundamente gelegt: `person` ist die
Elterntabelle, `PersonDao`/`PersonService` liefern die beiden Kandidaten-Joins
im Basic-Tier, und beide Outer-Joins (`sales_person`↔`person`↔`user`) existieren
in `dao_impl_sqlite/src/person.rs:150-189` bereits fast in der Form, die das
neue Aggregat braucht — nur ohne `WHERE`-Ausschluss und ohne Soft-Delete-Bug
(D-62-05/WR-02). Frontend-seitig ist der komplette Zwei-Tab-Screen
(`shifty-dioxus/src/page/user_management.rs`) und sein N+1-Store
(`shifty-dioxus/src/service/user_management.rs`) bekannt und wird ersetzt,
nicht neu erfunden.

Der einzige wirklich neue Baustein ist der **Business-Logic-Aggregat-Service**
(Arbeitstitel `PersonDirectoryService`) mit seinem `PersonEntryTO`. Er braucht
eine **neue DAO-Methode** (kein bestehender DAO liefert heute den
Drei-Wege-Outer-Join `sales_person` ↔ `person` ↔ `user` inklusive Rollen in
einer Abfrage) — die bestehenden `PersonDao`-Methoden sind bewusst auf die
Kandidaten-Anti-Joins beschränkt (Doc-Kommentar `dao/src/person.rs:18-19`:
"Vorratshaltung … ist ausdrücklich unerwünscht"). Rollen sind der einzige
Datenpunkt, der heute **nur** per Einzel-User-Query existiert
(`PermissionDao::roles_for_user`, `dao_impl_sqlite/src/lib.rs:292-303`) — es
gibt keine bestehende Bulk-Query "Rollen für alle Benutzer". Wer die
Rollen-Spalte im Aggregat mitnehmen will (Claude's Discretion), muss also
entweder eine neue Bulk-DAO-Methode bauen oder die Spalte bewusst weglassen —
beides ist zulässig, aber die Auflage aus CONTEXT.md ("kein Per-Zeile-
Nachladen") schließt den heutigen `roles_for_user`-Call-pro-Zeile aus.

**Primary recommendation:** Neue DAO-Methode `PersonDao::list_directory_entries`
(oder gleichwertig in einem neuen `PersonDirectoryDao`, falls der Tier-Zuschnitt
das nahelegt) mit einem einzigen SQL-Statement, das `sales_person` UNION-artig
mit `user` über `person_id` outer-joint (zwei `LEFT JOIN`s von `person`, nicht
verschachtelte Joins von `sales_person`↔`user`, damit Soft-Delete auf
`sales_person.deleted` unabhängig von der `user`-Seite bleibt), plus
Rollen-Zusammenführung per zweiter Query oder `GROUP_CONCAT`. Der Service
klassifiziert `kind` in Rust, nicht in SQL. Backend liefert `display_name`
fertig berechnet (`sales_person.name` wenn vorhanden, sonst `user.name`) —
Frontend rendert nur.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Drei-Wege-Outer-Join lesen (sales_person/user/person) | Backend — API/Business-Logic | Datenbank (SQLite/SQLx) | Kombiniert zwei Aggregate (`SalesPersonService`- und `PermissionService`-Domäne) — per Definition Business-Logic-Tier (`CLAUDE.md` §Service-Tier-Konventionen), nicht Basic. |
| `display_name`-Ableitung (Verkäufername vs. Username) | Backend — Business-Logic-Service | — | Fat-Backend-Prinzip; Phase-Constraint verbietet FE-Fallback-Logik explizit (Success Criterion 3). |
| Ausprägungs-Klassifikation (`kind`-Enum) | Backend — Business-Logic-Service | — | Gleiche Begründung; D-62-01 verbietet FE-seitige Ableitung aus zwei `Option`s. |
| Admin-Gate (Backend) | Backend — API/Business-Logic-Service | — | `check_permission("admin", …)` im neuen Service, analog `PermissionServiceImpl::get_all_users` (`service_impl/src/permission.rs:158-170`). |
| Admin-Gate (Frontend, Defense-in-Depth) | Browser/Client (Dioxus-Component) | — | Component-Guard, weil Nav-Ausblendung URL-Direktzugriff nicht verhindert (`page/settings.rs:556-570` Präzedenz). |
| Suche über `display_name`/`username` | Browser/Client (Dioxus) | — | Claude's Discretion in CONTEXT.md ausdrücklich als FE-State-Ausnahme vom Fat-Backend-Prinzip erlaubt (geringe Nutzerzahl). |
| Listen-Aktionen (Anlegen/Löschen/Impersonate/Rollen) | Backend — bestehende `PermissionService`/`ImpersonateService`-Endpoints | Browser/Client (Buttons) | Unverändert wiederverwendet (D-62-06) — keine neuen Backend-Pfade nötig. |
| Dev-Proxy-Routing | Build/Tooling (`Dioxus.toml`) | — | Kein Laufzeit-Tier, aber Pflicht-Artefakt im selben Commit (Roadmap-Constraint 6). |

## User Constraints

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-62-01:** `PersonEntryTO` trägt ein explizites `kind`-Enum (verknüpft /
  nur Benutzer / nur Verkäufer), **nicht** nur `Option`-gefüllte Felder, aus
  denen das Frontend die Ausprägung ableitet. Reversibility: reversibel —
  additive Enum-Variante in einem neuen, noch von nichts konsumierten TO.
- **D-62-02:** Die Liste wird aus dem Outer-Join `sales_person` ↔ `person` ↔
  `user` abgeleitet, nicht direkt aus `person`. Verwaiste `person`-Zeilen ohne
  Träger sind in der Liste unsichtbar — nach D-61-05 kann dieser Fall ohnehin
  nicht mehr durch Lösen entstehen.
- **D-62-03:** Das TO trägt Anzeigename und Username als zwei getrennte Felder
  (`display_name` + optionaler `username`), plus Verkäufer-Attribute
  (`sales_person_id`, `background_color`, `is_paid`, `inactive`) und Rollen.
  `display_name` ist der Verkäufername, wo vorhanden, sonst der Username — die
  Ableitung passiert im Backend. Bei einem verknüpften Eintrag sind beide
  Felder gefüllt und **beide werden in der Liste angezeigt**. Bei einem reinen
  Benutzerkonto trägt `username` denselben Wert wie `display_name`.
- **D-62-04:** Die Liste zeigt aktive **und** inaktive Verkäufer, inaktive
  weiter ausgegraut mit Inaktiv-Badge. Soft-gelöschte Verkäufer erscheinen
  nicht — `WHERE sp.deleted IS NULL` bleibt.
- **D-62-05:** WR-02 aus dem Phase-61-Code-Review wird mitgefixt.
  `users_without_sales_person` filtert heute nicht auf `sp.deleted IS NULL`;
  Fix: Filter in die `ON`-Klausel (nicht `WHERE`, sonst degeneriert der LEFT
  JOIN zum INNER JOIN), plus Test
  `user_mit_nur_soft_geloeschter_sales_person_ist_kandidat`, plus Korrektur des
  Doc-Kommentars (`dao/src/person.rs:65-66`) und
  `docs/features/F01-employee-management.md` (EN+DE). Reversibility:
  reversibel — reine Query-Änderung mit Test.
- **D-62-06:** Die heutigen Listen-Aktionen bleiben vollständig erhalten und
  wandern in die vereinte Liste: „Neuen Benutzer anlegen" (Dialog), Benutzer
  löschen (🗑 + Bestätigungs-Dialog), „Act as this person" (Impersonation,
  D-32-07/IMP-01) und die Rollen-Spalte. Anlegen, Löschen und Impersonation
  sind nur für Einträge mit **Benutzer-Ausprägung** sichtbar.
- **D-62-07:** Keine neue Detail-Route in dieser Phase. Eine Zeile verlinkt
  weiter auf die bestehenden Detailseiten: verknüpfter Eintrag/reiner
  Verkäufer → `Route::SalesPersonDetails`, reines Benutzerkonto →
  `Route::UserDetails`.
- **D-62-08:** Admin-Gate an beiden Enden, beides getestet. Backend: der
  Aggregat-Endpoint prüft `"admin"` — dasselbe Privileg wie
  `PermissionServiceImpl::get_all_users`, **nicht** `HR_PRIVILEGE`. Frontend:
  Component-Level-Guard nach dem Muster `page/settings.rs:556-570`. HR-Nutzer
  ohne Admin-Rolle verlieren dabei bewusst den Zugang zur
  Sales-Person-Verwaltung. Reversibility: reversibel — Gate-Konstante an zwei
  Stellen.

### Gesperrt — in dieser Phase nicht neu aufmachen

- `CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt **13** — keine der vier
  Bump-Klauseln greift, nicht „vorsichtshalber" bumpen.
- Keine Migration in dieser Phase — das Schema steht seit Phase 60/61, diese
  Phase liest nur.
- Fat Backend, Thin Client — Anzeigename und Ausprägung kommen fertig aus dem
  Backend; Grep-Nachweis ohne FE-Fallback-Regel ist Pflicht-Artefakt.
- Ein Aggregat-Request statt `LoadAllUsers` + `LoadAllSalesPersons`, ohne die
  heutigen N+1-Nachladepfade (`load_all_sales_person_user_links`,
  `load_all_user_sales_person_links`, `load_all_user_roles`).
- `Dioxus.toml`-Proxy-Eintrag für den neuen Routen-Präfix im selben Commit,
  verifiziert per `dx serve`-Smoke, nicht nur per Unit-Test.
- Tier-Regel: Das Lese-Aggregat ist Business-Logic-Tier, `PersonService`
  bleibt Basic-Tier. DI-Konstruktion strikt basic-vor-business-logic.
- `sales_person.id` und `user.name` werden nicht umgeschlüsselt;
  `/employees/:id` bleibt getrennte Sicht.
- `SalesPersonTO`/`UserTO` werden nicht wiederverwendet — `PersonEntryTO` ist
  klein und zweckgebaut, Präzedenz `SalesPersonAbsenceTO`.

### Claude's Discretion

- Routen-Präfix und Endpoint-Zuschnitt des Aggregat-`GET` (Auflage: Präfix im
  selben Commit in `Dioxus.toml`).
- Name und Modul-Ablage des Business-Logic-Services; neue DAO-Methode vs.
  Kombination bestehender.
- Serde-Form des `kind`-Enums und Feldnamen im `PersonEntryTO`.
- Sortierung der Liste (Vorschlag: case-insensitive nach `display_name`); ob
  die Ausprägung zusätzlich als Filter neben der Suche angeboten wird.
- Ob die Suche im Backend oder im Frontend filtert — FE-seitig bei dieser
  Nutzerzahl vertretbar, `matches_search` wiederverwendbar.
- Ob die Rollen-Spalte im Aggregat mitkommt oder weiter separat lädt —
  Auflage: kein Per-Zeile-Nachladen.
- i18n-Key-Namen für Ausprägungs-Badge und Spaltenköpfe; bestehende Keys
  (`ColumnType`, `ColumnLinkedUser`, `ColumnRoles`, `Unlinked`,
  `SearchPlaceholder`, `Paid`, `Volunteer`, `Inactive`) wiederverwenden, wo
  passend.
- Verbleib von `UserManagementTabBar`/`UserMgmtTab` — löschen oder stehen
  lassen, falls andere Aufrufer existieren.
- Wellen-Schnitt, Plan-Anzahl und Testablage.

### Deferred Ideas (OUT OF SCOPE)

- Eine gemeinsame Detail-Route `/person/{id}` samt ausprägungs-bewussten
  Abschnitten — Phase 63 (PADM-05).
- Anlegen mit zuschaltbaren Ausprägungen, Nachschalten einer fehlenden
  Ausprägung, Kandidaten-Picker zum Verknüpfen, ausprägungs-spezifische
  Lösch-Bestätigung — Phase 63 (PADM-02/03/04/07).
- Abschnittsweises Rollen-Gating — PADM-F-02, Future.
- HR-Report `/employees/:id` in die Personen-Ansicht einfalten — PADM-F-01,
  Future.
- Echte Personenstammdaten (E-Mail, Telefon, Geburtsdatum) an `person` —
  PERS-F-01, Future.
- Zusammenführen zweier Personen im UI — kein v2.7-Scope.
- Backend-seitige Suche/Pagination — bei dieser Nutzerzahl unnötig.
- `WR-01`, `WR-03`, `WR-07` und `IN-01..IN-07` aus `61-REVIEW.md` bleiben im
  Backlog; nur `WR-02` wird mitgenommen (D-62-05).
</user_constraints>

## Phase Requirements

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PADM-01 | Ein Admin sieht **eine** Liste aller Einträge — verknüpfte, reine Benutzer und reine Verkäufer — mit erkennbarer Ausprägung und Suche, ohne Tab-Wechsel. | Neuer Aggregat-Endpoint + `PersonEntryTO.kind` liefert die Ausprägung fertig; FE-Suche über `matches_search` (bereits vorhanden, `page/user_management.rs:32`) auf `display_name` + `username`. Siehe "Standard Stack", "Code Examples". |
| PADM-06 | Der Anzeigename eines Eintrags kommt fertig vom Backend, sodass jeder Eintrag beschriftet ist — auch ein reines Benutzerkonto ohne Verkäufernamen. | `display_name`-Berechnung im neuen Business-Logic-Service (nicht SQL, nicht FE) — siehe "Pattern 2: display_name-Ableitung im Service" und "Don't Hand-Roll". |
| PADM-08 | Die Personenverwaltung ist vollständig admin-gated — sie öffnet sich nur für Nutzer mit Admin-Rolle. | Doppel-Gate: Backend `check_permission("admin", …)` (Präzedenz `service_impl/src/permission.rs:162`) + FE-Component-Guard (Präzedenz `page/settings.rs:556-570`). Siehe "Security Domain" und "Common Pitfalls". |
</phase_requirements>

## Standard Stack

Dies ist eine reine Erweiterung des bestehenden Workspace — **keine neuen
externen Abhängigkeiten**. Alle benötigten Crates sind bereits Teil des
Workspace-`Cargo.lock`.

### Core (bereits im Projekt, keine neuen Versionen nötig)

| Library | Version (verifiziert im Workspace) | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `axum` | wie in `rest/Cargo.toml` (unverändert) | HTTP-Router für den neuen Aggregat-`GET` | Bereits durchgängig für alle Endpoints verwendet. |
| `sqlx` (SQLite, compile-time checked) | wie in `dao_impl_sqlite/Cargo.toml` (unverändert) | Neue DAO-Query für den Drei-Wege-Outer-Join | Projekt-Konvention, `query_as!`-Makro mit `AS "name!: Type"`-Annotationen. |
| `utoipa` | wie in `rest-types/Cargo.toml` (unverändert) | `ToSchema` auf `PersonEntryTO`, `#[utoipa::path]` auf dem neuen Handler | Pflicht laut `CLAUDE.md` §REST API. |
| `dioxus` | wie in `shifty-dioxus/Cargo.toml` (unverändert) | Listen-Komponente, Guard, Store | Bestehender Frontend-Stack. |
| `mockall` (`automock`) | wie in `dao`/`service` (unverändert) | Mock für die neue DAO-/Service-Trait-Methode | Projekt-Testkonvention (siehe `PersonDao`/`PersonService` mit `#[automock]`). |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Eine neue DAO-Methode mit einem SQL-Statement | Service-seitiges Zusammenführen dreier bestehender DAO-Calls (`SalesPersonDao::all`, `PermissionDao::all_users`, plus Matching in Rust) | Vermeidet neues SQL, verletzt aber die "ein Aggregat-Request"-Auflage nicht direkt (das betrifft nur den REST-Request), erzeugt aber O(N+M) Rust-seitiges Matching statt einem DB-Join und verliert die SQL-native Soft-Delete-Semantik aus D-62-05. Nicht empfohlen — die Join-Form existiert in `dao_impl_sqlite/src/person.rs` bereits als Vorbild und ist die günstigere, konsistentere Lösung. |
| Rollen-Spalte per Bulk-DAO-Query | Rollen-Spalte weglassen (bestehende `roles_for_user`-Präzedenz nur pro Zeile nutzbar → verstößt gegen "kein Per-Zeile-Nachladen") | Weglassen ist die risikoärmere Default-Wahl für diese Phase (Claude's Discretion erlaubt es explizit); eine Bulk-Query ist machbar, aber ein zusätzliches, in Phase 61 nicht vorhandenes DAO-Konstrukt. |

**Installation:** Keine — keine neuen Cargo-/npm-Pakete.

## Package Legitimacy Audit

**Nicht anwendbar.** Diese Phase führt keine neuen externen Pakete ein (weder
`cargo` noch `npm`/Dioxus-Assets). Alle verwendeten Crates sind bereits im
Workspace vorhanden und unverändert in der Version. Kein Legitimacy-Check
nötig.

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────┐
│ Browser: /user_management/  │
│  UserManagementPage          │
│   1. Admin-Guard (FE)        │
│      has_privilege("admin")  │
│   2. use_effect →             │
│      LoadDirectory action     │
└──────────────┬───────────────┘
               │ GET /person-directory  (Dioxus.toml-Proxy → :3000)
               ▼
┌─────────────────────────────────────────────┐
│ REST: rest/src/person_directory.rs            │
│  get_person_directory()                        │
│   1. Extension(Context) aus Middleware          │
│   2. rest_state.person_directory_service()      │
│        .list(context.into(), None)              │
│   3. error_handler(...) → 200 Json<Vec<TO>>     │
└──────────────────┬────────────────────────────┘
                    │
                    ▼
┌───────────────────────────────────────────────────────┐
│ Business-Logic: service_impl/src/person_directory.rs    │
│  PersonDirectoryServiceImpl::list()                      │
│   1. check_permission("admin", context)  ← Backend-Gate  │
│   2. tx = transaction_dao.use_transaction(tx)             │
│   3. entities = person_dao.list_directory_entries(tx)     │
│         (neue DAO-Methode, EIN SQL-Statement)              │
│   4. Rust: kind-Klassifikation + display_name-Ableitung    │
│         je Zeile (sales_person.name ?? user.name)           │
│   5. transaction_dao.commit(tx)                              │
│   6. Vec<PersonEntry> zurück                                  │
└──────────────────────┬────────────────────────────────────────┘
                        │
                        ▼
┌───────────────────────────────────────────────────────────┐
│ DAO: dao_impl_sqlite/src/person.rs                            │
│  PersonDaoImpl::list_directory_entries()                        │
│   SELECT sp.id, sp.name, sp.background_color, sp.is_paid,        │
│          sp.inactive, u.name AS username                          │
│   FROM person p                                                    │
│   LEFT JOIN sales_person sp                                         │
│          ON sp.person_id = p.id AND sp.deleted IS NULL               │
│   LEFT JOIN user u ON u.person_id = p.id                              │
│   -- p selbst nie NULL (jede sp/u-Zeile hat eine person_id) --          │
└─────────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

Kein neues Verzeichnis nötig — die Phase erweitert bestehende Module:

```
dao/src/person.rs                    # + list_directory_entries() Trait-Methode
dao_impl_sqlite/src/person.rs        # + Impl + Row-Struct + WR-02-Fix
service/src/person_directory.rs      # NEU: PersonDirectoryService-Trait + PersonEntry
service_impl/src/person_directory.rs # NEU: Business-Logic-Impl, admin-gated
service_impl/src/lib.rs              # + pub mod person_directory;
service/src/lib.rs                   # + pub mod person_directory;
rest-types/src/lib.rs                # + PersonEntryTO + PersonKindTO (oder inline kind)
rest/src/person_directory.rs         # NEU: generate_route() + Handler + ApiDoc
rest/src/lib.rs                      # + mod person_directory; + .nest(...) + ApiDoc-Eintrag
shifty_bin/src/main.rs               # + PersonDirectoryServiceDependencies + Konstruktion
                                      #   NACH person_service (Basic-Block-Ende)
shifty-dioxus/src/page/user_management.rs   # Tab-Leiste raus, eine Liste
shifty-dioxus/src/service/user_management.rs # neuer Loader statt 3 N+1-Loader
shifty-dioxus/Dioxus.toml            # + [[web.proxy]] backend = ".../person-directory"
shifty-dioxus/src/i18n/{mod,en,de,cs}.rs # neue Keys für Ausprägungs-Badge
docs/features/F01-employee-management.md (+_de)  # WR-02-Korrektur + neuer Endpoint
docs/architecture/02-service-tiers.md (+ keine _de-Datei — EN-only laut Repo-Konvention geprüft: tatsächlich existiert 02-service-tiers_de.md, siehe Docs Pairing unten)
```

### Pattern 1: Business-Logic-Aggregat-Service (Muster `VoluntaryStatsService`)

**What:** Ein reiner Lese-Service, der Basic-Services/DAOs kombiniert und
`Authentication::Full` intern nicht braucht (er bekommt den echten Context
durchgereicht und prüft selbst `admin`).
**When to use:** Immer wenn eine REST-Response Daten aus mehr als einem
Basic-Aggregat zusammenführt.
**Example (Gate-Reihenfolge, analog `PersonServiceImpl::create`):**
```rust
// Source: service_impl/src/person.rs:51-60 (Muster, nicht Ziel-Code)
async fn list(
    &self,
    context: Authentication<Self::Context>,
    tx: Option<Self::Transaction>,
) -> Result<Arc<[PersonEntry]>, ServiceError> {
    self.permission_service
        .check_permission("admin", context)   // D-62-08: admin, nicht hr
        .await?;

    let tx = self.transaction_dao.use_transaction(tx).await?;
    let entities = self.person_dao.list_directory_entries(tx.clone()).await?;
    self.transaction_dao.commit(tx).await?;

    Ok(entities.iter().map(PersonEntry::from).collect())
}
```

### Pattern 2: `display_name`-Ableitung im Service (nicht SQL, nicht FE)

**What:** `sales_person.name` gewinnt, sonst `user.name` — die Ableitung
passiert einmal, im `From`-Impl bzw. im Service, nicht per SQL-`COALESCE` (SQL
wäre für Tests schwerer nachvollziehbar) und nicht im Frontend (verboten,
Success Criterion 3).
**When to use:** Bei der Umwandlung der DAO-Row in das Transport-Objekt.
**Example:**
```rust
// Neu zu schreiben, Muster analog SalesPersonAbsenceTO::from
// (rest-types/src/lib.rs:1070-1078)
impl From<&PersonDirectoryEntity> for PersonEntry {
    fn from(row: &PersonDirectoryEntity) -> Self {
        let kind = match (&row.sales_person, &row.username) {
            (Some(_), Some(_)) => PersonKind::Linked,
            (Some(_), None) => PersonKind::SalesPersonOnly,
            (None, Some(_)) => PersonKind::UserOnly,
            (None, None) => unreachable!("D-61-01: jede Person hat mindestens einen Traeger"),
        };
        let display_name: Arc<str> = row
            .sales_person
            .as_ref()
            .map(|sp| sp.name.clone())
            .unwrap_or_else(|| row.username.clone().unwrap_or_default());
        // username-Feld: bei reinem Benutzerkonto = display_name (D-62-03)
        ...
    }
}
```

### Pattern 3: REST-Handler mit `error_handler` (Muster `get_all_users`)

**What:** Zentrales Error-Mapping statt Handler-lokalem Status-Mapping.
**Example:**
```rust
// Source: rest/src/permission.rs:313-330 (Muster get_all_users)
pub async fn get_person_directory<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(
        (async {
            let entries = rest_state
                .person_directory_service()
                .list(context.into(), None)
                .await?;
            Ok(Json(
                entries.iter().map(PersonEntryTO::from).collect::<Vec<_>>(),
            ))
        })
        .await,
    )
}
```

### Pattern 4: Component-Level Admin-Guard (Muster `page/settings.rs:556-570`)

**What:** Frühes `return` mit "Not authorized." — Defense-in-Depth zur
Nav-Ausblendung.
**Example:**
```rust
// Source: shifty-dioxus/src/page/settings.rs:556-570 (Muster, wörtlich übernehmbar)
let is_admin = AUTH
    .read()
    .auth_info
    .as_ref()
    .map(|a| a.has_privilege("admin"))
    .unwrap_or(false);
if !is_admin {
    return rsx! {
        TopBar {}
        div { class: "p-md text-ink-muted", "Not authorized." }
    };
}
```
**Wichtig:** `UserManagementPage` hat heute **keinen** Component-Guard (der
Nav-Eintrag `component/top_bar.rs:55` blendet zwar korrekt auf `has("admin")`
aus, aber die Seite selbst prüft nichts). Dieser Guard muss in Phase 62 neu
hinzugefügt werden — es ist kein Refactoring eines bestehenden Checks.

### Anti-Patterns to Avoid

- **Ableitung des `kind` im Frontend aus zwei `Option`-Feldern:** verboten
  durch D-62-01 — würde exakt die Fallback-Regel wiederherstellen, die
  Success Criterion 3 grep-verifiziert ausschließt.
- **`WHERE`-Soft-Delete-Filter statt `ON`-Klausel im LEFT JOIN:** degeneriert
  den LEFT JOIN zum INNER JOIN und verschluckt genau die Zeilen, die er zeigen
  soll (dieselbe Falle wie WR-02, siehe `61-REVIEW.md:327-339`).
- **Rollen-Spalte per Loop über `roles_for_user` pro Zeile:** exakt das
  N+1-Muster, das diese Phase abbauen soll (`user_management.rs:210-227`) —
  wenn Rollen ins Aggregat sollen, braucht es eine Bulk-Query.
- **`SalesPersonTO`/`UserTO` in den neuen Endpoint stecken:** explizit
  verboten (Präzedenz `SalesPersonAbsenceTO`); ein Entity-TO trägt Felder, die
  das Aggregat nicht braucht, und koppelt den neuen Endpoint an fremde
  Evolutionspfade.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Drei-Wege-Outer-Join mit Soft-Delete | Eigene Rust-seitige Merge-Logik über drei separate Queries | Ein SQL-`LEFT JOIN`-Statement mit dem Soft-Delete-Filter in der `ON`-Klausel | SQL-NULL-Semantik ist fail-closed by construction (dieselbe Begründung wie D-61-08 im Phase-61-Code); Rust-seitiges Matching über `Option<Uuid>` ist die Fehlerklasse, die Phase 61 gerade abgeschafft hat. |
| Admin-Gate | Eigene Rollenprüfung im Handler | `PermissionService::check_permission("admin", context)` | Bestehender, getesteter Mechanismus; jede andere Implementierung riskiert eine zweite, inkonsistente Gate-Semantik. |
| Suche über zwei Namensfelder | Neue Suchfunktion | `matches_search` (`page/user_management.rs:32`), auf beide Felder angewendet (OR-verknüpft) | Bereits getestete, pure Funktion — Wiederverwendung ist explizit in CONTEXT.md vorgesehen. |
| Ausprägungs-Badge/Pill-Darstellung | Neue Badge-Komponente | `PersonChip` + bestehende Badge-Klassen (`page/user_management.rs:310-319`) | Fertige Darstellung für Paid/Volunteer/Inactive, erweiterbar um eine vierte Variante. |

**Key insight:** Diese Phase hat fast keinen echten "neuen" Algorithmus — der
komplette Wert liegt in der korrekten Wiederverwendung/Zusammenführung
bestehender, bereits verifizierter Bausteine aus Phase 61 und dem
Zwei-Tab-Screen. Jede Stelle, an der ein Plan etwas "neu berechnet" statt
"kombiniert", ist ein Warnsignal.

## Runtime State Inventory

**Nicht anwendbar.** Diese Phase ist kein Rename-/Refactor-/Migrations-Phase —
sie fügt einen neuen Lese-Pfad hinzu und entfernt eine Tab-UI, ändert aber
keine Datenbank-Schemas, keine externen Service-Konfigurationen, keine
Secrets/Env-Var-Namen und keine OS-registrierten Zustände. Explizit geprüft:

| Kategorie | Ergebnis |
|-----------|----------|
| Gespeicherte Daten | Keine — keine Migration in dieser Phase (gesperrt in CONTEXT.md). |
| Live-Service-Konfiguration | Keine externen Services betroffen. |
| OS-registrierter Zustand | Keiner. |
| Secrets/Env-Vars | Keine. |
| Build-Artefakte | Keine — reiner Code-Zuwachs, kein Rename bestehender Pfade/Bezeichner mit externer Bindung. |

## Common Pitfalls

### Pitfall 1: LEFT JOIN degeneriert durch `WHERE`-Filter (WR-02-Klasse)

**What goes wrong:** Ein Soft-Delete- oder Existenz-Filter landet in der
`WHERE`-Klausel statt in der `ON`-Klausel eines `LEFT JOIN` — dadurch werden
NULL-Zeilen der rechten Seite herausgefiltert und der LEFT JOIN verhält sich
wie ein INNER JOIN.
**Why it happens:** `WHERE x IS NULL` sieht auf den ersten Blick nach dem
richtigen Anti-Join-Filter aus, aber wenn er sich auf eine **andere** Spalte
bezieht als die Join-Bedingung selbst (hier: `sp.deleted` statt `sp.id`),
kollabiert die Outer-Join-Semantik.
**How to avoid:** Jeder Soft-Delete-Filter auf der optionalen Seite eines
`LEFT JOIN` gehört in die `ON`-Klausel. Exakt der Fix aus `61-REVIEW.md:330-337`.
**Warning signs:** Ein Test, der eine Zeile mit soft-gelöschtem Join-Partner
erwartet, liefert plötzlich 0 statt 1 Ergebnis.

### Pitfall 2: Dev-Proxy vergessen (dreimal in diesem Repo passiert)

**What goes wrong:** Der neue Routen-Präfix funktioniert lokal über
`cargo test`/Unit-Tests, aber `dx serve` liefert 404/405, weil
`shifty-dioxus/Dioxus.toml` keinen `[[web.proxy]]`-Eintrag für den neuen
Präfix hat.
**Why it happens:** Kein Unit-Test kann eine fehlende Proxy-Config
detektieren — sie lebt außerhalb des Rust-Typsystems.
**How to avoid:** Der `Dioxus.toml`-Eintrag muss im **selben Commit** wie der
neue Endpoint stehen; Verifikation per `dx serve`-Smoke (Browser-Aufruf oder
`curl` gegen `localhost:8080/<prefix>`), nicht nur per grünem Test.
**Warning signs:** Frontend lädt die Liste nicht, Browser-Konsole zeigt 404
auf den neuen Pfad, obwohl `cargo test` grün ist. Historische Fälle in diesem
Repo: v1.6 `/toggle`, v1.8 `/vacation-entitlement-offset`, v1.10
`/special-days`.

### Pitfall 3: FE-Fallback-Regel für den Anzeigenamen schleicht sich ein

**What goes wrong:** Ein Reviewer/Executor fügt im Frontend eine Zeile wie
`sp.name.unwrap_or(username)` ein, "nur zur Sicherheit" — und verletzt damit
Fat-Backend/Success Criterion 3, selbst wenn das Ergebnis optisch identisch
aussieht.
**Why it happens:** Es ist der naheliegendste Rust-Reflex, wenn ein Feld
`Option<ImStr>` ist.
**How to avoid:** `display_name` im TO ist **nicht** `Option` — es ist immer
gefüllt (Backend garantiert das). Das Frontend rendert es stumpf. Ein Grep
nach `unwrap_or`/`.or(` im Umfeld der Listen-Zeile ist das Pflicht-Artefakt
aus CONTEXT.md.
**Warning signs:** `grep -n "unwrap_or\|\.or(" shifty-dioxus/src/page/user_management.rs`
findet einen Treffer im Kontext der Namensdarstellung.

### Pitfall 4: Admin- vs. HR-Gate verwechselt

**What goes wrong:** Der neue Endpoint übernimmt versehentlich `HR_PRIVILEGE`
von `PersonService` (das in Phase 61 bewusst HR-gated wurde, siehe
`service_impl/src/person.rs:10-15`), statt `"admin"` zu prüfen.
**Why it happens:** `PersonService`, das die Kandidaten-Reads liefert, ist
HR-gated — ein naheliegender Copy-Paste-Fehler, aber die Roadmap verlangt für
den Aggregat-Endpoint explizit `admin` (D-62-08, PADM-08, bewusste
Verschärfung gegenüber Phase 61).
**How to avoid:** Zwei getrennte Tests: HR-ohne-Admin bekommt 403 vom neuen
Endpoint, Admin bekommt 200 — analog zum Muster in
`shifty_bin/src/integration_test/voluntary_stats.rs:150-160`.
**Warning signs:** Ein HR-Nutzer ohne Admin-Rolle sieht die Liste trotzdem.

### Pitfall 5: `person`-Zeile ohne Träger wird stillschweigend gerendert

**What goes wrong:** Der neue LEFT-JOIN-Code liefert theoretisch eine
Person-Zeile ganz ohne `sales_person`- und `user`-Join-Partner (beide NULL) —
wenn das passiert, crasht entweder der `unreachable!()` in Pattern 2 oder die
Zeile wird mit leerem Namen gerendert.
**Why it happens:** D-61-05 garantiert, dass das nach jeder Operation nicht
mehr vorkommen kann — aber die Garantie lebt im Backend-Code von Phase 61, ein
neuer Aggregat-Code darf sich nicht blind darauf verlassen, ohne den Fall
definiert zu behandeln.
**How to avoid:** Entweder `unreachable!()` mit Kommentar auf D-61-01/D-61-05
(so bewusst dokumentiert, dass ein Panic hier ein Datenintegritäts-Alarm ist,
kein stiller Fehler) oder ein `WHERE`-Filter, der solche Zeilen aus dem
Query-Ergebnis von vornherein ausschließt (`sp.id IS NOT NULL OR u.name IS
NOT NULL`). Letzteres ist robuster gegen zukünftige Regressionen von D-61-05.
**Warning signs:** Ein leerer Listeneintrag ohne Namen und ohne Badge im
UI-Smoke-Test.

### "Looks Done But Isn't" — Fat-Backend-Verstöße, die Tests nicht fangen

Ein Unit-Test, der nur `display_name` auf den erwarteten String prüft, fängt
nicht, dass die Berechnung im Frontend statt im Backend passiert (wenn das FE
zufällig dieselbe Logik dupliziert). Das Pflicht-Artefakt ist der **Grep-Nachweis
der Abwesenheit** von Fallback-Code im Frontend, nicht nur ein grüner
Render-Test.

## Code Examples

### Neue DAO-Trait-Methode (Vorschlag, Feldnamen sind Discretion)

```rust
// Muster: dao/src/person.rs — Erweiterung des bestehenden PersonDao-Traits
// (oder eigenes PersonDirectoryDao, falls der Business-Logic-Service kein
// Basic-DAO direkt konsumieren soll — beides ist mit den Tier-Regeln
// vereinbar, solange PersonDao selbst Basic-Tier bleibt).
async fn list_directory_entries(
    &self,
    tx: Self::Transaction,
) -> Result<Arc<[PersonDirectoryRow]>, DaoError>;
```

### SQL-Muster für den Aggregat-Join (Erweiterung des bestehenden Musters)

```sql
-- Muster: dao_impl_sqlite/src/person.rs:154-171 (sales_persons_without_user),
-- hier aber OHNE den WHERE-Ausschluss und mit BEIDEN Seiten als LEFT JOIN von `person`.
SELECT
    p.id           AS "person_id!: Vec<u8>",
    sp.id          AS "sales_person_id?: Vec<u8>",
    sp.name        AS "sales_person_name?: String",
    sp.background_color AS "background_color?: String",
    sp.is_paid     AS "is_paid?: bool",
    sp.inactive    AS "inactive?: bool",
    u.name         AS "username?: String"
FROM person p
LEFT JOIN sales_person sp ON sp.person_id = p.id AND sp.deleted IS NULL
LEFT JOIN user u ON u.person_id = p.id
WHERE sp.id IS NOT NULL OR u.name IS NOT NULL  -- Pitfall 5: Ghost-Person ausschliessen
```

### WR-02-Fix (D-62-05, wörtlich aus `61-REVIEW.md:330-337`)

```sql
-- dao_impl_sqlite/src/person.rs, users_without_sales_person — Fix:
SELECT u.name AS "name!: String"
FROM user u
LEFT JOIN sales_person sp
       ON sp.person_id = u.person_id
      AND sp.deleted IS NULL
WHERE sp.id IS NULL
```
Plus Test `user_mit_nur_soft_geloeschter_sales_person_ist_kandidat` in
`service_impl/src/test/person_candidates.rs` (bestehende Datei, K-3 testet
heute nur die Gegenrichtung laut Review).

### `PersonEntryTO` — Formvorschlag (Feldnamen/Serde-Form sind Discretion)

```rust
// rest-types/src/lib.rs — Präzedenz SalesPersonAbsenceTO (Zeile 1058-1068)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PersonKindTO {
    Linked { sales_person_id: Uuid, username: Arc<str> },
    SalesPersonOnly { sales_person_id: Uuid },
    UserOnly { username: Arc<str> },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct PersonEntryTO {
    pub person_id: Uuid,
    pub display_name: Arc<str>,   // D-62-03: NIE Option — Backend leitet immer ab
    pub username: Arc<str>,        // D-62-03: bei reinem SP-Eintrag leer/Backend-Konvention
    pub kind: PersonKindTO,        // D-62-01
    pub background_color: Option<Arc<str>>,
    pub is_paid: Option<bool>,
    pub inactive: Option<bool>,
    pub roles: Option<Vec<Arc<str>>>, // Discretion: nur falls Bulk-Query gebaut wird
}
```
Hinweis: ein internes `#[serde(tag = "kind", ...)]` Enum wie oben liefert
D-62-01 direkt und macht "Option-Ableitung im Frontend" strukturell
unmöglich, weil die Nutzdaten pro Variante liegen, nicht als lose `Option`s
im äußeren Struct. Ob das TO stattdessen ein flaches Struct mit separatem
`kind: PersonKindTO`-Diskriminator (ohne Payload-Verschachtelung) plus lose
`Option`-Felder wird, ist Serde-Form-Discretion — beide erfüllen D-62-01,
solange das Frontend nirgends selbst zwischen "verknüpft" und "unverknüpft"
unterscheiden muss, weil es das Enum bereits tut.

### Admin-Gate-Test-Muster (REST-Layer, `tower::oneshot`)

```rust
// Muster: shifty_bin/src/integration_test/voluntary_stats.rs:52-160
let test_setup = TestSetup::new().await;
let router = /* echtes RestStateImpl-Router */;
let req = Request::builder()
    .uri("/person-directory")
    .header("...auth-header-fuer-hr-ohne-admin...")
    .body(Body::empty())
    .unwrap();
let resp = router.clone().oneshot(req).await.unwrap();
assert_eq!(resp.status(), StatusCode::FORBIDDEN, "HR ohne Admin darf nicht rein");
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|---------------|--------|
| `sales_person_user`-Verknüpfungstabelle als Datenquelle | `person`-Identität mit `person_id`-FK auf `sales_person`/`user` | Phase 61 (2026-09-17) | Diese Phase baut direkt auf der neuen `person`-Identität; kein Code darf mehr `sales_person_user` referenzieren (bereits grep-verifiziert leer). |
| Zwei separate Tabs mit eigenen Ladepfaden (`LoadAllUsers`/`LoadAllSalesPersons`) | Ein Aggregat-Request | Diese Phase | 2+N+2M Requests → 1 Request; Nebeneffekt, kein Performance-Versprechen. |
| `HR_PRIVILEGE` für Personen-bezogene Lesepfade (Phase 61, bewusst als Übergangs-Gate) | `admin`-Privileg für den vollständigen Aggregat-Endpoint | Diese Phase (D-62-08) | HR-Nutzer ohne Admin-Rolle verlieren Zugang zur Sales-Person-Verwaltung — bewusste, dokumentierte Verhaltensänderung (REQUIREMENTS.md §„Bewusste Änderung"). |

**Deprecated/outdated:** `UserManagementTabBar`/`UserMgmtTab`
(`shifty-dioxus/src/component/user_management_tab_bar.rs`) wird durch diese
Phase funktional überflüssig — ob die Datei gelöscht wird, ist Discretion
(prüfen, ob andere Aufrufer existieren; laut Grep in diesem Research nur
`page/user_management.rs` als Konsument gefunden).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Ein neuer Routen-Präfix wie `/person-directory` kollidiert nicht mit einer bestehenden Route (grep über `rest/src/lib.rs`-`.nest(...)`-Liste ergab keinen Treffer für `person`) | Architecture Patterns → Recommended Project Structure | Gering — reine Namensgebung, Discretion erlaubt jede Kollisionsfreie Wahl; im schlimmsten Fall Compile-Error beim `.nest()`, sofort sichtbar. |
| A2 | Die vorgeschlagene Query-Form (zwei `LEFT JOIN`s von `person` statt Join von `sales_person`↔`user`) ist syntaktisch mit SQLx `query_as!` compile-time-checkbar in der lokalen Dev-DB — nicht in dieser Session gegen die echte DB ausgeführt | Code Examples → SQL-Muster | Mittel — falls das Schema abweicht (z. B. fehlender Index), muss der Planner/Executor die Query gegen die migrierte lokale DB kompilieren und ggf. anpassen; die Spaltennamen selbst sind aus der Migration verifiziert. |
| A3 | `roles` als optionales Feld im `PersonEntryTO` ist die richtige Modellierung, falls die Rollen-Spalte mitkommt — keine Bulk-DAO-Methode existiert heute, ihre exakte SQL-Form wurde nicht entworfen | Standard Stack → Alternatives Considered | Gering — Discretion erlaubt auch, Rollen ganz wegzulassen; falls doch gewünscht, ist die Bulk-Query (`SELECT user_name, role_name FROM user_role`) trivial nachzubauen, aber nicht in dieser Session verifiziert. |

**Kein Claim in diesem Dokument ist rein `[ASSUMED]` ohne Codebase-Beleg** —
alle strukturellen Aussagen (Join-Spalten, Tabellennamen, Gate-Konstanten,
Dateipfade) sind per `Read`/`grep` gegen den tatsächlichen Code verifiziert
(`[VERIFIED: codebase]`). Die drei Einträge oben markieren Design-Vorschläge,
die der Planner noch gegen die migrierte lokale DB bzw. gegen die
Discretion-Entscheidung prüfen muss.

## Open Questions

1. **Wo genau lebt die neue DAO-Methode — `PersonDao` erweitert oder eigenes
   `PersonDirectoryDao`?**
   - What we know: `dao/src/person.rs:18-19` sagt explizit, das Trait bleibe
     "bewusst auf genau vier Methoden beschränkt" und "Vorratshaltung für
     Phase 62/63 ist ausdrücklich unerwünscht" — das ist ein Signal **gegen**
     eine fünfte Methode auf `PersonDao`.
   - What's unclear: Ob "Vorratshaltung" sich nur auf spekulative,
     ungenutzte Methoden bezieht (die jetzt gebrauchte Methode wäre keine
     Vorratshaltung) oder ob Phase 61 bewusst ein neues DAO für Phase 62
     erwartet.
   - Recommendation: Ein neues, schlankes `PersonDirectoryDao`-Trait (oder
     -Methode in einer neuen Datei `dao/src/person_directory.rs`) ist die
     sicherere Lesart des Phase-61-Kommentars und hält `PersonDao` als reinen
     Identitäts-Basic-DAO unverändert. Der Business-Logic-Service konsumiert
     dann `PersonDirectoryDao` statt `PersonDao` direkt (oder beide, falls
     `unlinked_*`-Reads aus Phase 61 zusätzlich gebraucht werden — aktuell
     nicht ersichtlich, dass Phase 62 sie braucht).

2. **Soll `PersonEntryTO.roles` überhaupt existieren?**
   - What we know: CONTEXT.md nennt Rollen als Teil der "heutigen Information
     aus beiden Tabs", die erhalten bleiben soll; gleichzeitig ist "ob die
     Rollen im Aggregat mitkommen" explizit Discretion mit der Auflage "kein
     Per-Zeile-Nachladen".
   - What's unclear: Ob die zusätzliche Bulk-DAO-Query (neu, ungetestet) den
     Aufwand für Phase 62 wert ist, oder ob die Rollen-Spalte für reine
     Benutzerkonten separat nachgeladen werden darf (dann aber max. 1
     zusätzlicher Request für die ganze Seite, nicht pro Zeile — auch das
     wäre technisch "kein Per-Zeile-Nachladen").
   - Recommendation: Planner sollte explizit entscheiden und im Plan-Header
     dokumentieren, ob Rollen im Aggregat mitkommen (dann: neue Bulk-DAO-
     Methode `PermissionDao::all_user_roles() -> HashMap<user, Vec<role>>`
     nötig) oder ob die Rollen-Spalte für Phase 62 komplett entfällt (dann:
     D-62-06 "Rollen-Spalte wandert in die vereinte Liste" muss präzisiert
     werden — evtl. reicht ein einmaliger Zusatz-Request statt N).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo`/`clippy`/`sqlx` (via `nix develop`) | Backend-Build, Clippy-Hard-Gate, DAO-Kompilierung | ✓ (nur über `nix develop --command bash -c '<cmd>'`, `SQLX_OFFLINE=true`) | siehe `flake.nix` | Kein direkter PATH-Zugriff — jeder Backend-Build/Test MUSS über `nix develop` laufen. |
| `node` (für `dx`/Tailwind/gsd-tools) | Frontend-Build, `dx serve`-Smoke | ✓ (nur über expliziten `PATH`-Export auf den Nix-Store-Pfad) | 24.16.0 (Store-Pfad kann bei Updates wandern) | `ls /nix/store/*/bin/node` vor Nutzung verifizieren. |
| `rustfmt` | CI-Format-Gate (`cargo fmt --all -- --check`) | ✗ (fehlt im Nix-DevShell) | — | Kein lokaler Formatierungs-Check möglich; **nicht** repo-weit reformatieren (bestehende Drift in ~135 Dateien laut `61-VERIFICATION.md`). |
| `dx serve` | Success Criterion 5 — Proxy-Smoke-Nachweis | ✓ (Teil des `shifty-dioxus`-Toolchains, gestartet via `dx serve --hot-reload`) | siehe `Dioxus.toml`/`Cargo.toml` | Kein Fallback — Pflicht-Artefakt laut Roadmap-Constraint 6, nicht durch Unit-Test ersetzbar. |

**Missing dependencies with no fallback:** keine (alle o.g. Tools sind über
die dokumentierten Umwege erreichbar).

**Missing dependencies with fallback:** `rustfmt` — Fallback ist "nicht
reformatieren", kein Ersatzwerkzeug.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (Rust `#[test]`/`#[tokio::test]`), kein separates Test-Runner-Config-File |
| Config file | keins — Workspace-`Cargo.toml` reicht |
| Quick run command | `nix develop --command bash -c 'SQLX_OFFLINE=true cargo test -p service_impl --lib test::person_directory -- --exact'` (Platzhaltername, je nach gewähltem Testmodul-Pfad) |
| Full suite command | `nix develop --command bash -c 'SQLX_OFFLINE=true cargo test --workspace'` + `nix develop --command bash -c 'SQLX_OFFLINE=true cargo clippy --workspace -- -D warnings'` (Clippy ist Hard-Gate, läuft NICHT automatisch mit `cargo test`) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PADM-01 | Aggregat liefert verknüpfte + reine Benutzer + reine Verkäufer nebeneinander, jeder mit `kind` | unit (service_impl) | `cargo test -p service_impl --lib test::person_directory::liste_enthaelt_alle_drei_auspraegungen -- --exact` | ❌ Wave 0 |
| PADM-01 | FE-Suche findet über `display_name` UND `username` | unit (shifty-dioxus SSR-Rendertest) | `cargo test -p shifty-dioxus --lib page::user_management::tests::directory_search_matches_both_names -- --exact` | ❌ Wave 0 |
| PADM-06 | `display_name` ist bei reinem Benutzerkonto gefüllt (kein leerer String) | unit (service_impl) | `cargo test -p service_impl --lib test::person_directory::display_name_reines_benutzerkonto_ist_username -- --exact` | ❌ Wave 0 |
| PADM-06 | Kein FE-Fallback-Code für den Namen (grep-Artefakt, kein Test im engeren Sinn) | Grep-Nachweis | `grep -n "unwrap_or\|\.or(" shifty-dioxus/src/page/user_management.rs` (erwartet: kein Treffer im Namenskontext) | n/a — Pflicht-Artefakt statt Test |
| PADM-08 | Backend: HR ohne Admin bekommt 403 vom neuen Endpoint | integration (`tower::oneshot`) | `cargo test -p shifty_bin --bin shifty_bin person_directory::hr_ohne_admin_bekommt_403 -- --exact` | ❌ Wave 0 |
| PADM-08 | Backend: Admin bekommt 200 mit Daten | integration (`tower::oneshot`) | `cargo test -p shifty_bin --bin shifty_bin person_directory::admin_bekommt_200 -- --exact` | ❌ Wave 0 |
| PADM-08 | Frontend: Component-Guard blockiert Nicht-Admin bei direktem Routenaufruf | unit (SSR-Rendertest) | `cargo test -p shifty-dioxus --lib page::user_management::tests::non_admin_sees_not_authorized -- --exact` | ❌ Wave 0 |
| Success Criterion 5 | Genau ein Request füllt die Liste (kein N+1) | unit (Store-Test, Anzahl `loader::`-Aufrufe im Action-Handler) + manueller `dx serve`-Smoke | `cargo test -p shifty-dioxus --lib service::user_management::tests::load_directory_ist_ein_request -- --exact` + manuelles `dx serve` | ❌ Wave 0 |
| D-62-05/WR-02 | Benutzer mit nur soft-gelöschter Sales Person ist weiterhin Kandidat | unit (service_impl, bestehende Datei) | `cargo test -p service_impl --lib test::person_candidates::user_mit_nur_soft_geloeschter_sales_person_ist_kandidat -- --exact` | ❌ Wave 0 (Datei existiert, Test fehlt) |

### Sampling Rate

- **Per task commit:** jeweiliger Quick-run-Befehl für das geänderte Modul
  (`service_impl`, `shifty_bin`, oder `shifty-dioxus`), plus
  `cargo clippy --workspace -- -D warnings` (Hard-Gate, läuft separat von
  `cargo test`).
- **Per wave merge:** volle `cargo test --workspace` + `cargo clippy
  --workspace -- -D warnings`.
- **Phase gate:** volle Suite grün + `dx serve`-Smoke gegen den neuen
  Routen-Präfix, bevor `/gsd-verify-work`.

### Wave 0 Gaps

- [ ] `service_impl/src/test/person_directory.rs` (neu) — deckt PADM-01,
      PADM-06.
- [ ] `service_impl/src/test/person_candidates.rs` — Ergänzung um
      `user_mit_nur_soft_geloeschter_sales_person_ist_kandidat` (D-62-05).
- [ ] `shifty_bin/src/integration_test/person_directory.rs` (neu) — deckt
      PADM-08 auf REST-Ebene (`tower::oneshot`, Muster
      `voluntary_stats.rs`), plus `mod person_directory;`-Eintrag in
      `shifty_bin/src/integration_test.rs`.
- [ ] `shifty-dioxus/src/page/user_management.rs` — Tests innerhalb des
      bestehenden `#[cfg(test)] mod tests` erweitern (SSR-Render-Muster
      bereits vorhanden, `render_with`/`render_users_with` werden zu einem
      gemeinsamen Renderer für die neue Einzelliste).
- [ ] Framework-Install: keins nötig — `cargo test`/`dioxus_ssr` sind bereits
      Workspace-Abhängigkeiten.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | nein (unverändert — Session/OIDC/mock_auth bestehen bereits) | — |
| V3 Session Management | nein (unverändert) | — |
| V4 Access Control | **ja** | `PermissionService::check_permission("admin", context)` im neuen Business-Logic-Service (Backend) + Component-Level-Guard (Frontend, Defense-in-Depth). Beide Ebenen MÜSSEN unabhängig getestet werden (PADM-08, D-62-08). |
| V5 Input Validation | ja (minimal) | Der neue Endpoint ist ein reiner `GET` ohne Body/Query-Parameter in dieser Phase (FE-seitige Suche, kein Backend-Suchparameter) — kein Injection-Vektor über Nutzereingabe. |
| V6 Cryptography | nein | Keine neuen Krypto-Operationen. |

### Known Threat Patterns for {stack}

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Broken Access Control — Endpoint gated mit falschem Privileg (`hr` statt `admin`) | Elevation of Privilege | Zwei unabhängige Tests: HR-ohne-Admin → 403, Admin → 200 (siehe Validation Architecture, Pitfall 4). |
| Broken Access Control — FE-Guard vorhanden, Backend-Gate vergessen (oder umgekehrt) | Elevation of Privilege | Success Criterion 4 verlangt explizit **beide** Ebenen getestet — kein "eine reicht"-Abkürzung. |
| Information Disclosure — Aggregat liefert mehr Felder als nötig (z. B. interne IDs, die das FE nicht braucht) | Information Disclosure | `PersonEntryTO` bleibt zweckgebaut (Präzedenz `SalesPersonAbsenceTO`) — keine Wiederverwendung von `UserTO`/`SalesPersonTO`, die potenziell mehr Felder als nötig tragen könnten. |
| SQL Injection über den neuen Join | Tampering | Nicht anwendbar — `sqlx::query_as!` mit Compile-Time-Checking, keine String-Konkatenation, keine Nutzereingabe im SQL (kein Suchparameter im Backend in dieser Phase). |

## Sources

### Primary (HIGH confidence — `[VERIFIED: codebase]`)

- `dao/src/person.rs`, `dao_impl_sqlite/src/person.rs` — bestehende
  `PersonDao`-Trait-Grenzen, Kandidaten-Joins, Row-Layout.
- `service/src/person.rs`, `service_impl/src/person.rs` — `PersonService`
  Basic-Tier-Muster, HR-Gate-Reihenfolge.
- `service_impl/src/permission.rs:110-166` — `admin`-Gate-Präzedenz
  (`get_all_users`, `create_user`, `delete_user`).
- `service/src/permission.rs:9-11` — `HR_PRIVILEGE`/`SALES_PRIVILEGE`-Konstanten
  (kein `ADMIN_PRIVILEGE`-Konstantenpendant — `"admin"` wird überall als
  String-Literal verwendet, bestätigt Konvention).
- `rest/src/permission.rs` — REST-Handler-Muster (`error_handler`,
  `#[utoipa::path]`, Router-Aufbau).
- `rest/src/lib.rs:64-79, 608-745` — Modul-Registrierung, `.nest()`-Liste,
  `ApiDoc`-Zusammenbau.
- `rest-types/src/lib.rs:59-69, 204-218, 1058-1078` — `UserTO`,
  `SalesPersonTO`, `SalesPersonAbsenceTO` als Formvorbilder.
- `shifty_bin/src/main.rs:1237-1250, 1286` — DI-Reihenfolge Basic-Block →
  Business-Logic (`person_service` vor `reporting_service`).
- `shifty-dioxus/src/page/user_management.rs` (komplette Datei) — Tab-Struktur,
  `matches_search`, SSR-Testharness, alle vier Listen-Aktionen.
- `shifty-dioxus/src/service/user_management.rs` (komplette Datei) —
  `UserManagementStore`, die drei N+1-Loader, `UserManagementAction`.
- `shifty-dioxus/src/page/settings.rs:556-570` — Component-Guard-Muster.
- `shifty-dioxus/src/component/top_bar.rs:35-58` — Nav-Visibility inkl.
  `user_management: has("admin")` (bereits korrekt).
- `shifty-dioxus/Dioxus.toml:45-116` — vollständige `[[web.proxy]]`-Liste
  (kein bestehender `person`-Präfix).
- `shifty-dioxus/src/i18n/mod.rs:364-416` — bestehende, wiederverwendbare
  Keys.
- `migrations/sqlite/20260916000000_create-person-backfill.sql:28-40` —
  `person`-Tabellenschema, `person_id`-FK auf `sales_person`/`user`.
- `dao/src/permission.rs`, `dao_impl_sqlite/src/lib.rs:140-303` —
  `PermissionDao` (kein Bulk-Rollen-Query, nur `roles_for_user` pro Nutzer).
- `shifty_bin/src/integration_test/voluntary_stats.rs`,
  `shifty_bin/src/integration_test/admin_auto_grant.rs` — Test-Muster
  für admin-gatete REST-Endpoints (`tower::oneshot`) bzw. DAO-Integrationstests
  (`TestSetup::new()`).
- `.planning/phases/61-.../61-VERIFICATION.md`,
  `.planning/phases/61-.../61-REVIEW.md` (WR-02, Zeile 304-344) — verifizierter
  Phase-61-Stand inkl. offen deferred WR-02.
- `docs/architecture/02-service-tiers.md:1-95` — Tier-Dokumentationsformat,
  bereits enthaltener `PersonService`-Eintrag als Textvorlage für den neuen
  `PersonDirectoryService`-Eintrag.

### Secondary (MEDIUM confidence)

- Keine externen Web-Quellen herangezogen — diese Phase ist vollständig
  innerhalb des bekannten Repo-Musters lösbar, keine neue
  Fremdbibliothek/Framework-Frage, die eine Dokumentations-Recherche
  gerechtfertigt hätte.

### Tertiary (LOW confidence)

- Keine.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — keine neuen Abhängigkeiten, alles im Workspace bereits vorhanden und versionsstabil.
- Architecture: HIGH — Tier-Zuordnung, DI-Reihenfolge und Join-Form sind direkt aus verifiziertem Phase-61-Code abgeleitet, nicht aus Training-Wissen.
- Pitfalls: HIGH — vier von fünf Pitfalls sind aus dokumentierten, im Repo bereits einmal (oder mehrfach) passierten Vorfällen abgeleitet (WR-02-Review, dreifacher Dev-Proxy-Vorfall, D-61-08-Fail-open-Präzedenz).

**Research date:** 2026-09-19
**Valid until:** 2026-10-19 (30 Tage — stabiler interner Stack, kein Fast-Moving-Ökosystem-Risiko)
