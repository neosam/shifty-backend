---
phase: 49
slug: pdf-download-button
status: draft
shadcn_initialized: false
preset: none
created: 2026-07-03
---

# Phase 49 — UI Design Contract

> Visueller und interaktiver Vertrag für den on-demand PDF-Download-Button
> in `shifty-dioxus/src/page/shiftplan.rs`. Scope ist bewusst minimal: EIN
> Button neben dem iCal-Button; keine neue Seite, kein neuer Component-File,
> kein Redesign des Shiftplan-Views.
>
> Alle Kern-Entscheidungen sind bereits in `49-CONTEXT.md` gelockt
> (D-49-10..D-49-14) und werden hier nur als visueller Vertrag re-
> kodifiziert, sodass Planner/Executor/UI-Checker die gleiche Referenz haben.

---

## Design System

| Property | Value |
|----------|-------|
| Tool | none (kein shadcn — Dioxus/Rust-WASM, nicht React/Next/Vite) |
| Preset | not applicable |
| Component library | keine (native Dioxus RSX + Tailwind Utility-Klassen) |
| Icon library | keine — Icon-Prefix `↓` als Unicode-Glyph in mono-Font (Precedent iCal-Button) |
| Font | Projekt-Default (Tailwind Basis-Sans); Icon-Glyph `↓` in `font-mono` |

**Registry-Situation:** Diese Phase erweitert eine bestehende Dioxus-App mit
Tailwind + CSS-Custom-Property-Tokens (`tailwind.config.js` §20–29:
`surface`, `surface-alt`, `ink`, `border-strong` etc.). Ein shadcn-Init ist
nicht anwendbar (Toolchain ≠ React) und wurde bewusst weggelassen — die
shadcn-Init-Gate aus der Researcher-Skill greift nur für React/Next/Vite.

---

## Spacing Scale

Die Phase führt keine neuen Spacing-Token ein. Alle verwendeten Werte
kommen aus der bestehenden Tailwind-Skala und sind identisch zum
iCal-Button-Precedent (`shiftplan.rs:1130–1138`).

Deklariert (Vielfache von 4):

| Token | Value | Usage in dieser Phase |
|-------|-------|-----------------------|
| xs | 4px  | Icon-Label-Gap innerhalb des Anchors (`gap-1` = `0.25rem`) |
| sm | 8px  | Optionale Row-Gaps zwischen iCal- und PDF-Button (kommt aus Parent-Flex-Layout, kein neues Token) |
| md | 16px | (nicht in dieser Phase gesetzt — Parent bestimmt) |
| lg | 24px | (nicht in dieser Phase gesetzt) |
| xl | 32px | (nicht in dieser Phase gesetzt) |

**Konkret am Button:**

- Horizontales Padding: `px-3` = 12px (Tailwind-Skala, exakt wie iCal — bewusste
  Ausnahme von reinem 4/8-Rhythmus, weil das der Precedent ist).
- Vertikales Padding: `py-1.5` = 6px (dito, Precedent).
- Icon-Label-Gap: `gap-1` = 4px.

**Exceptions:** `px-3` (12px) und `py-1.5` (6px) sind keine strengen
8-Point-Werte, werden aber bewusst 1:1 vom iCal-Button-Precedent übernommen,
damit die beiden Buttons visuell perfekt paaren. Neue Buttons in der
Toolbar dürfen von diesem Muster abweichen — dieser eine Button MUSS es
nicht.

---

## Typography

Die Phase führt keine neuen Typografie-Rollen ein. Der Button erbt die
bestehende Skala (siehe `text-body`, `font-medium` als Tailwind-Utility).

| Role | Size | Weight | Line Height | Usage in dieser Phase |
|------|------|--------|-------------|-----------------------|
| Body | `text-body` (Projekt-Default, ~14px) | `font-medium` (500) | Tailwind-Default 1.5 | Button-Label „PDF" |
| Icon-Glyph | `text-body` (erbt) | `font-medium` (erbt) | erbt | `↓`-Unicode in `font-mono` (nur Font-Family überschrieben) |
| Label | n.v. | n.v. | n.v. | (kein separates Label-Element) |
| Heading | n.v. | n.v. | n.v. | (Button ändert keine Headings) |

