---
phase: 17
slug: contract-editor-unpaid-volunteer-path
status: draft
shadcn_initialized: false
preset: none
created: 2026-06-24
---

# Phase 17 — UI Design Contract

> Visual and Interaction Contract für: `committed_voluntary`-Editorfeld im Vertrags-Editor +
> „alle"-Filter-Toggle in der Mitarbeiteransicht.
> Generiert von gsd-ui-researcher; wird von gsd-ui-checker verifiziert.
>
> **Scope-Notiz:** Dies ist eine *inkrementelle Ergänzung* innerhalb bestehender Dioxus-
> Komponenten — kein neuer Screen, kein neues Design-System. Sämtliche Token-Klassen,
> Spacing-Regeln und Typografie sind vom Bestand (`tailwind.config.js`, `input.css`,
> bestehende Komponenten) vorgegeben. Dieser Contract spezifiziert ausschließlich die
> *Deltas*: ein bedingtes Eingabefeld, einen Toggle-Button und vier bis sechs neue i18n-Keys.

---

## Design System

| Property | Value |
|----------|-------|
| Tool | none (bestehendes Token-basiertes Design-System; kein shadcn) |
| Preset | nicht anwendbar |
| Component library | none — Dioxus RSX + Tailwind Token-Klassen |
| Icon library | Unicode-Emoji-Glyphen für Daten-Token (`🎯` committed, bereits aus Phase 16); UI-Primitives nutzen bestehende Klassen (`text-ink`, `text-ink-muted`, `bg-surface`) |
| Font | System-Font geerbt; Body 16 px / 1.5 line-height (`input.css:16-19`), 15 px bei ≤ 720 px |

**Token-Quelldateien (nicht neu definieren):**
- `shifty-dioxus/input.css:27-77` — CSS Custom Properties (Light + Dark Theme).
- `shifty-dioxus/tailwind.config.js` — Token-benannte Utility-Klassen.
- `shifty-dioxus/src/component/contract_modal.rs` — Bestehendes numerisches Feld-Layout
  (`grid grid-cols-1 md:grid-cols-2 gap-3`, `Field`-Wrapper, `TextInput`-Primitive).
- `shifty-dioxus/src/component/employees_list.rs` — Bestehende Filter-Leiste + Sucheingabe.

---

## Spacing Scale

Diese Phase führt **kein neues Spacing** ein. Alle Abstände werden aus dem bestehenden
Raster übernommen. Deklariert als Referenz für den Executor:

| Token | Wert | Verwendung (bestehend, wiederverwendet) |
|-------|------|-----------------------------------------|
| form-gap | 12 px (`gap-3`) | Abstand zwischen Formularfeldern im `grid`-Container im `contract_modal` |
| field-gap | 4 px (`gap-1`) | Abstand Label → Input innerhalb `Field`-Wrapper |
| help-gap | 4 px (`gap-1`) | Abstand Checkbox → Hilfstext (`flex flex-col gap-1`) |
| panel-pad | 12 px (`p-3`) | Innenabstand im `EmployeesList`-Container |
| search-h | 34 px (`h-[34px]`) | Höhe Sucheingabe (bereits per `SEARCH_INPUT_CLASSES` pinned) |

Ausnahmen: keine. Das neue `committed_voluntary`-Feld sitzt im bestehenden `grid`-Block
(`class="grid grid-cols-1 md:grid-cols-2 gap-3"`) — es ergänzt das Raster, ohne eigene
Spacing-Tokens einzuführen. Der „alle"-Toggle wird im `EmployeesList`-Panel platziert;
kein neues Spacing nötig.

---

## Typography

Diese Phase führt **keine neue Typografie** ein. Alle Stile werden vom bestehenden
Token-Scale geerbt.

> **Hinweis (Checker-FLAG D-4 aufgelöst):** Der globale Body-Fließtext rendert mit
> 16 px / 1.5 (`input.css:16-19`, 15 px bei ≤ 720 px). Die Tailwind-Klasse `text-body`
> in der Tabelle unten meint bewusst den **Form-Control-/Toggle-Wert mit 14 px** — das
> ist die im Bestand verankerte Control-Größe und KEIN Widerspruch zum 16-px-Fließtext.
> Executor: `text-body` 1:1 vom bestehenden `expected_hours`-`TextInput` übernehmen,
> keine eigene Größe setzen.

