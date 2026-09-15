---
phase: 55
slug: manual-rebooking-hr-alert
status: approved
shadcn_initialized: false
preset: none
created: 2026-07-23
reviewed_at: 2026-07-23
---

# Phase 55 — UI Design Contract (Manuelle Umbuchung + HR-Alert-Modal, F3 + F5)

> Visueller und Interaktions-Vertrag für Phase 55. Erzeugt von gsd-ui-researcher,
> verifiziert von gsd-ui-checker.
> Diese Phase fügt der bestehenden Dioxus/WASM-App **drei neue Komponenten** +
> **zwei Integrationen** hinzu. Das Design-System wird 1:1 aus der Codebase
> abgeleitet — **keine Greenfield-Design-Entscheidungen**, keine neue visuelle Sprache.
> Alle D-55-*-Interaktionsentscheidungen sind bereits in `55-CONTEXT.md` gepinnt und
> werden hier NICHT neu gestellt — dieser Vertrag setzt sie visuell/copywise um.

---

## Design System

| Property | Value |
|----------|-------|
| Tool | none |
| Preset | not applicable |
| Component library | none (hand-geschriebenes Dioxus RSX + Tailwind Utility-Klassen) |
| Icon library | none (inline Unicode-Glyphs: ⚠️, ✓, ✕, ×, ▾) |
| Font | Inter (system-ui Fallback); JetBrains Mono für numerische/mono Kontexte |

Quelle: `shifty-dioxus/tailwind.config.js` — CSS-Variable-basiertes Design-Token-System
(via `input.css` aufgelöst). Dies ist ein Rust/Dioxus/WASM-Projekt → **shadcn-Gate nicht
anwendbar**, Registry-Safety-Gate nicht anwendbar (keine externe Component-Registry).

**Wiederverwendungs-Prinzip (bindend):** Alle drei neuen Komponenten mirroren
bestehende Präzedenzen. Kein neues Modal-Chrome, keine neuen Farb-Tokens, keine neuen
Typo-Rollen.

| Neue Komponente | Präzedenz zum Kopieren |
|-----------------|------------------------|
| `rebooking_alert_banner.rs` | `warning_list.rs` (Warn-Banner `border-l-[3px] border-warn bg-warn-soft`) + `impersonation_banner.rs` (dauerhafte, klickbare Zeile) |
| `rebooking_suggestion_modal.rs` | `dialog.rs` (`Dialog`-Shell mit title/subtitle/footer) + `atoms::TupleRow` (IST/DANN-Label-Value-Zeilen) + `voluntary_stats_row.rs` (`font-mono tabular-nums`, `text-warn` bei negativem Delta) |
| `manual_rebooking_modal.rs` | `dialog.rs` (`Dialog`-Shell) + `absence_convert_modal.rs` (Submit-Defense + inline `text-bad`-Error + `select`/`input`-Muster) |
| F3-Trigger (Integration) | `top_bar.rs` `TopBar {}` Action-Menu (D-55-06) + `atoms::Btn` `BtnVariant::Primary` |

---

## Spacing Scale

Tailwind numerische Skala (alle Werte Vielfache von 4). Effektive Schritte wie in der
bestehenden App:

| Token | Tailwind-Klasse | px | Verwendung in dieser Phase |
|-------|-----------------|-----|----------------------------|
| xs | `gap-1`, `p-1`, `mt-1` | 4px | Icon-Gaps, inline Error-Abstand unter Feldern |
| sm | `gap-2`, `p-2`, `py-2`, `py-1.5`, `gap-3`, `py-3` | 8/12px | Feld-Gaps, Button-Padding, TupleRow-Zeilen, Footer-Button-Gap |
| md | `px-4`, `p-4`, `mx-4`, `gap-4` | 16px | Banner-Innenpadding, Modal-Body-Padding, Sektions-Abstand |
| lg | `px-3` | 12px | Button-Horizontal-Padding, Nav-/Action-Menu-Item |

**Modal-Chrome kommt aus `Dialog`** (nicht neu spezifizieren): Backdrop-Padding 16px,
Header `padding:16px 18px 0`, Body `padding:14px 18px 16px`, Footer `px-[18px] py-3`.
Panel-Radius `var(--r-lg)`. Diese Werte sind bereits in `dialog.rs` gekapselt.

**Banner-Wrapper** (mirror `warning_list.rs`-Präzedenz + Phase-24-Overage-Sektion):
äußerer Abstand `mx-4 my-3`, innerer Block `px-4 py-3` (bzw. `p-4`).

### Ausnahmen / Legacy — NICHT für neue Phase-55-Elemente übernehmen

Diese Nicht-4er-Arbitrary-Werte existieren im Umfeld (Form-Inputs). Neue Elemente
nutzen sie NICHT.

| Legacy-Klasse | px | Wo (bestehender Code) |
|---------------|-----|------------------------|
| `px-[10px]` | 10px | Bestehende Form-Input-Horizontal-Paddings |
| `h-[34px]` | 34px | Bestehende Input-/Select-Höhe |
| `p-2.5` | 10px | `WarningList` dense-Variante (nur wenn dense-Prop) |

