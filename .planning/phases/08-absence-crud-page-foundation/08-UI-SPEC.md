---
phase: 8
slug: absence-crud-page-foundation
status: approved
shadcn_initialized: false
preset: none
created: 2026-05-07
reviewed_at: 2026-05-07
revision_count: 1
---

# Phase 8 — UI Design Contract

> Visual and interaction contract for `/absences` (Top-Level-Route mit CRUD gegen `/absence-period`). Generiert von gsd-ui-researcher, verifiziert von gsd-ui-checker. Alle Werte stammen aus dem bereits etablierten Design-System in `shifty-dioxus/input.css` + `tailwind.config.js`. Diese Phase fügt KEINE neuen Tokens hinzu — sie konsumiert das vorhandene System.

---

## Design System

| Property | Value |
|----------|-------|
| Tool | none (Dioxus 0.6.1 + Tailwind v3 mit token-basierten CSS-Variablen) |
| Preset | not applicable — shadcn ist nicht im Stack (Rust/WASM-Frontend, kein React) |
| Component library | none — eigene Atom-Bibliothek in `src/component/atoms/` und `src/component/form/` |
| Icon library | none — Inline-SVG-Glyphen oder Mono-Font-Glyphen (`font-mono`-Span im `Btn.icon`) |
| Font (sans) | Inter, system-ui, -apple-system, sans-serif (siehe `tailwind.config.js:fontFamily.sans`) |
| Font (mono) | "JetBrains Mono", ui-monospace, Menlo, monospace (für Datumsanzeigen, Tag-Werte) |

Quelle: `shifty-dioxus/tailwind.config.js`, `shifty-dioxus/input.css`.

---

## Spacing Scale

Tailwind-Default + projektspezifische Token-Anwendung. Die kanonische Skala besteht aus Vielfachen von 4 (Tailwind 1 = 4px). Phase 8 nutzt zusätzlich eine kleine, formal dokumentierte Menge an 2px-Stufen-Ausnahmen (10px, 12px, 14px), die aus dem Mockup-Pattern `absences.jsx` übernommen wurden — siehe **Exceptions** weiter unten.

| Token | Value | Tailwind-Klasse | Usage in dieser Phase |
|-------|-------|-----------------|------------------------|
| xs | 4px | `gap-1`, `p-1`, `m-1` | Icon-Gap im Btn (`mr-1`), Field-Label-zu-Input-Gap |
| sm | 8px | `gap-2`, `p-2` | Compact-Spacing zwischen verwandten Elementen, Card-Padding-Y |
| md | 16px | `gap-4`, `p-4`, `px-4`, `py-4` | Default Element-Spacing, Liste-Row-Padding-X (`px-4`), Modal-Backdrop-Padding |
| lg | 24px | `gap-6`, `p-6` | Card-Padding-X (`px-6`), Section-Gap zwischen Filter-Bar und Liste |
| xl | 32px | `gap-8`, `p-8` | (nicht aktiv genutzt in Phase 8) |
| 2xl | 48px | `p-12` | Empty-State-Padding (`py-12`) |
| 3xl | 64px | `p-16` | (nicht aktiv genutzt in Phase 8) |

**Component-Höhen** (aus etablierten Atoms):
- Form-Input-Höhe: `h-[34px]` (siehe `inputs.rs:12`) — bewusst 34px (nicht-Tailwind-Standard, aber `[34px]`-Notation erlaubt). Wird wiederverwendet, NICHT überschrieben.
- Button-Höhe ergibt sich aus `py-1.5` + `text-body` ≈ 32px — ebenfalls etabliert in `btn.rs:28`.

**Border-Radius** (aus `input.css` CSS-Variablen, exposed via Tailwind):
| Token | Value | Tailwind-Klasse | Usage |
|-------|-------|-----------------|-------|
| sm | 4px (`--r-sm`) | `rounded-sm` | UnavailabilityChip, Filter-Pill-Active-State |
| md | 6px (`--r-md`) | `rounded-md` | Buttons, Inputs, Cards-Inner, Modal-Stat-Boxes |
| lg | 10px (`--r-lg`) | `rounded-lg` | Modal-Panel, AbsencePage-Liste-Container, VacationEntitlementCard, Filter-Bar |

**Pill-Radius**: `rounded-full` (9999px) für `CategoryBadge`, `StatusPill`, Person-Filter-Pills, Avatar-Kreise.

### Spacing Exceptions (formale Ausnahme-Tabelle)

Die folgenden Werte sind KEINE Vielfachen von 4 und werden explizit als Ausnahmen geführt. Sie sind bewusst aus dem Mockup-Pattern `shifty-design/project/absences.jsx` übernommen und gelten nur für die genannten Slots; jede weitere Verwendung innerhalb Phase 8 (oder darüber hinaus) MUSS auf die Standard-Skala (4 / 8 / 16 / 24 / 32) zurückfallen.

| Token | Pixel | Used In | Reason |
|-------|-------|---------|--------|
| `gap-3` | 12px | Form-Modal-Grid (`AbsenceModal` Body, `grid-cols-2 gap-3`), Header-Section-Gap auf `AbsencePage` (`gap-3` zwischen Header / VacationEntitlementCard / StatsGrid / Filter-Bar / Liste) | 12px ist Vielfaches von 4 — kanonisch zulässig. Aufgeführt zur Vollständigkeit, weil `gap-3` zwischen `gap-2` (8px, zu eng für Modal-Felder) und `gap-4` (16px, zu luftig für Modal-Grid) sitzt. |
| `gap-2.5` / `p-2.5` / `py-2.5` | 10px | StatsGrid Card-Gap (`gap-2.5`), WarningList-Container in dense-Variante (`p-2.5`), AbsenceFilterBar Container-`py-2.5` & inner Group-Gap (`gap-2.5`) | Vielfaches-von-4 wäre entweder zu eng (8px) oder zu luftig (12px) für die kompakten Stat-Cards / Filter-Bar / dense Warning-Variante. Mockup-Vorlage `absences.jsx` nutzt 10px für genau diese Slots; Übernahme dient der Mockup-Treue der Filter-Leiste und der Stat-Card-Reihe. |
| `px-3.5` / `py-3.5` / `gap-3.5` | 14px | AbsenceList-Row-Grid (`gap-3.5`, `py-3.5`), AbsenceFilterBar Container-`px-3.5` | 12px wäre zu eng für die ~48px-Row-Zielhöhe der Liste (Avatar 22px + Person-Name + Description-Subline brauchen vertikale Luft); 16px wäre zu luftig und würde die Listendichte unter den Mockup-Wert drücken. Bewusst aus Mockup `absences.jsx` Row-Definition übernommen. |
| `px-0.5` / `p-0.5` | 2px | Pill-Group-Container im Filter (`bg-surface-alt p-0.5 rounded-md`), CategoryBadge-Padding-Y in `Sm`-Variante (`py-px`), Mini-Badge-Padding | 4px wäre für die Pill-Gruppe ein sichtbarer Rahmen statt einer 1-2px-Track-Andeutung; 2px ist die etablierte Tailwind-Konvention für "fast bündig" und stammt aus dem Mockup-Pattern für segmentierte Pill-Gruppen. |