| Rolle | Klasse | Größe | Gewicht | Zeilenhöhe | Verwendung (diese Phase) |
|-------|--------|-------|---------|------------|--------------------------|
| Body (Form-Control) | `text-body` | 14 px | 400 | 1.5 | Eingabefeld-Wert, Toggle-Label (Control-Größe, nicht Fließtext) |
| Small / Hilfstext | `text-small font-normal text-ink-muted` | 12 px | 400 | inherit | Hilfstext unterhalb Cap-Toggle (bestehend, unveränderter Stil) |
| Micro / Header | `text-micro font-bold uppercase` | 11 px | 700 | inherit | Sektions-Header in `EmployeesList` (bestehend) |
| Label | `Field`-Wrapper-Standard | 14 px | 400 | inherit | Feld-Label für `committed_voluntary` |

---

## Color

Diese Phase führt **keine neuen Farb-Token** ein. Alle Farbwerte werden aus bestehenden
CSS Custom Properties gelesen.

| Rolle | Token | Verwendung |
|-------|-------|------------|
| Dominante Oberfläche (60 %) | `var(--bg)` / `var(--surface)` | Seiten- und Modal-Hintergrund (bestehend) |
| Sekundär (30 %) | `var(--surface-alt)` | Formular-Felder, Sidebar-Panel (bestehend) |
| Akzent (10 %) | `var(--accent)` | Primärer CTA-Button im Modal, aktive Mitarbeiter-Zeile, Fokus-Ring (bestehend) |
| Muted-Text | `var(--ink-muted)` | Hilfstext, Placeholder (bestehend) |
| Fehler | `var(--bad)` | Validierungsfehler (bestehend, für Parse-Fehler im Eingabefeld) |

**Akzent reserviert für:** Primär-Aktionsbutton (`Speichern` / `Erstellen`) im Modal,
Fokus-Ring auf aktiv-fokussierten Form-Controls, aktive Mitarbeiter-Zeile in der Liste.
**Nicht** für den Toggle-Button oder das `committed_voluntary`-Feld-Label.

