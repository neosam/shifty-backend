---
phase: 63-personen-detailansicht-auspr-gungs-aktionen
plan: 03
subsystem: frontend-i18n
tags: [i18n, dioxus, person-detail, copywriting-contract, d-63-03, d-63-05, d-63-06, d-63-08]
status: complete
requires:
  - "shifty-dioxus/src/i18n/mod.rs — Key-Enum + Testmuster i18n_user_management_keys_present_in_all_locales (Phase 62)"
  - ".planning/phases/63-.../63-UI-SPEC.md § Copywriting Contract — die DE/EN/CS-Tabellen sind die Quelle, nicht eine Anregung"
  - ".planning/phases/63-.../63-CONTEXT.md D-63-08 — die zwei woertlich gesperrten Saetze"
provides:
  - "32 neue Key-Varianten in shifty-dioxus/src/i18n/mod.rs (Seitenrahmen, Anlege-Dialog, Facet-Add, Missing-Facet-Karte, Kandidaten-Picker, Verknuepfung loesen, Login loeschen, Verkaeufer loeschen, gemeinsame Zusaetze)"
  - "je 32 belegte Texte in de.rs, en.rs und cs.rs"
  - "Test i18n::tests::i18n_person_detail_keys_present_in_all_locales — Parity-Gate ueber alle drei Sprachen"
  - "Test i18n::tests::i18n_person_detail_templates_carry_name_placeholder — {name}-Garantie fuer die zwei parametrischen Texte"
affects:
  - "63-04 / 63-06 ff. (Detailansicht, Missing-Facet-Karten, Kandidaten-Picker, Bestaetigungsdialoge konsumieren die Keys ohne Nachpflege)"
  - "63-0x Toolbar-Umbau in page/user_management.rs (Key::CreatePersonEntry ersetzt CreateNewSalesPerson + AddNewUser, D-63-03)"
tech_stack:
  added: []
  patterns:
    - "i18n zuerst, Markup spaeter: der Parity-Test steht im selben Commit wie das Enum, die Texte im naechsten — kein spaeterer Plan pendelt zwischen Markup und Uebersetzung"
    - "Entscheidungs-ID im Doc-Kommentar des Keys statt nur im Planungsdokument: D-63-03/05/06/08 stehen an genau der Zeile, die ein spaeterer Textpolitur-Commit anfassen wuerde"
    - "Ein Key, zwei Verwendungsstellen (CreatePersonEntry als Toolbar-Knopf und Dialogtitel; LinkExisting* als Aktion und Picker-Titel) — bewusst kein Duplikat-Key"
    - "Platzhalter-Garantie als eigener Test neben dem Mengen-Test: der Mengen-Test faengt fehlende Keys, der Platzhalter-Test faengt eine Uebersetzung, die {name} verschluckt"
key_files:
  created: []
  modified:
    - shifty-dioxus/src/i18n/mod.rs
    - shifty-dioxus/src/i18n/de.rs
    - shifty-dioxus/src/i18n/en.rs
    - shifty-dioxus/src/i18n/cs.rs
    - .planning/phases/63-personen-detailansicht-auspr-gungs-aktionen/deferred-items.md
decisions:
  - "Die zwei D-63-08-Saetze stehen zeichengleich in de.rs; die Zusaetze (DeleteConfirmIrreversible, DeleteConfirmLastFacet) sind eigene Keys und werden im Markup unter den gesperrten Satz gerendert, statt ihn zu verlaengern — so bleibt der Grep-Nachweis stabil."
  - "In DeleteSalesPersonConfirmHint werden echte deutsche typografische Anfuehrungszeichen gesetzt („inaktiv setzen“). Die UI-SPEC-Tabellenzelle traegt oeffnend „ und schliessend ein gerades \" — das ist ein Markdown-Tabellen-Artefakt; der Plan verlangt ausdruecklich typografische Anfuehrungszeichen."
  - "Zusaetzlich zum geforderten Mengen-Test wurde i18n_person_detail_templates_carry_name_placeholder angelegt (Muster: i18n_rebooking_templates_carry_placeholders). Der Mengen-Test allein wuerde eine Uebersetzung, die {name} verliert, nicht bemerken — das waere ein stiller Text ohne Namen im Bestaetigungsdialog (T-63-03-A)."
  - "Keine Umbenennung der Seed-Begriffe: AddLoginToEntry = „Login hinzufügen“, AddSalesPersonToEntry = „Person hinzufügen“ (D-63-02, PADM-03)."
metrics:
  duration_minutes: 18
  tasks_completed: 2
  files_modified: 4
completed: 2026-09-20
---

# Phase 63 Plan 03: i18n-Texte der Personen-Detailansicht Summary

