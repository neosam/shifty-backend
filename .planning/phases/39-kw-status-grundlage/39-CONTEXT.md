# Phase 39: KW-Status Grundlage (BE+FE) - Context

**Gathered:** 2026-07-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Ein Schichtplaner kann jeder Kalenderwoche einen Status geben — **Kein (`Unset`) / In Planung / Geplant / Gesperrt** — persistiert pro **ISO-(Jahr, Woche)**. Der Status ist für alle Rollen als farbkodiertes Badge im Header oberhalb der Schichtplan-Wochenansicht sichtbar; nur Schichtplaner können ihn ändern.

**In Scope:** neue `week_status`-Tabelle + Migration; Basic-Tier `WeekStatusService`; Status-CRUD-REST; DI-Wiring in `main.rs`; Frontend Status-Badge (alle Rollen) + Status-Dropdown (nur Schichtplaner); i18n de/en/cs der 4 Labels; KW-53-/Jahresgrenzen-Unit-Tests.

**Out of Scope (Phase 40):** die tatsächliche Sperr-**Durchsetzung** von Schreibaktionen in einer Gesperrt-Woche (`assert_week_not_locked`, `delete_booking`-Re-Routing, HTTP-423). Phase 39 liefert nur das Datenmodell + die Anzeige/Bearbeitung des Status. Das `Gesperrt`-Badge existiert, sperrt aber in Phase 39 noch nichts.
</domain>

<decisions>
## Implementation Decisions

### Berechtigung & Status-Übergänge (WST-01)
- **D-39-01:** Nur **Schichtplaner** (`SHIFTPLANNER_PRIVILEGE`) darf den Status setzen/ändern — Muster analog `week_message`-Service (Basic-Tier). Alle anderen Rollen: reine Anzeige.
- **D-39-02:** **Alle Übergänge sind frei** — jeder Status → jeder Status, inkl. `Gesperrt` → zurück auf `In Planung`/`Unset` durch denselben Schichtplaner. **Kein** gesondertes Entsperr-Gate.

### Leer-Status: Enum-Name & Persistenz-Modell (WST-01)
- **D-39-03:** Die „Kein"-Variante heißt im Rust-Enum **`Unset`** (nicht `None` → Clippy/`Option`-Shadowing; nicht `Open` → Verwechslung mit „nicht gesperrt/offen bearbeitbar" vermeiden).
- **D-39-04:** **Persistenz-Modell = Zeilen-Abwesenheit.** Keine DB-Zeile für eine Woche ⇔ Status `Unset`. „Auf Kein zurücksetzen" = die Zeile **soft-deleten** (`deleted IS NULL`-Muster). Es wird also **kein** `Unset`-Diskriminant persistiert — nur `InPlanning`/`Planned`/`Locked` bekommen je eine Zeile. Analog `week_message` (Zeile existiert nur bei Inhalt).

### UI-Muster (WST-02) — Vorgabe des Users
- **D-39-05:** **Badge oberhalb der Wochenansicht.** Alle Rollen sehen das Badge **nur bei Status ≠ `Unset`**. Bei `Unset` wird für Nicht-Schichtplaner **gar nichts** angezeigt (kein leeres/graues Badge).
- **D-39-06:** Schichtplaner bekommen an derselben Stelle ein **Dropdown/Popover** mit den 4 Optionen zur Auswahl. **Kein controlled `<select>`** — bewusst, um die D-25-06-Desync-Klasse (SDF-Bug) zu vermeiden. Nach jeder Änderung Status frisch vom Server nachladen (kein optimistisches Signal, das driften kann).
- **D-39-07:** Für den Schichtplaner ist im Dropdown auch `Unset`/„Kein" als wählbarer Eintrag sichtbar (damit er zurücksetzen kann), obwohl das Badge selbst bei `Unset` verschwindet.

### Farben & Labels (WST-02, WST-05)
- **D-39-08:** Farb-Semantik: **Gesperrt = rot**, **Geplant = grün**, **In Planung = amber/gelb**, **`Unset` = neutral/grau** (nur im Dropdown des Schichtplaners sichtbar, nie als Badge).
- **D-39-09:** Labels de/en/cs (an Implementierung final justierbar, aber als Startpunkt gesetzt):

| Enum-Variante | de | en | cs |
|---|---|---|---|
| `Unset` | Kein | None | Žádný |
| `InPlanning` | In Planung | In planning | V plánování |
| `Planned` | Geplant | Planned | Naplánováno |
| `Locked` | Gesperrt | Locked | Uzamčeno |

### Datenmodell-Konventionen (aus ROADMAP/Research vorentschieden)
- **D-39-10:** Migration analog `week_message` / `vacation-entitlement-offset`: ISO-`(year, calendar_week)`-Composite-Key, TEXT-Enum-Diskriminant (manuelles `match` im `TryFrom`, Muster `special_day`/`extra_hours`), **partial UNIQUE `WHERE deleted IS NULL`**.
- **D-39-11:** ISO-Jahr **immer** aus `date.to_iso_week_date().0` ableiten — **nie** `date.year()`. KW-53-/Jahreswechsel-Tage müssen in der richtigen `(year, week)`-Zeile landen (Unit-Test-Pflicht, WST Success-Criterion 3).
- **D-39-12:** Tier-Einordnung: `WeekStatusService` = **Basic-Tier** (nur DAO + `PermissionService` + `TransactionDao`, keine Domain-Services). DI-Wiring in `main.rs` in der Basic-Service-Schicht (vor Business-Logic). Das spätere Sperr-Gate (Phase 40) lebt separat im Business-Logic-Tier.