**Konkret am Button:**

- Klasse: `text-body font-medium` (exakt wie iCal).
- Icon-Wrapper: `<span class="font-mono">↓</span>` — nur die Icon-Zelle
  wechselt auf Monospace, um mit dem iCal-Icon symmetrisch zu wirken.

**Line-Height:** Tailwind-Default (~1.5). Nicht überschrieben.

---

## Color

Die Phase führt keine neuen Farb-Token ein. Alle verwendeten Farben sind
CSS-Custom-Properties aus `tailwind.config.js` und identisch zum
iCal-Button-Precedent.

| Role | Value | Usage in dieser Phase |
|------|-------|-----------------------|
| Dominant (60%) | `bg-surface` = `var(--surface)` | Button-Hintergrund (Neutral) |
| Secondary (30%) | n.v. in dieser Phase | (Card-/Sidebar-/Nav-Farben nicht berührt) |
| Accent (10%) | KEIN Accent auf diesem Button | Bewusste Entscheidung: PDF-Download ist Sekundär-Aktion neben iCal-Export, kein Primary-CTA |
| Destructive | n.v. in dieser Phase | (keine destruktive Aktion) |
| Border | `border-border-strong` = `var(--border-strong)` | Button-Kontur |
| Text | `text-ink` = `var(--ink)` | Button-Text + Icon |
| Hover-Feedback | `hover:bg-surface-alt` = `var(--surface-alt)` | Hover-State |

**Accent reserved for:** Diese Phase reserviert Accent bewusst NICHT für
den PDF-Button. Accent-Farbe bleibt Primary-CTAs vorbehalten (z.B. „Slot
speichern", „Buchung anlegen"). Der PDF-Download-Button ist eine passive
Sekundär-Aktion (Precedent: iCal-Button ist ebenfalls nicht accent-farbig)
und teilt das Neutral-Grau-Muster der Toolbar-Anchor-Buttons.

**Farb-Kontrast:** Wird durch die bestehenden Tokens gewährleistet
(`ink` auf `surface` ist Projekt-weit WCAG-AA-verifiziert; nicht Teil dieser
Phase).

---

## Copywriting Contract

Diese Phase führt GENAU EINEN neuen i18n-Key ein: `Key::PdfDownload`
(D-49-14). Alle drei Locales bekommen dieselbe Kurz-Copy „PDF", weil das
Kürzel-Label neben dem iCal-Kürzel-Label „iCal" sitzt und die visuelle
Symmetrie das Ziel ist.

