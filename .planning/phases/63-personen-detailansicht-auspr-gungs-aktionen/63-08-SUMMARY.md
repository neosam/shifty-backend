---
phase: 63-personen-detailansicht-auspr-gungs-aktionen
plan: 08
subsystem: frontend (page / dialogs / pure helpers)
tags: [person-detail, eine-komponente-drei-auspraegungen, retokenization, ssr-rendertest, d-63-02, d-63-05, d-63-06, d-63-07, d-63-08, padm-02, padm-03, padm-04, padm-05, padm-07]
status: complete
requires:
  - "63-03: die 32 i18n-Keys der Detailansicht in de/en/cs"
  - "63-07: api/loader/store — fuenf Store-Felder und neun UserManagementAction-Varianten"
  - "rest_types::{PersonEntryTO, PersonKindTO, PersonFacetTO, AddPersonFacetTO, InvitationResponse, ShiftplanTO}"
  - "component-Atome Btn, Dialog, Field, TextInput, SelectInput, FormCheckbox, TopBar (alle unveraendert)"
provides:
  - "page::person_detail::PersonDetail — die eine Detailseite fuer alle drei Auspraegungen"
  - "page::person_detail::{visible_sections, is_last_facet, delete_confirmation_copy} plus SectionVisibility, FacetKind, DeleteConfirmationCopy"
  - "page::person_detail::{EntryDetailView, MissingFacetCard, UnlinkConfirmDialog, DeleteLoginConfirmDialog, DeleteSalesPersonConfirmDialog, CreatePersonEntryDialog, CandidatePickerDialog, CandidateOption}"
  - "Route::PersonDetail { person_id } an /person/:person_id/"
  - "23 neue Tests (12 Unit auf den Helfern, 7 SSR, 3 Dialog-SSR, 1 Quell-Assertion)"
affects:
  - "63-09 (Rueckbau: oeffnet CreatePersonEntryDialog aus der Werkzeugleiste, entfernt die beiden alten Routen und Seiten, zieht top_bar nach)"
  - "63-10 (Doku: docs/architecture/06-frontend.md + _de.md werden faellig — siehe unten)"
  - "63-11 (Human-UAT gegen dx serve)"
tech_stack:
  added: []
  patterns:
    - "Praesentations-Zwilling: die store-verbundene Seite haelt Guard, Effekte und Dialoge, das Markup liegt in einer Komponente mit injizierten Props — dieselbe Teilung wie PersonDirectoryTable in Phase 62, und genau die, die den SSR-Rendertest ohne Store und ohne Koroutine moeglich macht"
    - "Ein EventHandler<DetailIntent> statt zwoelf einzelner Handler-Props: die Ansicht meldet Absichten, die Seite uebersetzt sie in Aktionen"
    - "Identitaets-Waechter im Nachlade-Effekt: der Effekt liest denselben Store, in den seine Ladevorgaenge schreiben — ohne den Waechter stiesse jedes Ergebnis den naechsten Ladelauf an"
    - "Reine Helfer geben Key-Werte zurueck, nie fertige Zeichenketten — so bleiben sie ohne i18n-Laufzeit testbar"
key_files:
  created:
    - shifty-dioxus/src/page/person_detail.rs
  modified:
    - shifty-dioxus/src/page/mod.rs
    - shifty-dioxus/src/router.rs
    - shifty-dioxus/src/service/user_management.rs
