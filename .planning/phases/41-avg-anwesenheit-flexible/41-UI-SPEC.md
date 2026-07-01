---
phase: 41
slug: avg-anwesenheit-flexible
status: draft
shadcn_initialized: false
preset: none
created: 2026-07-02
---

# Phase 41 — UI Design Contract: Ø-Anwesenheit bei flexiblen Stunden

> Visueller und interaktiver Vertrag für den Frontend-Anteil von Phase 41.
> Erzeugt durch gsd-ui-researcher; verifiziert durch gsd-ui-checker.

---

## Design System

| Eigenschaft | Wert |
|-------------|------|
| Tool | none (kein shadcn) |
| Preset | not applicable |
| Component library | Dioxus atoms (`TupleRow`, `section`, `h2`) |
| Icon library | none (diese Phase braucht keine Icons) |
| Font | System-default (Tailwind-Default) — unveränderter Projektzustand |

Quelle: `tailwind.config.js`, `components.json` existiert nicht, Dioxus/Tailwind-Stack.

---

## Spacing Scale

Projektweiter 8-Punkt-Raster, abgebildet auf Tailwind-Utility-Klassen. Für diese Phase
werden ausschließlich bereits verwendete Werte des HR-Stats-Blocks übernommen — **keine
neuen Token**.

| Token | Tailwind-Klasse | Px-Äquivalent | Verwendung in dieser Phase |
|-------|-----------------|---------------|-----------------------------|
| xs | `gap-2` | 8px | Vertikaler Abstand zwischen Rows im `section`-Container (bereits vorhanden) |
| sm | `py-1.5` | 6px oben/unten | Innen-Padding jeder `TupleRow` (bereits in `ROW_BASE`) |
| md | `gap-3` | 12px | Horizontaler Abstand Label ↔ Wert in `TupleRow` (bereits in `ROW_BASE`) |
| lg | `mt-6 pt-4` | 24px / 16px | Abstand und Padding des gesamten HR-Stats-`section` (bereits vorhanden) |

Ausnahmen: keine. Keine neuen Spacing-Token in dieser Phase eingeführt.

Quelle: `employee_view.rs` Zeilen 522–539, `tuple_row.rs` Zeilen 12–13.

---

## Typography

Direkt aus dem Projekt-Typo-System übernommen (`tailwind.config.js` + `tuple_row.rs`):

| Rolle | Tailwind-Token | Px / Gewicht | Line-Height | Verwendung |
|-------|----------------|--------------|-------------|------------|
| Abschnitts-Überschrift | `text-micro font-bold uppercase` | 11px / 600 | ~1.2 | `h2` „Statistik" — bereits vorhanden, unverändert |
| Label | `text-small text-ink-soft` | 12px / 500 | ~1.4 | Label der `TupleRow` (via `label_class(false)`) |
| Wert (Zahl) | `text-body font-mono tabular-nums` | 14px / 400 | 1.5 | Numerischer Wert — exakt wie bestehende „Ø Std/Woche"-Zahl |
| Leerzustand-Wert | `text-body font-mono tabular-nums text-ink-muted` | 14px / 400 | 1.5 | `–` statt Zahl wenn < 2 Anwesenheitstage |
| Beschreibung (inline) | `text-small font-normal text-ink-muted` | 12px / 400 | ~1.4 | `TupleRow.description` Slot — Tooltip-Text als Inline-Notiz unterhalb der Row |

Quelle: `CONVENTIONS.md` §Styling Conventions, `tuple_row.rs` Zeilen 12–37, `employee_view.rs` Zeile 528–529.

---

## Color

Ausschließlich existierende Projekt-Token — keine neuen Farbdefinitionen.

| Rolle | Token | CSS-Variable | Verwendung |
|-------|-------|--------------|------------|
| Dominant (60 %) | `bg-surface` / `bg-surface-alt` | `--bg` / `--surface` | Seitenhintergrund — unverändert |
| Sekundär (30 %) | `border-border` | `--border-color` | Trennlinie unter jeder `TupleRow` |
| Primärtext | `text-ink` / `text-ink-soft` | `--ink` / `--ink-soft` | Wert-Slot und Label-Slot |
| Gedimmter Text | `text-ink-muted` | `--ink-muted` | Leerzustand-Wert (`–`) + `description`-Text + Abschnitts-Überschrift |
| Akzent (10 %) | nicht verwendet | — | Diese Phase führt keinen akzentuierten Zustand ein |
| Destruktiv | nicht verwendet | — | Keine destruktive Aktion in dieser Phase |

Akzent reserviert für: (nicht anwendbar für diese Phase).

Verbotene Klassen (Projekt-Lint, 18 Self-Tests): `bg-gray-*`, `bg-white`, `text-gray-*`,
`text-blue-*`, `text-red-*`, `text-green-*`, `bg-blue-*`, `bg-green-*`, `bg-red-*`,
`border-black`, `border-gray-*` — KEINER dieser Werte darf in neuem Code erscheinen.

