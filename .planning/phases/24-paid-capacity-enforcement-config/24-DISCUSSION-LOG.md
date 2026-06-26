# Phase 24: Paid-Limit konfigurierbar & rollenbasiert durchsetzen - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-06-27
**Phase:** 24-paid-capacity-enforcement-config
**Areas discussed:** Toggle-Speicherung & Konfig-Weg, Hard-Block-Semantik & Shiftplanner-Ausnahme, Bestandsbuchungen & Grenzregel, Overage-Anzeige, Fehler-UX beim harten Block, Settings-Seite Platzierung & Scope, Toggle-Seeding & Naming, Backend liest den Modus

---

## Toggle-Speicherung & Konfig-Weg

| Option | Description | Selected |
|--------|-------------|----------|
| feature_flag (reuse) | feature_flag-Tabelle + FeatureFlagService, admin-gated | |
| ToggleService (reuse) | toggle-Mechanismus (User-Toggles, toggle_admin, Gruppen) | ✓ |
| Neuer SettingsService | eigene typisierte Settings-Schicht bauen | |

**User's choice:** ToggleService (reuse). Zuvor geklärt: feature_flag wird bewusst für spätere Vermarktung/SaaS-Gating reserviert (Instanz-Admin soll es NICHT editieren); der Paid-Limit-Modus ist dagegen ein vom Admin editierbarer Betriebs-Schalter. ToggleService = faktisch der gewünschte „SettingsService".
**Notes:** Der User kannte den ToggleService nicht; nach Erklärung (generischer admin-editierbarer Boolean-Speicher, gated via toggle_admin, voll ausgebaut aber ohne Business-Konsumenten) als passend bestätigt. Neuer-SettingsService-Versuch war ein versehentlicher Klick (Claude-Code-Update sendet beim Klick ins Fenster).

### Konfig-UI
**User's choice:** Frontend-Settings-Oberfläche (sichtbar nur für toggle_admin), Admin legt Schalter per Klick um.

---

## Hard-Block-Semantik & Shiftplanner-Ausnahme

| Option | Description | Selected |
|--------|-------------|----------|
| Nur Shiftplanner | nur SHIFTPLANNER_PRIVILEGE darf überbuchen | ✓ |
| Shiftplanner ODER HR | beide dürfen überbuchen | |
| Du entscheidest | — | |

**User's choice:** Nur Shiftplanner. Im Laufe der Diskussion deckte der User einen Permission-Bug auf: das Buchungs-Gate ist `HR ∨ self`, aber ein Schichtplaner (nicht HR) muss zuweisen können — HR soll das gar nicht. Verifiziert: `get_bookable_sales_persons` ist bereits shiftplanner-gated, `book_slot_with_conflict_check` aber HR∨self → echte Inkonsistenz.

### Buchungs-Gate-Fix (D-24-04)
| Option | Description | Selected |
|--------|-------------|----------|
| Shiftplanner ∨ self | HR raus, Shiftplanner rein | ✓ |
| Shiftplanner ∨ HR ∨ self | additiv, HR behalten | |
| Du entscheidest | — | |

**User's choice:** Shiftplanner ∨ self (HR raus). In Phase 24 gefoldet, da Prerequisite für D-24-02.

### copy_week
**User's choice:** „copy_week ignorieren. Das ist deprecated." → kein harter Block, bleibt soft.

---

## Bestandsbuchungen & Grenzregel

| Option | Description | Selected |
|--------|-------------|----------|
| Strikt-größer (= heutige Warnung) | nur is_paid, current>max, bestehende nie angefasst | ✓ |
| Andere Regel | abweichend | |
| Du entscheidest | — | |

**User's choice:** Strikt-größer, deckungsgleich mit der bestehenden Warnung.
**Notes:** Impl-Hinweis aufgenommen: Check muss VOR dem Persistieren greifen (heute wird erst gebucht, dann gezählt).

---

## Overage-Anzeige (D-24-03)

| Option | Description | Selected |
|--------|-------------|----------|
| Warn-Icon + Tooltip | Icon in Zelle + Hover-Erklärung | |
| Zahlen-Badge (paid X/Y) | dauerhaftes Badge (Phase 23 verworfen) | |
| Du entscheidest | — | |

**User's choice:** (Other) „Über dem Schichtplan wie bei den anderen Warnungen." → persistente Warn-Sektion über dem Plan, analog WarningList/Konflikt-Sektion; zusätzlich zur roten Zelle; clientseitig, kein Backend-Change.

### Sichtbarkeit
| Option | Description | Selected |
|--------|-------------|----------|
| Alle Rollen | konsistent mit Phase-23 D-23-05 | ✓ |
| Nur Shiftplanner | wie Konflikt-Sektion | |
| Du entscheidest | — | |

**User's choice:** Alle Rollen.

---

## Fehler-UX beim harten Block (D-24-05)

| Option | Description | Selected |
|--------|-------------|----------|
| Dedizierte Inline-Meldung am Slot | neuer ServiceError, eigener Status, lokalisierte Inline-Meldung | ✓ |
| Generischer Fehler-Toast | ValidationError(422) via error_handler | |
| Du entscheidest | — | |

**User's choice:** Dedizierte Inline-Meldung am Slot.
**Notes:** Falle entdeckt: ServiceError::Forbidden→403 wird vom Frontend heute still ignoriert (D-13). Der Block braucht daher einen nicht-stillen, unterscheidbaren Status.

---

## Settings-Seite: Platzierung & Scope (D-24-06)

| Option | Description | Selected |
|--------|-------------|----------|
| Neue Seite, nur dieser Schalter | eigene /settings/-Route, admin-gated, ein Schalter | ✓ |
| Neue Seite, generische Toggle-Liste | get_all_toggles, alle Toggles | |
| In bestehende Seite einhängen | z.B. User-Management | |

**User's choice:** Neue Seite, nur dieser Schalter (später erweiterbar).

---

## Toggle-Seeding & Naming (D-24-07)

| Option | Description | Selected |
|--------|-------------|----------|
| Migration-INSERT, Key 'paid_limit_hard_enforcement' | enabled=0=weich, keine Gruppe | ✓ |
| Anderer Key/Ansatz | — | |
| Du entscheidest | — | |

**User's choice:** Migration-INSERT, Key `paid_limit_hard_enforcement` (enabled=0=weich, keine Gruppe). enabled=true→hart.

---

## Backend liest den Modus (D-24-08)

| Option | Description | Selected |
|--------|-------------|----------|
| Pre-Persist-Check im Buchungspfad | ToggleService-Dep, Toggle pro Buchung gelesen, Block vor Persist | ✓ |
| Anderer Ansatz | eigener Guard-Service / gecacht | |
| Du entscheidest | — | |

**User's choice:** Pre-Persist-Check im Buchungspfad.

---

## Claude's Discretion

- Exakter Name & HTTP-Status des neuen Block-ServiceError (409 vs 422).
- i18n-Formulierungen (Settings-Schalter, Block-Meldung, Overage-Sektion) De/En/Cs.
- Layout/Styling Settings-Seite + Overage-Sektion.
- Ob Overage-Sektion `WarningList` wiederverwendet oder eigene Komponente.

## Deferred Ideas

- feature_flag-basiertes Vermarktungs-/SaaS-Gating (eigener späterer Mechanismus).
- Generische Settings-/Toggle-Verwaltungs-UI (get_all_toggles).
- copy_week an Enforcement anpassen (deprecated).