decisions:
  - "Die Route /person/:person_id/ wird bereits hier registriert, nicht erst in 63-09: ohne Erreichbarkeit vom Wurzel-Item aus haelt rustc die ganze Seite samt ihrer Helfer fuer toten Code, und das Plan-Gate `clippy -D warnings` ist konstruktionsbedingt nicht erreichbar. Rule-3-Abweichung, im Detail unten."
  - "Acht der neun #[allow(dead_code)] aus 63-07 sind weg. CreatePersonEntry bleibt: sein Absender ist die Werkzeugleiste der Listenseite, die erst 63-09 umbaut — der Dialog dazu steht bereits hier."
  - "Die Missing-Facet-Karte traegt ein Namensfeld. Der Server verlangt je Seite ein Pflichtfeld (`username` bzw. `display_name`, `PersonDetailServiceImpl::add_facet`); ohne Eingabefeld waere „Login hinzufuegen\" ein garantierter 422. Rule-2-Abweichung, im Detail unten."
  - "Die Verkaeufer-Seite behaelt ihren Speichern-Knopf aus der abgeloesten Seite, aber NICHT deren Weiterleitung: `SaveSalesPersonAndNavigate` setzt nur das Erfolgs-Flag, navigiert hat der alte Seiten-Effekt. Auf der Detailseite bleibt man, wo man ist."
  - "Der Kandidaten-Picker schickt bei fehlender ausdruecklicher Wahl den ersten Eintrag — genau den, den das Auswahlfeld auch anzeigt. Ein Platzhalter-Eintrag waere die Alternative gewesen, haette aber die Optionszahl von der Kandidatenzahl entkoppelt."
  - "`SectionVisibility::as_tuple` ist #[cfg(test)]: die Produktion liest die vier Felder direkt, der Helfer existiert nur, damit die Unit-Tests den vollstaendigen Zustand in einer Zusicherung pruefen koennen."
metrics:
  duration_minutes: 38
  tasks_completed: 3
  files_modified: 4
  tests_added: 23
  completed: 2026-09-21
---

# Phase 63 Plan 08: Die Personen-Detailansicht — Summary

Eine Route, eine Komponente, drei Gesichter. `/person/:person_id/` zeigt fuer
einen verknuepften Eintrag Stammdaten, Schichtplan-Zuordnung, Rollen und
Einladungen; fuer ein reines Benutzerkonto fehlen die Verkaeufer-Abschnitte,
fuer einen reinen Verkaeufer die Benutzer-Abschnitte — und an der Stelle der
fehlenden Seite steht kein leerer Platzhalter, sondern genau eine Karte mit den
zwei getrennt benannten Aktionen.

## Was gebaut wurde

| Art | Symbol | Traegt |
|---|---|---|
| Seite | `PersonDetail` (Props: `person_id: String`) | PADM-05, SC-1 |
| Ansicht | `EntryDetailView` (Markup, injizierte Daten) | SC-1 |
| Reiner Helfer | `visible_sections(PersonKindTO) -> SectionVisibility` | SC-1, T-63-08-E |
| Reiner Helfer | `is_last_facet(PersonKindTO, FacetKind) -> bool` | D-63-01, PADM-07 |
| Reiner Helfer | `delete_confirmation_copy(PersonKindTO, FacetKind)` | D-63-08, T-63-08-B |
| Karte | `MissingFacetCard` | D-63-02, PADM-03 |
| Dialog | `CreatePersonEntryDialog` (pub, Absender ab 63-09) | PADM-02, D-63-07 |
| Dialog | `CandidatePickerDialog` + `CandidateOption` | PADM-04, SC-4 |
| Dialog | `UnlinkConfirmDialog` | D-63-05 |
| Dialog | `DeleteLoginConfirmDialog` | PADM-07, D-63-08 |
| Dialog | `DeleteSalesPersonConfirmDialog` | PADM-07, D-63-06 |
| Route | `Route::PersonDetail { person_id }` an `/person/:person_id/` | D-63-04 |

## Die drei Entscheidungen, die in reinen Funktionen liegen

Dioxus/WASM-DOM-Logik laesst sich headless schlecht pruefen (Verifikations-Hinweis
der ROADMAP, PITFALLS Nr. 10). Die Entscheidungslogik liegt deshalb in freien
Funktionen mit direkten Unit-Tests — dasselbe Muster wie `matches_search` in
`page/user_management.rs`:

