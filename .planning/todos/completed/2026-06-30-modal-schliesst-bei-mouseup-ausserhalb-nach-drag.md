---
created: 2026-06-30T20:52:29.810Z
title: Modal schließt bei mouseup außerhalb, wenn Drag (Text-Selektion) innen begann
area: frontend
resolves_phase: 44
files:
  - shifty-dioxus/src/component/dialog.rs:195
  - shifty-dioxus/src/component/dialog.rs:321
  - shifty-dioxus/src/component/contract_modal.rs
  - shifty-dioxus/src/component/absence_convert_modal.rs:88
---

## Problem

In einem Modal (z. B. **Arbeitsvertrag editieren**): Hält man die Maustaste **innerhalb** des
Modals gedrückt, um Text zu **selektieren**, und lässt sie **außerhalb** des Modals los,
**schließt sich das Modal**. Erwartung: Das Modal soll sich nur bei einem echten
Außerhalb-**Klick** schließen — ein `mouseup` außerhalb nach einem innen begonnenen Drag
darf NICHT schließen.

**Ursache:** Der Backdrop schließt per `onclick`/`mouseup` → `on_close.call(())`
(`dialog.rs:195`/`:321`; gleiches Muster `absence_convert_modal.rs:88-91`: `fixed inset-0
bg-modal-veil` outer + inner `stop_propagation`). Endet ein innen begonnener Drag auf dem
Backdrop, ist das `click`/`mouseup`-Target der Backdrop → `on_close` feuert. Das innere
`stop_propagation` greift nicht, weil das Event-Target bereits der Backdrop ist.

**Generisch:** Betrifft wahrscheinlich **alle Modals**, die das geteilte `dialog.rs`-Backdrop-
Muster nutzen (contract_modal, slot_edit, extra_hours_modal, absence_convert_modal, …).
Live gemeldet vom User 2026-06-30.

## Solution

Zentral in `dialog.rs` (damit alle Modals profitieren): Schließen nur, wenn **sowohl der
mousedown ALS AUCH der schließende Event** auf dem Backdrop selbst (nicht im Inhalt)
stattfinden.

Standard-Fix:
- Backdrop `onmousedown`: setze ein Flag `pressed_on_backdrop = (event.target == backdrop)`
  (d. h. nur wenn der Druck wirklich auf dem Veil und nicht im Modal-Inhalt begann).
- Backdrop-Close (`onclick`/`onmouseup`) feuert `on_close` **nur wenn** `pressed_on_backdrop`
  true ist; Flag danach zurücksetzen.
- Alternativ: auf `onclick` des Backdrops prüfen, dass `event.target == current_target`
  (Klick direkt auf den Veil, nicht durch-gebubbled) UND den mousedown-Ursprung tracken.

SSR/Interaktions-Test (soweit in Dioxus testbar): Drag-Selektion innen → mouseup außen
→ Modal bleibt offen. Hinweis: Maus-Drag ist im Browser schwer automatisierbar (D-25-06-Klasse)
→ ggf. strukturell über die Predikat-/Handler-Logik testen.