32 neue i18n-Keys fuer Detailansicht, Anlege-Dialog, Kandidaten-Picker und die drei
Bestaetigungen in de/en/cs angelegt, abgesichert durch einen Mengen- und einen
Platzhalter-Test — die Texte stehen vollstaendig bereit, bevor die erste Komponente sie braucht.

## What Was Built

**Task 1 — Key-Varianten und Parity-Test** (`shifty-dioxus/src/i18n/mod.rs`, Commit `5eb2c34a`)

32 neue `Key`-Varianten als zusammenhaengender Block direkt hinter dem
Phase-62-Personen-Verzeichnis-Block, in der Reihenfolge der Artefakt-Tabelle des Plans:
Seitenrahmen (2), Anlege-Dialog (6), Facet-Add (2), Missing-Facet-Karte (4),
Kandidaten-Picker (6), Verknuepfung loesen (3), Login loeschen (3), Verkaeufer loeschen (4),
gemeinsame Zusaetze (2).

Die Keys, die eine Entscheidung tragen, haben den Entscheidungs-Verweis direkt im
Doc-Kommentar: `CreatePersonEntry` (D-63-03 — ersetzt die beiden heutigen Anlege-Knoepfe),
`UnlinkAction` (D-63-05 — harmlos und umkehrbar, deshalb keine Danger-Beschriftung),
`DeleteLoginConfirmBody` und `DeleteSalesPersonConfirmBody` (D-63-08 — woertlich festgelegt),
`DeleteSalesPersonConfirmHint` (D-63-06 — die „inaktiv setzen“-Auflage).

Dazu zwei Tests am Ende des Testmoduls:
- `i18n_person_detail_keys_present_in_all_locales` — iteriert ueber `[En, De, Cs]` und
  prueft fuer jeden der 32 Keys, dass `t(key)` weder leer noch `??` ist (Muster:
  `i18n_user_management_keys_present_in_all_locales`).
- `i18n_person_detail_templates_carry_name_placeholder` — prueft, dass `UnlinkConfirmBody`
  und `DeleteSalesPersonConfirmHint` in allen drei Sprachen `{name}` behalten.

**Task 2 — Texte in de/en/cs** (`de.rs`, `en.rs`, `cs.rs`, Commit `35d5426b`)

Je 32 `add_text`-Aufrufe, Abschnitt „Personen-Detailansicht (Phase 63)“, in derselben
Reihenfolge und Gruppierung wie im Enum. Alle Zeichenketten stammen woertlich aus den
Copywriting-Contract-Tabellen der `63-UI-SPEC.md`. Lange Strings in der mehrzeiligen
`add_text`-Form (Zeilenbreite 100), kurze einzeilig — `cargo fmt` steht in der Dev-Shell
nicht zur Verfuegung, die Formatierung wurde deshalb generiert statt nachformatiert.

## Key Implementation Details

- **Ein Key, zwei Verwendungsstellen.** `CreatePersonEntry` dient als Toolbar-Knopf und als
  Dialogtitel; `LinkExistingUser`/`LinkExistingSalesPerson` dienen als Aktion und als
  Picker-Dialogtitel. Bewusst je ein Eintrag — ein Duplikat-Key koennte spaeter divergieren.
- **Keine Doppelregistrierung unter den neuen Keys.** Geprueft mit
  `grep -o 'Key::[A-Za-z]*' … | sort | uniq -d` ueber alle drei Dateien: der einzige Treffer
  ist das vorbestehende `Key::CopyToClipboard` (siehe Deferred Issues), keiner der 32 neuen Keys.
- **Kein Sammel-Loesch-Text.** `grep -ci 'DeletePersonEntry\|DeleteEntryAction'` auf `mod.rs`
  gibt 0 — D-63-05 verbietet die Aktion, also existiert auch keine Beschriftung dafuer.
- **Phase-62-Keys unberuehrt.** `PersonKindLinked`/`…SalesPersonOnly`/`…UserOnly` stehen
  unveraendert im Enum und werden nicht fuer ein farbiges Badge recycelt.

## Verification

Alle Kriterien beider Tasks ausgefuehrt und gruen:

| Gate | Ergebnis |
|---|---|
| `cargo build --manifest-path shifty-dioxus/Cargo.toml` | Exit 0 |
| `cargo test --manifest-path shifty-dioxus/Cargo.toml` | 832 passed, 0 failed |
| `cargo test … i18n` | 61 passed; `i18n_person_detail_keys_present_in_all_locales` ok |
| `cargo clippy --manifest-path shifty-dioxus/Cargo.toml --all-targets -- -D warnings` | Exit 0 |
| `grep -c 'Login, Rollen und offene Einladungen werden endgültig gelöscht.' de.rs` | 1 |
| `grep -c 'Schichten, Stundenkonto und Abrechnungs-Historie bleiben in der Datenbank erhalten' de.rs` | 1 |
| `grep -c 'Login hinzufügen' / 'Person hinzufügen' / 'Eintrag anlegen' de.rs` | je 1 |
| `grep -c 'Key::DeleteConfirmLastFacet'` in de/en/cs | je 1 |
| `grep -ci 'DeletePersonEntry\|DeleteEntryAction' mod.rs` | 0 |
| `grep -c 'PersonKindLinked' mod.rs` | 2 (Enum + Phase-62-Test) |
| Neue Eintraege je Locale-Datei | je 32 |