- **`visible_sections`** liest ausschliesslich das vom Backend gelieferte
  `PersonKindTO`. Kein Rollen-, Berechtigungs- oder Auth-Bezug; das
  Grep-Kriterium des Plans ueber den Funktionsrumpf gibt 0 aus.
- **`is_last_facet`** ist bewusst mit der Orphan-Pruefung des Backends
  (`PersonDao::has_live_facets`, D-63-01) verkoppelt — der Doc-Kommentar sagt
  warum: weicht sie ab, verspricht die Bestaetigung etwas anderes, als danach
  geschieht.
- **`delete_confirmation_copy`** liefert ausschliesslich `Key`-Werte. Die
  Uebersetzung passiert erst im Dialog; ein Test haelt fest, dass der
  Letzte-Auspraegung-Zusatz genau dann erscheint, wenn `is_last_facet` fuer
  dieselbe Kombination wahr ist.

## Der Beweis fuer „keine leeren Platzhalter"

Die SSR-Rendertests rendern je Auspraegung einzeln und pruefen in beide
Richtungen — die Negativ-Assertions sind der eigentliche Nachweis fuer Success
Criterion 1:

| Auspraegung | Positiv | Negativ |
|---|---|---|
| `Linked` | alle vier Ueberschriften | keine Missing-Facet-Karte |
| `SalesPersonOnly` | Stammdaten + Schichtplan-Zuordnung, Karte „No login yet" | `Role Assignments` und `User Invitations` stehen NICHT im Markup, genau **eine** Karte |
| `UserOnly` | Rollen + Einladungen, Karte „No sales-person record yet" | `Basic Information` und `Shiftplan Assignments` stehen NICHT im Markup, genau **eine** Karte |

Dazu `detail_linked_note_is_plain_text_and_only_when_linked`: der
Verknuepfungs-Hinweis erscheint nur bei `Linked`, sein umschliessendes Element
traegt `text-ink-soft` und **keine** Chip-Klasse (`rounded-full`,
`bg-accent-soft`, `bg-warn-soft`, `border-accent`, `border-warn`). Das im
Phase-62-UAT entfernte Auspraegungs-Badge kommt damit nachweislich nicht zurueck.

### Mutations-Nachweis gegen Vakuitaet

| Mutation | Erwartung | Ergebnis |
|---|---|---|
| `visible_sections(SalesPersonOnly)` auf `(true, true)` gesetzt | die Negativ-Assertion der Benutzer-Seite faellt | **FAILED** — `visible_sections_sales_person_only_hides_the_user_side` und `detail_sales_person_only_hides_the_user_side`, beide |

Die Mutation wurde vollstaendig zurueckgenommen; danach sind alle 26 Tests des
Moduls wieder gruen (`git diff --stat` leer).

## Retokenization: was aus den zwei abgeloesten Seiten wurde

Die Portierung ist Verhalten unveraendert, Klassen neu. Die
Legacy-Class-Retokenization-Tabelle der UI-SPEC wurde vollstaendig angewandt;
`no_legacy_classes_in_source` wacht ab jetzt bei jedem Commit darueber und
prueft 15 Begriffe (die 11 der Vorlage plus `text-yellow-`, `bg-yellow-`,
`border-blue-`, `border-green-`, die in den beiden Quelldateien vorkamen).

| Vorlage | Jetzt |
|---|---|
| `text-gray-800` / `text-gray-600` / `text-gray-500` | `text-ink` / `text-ink-soft` / `text-ink-muted` |
| `bg-white` / `bg-gray-50` / `bg-gray-100` | `bg-surface` / `bg-surface-alt` |
| `border-gray-200` / `border-gray-300` | `border-border` / `border-border-strong` |
| `bg-blue-50 border-blue-200 text-blue-700` (Info-Kasten) | `bg-surface-alt border border-border text-ink-soft` — neutral, kein vierter Farbton |
| `bg-green-100 text-green-800` (Aktiv / Gueltig) | `bg-good-soft text-good` |
| `bg-yellow-100 text-yellow-800` (Abgelaufen) | `bg-warn-soft text-warn` |
| `bg-blue-100 text-blue-800` (Eingeloest) | `bg-accent-soft text-accent` |
| `bg-red-100 text-red-800` (Sitzung widerrufen) | `bg-bad-soft text-bad` |
| rohe Knopf-Elemente (`bg-blue-600 text-white`, Kopieren-Knopf) | `Btn` mit passender Variante |
| rohe Eingabefelder mit `focus:ring-*` | `TextInput` / `SelectInput` / `FormCheckbox` |
| Zurueck-Knopf `mr-3 p-2 text-gray-600 …` | Link mit `text-ink-soft hover:text-ink hover:bg-surface-alt` |