| Element | Copy |
|---------|------|
| Primary CTA | n.v. — der PDF-Button ist KEIN Primary-CTA (siehe Color-Sektion) |
| Sekundär-Aktion Label (de) | **„PDF"** — Kurz-Label neben iCal-Button, i18n-Key `PdfDownload` |
| Sekundär-Aktion Label (en) | **„PDF"** — identisch (Kürzel) |
| Sekundär-Aktion Label (cs) | **„PDF"** — identisch (Kürzel) |
| Icon-Prefix | `↓` (Unicode U+2193, „Downwards Arrow") — kein neuer Key, konstantes Glyph |
| Empty state heading | n.v. — Button wird bei leerem State (WeekStatus ∈ {Unset, InPlanning} oder `selected_shiftplan_id == None`) einfach nicht gerendert (D-49-13, „Sichtbarkeit statt Disabled") |
| Empty state body | n.v. — kein Empty-State-UI (der Button verschwindet komplett; die Nutzer:in weiß aus dem KW-Status-Badge daneben, warum kein Download möglich ist) |
| Error state | n.v. — kein Toast, kein Banner, kein Fehler-Modal (D-49-13 Anti-Pattern-Klausel). Race-Case (Status wechselt zwischen Signal-Update und Klick) liefert eine 409-Browser-Response, die der Browser als „Download abgebrochen" darstellt. Kein zusätzliches UI |
| Destructive confirmation | n.v. — Download ist read-only (keine destruktive Aktion) |
| Tooltip | Optional `title="PDF"` (aus dem i18n-Key gefüttert) — kein separater Tooltip-Key. Kein Hover-Card, kein Popover |

**Kontext-Anker:** Der Button lädt die aktuell im UI selektierte
Kalenderwoche des ausgewählten Shiftplans (D-49-04) — dieser Kontext ist
über den umgebenden Wochennavigator sichtbar und wird NICHT im Button-
Label wiederholt (keine „PDF KW27" o.ä.). Der Download-Dateiname
(`schichtplan-2026-KW27.pdf`) trägt den Kontext für den offline-Fall.

**Sprach-Rationale „PDF" statt „PDF herunterladen":** Symmetrie zum
iCal-Kürzel; wenn die Nutzer:innenschaft später einen längeren String
wünscht, ist der Nachtrag trivial (D-49-14).

---

## Component Contract — PDF-Download-Button

Dieser Abschnitt ist der Kern-Vertrag der Phase. Er ist nicht Teil des
Standard-Templates, aber notwendig, weil die ganze Phase aus genau EINER
UI-Komponente besteht.

### Struktur

```rust
// Innerhalb der Toolbar-Row in shiftplan.rs, direkt NACH dem iCal-Anchor-
// Block (~Zeile 1140).
if let Some(sp_id) = selected_shiftplan_id.read().as_ref() {
    if matches!(*week_status.read(), WeekStatus::Planned | WeekStatus::Locked) {
        a {
            class: "px-3 py-1.5 rounded-md text-body font-medium border \
                    bg-surface text-ink border-border-strong \
                    inline-flex items-center gap-1 hover:bg-surface-alt",
            href: format!(
                "{}/shiftplan/{}/{}/{}/pdf",
                backend_url, sp_id, *year.read(), *week.read()
            ),
            download: format!("schichtplan-{}-KW{:02}.pdf", *year.read(), *week.read()),
            title: "{pdf_download_label}",
            span { class: "font-mono", "↓" }
            "{pdf_download_label}"
        }
    }
}
```

### Sichtbarkeits-Vertrag (D-49-13)

| Bedingung | Verhalten |
|-----------|-----------|
| `selected_shiftplan_id == None` | Button **nicht gerendert** (kein DOM-Node) |
| `WeekStatus == Unset` | Button **nicht gerendert** |
| `WeekStatus == InPlanning` | Button **nicht gerendert** |
| `WeekStatus == Planned` | Button **sichtbar und klickbar** |
| `WeekStatus == Locked` | Button **sichtbar und klickbar** |

**Explizite Anti-Patterns (bewusst weggelassen):**

- Kein `disabled`-Zustand.
- Kein Tooltip mit Begründung („Woche noch in Planung — kein Download").
- Kein Fehler-Toast bei Race-Klick.
- Kein Loading-Spinner (Browser-Standard-Download-UI reicht).
- Kein `target="_blank"` (das `download`-Attribut allein ist ausreichend
  und `_blank` würde in manchen Browsern das `download`-Attribut
  ignorieren, Anti-Pattern aus RESEARCH.md Pitfall).

### Interaktions-Vertrag

| Ereignis | Verhalten |
|----------|-----------|
| Klick | Browser folgt `href` (GET, Cookie-Auth), Backend liefert `application/pdf` mit `Content-Disposition: attachment; filename="…"`, Browser speichert unter dem `download`-Attribut-Namen |
| Hover | `bg-surface-alt`-Farbwechsel (siehe Color-Sektion) |
| Fokus | Tailwind-Default-Fokus-Ring (Browser-Standard) — nicht explizit gesetzt, aber Anchor bleibt tab-fokussierbar |
| Keyboard | `Enter` triggert Anchor-Navigation (nativer Anchor-Semantik) |

### Layout-Position

- **Datei:** `shifty-dioxus/src/page/shiftplan.rs`
- **Umgebung:** Toolbar-Row des Shiftplan-Headers (`~Zeile 1123–1140`).
- **Reihenfolge:** direkt NACH dem iCal-Anchor, VOR dem
  `is_shiftplanner`-abhängigen Booking-Log-Button (`~Zeile 1141`).
- **Parent-Layout:** Flex-Row des Toolbar-Containers (bestehend, nicht
  angefasst).
- **Rationale:** Symmetrie mit iCal (beide sind File-Download-Aktionen);
  User-Precedent aus CONTEXT.md D-49-11 („neben iCal").

---

## Registry Safety

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| shadcn official | keine — shadcn nicht anwendbar (kein React) | not required |
| Drittanbieter-Registry | keine | not required |
| Neue Cargo-Deps | keine (siehe REQUIREMENTS.md Nicht-Ziele, RESEARCH.md „Standard Stack") | not required — kein neuer Registry-Zugriff |
| Icon-Registry | keine — `↓` ist ein Unicode-Codepoint, kein externes Icon-Package | not required |

**Kein third-party-View-Gate nötig** — die Phase installiert nichts von
externen Registries. Alle Bausteine (Tailwind-Utility-Klassen, Dioxus RSX,
i18n-Enum-Extension) kommen aus dem bestehenden Workspace.

---

## Contract Pre-Population Trace

Nachvollziehbarkeit: welche Werte kamen woher.

| Feld | Quelle | Notiz |
|------|--------|-------|
| Button-Klassen (`px-3 py-1.5 rounded-md text-body font-medium border bg-surface text-ink border-border-strong inline-flex items-center gap-1 hover:bg-surface-alt`) | CONTEXT D-49-11 (locked) | iCal-Precedent 1:1 |
| Icon-Prefix `↓` in `font-mono` | CONTEXT D-49-11 (locked) | iCal-Precedent |
| i18n-Key `Key::PdfDownload` | CONTEXT D-49-14 (locked) | Neu, exakt einer |
| Copy „PDF" (de/en/cs) | CONTEXT D-49-14 (locked) | Symmetrie zu iCal-Kürzel |
| `download`-Attribut-Format `schichtplan-{yyyy}-KW{ww:02}.pdf` | CONTEXT D-49-11, REQUIREMENTS PDF-03 | Konsistent zu v2.2 WebDAV |
| URL-Muster `format!("{}/shiftplan/{}/{}/{}/pdf", …)` | CONTEXT D-49-12 (locked) | Nutzt `selected_shiftplan_id`/`year`/`week` |
| Sichtbarkeits-Prädikat | CONTEXT D-49-13, REQUIREMENTS PDF-04 | Pure fn `should_show_pdf_button` (Test-Punkt aus RESEARCH.md) |
| Kein Disabled/Tooltip/Toast/Spinner | CONTEXT D-49-13 (locked), User-Präferenz | „Warnungen inline statt Dialog" (MEMORY-Anker) |
| Kein `target="_blank"` | CONTEXT D-49-11, RESEARCH.md Anti-Pattern | Sonst ignoriert Browser `download` |
| Farb-Token (`surface`, `ink`, `border-strong`, `surface-alt`) | `shifty-dioxus/tailwind.config.js` §20–29 | Bestehende CSS-Custom-Properties |
| Typografie (`text-body`, `font-medium`) | iCal-Precedent | Nicht neu |

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS — genau EIN neuer Key `PdfDownload` in de/en/cs, Copy „PDF" (Symmetrie zu iCal), kein Empty/Error/Destructive-State nötig
- [ ] Dimension 2 Visuals: PASS — Anchor-Element, identische visuelle Sprache wie iCal-Button, keine neuen Bild-/Icon-Assets
- [ ] Dimension 3 Color: PASS — keine neuen Farb-Token; Accent bewusst NICHT auf Sekundär-Aktion; bestehende CSS-Custom-Properties
- [ ] Dimension 4 Typography: PASS — keine neuen Typografie-Rollen, `text-body`/`font-medium` erbend
- [ ] Dimension 5 Spacing: PASS — `gap-1` (4px), `px-3`/`py-1.5` bewusst als Precedent-Ausnahme dokumentiert
- [ ] Dimension 6 Registry Safety: PASS — keine externe Registry berührt, kein third-party-Block, keine neuen Cargo-Deps

**Approval:** pending