`shifty-dioxus` ist per `exclude = ["shifty-dioxus"]` **kein** Workspace-Member — der
CLAUDE.md-Gate `cargo clippy --workspace` deckt es nicht ab. Deshalb wurden Build, Test und
Clippy explizit mit `--manifest-path shifty-dioxus/Cargo.toml` gefahren, zusaetzlich mit
`--all-targets`, damit die neuen Tests selbst mit unter das Clippy-Gate fallen.

## Deviations from Plan

### Auto-fixed / ergaenzt

**1. [Rule 2 - Missing critical functionality] Zusaetzlicher Platzhalter-Test**
- **Found during:** Task 1
- **Issue:** Der geforderte Mengen-Test prueft nur, dass ein Key einen nicht-leeren Text hat.
  Eine Uebersetzung, die `{name}` verliert, waere damit unsichtbar — der Bestaetigungsdialog
  wuerde ohne Namen erscheinen, genau das Risiko aus T-63-03-A ("der Text ist die einzige
  Information, auf deren Grundlage ein Admin eine destruktive Aktion freigibt").
- **Fix:** `i18n_person_detail_templates_carry_name_placeholder` ergaenzt, Muster aus
  `i18n_rebooking_templates_carry_placeholders` derselben Datei.
- **Files modified:** `shifty-dioxus/src/i18n/mod.rs`
- **Commit:** `5eb2c34a`

**2. [Textpraezisierung] Typografische Anfuehrungszeichen in `DeleteSalesPersonConfirmHint`**
- **Found during:** Task 2
- **Issue:** Die UI-SPEC-Tabellenzelle traegt oeffnend `„` (U+201E), schliessend aber ein gerades
  `"` (U+0022) — nachweislich per Hexdump. Der Plan verlangt ausdruecklich "typografische
  Anfuehrungszeichen um den Ausdruck".
- **Fix:** Deutsches Anfuehrungszeichen-Paar gesetzt: `„inaktiv setzen“`. EN/CS tragen laut
  Contract ohnehin keine Anfuehrungszeichen.
- **Files modified:** `shifty-dioxus/src/i18n/de.rs`
- **Commit:** `35d5426b`

Sonst keine Abweichungen — die beiden gesperrten D-63-08-Saetze, die Seed-Begriffe
„Login hinzufügen“/„Person hinzufügen“ und „Eintrag anlegen“ stehen woertlich so da, wie
Kontext und UI-SPEC sie festlegen.

## Deferred Issues

**Vorbestehende Doppelregistrierung von `Key::CopyToClipboard`** — `de.rs:579` + `:645` und
`en.rs:532` + `:592` registrieren denselben Key zweimal; der zweite Aufruf ueberschreibt den
ersten still ("Kopieren" -> "In die Zwischenablage kopieren" bzw. "Copy" -> "Copy to
Clipboard"), `cs.rs` hat nur einen Eintrag. Stammt aus einer frueheren Phase, liegt ausserhalb
des Scopes von 63-03 (T-63-03-C betrifft nur die 32 neuen Keys). Festgehalten in
`deferred-items.md`; braucht eine eigene Produktentscheidung, welcher Text gewinnen soll.

**`cargo fmt` fehlt in der Nix-Dev-Shell** (`no such command: fmt`), obwohl CI ein fmt-Gate
faehrt. Bereits aus 63-02 bekannt, erneut in `deferred-items.md` notiert.

## Threat Flags

Keine. Der Plan fuegt keine neue Netz-, Auth- oder Dateioberflaeche hinzu; alle Aenderungen sind
statische Zeichenketten und Enum-Varianten. T-63-03-A ist durch die zwei Grep-Kriterien und den
neuen Platzhalter-Test abgesichert, T-63-03-C durch den `uniq -d`-Nachweis.

## Known Stubs

Keine. Die Keys sind vollstaendig belegt; dass sie noch keinen Renderer haben, ist die
beabsichtigte Reihenfolge dieses Plans (i18n vor Markup), kein Stub.

## Self-Check: PASSED

- `shifty-dioxus/src/i18n/mod.rs` — FOUND
- `shifty-dioxus/src/i18n/de.rs` — FOUND
- `shifty-dioxus/src/i18n/en.rs` — FOUND
- `shifty-dioxus/src/i18n/cs.rs` — FOUND
- Commit `5eb2c34a` — FOUND
- Commit `35d5426b` — FOUND