Eine einzige Ausnahme, bewusst: das schreibgeschuetzte Feld mit dem
Einladungs-Link bleibt ein rohes `input`. `TextInput` kennt kein
`readonly`-Attribut, und das Feld traegt gerade **keinen** Fokusring — die
Auflage der Tabelle („nie ein rohes `input` mit Fokus-Klassen") ist damit
eingehalten. Die Klassen sind trotzdem vollstaendig tokenisiert. Ein Kommentar
an Ort und Stelle sagt das.

Die einzigen Inline-Farbwerte sind die aus Daten stammende Verkaeufer-Farbe und
ihre Vorschauflaeche — genau die Ausnahme, die die UI-SPEC erlaubt.

## D-63-05 sichtbar gemacht

Die beiden Danger-Knoepfe stehen in getrennten Aktionsbereichen: „Verkaeufer
loeschen" am Fuss der Verkaeufer-Seite, „Login loeschen" am Fuss der
Benutzer-Seite. Dazwischen liegen zwei vollstaendige Abschnitte. Es gibt kein
Element „Eintrag loeschen", keinen gemeinsamen Handler und keinen gemeinsamen
Dialog. „Verknuepfung loesen" ist sekundaer gestaltet, nicht rot — beide
Objekte bleiben vollstaendig erhalten.

Die beiden Bestaetigungen sagen unterschiedliche Dinge, weil unterschiedliche
Dinge geschehen:

| | Login loeschen | Verkaeufer loeschen |
|---|---|---|
| Wirkung | Hard Delete | Soft Delete |
| Satz (D-63-08, woertlich) | „Login, Rollen und offene Einladungen werden endgueltig geloescht." | „Schichten, Stundenkonto und Abrechnungs-Historie bleiben in der Datenbank erhalten, verschwinden aber aus allen Ansichten." |
| Hinweis | keiner | „inaktiv setzen" als schonende Alternative (D-63-06) |
| Letzte-Auspraegung-Zusatz | wenn `is_last_facet` | wenn `is_last_facet` |

## Deviations from Plan

### Auto-behoben

**1. [Rule 3 — blockierend] Die Route wird hier registriert, nicht erst in 63-09**

- **Gefunden bei:** Task 2, beim ersten `clippy -D warnings`.
- **Problem:** `shifty-dioxus` ist ein Binaer-Crate. Die Totcode-Analyse von
  rustc laeuft als Erreichbarkeitssuche von den Wurzel-Items aus und steigt
  **nicht** in unerreichbare Funktionen hinab. Ohne Routen-Eintrag ist
  `PersonDetail` von `main` aus nicht erreichbar; gemeldet werden dann nicht
  nur die neun Helfer und Typen dieser Datei („never used"), sondern auch die
  neun Action-Varianten aus 63-07 — obwohl die Seite sie konstruiert. Das
  Plan-Gate `cargo clippy -- -D warnings` (Task 2 und Task 3) waere
  konstruktionsbedingt nicht erreichbar gewesen.
- **Verworfene Alternative:** acht `#[allow(dead_code)]` ueber Helfer, Typen und
  Komponenten streuen und die neun Marker aus 63-07 stehen lassen. Das haette
  den Erfolgsmassstab dieses Plans („alle neun Marker entfernt") in sein
  Gegenteil verkehrt und waere echte Drift gewesen.
- **Fix:** `Route::PersonDetail { person_id }` an `/person/:person_id/`, dazu
  `pub use person_detail::PersonDetail;` in `page/mod.rs` und der Re-Export in
  `router.rs`. Ein Kommentar an der Routen-Variante sagt, dass Plan 63-09 die
  beiden Routen darueber ersatzlos entfernt.
- **Nebenwirkung, bewusst:** zwischen diesem Plan und 63-09 existieren drei
  Detailrouten. Die Prohibitions dieses Plans sind davon nicht beruehrt (sie
  zielen auf eine zweite **Personen**-Detailroute bzw. -Komponente; `grep -c
  'pub fn PersonDetail'` gibt 1). Fuer 63-09 verkleinert sich die Arbeit:
  Anlegen entfaellt, nur noch Entfernen und das Nachziehen von
  `component/top_bar.rs`.
- **Dateien:** `shifty-dioxus/src/router.rs`, `shifty-dioxus/src/page/mod.rs`
- **Commit:** `8e209f87`

**2. [Rule 2 — fehlende kritische Funktionalitaet] Namensfeld in der Missing-Facet-Karte**

- **Gefunden bei:** Task 2.
- **Problem:** Die UI-SPEC zeichnet die Karte mit Ueberschrift, Fliesstext und
  zwei Knoepfen. `PersonDetailServiceImpl::add_facet` verlangt aber je Seite ein
  nicht-leeres Pflichtfeld und antwortet sonst mit einer Validierungs-
  Ablehnung: `username` fuer die Benutzer-Seite, `display_name` fuer die
  Verkaeufer-Seite. Ein Knopf ohne Eingabe haette entweder garantiert abgelehnt
  oder einen Benutzernamen aus dem Anzeigenamen ableiten muessen — genau die
  Fallback-Regel, die PADM-06 im Frontend verbietet.
- **Fix:** Die Karte traegt ein `Field` + `TextInput` ueber der Knopfzeile. Fuer
  die Verkaeufer-Seite mit dem Anzeigenamen des Eintrags vorbelegt (der neue
  Verkaeufer-Datensatz uebernimmt ihn), fuer die Benutzer-Seite leer — einen
  Benutzernamen erfindet das Frontend nicht. Der Primaer-Knopf ist gesperrt,
  solange das Feld leer ist; ein Kommentar haelt fest, dass die Sperre
  Bequemlichkeit ist und die verbindliche Pruefung im Service liegt (ASVS V5).
- **Beruehrt keine Prohibition:** das Verbot der Freitexteingabe (SC-4) gilt dem
  **Verknuepfen**; `sed -n '/fn CandidatePickerDialog/,/^}/p' … | grep -c
  'TextInput'` gibt weiterhin 0.
- **Datei:** `shifty-dioxus/src/page/person_detail.rs`
- **Commit:** `8e209f87`

### Abweichungen von der Plan-Prosa (nicht vom Verhalten)

**Zwei Testnamen weichen von der Artefakt-Tabelle ab — die Grep-Kriterien
widersprechen ihr.** Die Tabelle nennt `render_detail_*` und
`visible_sections_*` als Testnamen; die Akzeptanzkriterien verlangen zugleich
`grep -c 'fn render_detail' == 1` und
`sed -n '/fn visible_sections/,/^}/p' | grep -ci 'privilege|role|auth' == 0`.
Beides zusammen ist nicht erfuellbar:

- Jeder Test mit dem Praefix `render_detail_` erhoeht die Zahl der Zeilen mit
  `fn render_detail` ueber 1. Die SSR-Tests heissen deshalb `detail_*`; die
  Harness-Funktion heisst `render_detail` und kommt genau einmal vor.
- Ein Test mit dem Praefix `visible_sections_` eroeffnet fuer `sed` einen
  zweiten Bereich, der mangels `}` in Spalte 0 bis zum Dateiende laeuft. Die
  Tests behalten deshalb ihren vertraglichen Namen, stehen aber **zuletzt** in
  der Datei und pruefen ueber `as_tuple` statt ueber die vier Feldnamen — der
  Feldname `roles` allein wuerde das Kriterium sonst falsch-positiv ausloesen.
  Ein Kommentar an Ort und Stelle erklaert die Reihenfolge, damit ein spaeterer
  Commit sie nicht versehentlich aufhebt.

Ausserdem musste im Kommentarblock ueber diesen Tests das woertliche
Grep-Kommando entfallen: der Kommentar selbst enthielt den gesuchten Begriff
und erzeugte einen Treffer von 2. Die Prosa nennt die Suche jetzt, ohne sie zu
zitieren — derselbe Falsch-Positiv-Mechanismus, vor dem der Ausfuehrungsauftrag
warnt.

**Ein Praesentations-Zwilling statt einer einzigen Komponente.** Die SSR-Tests
verlangen `render_detail(entry) -> String` „Props injiziert, kein Store, kein
Netzwerk". `PersonDetail` liest Store und Koroutine und traegt den Admin-Guard —
ein SSR-Render der Seite selbst liefe in die Nicht-berechtigt-Ansicht und
verlangte einen Koroutinen-Anbieter. Das Markup liegt deshalb in
`EntryDetailView`, genau wie Phase 62 das Markup der Liste in
`PersonDirectoryTable` gelegt hat. Es bleibt **eine** Seitenkomponente und ein
Routenziel (`grep -c 'pub fn PersonDetail'` = 1).

**Kein Bezahlt/Freiwillig-Chip in den Stammdaten.** Der Strukturbaum der
UI-SPEC (§ Layout & Components) listet fuer die Stammdaten `Field Name`,
`Field ShiftplanColor+preview`, `FormCheckbox Paid`, `FormCheckbox Inactive`.
Der Farbabschnitt erwaehnt zusaetzlich einen Chip. Umgesetzt ist der
Strukturbaum: die beiden Umschalter sind die bearbeitbare Wahrheit, ein Chip
daneben waere ihre zweite, nur lesbare Anzeige. Die `warn`-Tokens sind auf der
Seite trotzdem in Gebrauch (Status-Pille „abgelaufen", Konflikt-Meldung des
Pickers).

**Speichern ohne Weiterleitung.** `SaveSalesPersonAndNavigate` heisst so, weil
die abgeloeste Seite einen eigenen Effekt hatte, der auf das Erfolgs-Flag hin
navigierte. Die Aktion selbst navigiert nicht. Die Detailseite uebernimmt sie
unveraendert und laesst den Effekt weg — man bleibt nach dem Speichern auf der
Seite. Beim Betreten wird `ClearSaveSuccess` geschickt, damit kein Erfolg eines
frueheren Besuchs nachwirkt.

## Auth Gates

Keine.

## Threat Flags

Keine neue Angriffsflaeche ausserhalb des Registers. Abgleich:

| Threat | Disposition | Umsetzung |
|---|---|---|
| T-63-08-A (EoP ueber direkten URL-Aufruf) | mitigate | `is_admin`-Guard mit fruehem `return` als erste Anweisung nach den Store-Lesern, vor jedem Zustand und vor jedem Effekt — dieselbe Stelle im Ablauf wie in `page/user_management.rs`. Defense-in-Depth; autoritativ bleibt das Service-Gate aus 63-04/63-05. |
| T-63-08-B (Bestaetigung sagt etwas anderes als sie tut) | mitigate | `delete_confirmation_copy` liefert je Seite Titel, Satz, optionalen Hinweis und optionalen Letzte-Auspraegung-Zusatz; vier Unit-Tests halten die Zuordnung fest, darunter einer, der den Zusatz direkt gegen `is_last_facet` prueft. Human-UAT in 63-11. |
| T-63-08-C (client-seitige Anlege-Validierung) | mitigate | Der Absende-Knopf ist gesperrt, solange kein Umschalter gesetzt ist oder der Login-Umschalter ohne Benutzernamen dasteht; ein Kommentar an Ort und Stelle nennt die Sperre ausdruecklich Bequemlichkeit und verweist auf die Service-Pruefung (ASVS V5). Derselbe Kommentar steht an der Missing-Facet-Karte. |
| T-63-08-D (Kandidatenliste) | accept | Unveraendert: der Picker zeigt Namen, die die admin-gegatete Seite ohnehin auflistet. |
| T-63-08-E (falsche Auspraegungs-Annahme im Client) | mitigate | `visible_sections` liest ausschliesslich `PersonKindTO`; das Grep-Kriterium ueber den Funktionsrumpf gibt 0 aus, und ein zusaetzlicher Quell-Test verbietet jede Fallback-Regel auf dem Anzeigenamen. |
| T-63-08-SC (Supply Chain) | mitigate | Keine neue Abhaengigkeit, kein neues Atom, kein Package-Install. `git diff` beruehrt keine `Cargo.toml`. |

## Known Stubs

Keine.

Die eine bewusst noch nicht verdrahtete Stelle ist kein Stub im Sinne einer
leeren Anzeige: `CreatePersonEntryDialog` existiert vollstaendig samt Tests,
hat aber bis 63-09 keinen Oeffner — die Werkzeugleiste, die ihn aufruft, wird
dort umgebaut. Entsprechend traegt `UserManagementAction::CreatePersonEntry`
als einzige der neun Varianten noch ihre Markierung, jetzt mit dem konkreten
Verweis auf 63-09.

## Docs-Freshness-Gate

Beruehrte Trigger-Dateien laut CLAUDE.md-Tabelle: **keine.**
`shifty_bin/src/main.rs`, `service/**`, `dao/**`, `migrations/sqlite/*.sql`,
`service_impl/src/permission.rs`, `service_impl/src/reporting.rs` und
`service_impl/src/billing_period_report.rs` wurden nicht angefasst.

Zusaetzlich faellig:

| Beruehrt | → faellig |
|---|---|
| `shifty-dioxus/src/router.rs` (neue Route `/person/:person_id/`) | `docs/architecture/06-frontend.md` **und** `_de.md` — Routen-Uebersicht; dazu der Hinweis, dass `/user_details/` und `/sales_person_details/` mit 63-09 ersatzlos entfallen |
| `shifty-dioxus/src/page/person_detail.rs` (neue Seite) | `docs/architecture/06-frontend.md` + `_de.md` — Seiten-/Komponenten-Uebersicht: eine Detailseite fuer drei Auspraegungen, die drei reinen Helfer als pruefbare Entscheidungslogik, der Praesentations-Zwilling als Testmuster |
| `shifty-dioxus/src/service/user_management.rs` (acht Varianten jetzt verdrahtet) | keine eigene Sektion — die Aktionen sind bereits durch 63-07 als faellig vermerkt |

**Nicht in diesem Plan erledigt** — Plan **63-10** fuehrt die
Doku-Aktualisierung der Phase zentral durch (Auflage aus dem
Ausfuehrungsauftrag). Die bereits von 63-07 vermerkte Faelligkeit fuer
`docs/architecture/06-frontend.md` / `_de.md` bleibt bestehen und waechst um
die drei Zeilen oben.

## Deferred Issues

Unveraendert gegenueber 63-07, plus einer:

- `service_impl/src/test/person_directory_dao.rs:149` loest unter
  `cargo clippy --all-targets` eine `needless_lifetimes`-Warnung aus
  (pre-existing, Phase 62, ausserhalb des Scopes).
- `cargo fmt` fehlt in der Nix-Dev-Shell; der neue Code wurde von Hand
  rustfmt-konform geschrieben.
- `Key::CopyToClipboard` ist in `i18n/de.rs` und `i18n/en.rs` je zweimal
  registriert. Aus 63-03 bekannt, ausserhalb des Scopes.
- Der `[[web.proxy]]`-Eintrag fuer `/person` bleibt der einzige Teil der Kette,
  den kein `cargo test` abdeckt — Laufzeit-Nachweis nur per `dx serve`
  (Plan 63-11).
- **Neu:** Die Reihenfolge der `visible_sections_*`-Tests am Dateiende ist ein
  Kriterium-Artefakt, kein fachlicher Zwang. Faellt das Grep-Kriterium in einer
  spaeteren Phase weg, koennen sie zurueck zu ihren Geschwistern wandern; der
  Kommentar dort sagt das.
- **Umgebung, kein Code-Defekt:** auf Port 3000 laeuft weiterhin ein fremder,
  aelterer `shifty_bin` (PID 460885) aus einer frueheren Sitzung. Bewusst nicht
  beendet.

## Verification

| Gate | Ergebnis |
|---|---|
| `cargo build --manifest-path shifty-dioxus/Cargo.toml` | Exit 0, **0 Warnungen** |
| `cargo test --manifest-path shifty-dioxus/Cargo.toml` | **857 passed**, 0 failed (vorher 834 → +23) |
| `cargo test … person_detail` | **26 passed** (gefordert: ≥ 24) |
| `cargo clippy --manifest-path shifty-dioxus/Cargo.toml -- -D warnings` | Exit 0 |
| `cargo clippy --manifest-path shifty-dioxus/Cargo.toml --all-targets -- -D warnings` | Exit 0 |
| `cargo build --manifest-path shifty-dioxus/Cargo.toml --target wasm32-unknown-unknown` | Exit 0 |
| `cargo build --workspace` | Exit 0 |
| `cargo clippy --workspace -- -D warnings` | Exit 0 |
| `grep -c 'fn visible_sections\|fn is_last_facet\|fn delete_confirmation_copy'` | 16 (gefordert: ≥ 3) |
| `sed -n '/fn visible_sections/,/^}/p' \| grep -ci 'privilege\|role\|auth'` | **0** |
| `grep -c 'fn no_legacy_classes_in_source'` | 1 |
| `grep -c 'pub fn PersonDetail'` | **1** |
| `grep -c 'is_admin'` | 2 (gefordert: ≥ 1) |
| `grep -c 'DeleteLoginConfirmDialog\|DeleteSalesPersonConfirmDialog'` | 7 (gefordert: ≥ 4) |
| `grep -c 'fn render_detail'` | **1** |
| `sed -n '/fn CandidatePickerDialog/,/^}/p' \| grep -c 'TextInput'` | **0** |
| `grep -c 'pub fn CreatePersonEntryDialog'` | 1 |
| `grep -c '#\[component\]'` | 11 — davon 8 in der Produktion (eine Seite, eine Ansicht, eine Karte, fuenf Dialoge) und 3 Test-Huellen im `#[cfg(test)]`-Modul |
| `grep -c 'allow(dead_code)'` in `service/user_management.rs` | **1** (vorher 9) |

## Commits

| Task | Commit | Typ |
|---|---|---|
| 1 | `332cf3d5` | `feat(63-08): drei reine Helfer der Personen-Detailansicht` |
| 2 | `8e209f87` | `feat(63-08): Detailseite mit ausprägungsabhängigen Abschnitten und drei Bestätigungen` |
| 3 | `0ecfeb41` | `feat(63-08): Anlege-Dialog, Kandidaten-Picker und SSR-Rendertests je Ausprägung` |

## Self-Check: PASSED

- `shifty-dioxus/src/page/person_detail.rs` — FOUND
- `shifty-dioxus/src/page/mod.rs` — FOUND
- `shifty-dioxus/src/router.rs` — FOUND
- `shifty-dioxus/src/service/user_management.rs` — FOUND
- `63-08-SUMMARY.md` — FOUND
- Commit `332cf3d5` — FOUND
- Commit `8e209f87` — FOUND
- Commit `0ecfeb41` — FOUND
