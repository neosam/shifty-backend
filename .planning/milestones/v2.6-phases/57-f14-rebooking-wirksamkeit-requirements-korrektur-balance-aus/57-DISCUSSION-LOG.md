# Phase 57: F14-Rebooking-Wirksamkeit — Requirements-Korrektur + Balance-Ausgleich - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-31
**Phase:** 57-f14-rebooking-wirksamkeit-requirements-korrektur-balance-aus
**Areas discussed:** Fachliches Sollverhalten, Rückwirkung auf Bestandsdaten, HR-Alert-Banner, Produktionsstand des Auto-Cron

---

## Ablauf der Sitzung

Die erste Fragenrunde bot zehn nummerierte Graubereiche mit Code-Referenzen an
(Filter-Semantik, Balance-Zeitpunkt, REQ-IDs, Snapshot-Fixture, …). Der User hat
abgebrochen: „Boa kannst du das alles einfacher formulieren? Du stellst da Fragen
ohne dass ich den Kontext kenne."

Konsequenz für den weiteren Verlauf: Die technischen Detailfragen wurden aus der
Diskussion herausgenommen und von Claude selbst entschieden (D-57-06 bis D-57-12);
dem User wurden nur noch drei Fragen zu fachlichem Verhalten, Bestandsdaten und
Produktionsstand gestellt — mit konkreten Zahlenbeispielen statt Funktionsnamen.
Als dauerhafte Arbeitsanweisung festgehalten in
`memory/feedback-einfache-fragen-mit-kontext.md`.

---

## Fachliches Sollverhalten

Nicht als Optionsliste gestellt — der User hat das Sollverhalten unaufgefordert
und vollständig spezifiziert, bevor die vereinfachte Fragenrunde lief.

**User's Beitrag (wörtlich):** „Die paid-balance soll automatisch 1x die Woche
ausgeglichen werden, solange die volunteer-balance nicht unter 0 geht. Sie darf bis
0 gehen. Alles darüber soll manuell in der UI an HR vorgeschlagen werden aber bei
den Vorschlägen sollen die insgesamten freiwilligen Stunden nicht unter 0 gehen,
denn das wäre Quatsch. Man kann nicht insgeamt im Jahr -3 Stunden freiwillig
arbeiten."

**Notes:** Claude hat das in einem Beispiel zurückgespiegelt (Vertrag 5 h + 5 h,
20 h gearbeitet → Übertrag 2 h; Gegenprobe 23 h → Übertrag trotzdem 2 h) — vom User
nicht korrigiert. Bestätigt und präzisiert die bestehende Memory
`f14-rebooking-sollverhalten`: Auto-Cron-Untergrenze ist das Freiwilligen-Delta
(`ist − committed ≥ 0`), HR-Untergrenze das Jahres-Ist (`ist ≥ 0`). Damit ist
`proposed_hours = min(|balance|, ist)` bestätigt und bleibt unverändert.
→ CONTEXT.md D-57-01, D-57-02

---

## Rückwirkung auf Bestandsdaten

| Option | Description | Selected |
|--------|-------------|----------|
| a) Rückwirkend wirken lassen | Alte Umbuchungen korrigieren die Konten nachträglich. Nur wenige Wochen aus 2026, kein Jahreswechsel dazwischen. | ✓ |
| b) Nur neue wirken lassen | Read-seitiger Stichtag-Guard, alte Rebooking-Rows bleiben wirkungslos. | |

**User's choice:** a)
**Notes:** Entlastender Befund aus dem Scout: `carryover_hours` wird erst zum
Jahreswechsel aus `balance_hours` geschrieben (`shiftplan_edit.rs:384`); alle
Rebooking-Rows stammen aus 2026 (F3 ab 2026-07-24). Es existiert daher kein stale
Carryover. → CONTEXT.md D-57-03

---

## HR-Alert-Banner (D-55-07)

| Option | Description | Selected |
|--------|-------------|----------|
| a) Nur bei ausgeglichenem Konto | Warnung verschwindet nur, wenn die Balance danach wirklich stimmt; Reject unterdrückt sie für die Woche. Macht künftiges Versagen sichtbar. | |
| b) Rein balance-getrieben | Nach Reject sofort wieder da, bis das Konto stimmt. | |
| Freitext | — | ✓ |

