# Phase 33: Special-Days-UI in den Einstellungen - Context

**Gathered:** 2026-06-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Ein **Shiftplanner** kann Special Days (Typ `Holiday` / `ShortDay`) über **zwei
Oberflächen** pflegen — beide mit vollem CRUD — verdrahtet gegen die bereits
existierende REST-CRUD (`POST /special-days/`, `DELETE /special-days/{id}`,
`GET /special-days/for-week/{year}/{week}`) plus einen **neuen Range/Jahr-Read-Endpoint**:

1. **Schichtplan-Seite** (`shifty-dioxus/src/page/shiftplan.rs`): interaktiv im
   Wochen-Kontext — ein **Dropdown pro Wochentag** im Wochenraster mit den Optionen
   **Feiertag / Kurzer Tag / Nichts** (Nichts = entfernen). Anlegen/Löschen direkt für
   die gezeigte KW; nutzt `for-week` nativ. Kein Datepicker-Caveat (klick-/select-basiert).
2. **Settings-Seite** (`shifty-dioxus/src/page/settings.rs`): shiftplanner-gegatete
   Sektion mit **Kalenderdatum-Picker** zum Anlegen (SPD-01), **Jahres-Liste** mit
   abgeleitetem Kontext (`15.08.2026 (Samstag, KW 33, 2026)`, SPD-02), Löschen (SPD-03).

**Achse:** Frontend-zentriert (FE Create/Delete erstmals verdrahtet) + **kleiner
Backend-Anteil** (neuer Range/Jahr-Read-Endpoint). i18n de/en/cs (SPD-04).

**Requirements:** SPD-01, SPD-02, SPD-03, SPD-04.

**Liefert NICHT:**
- Keine Soll-Wirkung im Schichtplan (das ist Phase 34 / HSP).
- Keine ShortDay-Soll-Automatik im Report (Future-Story, schon Phase 25 außer Scope).
- Kein Hover-Tooltip auf Feiertags-Zellen (deferred, Phase 34-Differentiator).

</domain>

<decisions>
## Implementation Decisions

### Berechtigung / Gate-Abgleich (SPD-04)
- **D-33-01 (shiftplanner, NICHT admin):** Die Special-Days-Pflege wird durchgängig auf
  **`shiftplanner`** gegated — auf **beiden** Flächen. Begründung (code-verifiziert):
  Special Days sind **Schichtplan-Struktur-Daten**; die Slot-Struktur-CRUD
  (`create_slot`/`update_slot`/`delete_slot`, `service_impl/src/slot.rs:211/293/269`) und
  die bestehende Special-Day-CRUD (`service_impl/src/special_days.rs:78/106`) gaten **bereits
  alle** auf `SHIFTPLANNER_PRIVILEGE` (`service/src/permission.rs:11 = "shiftplanner"`).
  **Korrektur zu SPD-04:** Das Requirement formulierte „admin-gated (Muster Phase 24/25)" —
  das war eine ungenaue Annahme. Phase 24/25 ist zufällig admin (echte App-Toggles wie
  paid-limit/Feiertags-Stichtag), Special Days gehören aber fachlich zu `shiftplanner`.
  **Kein Backend-Permission-Change** (Backend ist schon korrekt). FE-Gate = `AUTH … has_privilege("shiftplanner")`.
- **D-33-02 (Mismatch vermieden):** Die Card darf NICHT hinter das pauschale `admin`-Gate
  der heutigen `SettingsPage` (`settings.rs:32`). Sonst sähe ein Admin-ohne-shiftplanner die
  UI und kassierte beim Write ein 403 (vgl. offener Todo `admin-rolle-bekommt-alle-privilegien`).
  Die Settings-Sektion wird **pro Card** shiftplanner-gegated (Seiten-Gate entsprechend lockern
  oder Card mit eigenem Guard) — exakte Verdrahtung = Claude's Discretion, aber FE-Gate MUSS
  `shiftplanner` sein, deckungsgleich zum Backend.

### Zwei Oberflächen, beide voll-CRUD (SPD-01/02/03)
- **D-33-03 (Schichtplan-Seite = interaktiv):** Per-Tag-**Dropdown** im Wochenraster (Mo–So),
  Optionen **Feiertag / Kurzer Tag / Nichts**. Auswahl spiegelt/setzt den aktuellen Special-Day-
  Zustand des Tages: Feiertag/Kurzer Tag → `create`, Nichts → `delete`. Wochen-Kontext liefert
  `(year, calendar_week, day_of_week)` direkt aus dem Raster → **kein** Kalenderdatum-Mapping,
  **kein** Datepicker-Caveat. Konzipiert als generisches „Einstellungen für diesen Tag"-Dropdown,
  aktuell nur Special-Day-Typ (erweiterbar).
