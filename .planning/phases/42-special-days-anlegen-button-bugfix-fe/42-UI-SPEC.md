---
phase: 42
slug: special-days-anlegen-button-bugfix-fe
status: draft
shadcn_initialized: false
preset: none
created: 2026-07-02
---

# Phase 42 — UI Design Contract

> **Proportionalitätshinweis:** Phase 42 ist ein isolierter Frontend-State-Bugfix ohne neue
> UI-Elemente, neue Komponenten oder Designänderungen. Dieses Dokument ist bewusst minimal
> gehalten. Alle 6-Pillar-Sektionen, die nicht betroffen sind, sind mit N/A und einzeiligem
> Grund markiert. Der einzige normative Inhalt ist das Observable-State-Contract (Abschnitt
> "Verhaltensvertrag").

---

## Assumptions (autonomous)

Diese Spezifikation wurde im AUTONOMOUS-Modus ohne interaktive Rückfragen erstellt. Alle
Entscheidungen stammen vollständig aus den gesperrten CONTEXT.md-Decisions D-42-01 bis D-42-06
sowie dem Codebestand (`settings.rs`, `CLAUDE.md`).

- **Toolstack unverändert:** Dioxus 0.6.x + Tailwind CSS (custom Token-Set `text-body`,
  `text-small`, `text-ink`, `text-bad`, `bg-surface`, etc.). Kein shadcn, kein neues Icon,
  kein neues Design-Token.
- **Keine neue Komponente:** Der bestehende `Btn { variant: BtnVariant::Primary, disabled,
  on_click }` (`component/atoms/btn.rs`) wird nicht geändert — nur das `disabled`-Argument
  verhält sich nach dem Fix korrekt.
- **Kein neues i18n-Schlüssel:** Alle sichtbaren Texte (Button-Label `SettingsSpecialDaysAddBtn`,
  Duplikat-Hinweis `SettingsSpecialDaysDuplicateHint`, Erfolgsmeldung `SettingsSaved`) bleiben
  unverändert. Phase 42 verursacht keinen i18n-Bedarf.
- **Kein Backend-Anteil:** Reiner FE-State-Fix; API, TOs und Snapshot-Version bleiben
  unverändert.

---

## Design System

N/A — kein Systemwechsel in dieser Phase.

| Property | Value |
|----------|-------|
| Tool | none (Tailwind CSS, Dioxus RSX) |
| Preset | nicht anwendbar |
| Component library | Dioxus eigene Atom-Komponenten (`component/atoms/`) |
| Icon library | bestehend, unverändert |
| Font | bestehend, unverändert |

---

## Spacing Scale

N/A — kein neues Layout, keine neuen Abstände. Bestehende Tailwind-Utilities (`px-4`, `py-4`,
`gap-3` etc.) bleiben unverändert.

---

## Typography

N/A — keine neuen Textelemente. Bestehende Token-Klassen (`text-body`, `text-small`, `text-h2`,
`font-semibold`) bleiben unverändert.

---

## Color

N/A — keine neuen Farbelemente. Bestehende semantische Klassen (`text-bad` für Duplikat-Hinweis,
`text-ink-muted` für Erfolgsmeldung) bleiben unverändert.

---

## Verhaltensvertrag (Observable-State-Contract)

Dies ist der einzige normative Abschnitt dieser Spezifikation. Planner und Executor orientieren
sich ausschließlich hieran.

### Button-Enabled-Regel (D-42-01)

**Invariante:** `Btn.disabled` ist genau dann `true`, wenn `!sd_form_valid || *sd_saving.read()`.

**sd_form_valid** ist genau dann `true`, wenn alle drei Bedingungen erfüllt sind:
1. `sd_date_str` ist nicht leer.
2. `sd_type` ist `Some(_)` (irgendein Typ gewählt).
3. Wenn `sd_type == Some(ShortDay)`: `sd_time_str` ist nicht leer. Sonst: keine Bedingung an
   `sd_time_str`.

**Nach einem erfolgreichen Create** dürfen `sd_date_str`, `sd_type` und `sd_time_str` NICHT
zurückgesetzt werden. Die drei Felder bleiben unverändert. Damit bleibt `sd_form_valid` `true`
und der „Anlegen"-Button bleibt aktiv. (Quelle: D-42-01)

