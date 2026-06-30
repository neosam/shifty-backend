---
created: 2026-06-30T00:00:00
title: save_slot_edit hält SLOT_EDIT_STORE-Write-Borrow über .await (already-borrowed Panic-Risiko)
area: frontend / shiftplan
resolves_phase:
files:
  - shifty-dioxus/src/service/slot_edit.rs
---

## Problem

In `shifty-dioxus/src/service/slot_edit.rs` `save_slot_edit()` wird der Write-Guard
`let mut store = SLOT_EDIT_STORE.write();` (Z.55) über die `.await`-Aufrufe
(`save_slot` / `save_slot_single_week` / `create_slot`, Z.65/67/71) gehalten — `store`
wird danach in Z.72/77 (`has_errors`, `visible = false`) weiterbenutzt. Der Kommentar
„Read single_week before the await to avoid holding the write borrow" ist irreführend:
nur der Wert `single_week`, nicht der Guard wird freigegeben.

**Risiko:** `already borrowed`-Panic, falls eine Komponente während des Netzwerk-`await`
`SLOT_EDIT_STORE` liest/schreibt (Dioxus-Signale nutzen intern RefCell).

## Herkunft / Scope

**Pre-existing** (nicht von Phase 35 eingeführt): Die Borrow-über-`await`-Struktur
(New-Branch + Z.77-Reuse) existierte bereits vor Phase 35; Phase 35 fügte nur den
`single_week`-Zweig im selben Scope hinzu. Im Code-Review von Phase 35 als WR-02 (WARNING)
gefunden, bewusst **deferred** (out-of-scope für Phase 35, Verhaltensänderungs-Risiko im
New-Create-Failure-Pfad). Die harten Phase-35-Constraints (Atomarität, keine Doppelzählung)
sind davon nicht betroffen.

## Fix-Skizze

Alle benötigten Werte in einem kurzen `{ let store = SLOT_EDIT_STORE.read(); (…) }`-Block
in owned Locals extrahieren, den Borrow **vor** jedem `await` droppen, nach dem `await`
für `has_errors`/`visible` einen frischen `SLOT_EDIT_STORE.write()` nehmen. Dabei die
bestehende Semantik des New-Create-Failure-Pfads exakt erhalten.
