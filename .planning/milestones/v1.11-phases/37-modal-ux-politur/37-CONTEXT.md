# Phase 37: Modal-UX-Politur (FE) - Context

**Gathered:** 2026-07-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Zwei gemeldete Frontend-UX-Punkte am Modal-Verhalten beheben (reine FE-Phase):

- **MOD-01:** Ein innerhalb eines Modals begonnener Maus-Drag (Text-Selektion), der
  außerhalb losgelassen wird, schließt das Modal **nicht** mehr. Nur ein echter
  Außerhalb-Klick (mousedown **und** mouseup auf dem Backdrop) schließt.
- **MOD-02:** Das Arbeitsvertrag-Modal (`contract_modal.rs`) zeigt unter jedem relevanten
  Feld einen kurzen Erklärungssatz (Muster `CapPlannedHoursHelp`), Von/Bis ausgenommen.

**Keine neuen Fähigkeiten.** Kein Snapshot-Bump (bleibt 12), keine Migration, keine neuen
Deps. i18n de/en/cs Pflicht (MOD-02 bringt neue Text-Keys). Requirements-Zielverhalten ist
in REQUIREMENTS.md fixiert; diese Phase klärt nur den Implementierungsweg.

</domain>

<decisions>
## Implementation Decisions

### MOD-01 — Backdrop-Close-Fix (Drag)
- **D-01:** Fix-Mechanik = **Signal-Flag-Muster** (kein `web_sys`-target-Vergleich).
  Dioxus' `MouseData` exponiert kein `target`/`currentTarget`. Umsetzung: `onmousedown`
  am Backdrop setzt ein `use_signal`-Flag `pressed_on_backdrop = true`; der Panel-Inhalt
  verhindert per `stop_propagation` (bzw. eigenem `onmousedown`), dass ein Panel-Ursprung
  das Flag setzt; der Backdrop-Close feuert `on_close` **nur wenn** das Flag gesetzt ist,
  und setzt es danach zurück. (Der bestehende Backdrop schließt aktuell bei jedem `onclick`
  ohne Ursprungs-Tracking, `dialog.rs:194`.)
- **D-02:** Fix **zentral** in der geteilten `Dialog`-Komponente (`dialog.rs`) — deckt
  automatisch `contract_modal`, `extra_hours_modal`, `slot_edit` und alle weiteren
  `Dialog`-Nutzer ab.
- **D-03:** `absence_convert_modal.rs:88-91` hat einen **eigenen** Backdrop
  (`fixed inset-0 bg-modal-veil`, nicht via `dialog.rs`) mit demselben Bug — dieser wird
  im **selben** Phasen-Scope **identisch mitgefixt** (gleiches Signal-Flag-Muster inline).
  Nicht weglassen, sonst bleibt ein bekannter Duplikat-Bug stehen.
- **D-04:** ESC-Dismissal (`use_escape_dismiss`, `dialog.rs:315-333`) bleibt unberührt —
  unabhängig vom Drag-Bug.

### MOD-02 — Arbeitsvertrag-Modal Help-Texte
- **D-05:** Pro Feld ein neuer `*Help`-i18n-Key analog `CapPlannedHoursHelp`. Rendering als
  **Sibling-`span`** mit den exakten cap-Klassen `text-small font-normal text-ink-muted`
  (minimal & konsistent zum bestehenden Muster `contract_modal.rs:427`). **Kein** Umbau des
  `Field`-Atoms auf einen help-Slot.
- **D-06:** Neue Keys + de-Texte (verbatim, Von/Bis ausgenommen; en/cs werden übersetzt):

  | Feld (Label-Key) | neuer Help-Key | Text (de) |
  |---|---|---|
  | `WorkdaysLabel` | `WorkdaysHelp` | „Die Tage, an denen die Person in der Regel arbeitet." |
  | `ExpectedHoursPerWeekLabel` | `ExpectedHoursPerWeekHelp` | „Wie viele Sollstunden pro Woche." |
  | `DaysPerWeekLabel` | `DaysPerWeekHelp` | „An wie vielen Tagen die Person in der Regel reinkommt." |
  | `VacationEntitlementsPerYearLabel` | `VacationEntitlementsPerYearHelp` | „Der gesamte Jahresurlaub laut Vertrag im Jahr." |
  | `DynamicHourLabel` | `DynamicHourHelp` | „Das Soll entspricht immer den geleisteten Stunden — ideal, wenn die Person nach Stunden bezahlt wird." |
  | `CommittedVoluntaryLabel` | `CommittedVoluntaryHelp` | „Zugesagte freiwillige Stunden." |
  | `FromLabel` / `ToLabel` | — | (kein Help — selbsterklärend) |

- **D-07:** `CommittedVoluntary`-Help bekommt Text **„Zugesagte freiwillige Stunden."**
  (User-Entscheidung, bewusst knapp) und wird **innerhalb** des `if show_committed`-Blocks
  gerendert (`contract_modal.rs:368-391`), sodass er nur mit dem Feld erscheint.
- **D-08:** `CapPlannedHoursHelp` existiert bereits — unverändert lassen (nur als Vorbild).
- **D-09:** Pro neuem Key **4 Dateien**: Enum-Variante in `i18n/mod.rs` (nahe dem
  jeweiligen `*Label`) + je ein `add_text(...)` in `i18n/de.rs`, `i18n/en.rs`, `i18n/cs.rs`.
  Alle drei Locales Pflicht (sonst stiller Fallback laut CLAUDE.md). Dann in
  `contract_modal.rs` je `let x_help = ImStr::from(i18n.t(Key::XHelp).as_ref());` auflösen.

