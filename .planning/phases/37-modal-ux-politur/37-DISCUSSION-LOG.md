# Phase 37: Modal-UX-Politur (FE) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-01
**Phase:** 37-modal-ux-politur
**Areas discussed:** MOD-01 absence_convert Doppel-Fix, MOD-01 Fix-Mechanik, MOD-02 CommittedVoluntary Help, MOD-02 Platzierung/Klassen
**Format:** Textform (User-Präferenz — kein AskUserQuestion-Dialog)

---

## MOD-01 — absence_convert_modal Doppel-Fix

| Option | Description | Selected |
|--------|-------------|----------|
| (a) Auch absence_convert_modal.rs:88-91 mitfixen | Eigener Backdrop, gleicher Bug; zentraler dialog.rs-Fix erreicht ihn nicht | ✓ |
| (b) Bewusst weglassen | Nur dialog.rs | |

**User's choice:** (a) beide fixen
**Notes:** Gleiche gemeldete Bug-Klasse — sonst bleibt bekannter Duplikat-Bug.

---

## MOD-01 — Fix-Mechanik

| Option | Description | Selected |
|--------|-------------|----------|
| (a) Signal-Flag-Muster | onmousedown am Backdrop setzt Flag; Panel stop_propagation; schließen nur wenn Flag | ✓ |
| (b) web_sys target/currentTarget-Vergleich | Raw DOM | |

**User's choice:** (a) Signal-Flag
**Notes:** Dioxus MouseData exponiert kein target/currentTarget (Scout bestätigt). Idiomatisch, kein web_sys nötig, strukturell testbar.

---

## MOD-02 — CommittedVoluntary Help-Text

| Option | Description | Selected |
|--------|-------------|----------|
| (a) Help-Satz ergänzen | Braucht einen de-Text | ✓ |
| (b) Bewusst weglassen | — | |

**User's choice:** (a) — Text: „Zugesagte freiwillige Stunden." (bewusst knapp, User-Wortlaut; „mehr nicht")
**Notes:** Vorschlag des Assistenten war länger; User kürzte auf „Zugesagte freiwillige Stunden."

---

## MOD-02 — Platzierung/CSS-Klassen

| Option | Description | Selected |
|--------|-------------|----------|
| (a) Sibling-span mit cap-Klassen | text-small font-normal text-ink-muted, wie CapPlannedHoursHelp | ✓ |
| (b) help-Slot am Field-Atom | Sauberer, aber größerer Eingriff | |

**User's choice:** (a) Sibling-span (Empfehlung)
**Notes:** Minimal & konsistent zum bestehenden cap-Muster.

---

## Claude's Discretion

- Exakte en/cs-Übersetzungen der neuen `*Help`-Keys.
- Signal-Flag als `use_signal` in DialogContent vs. onmousedown+stop_propagation am Panel.

## Deferred Ideas

None — Diskussion blieb im Phasen-Scope. HYG-01/02 → Phase 38.