**Das Validitäts-Prädikat** (`date non-empty && type is Some && (type != ShortDay || time
non-empty)`) muss als reine Funktion extrahiert und unit-getestet werden, inkl. des Falls
„bleibt `true` für ein bereits ausgefülltes Formular nach Create". (Quelle: D-42-05)

### Feldrückhalt nach Create (D-42-01, D-42-02)

| Signal | Verhalten nach Create |
|--------|----------------------|
| `sd_date_str` | bleibt unveraendert (Reset-Zeile wird entfernt) |
| `sd_type` | bleibt unveraendert (Reset-Zeile wird entfernt) |
| `sd_time_str` | bleibt unveraendert (Reset-Zeile wird entfernt) |
| `sd_year` | wird auf `iso_year` des gerade angelegten Eintrags gesetzt (WR-04, bleibt erhalten) |
| `sd_resource` | wird neu gestartet, um die Liste zu aktualisieren (bleibt erhalten) |
| `sd_saving` | wird auf `false` zurückgesetzt (bleibt erhalten) |
| `sd_save_result` | wird auf `Some(true)` gesetzt → zeigt „Gespeichert"-Hinweis (bleibt erhalten) |

### Duplikat-Hinweis-Entkopplung (D-42-03)

Der bestehende Inline-Hinweis `sd_is_duplicate` (via `is_duplicate_special_day`) bleibt
informativ sichtbar (Klasse `text-small text-bad`). Er ist NICHT an `Btn.disabled` gekoppelt.
Ein sofortiger Zweitklick bei unveränderten Feldern (exaktes Duplikat) ist zulässig — das
Backend behandelt ihn idempotent (HTTP 422 → success, Phase-36-Verhalten). Es darf KEINE
zusätzliche Button-Sperre wegen eines Duplikats eingeführt werden.

### Erfolgsmeldung (D-42-04)

`sd_save_result == Some(true)` → zeigt den i18n-Text `SettingsSaved` in der Klasse
`text-small text-ink-muted`. Diese Meldung bleibt sichtbar, bis der nächste „Anlegen"-Klick
erfolgt (`sd_save_result.set(None)` am Kopf von `on_add_special_day`). Es darf KEIN
automatisches Ausblenden beim Bearbeiten von Feldern eingeführt werden.

### Benutzer-Flow-Prüfstein

```
Tag A anlegen → [Button bleibt aktiv, Felder gefüllt, "Gespeichert" sichtbar,
                 Duplikat-Hinweis erscheint für A]
→ Datum auf Tag B ändern → [Duplikat-Hinweis verschwindet, Button bleibt aktiv]
→ Tag B anlegen (kein Dropdown-Toggle nötig)
```

---

## Copywriting Contract

N/A — keine neuen Textelemente. Alle Kopien entstammen bestehenden i18n-Keys:

| Element | i18n-Key | Sichtbarkeit |
|---------|----------|-------------|
| Button-Label | `SettingsSpecialDaysAddBtn` | immer sichtbar (unverändert) |
| Erfolgsmeldung | `SettingsSaved` | nach Create, bis nächstem Submit |
| Duplikat-Hinweis | `SettingsSpecialDaysDuplicateHint` | bei `sd_is_duplicate == true` |
| Fehlermeldung | `SettingsSaved`-Negativ (Some(false)) | bei API-Fehler |

Kein neuer i18n-Key in de/en/cs erforderlich.

---

## Registry Safety

N/A — keine neuen Pakete, keine Registries. Kein shadcn, keine Drittanbieter-Blöcke.

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| — | keine | nicht anwendbar |

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS — keine neuen Texte; bestehende Keys unverändert
- [ ] Dimension 2 Visuals: PASS — keine neuen visuellen Elemente
- [ ] Dimension 3 Color: PASS — keine neuen Farben
- [ ] Dimension 4 Typography: PASS — keine neuen Typografie-Token
- [ ] Dimension 5 Spacing: PASS — keine neuen Abstände
- [ ] Dimension 6 Registry Safety: PASS — keine externen Registries

**Approval:** pending
