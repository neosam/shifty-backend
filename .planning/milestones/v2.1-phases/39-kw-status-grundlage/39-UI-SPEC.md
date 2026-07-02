---
phase: 39
slug: kw-status-grundlage
status: draft
shadcn_initialized: false
preset: none
created: 2026-07-02
---

# Phase 39 — UI Design Contract: KW-Status Grundlage

> Visueller und interaktiver Kontrakt fur die Frontend-Seite der Phase 39.
> Erzeugt von gsd-ui-researcher; gepruft von gsd-ui-checker.

---

## Design System

| Property | Value |
|----------|-------|
| Tool | none (Dioxus 0.6 + Tailwind CSS; kein shadcn/React) |
| Preset | not applicable |
| Component library | Dioxus-eigene Atoms (`Btn`, `DropdownTrigger`, `PersonChip`) aus `src/component/atoms/` und `src/component/dropdown_base.rs` |
| Icon library | Keine externe Bibliothek — Mono-Glyphen (`‹ › x …`) als Zeichen-Literale |
| Font | Inter (sans, `font-family`-Token), JetBrains Mono (mono, `font-mono`) |

Quelle: CONVENTIONS.md + STACK.md + `tailwind.config.js` (verifiziert)

---

## Spacing Scale

Alle Abstande folgen dem 4-Punkt-Raster, das bereits durchgangig im gesamten Projekt verwendet wird.

| Token | Value | Usage |
|-------|-------|-------|
| xs | 4px | Icon-Gaps, Gap innerhalb Badge (`gap-1`) |
| sm | 8px | Badge-Innenabstand (`px-2`), Abstande zwischen Toolbar-Elementen |
| md | 16px | Margin der m-4-Wrapper-div, Standard-Elementabstand |
| lg | 24px | Abschnittstrennungen |
| xl | 32px | — |
| 2xl | 48px | — |
| 3xl | 64px | — |

Ausnahmen: Badge-Padding `px-2 py-0.5` (8px / 2px) — identisch mit bestehenden kompakten Toolbar-Elementen. Der Status-Strip bekommt `mb-3` (12 px, Tailwind-Standard) zum WeekView-Abstand — kein 4pt-Bruch, da 12px = 3 x 4px.

Touch-Target-Ausnahme: Schichtplaner-Dropdown-Trigger hat `h-auto` (Hohe durch py-0.5 = ca. 24px DOM). In WASM/Browser liegt das unter der empfohlenen 44px-Grenze, ist aber im Desktop-Schichtplan-Kontext akzeptiert (analog bestehende Toolbar-Buttons mit `h-7`).

---

## Typography

Kanonischer Typografie-Stack aus `tailwind.config.js` (verifiziert):

| Rolle | Klasse | Grosse | Gewicht | Zeilenhohe |
|-------|--------|--------|---------|------------|
| Body (Standard-Text, Dropdown-Eintraege) | `text-body` | 14px | 400 | 20px (1.43) |
| Label / sekundarer Text / Badge-Text | `text-small` | 12px | 500 | 16px (1.33) |
| Micro-Tag (uppercase Kennzeichner) | `text-micro` | 11px | 600 | 14px (1.27) |
| Modal-Titel | `text-lg` | 16px | 600 | 22px |

**Status-Badge:** `text-small font-medium` (12px, 500) — konsistent mit bestehenden Inline-Tags und Toolbar-Labels, die durchgangig `text-small` verwenden.

**Dropdown-Eintraege:** `text-body` (14px, 400) — konsistent mit bestehenden `DropdownEntry`-Eintraegen in `dropdown_base.rs`.

**Dropdown-Trigger-Text:** `text-small font-medium` — gleich wie Badge, da der Trigger bei gesetztem Status auch farblich als Badge erscheint.

---

## Color

Ausschliesslich design-token-basierte Klassen aus `tailwind.config.js`. Verbotene Klassen (Linting-Gate, 18 Self-Tests): `bg-gray-*`, `bg-white`, `text-gray-*`, `text-blue-*`, `text-red-*`, `text-green-*`, `bg-blue-*`, `bg-green-*`, `bg-red-*`, `border-black`, `border-gray-*`.

