# Phase 24: Paid-Limit konfigurierbar & rollenbasiert durchsetzen - Context

**Gathered:** 2026-06-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Macht die Paid-Capacity-Grenze (`max_paid_employees` pro Slot/Woche) von einem rein
visuellen Soft-Hinweis (v1.1/Phase 5, Phase 23) zu einem **global konfigurierbar
durchsetzbaren** Limit. Backend **und** Frontend. Drei Roadmap-Decisions (D-24-01..03),
in der Diskussion um einen eng gekoppelten Permission-Bugfix (D-24-04) und vier
Detail-Entscheidungen (D-24-05..08) erweitert:

- **Globaler Toggle hart/weich** (D-24-01) — admin-konfigurierbar, Default = weich (keine
  Regression).
- **Rollenbasierte harte Durchsetzung** (D-24-02) — im harten Modus blockiert das Backend
  das Buchen über das Limit, außer der Akteur ist Shiftplanner.
- **Deutlichere Overage-Anzeige** im Wochenplan (D-24-03) — zusätzlich zur roten Zelle aus
  Phase 23.

**Baut auf vorhandenem Backend auf** (v1.1/Phase 5, verifiziert): `Warning::PaidEmployeeLimitExceeded`,
`count_paid_bookings_in_slot_week`, `slot.max_paid_employees`, `ShiftplanSlotTO.current_paid_count`,
`book_slot_with_conflict_check` sind alle vorhanden. Das vorhandene **soft**-Warning-Verhalten
bleibt der Default-Modus unverändert.

**Liefert NICHT:** Keine Änderung an `copy_week` (deprecated). Keine generische
Settings-/Toggle-Verwaltung (nur der eine Schalter). Kein Feature-Flag-für-Vermarktung-Mechanismus
(bewusst getrennt gehalten).

</domain>

<decisions>
## Implementation Decisions

### Globaler Toggle: Speicherung & Konfiguration (D-24-01)
- **D-24-01a (Speicher = ToggleService, reuse):** Der globale Modus wird über den
  **bestehenden `ToggleService`** gespeichert (generischer admin-editierbarer Boolean-Speicher,
  gated via `toggle_admin`). Bewusst **NICHT** `feature_flag` — der User reserviert
  `feature_flag` für eine spätere Vermarktung/SaaS-Gating und will, dass der Instanz-Admin es
  *nicht* editiert. Der ToggleService ist faktisch der vom User gewünschte „SettingsService".
  Boolean-Semantik: `enabled = true` → **hart**, `false` → **weich**. **Default = weich**
  (keine Regression). *(Begründung im Code: feature_flag-Migration deklariert feature_flag
  explizit als „Architektur-/Migrations-Schalter, KEINE User-Toggles"; toggle ist der
  User-Toggle-Mechanismus.)*
- **D-24-07 (Seeding & Naming):** Eine **neue Migration** seedet via `INSERT` einen Toggle
  mit Key **`paid_limit_hard_enforcement`** (`enabled = 0` = weich) inkl. Beschreibung, **keine**
  Toggle-Gruppe. Spiegelt das `feature_flag`-Seeding-Muster (`20260501000000_add-feature-flag-table.sql`).
  Nötig, weil `ToggleService` UPDATE-/Read-orientiert ist (`is_enabled`/`enable_toggle`/`disable_toggle`).
- **D-24-06 (Konfig-UI = neue Settings-Seite, nur dieser Schalter):** Eine **neue `/settings/`-Route**
  + admin-gated Nav-Eintrag (`toggle_admin`, Muster `is_admin_target`/`UserManagementPage`), die
  **nur den einen** Paid-Limit-Schalter (hart/weich) per Klick umlegt. `Key::Settings` existiert
  bereits in i18n. Bewusst **keine** generische Toggle-Liste (Scope-Eingrenzung; später erweiterbar).

### Harte Durchsetzung im Buchungspfad (D-24-02)
- **D-24-02 (Block außer Shiftplanner):** Im harten Modus blockt `book_slot_with_conflict_check`
  das Buchen über das Limit — **außer** der agierende Nutzer hat `SHIFTPLANNER_PRIVILEGE`. Im
  weichen Modus bleibt es bei der nicht-blockierenden `Warning::PaidEmployeeLimitExceeded`.