---

## Typography

Alle Rollen sind in `tailwind.config.js` (`fontSize`) kanonisch definiert. Phase 55 nutzt
eine Teilmenge und beschränkt sich auf **zwei Font-Weights**: `font-normal` (400) und
`font-semibold` (600). `font-medium` (500) wird in neuen Phase-55-Oberflächen NICHT
verwendet (Ausnahme: `Btn`/`nav_item_class`-Atome tragen ihr eigenes `font-medium` intern
— diese werden unverändert wiederverwendet, nicht neu geschrieben).

| Rolle | Tailwind-Klasse | Size | Weight | Line Height | Verwendung in dieser Phase |
|-------|-----------------|------|--------|-------------|----------------------------|
| Micro (Eyebrow) | `text-micro font-semibold uppercase` | 11px | 600 | 14px | Banner-Eyebrow-Header, Modal-Spalten-Überschrift IST/DANN |
| Small | `text-small` | 12px | 500 | 16px | Feld-Labels (`TupleRow`-Label), Modal-Hilfstext, inline Fehler/Erfolg |
| Body | `text-body` | 14px | 400 | 20px | Banner-Text, TupleRow-Value, Button-Text |
| Label (betont) | `text-body font-semibold` | 14px | 600 | 20px | Banner-Person-Name, Approve-Button-Text |
| Lg (Modal-Titel) | `text-lg` | 16px | 600 | 22px | Modal-Titel (kommt aus `Dialog { title }`, nicht neu setzen) |
| H2 | `text-h2 font-semibold` | 18px | 600 | 24px | (optional) Sektions-Überschrift falls Banner-Gruppe einen Header braucht |

> **Aktiv gesetzte Größen (Checker-Klarstellung):** Neue Phase-55-Oberflächen setzen
> netto nur **drei** Größen selbst — `micro` / `small` / `body`. `lg` (Modal-Titel) wird
> aus der `Dialog`-Shell **geerbt** (nicht neu gesetzt), `h2` ist **optional/konditional**.
> Damit bleibt der effektive Kontrakt innerhalb der ≤4-Größen-Regel; die Tabelle listet
> die vollständige kanonische `tailwind.config.js`-Skala nur zur Referenz.

**Numerik:** Alle Stunden-/Delta-Werte in `font-mono tabular-nums` (Präzedenz
`voluntary_stats_row.rs`). IST- und DANN-Zahlen im Suggestion-Modal rechtsbündig,
mono, damit die Spalten optisch aligned sind. Format via `format_hours(x, 2)` +
`Key::Hours`-Suffix; Delta signed via `format!("{:+.2} {}", delta, hours)`.

---

## Color

Alle Werte sind CSS-Variable-Aliase aus `tailwind.config.js`, aufgelöst in `input.css`.

**60/30/10-Split:** geerbt aus der App-Surface-Verteilung (dominant `surface`/`bg`,
sekundär `surface-alt`/Cards, Accent reserviert für aktive Nav + Primär-CTA). Phase 55
führt KEINE neue Dominant/Sekundär-Palette ein — sie legt nur die bestehenden
semantischen Tokens `accent`, `warn`, `bad`, `good` auf die drei neuen Oberflächen.

| Rolle | Token | Tailwind-Klassen | Verwendung in dieser Phase |
|-------|-------|------------------|----------------------------|
| Dominant Surface (60%) | `surface` / `bg` | `bg-surface` | Modal-Panel (via `Dialog`), Seiten-Hintergrund |
| Sekundär (30%) | `surface-alt` | `bg-surface-alt` | Modal-Footer, Hover-States, Banner-Hover |
| Border | `border` / `border-strong` | `border-border`, `border-border-strong` | Modal-Border, Feld-Border, Sekundär-Button-Border |
| Ink | `ink` / `ink-soft` / `ink-muted` | `text-ink`, `text-ink-soft`, `text-ink-muted` | Body-, Beschreibungs-, Hilfstext |
| Accent (10%) | `accent` | `bg-accent text-accent-ink`, `bg-accent-soft text-accent` | Primär-CTA (F3-Anlegen, Approve); F3-Menu-Item-Active |
| Warn | `warn` | `text-warn`, `bg-warn-soft`, `border-warn`, `border-l-[3px] border-warn` | HR-Alert-Banner (dauerhafte Warnzeile); negatives Delta (DANN-Balance-Defizit) |
| Bad | `bad` | `text-bad`, `bg-bad-soft`, `border-bad` | Inline-Fehler (409 Slot belegt), Reject-Button (destruktive Ablehnung) |
| Good | `good` | `text-good`, `bg-good-soft` | DANN-Balance ≥ 0 nach Umbuchung (Positiv-Signal in Vorschau) |