---

## Component Inventory

### Neue UI-Elemente

Alle über bereits vorhandene Komponenten realisiert — **keine neuen Atome** nötig.

| Element | Komponente | Props |
|---------|------------|-------|
| Neue `TupleRow` für die Kennzahl | `TupleRow` (bereits in `component/atoms/`) | `label`, `value`, `description` |
| Numerischer Wert (Normalzustand) | `span` | `class: "font-mono tabular-nums"` + `{format_hours(avg, 2)}` |
| Wert im Leerzustand (< 2 Tage) | `span` | `class: "font-mono tabular-nums text-ink-muted"` + Inhalt `"–"` (EN-DASH U+2013) |
| Inline-Beschreibung / Tooltip | `TupleRow.description` Slot | `rsx! { "{i18n.t(Key::AvgHoursPerAttendanceDayDescription)}" }` |

### Genaue Einfügeposition in `employee_view.rs`

```
section { class: "mt-6 pt-4 border-t border-border flex flex-col gap-2",
    h2 { class: "text-micro font-bold uppercase text-ink-muted",
        {i18n.t(Key::StatisticsHeading)}           // ← bestehend, unverändert
    }
    TupleRow {                                       // ← bestehend: „Ø Std/Woche"
        label: ...(Key::AverageWorkedHoursPerWeek),
        value: rsx! { span { class: "font-mono tabular-nums", ... } },
    }
    // ↓ NEU — direkt nach „Ø Std/Woche", vor „Einbezogene Wochen"
    TupleRow {
        label: ImStr::from(i18n.t(Key::AvgHoursPerAttendanceDay).as_ref()),
        value: rsx! { span { class: "font-mono tabular-nums [+ text-ink-muted wenn leer]",
            { /* format_hours(avg, 2) ODER "–" */ }
        } },
        description: Some(rsx! {
            "{i18n.t(Key::AvgHoursPerAttendanceDayDescription)}"
        }),
    }
    // ↑ NEU Ende
    TupleRow {                                       // ← bestehend: „Einbezogene Wochen"
        label: ...(Key::StatisticsIncludedWeeks),
        ...
        dim: true,
    }
}
```

D-AVG-07-konform: direkt neben (nach) der bestehenden „Ø Std/Woche"-Zeile,
innerhalb desselben `section`-Blocks hinter `should_show_hr_stats`.

Quellen: D-AVG-07, `employee_view.rs` Zeilen 519–541.

---

## Interaction Contract

### Normalzustand (Zahl vorhanden)

- Bedingung: `is_dynamic == true` UND ≥ 2 Anwesenheitstage im Report-Zeitraum
- Darstellung: `format_hours(avg_hours_per_attendance_day, 2)` — identische Formatierung
  wie benachbartes „Ø Std/Woche" (z. B. `"4.50"`)
- Klasse: `"font-mono tabular-nums"` (kein `text-ink-muted`)
- Keine Interaktion; read-only

### Leerzustand (< 2 Anwesenheitstage oder keine Arbeitstage)

- Bedingung: < 2 Anwesenheitstage im Zeitraum (D-AVG-06)
- Darstellung: EN-DASH `"–"` als Wert
- Klasse: `"font-mono tabular-nums text-ink-muted"`
- `TupleRow.dim`: NICHT gesetzt (die Row bleibt im normalen Kontrast; nur der Wert-Slot ist gedimmt)
- Kein separater Spinner oder Skeleton — Daten kommen mit dem bestehenden Report-Lade-Zyklus

### Nicht-flexible Mitarbeiter (`is_dynamic == false`)

- Zahl wird serverseitig nicht geliefert → `TupleRow` erscheint **nicht** (keine leere Row)
- Frontend-Bedingung: Row nur rendern wenn `avg_hours_per_attendance_day: Option<f32>` im
  Response-Struct `Some(...)` ist

### Nicht-HR-Rollen

- Endpoint ist HR-gated (D-AVG-05) → FE empfängt dieses Feld nie → identisch wie
  `should_show_hr_stats`-Gate: gesamter Block bleibt ausgeblendet

---

## Copywriting Contract

### i18n-Schlüssel (neu — alle drei Locales Pflicht: de/en/cs)

#### `Key::AvgHoursPerAttendanceDay` — Label der `TupleRow`

| Locale | Text |
|--------|------|
| de | `"Ø Std/Anwesenheitstag"` |
| en | `"Avg h/attendance day"` |
| cs | `"Prům. hod./den přítomnosti"` |

Kurz gehalten, damit die Row im selben Layout wie „Ø Std/Woche" bleibt.

#### `Key::AvgHoursPerAttendanceDayDescription` — Inline-Beschreibung (TupleRow.description)

