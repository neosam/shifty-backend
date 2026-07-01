# Phase 40: Wochen-Sperre durchsetzen (BE+FE) - Context

**Gathered:** 2026-07-01
**Status:** Ready for planning

<domain>
## Phase Boundary

In einer **Gesperrt**-Woche (`WeekStatus::Locked`, aus Phase 39) sind Buchungs- und
Slot-**Schreibaktionen** für **Nicht-Schichtplaner** auf **allen** Schreibpfaden
**server-seitig** blockiert; **Schichtplaner** behalten Vollzugriff. Der Sperr-Check läuft in
**derselben Transaktion** wie der Write (kein TOCTOU). Das Frontend blendet für
Nicht-Schichtplaner in einer Gesperrt-Woche die Schreib-Controls proaktiv aus.

**In Scope (BE):** geteilter `assert_week_not_locked(year, week, context, tx)`-Helper am Kopf
aller sechs Schreibmethoden im Business-Logic-Tier (`ShiftplanEditService`); **neue**
`ShiftplanEditService::delete_booking`-Methode + Re-Routing des `DELETE /booking/{id}`-Handlers
weg von `BookingService::delete` (schließt den einzigen echten Nicht-Schichtplaner-Bypass);
neue `ServiceError::WeekLocked { year, week }`-Variante + HTTP-**423**-Mapping in
`rest/src/lib.rs` (+ OpenAPI-Annotation); Test-Matrix 6 Pfade × {gesperrt, offen}.

**In Scope (FE):** In einer Gesperrt-Woche für Nicht-Schichtplaner die **+/- Buttons ausblenden**
(proaktiv, read-only-Woche). Reine UX-Ergänzung zum Server-Gate — kein Ersatz dafür.

**Out of Scope:** Das Status-Datenmodell + Badge/Dropdown selbst (Phase 39, bereits geliefert).
Bulk-KW-Sperre (WST-06, Backlog). Publish-Notification (WST-07, Backlog). Sperre anderer
Nicht-Shiftplan-Schreibpfade außerhalb der sechs genannten (z.B. Absence/Unavailable) —
bewusst nicht Teil dieser Phase.
</domain>

<decisions>
## Implementation Decisions

### HTTP-Status-Code für Locked-Write (WST-03)
- **D-40-01:** `ServiceError::WeekLocked { year, week }` mappt auf **HTTP 423 Locked**
  (nicht 409). Semantisch exakt, dokumentiert die Sperre klar in der OpenAPI-Spec. Bewusst
  abweichend vom bestehenden 409-Muster (`PaidLimitExceeded`, `OverlappingTimeRange`,
  `EntityConflicts`, `NotLatestBillingPeriod`, `EntityAlreadyExists` sind alle 409) — 423 ist
  der erste seiner Art im Codebase. Begründung: Der Sperr-Fehler ist ein reines Sicherheitsnetz
  (das FE liest den Sperr-Zustand direkt am Wochen-Status, siehe D-40-03), semantische Präzision
  überwiegt Codebase-Konsistenz; Kosten null, da der neue Match-Arm ohnehin compiler-erzwungen ist.

### Umfang der Sperre — welche Aktionen, welche Rollen (WST-03, WST-04)
- **D-40-02:** **Harte Sperre inkl. Entfernen/Selbst-Ausbuchen.** In einer Gesperrt-Woche werden
  für Nicht-Schichtplaner **alle** Schreibaktionen blockiert — inklusive des eigenen Ausbuchens
  bzw. Löschens der eigenen Buchung (genau der neue `delete_booking`-Pfad). **Keine** Ausnahme
  für Self-Service. Konsequenz ist bewusst: ein Nutzer kommt ohne Schichtplaner nicht mehr aus
  einer gesperrten Woche heraus.
- **Bypass-Regel:** Wer die `shiftplan.edit`-Permission hält (Schichtplaner, transitiv Admin),
  umgeht den Lock-Gate vollständig. Der Gate liegt hinter dem bestehenden
  `permission_service.check_permission("shiftplan.edit", context)` (Business-Logic-Tier).

### Frontend-Durchsetzung (WST-02, WST-03)
- **D-40-03:** **Proaktives Ausblenden der +/- Buttons** für Nicht-Schichtplaner, sobald der
  Wochen-Status `Locked` ist. Kein disabled-graues Rendern nötig — Buttons verschwinden. Der
  FE liest den Sperr-Zustand direkt am (bereits in Phase 39 geladenen) Wochen-Status, nicht am
  HTTP-Fehler. Das Server-Gate (D-40-01) bleibt die eigentliche Durchsetzung; das Ausblenden ist
  reine UX, damit niemand gegen eine Wand läuft (und schützt NICHT gegen direkte API-Calls —
  deshalb ist die Server-Sperre nicht verhandelbar).