**Accent ausschließlich reserviert für:**
- Primär-CTA „Umbuchung anlegen" (F3-Submit-Button, `BtnVariant::Primary`).
- Approve-Button im Suggestion-Modal (`bg-accent text-accent-ink`).
- F3-Action-Menu-Item Active-State in der TopBar (`nav_item_class(true)` = `bg-accent-soft text-accent`).

NICHT für: den Alert-Banner (der ist `warn`, nicht `accent` — es ist eine
Aufmerksamkeit, kein Call-to-Action-Highlight), nicht für Reject (das ist `bad`),
nicht für Sekundär-/Cancel-Buttons.

**Farb-Zuordnung pro Oberfläche:**
- **HR-Alert-Banner (F5, HR-ALERT-01):** `border-l-[3px] border-warn bg-warn-soft` —
  mirrort exakt das `WarningList`-Muster. `warn`, NICHT `bad`: es ist ein dauerhafter
  Hinweis auf Ausgleichsbedarf, kein harter Fehler. Klickbar → `hover:bg-surface-alt`
  oder subtiler Cursor-Pointer + `hover:brightness`.
- **Suggestion-Modal DANN-Spalte:** Balance-Delta-Färbung: `text-warn` wenn DANN-Balance
  weiterhin < 0, `text-good` wenn DANN-Balance ≥ 0 (Ausgleich erreicht). IST-Spalte immer
  neutral `text-ink`.
- **Reject-Button (HR-ALERT-03):** `BtnVariant::Danger` = `bg-surface text-bad border-bad`
  (Reject ist eine bewusste Ablehnung mit Audit-Persistenz, D-55-07). Kein
  Confirm-Dialog davor (Reject ist nicht destruktiv im Sinne von Datenverlust — es
  persistiert einen Audit-Eintrag; siehe Copywriting/Destructive).
- **Inline-Fehler 409 (D-55-05):** `text-bad text-small font-normal` inline im
  Manual-Modal — mirrort `absence_convert_modal.rs:166` Error-Muster.

---

## Component Inventory

Drei neue Komponenten + zwei Integrationen. Alle reusen bestehende Präzedenzen.

### Surface 1: HR-Alert-Banner — `rebooking_alert_banner.rs` (F5, HR-ALERT-01, D-55-01/07)

**Ort:** `shifty-dioxus/src/page/employees.rs` — Employee-Overview, pro betroffenem
SalesPerson eine dauerhafte Zeile (HR-Path).
**Trigger:** Backend-berechnet. Das `ShortEmployeeReportTO` trägt `has_pending_rebooking:
bool` (D-55-02). Der Alert-Predicate (`balance <= -0.5h ∧ voluntary_ist > 0 ∧ cap_active`,
D-55-01) ist ebenfalls backend-computed — das FE zeigt die Zeile, wenn das DTO sie
markiert. **Keine FE-Arithmetik** (Fat Backend, Thin Client).
**Sichtbarkeit:** nur HR (das DTO liefert Non-HR ohnehin `has_pending_rebooking=false` /
redaktierte Felder — API-Level-Redaktion, kein FE-Rollen-Check).
**Dismissible:** Nein. Die Zeile verschwindet ausschließlich, wenn der Predicate
`false` wird — nach Approve (Balance neu) ODER Reject (`state != Pending`) ODER
natürlicher Ausgleich (D-55-07). Kein Schließen-Button.
**Interaktion:** Die gesamte Zeile ist klickbar (`role="button"`, `tabindex="0"`,
`cursor-pointer`) → öffnet `rebooking_suggestion_modal` für diese Person.

Layout (mirror `WarningList`):
```
div.border-l-[3px].border-warn.bg-warn-soft.rounded-md.px-4.py-3.flex.items-center.justify-between.gap-3.cursor-pointer.hover:bg-surface-alt
  ├─ span.text-body.text-ink  →  "⚠️ {Person}: {RebookingAlertBannerText}"  (Person in font-semibold)
  └─ span.text-small.text-warn.font-semibold  →  "{RebookingAlertBannerAction}"  (z.B. "Vorschlag prüfen →")
```
Der `{RebookingAlertBannerText}` nennt Balance-Defizit + verfügbare Freiwilligenstunden
(backend-formatiert oder placeholder-substituiert). Zahlen `font-mono tabular-nums`.

### Surface 2: Suggestion-Modal — `rebooking_suggestion_modal.rs` (F5, HR-ALERT-02/03)

**Basis:** `Dialog`-Shell (`variant: Center`, `width: 460`), `title` = `RebookingSuggestionModalTitle`,
`subtitle` = Person-Name.
**Body:** IST/DANN-Tabelle — vier Kennzahl-Zeilen, alle DANN-Werte **backend-computed**
im `RebookingSuggestionTO` (HR-ALERT-02):

