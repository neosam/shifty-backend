---
created: 2026-06-30T20:58:23.610Z
title: Arbeitsvertrag-Modal — Erklärungssatz pro Feld (analog CapPlannedHoursHelp)
area: frontend
files:
  - shifty-dioxus/src/component/contract_modal.rs:137
  - shifty-dioxus/src/component/contract_modal.rs:144
  - shifty-dioxus/src/i18n/de.rs
  - shifty-dioxus/src/i18n/en.rs
  - shifty-dioxus/src/i18n/cs.rs
---

## Problem

Im **Arbeitsvertrag-Modal** (`contract_modal.rs`) soll **jedes Feld einen kurzen
Erklärungssatz** bekommen — genau wie es bei „Geplante Stunden auf Soll deckeln" bereits
der Fall ist (`CapPlannedHoursLabel` + `CapPlannedHoursHelp`, `contract_modal.rs:143-144`).
Aktuell haben die übrigen Felder nur Labels (`:135-142`), keinen Help-Text.

**Von/Bis sind ausgenommen** (selbsterklärend). User-Wunsch 2026-06-30.

## Solution

Pro Feld einen `*Help`-i18n-Key analog zu `CapPlannedHoursHelp` anlegen (de/en/cs) und unter
dem Feld als `text-small text-ink-muted` rendern (gleiches Muster wie `cap_help`). Texte
(verbatim vom User, de):

| Feld (Label-Key) | neuer Help-Key | Text (de) |
|------------------|----------------|-----------|
| Wochentage (`WorkdaysLabel`) | `WorkdaysHelp` | „Die Tage, an denen die Person in der Regel arbeitet." |
| Wochenarbeitsstunden (`ExpectedHoursPerWeekLabel`) | `ExpectedHoursPerWeekHelp` | „Wie viele Sollstunden pro Woche." |
| Arbeitstage pro Woche (`DaysPerWeekLabel`) | `DaysPerWeekHelp` | „An wie vielen Tagen die Person in der Regel reinkommt." |
| Urlaubsanspruch im Jahr (`VacationEntitlementsPerYearLabel`) | `VacationEntitlementsPerYearHelp` | „Der gesamte Jahresurlaub laut Vertrag im Jahr." |
| Dynamische Stunden (`DynamicHourLabel`) | `DynamicHourHelp` | „Das Soll entspricht immer den geleisteten Stunden — ideal, wenn die Person nach Stunden bezahlt wird." |
| Von / Bis (`FromLabel`/`ToLabel`) | — | (kein Help — selbsterklärend) |

Hinweise:
- **CapPlannedHoursHelp** existiert bereits → als Vorbild/Pattern nehmen (Rendering + Key-Struktur).
- **Committed-Voluntary** (`CommittedVoluntaryLabel`, conditional `show_committed`) hat der User
  nicht erwähnt — optional einen Help-Satz ergänzen oder bewusst weglassen.
- en/cs-Übersetzungen für alle neuen Keys ergänzen (i18n-Pflicht de/en/cs).
- Alle drei Locales in `i18n/{de,en,cs}.rs` + ggf. `i18n/mod.rs` (Key-Variante) erweitern.
- SSR-Test: Help-Texte werden unter den jeweiligen Feldern gerendert.
