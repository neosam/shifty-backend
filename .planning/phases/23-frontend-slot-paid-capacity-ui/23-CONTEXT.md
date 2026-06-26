# Phase 23: Frontend — Slot Paid-Capacity UI - Context

**Gathered:** 2026-06-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Reine **Frontend-Phase** in `shifty-dioxus` — kein Backend-/API-/DTO-Change. Zwei
zusammenhängende UI-Lieferungen rund um das bereits existierende Feld
`max_paid_employees` (Slot-weites Limit bezahlter Mitarbeiter:innen pro Woche,
`Option<u8>`, `None` = kein Limit):

- **FUI-02 — Capacity-Editor:** Im Slot-Editor-Formular
  (`shifty-dioxus/src/component/slot_edit.rs`, `SlotEditInner`) ein Eingabefeld für
  `max_paid_employees` ergänzen. Heute wird das Feld beim Edit-Roundtrip nur
  durchgereicht (`state/slot_edit.rs:22` „not edited in v1.2 UI but preserved"), aber
  nicht editiert. NULL = kein Limit muss eingebbar sein.
- **FUI-01 — Warn-Färbung im Week-View:** In der Schichtplan-Wochenansicht
  (`shifty-dioxus/src/component/week_view.rs`, `WeekCellSlot`) die Slot-Zelle visuell
  warnen, wenn `current_paid_count > max_paid_employees` (nur wenn ein Limit gesetzt
  ist). Buchen bleibt erlaubt (Backend ist nicht-blockierend) — die Färbung ist reine
  Sichtbarkeit.

**Backend ist vollständig vorhanden** (v1.1 / Phase 5, verifiziert): `SlotTO.max_paid_employees`,
`ShiftplanSlotTO.current_paid_count`, `WarningTO::PaidEmployeeLimitExceeded` werden über
REST geliefert; der Week-View-State (`state/shiftplan.rs:Slot`) spiegelt beide Felder
und wird vom Loader echt befüllt.

</domain>

<decisions>
## Implementation Decisions

### Capacity-Editor (FUI-02)
- **D-23-01 (Eingabefeld + NULL-Semantik):** Neues Feld für `max_paid_employees` im
  `SlotEditInner`-Formular, eingefügt nach dem bestehenden `min_resources`-Feld
  (`slot_edit.rs:185-202` als Vorlage: `Field` + number-Input, `min: "0"`). **Leeres
  Feld = `None` = kein Limit**, eine eingegebene Zahl setzt das Limit (`Some(u8)`).
  Eigener i18n-Label + dezenter Hinweis „leer = kein Limit" in allen 3 Locales.
  *(Default-Entscheidung — „NULL-Eingabe im Editor" wurde vom User nicht zur Diskussion
  gewählt; leeres-Feld-Pattern ist die natürlichste Abbildung von `Option<u8>` und
  vermeidet einen 0-Sentinel/Checkbox-Overhead.)*
- **D-23-02 (Inline-Hinweis bei zu niedrigem Limit, nicht blockierend):** Wenn der
  eingegebene `max_paid_employees`-Wert **unter** dem aktuellen `current_paid_count`
  des Slots liegt, zeigt der Editor ein **nicht-blockierendes** Inline-Warn-Banner
  (z.B. „aktuell X bezahlt, Limit Y") — Speichern bleibt möglich, das Backend erlaubt
  es ohnehin. Konsistent mit der User-Präferenz „Inline-Warnungen statt
  Bestätigungs-Dialoge". Kein Dialog, kein Disable des Save-Buttons.

### Warn-Färbung Week-View (FUI-01)
- **D-23-03 (eigene Farbe, kein Zahlen-Badge):** Bei `max_paid_employees = Some(n)` und
  `current_paid_count > n` wird die Slot-Zelle in einer **eigenen, deutlich anderen
  Farbe** als die bestehende Unterbesetzungs-Farbe (`warn-soft`/orange) eingefärbt —
  Richtung **`bad`/rot als Zell-Hintergrund**. **Keine zusätzliche Zahl/Badge** in der
  Zelle (kein „paid X/Y") — die Farbe ist das einzige Signal. Das bestehende
  „filled/need"-Badge (`week_view.rs:1069-1073`) bleibt unverändert.
- **D-23-04 (Vorrang bei gleichzeitiger Unterbesetzung):** Ein Slot kann gleichzeitig
  unterbesetzt (`filled < min_resources` → heute `warn-soft`) **und** über dem
  Paid-Limit sein. In diesem Fall hat die **Paid-Überschreitung Vorrang** und bestimmt
  den Hintergrund (`bad`/rot). Die bestehende `cell_background_class(missing, discourage)`-
  Logik (`week_view.rs:1037`) muss um diesen Fall erweitert werden, ohne die
  `discourage`-Behandlung zu brechen.
- **D-23-05 (sichtbar für alle Rollen):** Die Warn-Färbung ist **nicht** auf
  `is_shiftplanner` beschränkt — **alle Nutzer** sehen sie. Begründung des Users:
  Es sind primär bezahlte Mitarbeiter, die sich zu viel eintragen; sie sollen den
  Effekt sofort sehen.

### Tests & i18n
- **D-23-06:** SSR-Snapshot-Tests für (a) Editor rendert das `max_paid_employees`-Feld
  mit korrektem Wert / leer bei `None`; (b) Editor zeigt Inline-Hinweis wenn Limit
  < current_paid_count; (c) Week-View-Zelle trägt die `bad`-Hintergrundklasse wenn
  `current_paid_count > max_paid_employees`, und behält `warn-soft` bei reiner
  Unterbesetzung ohne Limit-Verletzung. Neue i18n-Keys (Editor-Label, Hint, ggf.
  Inline-Hinweis) in De/En/Cs.

### Claude's Discretion
- Konkrete Tailwind-Farbtokens für die Überschreitungs-Färbung (`bad`/`bad-soft` o.ä.,
  am bestehenden Token-Set ausrichten; statische Klassen — Pitfall 5, keine
  `format!()`-Arms; ggf. Safelist-Eintrag wie bei `warn`-Tokens).
- Genaue Formulierung der Editor-Labels/Hinweise und exakte Platzierung des
  Inline-Banners im Dialog.
- Ob die Editor-Validierung (D-23-02) zusätzlich rein lokal/clientseitig den
  `current_paid_count` aus dem aktuellen Slot-State zieht oder ein eigenes Prop braucht
  (Planner/Researcher entscheidet anhand der Props von `SlotEditInner`).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Frontend-Code (verifiziert)
- `shifty-dioxus/src/component/slot_edit.rs:45-209` — `SlotEditInner`-Formular;
  `min_resources`-Feld (`:185-202`) als Vorlage für das neue Input; Footer/Save (`:103-106`).
- `shifty-dioxus/src/state/slot_edit.rs:8-79` — `SlotEditItem` (`max_paid_employees` `:22`)
  + `From<&SlotTO>` (`:60`) / `From<&SlotEditItem>` (`:77`) Roundtrip.
- `shifty-dioxus/src/component/week_view.rs:1028-1150` — `WeekCellSlot`; `filled/need`-Logik
  + `bg_class = cell_background_class(missing, discourage)` (`:1034-1037`); Badge (`:1069-1073`).
- `shifty-dioxus/src/state/shiftplan.rs:167-211` — Week-View-`Slot` (`max_paid_employees` `:177`,
  `current_paid_count` `:180`); `From<&SlotTO>` setzt count=0 (`:208`, wird vom Loader überschrieben).
- `shifty-dioxus/src/loader.rs:154-256` — `load_shift_plan` (`:172`) und `load_day_aggregate`
  (`:225`) befüllen `current_paid_count` echt aus `ShiftplanSlotTO`; `save_slot`/`create_slot` (`:705-717`).
- `shifty-dioxus/src/component/warning_list.rs:91-168` — bestehendes `PaidEmployeeLimitExceeded`-
  Banner-Rendering (`:150-161`) — Referenz/ggf. nicht zu duplizieren.

### DTOs (read-only, kein Change)
- `rest-types/src/lib.rs:307-369` — `SlotTO` (`max_paid_employees` `:321-322`).
- `rest-types/src/lib.rs:1819-1869` — `WarningTO::PaidEmployeeLimitExceeded` (`:1861-1868`).

### i18n & Styling
- `shifty-dioxus/src/i18n/mod.rs` (`Key`-Enum), `i18n/de.rs` / `en.rs` / `cs.rs` —
  bestehender Key `BookingWarningPaidLimitExceeded`; neue Editor-Keys hier ergänzen (3 Locales).
- `shifty-dioxus/tailwind.config.js` (`warn`/`warn-soft` Tokens `:33-34`, Safelist `:76-82`)
  + `shifty-dioxus/input.css` (`--warn`/`--warn-soft` light `:42-43`, dark `:69-70`) — Vorlage
  für die `bad`-Färbung; prüfen ob `bad`-Token bereits existiert/safelisted ist.

### Regeln
- `shifty-dioxus/CLAUDE.md` — i18n alle 3 Locales; Tailwind via `npx tailwindcss …`;
  WASM-Build-Gate (`cargo build --target wasm32-unknown-unknown`); statische Tailwind-Klassen
  (Pitfall 5).

### Requirements / Roadmap
- `.planning/ROADMAP.md` § Phase 23 (FE; Backend aus v1.1/Phase 5 vorhanden).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `min_resources`-Input in `slot_edit.rs:185-202` (`Field` + number-Input + `oninput`-Closure,
  die `props.on_update_slot.call(updated)` ruft) ist das exakte Muster für das neue
  `max_paid_employees`-Feld — nur mit Empty→`None`-Parsing statt `parse::<i32>()`.
- `cell_background_class(missing, discourage)` (`week_view.rs:1037`) ist die zentrale
  Hintergrund-Entscheidung der Zelle — hier muss der Paid-Overage-Fall mit Vorrang einhängen.
- `warn`/`warn-soft` Token-Setup (tailwind.config + input.css + Safelist) ist die Blaupause
  für ein analoges `bad`-Token-Setup, falls noch nicht vorhanden.

### Established Patterns
- Slot-Edit-Roundtrip läuft über `SlotEditItem` (FE-State) ↔ `SlotTO` (DTO); `max_paid_employees`
  ist bereits in beiden Richtungen verdrahtet — Editor muss es nur noch schreiben.
- Save-Flow: `slot_edit` → `loader::save_slot/create_slot` → `api::update_slot/post_slot` →
  `SHIFTPLAN_REFRESH` → `load_shift_plan` (current_paid_count neu).

### Integration Points
- Editor schreibt `max_paid_employees`; Week-View liest `current_paid_count` + `max_paid_employees`
  aus dem `Slot`-State (befüllt durch loader). Beide Felder sind end-to-end verfügbar.

</code_context>

<specifics>
## Specific Ideas

- Reihenfolge der Editor-Felder: `max_paid_employees` direkt unter `min_resources`
  (verwandtes Kapazitäts-Konzept).
- „bad/rot" als Überschreitungs-Hintergrund — bewusst stärker/anders als das orange
  `warn-soft` der Unterbesetzung, damit die zwei Zustände nicht verwechselt werden.

</specifics>

<deferred>
## Deferred Ideas

- Numerische Paid-Count-Anzeige in der Zelle („paid X/Y") — vom User bewusst **nicht**
  gewünscht; reine Färbung reicht. (Nicht umsetzen.)
- Hartes Blockieren/Disablen des Save bei Limit-Verletzung — bewusst verworfen
  zugunsten nicht-blockierender Inline-Hinweise.

None weiter — Diskussion blieb im Phasen-Scope.

</deferred>

---

*Phase: 23-frontend-slot-paid-capacity-ui*
*Context gathered: 2026-06-26*