- **D-24-Grenzregel (strikt-größer, deckungsgleich mit Warnung):** Geblockt wird genau dann,
  wenn die neue Buchung den bezahlten Count **über** das Limit brächte — also bei genau `max`
  bezahlten darf der nächste **bezahlte** nicht mehr (`current > max`, strikt-größer wie heute).
  **Nur bezahlte** Personen zählen (`is_paid`, via `count_paid_bookings_in_slot_week`), unbezahlte
  werden nie geblockt. **Bestehende Buchungen werden NIE rückwirkend angefasst/entfernt** —
  Moduswechsel weich→hart auf einem schon überzogenen Slot blockt nur die *nächste* überziehende
  Buchung. (Konsistent mit v1.1 D-07 „kein Rollback".)
- **D-24-08 (Verdrahtung = Pre-Persist-Check):** `ShiftplanEditService` bekommt eine
  **`ToggleService`-Dependency** (ToggleService ist Basic-Tier — nur DAO+Permission+Transaction →
  in `main.rs` vor dem Business-Tier konstruierbar; ShiftplanEditService ist Business-Logic und
  darf konsumieren). In `book_slot_with_conflict_check` wird der Toggle **pro Buchung frisch**
  gelesen (`is_enabled` ist auth-only); wenn `hart` UND strikt-überzogen UND Akteur ohne
  Shiftplanner → **neuer ServiceError VOR dem Persistieren** (kein Booking angelegt). Sonst wie
  heute: persistieren + ggf. Soft-Warning. **Wichtig:** Der heutige Pfad persistiert ERST und
  zählt DANN (`shiftplan_edit.rs:529-548`) — für den harten Block muss die Zählung/Prüfung **vor**
  `booking_service.create(...)` gezogen werden.
- **copy_week:** `copy_week_with_conflict_check` ist **deprecated** → wird vom harten Enforcement
  **nicht** angefasst (bleibt soft/Warnungen, kein harter Block, keine Transaktions-Rollback-Gefahr).

### Permission-Bugfix (D-24-04, in Phase 24 gefoldet)
- **D-24-04 (Buchungs-Gate korrigieren):** Das Gate von `book_slot_with_conflict_check` wird von
  **`HR ∨ self`** (`shiftplan_edit.rs:407-417`) auf **`Shiftplanner ∨ self`** korrigiert (HR raus).
  Grund: Es ist eine echte Inkonsistenz — `get_bookable_sales_persons` ist bereits
  `shiftplanner`-gated (`sales_person_shiftplan.rs:84-88`), aber das tatsächliche Buchen verlangt
  HR. Ein Schichtplaner (ohne HR) sieht die Personen, kann sie aber nicht buchen. Korrektes Modell:
  Schichtplaner weist andere zu, ein Mitarbeiter trägt sich selbst ein. (Admin behält Zugriff via
  `admin-auto-grant`-Trigger, der alle Privilegien grantet.) Dieser Fix ist **Prerequisite** für ein
  kohärentes D-24-02. **Blast-Radius:** `service_impl/src/test/shiftplan_edit.rs` (HR-basierte
  Booking-Tests) muss angepasst werden.

### Fehler-UX beim harten Block (D-24-05)
- **D-24-05 (dedizierte Inline-Meldung am Slot):** Der Block liefert einen **neuen,
  unterscheidbaren ServiceError** (z.B. `PaidLimitExceeded { current, max }`) mit eigenem
  HTTP-Status — **NICHT** `ServiceError::Forbidden` (→ 403, das das Frontend heute
  **still ignoriert**, `shiftplan.rs:431-436`, D-13). Das Frontend erkennt den Status und zeigt
  eine **spezifische, lokalisierte Inline-Meldung** an der Buchungsstelle (sinngemäß „Bezahlt-Limit
  erreicht — nur die Schichtplanung kann darüber buchen"). Konsistent mit der User-Präferenz
  „Inline-Meldungen statt Dialoge". i18n En/De/Cs.

### Deutlichere Overage-Anzeige (D-24-03)
- **D-24-03 (persistente Warn-Sektion über dem Plan, alle Rollen):** Über die rote Zelle aus
  Phase 23 hinaus (die **bleibt**) wird eine **persistente Warn-Sektion über dem Schichtplan**
  gerendert — analog zur bestehenden `WarningList`-/„Konflikt-Buchungen"-Sektion —, die alle Slots
  der geladenen Woche mit `current_paid_count > max_paid_employees` auflistet. **Für alle Rollen**
  sichtbar (konsistent mit Phase-23 D-23-05; „bezahlte Mitarbeiter sollen den Effekt sofort
  sehen"). **Rein clientseitig** (beide Felder liegen pro Slot schon im Week-View-State) → **kein**
  Backend-/DTO-Change. **Nicht** an den Modus gekoppelt (zeigt Overage immer, wenn er existiert).
  i18n En/De/Cs. *(Unterschied zum bestehenden Banner: der heutige `booking_warnings`-Banner ist
  transient — nur direkt nach einer Buchung; die neue Sektion ist persistent pro geladener Woche.)*

### Claude's Discretion
- Exakter Name & HTTP-Status-Code des neuen Block-ServiceError (z.B. 409 vs 422; an bestehender
  Mapping-Tabelle `rest/src/lib.rs:147-199` ausrichten).
- Genaue Formulierung aller i18n-Labels/Meldungen (Settings-Schalter, Block-Inline-Meldung,
  Overage-Sektion) in De/En/Cs.
- Konkretes Layout/Styling der Settings-Seite und der Overage-Warn-Sektion (am bestehenden
  WarningList-/Token-Set ausrichten).
- Ob die Frontend-Overage-Sektion eine bestehende Komponente (`WarningList`) wiederverwendet oder
  eine eigene kleine Komponente bekommt.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Buchungspfad & Enforcement (Backend)
- `service_impl/src/shiftplan_edit.rs:399-555` — `book_slot_with_conflict_check`; Gate `HR ∨ self`
  (`:407-417`, → D-24-04), Slot-Lookup (`:419-423`), Persist (`:471-474`), Paid-Limit-Soft-Warning
  (`:529-548`, → Pre-Persist-Umbau für D-24-08), `count_paid_bookings_in_slot_week`-Helper (`:624+`).
- `service_impl/src/shiftplan_edit.rs:557-616` — `copy_week_with_conflict_check` (deprecated, NICHT anfassen).
- `service/src/shiftplan_edit.rs` — Trait `ShiftplanEditService` + `BookingCreateResult`.
- `service/src/warning.rs:55-72` — `Warning::PaidEmployeeLimitExceeded` (Soft-Pfad bleibt für weichen Modus).
- `rest/src/shiftplan_edit.rs:120-160` — REST-Handler `book_slot_with_conflict_check`.

### Toggle / Settings (Speicher + Konfig)
- `service/src/toggle.rs` — `ToggleService`-Trait (`is_enabled`/`enable_toggle`/`disable_toggle`/
  `get_all_toggles`), `TOGGLE_ADMIN_PRIVILEGE = "toggle_admin"`.
- `service_impl/src/toggle.rs` — Impl (Read-Pattern für `is_enabled`).
- `rest/src/toggle.rs` — bestehende Toggle-REST-Endpoints (Frontend-Anbindung).
- `migrations/sqlite/20260105000000_app-toggles.sql` — `toggle`/`toggle_group`-Schema + `toggle_admin`-Privileg.
- `migrations/sqlite/20260501000000_add-feature-flag-table.sql` — **Seeding-Muster** (INSERT eines
  Keys mit Default-Wert) als Vorlage für D-24-07.

### Permission-Modell (D-24-04)
- `service/src/permission.rs:9-11` — `HR_PRIVILEGE`, `SHIFTPLANNER_PRIVILEGE`, `SALES_PRIVILEGE`.
- `service_impl/src/sales_person_shiftplan.rs:77-102` — `get_bookable_sales_persons` (bereits
  `shiftplanner`-gated → Referenz für korrektes Gate).
- `migrations/sqlite/20260508120000_admin-auto-grant-privilege.sql` — Admin bekommt alle Privilegien
  automatisch (warum Admin nach D-24-04 funktionsfähig bleibt).
- `service_impl/src/test/shiftplan_edit.rs` — bestehende Booking-Tests (HR-basiert) → anzupassen.

### ServiceError / HTTP-Mapping (D-24-05)
- `service/src/lib.rs:65-127` — `ServiceError`-Enum (neuer Variant hier ergänzen).
- `rest/src/lib.rs:147-199` — ServiceError→HTTP-Mapping (`Forbidden`→403 still ignoriert,
  `ValidationError`→422, `EntityConflicts`→409); neuer Variant hier mappen.

### Frontend
- `shifty-dioxus/src/page/shiftplan.rs:403-441` — `AddUserToSlot`-Handling: Ok/403-silent/Err-Pfade
  (→ D-24-05 muss den Block-Status hier explizit als Inline-Meldung behandeln, nicht stilles 403).
- `shifty-dioxus/src/page/shiftplan.rs:99-107,815-871` — `is_shiftplanner`/`is_hr`-Ableitung,
  Konflikt-/Warn-Sektionen über dem Plan (Muster für D-24-03).
- `shifty-dioxus/src/component/warning_list.rs:90-168` — `WarningList`-Komponente inkl.
  `PaidEmployeeLimitExceeded`-Rendering (`:150-161`) — Vorlage/Reuse für die Overage-Sektion.
- `shifty-dioxus/src/state/shiftplan.rs:167-211` — Week-View-`Slot` mit `max_paid_employees`/
  `current_paid_count` (clientseitige Datenquelle für D-24-03).
- `shifty-dioxus/src/router.rs:27-58` — Route-Enum (neue `/settings/`-Route für D-24-06).
- `shifty-dioxus/src/component/top_bar.rs:68-169,424` — Nav-Einträge + `is_admin_target` (admin-gated
  Settings-Nav für D-24-06).
- `shifty-dioxus/src/service/feature_flag.rs` — **toter Shell** nach Phase-8.6-Cutover
  (`#![allow(dead_code)]`); NICHT als Vorlage missverstehen — wir nutzen den ToggleService.

### Regeln / Konventionen
- `shifty-backend/CLAUDE.md` — Service-Tier-Konventionen (Basic vs Business-Logic), `Option<Transaction>`,
  `#[utoipa::path]`, **Clippy ist hartes Gate** (`cargo clippy --workspace -- -D warnings`).
- `shifty-dioxus/CLAUDE.md` — i18n alle 3 Locales, statische Tailwind-Klassen, WASM-Build-Gate
  (`cargo build --target wasm32-unknown-unknown`).

### Roadmap / Vorgänger-Kontext
- `.planning/ROADMAP.md` § Phase 24 (D-24-01..03, offene Punkte).
- `.planning/phases/23-frontend-slot-paid-capacity-ui/23-CONTEXT.md` — Phase-23-Entscheidungen
  (rote `bad`-Zelle D-23-03/04, „kein Zahlen-Badge"-Verzicht, D-23-05 alle-Rollen-Sichtbarkeit).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`ToggleService`** (Service+DAO+REST) ist komplett vorhanden und ohne Business-Konsumenten —
  ideale, einsatzbereite Infrastruktur für den globalen Schalter (D-24-01a). Nur Toggle seeden
  (D-24-07) + Frontend-Anbindung (D-24-06).
- **`count_paid_bookings_in_slot_week`** (`shiftplan_edit.rs`) liefert exakt die Zählung, die der
  harte Block braucht — gleiche Quelle wie die bestehende Warnung (Grenzregel deckungsgleich).
- **`WarningList`** (`warning_list.rs`) rendert `PaidEmployeeLimitExceeded` bereits hübsch mit
  „X/Y" — wiederverwendbar für die persistente Overage-Sektion (D-24-03).
- **`feature_flag`-Migration** ist das Seeding-Vorbild für den neuen Toggle.
- **`UserManagementPage` + `is_admin_target`** sind das Vorbild für eine neue admin-gated
  Settings-Route (D-24-06).

### Established Patterns
- Service-Tier: ShiftplanEditService = Business-Logic (darf ToggleService=Basic konsumieren);
  DI-Reihenfolge in `main.rs` = erst Basic, dann Business.
- ServiceError→HTTP-Mapping zentral in `rest/src/lib.rs`; Frontend unterscheidet heute
  403 (silent) vs 422 (surfaced) — D-24-05 braucht einen *nicht*-stillen, unterscheidbaren Status.
- Permission via `permission_service.check_permission(PRIVILEGE, ctx)`; `.is_ok()` für
  „bypass wenn vorhanden"-Checks (Muster aus `get_bookable_sales_persons`).

### Integration Points
- Backend: ShiftplanEditService ↔ ToggleService (neue Dep) + neuer ServiceError + Gate-Fix.
- Frontend: neue Settings-Route ↔ Toggle-REST (set/get) + Inline-Block-Meldung im
  `AddUserToSlot`-Handler + neue Overage-Sektion im Shiftplan-Header (clientseitig aus State).
- Keine DTO-Änderung für D-24-03 (Week-View-State trägt beide Felder schon).

</code_context>

<specifics>
## Specific Ideas

- „feature_flag NICHT für diesen Schalter" — bewusst reserviert für spätere Vermarktung; der
  Instanz-Admin soll feature_flags nicht editieren, diesen Betriebs-Schalter aber schon.
- Block-UX wie eine klare Inline-Meldung am Slot, nicht als stilles 403 oder Dialog.
- Overage-Anzeige „über dem Schichtplan wie bei den anderen Warnungen" — persistent, nicht nur
  transient nach einer Buchung.
- Booking-Permission-Bug („Schichtplaner muss zuweisen können, HR nicht") als Teil dieser Phase
  mitgefixt.

</specifics>

<deferred>
## Deferred Ideas

- **feature_flag-basiertes Vermarktungs-/SaaS-Gating** — eigener späterer Mechanismus, bewusst von
  diesem Betriebs-Toggle getrennt gehalten. (Nicht in Phase 24.)
- **Generische Settings-/Toggle-Verwaltungs-UI** (alle Toggles via `get_all_toggles`) — bewusst
  zugunsten einer minimalen Ein-Schalter-Settings-Seite zurückgestellt; später erweiterbar.
- **`copy_week` an Enforcement anpassen** — deprecated, daher nicht investiert.

</deferred>

---

*Phase: 24-paid-capacity-enforcement-config*
*Context gathered: 2026-06-27*