| Rolle | Tailwind-Token | Verwendung |
|-------|----------------|------------|
| Dominant (60%) | `bg-surface` / `bg-bg` | Seiten- und Toolbar-Hintergrund |
| Sekundar (30%) | `bg-surface-alt` | Toolbar-Karte, Grid-Header, Unset-Trigger |
| Semantisch: gut | `bg-good-soft` + `text-good` + `border-good` | Status „Geplant" (Planned) |
| Semantisch: Warnung | `bg-warn-soft` + `text-warn` + `border-warn` | Status „In Planung" (InPlanning) |
| Semantisch: schlecht | `bg-bad-soft` + `text-bad` + `border-bad` | Status „Gesperrt" (Locked) |
| Neutral | `bg-surface-alt` + `text-ink-muted` + `border-border` | Unset-Trigger (Schichtplaner) — niemals als Badge |

**Status-Farb-Mapping (direkt implementierbar, `match`-Ast pro Variante):**

| `WeekStatus` | Tailwind-Klassen (vollstandig) | Wann sichtbar |
|---|---|---|
| `Locked` | `bg-bad-soft border border-bad text-bad` | Badge (alle Rollen) + Dropdown-Trigger (Schichtplaner) |
| `Planned` | `bg-good-soft border border-good text-good` | Badge (alle Rollen) + Dropdown-Trigger (Schichtplaner) |
| `InPlanning` | `bg-warn-soft border border-warn text-warn` | Badge (alle Rollen) + Dropdown-Trigger (Schichtplaner) |
| `Unset` | `bg-surface-alt border border-border text-ink-muted` | NUR Dropdown-Trigger (Schichtplaner); NIE als Badge |

Akzent-Token (`bg-accent`, `text-accent`, `bg-accent-soft`) sind in Phase 39 nicht fur Status-Elemente belegt.

Akzent reserviert fur: bestehende Fokus-Ringe (`border-accent` in `.form-input:focus`), Accent-Schaltflachen — keine neuen Elementen in Phase 39.

---

## Copywriting Contract

Alle Strings kommen aus `I18N.read().t(Key::...)`. Neue i18n-Keys werden in `src/i18n/mod.rs` (Enum-Variante), `en.rs`, `de.rs`, `cs.rs` erganzend hinzugefugt. Jede Variante braucht alle drei Lokale — fehlende Variante ergibt `"??"` zur Laufzeit.

### Neue i18n-Keys

| Key (Enum-Variante) | de | en | cs |
|---------------------|----|----|-----|
| `WeekStatusUnset` | Kein | None | Zadny |
| `WeekStatusInPlanning` | In Planung | In planning | V planovaní |
| `WeekStatusPlanned` | Geplant | Planned | Naplanovano |
| `WeekStatusLocked` | Gesperrt | Locked | Uzamceno |
| `WeekStatusSetError` | Status konnte nicht gespeichert werden. | Failed to save week status. | Nepodarilo se ulozit stav tydne. |

### Element-Copywriting

| Element | Text (de) | Zustand |
|---------|-----------|---------|
| Badge | „In Planung" | Nicht-Schichtplaner, Status = InPlanning |
| Badge | „Geplant" | Nicht-Schichtplaner, Status = Planned |
| Badge | „Gesperrt" | Nicht-Schichtplaner, Status = Locked |
| Kein Element | (leer) | Nicht-Schichtplaner, Status = Unset |
| Dropdown-Trigger | „Kein" | Schichtplaner, Status = Unset |
| Dropdown-Trigger | „In Planung ▾" | Schichtplaner, Status = InPlanning |
| Dropdown-Trigger | „Geplant ▾" | Schichtplaner, Status = Planned |
| Dropdown-Trigger | „Gesperrt ▾" | Schichtplaner, Status = Locked |
| Dropdown-Eintrag 1 | „Kein" | immer sichtbar im Dropdown |
| Dropdown-Eintrag 2 | „In Planung" | immer sichtbar im Dropdown |
| Dropdown-Eintrag 3 | „Geplant" | immer sichtbar im Dropdown |
| Dropdown-Eintrag 4 | „Gesperrt" | immer sichtbar im Dropdown |
| Fehler (ERROR_STORE) | „Status konnte nicht gespeichert werden." | nach fehlgeschlagenem Set-API-Call |

Leerzustand: kein separater Leer-Text notwendig — Unset ist implizit leer (kein Element fur Nicht-Schichtplaner). Der Dropdown-Trigger zeigt „Kein" fur Schichtplaner.

Destruktive Aktionen: keine in Phase 39. Der Wechsel nach Locked oder zuruck gilt als normaler Statuswechsel ohne Bestatigungsdialog (D-39-02: alle Ubergange frei).