### Test-/Verifikations-Strategie
- **D-10:** cargo/SSR-Tests sind die harten Gates (Carry-forward Phase 36, D-25-06):
  - MOD-01: **strukturell** über die Prädikat-/Handler-Logik testen (Drag-innen→mouseup-außen
    lässt Modal offen). Maus-Drag ist im Browser nicht zuverlässig automatisierbar →
    Handler-Logik statt Browser-Automation.
  - MOD-02: **SSR-Test** — Help-Texte werden unter den jeweiligen Feldern gerendert (in allen
    drei Locales prüfbar).
- **D-11:** Standard-Gates: `cargo build`, `cargo test -p shifty-dioxus`, WASM-Build
  (`cargo build --target wasm32-unknown-unknown` aus `shifty-dioxus/`); Backend bleibt
  `cargo clippy --workspace -- -D warnings` grün.

### Claude's Discretion
- Exakte en/cs-Übersetzungen der neuen `*Help`-Keys (fachlich sinngemäß zu den de-Texten).
- Ob das Signal-Flag als `use_signal` in `DialogContent` oder via eigenem `onmousedown` +
  `stop_propagation` am Panel realisiert wird — beides erfüllt D-01.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Roadmap
- `.planning/REQUIREMENTS.md` §Modal-UX (MOD) — MOD-01/MOD-02 Zielverhalten (locked)
- `.planning/ROADMAP.md` §Phase 37 — Scope + Success-Kriterien
- `.planning/todos/2026-06-30-modal-schliesst-bei-mouseup-ausserhalb-nach-drag.md` — MOD-01 Ursprung/Repro
- `.planning/todos/2026-06-30-arbeitsvertrag-modal-erklaerungssatz-pro-feld.md` — MOD-02 Ursprung + de-Texte

### MOD-01 — Dialog/Backdrop
- `shifty-dioxus/src/component/dialog.rs:113-132` — `DialogProps` (open/on_close/title/…)
- `shifty-dioxus/src/component/dialog.rs:190-253` — Backdrop-`div` (onclick-close `:194`) + Panel (`stop_propagation` `:204`); **hier** Signal-Flag ergänzen
- `shifty-dioxus/src/component/dialog.rs:315-333` — `use_escape_dismiss` (unberührt lassen)
- `shifty-dioxus/src/component/absence_convert_modal.rs:88-91` — **eigener** Backdrop, identischer Bug → separat mitfixen (D-03)
- `shifty-dioxus/src/component/overlay.rs:13` — generisches Overlay OHNE Close-Handler (nicht betroffen)

### MOD-02 — Contract-Modal + i18n
- `shifty-dioxus/src/component/contract_modal.rs:135-146` — Label-Auflösung; `CapPlannedHoursHelp` `:144`
- `shifty-dioxus/src/component/contract_modal.rs:410-428` — bestehendes Help-Rendering-Muster (`span text-small font-normal text-ink-muted` `:427`)
- `shifty-dioxus/src/component/contract_modal.rs:179,200,224,303,325,349,368-391,394-409` — Felder From/To/Workdays/ExpectedHours/DaysPerWeek/Vacation/CommittedVoluntary(cond.)/DynamicHour
- `shifty-dioxus/src/i18n/mod.rs:150-151,272` — `Key`-Enum-Variante-Muster (`CapPlannedHoursHelp`, `CommittedVoluntaryLabel`)
- `shifty-dioxus/src/i18n/de.rs:184-188`, `i18n/en.rs:156-160`, `i18n/cs.rs:180-184` — `add_text`-Muster für ein `*Help`-Key über alle Locales

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Eine** geteilte `Dialog`-Komponente (`dialog.rs:138`, delegiert an `DialogContent:172`) →
  zentraler MOD-01-Fix deckt die meisten Modals in einem Zug ab.
- Bestehendes `CapPlannedHoursHelp`-Rendering (`contract_modal.rs:427`) → 1:1-Vorlage für die
  neuen MOD-02-Help-Spans (Klassen + Key-Struktur).
- `dialog.rs` nutzt bereits `#[cfg(target_arch=wasm32)]`-`web_sys`-Blöcke (Scroll-Lock/ESC) →
  falls doch web_sys nötig wäre, ist der Rahmen da; per D-01 aber Signal-Flag bevorzugt.

### Established Patterns
- i18n: `Key`-Enum + `add_text(Locale, Key, "…")` je Locale; alle drei Pflicht (stiller
  de-Fallback sonst).
- Modal-Close via `on_close: EventHandler<()>`; Panel schützt sich per `evt.stop_propagation()`.

### Integration Points
- MOD-01: Änderung in `dialog.rs` (Backdrop-Handler) + spiegelbildlich in
  `absence_convert_modal.rs:88-91`. Prop-Contract von `Dialog` bleibt unverändert.
- MOD-02: neue Enum-Varianten in `i18n/mod.rs`, `add_text` in de/en/cs, Render-Spans in
  `contract_modal.rs`. Keine Signatur-/Prop-Änderungen an geteilten Atomen (D-05).

</code_context>

<specifics>
## Specific Ideas

- MOD-02-de-Texte sind wörtlich vorgegeben (Todo + D-06/D-07); nicht umformulieren.
  `CommittedVoluntaryHelp` = „Zugesagte freiwillige Stunden." (bewusst knapp, User-Wortlaut).
- Zielverhalten „mousedown UND mouseup auf dem Backdrop schließt; innen begonnener Drag
  schließt nicht" (MOD-01) darf nicht aufgeweicht werden.

</specifics>

<deferred>
## Deferred Ideas

None — Diskussion blieb innerhalb des Phasen-Scopes (MOD-01/MOD-02). Build-Hygiene
(HYG-01/02) → Phase 38 laut Roadmap.

</deferred>

---

*Phase: 37-modal-ux-politur*
*Context gathered: 2026-07-01*