- **D-40-04:** **Kein Banner / kein zusätzlicher Hinweis.** Das rote **„Gesperrt"-Badge aus
  Phase 39** (bereits im Wochen-Header, für alle Rollen sichtbar) signalisiert die Sperre
  eindeutig; zusammen mit den fehlenden +/- Buttons ist die Read-only-Woche selbsterklärend. Ein
  zusätzlicher Inline-Banner wäre Doppelung und wird **nicht** gebaut. Damit wird die SC1-Formulierung
  „nicht-blockierendes Inline-Banner bei 423" bewusst auf „Badge + ausgeblendete Buttons"
  reduziert (der 423-Pfad ist reines Sicherheitsnetz und braucht keine dedizierte UI-Reaktion).

### i18n (WST-05)
- **D-40-05:** Die lokalisierte Sperr-Rückmeldung (Message-Text der `WeekLocked`-Antwort) in
  de/en/cs. Da kein permanenter FE-Banner gebaut wird (D-40-04), betrifft dies primär den
  Fehler-/Antwort-Text des 423-Sicherheitsnetzes; Wortlaut ist Claude's Discretion (z.B.
  de „Diese Woche ist gesperrt — Änderungen sind nicht möglich.").

### Claude's Discretion
- Exakter Name/Signatur des `assert_week_not_locked`-Helpers und wo er lebt (freie Funktion vs.
  Methode auf `ShiftplanEditService`), solange er in derselben Transaktion wie der Write läuft.
- Genaue Signatur/Verhalten der neuen `ShiftplanEditService::delete_booking` (muss die bisherige
  `BookingService::delete`-Semantik erhalten — Permission/Conflict-Verhalten — und zusätzlich den
  Lock-Gate ziehen; Booking zuerst `get`en, um `year`/`calendar_week` zu lesen).
- Wie der `WeekStatusService`/Week-Status-Read als neue Dependency in `ShiftplanEditServiceDeps`
  verdrahtet wird (Basic-Tier-Service als Dep des Business-Logic-Tier-Service — zulässig laut
  Service-Tier-Konvention).
- Wortlaut der de/en/cs-Sperr-Meldung.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Roadmap & Requirements
- `.planning/ROADMAP.md` §"Phase 40: Wochen-Sperre durchsetzen" — Goal, Success Criteria (4),
  Scope (BE+FE), offene Entscheidung (HTTP-Code → hier als D-40-01 auf 423 entschieden).
- `.planning/REQUIREMENTS.md` — WST-03, WST-04, WST-05 (Volltext + „WST-Sperr-Bypass (kritisch)"-Notiz
  zum `DELETE /booking/{id}`-Re-Routing).

### Phase-39-Fundament (Dependency)
- `.planning/phases/39-kw-status-grundlage/39-CONTEXT.md` — `WeekStatus`-Enum (`Locked`-Variante),
  Persistenz-Modell (Zeilen-Abwesenheit), `week_status`-DAO/Service (Read-by-`(year, week)` ohne
  Privileg-Gate), rotes „Gesperrt"-Badge im Wochen-Header (Basis für D-40-04).

### Backend — Schreibpfade & Error-Mapping (code-verifiziert im Scout)
- `service/src/shiftplan_edit.rs` — Trait der 5 bestehenden Schreibmethoden:
  `book_slot_with_conflict_check` (:118, `Booking` trägt `year`+`calendar_week`),
  `modify_slot` (:41), `modify_slot_single_week` (:62), `remove_slot` (:71),
  `copy_week_with_conflict_check` (:134). Alle tragen bereits `tx` + `context`; (year, week)
  überall in-hand (explizite `change_year`/`change_week`/`from_year`/`to_year`-Args bzw.
  `booking.year`/`booking.calendar_week`).
- `service_impl/src/shiftplan_edit.rs` — Impls; jeweils direkt nach
  `permission_service.check_permission("shiftplan.edit", context)` ist die Einfügestelle des
  Lock-Gates (`modify_slot` :51/:59, `remove_slot` :145/:154, `modify_slot_single_week` :199/:210,
  `book_slot_with_conflict_check` :551, `copy_week_with_conflict_check` :759 → ruft intern
  `book_slot_with_conflict_check` :807, deckt Copy-Ziele mit ab). `ShiftplanEditServiceDeps`
  :26-44 (Week-Status-Dep hier ergänzen).
- `service/src/lib.rs:65-132` — `ServiceError`-Enum (neue `WeekLocked { year, week }`-Variante).
- `rest/src/lib.rs` — `error_handler` :133-285 (KEIN Catch-all → neuer Arm compiler-erzwungen);
  `PaidLimitExceeded` → 409 bei :253-258 als Mapping-Präzedenz (hier bewusst 423 statt 409).