| Kennzahl | IST-Spalte | DANN-Spalte | Färbung DANN |
|----------|-----------|-------------|--------------|
| Stundenkonto (Balance) | `balance` | `balance_after` | `text-good` wenn ≥ 0, sonst `text-warn` |
| Freiwillig Ist | `voluntary_ist` | `voluntary_ist_after` | neutral `text-ink` |
| Freiwillig Soll | `voluntary_soll` | `voluntary_soll_after` | neutral `text-ink` |
| Freiwillig Delta | `voluntary_delta` | `voluntary_delta_after` | `text-warn` wenn < 0 |

Zusätzlich eine hervorgehobene Zeile „Vorgeschlagene Umbuchung: {proposed_hours}h"
(`proposed_hours = min(|balance|, voluntary_ist)`, D-55-03, backend-computed).

**Tabellen-Layout (empfohlen — Klärung offener UX-Frage, s.u.):** Drei-Spalten-Grid
`grid grid-cols-[1fr_auto_auto] gap-x-4 gap-y-1.5`. Header-Zeile: leer / „IST" / „DANN"
(`text-micro font-semibold uppercase text-ink-muted`, IST/DANN rechtsbündig). Datenzeilen:
Label (`text-small text-ink-soft`) + IST-Wert + DANN-Wert (beide `font-mono tabular-nums`,
rechtsbündig). Ein `→`-Glyph zwischen IST und DANN ist optional; das Grid-Alignment reicht.
`TupleRow` selbst ist nur label+value (zwei Spalten) — für die IST/DANN-Drei-Spalten-Optik
wird ein eigenes Grid gebaut, ABER Typo/Farb-Konventionen von `TupleRow`/`voluntary_stats_row`
übernommen.

**Footer** (via `Dialog { footer }`): zwei Buttons, rechtsbündig (`justify-end gap-2`):
- Reject: `BtnVariant::Danger` (`bg-surface text-bad border-bad`), Label `RebookingReject`.
- Approve: `BtnVariant::Primary` (`bg-accent text-accent-ink border-accent`), Label
  `RebookingApprove`. Primär (rechts außen).

Beide Buttons während in-flight-Request disabled (`disabled` → `opacity-50 cursor-not-allowed`),
verhindert Double-Approve auf FE-Seite (Backend-Race via state-conditional UPDATE, D-55-07
/ Pitfall 12; das FE-Disable ist nur UX-Politur, nicht die Autorität).

### Surface 3: Manual-Rebooking-Modal — `manual_rebooking_modal.rs` (F3, REB-MANUAL-01/02/03)

**Basis:** `Dialog`-Shell (`variant: Center`, `width: 460`), `title` =
`ManualRebookingModalTitle`, `subtitle` = Person-Name.
**Body (minimalistisch — D-55-05/06-Kontext, Empfehlung bestätigt):** drei Eingaben +
Vorschau, KEINE DANN-Duplikat-Rechnung (HR kennt den Kontext; die DANN-Tabelle ist dem
F5-Alert-Modal vorbehalten):

1. **Richtung** (`RebookingDirection`, D-55-06) — `select` mit zwei Optionen:
   `VolunteerToExtra` (Standard-Ausgleich, Default) / `ExtraToVolunteer` (Korrektur).
   Muster: `select.border.border-border.rounded-md.p-2` (mirror `absence_convert_modal.rs`).
2. **Menge** (Stunden) — `input type="number" min=0 step=0.5`,
   `border border-border rounded-md p-2`. Submit-Defense: `> 0` sonst inline `text-bad`-Error.
3. **Woche** (D-55-05) — ISO-Jahr + ISO-Woche, Default = aktuelle KW; HR kann
   retrospektiv wählen. Zwei `input type="number"` (Jahr / KW) ODER ein `week`-Picker;
   Empfehlung: zwei nummerierte Felder für WASM-Robustheit (Präzedenz `absence_convert_modal`
   nutzt getippte Inputs, kein `week`-Picker-Signal-Risiko / MOD-Pitfall D-25-06).
4. **Vorschau-Zeile** (REB-MANUAL-03): „{Richtung}: {Menge}h in KW {week}/{year}" —
   reine Text-Zusammenfassung der Eingaben, `text-small text-ink-muted`. Kein
   Backend-Roundtrip nötig; keine DANN-Rechnung.

**Footer:** Cancel (`BtnVariant::Secondary`, Label `RebookingCancel`) + Anlegen
(`BtnVariant::Primary`, Label `RebookingManualSubmit`).

**Fehlerzustand (D-55-05):** Bei 409 (UNIQUE-Slot `(sp, iso_year, iso_week)` belegt) zeigt
das Modal inline `RebookingErrorSlotTaken` als `text-bad text-small font-normal` unter der
Woche-Auswahl. Kein Auto-Overwrite, Modal bleibt offen, HR kann Woche ändern.

### Integration A: F3-Trigger in `page/employee_details.rs` (D-55-06)

