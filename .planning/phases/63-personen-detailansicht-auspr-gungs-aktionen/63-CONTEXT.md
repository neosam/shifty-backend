# Phase 63: Personen-Detailansicht + Ausprägungs-Aktionen - Context

**Gathered:** 2026-09-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Diese Phase gibt dem Personen-Verzeichnis aus Phase 62 seine **Detailansicht** und
alle **Ausprägungs-Mutationen**. Eine einzige Route `/person/:person_id/` und eine
einzige Komponente bedienen alle drei Ausprägungen (verknüpft / nur Benutzer /
nur Verkäufer) und blenden Abschnitte nach der **Ausprägung** ein und aus — nicht
nach der Rolle. Die beiden heutigen Detailseiten (`/user_details/:user_id/`,
`/sales_person_details/:sales_person_id/`) werden dabei **abgelöst**, nicht
ergänzt.

Dazu kommen die Mutationen: **ein** Anlege-Dialog mit zuschaltbaren Ausprägungen
(der die beiden heutigen Anlege-Knöpfe ersetzt), „Login hinzufügen" /
„Person hinzufügen" als Aktionen auf dem Eintrag, „Bestehendes verknüpfen" über
einen Kandidaten-Picker, „Verknüpfung lösen" und ausprägungs-spezifisches Löschen
mit aussagekräftiger Bestätigung.

**Backend + Frontend.** Backend: Detail-Aggregat für eine Person (Business-Logic-
Tier, analog `PersonDirectoryService`), ggf. ein Anlege-Endpoint, der Benutzer und
Sales Person in **einer** Transaktion erzeugt und verknüpft, Kandidaten-Endpoints
(`PersonService::unlinked_sales_persons` / `unlinked_users` existieren bereits),
Freischaltung des Verkäufer-Löschpfades nach außen, `rest-types`-TOs, DI in
`shifty_bin/src/main.rs`, `Dioxus.toml`-Proxy im selben Commit. Frontend: neue
Detail-Route + Komponente, Rückbau von `page/user_details.rs` und
`page/sales_person_details.rs`, Umbau der Toolbar in `page/user_management.rs`,
Breadcrumb-Zweige in `component/top_bar.rs`, i18n de/en/cs.

**Nicht in dieser Phase:** das Planungs-Soll (Phase 64), abschnittsweises
Rollen-Gating (PADM-F-02), der HR-Report `/employees/:id` (PADM-F-01), echte
Personenstammdaten an `person` (PERS-F-01), eine UI zum Zusammenführen zweier
Personen.

**Requirements:** PADM-02, PADM-03, PADM-04, PADM-05, PADM-07

</domain>

<decisions>
## Implementation Decisions