**Begründung des Snap-NICHT-Approaches (warum nicht alles auf Vielfache-von-4 ziehen):** Phase 8 ist eine Mockup-Treue-Phase — der Mockup `absences.jsx` ist die UX-Vorlage, die der Designer zur Abnahme freigegeben hat. Snap auf 8/12/16 würde sichtbar von der Vorlage abweichen (Liste-Row-Höhe, Filter-Bar-Höhe, Stat-Card-Dichte). Die 2px-Stufen sind konsistent in der Tailwind-Toolchain (`gap-2.5`, `gap-3.5` sind erste-Klasse-Tokens, kein arbitrary-value-Hack) und werden in dieser Phase domain-lokal eingesetzt; sie sind nicht als globale Erweiterung der Spacing-Skala gedacht.

---

## Typography

Phase 8 fügt KEINE neuen Typografie-Tokens hinzu. Die Skala ist die kanonische Skala aus `tailwind.config.js:fontSize` (`text-body`, `text-small`, `text-micro`, `text-lg`, `text-h2`, `text-h1`, `text-display`); Body-Baseline (16px / 1.5) kommt aus `input.css:body`. Die scheinbar hohe Anzahl aktiver Größen ist **role-driven, nicht stylistisch**: jede Größe trägt eine eindeutige funktionale Rolle (Page-Title vs. Section-Heading vs. Body vs. Field-Label/Caption vs. Hero-Number), nicht ästhetische Variation. Display (32px) ist explizit als **Hero-Slot** klassifiziert — eine einmalige große Zahl in der `VacationEntitlementCard` Self-Variante — und damit funktional vom Body-Skala-System getrennt.

**Aktive Skala in Phase 8:** 4 reguläre Slots (`text-body` 14, `text-small` 12, `text-micro` 11, `text-h1` 22) + 1 Section-Slot (`text-lg` 16, nur Modal-Titel und Card-Section-Titel) + 1 Hero-Slot (`text-display` 32, nur VacationEntitlementCard). `text-h2` (18) ist deklariert, aber in Phase 8 nicht aktiv.

| Role | Size | Weight | Line Height | Tailwind-Klasse | Usage in Phase 8 |
|------|------|--------|-------------|-----------------|-------------------|
| Body (Default Run) | 14px | 400 | 20px (1.43) | `text-body` | Liste-Row-Inhalte, Form-Input-Text, Description, Vorschau-Text, Banner-Body |
| Caption / Label (Chrome) | 11px | 600 | 14px (1.27) | `text-micro` (uppercase, letter-spacing 0.06em) | Field-Label, Stat-Card-Header, Spalten-Header in der Liste, Vorschau-Header, Banner-Header, StatusPill-Text, Warning-Header |
| Small (Meta-Run) | 12px | 500 | 16px (1.33) | `text-small` | Sekundäre Meta-Info im normalen Run (Tag-Anzahl in Liste-Row, Filter-Counter "{n} von {m}", Hint-Texte unter Inputs, CategoryBadge-Text) |
| Section-Heading | 16px | 600 | 22px (1.375) | `text-lg` | Modal-Titel, Card-Section-Titel ("Dein Urlaubskonto") |
| Heading-M (h2) | 18px | 600 | 24px (1.33) | `text-h2` | (in Phase 8 nicht aktiv genutzt) |
| Page-Heading | 22px | 600 | 28px (1.27) | `text-h1` | Page-Titel "Abwesenheiten", VacationEntitlementCard Team-Variante Resturlaubs-Zahl |
| Hero (Display) | 32px | 700 | 38px (1.19) | `text-display` (mono) | VacationEntitlementCard Self-Variante Hero-Zahl `{remaining}/{entitled}` — einmaliger Hero-Slot, KEIN Body-Heading |

**Rolle `text-micro` (11px) vs `text-small` (12px) — die Trennung ist NICHT eine 1px-Größe-Variation, sondern ein Rollenwechsel:**

- `text-micro` (11px / 600 / **uppercase** / **letter-spacing 0.06em**) ist der **UI-Chrome-/Caption-Slot**: Labels, Spaltenheader, Pill-Texte, Banner-Header. Hier ist die Großschreibung + Letter-Spacing der visuelle Unterschied; die 11px-Größe ist nur sekundär.
- `text-small` (12px / 500 / **normal case** / **default letter-spacing**) ist der **Meta-Run-Slot** im normalen Schriftbild: Tag-Anzahl, Filter-Counter, Hint-Text, Badge-Inhalt. Hier dominiert die Lesbarkeit eines kurzen Sekundär-Runs.

Visuell und semantisch sind die beiden Tokens damit klar getrennt — `text-micro` ist nie "kleiner Body", sondern immer "all-caps caption". Eine Konsolidierung würde entweder Field-Labels Mixed-Case mit Letter-Spacing setzen (visuell falsch) oder Tag-Counter in Uppercase rendern ("3 TAGE" statt "3 Tage" — visuell falsch). Beide Tokens bleiben aktiv.

**Font-Weights** (genutzt in Phase 8): genau zwei aktive Gewichte plus ein Hero-Weight:
- 400 (regular) — Body-Text in Listen-Rows, Description, Hint-Texte (kommt automatisch aus `text-body`)
- 600 (semibold) — Headings (h1/lg), Labels (micro), CategoryBadge-Text, Person-Name in Liste, Filter-Pill-Active
- 700 (bold) — ausschließlich Hero-Display-Zahl in der `VacationEntitlementCard` (`text-display` bringt den 700-Weight per Token-Default mit; KEIN Body-Heading nutzt 700)

Mono-Klasse `font-mono` zusätzlich für numerische Werte (Datumsanzeigen `dd.mm.yyyy`, Tag-Anzahl, Resturlaubs-Zahl) — ist KEIN Weight-Token, sondern Family-Switch.

Quelle: `shifty-dioxus/tailwind.config.js:56-64`, `shifty-dioxus/input.css:11-19`.

---

## Color

Shifty hat bereits ein konsistentes 60/30/10-Farbsystem mit semantischen Tokens. Phase 8 konsumiert es 1:1.

