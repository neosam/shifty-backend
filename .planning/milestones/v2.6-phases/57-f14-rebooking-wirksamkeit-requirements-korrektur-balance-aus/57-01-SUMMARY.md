---
phase: 57-f14-rebooking-wirksamkeit-requirements-korrektur-balance-aus
plan: 01
subsystem: planning-spec
tags: [requirements, spec-correction, rebooking, voluntary-hours, traceability]
status: complete
requires:
  - .planning/phases/57-.../57-CONTEXT.md (D-57-01..D-57-12)
  - .planning/phases/57-.../57-PATTERNS.md (REQ-Format-Vorbild REB-MANUAL-01)
provides:
  - VOL-ACCT-03 (Neufassung — nur noch Soll-Schutz)
  - VOL-ACCT-04 (Ausgleichswirkung Balance +h / Ehrenamt-Ist −h)
  - REB-AUTO-06 (Balance-Deckel im Auto-Cron inkl. Backfill-Preview)
  - Bestandsdaten-Befund D-57-05 / D-57-03 (gemessen, nicht angenommen)
affects:
  - Plan 02 (Filter-Fix — implementiert gegen VOL-ACCT-04)
  - Plan 03 (Balance-Deckel — implementiert gegen REB-AUTO-06)
  - Plan 04 (Ende-zu-Ende-Test gegen Referenz-Szenario D-57-02)
  - Plan 05 (Snapshot-Bump 12→13 — Spec-Rückendeckung via REB-AUTO-05-Nachtrag)
tech-stack:
  added: []
  patterns:
    - "REQ-Format REB-MANUAL-01: `- [x] **ID**: …`, Folgezeilen 6 Spaces eingerückt, Code-Identifier in Backticks"
    - "Fehlerspur als `> **Historie:**`-Blockquote erhalten statt gelöscht (greppbar über D-55-A1)"
key-files:
  created: []
  modified:
    - .planning/REQUIREMENTS.md
    - .planning/STATE.md
decisions:
  - "D-57-09 umgesetzt: VOL-ACCT-03 schützt nur noch VOL-ACCT-01-Soll; Balance und VOL-STAT-01-Ist sind ausdrücklich NICHT Teil des Requirements"
  - "Der ⚠️-Fehlervermerk wurde durch einen Historien-Blockquote ersetzt statt gelöscht — D-55-A1 und Phase 55 bleiben greppbar"
  - "REB-AUTO-05 erhält eine Nachtragsnotiz (D-57-10/D-57-11), aber KEINEN Owner-Wechsel: Traceability-Zeile bleibt Phase 56, Per-Phase-Coverage von Phase 56 bleibt 5"
  - "Traceability: Coverage 17/17 → 19/19, VOL-ACCT-03 von Phase 54 auf Phase 57 umgehängt, Phase 54 dadurch von 5 auf 4 Requirements korrigiert"
  - "Bestandsdaten-Pre-Flight: localdb.sqlite3 ist laut Nutzer die Ziel-DB; rebooking_batch ist leer → D-57-05 und D-57-03 nicht widerlegt, Wave 2 freigegeben"
metrics:
  duration: ~25min
  completed: 2026-07-31
---

# Phase 57 Plan 01: Requirements-Korrektur + Bestandsdaten-Pre-Flight Summary

Die Spec wurde vor dem Code korrigiert: VOL-ACCT-03 fordert nicht länger das
Gegenteil des Milestone-Goals, VOL-ACCT-04 und REB-AUTO-06 operationalisieren
Ausgleichswirkung und Balance-Deckel, und der Bestandsdaten-Stand aus D-57-05 /
D-57-03 ist gegen die Ziel-DB gemessen statt angenommen.

## Was gebaut wurde

### Task 1 — `.planning/REQUIREMENTS.md` (Commit `f169b176`)

**(a) VOL-ACCT-03 neu gefasst (D-57-09).** Das Requirement schützt jetzt nur noch
das **VOL-ACCT-01-Soll** gegen Doppelzählung — begründet damit, dass das Soll aus
`committed_voluntary` im Arbeitsvertrag stammt und überhaupt keine `extra_hours`
liest. Property-Test-Zusage: Rebooking über eine Woche → `VoluntaryStats::soll_total`
unverändert. Ein ausdrücklicher Zusatzsatz stellt klar, dass Balance und
VOL-STAT-01-Ist **nicht** Teil dieses Requirements sind; deren Veränderung ist ab
VOL-ACCT-04 gewollt. Die Formulierung „(balance-neutral)" und der Anspruch auf
unverändertes Ist sind ersatzlos entfallen.