**Ort:** TopBar / Action-Menu der Employee-Details-Seite — NICHT in der
Voluntary-Stats-Zeile (die bleibt reine Lese-Anzeige, D-55-06).
**Element:** ein Menu-Item bzw. `Btn` in der TopBar-Action-Zone, das `manual_rebooking_modal`
öffnet. Label `RebookingManualTrigger` (z.B. „Umbuchung anlegen"). HR-gated (nur sichtbar
wenn HR-Report-Kontext).
**Styling:** falls Nav-Menu-Item → `nav_item_class` / `admin_panel_item_class`-Präzedenz;
falls dedizierter Button → `Btn` `BtnVariant::Secondary` (Trigger ist neutral; der Primär-
Accent sitzt erst im Modal-Submit).

### Integration B: Banner-Einbindung in `page/employees.rs` (F5)

Der `RebookingAlertBanner` wird pro SalesPerson gerendert, dessen
`ShortEmployeeReportTO.has_pending_rebooking == true`. Reihenfolge: natürliche
Report-Reihenfolge. Klick öffnet das Suggestion-Modal mit `pending_rebooking_id`
(D-55-02).

---

## Copywriting Contract

### Neue i18n-Keys (alle drei Locales de/en/cs — Test-Gate `i18n_*_present_in_all_locales`)

Keys in `src/i18n/mod.rs` enum `Key`, Übersetzungen in `en.rs`, `de.rs`, `cs.rs`.

#### HR-Alert-Banner (F5)

| Key | De | En | Cs |
|-----|----|----|----|
| `RebookingAlertBannerText` | "Stundenkonto {balance} h bei {voluntary} h Freiwilligenarbeit — Ausgleich möglich." | "Balance {balance} h with {voluntary} h voluntary work — rebooking possible." | "Stav hodin {balance} h při {voluntary} h dobrovolné práce — vyrovnání možné." |
| `RebookingAlertBannerAction` | "Vorschlag prüfen →" | "Review suggestion →" | "Zkontrolovat návrh →" |

#### Suggestion-Modal (F5)

| Key | De | En | Cs |
|-----|----|----|----|
| `RebookingSuggestionModalTitle` | "Umbuchungs-Vorschlag" | "Rebooking suggestion" | "Návrh přeúčtování" |
| `RebookingColIst` | "Ist" | "Now" | "Nyní" |
| `RebookingColDann` | "Dann" | "After" | "Poté" |
| `RebookingRowBalance` | "Stundenkonto" | "Balance" | "Stav hodin" |
| `RebookingRowVoluntaryIst` | "Freiwillig Ist" | "Voluntary actual" | "Dobrovolně skutečnost" |
| `RebookingRowVoluntarySoll` | "Freiwillig Soll" | "Voluntary target" | "Dobrovolně cíl" |
| `RebookingRowVoluntaryDelta` | "Freiwillig Delta" | "Voluntary delta" | "Dobrovolně rozdíl" |
| `RebookingProposedHours` | "Vorgeschlagene Umbuchung: {hours} h" | "Proposed rebooking: {hours} h" | "Navržené přeúčtování: {hours} h" |
| `RebookingApprove` | "Bestätigen" | "Approve" | "Potvrdit" |
| `RebookingReject` | "Ablehnen" | "Reject" | "Odmítnout" |

#### Manual-Rebooking-Modal (F3)

| Key | De | En | Cs |
|-----|----|----|----|
| `ManualRebookingModalTitle` | "Manuelle Umbuchung" | "Manual rebooking" | "Ruční přeúčtování" |
| `RebookingManualTrigger` | "Umbuchung anlegen" | "Create rebooking" | "Vytvořit přeúčtování" |
| `RebookingDirectionLabel` | "Richtung" | "Direction" | "Směr" |
| `RebookingDirectionVolunteerToExtra` | "Freiwillig → Bezahlt" | "Voluntary → Paid" | "Dobrovolně → Placené" |
| `RebookingDirectionExtraToVolunteer` | "Bezahlt → Freiwillig" | "Paid → Voluntary" | "Placené → Dobrovolně" |
| `RebookingAmountLabel` | "Menge (Stunden)" | "Amount (hours)" | "Množství (hodiny)" |
| `RebookingWeekLabel` | "Woche (KW)" | "Week (CW)" | "Týden (KT)" |
| `RebookingYearLabel` | "Jahr" | "Year" | "Rok" |
| `RebookingPreview` | "{direction}: {hours} h in KW {week}/{year}" | "{direction}: {hours} h in CW {week}/{year}" | "{direction}: {hours} h v KT {week}/{year}" |
| `RebookingManualSubmit` | "Umbuchung anlegen" | "Create rebooking" | "Vytvořit přeúčtování" |
| `RebookingCancel` | "Abbrechen" | "Cancel" | "Zrušit" |

#### Fehler / Zustände

| Key | De | En | Cs |
|-----|----|----|----|
| `RebookingErrorSlotTaken` | "Für diese Woche existiert bereits eine Umbuchung. Bitte andere Woche wählen." | "A rebooking already exists for this week. Please choose another week." | "Pro tento týden již existuje přeúčtování. Zvolte prosím jiný týden." |
| `RebookingErrorAmount` | "Menge muss größer als 0 sein." | "Amount must be greater than 0." | "Množství musí být větší než 0." |
| `RebookingErrorGeneric` | "Umbuchung konnte nicht ausgeführt werden." | "Rebooking could not be completed." | "Přeúčtování se nepodařilo dokončit." |
| `RebookingSuggestionEmpty` | "Kein offener Vorschlag." | "No open suggestion." | "Žádný otevřený návrh." |

