# Phase 9: Booking-Flow Reverse-Warnings (+ Copy-Week DESCOPED) - Context

**Gathered:** 2026-06-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Frontend-only Phase im Monorepo (`shifty-dioxus/`). Die **einzige** Booking-Call-Site
des Shiftplan-Editors wird vom alten `POST /booking` auf den konflikt-bewussten
`POST /shiftplan-edit/booking` umgestellt. Die zurückgelieferten Reverse-Warnings
(`BookingCreateResultTO.warnings[]`) werden über einen Dioxus-Dialog (statt
`window.confirm`) angezeigt, mit **Optimistic-Create-plus-Rollback**-Semantik. Der alte
`POST /booking` bleibt parallel verfügbar (Regression-Lock, grep-verifiziert).

**Copy-Week ist aus Phase 9 DESCOPED.** Das Feature wurde vom User bewusst per Commit
`294566f` ("feat: Remove copy last week feature") entfernt; übrig blieb nur toter Code.
Requirement **FUI-A-06 → dropped/superseded**. Der tote Copy-Week-Code wird im Rahmen
dieser Phase aufgeräumt. Backend-Endpoint `POST /shiftplan-edit/copy-week` +
`CopyWeekResultTO` bleiben unangetastet (Backend behält die Fähigkeit, kein
Frontend-Konsument mehr).

**Kein neues Backend** — alle DTOs/Endpoints existieren bereits (v1.0 Phase 3).

**Out of Scope:**
- Jegliche Copy-Week-UI (descoped; Backend-Fähigkeit bleibt für eine etwaige spätere Phase)
- From/To-Wochen-Picker (war Copy-Week-bezogen)
- Neue Backend-Endpoints oder DTO-Änderungen
- Self-Booking-Pfade außerhalb des Editors (es existiert keiner)
- Neue 403-UX (bestehendes Silent-Swallow bleibt)

</domain>

<decisions>
## Implementation Decisions

### Booking-Endpoint-Umstellung
- **D-01:** Die einzige Editor-Booking-Call-Site (`page/shiftplan.rs:409`
  `ShiftPlanAction::AddUserToSlot` → `loader::register_user_to_slot` →
  `api::add_booking`, `api.rs:197`) wird auf `POST /shiftplan-edit/booking` umgestellt.
  Die API-Funktion liefert künftig `BookingCreateResultTO { booking, warnings }` statt
  `()`. **Es existiert keine weitere Booking-Call-Site** (verifiziert via grep über
  `add_booking`/`register_user_to_slot`).
- **D-02:** Alter `POST /booking` (`rest/src/booking.rs:100` `create_booking`) bleibt
  unverändert. Regression-Lock per grep-Check (alte Handler + Route unangetastet) —
  Success-Criterion 3.

### Confirm-Dialog-Semantik (Optimistic + Rollback)
- **D-03:** Der neue Endpoint ist **kein Dry-Run** — er persistiert die Buchung sofort
  (201) und liefert `warnings` im selben Response-Body. Flow:
  - POST bucht → `warnings` leer ⇒ fertig (`update_shiftplan()` reload).
  - `warnings` vorhanden ⇒ Dioxus-Dialog (`component/dialog.rs`).
    - **"Abbrechen"** → `api::remove_booking(result.booking.id)` (DELETE, existiert
      `api.rs:227`) → reload.
    - **"Trotzdem buchen"** → behalten → reload.
  - `BookingTO.id` ist im Result vorhanden (rest-types) → Rollback-ID direkt verfügbar.
- **D-04:** **Rollback-Fehlerbehandlung:** Schlägt der DELETE beim "Abbrechen" fehl →
  Fehler über `error_handler`/Toast anzeigen **und** `update_shiftplan()` (echter
  Zustand wird sichtbar, evtl. verwaiste Buchung erkennbar). Kein Silent-Swallow für
  diesen Pfad.

### Copy-Week — DESCOPED + Cleanup
- **D-05:** Copy-Week vollständig aus Phase 9 gestrichen. FUI-A-06 → `dropped/superseded`.
- **D-06:** Toter-Code-Cleanup **in Scope**: `ShiftPlanAction::CopyFromPreviousWeek`
  (`shiftplan.rs:66`) + Handler (`shiftplan.rs:523`), `api::copy_week` (`api.rs:237`),
  `loader::copy_from_previous_week` (`loader.rs:318`) sowie der ungenutzte i18n-Key
  `ShiftplanTakeLastWeek` (inkl. der drei Locale-Strings en/de/cs) entfernen.
- **D-07:** Backend `POST /shiftplan-edit/copy-week` (`rest/src/shiftplan_edit.rs:173`) +
  `CopyWeekResultTO` (rest-types) **bleiben unangetastet** — Backend behält die
  Fähigkeit für eine etwaige spätere Phase.