**(b) Fehlervermerk → Historien-Hinweis.** Der ⚠️-Blockquote („FEHLERHAFT — nicht
darauf weiterbauen") ist durch einen `> **Historie:**`-Blockquote ersetzt. Er hält
fest: die Fassung bis Phase 56 forderte zusätzlich die Neutralität von Ist und
Balance; wörtlich umgesetzt in **Phase 55** (**D-55-A1**, `source == Manual`-Filter
an 4 Aggregations-Sites) machte das den Übertrag wirkungslos (empirisch `−53 → −53`);
gemeint war der Schutz der Freiwilligen-Statistik, nicht das Einfrieren der Balance.
Die Fehlerspur bleibt damit greppbar — sie wurde nicht gelöscht.

**(c) Zwei neue Requirements** im Format-Vorbild `REB-MANUAL-01`:

- **VOL-ACCT-04** (unter VOL-ACCT-03, §Voluntary-Stundenkonto): Eine Umbuchung über
  `h` Stunden wirkt auf der Leseseite tatsächlich — Balance um `h` höher,
  Ehrenamt-Ist um `h` niedriger. Gilt für alle drei Schreib-Trigger (F3/F4/F5) und
  rückwirkend ohne read-seitigen Stichtag-Guard (D-57-03). Akzeptanzbedingung ist
  das Referenz-Szenario D-57-02 mit den konkreten Zahlen im Text: Vertrag 5 h
  bezahlt + 5 h freiwillig pro Woche, Woche 1 = 3 h, Woche 2 = 17 h; vorher 8 h /
  12 h bei Balance −2, nachher 10 h / 10 h bei Balance 0.

- **REB-AUTO-06** (letztes REQ in §Automatische Wochen-Umbuchung): Der Auto-Cron
  überträgt höchstens `min( max(ist − committed_voluntary, 0), max(−balance, 0) )`.
  Die Balance steigt durch die Automatik nie über 0; bei Balance ≥ 0 wird trotz
  Überschuss nichts gebucht. Balance-Quelle ist die kumulierte Jahres-Balance bis
  einschließlich der verarbeiteten Woche inklusive Vorjahres-Carryover (D-57-07).
  Der Deckel gilt für `run_for_week` **und** `BackfillPreviewItem.excess`. Die
  HR-Vorschlagsformel `proposed_hours = min(|balance|, ist)` ist ausdrücklich
  ausgenommen und bleibt **unverändert** (D-57-01). Gegenprobe im Text: Woche 2 mit
  20 h ergibt Überschuss 10, Übertrag bleibt 2, Balance landet auf 0.

**(d) Traceability + Kopfangaben.** Einleitungszeile von „17 v1-Requirements →
3 Phasen … Coverage: 17/17" auf „19 v1-Requirements → 4 Phasen … Coverage: 19/19".
VOL-ACCT-03 von Phase 54 auf **Phase 57** umgehängt; zwei neue Zeilen VOL-ACCT-04
und REB-AUTO-06 an Phase 57. Per-Phase-Coverage: Phase 54 von 5 auf 4 Requirements
korrigiert, neuer Block „Phase 57 (Rebooking-Wirksamkeit): 3 Requirements",
Schlusszeile auf 19/19. Zusätzlich `last_updated` auf `2026-07-31` und die
Snapshot-Schema-Kopfzeile darauf umgeschrieben, dass der Bump 12→13 in Phase 57
per D-57-10 entschieden ist.

**(e) Nachtragsnotiz an REB-AUTO-05 — kein Owner-Wechsel.** Direkt unter dem
REB-AUTO-05-Block steht ein Blockquote, der mit `> **Nachtrag (Phase 57, D-57-10):**`
beginnt. Er hält fest, dass der Phase-56-Ausgang „bleibt bei 12" sich als Beweislast
auf einen byte-identischen Straddling-Golden-Snapshot (D-01-BEWEISLAST) stützte und
diese Beweislage mit dem Filter-Fix entfallen ist — ein Rebooking-Paar ist nicht mehr
snapshot-neutral, es verschiebt **Balance**, ExtraWork und Volunteer um `±h`. Nach der
CLAUDE.md-Klausel „Change the input set the computation reads from" ist der Bump damit
Pflicht; Phase 57 revidiert den Ausgang auf 13 (D-57-10) und invertiert die
Golden-Fixture (**D-57-11**). Die Notiz sagt selbst ausdrücklich, dass REB-AUTO-05
inhaltlich und in der Traceability bei Phase 56 bleibt.

### Task 2 — Bestandsdaten-Pre-Flight (blockierender Human-Verify-Gate)

Der Gate wurde vom Nutzer beantwortet: **`localdb.sqlite3` IST die Ziel-DB für diese
Phase.** Damit sind die dort gemessenen Werte die maßgeblichen Query-Ergebnisse; ein
weiterer Prod-Befund steht nicht aus.

**Herkunft:** `sqlite3 localdb.sqlite3` im Repo-Root
(`/home/neosam/programming/projects/shifty-backend/localdb.sqlite3`), ausschließlich
lesende Statements — kein `UPDATE`, kein `DELETE`, kein `ALTER` (T-57-02).

**Wörtliche Ergebnisse:**

```
sqlite> SELECT kind, COUNT(*) FROM rebooking_batch WHERE deleted IS NULL GROUP BY kind;
(keine Zeilen — leeres Resultset)

sqlite> SELECT MIN(iso_year), MAX(iso_year), COUNT(*) FROM rebooking_batch WHERE deleted IS NULL;
NULL|NULL|0
```

**Bewertung gegen die Akzeptanzkriterien:**

| Kriterium | Befund | Ergebnis |
|---|---|---|
| Keine Zeile mit `auto_cron` oder `auto_cron_backfill` | leeres Resultset | ✅ erfüllt — D-57-05 nicht widerlegt |
| `MIN(iso_year) ≥ 2026` **oder die Tabelle ist leer** | `COUNT(*) = 0`, Tabelle leer | ✅ erfüllt über den ausdrücklich vorgesehenen Leer-Fall — D-57-03 nicht widerlegt |

**Kein Blocker-Befund.** Es existieren keine ungedeckelt geschriebenen
`auto_cron`/`auto_cron_backfill`-Paare und keine Rebooking-Rows mit `iso_year ≤ 2025`,
also auch kein stale `carryover_hours` (`shiftplan_edit.rs:384`), der nachgezogen
werden müsste. Die rückwirkende Leseänderung aus Wave 2 (D-57-03, kein
Stichtag-Guard) ist damit abgesichert — **Wave 2 ist freigegeben.**

## Verifikation

Alle Akzeptanzkriterien von Task 1 wurden per `grep` gegen die geänderte Datei
gemessen:

| Kriterium | Erwartet | Gemessen |
|---|---|---|
| `grep -c 'VOL-ACCT-04'` | ≥ 2 | 5 |
| `grep -c 'REB-AUTO-06'` | ≥ 2 | 4 |
| `grep -c 'balance-neutral'` | 0 | 0 |
| `grep -c 'D-55-A1'` | ≥ 1 | 1 |
| `grep -c 'FEHLERHAFT'` | 0 | 0 |
| `grep -c '19/19'` | ≥ 1 | 2 |
| `grep -c '17/17'` | 0 | 0 |
| `grep -cE '^\| VOL-ACCT-03 +\| F2 Konto +\| Phase 57 '` | 1 | 1 |
| `min(\|balance\|, ist)` im REB-AUTO-06-Block | vorhanden | 1 Treffer |
| `unverändert` im REB-AUTO-06-Block | vorhanden | vorhanden |
| `BackfillPreviewItem.excess` | vorhanden | 1 Treffer |
| `grep -c 'Nachtrag (Phase 57, D-57-10)'` | 1 | 1 |
| Nachtrag innerhalb `grep -A 12 'REB-AUTO-05'` | vorhanden | 1 Treffer |
| `D-57-11` + `Balance` in der Nachtragsnotiz | vorhanden | je 1 Treffer |
| `grep -cE '^\| REB-AUTO-05 +\| F4 Cron +\| Phase 56 '` | 1 | 1 |
| `grep -cE '^\| REB-AUTO-05 +\| F4 Cron +\| Phase 57 '` | 0 | 0 |
| `grep -c 'Phase 56 (F4 Wochen-Cron + Backfill): 5 Requirements'` | 1 | 1 |
| `grep -ci 'banner'` (Prohibition: unverändert) | 0 | 0 |

Die zusammengesetzte `<automated>`-Verifikationszeile des Plans lief mit **PASS**
durch. Die VOL-ACCT-04-Zahlen aus D-57-02 (`3`, `17`, `8`, `12`, `−2`, `10`, `0`)
stehen wörtlich im Requirement-Text.

**Kein Code-Gate nötig:** Dieser Plan ändert ausschließlich
`.planning/`-Dokumentation (`files_modified` = `.planning/REQUIREMENTS.md`). Es gibt
keinen Rust-Diff, also greifen `cargo build` / `cargo test` /
`cargo clippy --workspace -- -D warnings` hier nicht — sie sind ab Plan 02 (erster
Code-Eingriff) wieder Pflicht-Gate.

## Prohibitions — Status

| Prohibition | Status | Beleg |
|---|---|---|
| HR-Vorschlagsformel `proposed_hours = min(\|balance\|, ist)` (`reporting.rs:405`) nicht ändern | eingehalten | `reporting.rs` steht nicht in `files_modified` und wurde nicht angefasst; REB-AUTO-06 nennt den HR-Pfad ausdrücklich als ausgenommen und **unverändert** |
| Fehlervermerk zu VOL-ACCT-03 nicht ersatzlos löschen | eingehalten | `grep -c 'D-55-A1'` = 1; Historien-Blockquote nennt zusätzlich „Phase 55" und den empirischen Befund `−53 → −53` |
| Keine Entkopplung von D-55-07 als Requirement aufnehmen | eingehalten | `grep -ci 'banner'` = 0, unverändert gegenüber dem Ausgangsstand |
| REB-AUTO-05 nicht auf Phase 57 ummappen | eingehalten | Traceability-Zeile weiterhin Phase 56 (1 Treffer), Phase-57-Zeile 0 Treffer, Phase-56-Coverage weiterhin 5 Requirements, Gesamtsumme 19 |

## Deviations from Plan

Keine — der Plan wurde exakt wie geschrieben ausgeführt.

Eine **Faktenkorrektur** ohne Plan-Abweichung: STATE.md behauptete „lokal existiert
keine `localdb.sqlite3`". Die Datei existiert (434 KB, Repo-Root). Der Nutzer hat sie
im Checkpoint als Ziel-DB bestätigt; die Behauptung in STATE.md war also schlicht
falsch und ist mit diesem Plan überholt.

## Auth-Gates

Keine.

## Known Stubs

Keine — dieser Plan erzeugt keinen Code.

## Threat Flags

Keine neue Angriffsfläche. Die Pre-Flight-Queries waren wie in T-57-02 vorgesehen
ausschließlich lesend; das Ergebnis ist oben wörtlich protokolliert (T-57-03:
gemessen statt angenommen).

## Anschluss für Wave 2

- **Plan 02** implementiert den Filter-Fix gegen VOL-ACCT-04 und lässt
  `test_neutrality_f2_soll` bewusst **unverändert** — er ist jetzt der einzige
  verbliebene Neutralitäts-Test (nur noch Soll-Schutz).
- **Plan 03** implementiert den `week_excess`-Deckel gegen die REB-AUTO-06-Formel,
  inklusive `BackfillPreviewItem.excess`.
- **Plan 04** schreibt den Ende-zu-Ende-Test gegen das Referenz-Szenario D-57-02,
  das jetzt wörtlich in VOL-ACCT-04 steht.
- **Plan 05** hat mit der REB-AUTO-05-Nachtragsnotiz die Spec-Rückendeckung für den
  Bump `CURRENT_SNAPSHOT_SCHEMA_VERSION` 12 → 13.

## Commits

| Commit | Inhalt |
|---|---|
| `f169b176` | `docs(57-01)`: VOL-ACCT-03 neu fassen, VOL-ACCT-04 + REB-AUTO-06, Traceability 19/19 |
| `c1265e7e` | `docs(57-01)`: STATE.md — Plan-01-Decisions + Checkpoint-Position |
| `c77364e3` | `docs(57-01)`: SUMMARY — Requirements-Korrektur + Bestandsdaten-Pre-Flight |

## Self-Check: PASSED

- `.planning/REQUIREMENTS.md` — FOUND
- `.planning/phases/57-.../57-01-SUMMARY.md` — FOUND
- Commit `f169b176` — FOUND
- Commit `c1265e7e` — FOUND
- Commit `c77364e3` — FOUND
