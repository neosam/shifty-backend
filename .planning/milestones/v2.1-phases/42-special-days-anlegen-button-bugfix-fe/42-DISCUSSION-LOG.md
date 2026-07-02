# Phase 42: Special-Days-„Anlegen"-Button-Bugfix (FE) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-02
**Phase:** 42-special-days-anlegen-button-bugfix-fe
**Areas discussed:** Duplikat-Schutz, Was bleibt/was gebraucht, Erfolgs-Feedback, Test-Strategie

---

## WR-02 Duplikat-Schutz

| Option | Description | Selected |
|--------|-------------|----------|
| Button bei Duplikat sperren | `sd_is_duplicate` an `button.disabled` koppeln — Button bleibt aktiv außer bei exaktem Duplikat | |
| Duplikat akzeptieren | Keine Button-Sperre; Backend behandelt Duplikat als in-place Replace (422→success) | ✓ |

**User's choice:** Einfach akzeptieren. Wird im Backend geprüft.
**Notes:** Inline-Hinweis `sd_is_duplicate` bleibt informativ sichtbar, aber ohne Kopplung an den Button. Kehrt bewusst die Phase-36-WR-02-Entscheidung um.

---

## Was bleibt vs. was noch gebraucht wird

| Option | Description | Selected |
|--------|-------------|----------|
| Wörtlich nichts zurücksetzen | Auch `sd_year.set` + `sd_resource.restart()` entfernen | |
| Nur die drei Feld-Resets raus (Claude-Empfehlung) | `sd_year.set(iso_year)` (WR-04) + `sd_resource.restart()` bleiben | ✓ |

**User's choice:** Mach was du empfiehlst.
**Notes:** Year-Switch (Jahresgrenzen-Sichtbarkeit) und Liste-neu-Laden sind Korrektheits-Logik, keine Formular-Resets.

---

## Erfolgs-Feedback

| Option | Description | Selected |
|--------|-------------|----------|
| Beim nächsten Feld-Edit clearen | „Gespeichert" verschwindet, sobald der User ein Feld ändert | |
| Bis zum nächsten Klick stehen lassen | „Gespeichert" bleibt sichtbar bis zum nächsten Submit | ✓ |

**User's choice:** Gespeichert bis zum nächsten Klick.
**Notes:** `sd_save_result` unverändert; `set(None)` am Kopf von `on_add_special_day` räumt beim nächsten Submit auf.

---

## Test-Strategie (SC #3: SSR-/Komponenten-Test)

| Option | Description | Selected |
|--------|-------------|----------|
| Nur Pure-Unit | Validitäts-Prädikat extrahieren + unit-testen | |
| Nur SSR/VirtualDom | Komponente mounten, Button-Zustand prüfen | |
| Beides / Claude entscheidet | Pure-Unit als hartes Gate, SSR best-effort je nach Mountbarkeit | ✓ |

**User's choice:** Ich kanns dir echt nicht beantworten. Prüfe was besser ist und mach es. Oder evtl. beides? Kannst du entscheiden.
**Notes:** Entscheidung: Pure-Unit-Test (Prädikat + Retention-Policy-Modell) ist Pflicht/hartes Gate; VirtualDom-/SSR-Render-Test best-effort, nur falls ohne Live-Backend/Config mountbar, sonst begründete Skip-Notiz. Motiviert durch bekannte Dioxus-Browser-Test-Grenzen bei `<input type=date>`.

---

## Claude's Discretion

- Test-Granularität und Pure-Unit-only vs. Pure-Unit + SSR (D-42-05/06) an Claude delegiert.
- Konkrete Wahl „nur Feld-Resets entfernen" (D-42-02) als Claude-Empfehlung angenommen.

## Deferred Ideas

None — Diskussion blieb im Phasen-Scope (isolierter FE-Bugfix).