- **D-08:** ROADMAP (Phase-9-Titel + SC2) und REQUIREMENTS (FUI-A-06) werden im Rahmen
  dieser Phase nachgezogen — **vom User autorisiert** als Teil der Descope-Entscheidung.

### Warning-Anzeige-Komponente
- **D-09:** Die bestehende `AbsenceWarningDisplay` (`page/absences.rs:195`) wird (a) um die
  Reverse-Varianten `BookingOnAbsenceDay` + `BookingOnUnavailableDay` sowie
  `PaidEmployeeLimitExceeded` erweitert und (b) aus `absences.rs` nach `component/`
  herausgelöst (geteilte Komponente). `shiftplan.rs` (Dialog) und `absences.rs` nutzen
  sie gemeinsam — single source of truth fürs Warning-Rendering. `WarningsList`-Newtype-
  PartialEq-Wrapper-Pattern (Plan 08-05) beibehalten, falls Props-`PartialEq` nötig.
- **D-10:** **Nur die auf dem Booking-Pfad emittierten Varianten** sind hier relevant:
  `BookingOnAbsenceDay`, `BookingOnUnavailableDay`, `PaidEmployeeLimitExceeded`. Die
  Forward-Varianten (`AbsenceOverlapsBooking`, `AbsenceOverlapsManualUnavailable`)
  kommen auf dem Booking-Pfad **nicht** vor (die behandelt absences.rs weiterhin).

### Warning-Texte
- **D-11:** Detailgrad **Person + Datum + Grund** (z.B. "Maria ist am 24.12. im Urlaub.").
  Erfordert Personname-Auflösung im Render; im Confirm-Dialog ist die gebuchte Person
  aus dem Action-Kontext bekannt. Personname via Sales-Person-Liste joinen
  (Side-Join-Pattern aus Plan 08-04 / `loader`). Die Varianten tragen
  `booking_id`/`date`/`category` bzw. `year`/`week`/`day_of_week`.

### Dialog-Wording
- **D-12:** Buttons: primär **"Trotzdem buchen"**, sekundär **"Abbrechen"**. Header
  Singular/Plural mit `{count}`-Interpolation analog `AbsenceWarningHeaderSingular`/
  `…Plural`. Alle neuen Texte in De/En/Cs.

### 403/422-Verhalten
- **D-13:** **403** (kein Buchungs-Privileg) weiter **still schlucken** — konsistent mit
  dem bestehenden `AddUserToSlot`-Handler (`shiftplan.rs:~424`). **422** (Validation)
  über `error_handler` als Fehler. Keine neue 403-UX.

### Test-Strategie
- **D-14:** **Voller Test-Satz analog Phase 8:** SSR-Snapshots (Render je Warning-Variante,
  leeres-Array ⇒ kein Dialog, Rollback-Action-Dispatch), Render-/Helper-Tests sowie
  **Per-Locale-Reference-Matcher-Tests** für alle neuen i18n-Keys (Pitfall-2-Guard,
  Plan 08-04). i18n-Parity-Test-Gruppe (`i18n/mod.rs:422+`) erweitern.

### Claude's Discretion
- Exaktes Tailwind-Styling des Dialogs.
- Ob die geteilte Komponente `component/warning_list.rs` heißt oder
  `AbsenceWarningDisplay` umbenannt/verschoben wird.
- Genaue i18n-Key-Namen.
- Ob `api::add_booking` erweitert oder eine neue Funktion (z.B.
  `book_slot_with_conflict_check`) angelegt wird; analog die loader-Wrapper-Signatur.
- Genaue Wave-/Plan-Aufteilung (kleine Phase — vermutlich 1–2 Plans).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase-Definition & Requirements
- `.planning/ROADMAP.md` — Phase 9 (SC1 live, SC2 wird descoped; Titel nachziehen)
- `.planning/REQUIREMENTS.md` — `FUI-A-05` (live), `FUI-A-06` (→ dropped/superseded)

### Backend-Verträge (existieren, keine Änderung)
- `rest/src/shiftplan_edit.rs:134` — `book_slot_with_conflict_check` (`POST /shiftplan-edit/booking`, 201/403/422)
- `rest/src/shiftplan_edit.rs:173` — `copy_week_with_conflict_check` (bleibt, kein FE-Konsument)
- `rest/src/booking.rs:100` — `create_booking` (`POST /booking`, **muss unverändert bleiben**)
- `rest-types/src/lib.rs` — `WarningTO` (~1757–1911, 5 Varianten), `BookingCreateResultTO`
  (~1912–1925), `CopyWeekResultTO` (~1929–1942), `BookingTO` (inkl. `id`, `$version`)