| Role | Value (light) | Value (dark) | Tailwind-Klasse | Usage |
|------|---------------|--------------|-----------------|-------|
| Dominant (60%) | `#fbfbfc` (`--bg`) | `#0e1014` | `bg-bg` | Page-Body-Background |
| Surface | `#ffffff` (`--surface`) | `#16191f` | `bg-surface` | Listen-Container, Cards, Modal-Panel, Form-Inputs |
| Surface-Alt (30%) | `#f4f5f7` (`--surface-alt`) | `#1c2027` | `bg-surface-alt` | Liste-Spaltenheader, Vorschau-Box im Modal, VacationPerPerson-Bereich, Hover-State auf Listen-Rows |
| Surface-2 | `#eef0f4` (`--surface-2`) | `#232831` | `bg-surface-2` | StatusPill (`Geplant`/Plan-State), UnavailabilityChip Manual-Background |
| Border | `#e6e8ec` (`--border`) | `#2a2f39` | `border-border` | Card- und Modal-Border, Liste-Row-Trenner |
| Border-Strong | `#d0d3da` (`--border-strong`) | `#3a4151` | `border-border-strong` | Form-Input-Border, sekundäre Btn-Border |
| Ink | `#0e1117` (`--ink`) | `#eef0f4` | `text-ink` | Primärer Body-Text |
| Ink-Soft | `#3a4150` (`--ink-soft`) | `#b8bdc7` | `text-ink-soft` | Sekundärer Text (Btn-Ghost-Text, Field-Label) |
| Ink-Muted | `#6b7382` (`--ink-muted`) | `#7a8290` | `text-ink-muted` | Tertiärer Meta-Text (Hint, Counter, Status-Neutral) |
| Accent (10%) | `#3a4cd1` (`--accent`) | `#8b97ff` | `bg-accent` / `text-accent` | Primärer CTA "Neue Abwesenheit", StatusPill `Aktiv`, Filter-Action-Link |
| Accent-Ink | `#ffffff` (`--accent-ink`) | `#0e1014` | `text-accent-ink` | Text auf Accent-Background (CTA-Btn-Inhalt) |
| Accent-Soft | `#eaecfb` (`--accent-soft`) | `#232a4a` | `bg-accent-soft` | StatusPill `Aktiv`-Background, Focus-Ring (`form-input:focus`) |
| Good | `#0e7a4d` (`--good`) | `#4ed59a` | `text-good` / `bg-good` | Vacation-Kategorie-Akzent (Resturlaub-Hero-Zahl, VacationProgress-Bar) |
| Good-Soft | `#defaee` (`--good-soft`) | `#16322a` | `bg-good-soft` | Vacation-Kategorie-Hintergrund (Hero-Spalte VacationEntitlementCard, CategoryBadge-Vacation-Background) |
| Warn | `#a65a08` (`--warn`) | `#f0b766` | `text-warn` | SickLeave-Kategorie-Akzent, Warning-Text in WarningList, Resturlaub-low-Indicator (≤3 Tage) |
| Warn-Soft | `#fef0d6` (`--warn-soft`) | `#3a2a14` | `bg-warn-soft` | SickLeave-Kategorie-Hintergrund, WarningList-Container, Liste-Warning-Counter-Pill |
| Bad | `#b8281a` (`--bad`) | `#ef6a5b` | `text-bad` / `border-bad` | Self-Overlap-Validation-Error-Text, Field-Error-Text, Btn-Danger (Delete) |
| Bad-Soft | `#fde4e1` (`--bad-soft`) | `#3a1c18` | `bg-bad-soft` | Self-Overlap-Banner-Background, Version-Conflict-Banner-Background |
| Modal-Veil | `rgba(14,17,23,0.4)` | `rgba(0,0,0,0.6)` | `bg-modal-veil` (CSS-var) | Dialog-Backdrop |

**Accent reserved for** (10% Quote — explizite Liste, NICHT für alle interaktiven Elemente):
1. Primärer CTA "Neue Abwesenheit" (Header der Page) → `BtnVariant::Primary`
2. Modal-Save/Anlegen-Button → `BtnVariant::Primary`
3. StatusPill `Aktiv` (Background `accent-soft`, Text `accent`) — NUR für aktive Zeiträume
4. Form-Input-Focus-Ring (`.form-input:focus`) — automatisch via `input.css:155-159`
5. "Alle ({rows.length})" Toggle-Link in `VacationPerPersonList` (Text-Akzent, kein Background)
6. Reload-Button im Version-Conflict-Banner

**Accent NICHT verwenden für**:
- Nicht-Hover- oder Nicht-Active-Zustände der Liste (dort: `surface-alt` für Hover)
- Filter-Pill-Active (dort: `surface` auf `surface-alt`-Strip → kein Akzent, nur Kontrast)
- StatusPill `Geplant`/`Beendet` (dort: `surface-2`/`surface-alt` mit `ink-soft`/`ink-muted`)

**Kategorie-Farb-Mapping** (semantische Domain-Farben — KEIN zusätzliches Brand-Token, sondern Wiederverwendung der `good`/`warn`/`ink-muted`-Tokens):