---

## Component Inventory (Implementierungs-Anleitung)

### 1. `WeekStatusBadge` — neu, Atom

**Datei:** `src/component/atoms/week_status_badge.rs`

Reines Anzeige-Atom ohne eigenen State, ohne API-Aufrufe.

```rust
#[derive(Props, Clone, PartialEq)]
pub struct WeekStatusBadgeProps {
    pub status: WeekStatus, // Aufruf nur bei != Unset; Unset -> Caller rendert nichts
}

// Klassen-Helper — unit-testbar ohne VirtualDom
pub(crate) fn week_status_badge_class(status: &WeekStatus) -> &'static str {
    match status {
        WeekStatus::Locked =>
            "inline-flex items-center px-2 py-0.5 rounded-sm text-small font-medium bg-bad-soft border border-bad text-bad",
        WeekStatus::Planned =>
            "inline-flex items-center px-2 py-0.5 rounded-sm text-small font-medium bg-good-soft border border-good text-good",
        WeekStatus::InPlanning =>
            "inline-flex items-center px-2 py-0.5 rounded-sm text-small font-medium bg-warn-soft border border-warn text-warn",
        WeekStatus::Unset =>
            unreachable!("Badge wird nie fuer Unset gerendert"),
    }
}
```

Keine dynamischen Klassen via `format!()` — nur statische `match`-Arme (Tailwind-Detect-Pflicht).

Test: `no_legacy_classes_in_source`-Pruefung inline neben der Implementierung (Pflicht-Pattern — analog `src/page/employees.rs:18-43`).

### 2. `WeekStatusDropdown` — neu, Komponent

**Datei:** `src/component/week_status_dropdown.rs`

Schichtplaner-Aktionselement. Nutzt den bestehenden `DropdownTrigger` aus `src/component/dropdown_base.rs`. Kein `controlled <select>` (D-39-06).

```rust
#[derive(Props, Clone, PartialEq)]
pub struct WeekStatusDropdownProps {
    pub current_status: WeekStatus,
    pub year: u32,
    pub week: u8,
    pub on_change: EventHandler<WeekStatus>,
}
```

Trigger-Klassen (statischer `match`):

| Zustand | Trigger-Klasse |
|---------|---------------|
| `Unset` | `"inline-flex items-center gap-1 px-2 py-0.5 rounded-sm text-small font-medium bg-surface-alt border border-border text-ink-muted"` |
| `InPlanning` | wie `week_status_badge_class(InPlanning)` + ` gap-1` |
| `Planned` | wie `week_status_badge_class(Planned)` + ` gap-1` |
| `Locked` | wie `week_status_badge_class(Locked)` + ` gap-1` |

Dropdown-Eintrags-Reihenfolge (Kein → Locked, aufsteigend nach Verbindlichkeit):
1. „Kein" → `on_change.call(WeekStatus::Unset)`
2. „In Planung" → `on_change.call(WeekStatus::InPlanning)`
3. „Geplant" → `on_change.call(WeekStatus::Planned)`
4. „Gesperrt" → `on_change.call(WeekStatus::Locked)`

Tastatur: Standard-HTML-Button-Semantik des `DropdownTrigger` (Enter/Space offnet, Tab navigiert Eintraege, Enter wahlt aus) — kein Custom-Focus-Management.

`aria-label` auf dem Trigger-Button: `"Wochenstatus andern"` (de) / `"Change week status"` (en) / `"Zmenit stav tydne"` (cs). Separater Key `WeekStatusChangeAriaLabel`.

### 3. Service: `src/service/week_status.rs` — neu

```rust
pub static WEEK_STATUS_STORE: GlobalSignal<WeekStatusStore> =
    Signal::global(|| WeekStatusStore::default());

#[derive(Clone, Default)]
pub struct WeekStatusStore {
    pub status: WeekStatus,          // WeekStatus::Unset als Default
    pub loaded_year: Option<u32>,
    pub loaded_week: Option<u8>,
}

pub enum WeekStatusAction {
    Load { year: u32, week: u8 },
    Set  { year: u32, week: u8, status: WeekStatus },
}
```

Nach-Mutation-Flow (D-39-06, kein optimistisches Signal):

