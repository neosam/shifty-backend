---
phase: 42-special-days-anlegen-button-bugfix-fe
reviewed: 2026-07-02T00:00:00Z
depth: standard
files_reviewed: 1
files_reviewed_list:
  - shifty-dioxus/src/page/settings.rs
findings:
  critical: 0
  warning: 1
  info: 1
  total: 2
status: issues_found
---

# Phase 42: Code Review Report

**Reviewed:** 2026-07-02
**Depth:** standard
**Files Reviewed:** 1 (`shifty-dioxus/src/page/settings.rs`)
**Status:** issues\_found (0 BLOCKER, 1 WARNING, 1 INFO)

## Zusammenfassung

Der Fix ist korrekt und sicher. Die extrahierte pure Funktion `is_special_day_form_valid` ist semantisch byte-for-byte identisch mit dem alten Inline-Ausdruck. Die Retention-Policy setzt die drei Felder auf die Werte vor dem API-Call — das ist funktional äquivalent zu „nicht zurücksetzen". Kein Scope Creep (kein Backend, keine REST-Types, keine i18n-Änderungen). Die Tests sind sinnvoll und tautologiefrei: `special_day_form_retained_after_create` prüft die Policy-Invariante, `special_day_form_valid_stays_true_after_create` prüft den End-zu-End-Zusammenhang. `sd_is_duplicate` bleibt korrekt rein informativ (kein Buttonblock). Kein Signal↔DOM-Desync durch die Änderung — `sd_type` wird auf den retained Wert (non-None) gesetzt, nicht auf None, was die controlled-select-Invarianz (D-06/D-08) erhält, nicht bricht.

Ein veralteter Doc-Comment bleibt nach dem Bugfix stehen (WARNING). Eine Design-Anmerkung zur trivialen Identity-Funktion (INFO).

---

## Warnings

### WR-01: Veralteter Doc-Comment auf `sd_type_to_select_value`

**File:** `shifty-dioxus/src/page/settings.rs:68-70`
**Issue:** Der Doc-Comment beschreibt das Pre-Phase-42-Verhalten, nämlich dass `sd_type.set(None)` nach erfolgreichem Create das Dropdown leert und den Anlegen-Button wieder aktiviert. Phase 42 hat diesen Reset explizit entfernt — nach Create wird `sd_type.set(retained.ty)` mit dem **unveränderten** Wert aufgerufen. Der Kommentar ist damit in seiner Kernaussage falsch und könnte zukünftige Entwickler in die Irre führen, wenn sie die Funktion oder das D-08-Verhalten verstehen wollen.

```rust
// IST (veraltet nach Phase 42):
/// Used to derive a controlled `value` prop from the `sd_type` signal so that
/// `sd_type.set(None)` after a successful create visibly clears the dropdown and
/// re-enables the Anlegen button (D-08: the date field is already controlled via
/// `value: ImStr::from(sd_date_val.as_str())` so no change is needed there).

// SOLL (korrekte Beschreibung post-Phase-42):
/// Used to derive a controlled `value` prop from the `sd_type` signal so that
/// the Dioxus-managed DOM attribute stays in sync with the signal state (D-06/D-08).
/// After Phase-42 the form is retained after create (`special_day_form_after_create`),
/// so the select continues to show the chosen type — no reset to `None` occurs.
```

---

## Info

### IN-01: `special_day_form_after_create` ist eine triviale Identity-Funktion

**File:** `shifty-dioxus/src/page/settings.rs:133-135`
**Issue:** Die Funktion macht ausschließlich `before.clone()` und enthält keine eigene Logik. Das Design als „Policy-Hook" ist intentional und im Kommentar begründet, aber der Aufruf-Stack ist dadurch nur schwerlich nachvollziehbar (Leser müssen in die Funktion navigieren, um zu sehen, dass nichts selektiert oder transformiert wird). Kein Bug, aber bei einer künftigen Retention-Policy-Änderung (z. B. Datum behalten, Typ zurücksetzen) würde diese Funktion tatsächlich Logik enthalten und ihr Zweck klar. Solange die Policy trivial ist, könnte ein Inline-Kommentar `// D-42-01: keep all three fields` mit direkten `.clone()`-Aufrufen die Lesbarkeit erhöhen.

**Fix:** (optional) Entweder so belassen mit dem Hinweis, dass die Funktion als Erweiterungspunkt dient — oder die drei Felder direkt mit einem erklärenden Kommentar inline setzen. Kein Handlungsbedarf vor dem Ship.

---

_Reviewed: 2026-07-02_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