| Kategorie | Akzent-Farbe | Soft-Background | Tailwind-Klassen |
|-----------|--------------|-----------------|------------------|
| `Vacation` | `good` (#0e7a4d) | `good-soft` (#defaee) | `text-good bg-good-soft` |
| `SickLeave` | `warn` (#a65a08) | `warn-soft` (#fef0d6) | `text-warn bg-warn-soft` |
| `UnpaidLeave` | `ink-muted` (#6b7382) | `surface-2` (#eef0f4) | `text-ink-muted bg-surface-2` |

Diese drei Kombinationen sind das Token-Set für `CategoryBadge` und MÜSSEN über statische `match`-Arme (nicht dynamische `format!`-Strings) in den Tailwind-Build kommen — siehe `tailwind.config.js:7` Hinweis. Falls ein dynamischer Pfad nötig wird, MÜSSEN die Klassen in `safelist` ergänzt werden.

**Destructive Color Path**:
- `Btn::Danger` (Delete im Modal): `text-bad border-bad bg-surface` — siehe `btn.rs:38`
- Self-Overlap-Banner (422 inline): Linker `border-l-[3px] border-bad`, Background `bg-bad-soft`, Header-Text `text-bad`
- Version-Conflict-Banner (409 — D-08): Linker `border-l-[3px] border-warn`, Background `bg-warn-soft`, Header-Text `text-warn` (KEIN bad — der Konflikt ist nicht destruktiv, nur reload-pflichtig)

---

## Copywriting Contract

Alle Strings sind i18n-Keys in `src/i18n/mod.rs` unter Comment-Block `// Absence management`. Sofortige Befüllung in De / En / Cs in der gleichen Phase (D-13). Locale-Tabelle sortiert nach Use-Site.

### Page-Level

| Element | i18n-Key | De | En | Cs |
|---------|----------|----|----|----|
| Page-Titel | `AbsencePageTitle` | Abwesenheiten | Absences | Nepřítomnosti |
| Page-Subtitle | `AbsencePageSubtitle` | Urlaub, Krankheit und unbezahlte Freistellung als Zeiträume. Stunden pro Tag werden aus dem gültigen Arbeitsvertrag abgeleitet. | Vacation, sick leave and unpaid leave as date ranges. Hours per day are derived from the active employment contract. | Dovolená, nemoc a neplacené volno jako období. Hodiny za den vycházejí z platné pracovní smlouvy. |
| Menü-Eintrag | `AbsenceMenuLabel` | Abwesenheiten | Absences | Nepřítomnosti |

### Primary CTA

| Element | i18n-Key | De | En | Cs |
|---------|----------|----|----|----|
| Primary CTA (Page-Header-Btn) | `AbsenceNewBtn` | Neue Abwesenheit | New absence | Nová nepřítomnost |
| Modal-Submit (Create) | `AbsenceModalCreateBtn` | Anlegen | Create | Vytvořit |
| Modal-Submit (Edit) | `AbsenceModalSaveBtn` | Speichern | Save | Uložit |
| Modal-Cancel | `AbsenceModalCancelBtn` | Abbrechen | Cancel | Zrušit |
| Modal-Delete | `AbsenceModalDeleteBtn` | Löschen | Delete | Smazat |

### Empty State

| Element | i18n-Key | De | En | Cs |
|---------|----------|----|----|----|
| Empty-State-Heading (HR + Filter aktiv) | `AbsenceEmptyFilterHeading` | Keine Treffer | No results | Žádné výsledky |
| Empty-State-Body (HR + Filter aktiv) | `AbsenceEmptyFilterBody` | Keine Abwesenheiten passen zum aktuellen Filter. Filter zurücksetzen oder neue Abwesenheit anlegen. | No absences match the current filter. Reset filters or create a new absence. | Žádné nepřítomnosti neodpovídají filtru. Resetuj filtr nebo vytvoř novou nepřítomnost. |
| Empty-State-Heading (Employee + 0 Einträge) | `AbsenceEmptySelfHeading` | Noch keine Abwesenheiten | No absences yet | Zatím žádné nepřítomnosti |
| Empty-State-Body (Employee + 0 Einträge) | `AbsenceEmptySelfBody` | Lege deine erste Abwesenheit an (Urlaub, Krankheit oder unbezahlte Freistellung). | Create your first absence (vacation, sick leave or unpaid leave). | Vytvoř svou první nepřítomnost (dovolenou, nemoc nebo neplacené volno). |

### Form Labels & Hints

| Element | i18n-Key | De | En | Cs |
|---------|----------|----|----|----|
| Field-Label Mitarbeiter | `AbsenceFieldEmployee` | Mitarbeiter | Employee | Zaměstnanec |
| Field-Label Kategorie | `AbsenceFieldCategory` | Kategorie | Category | Kategorie |
| Field-Label Von | `AbsenceFieldFrom` | Von | From | Od |
| Field-Label Bis | `AbsenceFieldTo` | Bis (inklusiv) | To (inclusive) | Do (včetně) |
| Field-Label Beschreibung | `AbsenceFieldDescription` | Beschreibung | Description | Popis |
| Field-Hint Beschreibung | `AbsenceFieldDescriptionHint` | Optional — z. B. Reiseziel oder Anmerkung. | Optional — e.g. travel destination or note. | Volitelné — např. místo cesty nebo poznámka. |
| Modal-Subtitle (Create) | `AbsenceModalCreateSubtitle` | Ganztägiger Zeitraum. Stunden werden aus dem Vertrag abgeleitet. | Full-day range. Hours are derived from the contract. | Celodenní období. Hodiny vycházejí ze smlouvy. |
| Modal-Subtitle (Edit) | `AbsenceModalEditSubtitle` | Änderungen werden mit optimistischem Locking gespeichert. | Changes are saved with optimistic locking. | Změny se ukládají s optimistickým zamykáním. |
| Vorschau-Header | `AbsencePreviewHeader` | Vorschau | Preview | Náhled |
| Vorschau-Footer | `AbsencePreviewFooter` | Feiertage im Bereich werden mit 0 h verrechnet. Stunden pro Tag stammen aus dem am jeweiligen Tag gültigen Arbeitsvertrag. | Holidays in the range count as 0 h. Hours per day come from the contract active on that day. | Svátky v období se počítají jako 0 h. Hodiny za den vycházejí ze smlouvy platné v daný den. |

### Categories (Pills + Filter)

| Element | i18n-Key | De | En | Cs |
|---------|----------|----|----|----|
| Vacation-Label | `AbsenceCategoryVacation` | Urlaub | Vacation | Dovolená |
| SickLeave-Label | `AbsenceCategorySickLeave` | Krankheit | Sick leave | Nemoc |
| UnpaidLeave-Label | `AbsenceCategoryUnpaidLeave` | Unbezahlt | Unpaid leave | Neplacené |
| Filter-Group-Label "Kategorie" | `AbsenceFilterCategoryLabel` | Kategorie | Category | Kategorie |
| Filter-Pill "Alle" | `AbsenceFilterCategoryAll` | Alle | All | Všechny |
| Filter-Group-Label "Person" | `AbsenceFilterPersonLabel` | Person | Person | Osoba |
| Filter-Person "Alle Personen" | `AbsenceFilterPersonAll` | Alle Personen | All people | Všichni lidé |
| Filter-Group-Label "Status" | `AbsenceFilterStatusLabel` | Status | Status | Stav |
| Filter-Status "Alle" | `AbsenceFilterStatusAll` | Alle | All | Všechny |

### Status (clientseitig berechnet, D-06)

| Element | i18n-Key | De | En | Cs |
|---------|----------|----|----|----|
| Status `Aktiv` | `AbsenceStatusActive` | Aktiv | Active | Aktivní |
| Status `Geplant` | `AbsenceStatusPlanned` | Geplant | Planned | Plánováno |
| Status `Beendet` | `AbsenceStatusFinished` | Beendet | Finished | Ukončeno |

### Liste-Spaltenheader

| Element | i18n-Key | De | En | Cs |
|---------|----------|----|----|----|
| Spalte Mitarbeiter | `AbsenceColEmployee` | Mitarbeiter | Employee | Zaměstnanec |
| Spalte Zeitraum | `AbsenceColRange` | Zeitraum | Range | Období |
| Spalte Kategorie | `AbsenceColCategory` | Kategorie | Category | Kategorie |
| Spalte Status | `AbsenceColStatus` | Status | Status | Stav |
| Spalte Hinweise | `AbsenceColWarnings` | Hinweise | Warnings | Upozornění |
| Tag-Singular | `AbsenceDayUnit` | Tag | day | den |
| Tag-Plural | `AbsenceDaysUnit` | Tage | days | dny |

### VacationEntitlementCard

| Element | i18n-Key | De | En | Cs |
|---------|----------|----|----|----|
| Hero-Header (eigener User) | `VacationEntitlementHero` | Urlaubsanspruch {year} | Vacation entitlement {year} | Nárok na dovolenou {year} |
| Hero-Sublabel | `VacationDaysRemaining` | Tage verbleibend | days remaining | dnů zbývá |
| Card-Title (Self) | `VacationCardSelfTitle` | Dein Urlaubskonto | Your vacation balance | Tvé saldo dovolené |
| Card-Subtitle (Self) | `VacationCardSelfSubtitle` | Anspruch aus Vertrag + Übertrag aus dem Vorjahr. | Entitlement from contract + carryover from previous year. | Nárok ze smlouvy + převod z předchozího roku. |
| Card-Title (Team / HR) | `VacationCardTeamTitle` | Urlaubsanspruch Team · {count} Personen | Vacation entitlement team · {count} people | Nárok na dovolenou tým · {count} osob |
| Card-Subtitle (Team / HR) | `VacationCardTeamSubtitle` | Summe über alle bezahlten Mitarbeiter. | Sum across all paid employees. | Součet za všechny placené zaměstnance. |
| Stat Vertrag | `VacationStatContract` | Vertrag | Contract | Smlouva |
| Stat Übertrag | `VacationStatCarryover` | Übertrag '{year-1}' | Carryover '{year-1}' | Převod '{year-1}' |
| Stat Genommen | `VacationStatUsed` | Genommen | Used | Vyčerpáno |
| Stat Beantragt | `VacationStatPending` | Beantragt | Pending | Čeká |
| Stat Verbleibend | `VacationStatRemaining` | Verbleibend | Remaining | Zbývá |
| Per-Person Section-Header | `VacationPerPersonHeader` | Pro Person · sortiert nach verbleibenden Tagen | Per person · sorted by days remaining | Podle osoby · seřazeno podle zbývajících dnů |
| Toggle "Alle" | `VacationPerPersonShowAll` | Alle ({count}) | All ({count}) | Všechny ({count}) |
| Toggle "Weniger" | `VacationPerPersonShowLess` | Weniger | Less | Méně |

### Statistik-Cards

| Element | i18n-Key | De | En | Cs |
|---------|----------|----|----|----|
| Card "Krankheitstage {year}" | `AbsenceStatSickLeaveDays` | Krankheitstage {year} | Sick days {year} | Dny nemoci {year} |
| Card "Unbezahlt {year}" | `AbsenceStatUnpaidDays` | Unbezahlt {year} | Unpaid {year} | Neplacené {year} |
| Card "Aktive Abwesenheiten" | `AbsenceStatActive` | Aktive Abwesenheiten | Active absences | Aktivní nepřítomnosti |

### Errors & Warnings

| Element | i18n-Key | De | En | Cs |
|---------|----------|----|----|----|
| Inline-Error: Endedatum vor Startdatum | `AbsenceErrorRangeInverted` | Enddatum liegt vor Startdatum | End date is before start date | Konec je před začátkem |
| Self-Overlap-Banner-Header (422) | `AbsenceErrorSelfOverlapHeader` | Selbst-Überlappung | Self-overlap | Vlastní překryv |
| Self-Overlap-Banner-Body (422) | `AbsenceErrorSelfOverlapBody` | {category}-Eintrag von {from} bis {to} überschneidet sich. Bitte Zeitraum oder Kategorie anpassen. | A {category} entry from {from} to {to} overlaps. Please adjust the range or category. | Položka {category} od {from} do {to} se překrývá. Uprav období nebo kategorii. |
| Version-Conflict-Banner-Header (409, D-08) | `AbsenceErrorVersionConflictHeader` | Eintrag wurde anderswo geändert | Entry changed elsewhere | Položka byla změněna jinde |
| Version-Conflict-Banner-Body (409, D-08) | `AbsenceErrorVersionConflictBody` | Bitte erneut laden, dann nochmal speichern. Deine Eingabe bleibt erhalten. | Please reload, then save again. Your input is preserved. | Načti znovu a ulož. Tvé zadání zůstane zachováno. |
| Version-Conflict-Reload-Btn | `AbsenceErrorVersionConflictReload` | Erneut laden | Reload | Znovu načíst |
| Generic-Network-Error | `AbsenceErrorNetwork` | Netzwerkfehler. Bitte erneut versuchen. | Network error. Please try again. | Chyba sítě. Zkus to znovu. |
| Forward-Warning-Header (Singular) | `AbsenceWarningHeaderSingular` | Hinweis · 1 Konflikt (nicht blockierend) | Notice · 1 conflict (non-blocking) | Upozornění · 1 konflikt (neblokující) |
| Forward-Warning-Header (Plural) | `AbsenceWarningHeaderPlural` | Hinweis · {count} Konflikte (nicht blockierend) | Notice · {count} conflicts (non-blocking) | Upozornění · {count} konfliktů (neblokující) |
| Warning-Acknowledge-Btn | `AbsenceWarningAcknowledgeBtn` | Verstanden | Got it | Rozumím |
| Warning-Text: absence_overlaps_booking | `AbsenceWarningOverlapsBooking` | Bestehende Buchung am {date} überschneidet sich mit dieser Abwesenheit. | Existing booking on {date} overlaps with this absence. | Existující rezervace dne {date} se s touto nepřítomností překrývá. |
| Warning-Text: absence_overlaps_manual_unavailable | `AbsenceWarningOverlapsManual` | Manuell als unverfügbar markierter Tag überschneidet sich. Nach dem Cutover ist dieser Eintrag redundant. | Manually marked unavailable day overlaps. After cutover this entry is redundant. | Den ručně označený jako nedostupný se překrývá. Po cutoveru je záznam nadbytečný. |

### Destructive Confirmation (D-07)

| Element | i18n-Key | De | En | Cs |
|---------|----------|----|----|----|
| Delete-Dialog-Title | `AbsenceDeleteConfirmTitle` | Abwesenheit löschen? | Delete absence? | Smazat nepřítomnost? |
| Delete-Dialog-Body | `AbsenceDeleteConfirmBody` | Soft-Delete — der Eintrag bleibt für Audit-Logs erhalten, wird aber nicht mehr in Reports und im Shiftplan berücksichtigt. | Soft-delete — the entry stays in audit logs but is no longer counted in reports or shown in the shiftplan. | Měkké smazání — položka zůstane v auditních záznamech, ale nebude se počítat v reportech ani zobrazovat v plánu směn. |
| Delete-Dialog-Confirm-Btn | `AbsenceDeleteConfirmBtn` | Löschen | Delete | Smazat |
| Delete-Dialog-Cancel-Btn | `AbsenceDeleteCancelBtn` | Abbrechen | Cancel | Zrušit |

### Filter-Toggle "Vergangene anzeigen"

| Element | i18n-Key | De | En | Cs |
|---------|----------|----|----|----|
| Toggle-Label | `AbsenceFilterShowPast` | Vergangene anzeigen | Show past | Zobrazit minulé |
| Counter "{n} von {m}" | `AbsenceFilterCounter` | {n} von {m} | {n} of {m} | {n} z {m} |

### Standard-CTA-Pattern (Zusammenfassung)

| Element | Copy (De) |
|---------|-----------|
| Primary CTA | "Neue Abwesenheit" (Verb + Noun, im Header der Page) |
| Empty state heading | "Noch keine Abwesenheiten" (Self) / "Keine Treffer" (HR + Filter) |
| Empty state body | Self: "Lege deine erste Abwesenheit an (Urlaub, Krankheit oder unbezahlte Freistellung)." / HR: "Keine Abwesenheiten passen zum aktuellen Filter." |
| Error state | Self-Overlap (422): "Selbst-Überlappung — bitte Zeitraum oder Kategorie anpassen." / Version-Conflict (409): "Eintrag wurde anderswo geändert. Bitte erneut laden, dann nochmal speichern." |
| Destructive confirmation | "Abwesenheit löschen? Soft-Delete — der Eintrag bleibt für Audit-Logs erhalten, wird aber nicht mehr in Reports und im Shiftplan berücksichtigt." |

---

## Component Inventory (Phase 8)

Verbindlich für Planner und Executor. Wiederverwendung > Neuerstellung.

### Wiederverwendet (KEINE Neuimplementierung)

| Komponente | Pfad | Verwendung in Phase 8 |
|------------|------|------------------------|
| `Dialog` | `src/component/dialog.rs` | Center-Variante für `AbsenceModal` (Width 520) und Confirm-Variante (Width ~360) für Delete-Confirmation |
| `Btn` (Primary/Ghost/Danger) | `src/component/atoms/btn.rs` | Save = Primary, Cancel = Ghost, Delete = Danger, Reload = Ghost mit `text-warn`-Override-Class? Nein — Reload bleibt `Ghost` ohne Override (Reload ist nicht destruktiv). |
| `TextInput` (mit `input_type="date"`) | `src/component/form/inputs.rs` | Datum-Range-Felder Von / Bis (D-05 — KEIN neues `RangePicker`-Atom) |
| `TextInput` (mit `input_type="text"`) | `src/component/form/inputs.rs` | (nicht direkt — Description nutzt Textarea) |
| `SelectInput` | `src/component/form/inputs.rs` | Kategorie-Dropdown im Modal (Alternative zu Pill-Group), Person-Filter-Dropdown in HR-Filter-Bar |
| `TextareaInput` | `src/component/form/inputs.rs` | Description-Feld |
| `Field` | `src/component/form/field.rs` | Wrapper für jedes Form-Feld; `error`-Slot für Cross-Field-Validation am Bis-Feld |
| `PersonChip` | `src/component/atoms/person_chip.rs` | (im Mockup als Avatar-Kreis genutzt — Plan-Phase entscheidet, ob `PersonChip` direkt passt oder ein lokaler `PersonAvatar` reicht) |
| Auth-Helper | `src/state/auth_info.rs` `has_privilege("hr")` | HR vs Employee-Sicht-Auswahl (D-09) |

### Neu in Phase 8 (Component-Layer)

Diese Komponenten werden **innerhalb der Page-Datei oder als Submodule der Page** implementiert (nicht als wiederverwendbare Atoms in `src/component/`), weil sie domain-spezifisch sind und in keiner anderen Phase wiederverwendet werden (D-01-Scope):

| Komponente | Vorgesehener Pfad | Inputs | Visuelle Notes |
|------------|-------------------|--------|----------------|
| `AbsencePage` | `src/page/absences.rs` | (Router-Provided, nutzt `AUTH_INFO` GlobalSignal) | Layout: `padding-md`, `gap-3` (12px) zwischen Sektionen, Sektionen = Header → VacationEntitlementCard (optional) → 3-Stat-Cards-Grid → Filter-Bar → Liste-Container |
| `AbsenceModal` | inline in `src/page/absences.rs` oder `src/component/absence_modal.rs` | `open: bool`, `mode: ModalMode { Create, Edit(AbsencePeriod) }`, `lock_person: bool`, `default_person_id: Option<Uuid>` | Center-Dialog Width 520, Grid `grid-cols-2 gap-3` (12px); Footer: links Delete-Btn (nur Edit-Mode), Spacer, rechts Cancel + Save |
| `WarningList` | inline-Helper in `src/page/absences.rs` | `warnings: &[WarningTO]`, optional `dense: bool` | Linker `border-l-[3px] border-warn`, Background `bg-warn-soft`, `rounded-md`, `p-3` (12px) standard / `p-2.5` (10px, **Spacing-Exception**) wenn `dense=true`, Header `text-micro text-warn`, Liste `text-body text-ink` mit `pl-4` (16px Bullet-Indent) |
| `CategoryBadge` | inline-Helper in `src/page/absences.rs` | `category: AbsenceCategoryTO`, optional `size: BadgeSize { Sm, Md }` | Pill (`rounded-full`), Background+Text aus Kategorie-Farb-Mapping, Dot 7×7px `rounded-full` als Indicator-Prefix, `gap-1`, `px-2 py-0.5` (md) bzw. `px-1.5 py-px` (sm — `py-px` ist 1px-Spacing-Exception für Mini-Variante), `text-small font-semibold` |
| `StatusPill` | inline-Helper in `src/page/absences.rs` | `status: AbsenceStatus { Active, Planned, Finished }` | Pill (`rounded-full`), `text-micro font-semibold`, `px-2 py-0.5`. Aktiv: `bg-accent-soft text-accent`. Geplant: `bg-surface-2 text-ink-soft`. Beendet: `bg-surface-alt text-ink-muted`. |
| `VacationEntitlementCard` | inline-Helper in `src/page/absences.rs` (oder `src/component/vacation_entitlement_card.rs` falls Plan-Phase die Wiederverwendung antizipiert) | `summary: VacationBalance` (Self oder Team-Aggregat) | Card `bg-surface border border-border rounded-lg overflow-hidden`. Self-Variante: 2-Spalten-Grid `[180px-240px] [1fr]`, Hero-Spalte `bg-good-soft`, Hero-Zahl `text-display font-mono text-good`. Team-Variante: 1-Spalten-Layout, Resturlaubs-Zahl `text-h1 font-mono text-good`. Progress-Bar: `h-2 rounded-full bg-surface-alt`, Used = `bg-good`, Pending = `bg-good opacity-40` mit diagonalem Stripe-Pattern (45deg, transparent/white). |
| `VacationPerPersonList` | inline-Helper in `src/page/absences.rs` | `rows: Vec<VacationPerPersonRow>` (sortiert nach `remaining` aufsteigend, Initial-Limit 4, Toggle "Alle ({n})") | Section innerhalb `VacationEntitlementCard` (HR-only). `border-t border-border bg-surface-alt`, Header `text-micro`. Grid: `grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-2`. Kachel: `bg-surface border border-border rounded-md p-2 px-3`, Avatar 22×22 `rounded-full`, Name `text-body font-semibold truncate`, Resturlaub `font-mono text-body font-bold`, Mini-Progress-Bar `h-1 rounded-full`. Low-Indicator (`remaining ≤ 3`): Resturlaub und Progress-Bar in `text-warn`/`bg-warn`. |
| `AbsenceList` (Liste-Container) | inline in `src/page/absences.rs` | `rows: Vec<AbsencePeriod>`, `is_hr: bool` | Container `bg-surface border border-border rounded-lg overflow-hidden`. Spaltenheader: `bg-surface-alt border-b border-border px-4 py-2 text-micro text-ink-muted`. Row: 5-Spalten-Grid `[1.5fr 170px 140px 90px 70px] gap-3.5 px-4 py-3.5` (`gap-3.5` und `py-3.5` = 14px **Spacing-Exception**), Hover `bg-surface-alt`. Trenner: `border-t border-border` außer Erste. |
| `AbsenceFilterBar` | inline in `src/page/absences.rs` | `filter: FilterState`, `is_hr: bool`, `total: usize`, `filtered: usize` | Container `bg-surface border border-border rounded-lg px-3.5 py-2.5 flex flex-wrap items-center gap-2.5` (`px-3.5` = 14px, `py-2.5` = 10px, `gap-2.5` = 10px — alle **Spacing-Exceptions**). Kategorie-Filter als Pill-Group auf `bg-surface-alt p-0.5 rounded-md` (`p-0.5` = 2px **Spacing-Exception**). Person-Dropdown (HR-only) als `SelectInput`. Status-Dropdown als `SelectInput`. Trennlinie zwischen Gruppen: `w-px h-[22px] bg-border mx-1`. "Vergangene anzeigen"-Checkbox rechts mit `ml-auto`. Counter "{n} von {m}" `text-small text-ink-muted`. |
| `StatsGrid` (3-Card-Reihe) | inline in `src/page/absences.rs` | `stats: AbsenceStats` (sick days year, unpaid days year, active count) | Grid: `grid-cols-[repeat(auto-fit,minmax(160px,1fr))] gap-2.5` (`gap-2.5` = 10px **Spacing-Exception** — bewusst dichter als `gap-3` damit die drei Cards in der Default-Viewport-Breite eng zusammensitzen). Card: `bg-surface border border-border rounded-md p-3`, Header `text-micro text-ink-muted`, Wert `text-h1 font-mono`. |
| `VersionConflictBanner` (D-08) | inline-Helper in `src/page/absences.rs` | `on_reload: EventHandler<()>` | Banner im Modal-Body (oberhalb des Form-Grids). `border-l-[3px] border-warn bg-warn-soft rounded-md p-3 flex items-center justify-between gap-3`. Header `text-micro text-warn font-bold`, Body `text-body text-ink`, Reload-Btn rechts (`Btn::Ghost`). |
| `SelfOverlapBanner` (D-11) | inline-Helper in `src/page/absences.rs` | `error_data: SelfOverlapPayload { category, from, to }` | Banner im Modal-Body unterhalb Range-Felder, oberhalb Vorschau. `border-l-[3px] border-bad bg-bad-soft rounded-md p-3`. Header `text-micro text-bad font-bold uppercase`, Body `text-body text-ink`. KEIN globaler Toast (D-11). |

### Out-of-Scope für Phase 8 (verwiesen auf andere Phasen)

| Komponente | Phase | Nicht in Phase 8 implementieren |
|------------|-------|----------------------------------|
| `UnavailabilityChip` | Phase 10 | Mockup-Helper für Shiftplan-Wochen-View |
| Deprecation-Banner für Legacy-extra_hours | Phase 11 | `403 ExtraHoursCategoryDeprecatedErrorTO`-Toast/Banner |
| Booking-Reverse-Warning-Confirm-Dialog | Phase 9 | `BookingCreateResultTO.warnings[]`-Confirm |

---

## Interaction Contract

### Cross-Field-Validation (D-05)

- Bis-Datum < Von-Datum → `Field.error` am Bis-Feld mit `AbsenceErrorRangeInverted`-Key. Save-Button DISABLED.
- Beide Felder leer → Save-Button DISABLED.
- Validation läuft client-side bei jeder `oninput`-Änderung (sofort sichtbar).

### Status-Berechnung (D-06)

```text
status = if to_date < today        → Beendet (Tone: Neutral)
         else if from_date > today → Geplant  (Tone: Plan)
         else                      → Aktiv    (Tone: Active)
```

`today` = `chrono::Local::now().date_naive()`. Status ist KEIN Backend-Feld; rein clientseitig berechnet. Filter-Status filtert nach derselben Funktion.

### Modal-Mode-Transitions

```text
Liste-Row-Click → Modal mit mode = Edit(absence)
"Neue Abwesenheit"-Btn → Modal mit mode = Create
Esc / Backdrop-Click / Cancel-Btn → Modal-Close ohne Speichern (Form-State verworfen)
Save-Btn → POST oder PUT je nach mode
   200/201 mit warnings[] → WarningList rendern, Save-Btn → "Verstanden" (AbsenceWarningAcknowledgeBtn) → Modal-Close
   200/201 ohne warnings → Modal-Close
   422 Self-Overlap → SelfOverlapBanner inline rendern, Modal bleibt offen, Form-State erhalten
   409 Version-Conflict → VersionConflictBanner inline rendern, Modal bleibt offen, Form-State erhalten
   andere Fehler → Generic-Network-Error-Toast (FUTURE: Toast-System ist nicht in Phase 8 — fallback ist `text-bad` Hint im Modal-Footer)
Delete-Btn (nur Edit-Mode) → öffnet zweite Confirm-Dialog (separater Center-Dialog, Width ~360)
   Confirm "Löschen" → DELETE → Modal-Close
   Confirm "Abbrechen" → bleibt im Edit-Modal
```

### Forward-Warnings-Flow (D-12)

Nach erfolgreichem POST/PUT mit `AbsencePeriodCreateResultTO.warnings[].len() > 0`:

1. Form-Felder werden DISABLED (visuell `opacity-60`).
2. `WarningList` wird unten im Modal-Body angezeigt.
3. Submit-Btn-Beschriftung wechselt von "Anlegen"/"Speichern" zu `AbsenceWarningAcknowledgeBtn` ("Verstanden").
4. User-Click auf "Verstanden" → Modal-Close. Daten sind bereits persistiert (Backend hat 201 zurückgegeben).
5. KEINE zweite Speicher-Aktion. Acknowledge ist rein UI.

### HR vs Employee-Sicht (D-09, D-10)

- Menü-Eintrag "Abwesenheiten" sichtbar für ALLE eingeloggten User in `top_bar.rs`.
- Page-Load: `let is_hr = AUTH_INFO.read().has_privilege("hr");`
- `is_hr = true` (HR):
  - Filter-Bar zeigt Person-Dropdown.
  - Liste zeigt alle Mitarbeiter (mit Personenspalte).
  - VacationEntitlementCard zeigt Team-Aggregat + VacationPerPersonList.
  - Modal `lock_person = false`, Person-Dropdown editierbar.
- `is_hr = false` (Employee):
  - Filter-Bar zeigt KEIN Person-Dropdown.
  - Liste filtert auf `current_user.sales_person_id`.
  - VacationEntitlementCard zeigt nur Self-Variante (Hero-Layout) ohne PerPersonList.
  - Modal `lock_person = true`, Person-Dropdown disabled, vorgefüllt mit `current_user.sales_person_id`.

### Refresh-Flow

- POST/PUT/DELETE → bumps `ABSENCE_REFRESH: GlobalSignal<u64>` in `src/service/absence.rs`.
- Liste re-fetched über `loader.rs`-Effect, der auf `ABSENCE_REFRESH` lauscht (analog `SHIFTPLAN_REFRESH`).
- `VacationBalance` wird ebenfalls bei jedem Refresh neu geladen (Vacation-Aktion ändert die Bilanz).

---

## Layout-Contract per Component

### AbsencePage (Top-Level)

```text
┌─────────────────────────────────────────────────────────────────┐
│ <padding md, gap-3 vertical>                                    │
│ ┌─Header (flex justify-between, flex-wrap)──────────────────┐  │
│ │ Title (text-h1) + Subtitle (text-body text-ink-muted)     │  │
│ │                                       [Btn Primary +]     │  │
│ └────────────────────────────────────────────────────────────┘  │
│ ┌─VacationEntitlementCard (rounded-lg, conditional)─────────┐  │
│ │ Hero-Spalte (Self) + Breakdown-Spalte                     │  │
│ │ + (HR-only) VacationPerPersonList                         │  │
│ └────────────────────────────────────────────────────────────┘  │
│ ┌─StatsGrid (3 Cards, auto-fit minmax(160px,1fr) gap-2.5)─┐    │
│ │ [SickDays] [UnpaidDays] [ActiveAbsences]                 │    │
│ └──────────────────────────────────────────────────────────┘    │
│ ┌─AbsenceFilterBar (rounded-lg, flex-wrap)─────────────────┐    │
│ │ [Cat-PillGroup] | [Person-Select HR] [Status-Select]     │    │
│ │                                          [showPast] [n/m]│    │
│ └──────────────────────────────────────────────────────────┘    │
│ ┌─AbsenceList (rounded-lg, overflow-hidden)────────────────┐    │
│ │ [Spaltenheader bg-surface-alt]                           │    │
│ │ [Row …]                                                  │    │
│ │ [Row …]                                                  │    │
│ │ ── oder Empty-State (py-12 text-center text-ink-muted) ──│    │
│ └──────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

### AbsenceModal Layout (Center, Width 520)

```text
┌─Modal Panel (rounded-lg, bg-surface, border)──────┐
│ Header: Title (text-lg) + Subtitle (text-body muted)│
│ Body (p-md, grid-cols-2 gap-3):                    │
│   [VersionConflictBanner — span-2, conditional]    │
│   [Field Mitarbeiter — span-2]                     │
│   [Field Kategorie — span-2 (Pill-Group)]          │
│   [Field Von]      [Field Bis (mit error-Slot)]    │
│   [SelfOverlapBanner — span-2, conditional]        │
│   [Field Beschreibung — span-2]                    │
│   [Vorschau-Box — span-2, bg-surface-alt]          │
│   [WarningList — span-2, conditional nach Save]    │
│ Footer (px-md py-3, flex):                         │
│   [Btn Danger (Edit-Mode only)] [Spacer flex-1]    │
│                          [Btn Ghost] [Btn Primary] │
└────────────────────────────────────────────────────┘
```

### Liste-Row Grid

```text
grid-cols-[1.5fr_170px_140px_90px_70px] gap-3.5 px-4 py-3.5
[Person+Description] [Range+Days] [CategoryBadge] [StatusPill] [Warnings+Chevron]
```

Auf mobilen Viewports (`max-w: 720px`) bleibt das Grid; horizontaler Scroll des Listen-Containers ist akzeptiert (entspricht Pattern in `week_view.rs`). Mobile-Spezialdesign ist NICHT in Phase 8 — Visualreference `absences.jsx` zeigt Desktop-First.

---

## State Pattern

| State | Owner | Lifecycle |
|-------|-------|-----------|
| `ABSENCE_STORE: GlobalSignal<Vec<AbsencePeriod>>` | `src/service/absence.rs` (neu) | Initial leer, populated über Loader bei Page-Mount, refreshed bei `ABSENCE_REFRESH`-Bump |
| `ABSENCE_REFRESH: GlobalSignal<u64>` | `src/service/absence.rs` (neu) | Bumped bei jedem POST/PUT/DELETE |
| `VACATION_BALANCE_STORE: GlobalSignal<Option<VacationBalance>>` (Self) | `src/service/vacation_balance.rs` (neu, Backend-Endpoint-Konsument) | Mounted in Self-Variante, refreshed bei `ABSENCE_REFRESH` |
| `VACATION_TEAM_STORE: GlobalSignal<Option<TeamVacationAggregate>>` (Team) | `src/service/vacation_balance.rs` | Mounted in HR-Variante |
| Lokaler Modal-State (`mode`, `editing_id`, Form-Felder) | `use_signal` in `AbsencePage` | Page-lokal, verworfen bei Modal-Close |

---

## Registry Safety

Phase 8 zieht KEINE shadcn-Blöcke und KEINE Dritt-Registry-Komponenten ein. Stack ist Rust/Dioxus + Tailwind + handgeschriebene Atoms. `components.json` existiert nicht; das ist Absicht (kein React).

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| shadcn official | (none) | not applicable |
| Dritt-Registry | (none) | not applicable |
| Eigene Atoms (`src/component/atoms/`, `src/component/form/`, `src/component/dialog.rs`) | wiederverwendet 1:1 | nicht erforderlich (in-tree) |

Falls in einer späteren Phase ein Dritt-Block erwogen wird, MUSS der `<shadcn_gate>`-Prozess + `npx shadcn view`-Vetting nachgeholt werden. In Phase 8: nicht relevant.

---

## Accessibility Contract

- Modal: ESC-Dismiss + Backdrop-Click-Dismiss bereits in `Dialog`-Atom — wird wiederverwendet.
- Form-Inputs haben `<label>`-Wrapper über `Field`-Atom — semantisch korrekt.
- Buttons sind `<button type="button">` (kein `<a>`-Hack).
- Liste-Rows sind `<button>`-Elemente (siehe Mockup, `absences.jsx:514`) → tab-fokussierbar.
- Pflicht-Aria:
  - `aria-label` an PersonAvatar-Spans im Mockup mit `aria-hidden="true"` (`absences.jsx:531`) — beibehalten.
  - Modal: `Dialog`-Komponente sollte `role="dialog"` + `aria-modal="true"` setzen — Plan-Phase verifiziert, ob bereits gesetzt.
  - Kategorie-Pills im Modal sind `<button type="button">` (D-14-Pattern aus Mockup `absences.jsx:659`).
- Kontrast: Alle Token-Paare (Accent auf Accent-Ink, Bad auf Bad-Soft, Warn auf Warn-Soft, Good auf Good-Soft) sind in `input.css` so abgestimmt, dass WCAG AA für Text ≥ 14px erfüllt ist (Designer-Vorgabe — nicht in Phase 8 neu validiert).

---

## i18n Compliance (D-13)

Alle in der Copywriting-Tabelle genannten Keys werden in `src/i18n/mod.rs` als `Key::*`-Enum-Variants definiert (Comment-Block `// Absence management`) und in `en.rs`, `de.rs`, `cs.rs` BEFÜLLT — alle drei Locales gleichzeitig. KEIN nachgelagerter Audit; Phase 13 ist nur das cross-phase Compliance-Gate. Locale::En-statt-Locale::De-Bug aus der Vergangenheit MUSS gemieden werden — jede Locale-Datei wird auf `Locale::De` (bzw. korrekten Variant) geprüft, bevor das Feature gemerged wird.

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS — vollständige i18n-Tabelle, alle CTA/Empty/Error/Destructive-Slots befüllt, drei Locales je Key
- [ ] Dimension 2 Visuals: PASS — Component-Inventory mit klarem Wiederverwendungs- vs Neu-Split; Layout-Contract pro Komponente
- [ ] Dimension 3 Color: PASS — 60/30/10-Tokens aus existierendem System, Accent-Reserved-For-Liste mit 6 expliziten Slots
- [ ] Dimension 4 Typography: PASS — Skala ist role-driven (Body/Caption/Small/Section/Page-Heading/Hero), keine neuen Tokens; Trennung `text-micro` vs `text-small` ist Rollenwechsel (UI-Chrome uppercase 600 vs Meta-Run normalcase 500), nicht 1px-Variation; 4 reguläre + 1 Section + 1 Hero-Slot, 2 Body-Weights (400/600) plus Hero-Weight (700, nur Display)
- [ ] Dimension 5 Spacing: PASS — Tailwind-4er-Skala wiederverwendet; formale Exceptions-Tabelle dokumentiert 2px / 10px / 12px / 14px (`p-0.5`, `gap-2.5`/`p-2.5`/`py-2.5`, `gap-3`, `gap-3.5`/`px-3.5`/`py-3.5`) je Slot mit Mockup-Treue-Begründung; alle nicht-4-Vielfachen sind innerhalb der Exceptions-Tabelle aufgeführt
- [ ] Dimension 6 Registry Safety: PASS — keine Dritt-Registry, keine shadcn-Init nötig (Rust/WASM-Stack)

**Approval:** pending