### Frontend (zu ändern)
- `shifty-dioxus/src/api.rs:197` (`add_booking`), `:227` (`remove_booking`), `:237` (`copy_week` → löschen)
- `shifty-dioxus/src/loader.rs:286` (`register_user_to_slot`), `:318` (`copy_from_previous_week` → löschen)
- `shifty-dioxus/src/page/shiftplan.rs:409` (`AddUserToSlot`), `:66`/`:523` (`CopyFromPreviousWeek` → löschen)
- `shifty-dioxus/src/component/dialog.rs` — Dialog-Primitive (Footer-Buttons, ESC/Backdrop)
- `shifty-dioxus/src/page/absences.rs:195` — `AbsenceWarningDisplay` (erweitern + nach `component/` teilen)
- `shifty-dioxus/src/i18n/{mod.rs,en.rs,de.rs,cs.rs}` — `Key`-Enum + 3 Locales; bestehende
  `AbsenceWarning*`-Keys (Header-Singular/Plural-Pattern als Vorlage)

### Konventionen & Codebase-Maps
- `shifty-backend/CLAUDE.md` (Service-Tier, jj-only, Snapshot-Versioning — hier nicht berührt)
- `shifty-dioxus/CLAUDE.md` (Frontend-Konventionen, WASM-Build-Gate)
- `.planning/codebase/frontend/{STRUCTURE,CONVENTIONS,ARCHITECTURE}.md`

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `component/dialog.rs`: fertige Modal-Primitive (Center/Sheet/Bottom/Auto, ESC + Backdrop,
  Scroll-Lock, Footer-Slot mit `Btn`-Buttons) — direkt für den Confirm-Dialog nutzbar.
- `api::remove_booking(config, booking_id)` (`api.rs:227`): existierender DELETE-Pfad für
  den Rollback.
- `AbsenceWarningDisplay` (`absences.rs:195`): Warning-Listen-Renderer mit gelb-umrandeter
  Box, Singular/Plural-Header, `{date}`-Interpolation — Basis für die geteilte Komponente.
- `error_handler`/`result_handler` (`error.rs`): bestehendes Fehler-Toast-Handling.
- Personname-/Background-Color-Side-Join im `loader` (Plan 08-04-Pattern) für D-11.
- `WarningsList`-Newtype-PartialEq-Wrapper (Plan 08-05) für Props ohne `WarningTO: PartialEq`.

### Established Patterns
- **Non-blocking Warnings** (`{ booking, warnings }`-Wrapper, v1.0).
- **Single-File-Page-Composition** + Inline-Helper-Components (Phase 8) — aber hier
  explizit geteilte Komponente in `component/` (D-09).
- **Per-Locale-Reference-Matcher-Tests** gegen Pitfall 2 (Plan 08-04).
- **SSR-Snapshot-Tests** via `VirtualDom` + `dioxus-ssr` (Phase 8); `pin_de_locale()`-Hook
  für Locale-spezifische Snapshots.
- **403-Silent-Swallow** im `AddUserToSlot`-Handler (Vorlage für D-13).

### Integration Points
- `shiftplan.rs` `AddUserToSlot`-Handler: Result auswerten, Dialog triggern, Rollback dispatchen.
- `api.rs` `add_booking`: Signatur-Änderung auf `BookingCreateResultTO`-Return.
- `i18n` `Key`-Enum: neue Keys für Dialog-Titel/Buttons + 3 Reverse-Warning-Texte.

</code_context>

<specifics>
## Specific Ideas

- Der User hat Copy-Week **bewusst** entfernt (Commit `294566f`) — das ist eine
  Produkt-Entscheidung, nicht versehentlich. Re-Introduction wäre ein eigenes Vorhaben.
- "Person + Datum + Grund"-Texte sollen konkret klingen, z.B. "Maria ist am 24.12. im
  Urlaub." / "Maria ist am Mo (KW 12) als nicht verfügbar markiert." /
  "Bezahlt-Limit überschritten (2/3)."
- Reverse-Warnings auf dem Booking-Pfad: nur `BookingOnAbsenceDay`,
  `BookingOnUnavailableDay`, `PaidEmployeeLimitExceeded`.

</specifics>

<deferred>
## Deferred Ideas

- **Copy-Week mit Konflikt-Awareness (UI-Reaktivierung):** Backend
  `POST /shiftplan-edit/copy-week` + `CopyWeekResultTO` bleiben funktionsfähig. Falls je
  wieder eine Copy-Week-UI gewünscht ist (ob "Vorwoche → aktuell" oder From/To-Picker mit
  aggregierter Warning-Anzeige), wäre das eine eigene Phase. FUI-A-06 ist vorerst dropped.
- **Optionale Migration von `absences.rs` auf die geteilte Warning-Komponente** über das
  in Phase 9 Nötige hinaus — nur falls sich Re-Use jenseits der zwei Surfaces lohnt.

</deferred>

---

*Phase: 09-booking-flow-reverse-warnings-copy-week*
*Context gathered: 2026-06-11*
