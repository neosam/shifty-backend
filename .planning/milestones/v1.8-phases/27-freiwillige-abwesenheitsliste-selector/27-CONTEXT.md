# Phase 27: Freiwillige in Abwesenheitsliste auswählbar (FE) - Context

**Gathered:** 2026-06-29
**Status:** Ready for planning
**Mode:** Smart-Discuss (autonomous) — Seeds + ROADMAP-Entscheidungen + 1 Grey-Area geklärt

<domain>
## Phase Boundary

Reines **Frontend** (`shifty-dioxus`). Auf der Abwesenheitsseite werden **Freiwillige**
(`sales_person.is_paid == false`, aktiv) in den beiden Personen-Selektoren auswählbar —
sichtbar getrennt von Angestellten über native `optgroup`-Gruppierung:

- **AbsenceModal** Personen-Dropdown (`absences.rs:1217`) — beim Anlegen/Bearbeiten einer Abwesenheit.
- **AbsenceFilterBar** Personenfilter (HR-only, `absences.rs:1428`) — Filter der Abwesenheitsliste.

**Requirements:** VOL-SEL-01.

**Liefert NICHT:**
- **Keine** Backend-Änderung — VFA (Phase 26) + EmployeeWorkDetails (Phase 17) decken
  Freiwilligen-Abwesenheiten bereits ab; Personenliste wird bereits vollständig geladen.
- **Keine** Änderung an der HR-Urlaubsübersicht (`VacationPerPersonList` / `selectable_balances`)
  — bleibt paid-only (Grey-Area-Entscheidung D-27-02). Freiwillige in der Urlaubsübersicht
  gehören thematisch zu Phase 28.
- **Keine** neue Abwesenheitskategorie-Logik — Freiwillige bekommen dieselben Kategorien wie
  Angestellte (D-27-05).
- **Keine** Änderung an `SelectInput` (rendert `children` → optgroup als Children möglich).

</domain>

<decisions>
## Implementation Decisions

### VOL-SEL-01: Selector-UX (gruppierter Dropdown)
- **D-27-01 (Native optgroup, gemeinsamer Helfer):** Beide Selektoren rendern zwei native
  `optgroup`s — **„Angestellte" zuerst, dann „Freiwillige"**. Ein **gemeinsamer Helfer**
  (z.B. `grouped_employee_options(...)` bzw. eine pure Partition-Funktion + RSX-Wrapper)
  bedient beide Call-Sites (AbsenceModal `:1217`, AbsenceFilterBar `:1428`) — kein Copy-Paste.
  `SelectInput` (`component/form/inputs.rs:83`, rendert `select { children }`) bleibt unverändert.

### VOL-SEL-01: Predicate / Geltungsbereich (Grey-Area, entschieden 2026-06-29)
- **D-27-02 (Eigenes Dropdown-Predicate, `is_selectable_employee` NICHT global lockern):**
  Der neue Gruppierungs-Helfer nutzt ein **eigenes** Predicate `!inactive` und gruppiert nach
  `is_paid`. **`is_selectable_employee` bleibt `is_paid && !inactive`** und damit auch
  `selectable_balances` (`:129`) unverändert → die **HR-Urlaubsübersicht** (`VacationPerPersonList`,
  `:608`) bleibt **paid-only**. Begründung: Phase-27-Ziel ist strikt die beiden Abwesenheits-
  Selektoren; Freiwillige in der Urlaubsübersicht wäre eine zweite Ansichtsänderung (Phase-28-Thema)
  und eine Überraschung. Bestehende `selectable_balances`-Tests (`:3557+`) bleiben unverändert grün.

### VOL-SEL-01: Leere Gruppen
- **D-27-03 (Kein leeres optgroup):** Eine Gruppe wird **nur gerendert, wenn sie ≥1 aktive Person**
  enthält. Keine Freiwilligen → kein „Freiwillige"-optgroup; keine Angestellten → kein
  „Angestellte"-optgroup.

### VOL-SEL-01: i18n
- **D-27-04 (2 neue Keys, de/en/cs):** Neue i18n-Keys `AbsenceGroupEmployees` („Angestellte" /
  „Employees" / „Zaměstnanci") und `AbsenceGroupVolunteers` („Freiwillige" / „Volunteers" /
  „Dobrovolníci") — Key-Enum (`i18n/mod.rs`) + `en.rs`/`de.rs`/`cs.rs`. (Cs-Wortwahl = Claude's
  Discretion, fachlich gängige Übersetzung.)

### VOL-SEL-01: Inaktive
- **D-27-06 (Inaktive in beiden Gruppen ausgeblendet):** `inactive`-Personen erscheinen in **keiner**
  Gruppe — egal ob Angestellte oder Freiwillige. Die bestehende „Alle"-Option der AbsenceFilterBar
  (`:1425`) bleibt erhalten und unverändert.

### Kategorien für Freiwillige
- **D-27-05 (Identisch zu Angestellten):** Das Kategorie-Dropdown im Modal (`:1229`) bleibt
  unverändert — Freiwillige bekommen dieselben Kategorien (aktuell **Vacation + UnpaidLeave**;
  `SickLeave` ist über `SICK_LEAVE_ENABLED = false` global ausgeblendet, `:147`). Keine fachliche
  Einschränkung genannt → kein Special-Casing. Betrifft ohnehin nur den Kategorie-, nicht den
  Personen-Selektor.