```
Set empfangen
  → api::set_week_status(config, year, week, status).await
    → Ok  → WeekStatusAction::Load { year, week } selbst senden
    → Err → error_handler(e) (ERROR_STORE, kein Store-Update)

Load empfangen
  → api::get_week_status(config, year, week).await
    → Ok  → WEEK_STATUS_STORE.write().status = parsed_status
             WEEK_STATUS_STORE.write().loaded_year = Some(year)
             WEEK_STATUS_STORE.write().loaded_week = Some(week)
    → Err → error_handler(e) (Store bleibt auf letztem Wert)
```

Coroutine-Registrierung in `src/app.rs`:
```rust
use_coroutine(service::week_status::week_status_service);
```

### 4. State: `src/state/week_status.rs` — neu

```rust
#[derive(Clone, PartialEq, Debug, Default)]
pub enum WeekStatus {
    #[default]
    Unset,
    InPlanning,
    Planned,
    Locked,
}
```

`From<&WeekStatusTO>` fur TO-Konvertierung. `TryFrom<&str>` fur manuelles Diskriminant-Parsing in DAO (analog `special_day`/`extra_hours` TEXT-Enum-Muster, D-39-10).

Re-Export in `src/state/mod.rs`.

---

## Placement Contract: Wo im Schichtplan

**Integrationsort:** `src/page/shiftplan.rs`, im `rsx!`-Block.

**Ladezeitpunkt:** `WeekStatusAction::Load { year, week }` wird gesendet wenn:
- Die Seite initial mit year/week rendert (in `use_effect` analog `SHIFTPLAN_REFRESH`).
- year oder week durch Navigation andert sich (im `ShiftPlanAction::NextWeek` / `PreviousWeek`-Handler des Coroutine).

**Platzierung im Render-Baum:**

```
TopBar {}
div { class: "px-4 py-3 print:hidden",           // Toolbar
    div { class: "flex flex-wrap items-center..." }
}
// ... ShiftplanTabBar, Konflikte, Overage-Warnung ...
// Week-View-Bereich:
if view_mode == ViewMode::Week {
    div { class: "m-4",
        SlotEdit {}

        // NEU: Status-Strip oberhalb der WeekView-Karte
        div { class: "mb-3 flex items-center gap-2 print:hidden",
            if is_shiftplanner {
                WeekStatusDropdown {
                    current_status: week_status,
                    year: *year.read(),
                    week: *week.read(),
                    on_change: move |new_status| {
                        week_status_service.send(WeekStatusAction::Set {
                            year: *year.read(),
                            week: *week.read(),
                            status: new_status,
                        });
                    },
                }
            } else if week_status != WeekStatus::Unset {
                WeekStatusBadge { status: week_status }
            }
        }

        WeekView { /* unverandert */ }
    }
}
```

**Sichtbarkeitsmatrix:**