### Bestehende Keys wiederverwendet (unverändert)

| Key | Verwendung |
|-----|-----------|
| `Key::Hours` | Stunden-Suffix an allen numerischen Werten (IST/DANN, proposed_hours) |
| `Key::VoluntaryHoursIstPerWeek` / `VoluntaryHoursSoll` / `VoluntaryHoursDelta` | Referenz-Konsistenz für Voluntary-Labels (Suggestion-Modal darf diese reusen statt neue `RebookingRow*`-Keys, falls Wortlaut identisch gewünscht) |

### Primär-CTAs

| Oberfläche | Primär-CTA | Label-Key |
|-----------|------------|-----------|
| Manual-Modal | Umbuchung anlegen | `RebookingManualSubmit` |
| Suggestion-Modal | Vorschlag bestätigen | `RebookingApprove` |
| Employee-Details TopBar | Umbuchungs-Modal öffnen | `RebookingManualTrigger` |

### Empty / Zero-States

| Oberfläche | Bedingung | Verhalten / Copy |
|-----------|-----------|------------------|
| Alert-Banner | `has_pending_rebooking == false` bzw. Predicate false | Zeile wird NICHT gerendert (Zero-State = unsichtbar, wie die Overage-Sektion in Phase 24). Kein Leerzustands-Text. |
| Suggestion-Modal | Vorschlag beim Öffnen bereits weg (Race) | `RebookingSuggestionEmpty` als `text-small text-ink-muted`, Modal zeigt nur Close. |

### Error-States

| Oberfläche | Fehler | Copy (`text-bad text-small font-normal`) |
|-----------|--------|-------------------------------------------|
| Manual-Modal | 409 UNIQUE-Slot belegt (D-55-05) | `RebookingErrorSlotTaken` — inline unter Woche-Auswahl, Modal bleibt offen |
| Manual-Modal | Menge ≤ 0 (Submit-Defense) | `RebookingErrorAmount` — inline, kein Dispatch |
| Suggestion-Modal | Approve verliert Race (409, Pitfall 12) | `RebookingErrorGeneric` — inline; Modal danach schließen/refetchen |
| beide Modals | Netzwerk-/sonstiger Fehler | `RebookingErrorGeneric` |

### Destruktive Aktionen

| Aktion | Confirmation-Ansatz |
|--------|---------------------|
| **Reject** (HR-ALERT-03, D-55-07) | KEIN Confirm-Dialog. Reject ist NICHT löschend — es persistiert `state=Rejected` als Audit-Eintrag (D-55-04, kein Undo). Der `BtnVariant::Danger`-Style (`text-bad border-bad`) signalisiert die Ablehnung ausreichend; ein zweiter Bestätigungs-Dialog wäre Overhead (konsistent mit User-Präferenz für nicht-blockierendes inline Feedback). |
| **Approve** | KEIN Confirm-Dialog. Approve schreibt die Pair-ExtraHours + hebt den Batch auf `Approved`. Die IST/DANN-Vorschau IST bereits die Bestätigungs-Fläche (HR sieht das Ergebnis vor Klick). |
| **Manual-Umbuchung anlegen** | KEIN Confirm-Dialog. Die Vorschau-Zeile im Modal (REB-MANUAL-03) ist die Bestätigung; Klick auf „Anlegen" schreibt direkt. |

Hinweis: D-55-04 pinnt „kein Undo nach Approve/Reject". Die Copy suggeriert daher KEINE
Rückgängig-Option; die Vorschau/DANN-Anzeige übernimmt die Fehler-Vermeidung vor dem Klick.

---

## Interaction Contracts

### Alert-Banner (F5, HR-ALERT-01, D-55-07)

1. Bei jedem Render der Employee-Overview zeigt das FE pro SalesPerson mit
   `has_pending_rebooking == true` eine Warnzeile (backend-getriggert, keine FE-Arithmetik).
2. Klick auf die Zeile (oder Enter/Space bei Fokus) öffnet das Suggestion-Modal mit
   `pending_rebooking_id`.
3. Nach Approve ODER Reject schließt das Modal, das FE refetcht die Overview-Daten; die
   Zeile verschwindet, weil der Predicate `false` wird (D-55-07). Kein manuelles Dismiss.

### Suggestion-Modal (F5, HR-ALERT-02/03)

1. Öffnet mit backend-berechneten IST- und DANN-Werten (`RebookingSuggestionTO`). Kein
   FE-Rechnen.
2. Approve → `POST /rebooking-suggestions/{id}/approve`. Button während Request disabled.
   Bei Erfolg: Modal schließt, Overview refetch. Bei 409 (Race): `RebookingErrorGeneric`
   inline, dann Modal schließen + refetch.