### Claude's Discretion
- Genaue Signatur des Helfers (pure Partition-Funktion `partition_selectable(&[SalesPerson]) ->
  (employees, volunteers)` für Unit-Tests + RSX-Wrapper, **oder** ein RSX-Element-Helfer mit
  ausgelagerter Partition). Empfehlung: pure Partition-Funktion (testbar, im Stil der bestehenden
  `is_selectable_employee`/`selectable_balances`-Tests), darüber ein dünner RSX-Wrapper.
- Sortierung innerhalb einer Gruppe (Default: Eingangsreihenfolge der geladenen Liste beibehalten,
  konsistent mit dem heutigen flachen Loop).
- Exakte cs-Übersetzung der Gruppenlabels.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirement & Seed
- `.planning/ROADMAP.md` § "Phase 27" — Goal + 5 Success Criteria + Konzept-Eckpunkte.
- `.planning/phases/27-freiwillige-abwesenheitsliste-selector/SEED.md` — Status quo + Entscheidungen.

### Frontend (alles in `shifty-dioxus/src/page/absences.rs`)
- `:110-117` — `is_selectable_employee` (bleibt `is_paid && !inactive`; **NICHT** ändern, D-27-02).
- `:129-141` — `selectable_balances` (nutzt `is_selectable_employee`; bleibt paid-only).
- `:147-153` — `SICK_LEAVE_ENABLED` (=false) + `category_visible_with` (Kategorie-Sichtbarkeit).
- `:1207-1258` — **AbsenceModal**: Personen-Dropdown (`:1217`, flacher Filter-Loop → Helfer) +
  Kategorie-Dropdown (`:1229`, unverändert, D-27-05). `is_hr`-Gate auf dem Personenfeld (`:1211`).
- `:1411-1436` — **AbsenceFilterBar** (HR-only): „Alle"-Option (`:1425`) + Personen-Loop (`:1428`).
- `:2476-2504` — bestehende `is_selectable_employee`-Tests (Stil-Vorlage; **diese 4 bleiben gleich**,
  da Predicate unverändert).
- `:3557-3610` — bestehende `selectable_balances`-Tests (bleiben grün, D-27-02).
- `shifty-dioxus/src/component/form/inputs.rs:83` — `SelectInput` (rendert `select { children }`,
  unverändert; optgroup als Children).
- `shifty-dioxus/src/i18n/mod.rs` (Key-Enum) + `en.rs`/`de.rs`/`cs.rs` — neue Gruppenlabels.
- `shifty-dioxus/src/state/...` `SalesPerson` — Felder `is_paid: bool`, `inactive: bool`, `id`, `name`.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Beide Call-Sites haben dasselbe Muster** (`for sp in sales_persons.iter().filter(is_selectable_employee)
  { option {...} }`) → ein gemeinsamer Helfer ersetzt beide Loops 1:1 (D-27-01).
- **`SelectInput` rendert `children`** → native `optgroup { label, option {...} }` als Children
  funktioniert ohne Komponenten-Änderung.
- **Pure-Function-Test-Muster** etabliert (`is_selectable_employee`-Tests `:2486+`,
  `selectable_balances`-Tests `:3560+`) → neue Partition-/Gruppierungs-Logik genauso testbar.

### Established Patterns
- **i18n de/en/cs**: Key-Enum + 3 Locale-Maps; alle benutzersichtbaren Texte übersetzt.
- **Aktiv-Filter** (`!inactive`) für Selektoren; vollständige `sales_persons`-Liste bleibt geladen
  (Namensauflösung bestehender Abwesenheiten inaktiver/unbezahlter Personen).
- **SSR/Pure-Helper-Tests** statt Browser-Asserts für Logik (vgl. Memory: Datepicker/Signale).

### Integration Points
- AbsenceModal-Personen-Dropdown (`:1217`) und AbsenceFilterBar-Personen-Loop (`:1428`) →
  beide auf den gemeinsamen Helfer umstellen; in der FilterBar **nach** der „Alle"-Option.

</code_context>

<specifics>
## Specific Ideas

- **Partition-Test:** Liste mit {aktiv-paid, aktiv-unpaid, inaktiv-paid, inaktiv-unpaid} →
  employees = [aktiv-paid], volunteers = [aktiv-unpaid]; inaktive in keiner Gruppe.
- **Leere-Gruppe-Test:** nur Angestellte → volunteers leer (kein optgroup); nur Freiwillige →
  employees leer (kein optgroup).
- **Reihenfolge-Test:** Angestellten-Gruppe wird vor Freiwilligen-Gruppe ausgegeben.
- **Browser-Smoke (Backend-Roundtrip):** Ein aktiver Freiwilliger erscheint im Modal- UND
  FilterBar-Dropdown unter „Freiwillige"; Abwesenheit für ihn anlegbar (create-Pfad).

</specifics>

<deferred>
## Deferred Ideas

- **Freiwillige in der HR-Urlaubsübersicht** (`selectable_balances` lockern) — bewusst draußen
  (D-27-02); thematisch Phase 28 (Urlaubsanspruch).
- **Typ-Filter/Toggle** (nur Angestellte / nur Freiwillige) — nicht gewünscht; Gruppierung reicht.
- **SickLeave-Reaktivierung** für Freiwillige — unabhängig vom global gesetzten
  `SICK_LEAVE_ENABLED=false`; außer Scope.

### Reviewed Todos (not folded)
None — Diskussion blieb im Phasen-Scope.

</deferred>

---

*Phase: 27-Freiwillige in Abwesenheitsliste auswählbar (FE)*
*Context gathered: 2026-06-29*