Alle acht Entscheidungen sind am 2026-09-20 vom Nutzer als Block bestätigt worden
(„dann klingt die Empfehlung überall gut"). D-63-01 bis D-63-03 beantworten die
drei in der ROADMAP gepinnten Discuss-Points unter derselben Nummer.

### Lebenszyklus von `person` und Bedeutung der Lösch-Aktionen

- **D-63-01 (beantwortet ROADMAP-Discuss-Point D-63-01):** **Die `person`-Zeile
  wird mitgelöscht, sobald sie ihre letzte Ausprägung verliert.** Keine Waisen,
  keine Admin-Sicht auf trägerlose Personen. Sicher, weil `person` nach **D-60-04**
  ausschließlich Identität plus Audit-Spalten trägt und nach dem Wegfall beider
  Träger von nichts mehr referenziert wird — dieselbe Begründung, die D-61-03 schon
  für das Löschen der absorbierten Person beim Merge trägt. Sichtbar wäre eine
  Waise ohnehin nicht: die Liste kommt aus dem Outer-Join (**D-62-02**), nicht aus
  `person`.

  Abgrenzung zu D-61-05: Beim **Lösen** entsteht weiterhin keine trägerlose
  Person (die Sales Person behält ihre, der Benutzer bekommt eine frische). D-63-01
  greift ausschließlich beim **Löschen** des letzten Trägers.
  — **Reversibility:** reversible — Service-Logik im Löschpfad, keine
  Schema-Wirkung.

- **D-63-05:** **„Verknüpfung lösen" und „löschen" sind zwei getrennte, verschieden
  beschriftete Aktionen — und es gibt bewusst KEINEN Sammelknopf „Eintrag
  löschen", der beide Seiten auf einmal nimmt.**

  | Aktion | Wirkung | Charakter |
  |---|---|---|
  | „Verknüpfung lösen" | Aus einem Eintrag werden zwei; beide Objekte bleiben vollständig erhalten (D-61-05) | harmlos, umkehrbar über den Kandidaten-Picker |
  | „Login löschen" | `delete_user` — **Hard Delete** inkl. Rollen und Einladungen | destruktiv, endgültig |
  | „Verkäufer löschen" | `SalesPersonService::delete` — **Soft Delete** (`deleted`-Stempel) | destruktiv, Daten bleiben in der DB |

  Begründung: Der Sammelknopf wäre der gefährlichste Knopf der Anwendung — er
  würde auf einem verknüpften Eintrag einen Hard Delete und einen Soft Delete in
  einer einzigen unbeschrifteten Geste bündeln. PADM-07 („Beim Entfernen einer
  Ausprägung **oder** beim Löschen eines Eintrags") ist durch die drei Aktionen
  oben vollständig erfüllt: ein Eintrag verschwindet, indem seine letzte
  Ausprägung gelöscht wird — und dann greift D-63-01.

- **D-63-06:** **Das Löschen der Verkäufer-Seite wird im Frontend erstmals
  freigeschaltet.** Heute existiert `SalesPersonService::delete` nur im Backend;
  das Frontend hat keinen Aufrufpfad (grep-belegt: nur `delete_user` ist
  verdrahtet). Ohne Freischaltung bliebe PADM-07 für die Verkäufer-Seite eine
  leere Zusage. **Auflage:** Die Bestätigung nennt das bestehende
  `inactive`-Flag ausdrücklich als schonende Alternative („inaktiv setzen statt
  löschen") — das ist der heute etablierte Weg, einen Verkäufer aus der Planung
  zu nehmen, ohne Historie anzufassen.
  — **Reversibility:** reversible — ein Aufrufpfad im Frontend, der Backend-Pfad
  existiert seit Langem.

- **D-63-08:** **Die Lösch-Bestätigung trägt festen, ausprägungs-spezifischen
  Text ohne gezählte Betroffenheit.** Zwei Textbausteine, nach der Seite
  unterschieden, die verschwindet:

  - **Verkäufer-Seite:** Schichten, Stundenkonto und Abrechnungs-Historie bleiben
    in der Datenbank erhalten (Soft Delete), verschwinden aber aus allen Ansichten;
    Hinweis auf „inaktiv setzen" als Alternative (D-63-06).
  - **Benutzer-Seite:** Login, Rollen und offene Einladungen werden endgültig
    gelöscht (Hard Delete).

  Die Alternative — ein Zähl-Endpoint, der vor dem Löschen die echte Betroffenheit
  ermittelt („47 Buchungen, 3 Abrechnungsperioden") — wurde **verworfen**: ehrlicher,
  aber ein eigener Aggregat-Endpoint samt Tests, und damit spürbarer Umfang in
  einer Phase, die ohnehin fünf Requirements trägt. Kandidat für die Deferred-Liste.

### Route, Rückbau und Anlege-Einstieg

- **D-63-04:** **Eine neue Route `/person/:person_id/`; die beiden alten
  Detail-Routen werden ersatzlos entfernt** — keine Weiterleitungs-Hüllen. Der
  Routen-Parameter ist die `person_id`, die `PersonEntryTO` bereits trägt
  (`rest-types/src/lib.rs:2861`), es braucht also keinen neuen Lesepfad für die
  Navigation. Rückzubauen sind: `Route::UserDetails` und
  `Route::SalesPersonDetails` (`shifty-dioxus/src/router.rs:48,50`), die Seiten
  `page/user_details.rs` und `page/sales_person_details.rs`, die Verlinkungen in
  `page/user_management.rs:306,366,375` und die beiden Breadcrumb-Zweige in
  `component/top_bar.rs:94-95,883-889`. Grep-verifiziert gibt es keine weiteren
  Aufrufer. **Nachweis-Auflage:** ein abschließender Grep auf `UserDetails` /
  `SalesPersonDetails` muss leer sein (außer in Archiv-/Planungsdateien).

- **D-63-02 (beantwortet ROADMAP-Discuss-Point D-63-02):** **„Neu anlegen" und
  „Bestehendes verknüpfen" bleiben zwei getrennt beschriftete Aktionen**, kein
  mehrdeutiges Kombi-Feld. Der Kandidaten-Picker (PADM-04) hängt ausschließlich an
  der zweiten Aktion und zeigt nur **unverknüpfte** Kandidaten
  (`PersonService::unlinked_sales_persons` / `unlinked_users`, seit Phase 61
  vorhanden). Verworfen: ein Dialog mit Auswahlliste plus „ist nicht dabei, neu
  anlegen" — er verwischt genau die Unterscheidung, die die FEATURES-Empfehlung
  und der Keycloak-Präzedenzfall trennen wollen.

  Serverseitig bleibt es bei **D-61-04**: bereits verknüpfte Ziele werden mit
  **409** abgelehnt, auch wenn sie im Picker gar nicht erscheinen (Success
  Criterion 4 verlangt beides — Ausblendung *und* serverseitige Ablehnung).

- **D-63-03 (beantwortet ROADMAP-Discuss-Point D-63-03):** **Der neue Anlege-Dialog
  ERSETZT die beiden heutigen Knöpfe**, er ergänzt sie nicht. Entfernt werden
  „Neuen Verkäufer erstellen" (`Key::CreateNewSalesPerson`, Navigation nach
  `Route::SalesPersonDetails` mit leerer ID) und „Neuen Benutzer hinzufügen"
  (`Key::AddNewUser`, öffnet `AddUserDialog`) — beide in
  `page/user_management.rs:296-315`. Hintergrund ist die wörtliche
  UAT-Beobachtung aus Phase 62: „Es gibt jetzt trotzdem neuen benutzer und neuen
  verkaeufer separat. Das checkt niemand." Für diesen Teilaspekt ist der Zustand
  seit dem Tab-Merge schlechter als vorher, weil der Tab-Kontext die Ausprägung
  implizit erklärt hat. **Nachweis-Auflage:** nach dem Umbau darf in der Toolbar
  genau ein Anlege-Einstieg stehen; `Key::CreateNewSalesPerson` verschwindet oder
  wandert in den neuen Dialog.

- **D-63-07:** **Der Anlege-Dialog bleibt schlank.** Er erfasst:
  1. den Anzeigenamen,
  2. zwei Umschalter — „bekommt ein Login" / „bekommt einen Verkäufer-Datensatz",
  3. das Feld Benutzername, sichtbar genau dann, wenn „bekommt ein Login" an ist
     (`user.name` ist Primärschlüssel und lässt sich nicht ableiten).

  Farbe, bezahlt/freiwillig, inaktiv und die Schichtplan-Zuordnung kommen
  **danach** in der Detailansicht, in die der Dialog nach dem Speichern direkt
  weiterleitet. Verworfen: der Dialog erfragt gleich alles — das baut die abgelöste
  Verkäufer-Detailseite im Dialog nach und macht das Anlegen eines reinen
  Benutzerkontos unnötig schwer. Mindestens ein Umschalter muss an sein.

### Gesperrt — in dieser Phase nicht neu aufmachen

- **Sektions-Sichtbarkeit richtet sich nach der Ausprägung, nicht nach der Rolle.**
  Abschnittsweises Rollen-Gating ist **PADM-F-02** (Future). Die Seite als Ganzes
  ist admin-gated (**PADM-08**, **D-62-08**) — Backend-Gate `"admin"` plus
  Component-Level-Guard nach dem Muster `page/settings.rs:556-570`.
- **`CURRENT_SNAPSHOT_SCHEMA_VERSION` bleibt 13.** Keine der vier Bump-Klauseln aus
  `CLAUDE.md` greift: kein neuer persistierter `value_type`, keine geänderte
  Computation, kein geändertes Input-Set. **Nicht „vorsichtshalber" bumpen.**
- **Keine Migration in dieser Phase.** Das Schema steht seit Phase 60/61.
- **`sales_person.id` und `user.name` werden nicht umgeschlüsselt.** Zwölf Tabellen
  hängen an der Sales-Person-UUID, darunter write-once-Abrechnungs-Snapshots;
  `user.name` trägt die Auth-Schicht.
- **Verknüpfen ist ein Merge zweier Personen** (**D-61-03**), 409 nur bei echter
  Mehrdeutigkeit (**D-61-04**), Lösen hinterlässt nie eine trägerlose Person
  (**D-61-05**).
- **Fat Backend, Thin Client.** Ausprägung und Anzeigename kommen fertig aus dem
  Backend; im Frontend entsteht keine Namens-Fallback- oder
  Ausprägungs-Ableitungsregel. Grep-Nachweis ist die etablierte Beweisform
  (v2.4 `clip_to`-Lock, v2.5 D-53-05/06, Phase 62).
- **Tier-Regel:** `PersonService` bleibt **Basic-Tier** (nur DAOs + Permission +
  Clock/Uuid + Transaction, kein Domain-Service als Dep — Roadmap-Constraint 5).
  Das Detail-Aggregat und ein transaktionaler Anlege-Pfad über beide Aggregate
  sind **Business-Logic-Tier** und werden in `shifty_bin/src/main.rs` **nach** dem
  Basic-Block konstruiert, ohne `OnceLock`-/Forward-Decl-Tricks.
- **`Dioxus.toml`-Proxy-Eintrag für jeden neuen Routen-Präfix im selben Commit**,
  verifiziert per `dx serve`-Smoke, nicht nur per Unit-Test. Dieses Repo hat den
  Bug dreimal geshipt (v1.6 `/toggle`, v1.8 `/vacation-entitlement-offset`,
  v1.10 `/special-days`).
- **`find_sales_person_by_user_id` ist eine Autorisierungsprüfung, kein CRUD**
  (`service_impl/src/sales_person.rs:356-401`). Sie trägt das Self-vs-HR-Gate und
  wurde in Phase 61 auf `person_id` umgestellt — in dieser Phase nicht anfassen.

### Claude's Discretion

- **Zuschnitt des Detail-Aggregats:** ein Endpoint `GET /person/{id}`, der
  Stammdaten, Schichtplan-Zuordnung, Rollen und Einladungen zusammen liefert, oder
  ein Kern-Aggregat plus gezielte Nachladepfade für die schwereren Abschnitte
  (Einladungen). Auflage: **kein N+1 pro Zeile/Abschnitt** und keine
  Ausprägungs-Ableitung im Frontend.
- **Name, Modul-Ablage und Trait-Zuschnitt** des Business-Logic-Services für
  Detail-Aggregat und Anlege-Pfad (Arbeitstitel z. B. `PersonDetailService` bzw.
  Erweiterung von `PersonDirectoryService`).
- **Form des Anlege-Endpoints:** ein neuer transaktionaler Endpoint, der beide
  Objekte anlegt und verknüpft, gegenüber einer Frontend-Orchestrierung aus
  bestehenden Calls. Die transaktionale Variante ist deutlich zu bevorzugen
  (SC-2 verlangt „nach dem Speichern existieren **genau** die gewählten Objekte") —
  die Entscheidung liegt aber beim Planer.
- **Routen-Präfixe und Endpoint-Zuschnitt** (`/person/{id}`, Kandidaten-Pfade),
  mit der Auflage aus dem Dev-Proxy-Lock.
- **Darstellung der Abschnitte** in der Detailansicht (Reihenfolge, Karten vs.
  Sektionen mit Trennlinie). Wiederverwendbar sind die `h2`-Sektionsköpfe aus
  `sales_person_details.rs:95,147,227` und `user_details.rs:76,148`.
- **Form des Kandidaten-Pickers** (Dropdown, durchsuchbare Liste, Dialog) und ob
  er clientseitig filtert.
- **Wie „mindestens ein Umschalter muss an sein"** im Anlege-Dialog durchgesetzt
  wird (deaktivierter Speichern-Knopf vs. Validierungsmeldung).
- **i18n-Key-Namen**; bestehende Keys werden wiederverwendet, wo sie passen. Alle
  drei Sprachen de/en/cs synchron — es existiert ein Test, der die Key-Mengen
  gegen die deutsche Referenz vergleicht.
- **Wellen-Schnitt, Plan-Anzahl und Testablage.**

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Gesperrte Entscheidungen & Scope
- `.planning/ROADMAP.md` §„Milestone v2.7" → „Gesperrte Entscheidungen",
  „Harte Constraints aus der Research" (Punkte **5** Tier-Regel und **6**
  Dev-Proxy betreffen diese Phase), „Backend-/Frontend-Abdeckung pro Phase",
  „Docs-Freshness-Gate" (Zeile Phase 62/63), „Phase Details → Phase 63"
  (Success Criteria 1–5, Rollen-Gating-Absatz, Discuss-Points D-63-01/02/03,
  Verifikations-Hinweis).
- `.planning/REQUIREMENTS.md` — PADM-02, PADM-03, PADM-04, PADM-05, PADM-07;
  §„Out of Scope"; PADM-F-01/F-02 und PERS-F-01 als Future.
- `.planning/seeds/personenverwaltung-zusammenlegen.md` §„Festgelegt (User,
  2026-09-15)" — die Ausprägungs-Tabelle mit „Was der Eintrag hat" je Ausprägung;
  „‚Login hinzufügen' und ‚Person hinzufügen' sind **Aktionen auf dem Eintrag**".
- `.planning/PROJECT.md` §„Fat Backend, Thin Client", §„GSD-Scope-Regel"
  (Plan-Header muss Backend- und Frontend-Pfade explizit nennen), Zeile 532-533
  (Milestone-Zusage zu den Eintrags-Aktionen).

### Vorgänger-Entscheidungen, die hier tragen
- `.planning/phases/62-personen-verzeichnis-eine-liste-statt-tabs/62-CONTEXT.md`
  — **D-62-01** (`kind`-Enum explizit), **D-62-02** (Liste aus dem Outer-Join —
  Waisen sind unsichtbar, Grundlage für D-63-01), **D-62-03** (`display_name` +
  `username` getrennt), **D-62-06** (welche Aktionen Phase 63 ablöst),
  **D-62-07** (Phase 62 legt bewusst keine Detail-Route an), **D-62-08**
  (Admin-Gate an beiden Enden).
- `.planning/phases/62-personen-verzeichnis-eine-liste-statt-tabs/62-07-SUMMARY.md`
  §„Hinweis an Phase 63" — die belegte Toolbar-Diagnose zu D-63-03 inkl.
  i18n-Keys und Zeilennummern; §„Ausprägungs-Badge entfernt" (die Liste zeigt die
  Ausprägung über Verkäufername/Benutzername, nicht über ein Badge).
- `.planning/phases/61-personservice-aufl-sung-von-sales-person-user/61-CONTEXT.md`
  — **D-61-01** (jede Sales Person und jeder Benutzer trägt genau eine Person),
  **D-61-02** („unverknüpft" ist ein Anti-Join, nicht `person_id IS NULL`),
  **D-61-03** (Verknüpfen = Merge, absorbierte Person wird gelöscht),
  **D-61-04** (409-Semantik — trägt Success Criterion 4), **D-61-05** (Lösen
  hinterlässt keine trägerlose Person).
- `.planning/phases/60-person-datenmodell-backfill/60-CONTEXT.md` — **D-60-03**
  (drei gleichwertige Ausprägungen), **D-60-04** (`person` trägt nur Identität +
  Audit — Grundlage dafür, dass D-63-01 gefahrlos ist).

### Fallen mit direktem Phase-63-Bezug
- `.planning/research/PITFALLS.md` §Pitfall 5 (verwaiste/umgehängte
  `Person`-Zeilen — von D-63-01 abschließend geregelt), §Pitfall 10
  (DOM-Rendering-Logik lässt sich in Dioxus/WASM headless schlecht prüfen;
  Entscheidungslogik in reine, unit-getestete Helfer ziehen **und** einen
  Human-UAT von vornherein einplanen), §„Looks Done But Isn't".
- `.planning/research/ARCHITECTURE.md` — Build-Order, Tier-Klassifikation,
  Blast Radius mit `file:line`.

### Bestehender Code, der umgebaut oder gelesen wird
- `shifty-dioxus/src/page/user_details.rs` (333 Zeilen) — wird abgelöst; liefert
  die Benutzer-Abschnitte: Rollen-Zuordnung (`:76-146`), Einladungen inkl.
  Status/Kopieren/Widerrufen (`:148-288`), Einladung erzeugen (`:290-326`).
- `shifty-dioxus/src/page/sales_person_details.rs` (339 Zeilen) — wird abgelöst;
  liefert die Verkäufer-Abschnitte: Stammdaten (`:95-145`), Einstellungen inkl.
  bezahlt/inaktiv und der heutigen „Benutzerkonto verbinden"-Stelle (`:147-225`),
  Schichtplan-Zuordnung mit Berechtigungsstufen (`:227-305`).
- `shifty-dioxus/src/page/user_management.rs:296-315` — die beiden zu ersetzenden
  Anlege-Einstiege (D-63-03); `:358-375` — die Zeilen-Verlinkung auf die alten
  Routen (D-63-04); `:32` `matches_search`; `:506+` SSR-Testharness.
- `shifty-dioxus/src/router.rs:45-50` — `UserManagementPage`, `UserDetails`,
  `SalesPersonDetails`; hier entsteht `/person/:person_id/`.
- `shifty-dioxus/src/component/top_bar.rs:94-95, 883-889` — Breadcrumb-/
  Aktiv-Zweige für die beiden alten Routen, Teil des Rückbaus.
- `shifty-dioxus/src/loader.rs:75, 500` — `delete_user_from_sales_person` und
  `delete_user`; `api.rs` daneben. Hier fehlt heute jeder Pfad zu
  `delete_sales_person` (D-63-06).
- `shifty-dioxus/Dioxus.toml:45-116` — `[[web.proxy]]`-Liste.
- `rest/src/person_directory.rs` — der Phase-62-Aggregat-Endpoint, Vorbild für
  Routing, Gate und utoipa-Annotation.
- `rest/src/sales_person.rs:21-46` — `POST /`, `PUT /{id}`, `DELETE /{id}`,
  `GET|POST|DELETE /{id}/user` (der 409-Pfad aus D-61-04 hängt an `:271-307`).
- `rest/src/permission.rs:20-31` — `POST /user`, `DELETE /user/`,
  `GET /user/{user}/roles`, Rollen-Zuordnung.
- `rest/src/user_invitation.rs` — Einladungs-Endpoints für den Benutzer-Abschnitt.
- `service/src/person.rs` — `PersonService` (Basic-Tier, bleibt es) mit
  `unlinked_sales_persons` / `unlinked_users`, ausdrücklich als
  „Backend-Vorarbeit für Phase 63" dokumentiert.
- `dao/src/person.rs:58-80` + `dao_impl_sqlite/src/person.rs:145-189` — die beiden
  Kandidaten-Anti-Joins inkl. des in Phase 62 korrigierten Soft-Delete-Filters.
- `service_impl/src/permission.rs:135-143` — `delete_user`: **Hard Delete**,
  admin-gated.
- `service_impl/src/sales_person.rs:253-280` — `delete`: **Soft Delete** über
  `deleted`-Stempel, HR-gated.
- `service_impl/src/person_directory.rs` — der Phase-62-Business-Logic-Service.
- `rest-types/src/lib.rs:2854-2925` — `PersonEntryTO` (trägt `person_id`) und
  `PersonKindTO`.
- `shifty_bin/src/main.rs` — DI-Verdrahtung, Business-Logic **nach** Basic.
- `shifty-dioxus/src/page/settings.rs:556-570` — Muster für den
  Component-Level-Admin-Guard.

### Konventionen & Doku-Gate
- `CLAUDE.md` §„Service-Tier-Konventionen: Basic vs. Business-Logic Services",
  §„REST API" (utoipa-Annotation Pflicht, `ToSchema` auf jedem TO),
  §„Billing Period Snapshot Schema Versioning" (kein Bump),
  §„Docs-Freshness-Gate", §Clippy-Hard-Gate
  (`cargo clippy --workspace -- -D warnings` vor jedem Commit).
- `shifty-dioxus/CLAUDE.md` — Frontend-Konventionen.
- `.planning/codebase/frontend/` — `ARCHITECTURE.md`, `CONVENTIONS.md`,
  `TESTING.md` (SSR-Rendertests als Nachweisform).
- **Docs-Trigger dieser Phase (EN **und** DE im selben Commit):** wegen
  `service/**` + `dao/**` die neue/erweiterte `docs/features/F*.md` zur
  Personenverwaltung (Phase 62 hat sie angelegt/erweitert — hier fortschreiben:
  Detail-Aggregat, Anlege-Pfad, Lösch-Semantik je Ausprägung);
  `docs/features/F01-employee-management.md` wegen des neu freigeschalteten
  Verkäufer-Löschpfades; `docs/architecture/02-service-tiers.md` +
  `diagrams/service-graph-runtime.mmd` **nur**, falls die DI-Struktur sichtbar um
  einen Service wächst.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Die beiden abzulösenden Detailseiten sind der Bauplan.** `user_details.rs`
  (333 Zeilen) und `sales_person_details.rs` (339 Zeilen) enthalten die fünf
  Abschnitte, die die neue Komponente ausprägungs-abhängig ein- und ausblendet.
  Das ist überwiegend Verschieben und Umhängen, nicht Neuschreiben.
- **`FormPair`** (in `sales_person_details.rs` durchgehend verwendet) — fertige
  Label-/Feld-Paarung für die Stammdaten-Abschnitte.
- **`Key::ConnectUserAccount`** und die „Benutzerkonto"-Stelle in
  `sales_person_details.rs:189-225` — die heutige, primitive Verknüpfungs-UI;
  sie wird vom Kandidaten-Picker abgelöst und liefert Text und Platzierung.
- **`AddUserDialog` / `DeleteUserConfirmDialog`** (`user_management.rs:91-114`,
  `:144+`) — Dialog-Muster inkl. Bestätigungs-Signal; der neue Anlege-Dialog und
  die zwei Lösch-Bestätigungen folgen derselben Form.
- **SSR-Rendertest-Harness** (`user_management.rs:506+`, 18 grüne Tests in Phase
  62) — rendert den echten Komponentenbaum nach HTML. Direktes Zuhause für die
  Ausprägungs-Sichtbarkeitstests: „bei nur-Benutzer fehlen die Verkäufer-
  Abschnitte", „bei nur-Verkäufer fehlen Rollen und Einladungen".
- **Guard-Muster** `page/settings.rs:556-570` — frühes `return` mit
  „Not authorized.", in Phase 62 bereits für `/user_management/` übernommen.
- **`PersonService::unlinked_sales_persons` / `unlinked_users`** — die
  Kandidaten-Lesepfade existieren seit Phase 61 samt DAO-Anti-Joins und Tests;
  sie brauchen nur REST-Endpoints und eine Picker-UI.
- **`person_directory.rs` (REST + Service)** — das Phase-62-Aggregat als Vorbild
  für Tier-Ablage, Gate, utoipa-Annotation und TO-Zuschnitt.

### Established Patterns
- **`gen_service_impl!`** für DI; Services nehmen `Option<Transaction>` und
  committen selbst — Grundlage für einen transaktionalen Anlege-Pfad, der beide
  Objekte in **einer** Transaktion erzeugt und verknüpft (SC-2).
- **`sqlx::query_as!` mit Compile-Time-Checking**; die lokale Dev-DB muss
  migriert sein (`SQLX_OFFLINE=true` in der Nix-Shell).
- **Soft-Delete-Filter `WHERE deleted IS NULL`** als Konvention; in Joins gehört
  er in die `ON`-Klausel (D-62-05).
- **Zentrales Error-Mapping** über den `error_handler`-Wrapper statt
  Handler-lokalem Status-Mapping (`rest/src/rebooking.rs:14-18`) — auch für den
  409-Pfad aus D-61-04.
- **i18n-Keys in allen drei Sprachen** (`shifty-dioxus/src/i18n/{de,en,cs}.rs`),
  abgesichert durch einen Key-Mengen-Test gegen die deutsche Referenz.
- **Fat-Backend-Nachweis per grep** ist in diesem Projekt die etablierte
  Beweisform.

### Integration Points
- `shifty_bin/src/main.rs` — Detail-Aggregat und Anlege-Service kommen **nach**
  dem Basic-Block.
- `rest/src/lib.rs` (Router-Zusammenbau) + `ApiDoc`-Struct — jeder neue Endpoint
  braucht Route **und** OpenAPI-Eintrag.
- `shifty-dioxus/Dioxus.toml` — Dev-Proxy für `/person`; ohne Eintrag liefert
  `dx serve` 404/405 und kein Unit-Test fängt das.
- `shifty-dioxus/src/loader.rs` + `api.rs` — hier hängen die Lade- und
  Mutations-Funktionen; neu dazu kommen Detail-Aggregat, Anlegen, Kandidaten und
  der bislang fehlende Verkäufer-Löschpfad (D-63-06).
- `shifty-dioxus/src/component/top_bar.rs` — Breadcrumb-/Aktiv-Logik muss die
  neue Route kennen und die beiden alten verlieren.

</code_context>

<specifics>
## Specific Ideas

- **Der sichtbare Maßstab ist „eine Seite, drei Gesichter".** Erfüllt ist die
  Phase, wenn ein Admin dieselbe Route für einen verknüpften Eintrag, ein reines
  Benutzerkonto und einen reinen Verkäufer öffnet und jedes Mal genau die
  Abschnitte sieht, die es zu dem Eintrag gibt — ohne leere Platzhalter.
- **„Das checkt niemand" ist die Messlatte für den Anlege-Einstieg** (User,
  UAT Phase 62). Ein einziger Knopf „Eintrag anlegen", der im Dialog fragt, was
  der Eintrag bekommt — nicht zwei Knöpfe, die vorher eine Ausprägung erzwingen.
- **Der gefährlichste Knopf wird nicht gebaut.** Ausdrücklich entschieden: kein
  Sammel-„Eintrag löschen", das auf einem verknüpften Eintrag einen Hard Delete
  (Benutzer) und einen Soft Delete (Verkäufer) in einer Geste bündelt (D-63-05).
- **„Inaktiv setzen" ist die schonende Alternative zum Verkäufer-Löschen** und
  gehört in die Bestätigung (D-63-06). Das Flag existiert und ist der heute
  genutzte Weg, jemanden aus der Planung zu nehmen.
- **Die Ausprägungs-Sichtbarkeit braucht reine Helfer plus Human-UAT.** Dioxus/
  WASM-DOM-Logik lässt sich headless schlecht prüfen (PITFALLS Nr. 10, dieselbe
  Klasse wie die zwei `human_needed`-Overrides aus v2.5). Die Entscheidung
  „welcher Abschnitt erscheint" gehört in unit-getestete Funktionen, und ein
  Browser-Durchgang ist von vornherein einzuplanen — nicht beim Close zu
  entdecken.
- **Der Dev-Proxy ist der teuerste vergessbare Handgriff.** Dreimal in diesem
  Repo geshipt; der Nachweis ist ein `dx serve`-Smoke gegen `/person`, nicht ein
  grüner Unit-Test.
- **Toolchain:** weder `node` noch `cargo` liegen im Default-PATH. `cargo`,
  `clippy` und `sqlx` laufen über `nix develop --command bash -c '<cmd>'` aus dem
  Repo-Root mit `SQLX_OFFLINE=true`. `rustfmt` fehlt in der Dev-Shell, obwohl CI
  `cargo fmt --all -- --check` fährt — **nichts repo-weit umformatieren.**

</specifics>

<deferred>
## Deferred Ideas

- **Gezählte Betroffenheit in der Lösch-Bestätigung** („47 Buchungen, 3
  Abrechnungsperioden") — ehrlicher als der statische Text aus D-63-08, kostet
  aber einen eigenen Zähl-Endpoint samt Tests. Kandidat für eine spätere Phase
  oder einen Todo.
- **Weiterleitungs-Hüllen für `/user_details/:id/` und
  `/sales_person_details/:id/`** — bewusst nicht gebaut (D-63-04). Falls sich
  gespeicherte Lesezeichen als Problem zeigen, nachträglich nachrüstbar.
- **Abschnittsweises Rollen-Gating** (HR bekommt Teile der Personenverwaltung
  zurück) — **PADM-F-02**, Future. Diese Phase gated die Seite als Ganzes auf
  Admin.
- **HR-Report `/employees/:id` in die Personen-Ansicht einfalten** — **PADM-F-01**,
  Future. Bleibt eine getrennte Sicht.
- **Echte Personenstammdaten** (E-Mail, Telefon, Geburtsdatum) an `person` —
  **PERS-F-01**, Future. `display_name` bleibt bis dahin abgeleitet.
- **Zusammenführen zweier Personen im UI** — die Merge-Mechanik existiert seit
  D-61-03, eine eigene Bedienoberfläche dafür ist kein v2.7-Scope. Der
  Kandidaten-Picker aus PADM-04 deckt den praktischen Bedarf ab.
- **Planungs-Soll für dynamische Verträge** — Phase 64 (PSOLL-01..04).
- **`WR-01`, `WR-03`, `WR-07` und `IN-01..IN-07`** aus `61-REVIEW.md` bleiben im
  Backlog; nur **WR-02** wurde in Phase 62 mitgenommen.

### Reviewed Todos (not folded)

`todo.match-phase 63` liefert acht Treffer, alle mit Score 0.6 und alle aus
Keyword-Zufall über „admin", „user", „service", „dioxus", „page" — keiner betrifft
die Personen-Detailansicht. Es sind dieselben Kandidaten, die schon Phase 62
geprüft und verworfen hat:

- „Cutover-UI als Admin-Feature", „admin-Rolle automatisch alle Privilegien"
  (würde das Gate aus D-62-08 verändern und gehört deshalb bewusst getrennt),
  „Warnung bei Eintrag außerhalb der Vertragszeiten",
  „list_user_invitations stilles leeres Array", „silentRenewIframe in index.html",
  „Committed Voluntary Capacity", „End-to-End Integrationstests mit
  ferngesteuertem Browser" (Phase 59, reserviert).

</deferred>

---

*Phase: 63-Personen-Detailansicht + Ausprägungs-Aktionen*
*Context gathered: 2026-09-20*