3. Reject → `POST /rebooking-suggestions/{id}/reject`. Analog; bei Erfolg schließt Modal,
   Zeile verschwindet (Slot bleibt belegt bis nächste ISO-Woche, D-55-07).
4. ESC / Backdrop-Klick / × schließt ohne Aktion (Dialog-Shell-Standard, drag-safe via
   BackdropPress).

### Manual-Modal (F3, REB-MANUAL-01/02/03, D-55-05)

1. Öffnet aus TopBar/Action-Menu der Employee-Details-Seite (D-55-06), Default-Woche = aktuelle KW.
2. HR wählt Richtung, Menge, Woche. Vorschau-Zeile aktualisiert live aus den Eingaben.
3. Submit-Defense: Menge > 0, gültige Woche/Jahr → sonst inline `text-bad`, kein Dispatch.
4. „Anlegen" → `POST /rebooking/manual`. Bei Erfolg: Modal schließt, Report refetch,
   Umbuchung im Report sichtbar. Bei 409: `RebookingErrorSlotTaken` inline, Modal bleibt
   offen (kein Auto-Overwrite, D-55-05).
5. Submit-Button während Request disabled (Doppelklick-Schutz).

---

## Accessibility

- Alert-Banner: `role="button"`, `tabindex="0"`, `aria-label` mit Person + Kurz-Kontext;
  Enter/Space öffnet das Modal (nicht nur Maus-Klick).
- Modals nutzen die `Dialog`-Shell → `role="dialog"`, `aria-modal="true"`,
  `aria-labelledby="shifty-dialog-title"`, ESC-Dismiss, Body-Scroll-Lock, drag-safe
  Backdrop-Close (BackdropPress) — alles bereits in `dialog.rs` gekapselt, nicht neu bauen.
- Reject-/Approve-/Anlegen-Buttons haben sichtbaren Text-Label (kein Icon-only).
- IST/DANN-Tabelle: Zahlen mono/tabular für konsistentes Alignment; DANN-Färbung
  (`text-warn`/`text-good`) ist NICHT der einzige Bedeutungsträger — das Vorzeichen (`{:+.2}`)
  trägt die Information redundant (Farb-Blind-Sicherheit).
- Inline-Fehler stehen direkt am auslösenden Feld/Button (keine ARIA-Live-Region nötig).
- Alle neuen Strings in allen drei Locales (de/en/cs) — Test-Gate.

---

## Registry Safety

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| shadcn official | none | not applicable — kein shadcn (Rust/Dioxus) |
| externe Registries | none | not applicable |

Keine Third-Party-Component-Registry. Alle Komponenten sind hand-geschriebenes Dioxus RSX.
Vetting-Gate nicht anwendbar.

---

## Offene UX-Fragen (nicht in D-55-* entschieden — hier mit Empfehlung geklärt)

Diese zwei Punkte waren in CONTEXT.md `<open_questions>` an den Planner delegiert und
NICHT in D-55-* gepinnt. Der UI-Vertrag pinnt sie prescriptiv:

1. **F3-Modal-Vorschau-Umfang** (CONTEXT Open-Q 2): **minimalistisch** — Menge + Richtung
   + Woche + Text-Vorschau-Zeile. KEINE IST/DANN-Duplikat-Tabelle. Begründung: HR kennt
   den Kontext; die DANN-Rechnung ist dem F5-Alert-Modal vorbehalten (dort ist sie der
   Zweck). Konsistent mit CONTEXT-Empfehlung.
2. **IST/DANN-Tabellen-Shape** (Suggestion-Modal): eigenes **Drei-Spalten-Grid**
   (`grid-cols-[1fr_auto_auto]`) statt `TupleRow` (das nur 2 Spalten kann), aber mit den
   Typo/Farb-Konventionen von `TupleRow` + `voluntary_stats_row` (`font-mono tabular-nums`,
   `text-warn`/`text-good`, `text-small text-ink-soft`-Labels). Header-Zeile IST/DANN als
   `text-micro font-semibold uppercase text-ink-muted`.

Beide sind reine UX-Umfangs-Fragen ohne Datenrisiko (CONTEXT A3). Falls der Planner/User
abweichen will, sind es lokale Änderungen am Modal-Body.

---

## Implementation Notes for Executor

### File-Modification-Map (FE-Anteil)