- **D-33-04 (Settings-Seite = Datepicker-Flow):** Kalenderdatum-Picker → Mapping Datum →
  `(year, iso_week, weekday)` (`time::Date::from_iso_week_date` / `as_shifty_week`), Typ-Auswahl
  Holiday/ShortDay, Anlegen + Jahres-Liste + Löschen. **WASM-Datepicker-Caveat (D-25-06)** gilt
  hier → Persistenz-/Anzeige-Loop im echten Browser verifizieren (vgl. Card 2 Phase 25 als Muster).

### Listen-Scope / Multi-Woche (SPD-02)
- **D-33-05 (Neuer Range/Jahr-Read-Endpoint, BE):** Für die Settings-Übersicht wird ein
  **neuer schlanker Backend-Read-Endpoint** ergänzt (z. B. `GET /special-days/for-year/{year}`
  oder Range), damit **ein** Request die Jahres-Liste füllt — statt ~53 wochenweiser `for-week`-
  GETs. **Hebt das in REQUIREMENTS deferierte Item „Multi-Wochen-Read-Endpoint" bewusst in diese
  Phase.** Umfasst DAO-Query (Range über `year`), Service-Methode, Route + `#[utoipa::path]` +
  ApiDoc, FE-`api.rs`/`loader.rs`. Read-Permission konsistent zum bestehenden `for-week` halten
  (heute ungegated) — exakte Read-Gate-Wahl = Claude's Discretion (UI ist ohnehin shiftplanner-gated).

### Eingabe-/Form-Regeln (SPD-01)
- **D-33-06 (ShortDay-Uhrzeit Pflicht):** Bei Typ `ShortDay` ist `time_of_day` Pflichtfeld
  (Submit erst aktiv mit gültiger Uhrzeit). Bei `Holiday` kein Uhrzeitfeld.