| Locale | Text |
|--------|------|
| de | `"Durchschnittliche Arbeitsstunden pro Anwesenheitstag (nur flexible MA). Urlaub und Abwesenheiten sind nicht im Nenner."` |
| en | `"Average working hours per day actually worked (flexible employees only). Absences excluded from the denominator."` |
| cs | `"Průměrné pracovní hodiny za den skutečné přítomnosti (pouze flexibilní zaměstnanci). Absence nejsou ve jmenovateli."` |

Wird als `text-small font-normal text-ink-muted` unterhalb der Zeile gerendert
(TupleRow.description-Slot — bestehende Mechanik, kein neues Atom).

#### `Key::AvgHoursPerAttendanceDayEmpty` — Leerzustand-Tooltip / Screen-Reader-Label

| Locale | Text |
|--------|------|
| de | `"Nicht aussagekräftig (weniger als 2 Anwesenheitstage)"` |
| en | `"Not meaningful (fewer than 2 attendance days)"` |
| cs | `"Nevýznamné (méně než 2 dny přítomnosti)"` |

Dieser Text wird als `title`-Attribut auf dem `span` der Leerzustand-Zelle gesetzt,
damit Screen-Reader und Hover-Tooltip den Grund für „–" erklären:

```rust
span {
    class: "font-mono tabular-nums text-ink-muted",
    title: "{i18n.t(Key::AvgHoursPerAttendanceDayEmpty)}",
    "–"
}
```

### Leerzustand

| Element | Wert |
|---------|------|
| Wert im Value-Slot | `–` (EN-DASH U+2013) |
| Accessible-Label (title-Attr.) | Key::AvgHoursPerAttendanceDayEmpty |
| Keine separate Überschrift nötig | — (die Row-Struktur bleibt, nur der Wert ändert sich) |

### Error State

Kein separater Fehlerzustand für diese Zahl nötig. API-Fehler werden durch den
bestehenden `ERROR_STORE`-Mechanismus abgefangen (identisch mit allen anderen Report-Werten).
Bei fehlendem Wert im Response → `None` → Row wird nicht gerendert (identisch mit
Nicht-flexible-MA-Behandlung).

### Destructive Actions

Keine destruktiven Aktionen in dieser Phase.

---

## Registry Safety

| Registry | Verwendete Blöcke | Safety Gate |
|----------|-------------------|-------------|
| shadcn official | keine | not applicable |
| Drittanbieter | keine | not applicable |

Diese Phase führt keine neuen Abhängigkeiten, Pakete oder Registry-Blöcke ein.

---

## Annahmen (Autonomous Mode)

Da dieser Vertrag ohne interaktive Rückfragen erstellt wurde, werden folgende
Entscheidungen als explizite Annahmen dokumentiert:

| Nr. | Annahme | Begründung |
|-----|---------|------------|
| A-1 | Neue `TupleRow` direkt **nach** „Ø Std/Woche", **vor** „Einbezogene Wochen" | D-AVG-07: „neben" der bestehenden Zahl; thematisch zusammengehörig; „Einbezogene Wochen" bleibt als Fußnote am Ende |
| A-2 | Leerzustand = EN-DASH `–` + `text-ink-muted` (kein separater Platzhalter-Text in der Row) | Konsistent mit Tabellen-Konventionen im Projekt; kurz + scanbar; kein neues Atom nötig |
| A-3 | Beschreibungstext via `TupleRow.description`-Prop (Inline unter der Row), nicht via CSS-`title`-Tooltip | `description`-Prop ist bereits implementiert und gestylt; kein neues Tooltip-Atom nötig; matches bestehende UX-Muster (Feedback-Warnungen inline, nicht als Dialog — Phase 9) |
| A-4 | Leerzustand erhält zusätzlich `title`-Attribut für Screen-Reader / Hover | Kostet nichts, verbessert Barrierefreiheit; kein Widerspruch zu CONTEXT |
| A-5 | `TupleRow.dim: false` für die neue Row (normaler Kontrast wie „Ø Std/Woche") | Kennzahl ist primär, nicht sekundär; `dim: true` ist nur für Hilfszeilen (→ „Einbezogene Wochen") |
| A-6 | `format_hours(avg, 2)` — 2 Dezimalstellen | Identisch mit bestehender „Ø Std/Woche"-Zahl; Konsistenz > Kompaktheit |
| A-7 | Row komplett weglassen wenn `Option<f32>` = `None` (Nicht-flexible MA oder API liefert kein Feld) | Sauberer als leere Row; server-seitiger Filter per D-AVG-05 |

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS
- [ ] Dimension 2 Visuals: PASS
- [ ] Dimension 3 Color: PASS
- [ ] Dimension 4 Typography: PASS
- [ ] Dimension 5 Spacing: PASS
- [ ] Dimension 6 Registry Safety: PASS

**Approval:** pending