- `rest/src/booking.rs:25` (`DELETE /{id}` Route) + `delete_booking`-Handler :156-172 (ruft heute
  `booking_service().delete(...)` — auf `ShiftplanEditService::delete_booking` umzurouten).
- `service/src/booking.rs:108-113` — bestehende `BookingService::delete`-Signatur (Semantik-Referenz
  fürs neue `delete_booking`); `service/src/booking.rs:12-24` — `Booking` trägt `year`/`calendar_week`.

### Read-Template (Week-Status-Lookup)
- `service_impl/src/week_message.rs` — `get_by_year_and_week` :43-57 (Read ohne Privileg-Gate),
  `SHIFTPLANNER_PRIVILEGE` importiert :6. Nächste Analogie für den Lock-Read.

### Cross-cutting Gates
- `shifty-backend/CLAUDE.md` — Service-Tier-Konvention (Basic vs. Business-Logic; Basic-Dep im
  Business-Logic-Service zulässig), OpenAPI-/Clippy-/`sqlx prepare`-Pflicht-Gates.
- `.planning/codebase/frontend/` — Frontend-Codebase-Map (Schichtplan-Wochenansicht,
  +/- Button-Integrationspunkt für D-40-03).
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Bestehende 5 Schreibmethoden** tragen bereits `tx` + `Authentication<Context>` und haben
  (year, week) in-hand → Lock-Gate ist ein einzeiliger Aufruf am Methoden-Kopf, kein Signatur-Change.
- **`copy_week_with_conflict_check` → `book_slot_with_conflict_check`-Delegation** (:807): ein
  Gate im Book-Pfad deckt Copy-Ziele automatisch mit ab (aber Quell-/Ziel-Woche getrennt prüfen —
  die Ziel-Woche ist die schreibende).
- **`week_message.get_by_year_and_week`**: 1:1-Muster für den Sperr-Read (kein Privileg-Gate auf Read).
- **`PaidLimitExceeded` → 409-Arm** in `error_handler`: strukturelle Vorlage für den neuen
  `WeekLocked`-Arm (nur Status 423 statt 409).

### Established Patterns
- Lock-Check läuft **innerhalb** der bereits offenen Transaktion (`use_transaction(tx)`) → kein
  TOCTOU (SC4). Kein separates Read außerhalb der Write-Transaktion.
- `ServiceError`-Match in `rest/src/lib.rs` ist exhaustiv (kein `_`) → Compiler erzwingt das
  HTTP-Mapping der neuen Variante (Schutz gegen vergessenes Mapping).
- ISO-`(year, week)`: für Slots wird die Woche als explizites Arg geführt (Slot hat kein
  `year`/`calendar_week`); für Bookings direkt aus dem Entity gelesen.

### Integration Points
- `service_impl/src/shiftplan_edit.rs` — 6 Methoden-Köpfe (5 bestehende + neu `delete_booking`);
  `ShiftplanEditServiceDeps` um Week-Status-Read-Dep erweitern.
- `rest/src/booking.rs` — `DELETE /{id}`-Handler auf `ShiftplanEditService::delete_booking` umrouten.
- `service/src/lib.rs` + `rest/src/lib.rs` — neue `WeekLocked`-Variante + 423-Arm + OpenAPI.
- Frontend Schichtplan-Wochenansicht — +/- Buttons an Wochen-Status `Locked` + Nicht-Schichtplaner
  koppeln (ausblenden).
</code_context>

<specifics>
## Specific Ideas

- User zu #2 (FE): „Controls deaktivieren oder einfach die + und - Buttons ausblenden." → +/-
  Buttons ausblenden gewählt.
- User zu #3: „Soll auch für das Entfernen gelten." → harte Sperre inkl. Selbst-Ausbuchen.
- User zu #4: „Es braucht keinen Hinweis. Finde A auch gut." → kein Banner, nur Phase-39-Badge.
- User zu #1: „423." → HTTP 423 Locked.
</specifics>

<deferred>
## Deferred Ideas

- **Bulk-KW-Sperre** (mehrere Wochen auf einmal sperren) → v2-Backlog **WST-06**.
- **Publish-Notification** bei Wechsel auf „Geplant"/Sperre → v2-Backlog **WST-07**.
- **Sperre weiterer Nicht-Shiftplan-Schreibpfade** (z.B. Absence/Unavailable in einer Gesperrt-Woche)
  → bewusst außerhalb dieser Phase; nur die sechs Schichtplan-Schreibpfade werden gegated.
</deferred>

---

*Phase: 40-Wochen-Sperre-durchsetzen*
*Context gathered: 2026-07-01*