| Datei | Änderung |
|-------|----------|
| `shifty-dioxus/src/i18n/mod.rs` | Neue `Key`-Varianten (siehe Copywriting) |
| `shifty-dioxus/src/i18n/{de,en,cs}.rs` | Übersetzungen für alle neuen Keys (3 Locales) |
| `shifty-dioxus/src/component/rebooking_alert_banner.rs` | NEU — Warn-Banner (mirror `warning_list.rs`), klickbar |
| `shifty-dioxus/src/component/rebooking_suggestion_modal.rs` | NEU — `Dialog`-Shell + IST/DANN-Grid + Approve/Reject-Footer |
| `shifty-dioxus/src/component/manual_rebooking_modal.rs` | NEU — `Dialog`-Shell + Richtung/Menge/Woche + Submit-Defense (mirror `absence_convert_modal.rs`) |
| `shifty-dioxus/src/component/mod.rs` | Exports der 3 neuen Komponenten |
| `shifty-dioxus/src/state/rebooking.rs` | NEU — thin `From<&…TO>`-Mapper, keine Arithmetik |
| `shifty-dioxus/src/loader.rs` | 4 Loader (submit_manual, load_pending, approve, reject) |
| `shifty-dioxus/src/page/employees.rs` | Banner-Einbindung im HR-Path |
| `shifty-dioxus/src/page/employee_details.rs` | F3-Trigger in TopBar/Action-Menu (D-55-06) |
| `shifty-dioxus/Dioxus.toml` | Proxy `/rebooking`, `/rebooking-suggestions` (Pitfall 14) |

### Kanonische Tailwind-Klassen-Strings für neue Elemente

**Alert-Banner-Container** (mirror `WarningList`, klickbar):
```
"border-l-[3px] border-warn bg-warn-soft rounded-md px-4 py-3 flex items-center justify-between gap-3 cursor-pointer hover:bg-surface-alt"
```

**Alert-Banner-Text** (Person betont):
```
Person: "text-body text-ink font-semibold"   Rest: "text-body text-ink"
```

**Alert-Banner-Action-Hinweis**:
```
"text-small text-warn font-semibold"
```

**IST/DANN-Grid-Container**:
```
"grid grid-cols-[1fr_auto_auto] gap-x-4 gap-y-1.5 items-baseline"
```

**IST/DANN-Header-Zellen** (IST/DANN rechtsbündig):
```
"text-micro font-semibold uppercase text-ink-muted text-right"
```

**IST/DANN-Label-Zelle**:
```
"text-small text-ink-soft"
```

**IST/DANN-Wert-Zelle** (mono, rechtsbündig; DANN ggf. + Farb-Suffix):
```
"font-mono tabular-nums text-right"           (neutral)
"font-mono tabular-nums text-right text-warn" (DANN Balance < 0 / Delta < 0)
"font-mono tabular-nums text-right text-good" (DANN Balance >= 0)
```

**proposed_hours-Zeile** (hervorgehoben):
```
"text-body font-semibold text-ink pt-2 mt-2 border-t border-border"
```

**Modal-Felder** (Richtung/Menge/Woche — mirror `absence_convert_modal.rs`):
```
label: "flex flex-col gap-1"
label-span: "text-small text-ink-muted"
input/select: "border border-border rounded-md p-2"
```

**Inline-Fehler** (409 / Menge / generisch):
```
"text-bad text-small font-normal mt-1"
```

**Vorschau-Zeile (Manual-Modal)**:
```
"text-small text-ink-muted"
```

**Buttons** — über `atoms::Btn` mit Variant:
```
Approve / Anlegen: BtnVariant::Primary   → "bg-accent text-accent-ink border-accent"
Reject:            BtnVariant::Danger    → "bg-surface text-bad border-bad"
Cancel:            BtnVariant::Secondary → "bg-surface text-ink border-border-strong"
```

### Bindende Prinzipien (Executor-Recap)

- **Fat Backend, Thin Client:** alle IST/DANN-Werte, `proposed_hours`, Alert-Predicate
  sind backend-computed. FE-State-Mapper (`state/rebooking.rs`) ist ein thin `From<&…TO>`
  OHNE Arithmetik.
- **Dialog-Shell wiederverwenden:** kein eigenes Backdrop/Header/Footer-Chrome. Für ein
  eigenes Backdrop (falls nötig) MUSS `BackdropPress` verwendet werden (BUG-03-Grep-Invariant
  in `dialog.rs`; neue Modal-Files ggf. in dessen `FILES`-Set eintragen).
- **Predicate strukturell testbar (Pitfall 11):** die Sichtbarkeit hängt an einem
  backend-berechneten DTO-Flag, nicht an FE-Logik → SSR-Component-Tests (wie
  `voluntary_stats_row.rs`) statt Dioxus-Browser-Tests.
- **Zwei-Weight-Kontrakt:** neue Oberflächen nur `font-normal` + `font-semibold`.
- **Accent nur für Primär-CTA/Approve/Active-Nav** — Banner bleibt `warn`, Reject `bad`.

---

## Checker Sign-Off

- [x] Dimension 1 Copywriting: PASS
- [x] Dimension 2 Visuals: PASS
- [x] Dimension 3 Color: PASS
- [x] Dimension 4 Typography: PASS (Empfehlung umgesetzt: aktiv-gesetzte-Größen-Klarstellung ergänzt)
- [x] Dimension 5 Spacing: PASS
- [x] Dimension 6 Registry Safety: PASS

**Approval:** approved 2026-07-23 (gsd-ui-checker, 6/6 PASS)