**User's choice:** Freitext — „Solange bis HR auf annehmen oder ablehnen klickt.
Dann hat HR entschieden und es gibt keine Frage mehr in der Woche."

**Notes:** Das ist das heutige Verhalten (D-55-07) und **verwirft ROADMAP-Scope-Punkt
2** („D-55-07 entkoppeln, sonst bleibt jedes künftige Versagen unsichtbar"). Claude
hat die Konsequenz einmal benannt: der Banner ist damit kein Wächter mehr gegen ein
erneutes Versagen des Übertrags — diese Rolle übernehmen die invertierten
Neutralitäts-Tests, die fehlschlagen, sobald sich die Balance nach einer Umbuchung
nicht bewegt. Das deckt das Anliegen des ROADMAP-Punkts strukturell ab (Test statt
Sichtprüfung) und ist der bessere Wächter. Keine Nachverhandlung.
→ CONTEXT.md D-57-04

---

## Produktionsstand des Auto-Cron

| Option | Description | Selected |
|--------|-------------|----------|
| a) Nie aktiviert | Kein Cron-Lauf, kein Backfill — keine ungedeckelten Auto-Rows im Bestand. | ✓ |
| b) Läuft / lief schon | Bestandspaare können zu groß sein und würden Konten ins Plus heben. | |

**User's choice:** a) — „Ich habe nichts aktiviert."
**Notes:** Faktfrage, im Repo nicht beantwortbar (`voluntary_rebooking_auto_active_from`
ist ein DB-Wert mit Default `None`). Damit entfällt jede Bestandsdaten-Korrektur aus
dem Phasen-Scope. Der Planner soll den Stand trotzdem gegen die Ziel-DB verifizieren
statt ihn anzunehmen. → CONTEXT.md D-57-05

---

## Claude's Discretion

Vom User ausdrücklich delegiert („Den Rest — welche Filter genau rausfliegen, wie die
Requirements neu heißen, wie die Tests umgebaut werden, was mit der Snapshot-Version
passiert — entscheide ich selbst"):

- **D-57-06 Filter-Neufassung.** Ursprüngliche Annahme („Filter überall ersatzlos
  streichen") war **falsch** und wurde durch eigene Code-Verifikation korrigiert:
  `VolunteerWork.availability() == Available` (`extra_hours.rs:122`), und
  `assemble_weeks` summiert `extra_working_hours` über `availability()`
  (`reporting.rs:876`) statt über `as_report_type()`. Dort würde pauschales Entfernen
  das Paar netto auf 0 bringen. Ergebnis: drei Sites ersatzlos streichen, eine Site
  gezielt behandeln.
- **RISK-01** als Nebenbefund dokumentiert: Alt-Inkonsistenz zwischen
  Wochenübersicht und Jahresreport bezüglich VolunteerWork-in-Balance. Vorbestehend,
  ausdrücklich nicht Scope.
- **D-57-07 Balance-Quelle im Cron** — Ende der verarbeiteten Woche, nicht
  Laufzeitpunkt (sonst Mehrfachvergabe desselben Spielraums beim Backfill).
- **D-57-08 Test-Zuschnitt** — invertieren statt löschen; der Soll-Test bleibt
  inhaltlich ein Neutralitäts-Test.
- **D-57-09 REQ-IDs** — VOL-ACCT-03 (Neufassung) + VOL-ACCT-04 + REB-AUTO-06.
- **D-57-10/11/12 Snapshot** — Bump 12→13, Straddling-Fixture invertieren,
  Bestands-Snapshots unangetastet.

## Deferred Ideas

- Alt-Inkonsistenz `assemble_weeks` vs. `hours_per_week` (RISK-01) — eigener Fix,
  eigene Phase.
- Undo/Rollback von Rebooking-Batches — REB-UNDO-01, v2.7+.
- Repair-Werkzeug für zu große Auto-Cron-Paare — durch D-57-05 gegenstandslos.

## Reviewed Todos (not folded)

`todo.match-phase 57` lieferte drei Treffer, alle Stoppwort-Fehlalarme
(„bei, der, wird, ein, eine"), keiner mit F14-Bezug — nicht vorgelegt:
`2026-05-05-warnung-eintrag-ausserhalb-vertragszeiten.md`,
`2026-05-08-cutover-ui-admin-feature.md`,
`2026-07-01-urlaub-beantragen-mit-genehmigung.md`.