### Claude's Discretion
- Exakter Tabellen-/Spaltenname, DTO-Feldnamen, REST-Pfad (`/week-status/...` in Anlehnung an `/week-message/...`), Dropdown-vs-Popover-Detail und genaue Header-Position im Frontend. Die de/en/cs-Labels dürfen bei der Umsetzung sprachlich fein justiert werden, solange die 4 Status semantisch erhalten bleiben.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Roadmap & Requirements
- `.planning/ROADMAP.md` §"Phase 39: KW-Status Grundlage" — Goal, Success Criteria, Scope (BE+FE), offene Entscheidungen.
- `.planning/REQUIREMENTS.md` — WST-01, WST-02, WST-05 (Volltext + Ausgangslage/Code-Verifikation).
- `.planning/research/SUMMARY.md` — Research-Synthese (keine neuen Deps; Copy-Vorlagen-Tabelle; Tier-Platzierung).

### Copy-Vorlagen (Backend)
- `dao/src/week_message.rs`, `dao_impl_sqlite/src/week_message.rs` — `(year, calendar_week)`-Composite-Key + DAO-CRUD (nächste Analogie zu `week_status`).
- `service_impl/src/week_message.rs` — `SHIFTPLANNER_PRIVILEGE`-Gate im Basic-Tier-Service + `gen_service_impl!`-Muster.
- `rest/src/week_message.rs` — REST-Handler-Muster (`#[utoipa::path]`, `ToSchema`-DTO).
- `dao_impl_sqlite/src/special_day.rs`, `dao_impl_sqlite/src/extra_hours.rs` — TEXT-Enum-Diskriminant + manuelles `match` im `TryFrom`.
- `migrations/sqlite/20260629000000_create-vacation-entitlement-offset.sql` — Migration mit Soft-Delete + partial UNIQUE als Template.
- `service_impl/src/absence.rs` — `to_iso_week_date()`-Nutzung für ISO-Wochen-Jahr.

### Frontend
- `.planning/codebase/frontend/` — Codebase-Map Frontend.
- Schichtplan-Wochenansicht (Header-Bereich oberhalb des Wochenrasters) — Integrationspunkt für Badge + Schichtplaner-Dropdown.

### Cross-cutting Gates
- `shifty-backend/CLAUDE.md` — Service-Tier-Konvention (Basic vs. Business-Logic), OpenAPI-, Clippy- und `sqlx prepare`-Pflicht-Gates.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`week_message`-Stack** (DAO/Service/REST): nahezu 1:1 als Skelett für `week_status` kopierbar — gleiche `(year, calendar_week)`-Semantik, gleicher `SHIFTPLANNER_PRIVILEGE`-Gate, gleiche Transaktions-/DI-Struktur.
- **`special_day`/`extra_hours` TEXT-Enum-Muster**: Vorlage für den `WeekStatus`-Enum-Diskriminant (String ↔ Enum via manuellem `match` im `TryFrom`).
- **`gen_service_impl!`-Makro**: Standard-DI-Konstruktion für den neuen Basic-Tier-Service.

### Established Patterns
- Soft-Delete + `WHERE deleted IS NULL` + partial UNIQUE — gilt für alle DAOs; hier für die `(year, week)`-Eindeutigkeit.
- ISO-Wochen-Arithmetik über `time` 0.3.36 (`to_iso_week_date`), kein `chrono`.
- Frontend lädt Status nach jeder Mutation frisch vom Server (kein driftbares optimistisches Signal — konsistent mit den Staleness-Guard-Lehren aus v1.9/SDF).

### Integration Points
- `shifty_bin/src/main.rs` — neuer Basic-Tier-Service ins DI-Wiring (vor Business-Logic-Schicht).
- REST-Router — neues `week_status`-Modul + Eintrag in die `ApiDoc`-Struct (Swagger).
- Schichtplan-Wochenansicht-Header (Frontend) — Badge (alle Rollen, nur bei ≠ `Unset`) + Dropdown (nur Schichtplaner).
</code_context>

<specifics>
## Specific Ideas

- User-Zitat (UI): „alle, die kein Schichtplanner sind, den Status sehen, außer er ist None/Open/Unset … dann soll er gar nicht angezeigt werden. Ein Schichtplanner hat einfach ein Dropdown in dem der Status ausgewählt werden kann. Ich sehe das irgendwo oberhalb der Wochenansicht."
- Badge nur bei tatsächlich gesetztem Status — bewusst **kein** leeres Grau-Badge im Default-Fall, um die Wochenansicht ruhig zu halten.
</specifics>

<deferred>
## Deferred Ideas

- **Sperr-Durchsetzung** (Schreibaktionen in Gesperrt-Woche blockieren, `assert_week_not_locked`, `delete_booking`-Re-Routing, HTTP-423, Inline-Banner) → **Phase 40** (WST-03/04), bereits geroadmappt.
- **Bulk-KW-Status** (mehrere Wochen auf einmal setzen/sperren) → v2-Backlog **WST-06**.
- **Publish-Notification** bei Wechsel auf „Geplant" → v2-Backlog **WST-07**.
</deferred>