**Toggle-Stil (Claude's Discretion, Standard-Default):** Der „alle"-Toggle wird als
einfacher `button`-Element mit bestehendem Klassen-Pattern gerendert (`text-small
font-normal`, Border oder leichte Hintergrund-Differenzierung im aktiven Zustand analog
`bg-accent-soft` bei aktiven Elementen). Kein neuer Farbwert — ausschließlich bestehende
Token. Aktiver Zustand: `bg-accent-soft text-ink` (identisch zum bestehenden
Aktiv-Pattern in der Applikation). Inaktiver Zustand: `text-ink-muted`.

Dark-Theme: Alle genannten Token haben Dark-Theme-Werte in `input.css:54-77`;
Token-Nutzung (kein Hex) stellt automatische Dark-Mode-Kompatibilität sicher.

---

## Component Inventory (UI-Oberfläche dieser Phase)

| Oberfläche | Datei | Änderung |
|------------|-------|----------|
| Numerisches Eingabefeld `committed_voluntary` | `src/component/contract_modal.rs` | Neues `Field { TextInput { input_type: "number", step: "0.01" } }` nach dem `vacation_days`-Block (innerhalb bestehender `ContractModalBody`), bedingt sichtbar (D-01) |
| Sichtbarkeits-Signal | `src/component/contract_modal.rs` | `let show_committed = details.cap_planned_hours_to_expected \|\| details.expected_hours == 0.0;` — kein neues State, rein reaktiv aus bestehenden Props |
| „alle"-Toggle-Button | `src/component/employees_list.rs` | Neues `use_signal(|| false)` (`show_all`) + Button oberhalb der Mitarbeiter-Liste (unterhalb Sucheingabe), der `show_all` toggelt; ist-aktiv-Klasse analog Bestand |
| Filter-Kette | `src/component/employees_list.rs` | `.filter(\|e\| *show_all.read() \|\| e.sales_person.is_paid.unwrap_or(false))` nach `.filter(\|e\| !e.sales_person.inactive)` eingehängt |
| State-Struct + TryFrom | `src/state/employee_work_details.rs` | `committed_voluntary: f32`-Feld hinzufügen; beide `TryFrom`-Richtungen patchen (TO→State Z. 145, State→TO Z. 185/218) |
| i18n Keys | `src/i18n/mod.rs` + `de.rs` / `en.rs` / `cs.rs` | 2–3 neue Keys (s. Copywriting Contract); alle drei Locales gleichzeitig befüllen |

**Platzierung des `committed_voluntary`-Felds im Modal:**

Der bestehende `contract_modal.rs`-Layout-Fluss lautet:
1. Wochentags-Pills (Montag–Sonntag)
2. Numerische Felder: `expected_hours` + `workdays_per_week` (2-Spalten-Grid)
3. `vacation_days` (nur wenn `!read_only`)
4. Toggle-Felder: `dynamic`, `cap_planned_hours_to_expected` (mit Hilfstext)
5. Abgeleitete Stunden-Info (Border-Top-Block)

Das neue `committed_voluntary`-Feld wird **nach Block 3 / vor Block 4** als einzelnes
`Field`-Element eingefügt (kein zweites Grid-Element; das Feld hat keinen natürlichen
Partner in der 2-Spalten-Ansicht). Sichtbarkeits-Bedingung: `show_committed = cap || expected_hours == 0.0`.

Diese Position ist semantisch korrekt: das Feld wird erst relevant, wenn der
Cap-Toggle (Block 4) gesetzt ist oder `expected_hours == 0` gilt — der User sieht erst
die Stunden-Felder, dann (bedingt) die Zusage, dann den Toggle der Bedingung kontrolliert.

**Alternativ-Default:** Falls der Executor feststellt, dass das Layout bei `read_only=true`
und sichtbarem `committed_voluntary`-Feld besser in den 2-Spalten-Grid passt (neben
`expected_hours`), ist dieser Ansatz ebenfalls akzeptabel — Vorlage bleibt die
`expected_hours`-Feld-Struktur, `step="0.01"`, `input_type="number"`.

---

## Copywriting Contract

### Neue i18n-Keys (alle 3 Locales gleichzeitig)

| Key | De | En | Cs | Konfidenz |
|-----|----|----|-----|-----------|
| `Key::CommittedVoluntaryLabel` | `Freiwillige Zusage (h)` | `Voluntary Commitment (h)` | `Dobrovolný závazek (h)` | De/En: HIGH; Cs: MEDIUM — User-Bestätigung empfohlen |
| `Key::EmployeesShowAll` | `alle` | `all` | `vše` | HIGH (kurzes Toggle-Label) |
| `Key::CommittedVoluntaryHelp` *(optional, Default: weglassen)* | `Wird in der Jahresansicht als zugesagte freiwillige Kapazität ausgewiesen` | `Shown as committed voluntary capacity in the year view` | `Zobrazeno jako přislíbená dobrovolná kapacita v ročním přehledu` | MEDIUM (Cs); nur hinzufügen wenn der Executor es sinnvoll findet — der Cap-Hilfstext ist bereits vorhanden |

**Pflicht: genau 2 neue Keys** (`CommittedVoluntaryLabel`, `EmployeesShowAll`).
`CommittedVoluntaryHelp` ist optional.

### Standard-Copywriting-Elemente

| Element | Wert |
|---------|------|
| Primärer CTA (Modal) | bestehend: `Speichern` / `Save` / `Uložit` — unveränderter Bestands-Text |
| Leerzustand (Mitarbeiteransicht) | bestehend: `Loading…` / kein neuer Leerzustand |
| Fehlerzustand (Eingabefeld) | bestehend: Parse-Fehler in `TextInput` führt zu keiner Dispatch (stille Ignorierung per `if let Ok(n)`) — kein neuer Fehler-Copy nötig |
| Destruktive Aktion | keine in dieser Phase |

**Hinweis zu `Key::CommittedVoluntaryLabel`:** Der Zusatz `(h)` im Label signalisiert
die Einheit (Stunden), konsistent mit dem bestehenden `expected_hours`-Label-Kontext.
Wenn der Executor feststellt, dass bestehende Labels die Einheit nicht ausweisen, kann
`(h)` entfallen — dann lautet der Key-Wert `Freiwillige Zusage` / `Voluntary Commitment`
/ `Dobrovolný závazek`.

---

## Interaction Contract

### `committed_voluntary`-Eingabefeld

| Verhalten | Spezifikation |
|-----------|---------------|
| Sichtbarkeit | Sichtbar und editierbar wenn `cap_planned_hours_to_expected == true` ODER `expected_hours == 0.0` (D-01, LOCKED) |
| Ausgeblendet | Wenn `cap_planned_hours_to_expected == false && expected_hours != 0.0` — kein leeres Feld, sondern vollständig nicht gerendert |
| Input-Typ | `input_type="number"`, `step="0.01"` (identisch `expected_hours`-Vorlage) |
| Parse-Guard | `if let Ok(n) = value.parse::<f32>()` — NaN / Infinity / ungültige Strings werden nicht dispatched; kein User-sichtbarer Fehler (stille Ignorierung, konsistent mit Bestand) |
| Dispatch | `EmployeeWorkDetailsAction::UpdateWorkingHours` mit gemutiertem `EmployeeWorkDetails`-Clone — identischer Pfad wie `expected_hours` |
| Read-Only-Modus | `disabled: read_only` — identisch zu `expected_hours`-Feld |
| Round-Trip | Open→Speichern-unverändert muss den Backend-Wert bewahren (CVC-09); sichergestellt durch TryFrom-Patch in State-Struct (beide Richtungen) |
| Initialwert | `details.committed_voluntary.to_string()` — kein `format!("{:.2}")` nötig (Browser-Input zeigt Rohwert; identisch zu `expected_hours`) |

### „alle"-Toggle in EmployeesList

| Verhalten | Spezifikation |
|-----------|---------------|
| Trigger | Click auf Toggle-Button setzt `show_all` Signal (D-03, LOCKED) |
| Default-Zustand | `false` — nur bezahlte Mitarbeiter sichtbar |
| Aktiv-Zustand | `true` — bezahlte UND rein unbezahlte Freiwillige (`is_paid=false`) sichtbar |
| Inaktive-Filter | Unverändert: `!sales_person.inactive` greift unabhängig von `show_all` (D-03, kein Kombinieren) |
| Loader-Strategie | Im `show_all`-Modus: zusätzlich `GET /sales-person` laden; `is_paid=false && !inactive`-Personen als `Employee`-Dummy mit Null-Stunden-Daten in die Liste einfügen. Kein neuer Backend-Endpoint nötig (RESEARCH Option A, Pitfall 2) |
| Visuelle Unterscheidung | Unbezahlte Personen tragen `is_paid=false` auf `SalesPerson`; die bestehende `is_paid`-Pill/Label-Logik in `employee_view.rs` (`Key::Paid` / unpaid) kennzeichnet sie automatisch — kein zusätzliches Styling in dieser Phase |
| Toggle-Label | `Key::EmployeesShowAll` — kurzes Label `alle` / `all` / `vše` |
| Aktiv-Klasse | `bg-accent-soft text-ink` (bestehend, kein neuer Token) |
| Inaktiv-Klasse | `text-ink-muted` (bestehend) |

### D-07: Zero-Display in Mitarbeiteransicht

| Situation | Anzeige |
|-----------|---------|
| `committed_voluntary == 0` | `🎯0.00` — schlichte Null, kein Strich/Blank (D-07, LOCKED; konsistent mit Phase-16-Jahresansicht) |
| Bestehende `EmployeeShort`-/`target_hours_for`-Logik | Unverändert — `committed_voluntary` wird in dieser Ansicht nicht separat angezeigt; der Wert wirkt sich über den Reporting-Pfad auf die Jahresansicht aus (Phase 14–16), nicht direkt auf die Listen-Kurzansicht |

*Hinweis: `target_hours_for` liest `expected_hours` aus `working_hours_by_week`. Der
`committed_voluntary`-Wert erscheint in `EmployeeShort` nicht explizit als eigene Zahl —
er ist ausschließlich über den Editor sichtbar und zugänglich.*

---

## Registry Safety

| Registry | Verwendete Blöcke | Safety Gate |
|----------|------------------|-------------|
| keine | keine — kein shadcn, kein Third-Party-Registry | nicht anwendbar |

Kein externes Component-Registry verwendet oder deklariert. Registry-Vetting-Gate: nicht anwendbar.

---

## Pre-Population-Quellen

| Quelle | Übernommene Entscheidungen |
|--------|---------------------------|
| CONTEXT.md D-01 | Sichtbarkeits-Bedingung `cap \|\| expected_hours == 0` |
| CONTEXT.md D-02 | State-Threading, beide TryFrom-Richtungen, Round-Trip-Pflicht |
| CONTEXT.md D-03 | Filter-Semantik: paid-Default, `show_all`-Toggle, Inaktive separat |
| CONTEXT.md D-04 | Erzeugungs-Pfad über bestehenden Editor; kein neues `is_paid`-Control |
| CONTEXT.md D-07 | `🎯0.00` statt Strich/Blank |
| RESEARCH.md Pattern 2 | Exaktes `TextInput`-Block-Template für `committed_voluntary` |
| RESEARCH.md Pattern 4 | `show_all`-Signal + Filter-Ketten-Erweiterung |
| RESEARCH.md Pitfall 2 | Loader-Strategie für unbezahlte Personen (Option A: `GET /sales-person`) |
| Phase-16-UI-SPEC | `🎯`-Emoji-Glyph, Token-Display-Semantik, Zero-Rule |
| contract_modal.rs Z. 297–407 | Layout-Fluss, Grid-Klassen, `Field`/`TextInput`-Primitive |
| employees_list.rs Z. 82–88 | Bestehende Filter-Kette als Einhängepunkt |
| i18n/mod.rs Z. 266–267 | `Key::ShowPaid` / `Key::ShowUnpaid` bereits vorhanden → kein Konflikt |

---

## Verification Hooks (für gsd-ui-checker / gsd-ui-auditor)

- **SSR-Snapshot `committed_visible_when_cap_true`:** `ContractModalBody` mit
  `cap_planned_hours_to_expected=true` → rendert ein `input[type=number][step=0.01]`
  für `committed_voluntary`.
- **SSR-Snapshot `committed_visible_when_expected_hours_zero`:** `ContractModalBody` mit
  `expected_hours=0.0, cap=false` → Feld sichtbar.
- **SSR-Snapshot `committed_hidden_when_no_cap_no_zero`:** `ContractModalBody` mit
  `cap=false, expected_hours=40.0` → kein `committed_voluntary`-Input gerendert.
- **Round-Trip-Test:** `EmployeeWorkDetails { committed_voluntary: 3.5 }` →
  `EmployeeWorkDetailsTO` → zurück → `committed_voluntary == 3.5` (beide TryFrom-Richtungen).
- **i18n-Per-Locale-Matcher:** `i18n_committed_keys_match_german_reference`,
  `i18n_committed_keys_match_english_reference`, `i18n_committed_keys_match_czech_reference`
  — pinnen `Key::CommittedVoluntaryLabel` + `Key::EmployeesShowAll` in allen drei Locales
  (Pitfall-6-Guard aus RESEARCH).
- **Filter-Test:** `show_all=false` → `is_paid=false`-Person nicht in gefilterter Liste;
  `show_all=true` → `is_paid=false && !inactive`-Person erscheint.
- **Kein Legacy-Hex:** Alle neuen Elemente nutzen Token-Klassen (`bg-accent-soft`,
  `text-ink-muted`), keine Hex-Literale in Produktions-RSX.

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS
- [ ] Dimension 2 Visuals: PASS
- [ ] Dimension 3 Color: PASS
- [ ] Dimension 4 Typography: PASS
- [ ] Dimension 5 Spacing: PASS
- [ ] Dimension 6 Registry Safety: PASS

**Approval:** pending