- **D-33-07 (Duplikat blocken):** Existiert am selben `(year, calendar_week, day_of_week)`
  bereits ein Special Day, wird ein zweites Anlegen mit **Inline-Hinweis** verhindert (kein
  zweiter Eintrag). Gilt auf beiden Flächen. (Kein DB-Unique-Constraint sichtbar → FE-/Service-
  seitige Prüfung; ob zusätzlich im Service abgesichert = Claude's Discretion.)

### Settings-Listen-Darstellung (SPD-02)
- **D-33-08 (Chronologisch, nach Jahr gruppiert):** Jahr-Picker wählt das geladene Jahr;
  Einträge chronologisch aufsteigend, optisch **nach Jahr gruppiert**, innerhalb nach Datum.
  Typ als **Badge** (Holiday/ShortDay), ShortDay zeigt die Uhrzeit. **Empty-State** mit
  Hinweistext. Datumsformat pro Locale + abgeleiteter Kontext `(Wochentag, KW, Jahr)` (SPD-02),
  mitübersetzt.

### Claude's Discretion
- Exakte Seiten-Gate-Verdrahtung der Settings-Sektion (Page-Gate lockern vs. Card-eigener Guard) —
  solange FE-Gate `shiftplanner` ist (D-33-02).
- Konkreter Name/Pfad des neuen Read-Endpoints (`for-year/{year}` vs. Range-Query) und dessen
  Read-Permission (ungegated wie `for-week` vs. shiftplanner).
- Ob die Duplikat-Prüfung zusätzlich serverseitig erzwungen wird (D-33-07).
- Konkretes UI-Layout (Dropdown-Komponente im Raster, Form-Layout inline vs. Card,
  Badge-/Empty-State-Styling) — ausgerichtet am bestehenden Settings-/Wochenraster-Set;
  ggf. in einer UI-Phase verfeinern.
- Alle i18n-Labels/Texte (de/en/cs): Typen, Optionen (Feiertag/Kurzer Tag/Nichts), Form-Labels,
  Listen-Kontext, Empty-State, Inline-Hinweise.

### Folded Todos
- **`2026-06-30-special-days-ui-bearbeiten-einstellungen.md`** (`resolves_phase: 33`) — kanonische
  Quelle dieser Phase. Problem: Special Days lassen sich nur über die DB pflegen, FE liest nur
  (`get_special_days_for_week`). Gefoldet als gesamter Phasen-Scope; die Diskussion erweitert die
  todo-Richtung um die zweite (Schichtplan-)Fläche und den Range/Jahr-Read-Endpoint.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Projektregeln
- `.planning/REQUIREMENTS.md` — v1.10 SPD-01..04 (inkl. Future/Out-of-Scope: ShortDay-Soll-
  Automatik, Hover-Tooltip, Multi-Wochen-Read-Endpoint → letzterer wird per D-33-05 in diese
  Phase gehoben).
- `.planning/ROADMAP.md` § "Phase 33" — Goal + 5 Success Criteria.
- `.planning/phases/25-feiertags-auto-anrechnung-stichtag-konfiguration/25-CONTEXT.md` —
  D-25-06 (admin-gated Settings-Card-Muster + **WASM-Datepicker-Caveat**), Special-Day-Quellen.
- `.planning/phases/24-paid-capacity-enforcement-config/24-CONTEXT.md` — Settings-Seiten-/
  Card-Muster (Route, admin-Gate, Card-Layout).
- `.planning/todos/pending/2026-06-30-special-days-ui-bearbeiten-einstellungen.md` — gefoldete
  kanonische Quelle (Ist-Stand, Felder, Datum→KW-Mapping).

### Backend CRUD (existiert) + neuer Read-Endpoint (D-33-05)
- `rest/src/special_day.rs` — `get_special_days_for_week`, `create_special_days` (POST `/`),
  `delete_special_day` (DELETE `/{id}`), `SpecialDayApiDoc`. **Hier** den neuen Range/Jahr-Read
  ergänzen (Route + `#[utoipa::path]` + ApiDoc).
- `service/src/special_days.rs:35-99` — `SpecialDay`, `SpecialDayType`, `SpecialDayService`-Trait
  (`get_by_week`/`create`/`delete`) — um Range/Jahr-Read erweitern.
- `service_impl/src/special_days.rs:58-120` — Impl; `create`/`delete` gaten auf
  `SHIFTPLANNER_PRIVILEGE`, `get_by_week` ungegated (`_context`).
- `dao/src/special_day.rs` + `dao_impl_sqlite/src/special_day.rs` — `find_by_week`/`find_by_id`/
  `create`/Soft-Delete; **neue** Range-Query (`find_by_year` o. ä.) hier ergänzen.
- `migrations/sqlite/20241020064536_add-special-day-table.sql` — Schema (`year`,
  `calendar_week`, `day_of_week`, `day_type`, `time_of_day`).

### Permission (D-33-01)
- `service/src/permission.rs:11` — `SHIFTPLANNER_PRIVILEGE = "shiftplanner"`.
- `service_impl/src/slot.rs:211/269/293` — Slot-Struktur-CRUD auf `shiftplanner` (Konsistenz-Anker).
- `shifty-dioxus/src/page/shiftplan.rs:104` — FE `has_privilege("shiftplanner")`-Muster.

### Frontend
- `shifty-dioxus/src/page/settings.rs` — admin-gated Settings-Seite, Card-1 (Toggle)/Card-2
  (Datums-Input) als Muster; hier die shiftplanner-Special-Days-Sektion (Card-3) ergänzen.
- `shifty-dioxus/src/page/shiftplan.rs` — Wochenraster; hier das Per-Tag-Dropdown ergänzen.
- `shifty-dioxus/src/api.rs:974-985` — `get_special_days_for_week` (Vorlage); **neu**:
  `create_special_day`, `delete_special_day`, `get_special_days_for_year`.
- `shifty-dioxus/src/loader.rs:115-131` — heutige Special-Day-Nutzung (Read).
- `rest-types/src/lib.rs:1119-1166` — `SpecialDayTO` (`id`, `year`, `calendar_week`,
  `day_of_week`, `day_type`, `time_of_day`, `$version`) + `SpecialDayTypeTO` (`Holiday`/`ShortDay`).

### i18n (SPD-04)
- `shifty-dioxus/src/i18n.rs` (`Key`-Enum, `Key::Settings` existiert) — neue Keys de/en/cs.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Backend Special-Day-CRUD komplett vorhanden** — nur `create`/`delete` FE-seitig noch nicht
  verdrahtet; Read-by-week existiert. Nur der Range/Jahr-Read fehlt (D-33-05).
- **`SpecialDayTO` / `SpecialDayTypeTO`** (rest-types) — Wire-Format steht; POST nimmt vollen
  `SpecialDayTO` (`id`/`$version` müssen nil sein, sonst `IdSetOnCreate`/`VersionSetOnCreate`).
- **Settings-Card-Muster (Phase 24/25)** auf `settings.rs` — Card-Stapel, `use_resource`-Load,
  `spawn`-Save, Datums-Input (Card 2) als direkte Vorlage inkl. Datepicker-Caveat-Handling.
- **`has_privilege("shiftplanner")`-Gate** (shiftplan.rs) — fertiges FE-Gate-Muster für D-33-01.
- **`time::Date::from_iso_week_date` / `as_shifty_week`** — Datum↔KW/Wochentag-Mapping (Settings-
  Flow); Wochenraster liefert `(year, KW, weekday)` direkt (Schichtplan-Flow, kein Mapping).

### Established Patterns
- **Service-Tier-Konvention:** `SpecialDayService` ist Basic-Tier (DAO+Permission+Transaction) —
  der neue Range-Read bleibt dort (keine Cross-Entity-Logik, keine Zyklen).
- **`WHERE deleted IS NULL`** Soft-Delete in allen DAOs (Range-Query mit einbeziehen).
- **`#[utoipa::path]` + ApiDoc** Pflicht für den neuen Endpoint.
- **i18n de/en/cs** für jeden neuen sichtbaren Text.
- **FE/BE-Gate-Konsistenz** (Memory „Backend-Roundtrip e2e"): create-Pfad ≠ Read-Pfad; FE-Gate
  MUSS `shiftplanner` sein, sonst 403 trotz sichtbarer UI.

### Integration Points
- **Schichtplan-Seite:** Wochenraster-Rendering bekommt pro Tag ein Dropdown; State spiegelt
  `for-week`-Special-Days; Auswahl → `create_special_day`/`delete_special_day` + Reload der Woche.
- **Settings-Seite:** neue shiftplanner-Card; `get_special_days_for_year` füllt die gruppierte
  Liste; Picker→Mapping→`create_special_day`; Delete-Button→`delete_special_day`.
- **rest-types:** evtl. kein neuer Typ nötig (Range-Read liefert `[SpecialDayTO]`).

</code_context>

<specifics>
## Specific Ideas

- **Anzeige-Format (SPD-02):** `15.08.2026 (Samstag, KW 33, 2026)` — Datum locale-üblich
  (de `TT.MM.JJJJ`), Klammer-Kontext (Wochentag, KW, Jahr) aus dem Datum berechnet + übersetzt.
- **Per-Tag-Dropdown (Schichtplan):** generisches „Einstellungen für diesen Tag" mit aktuell
  Feiertag / Kurzer Tag / Nichts — bewusst als erweiterbarer Einstiegspunkt formuliert.
- **Roundtrip-Test:** Frontend-API-Roundtrip create → for-year/for-week → delete; WASM-Build-Gate;
  Datepicker-/Persistenz-Loop der Settings-Fläche im echten Browser verifizieren.

</specifics>

<deferred>
## Deferred Ideas

- **ShortDay-Soll-Automatik im Report** (anteilig, `time_of_day`) — Future-Story, Phase-25-
  bestätigt außer Scope.
- **Hover-Tooltip auf Feiertags-Zelle** in der Schichtplan-Tabelle — deferred (Phase-34-
  Differentiator, rein additiv).
- **Weitere „Tag-Einstellungen" im Dropdown** über Special Days hinaus — das Dropdown ist
  erweiterbar gedacht, aber v1.10 liefert nur Feiertag/Kurzer Tag/Nichts.

### Reviewed Todos (not folded)
- Die übrigen `todo.match-phase`-Treffer (alle Score 0.6, reines Stopwort-Rauschen:
  booking-log-500, Vertragszeiten-Warnung, list_user_invitations, silentRenew, admin-rolle-
  privilegien, PDF-Export, Dependency-Update, slot-einzelne-KW, e2e-Browser-Tests u. a.) —
  inhaltlich unverwandt mit Special-Days-UI, nicht gefoldet. **Hinweis:**
  `2026-05-08-admin-rolle-bekommt-alle-privilegien.md` ist thematisch relevant (Hintergrund für
  D-33-02, warum admin ≠ shiftplanner), bleibt aber eigener Todo.

</deferred>

---

*Phase: 33-special-days-ui-einstellungen*
*Context gathered: 2026-06-30*
</content>
</invoke>