| Rolle | Status = Unset | Status gesetzt |
|-------|----------------|----------------|
| Nicht-Schichtplaner | kein Element (D-39-05) | WeekStatusBadge |
| Schichtplaner | WeekStatusDropdown (zeigt „Kein") | WeekStatusDropdown |

`print:hidden`: Der Status-Strip wird beim Drucken ausgeblendet — der Schichtplan-Druck zeigt das Raster, keinen Planungsstatus-Banner.

---

## Interaction Contract

### Keyboard-Zuganglichkeit

- Trigger-Button fokussierbar per `Tab`, aktivierbar per `Enter` / `Space` (standard HTML `<button>`).
- Dropdown-Eintraege per `Tab` erreichbar, per `Enter` auswahlbar — wird durch bestehenden `DropdownTrigger` gehandelt.
- Kein Custom-Focus-Trap oder ARIA-Combobox notwendig.

### Nach-Mutation-Flow (Fresh-Fetch, D-39-06)

```
1. User klickt Dropdown-Eintrag
2. on_change.call(new_status) -> WeekStatusAction::Set { year, week, status: new_status }
3. week_status_service verarbeitet Set:
   a. api::set_week_status(config, year, week, new_status) [API-Call]
   b. Erfolg: sofort api::get_week_status(config, year, week) [Reload]
   c. WEEK_STATUS_STORE.write().status = frischer Wert vom Server
   d. Dioxus re-rendert Badge / Dropdown-Trigger mit neuem Wert
   e. Fehler: error_handler -> ERROR_STORE (existierende error_view zeigt Toast)
```

Kein optimistisches Update vor Server-Roundtrip.

### Ladezustand beim KW-Wechsel

Wahrend `WeekStatusAction::Load` lauft, enthalt `WEEK_STATUS_STORE.status` den Wert `WeekStatus::Unset` (Default). Damit:
- Nicht-Schichtplaner sehen kein Element.
- Schichtplaner sehen Dropdown-Trigger mit „Kein" (neutral).

Kein expliziter Lade-Spinner — der kurze Flicker ist akzeptiert (analog KW-Wechsel-Verhalten aller anderen Stores im Schichtplan).

### Fehlerzustand

Bei fehlgeschlagenem API-Call (`Set` oder `Load`) wird `error_handler(e)` aufgerufen. Die bestehende globale `error_view`-Komponente zeigt eine nicht-blockierende Fehlermeldung via `ERROR_STORE`. Kein inline Fehlerstatus im Badge selbst.

---

## API Contract (Frontend-Seite)

Neue API-Funktionen in `src/api.rs`:

```rust
pub async fn get_week_status(config: Config, year: u32, week: u8)
    -> Result<Option<WeekStatusTO>, reqwest::Error>

pub async fn set_week_status(config: Config, year: u32, week: u8, status: WeekStatus)
    -> Result<(), reqwest::Error>
```

REST-Pfad: `GET /week-status/{year}/{week}`, `PUT /week-status/{year}/{week}` (mit Body `{ "status": "InPlanning" }`). Analog `/week-message/{year}/{week}`.

Proxy-Eintrag in `Dioxus.toml`: `[[web.proxy]] backend = "http://localhost:3000/week-status"` (neuer Eintrag).

---

## Registry Safety

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| shadcn official | nicht verwendet | not applicable |
| Drittanbieter | keine | not applicable |

Phase 39 fuhrt keine neuen externen Abhaengigkeiten ein. Alle neuen Komponenten sind codebase-nativ (Dioxus + Tailwind).

---

## Assumptions (autonomous — Claude's Discretion)

Folgende Entscheidungen wurden ohne interaktive Nutzerfrage getroffen (CONTEXT delegiert diese an „Claude's Discretion"):

1. **Platzierungsort:** Status-Strip als `div { class: "mb-3 flex items-center gap-2 print:hidden" }` direkt vor `WeekView` in der `m-4`-div — nicht in der Toolbar-Zeile. Die Toolbar ist bereits dicht belegt (Navigation, View-Toggle, iCal, Booking-Log, Structure-Modus-Dropdown). Eine eigene Zeile gibt dem Status visuelle Eigenstandigkeit.

2. **Badge-Form:** Kompaktes Inline-Pill (`rounded-sm`, `px-2 py-0.5`, `text-small font-medium`) mit Hintergrundfarbe und Border — analog den bestehenden Warning-/Error-Badges im Codebase, aber kleiner als die Banner-Boxes (die `px-4 py-3` nutzen).

3. **Dropdown-Trigger-Klasse bei Unset:** Neutrale Darstellung (`bg-surface-alt border-border text-ink-muted`) — nicht farbig, signalisiert „noch kein Status gesetzt", ohne die Negativ-Assoziationen der Statusfarben zu wecken.

4. **Eintragsreihenfolge im Dropdown:** Kein → In Planung → Geplant → Gesperrt (aufsteigend nach Verbindlichkeit). Logische Progressionsreihenfolge erleichtert den Arbeitsfluss.

5. **Neue Dateien:** `WeekStatusBadge` als Atom in `src/component/atoms/week_status_badge.rs`; `WeekStatusDropdown` in `src/component/week_status_dropdown.rs` (nicht Atom-Ebene, da es einen EventHandler nach oben gibt). Service in `src/service/week_status.rs`. State in `src/state/week_status.rs`.

6. **REST-Pfad:** `/week-status/{year}/{week}` (Analogie zu `/week-message/{year}/{week}`). Exakte HTTP-Verben (GET/PUT/DELETE vs. GET/POST) werden bei der Implementierung analog `week_message` ubernommen.

7. **print:hidden:** Status-Strip wird beim Drucken ausgeblendet.

8. **Aria-Label als eigener i18n-Key:** `WeekStatusChangeAriaLabel` mit festem deutschen Wert „Wochenstatus andern" — kein Placeholder benotigt, da der zugehorige Trigger bereits den Label-Text als sichtbaren Text tragt.

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS
- [ ] Dimension 2 Visuals: PASS
- [ ] Dimension 3 Color: PASS
- [ ] Dimension 4 Typography: PASS
- [ ] Dimension 5 Spacing: PASS
- [ ] Dimension 6 Registry Safety: PASS

**Approval:** pending
