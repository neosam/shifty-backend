---
phase: 57-f14-rebooking-wirksamkeit-requirements-korrektur-balance-aus
plan: 06
subsystem: docs
tags: [docs-freshness-gate, rebooking, reporting, billing-period, bilingual]
status: complete
requires:
  - 57-02-SUMMARY.md (Filter-Fix — die vier Aggregations-Sites, D-57-06)
  - 57-03-SUMMARY.md (Balance-Deckel — week_excess-Formel, D-57-07)
  - 57-04-SUMMARY.md (Referenz-Szenario — die Zahlen 8/12/−2 → 10/10/0)
  - 57-05-SUMMARY.md (Snapshot-Bump — inkl. Nachtrag: VIER value_types)
  - CLAUDE.md §Docs-Freshness-Gate (Trigger-Tabelle, Zweisprachigkeit)
provides:
  - "F14 §3 im Stand nach D-57-06: Kategorie-Semantik trägt die Fachlogik, source ist Provenienz"
  - "F14 §9.1 mit Balance-Deckel, Balance-Quelle und der Abgrenzung zum ungedeckelten HR-Vorschlag"
  - "F14 §9.3 Preview == Apply (I-7), F14 §9.4 Snapshot-Entscheidung gekippt"
  - "F14 §8.4 Banner-Klarstellung D-57-04 (Wächter sind die Tests, nicht das UI)"
  - "F07 §2.6/§3 kategoriebasierte Aggregation plus Site-1-Ausnahme"
  - "docs/domain/time-accounting: Übertrag in Kontobegriffen, beide Untergrenzen"
  - "F08 §7.4 v13-Zeile + als historisch markierter Non-Bump-Absatz, §7.5 Validator-Randfall v12↔v13"
  - "docs/domain/billing-period: aktuelle Version 13 + Phase 57 als Musterfall für Input-Set-Change"
affects:
  - Milestone-Close v2.7 (Docs-Freshness-Gate für Phase 57 geschlossen)
tech-stack:
  added: []
  patterns:
    - "Widerrufene Entscheidungen werden als HISTORISCH markiert und mit Umschlagpunkt versehen statt gelöscht — der Leser findet Begründung UND Widerruf"
    - "Fachdoku (docs/domain) in Kontobegriffen, Feature-Doku (docs/features) in Code-Begriffen — dieselbe Aussage, zwei Abstraktionsebenen"
key-files:
  created: []
  modified:
    - docs/features/F14-rebooking.md
    - docs/features/F14-rebooking_de.md
    - docs/features/F07-reporting-balance.md
    - docs/features/F07-reporting-balance_de.md
    - docs/features/F08-billing-period.md
    - docs/features/F08-billing-period_de.md
    - docs/domain/time-accounting.md
    - docs/domain/time-accounting_de.md
    - docs/domain/billing-period.md
    - docs/domain/billing-period_de.md
decisions:
  - "Der v13-Eintrag nennt VIER betroffene value_types (Overall, Balance, ExtraWork, Volunteer) statt der drei aus dem Plantext — Übernahme des Messbefunds aus 57-05 (Rule 3)"
  - "Der Non-Bump-Absatz in F08 §7.4 wird nicht gelöscht, sondern mit einem vorangestellten Blockquote-Widerruf und einer per-Phase-Bewertung ('Weiterhin korrekt' / 'Seit Phase 57 hinfällig') versehen"
  - "F08 §7.5 hat den v12↔v13-Randfall bekommen (Plan ließ die Entscheidung offen) — die Validator-Anweisung steht sonst nur im Code-Kommentar"
  - "docs/domain/time-accounting §'Wo die Rechnung passiert' blieb unverändert — der Abschnitt erwähnt den Marker-Filter nicht"
  - "Über den Plan hinaus mit-korrigiert: F14-Kopfzeile, F14 §2-Slice-Tabelle, F14 §5 VoluntaryStats-Aggregationsmodell, F14 §6 ist_total, F14 §7, F14-Fazit, F07 §3 Reader-Impact, F08-Statuszeile/Konstante/§7.7/Fussnote — alle trugen dieselbe überholte Aussage"
metrics:
  duration: ~35min
  completed: 2026-07-31
---

# Phase 57 Plan 06: Doku auf den Stand nach Phase 57 Summary

Zehn Doku-Dateien, fünf Einheiten in je zwei Sprachen: die Doku
beschreibt nicht mehr den Marker-Filter, sondern die Kategorie-Semantik,
den Balance-Deckel mit seinen zwei Untergrenzen und den vollzogenen
Snapshot-Bump — und die alte, fehlerhafte Begründung findet man nur noch
ausdrücklich als Historie gekennzeichnet.

## Was gebaut wurde

### Task 1 — F14-rebooking, beide Sprachen (Commit `74949ea2`)

**§3 Marker-Filter-Regel.** Der Absatz, der den unbedingten
`source == Manual`-Filter mit „Doppelzählung" begründete, ist durch die
Beschreibung der Kategorie-Semantik ersetzt: `as_report_type()`
entscheidet über die Balance-Relevanz (`ExtraWork` → `WorkingHours`,
`VolunteerWork` → `Documented`), `availability()` über die Anwesenheit.
Vier neue Absätze:

| Absatz | Aussage |
|---|---|
| Reader-Regel | Kategorie trägt die Fachlogik; Paar hebt Balance +h, senkt Ehrenamt-Ist −h |
| Die eine Ausnahme | `assemble_weeks` filtert konjunktiv `VolunteerWork` UND `Rebooking`, weil die Summe dort über `availability()` läuft; in `manual_volunteer_hours` bleibt dieselbe Zeile bewusst drin |
| RISK-01 | gewöhnliche VolunteerWork-Stunden zählen in der Wochensicht in die Balance, im Jahresreport nicht — Phase 57 hat das bewusst nicht angetastet |
| Historien-Hinweis | bis Phase 56 unbedingter Filter an vier Sites, aus der ursprünglichen VOL-ACCT-03-Fassung; diese Forderung war selbst der Fehler |

Der Absatz „Balance-Neutralitäts-Garantie" heißt jetzt „Was neutral
bleibt" und benennt nur noch das VOL-ACCT-01-Soll, mit
`test_neutrality_f2_soll` als Beleg und den fünf Tests für die
Delta-Zusage.

**§9.1 Engine.** Der Deckel steht in Kontobegriffen („die Automatik
gleicht das bezahlte Stundenkonto aus, aber nur so weit, wie der
Mitarbeiter seine Freiwilligen-Zusage noch erfüllt"), gefolgt von der
Formel `min( max(ist − committed_voluntary, 0), max(−balance, 0) )`, der
Balance-Quelle (year-to-date ab `first_day_in_year`, inkl.
Vorjahres-Carryover, mit der Backfill-Begründung), der Abgrenzung zum
ungedeckelten HR-Vorschlag und dem Referenz-Szenario inklusive Gegenprobe
(23 h bei 20 h zugesagt → trotzdem nur 2).

**§9.3 Backfill.** Neuer Bullet „Preview == Apply (Invariante I-7)": der
Preview rechnet den Überschuss eigenständig und läuft nicht über
`run_for_week`, braucht also einen eigenen Balance-Read; ein
`0.0`-Platzhalter würde den Preview still lügen lassen. Die zwei sich
widersprechenden Tests sind namentlich genannt.

**§9.4 Snapshot.** Vom „Nein"-Zweig auf den vollzogenen Bump
umgeschrieben: die Neutralität war die Voraussetzung der Phase-56-
Entscheidung, sie war selbst der Fehler, die vier betroffenen
`value_type`s stehen drin, die nicht betroffenen ebenfalls, D-57-12
(keine Migration) und die umgedrehte Beweisrichtung der Fixture.

**§8.4 Banner.** Klarstellender Absatz, ohne das Verhalten zu ändern: der
Alert erlischt weiterhin bei Approve, Reject und belegtem UNIQUE-Slot;
das User-Zitat aus D-57-04 ist mit aufgenommen; die Wächter-Rolle liegt
ausdrücklich bei den Property-Tests, nicht beim UI. Eine Entkopplung wird
nirgends beschrieben oder angekündigt.

### Task 2 — F07 + time-accounting, beide Sprachen (Commit `2af49b2e`)

**F07 §2.6.** Der Bullet „Rebooking-Neutralitäts-Filter" heißt jetzt
„Rebooking-Aggregation — Kategorie-Semantik statt Marker" und zerfällt in
fünf Unterabsätze: Kategorie-Semantik, die eine `assemble_weeks`-Site
inkl. RISK-01, „Was noch neutral ist" (nur das Soll) mit der
Delta-Zusage und den fünf Belegtests, die Snapshot-Konsequenz 12 → 13,
und der Historien-Hinweis auf den alten Filter.

**F07 §3 Datenmodell.** Der Reader-Impact-Absatz zur `source`-Spalte
behauptet nicht mehr, die vier Aggregations-Eingänge filterten nach dem
Marker; er beschreibt `source` als Herkunftskennzeichnung, die für die
Aggregation nur an der `assemble_weeks`-Summe ausgewertet wird.

**docs/domain/time-accounting.** Neuer Unterabschnitt „Übertrag
freiwilliger Stunden auf das bezahlte Konto (Rebooking)" im Abschnitt
„Was zählt als `worked`?" — bewusst in Kontobegriffen statt in
Filterbegriffen: Gesamtstundenzahl bleibt gleich, nur das Konto ändert
sich; `committed_voluntary` als Sollgröße wird nie berührt; beide
Untergrenzen mit ihren zwei Akteuren; das Referenz-Szenario als Beispiel;
Verweis auf F14 für die Mechanik und auf F07 §2.6 für RISK-01.

Der Unterabschnitt zur Ist/Soll-Symmetrie enthält keine widersprechende
Aussage und blieb unverändert. Der Abschnitt „Wo die Rechnung passiert"
erwähnt den Marker-Filter nicht und blieb ebenfalls unverändert (der Plan
hatte beide Fälle vorgesehen).

### Task 3 — F08 + docs/domain/billing-period, beide Sprachen (Commit `43dced82`)

**F08 §7.4 v13-Zeile.** Im Spaltenformat der v12-Zeile, mit dem Trigger
(Vorfilter entfernt, D-57-06) und den betroffenen `value_type`s. Die
Formulierung ist aus dem in Plan 05 geschriebenen v13-Doc-Kommentar
abgeleitet, damit Code und Doku dieselbe Begründung tragen.

**F08 §7.4 Non-Bump-Widerruf.** Der Absatz ist nicht gelöscht (T-57-18).
Ihm ist ein Blockquote vorangestellt, das die Bestätigung als historisch
ausweist, die Voraussetzung benennt (Marker-Filter hält Rebooking aus der
Snapshot-Kette), festhält, dass die Voraussetzung selbst der Fehler war,
und den Umschlagpunkt Phase 57 nennt. Die drei Phasen-Bullets tragen
jeweils eine Bewertung: Phase 54 „Weiterhin korrekt", Phase 55 „Seit
Phase 57 nicht mehr korrekt", Phase 56 „Seit Phase 57 hinfällig". Darunter
ein neuer Absatz zum vollzogenen Bump mit der `Overall`-Herleitung, der
CLAUDE.md-Klausel, den zwei getrennten Beweislasten der Fixture und
D-57-12.

**F08 §7.5.** Der v12↔v13-Randfall im Format des v11↔v12-Beispiels: ein
Validator MUSS `Overall`, `Balance`, `ExtraWork` und `Volunteer`
ausnehmen, alle übrigen `value_type`s dürfen re-validiert werden — mit
der Begründung, warum (`rebook()` schreibt nur zwei Kategorien, das Soll
stammt aus dem Vertrag).

**docs/domain/billing-period.** Aktuelle Version 12 → 13 (je eine Stelle
pro Sprache). Der Abschnitt „Wann eine Bump-Pflicht besteht" führte den
Rebooking-Fall nicht als Nicht-Bump-Fall; ergänzt wurde ein Musterfall
für Trigger 4, der den didaktischen Punkt macht: keine Formel geändert,
kein `value_type` hinzugefügt — nur die Menge der gelesenen Zeilen ist
größer geworden, und das reicht.

## Verifikation

### Hard Gates (Phasen-Abschluss)

| Gate | Ergebnis |
|---|---|
| `cargo test --workspace` | ✅ 0 failed (832 in `service_impl`, 66 Integration) |
| `cargo clippy --workspace -- -D warnings` | ✅ exit 0 |

Beides wurde gezogen, obwohl dieser Plan **ausschließlich Markdown unter
`docs/`** geändert hat — kein Rust-Code wurde angefasst, die Gates sind
hier reine Phasen-Abschluss-Bestätigung, nicht Wirkungsnachweis.

### Akzeptanzkriterien

| Kriterium | Erwartet | Gemessen (EN / DE) |
|---|---|---|
| `grep -c 'REB-AUTO-06' F14` | ≥ 1 | 4 / 4 |
| `grep -c 'VOL-ACCT-04' F14` | ≥ 1 | 4 / 4 |
| `grep -c 'D-57-04' F14` | ≥ 1 | 1 / 1 |
| `grep -c 'D-57-10' F14` | ≥ 1 | 5 / 5 |
| `grep -c 'rebooking_reference_scenario' F14` | ≥ 1 | 3 / 3 |
| Balance-Quelle benannt (`year-to-date` / `first_day_in_year`) | ≥ 1 | ✅ beide |
| `grep -c 'BackfillPreviewItem' F14` | ≥ 1 | 1 / 1 |
| Sektionszahl F14 (`^## \|^### `) | unverändert 22 | 22 / 22 |
| `grep -c 'VOL-ACCT-04' F07` | ≥ 1 | 2 / 2 |
| `grep -c 'as_report_type' F07` | ≥ 1 | 2 / 2 |
| `grep -c 'assemble_weeks' F07` | ≥ 1 | 5 / 5 |
| `grep -c 'rebooking_reference_scenario' F07` | ≥ 1 | 1 / 1 |
| Snapshot-Version in F07 auf 13 korrigiert | ja | ✅ „12 → 13" |
| `grep -c 'committed_voluntary' time-accounting` | ≥ 1 | 3 / 3 |
| Beide Untergrenzen in time-accounting benannt | ja | ✅ zwei Bullets |
| `grep -c '\| \*\*v13\*\*' F08` | == 1 | 1 / 1 |
| v13-Zeile nennt Balance/ExtraWork/Volunteer | ja | ✅ (plus Overall) |
| `grep -c 'D-57-10' F08` | ≥ 1 | 4 / 4 |
| `grep -c 'test_straddling_snapshot_rebooking_shifts_values' F08` | ≥ 1 | 1 / 1 |
| `grep -c 'currently \*\*13\*\*'` / `'aktuell \*\*13\*\*'` | == 1 | 1 / 1 |
| `grep -c 'currently \*\*12\*\*'` / `'aktuell \*\*12\*\*'` | == 0 | 0 / 0 |
| Non-Bump-Absatz als historisch markiert + Phase 57 genannt | ja | ✅ Blockquote-Widerruf |
| `git status --porcelain docs/` vor den Commits | 10 Dateien | 10 |

### Prohibitions — Status

| Prohibition | Status | Beleg |
|---|---|---|
| Keine einseitige Sprachpflege | eingehalten | Alle drei Commits enthalten je Einheit beide Dateien; die Sektionsnummern stimmen zwischen den Sprachvarianten überein (F14 22/22 Sektionen) |
| Keine Entkopplung des HR-Alert-Banners beschreiben oder ankündigen (D-57-04) | eingehalten | F14 §8.3/§8.4 beschreiben unverändert dieselben Erlöschungsgründe; der ergänzte Absatz sagt ausdrücklich, dass die Entkopplung verworfen wurde und die Regressions-Absicherung in den Tests liegt |
| HR-Vorschlagsformel nicht als gedeckelt beschreiben (I-5, D-57-01) | eingehalten | F14 §9.1 und F07 §2.6 nennen beide Untergrenzen getrennt; `min(\|balance\|, ist)` steht in F14 §8.3 unverändert zitiert und wird in §9.1 ausdrücklich als „deliberately not capped" abgegrenzt |
| Keine Stelle behauptet mehr Balance- oder Ist-Neutralität | eingehalten | `grep -i neutral` über alle zehn Dateien: jede Fundstelle bezieht sich entweder auf das VOL-ACCT-01-Soll, auf Testnamen (`test_neutrality_*`, `rebooking_neutrality_*`) oder auf einen als historisch markierten Absatz |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking/Korrektheit] Vier betroffene `value_type`s statt drei**

- **Gefunden bei:** Task 3, Abgleich mit `57-05-SUMMARY.md` (vom
  Orchestrator als `upstream_correction` mitgegeben)
- **Problem:** 57-06-PLAN.md nennt an drei Stellen (`must_haves`
  implizit, Task-3-`action`, Akzeptanzkriterium) `Balance`, `ExtraWork`
  und `Volunteer` als die betroffenen `value_type`s. Gemessen wurde in
  Plan 05 ein vierter: `Overall` verschiebt sich mit, weil
  `report_delta.overall_hours = shiftplan_hours_by_week +
  overall_extra_work_hours` das `+h ExtraWork`-Bein enthält. Der in Plan
  05 geschriebene v13-Doc-Kommentar nennt bereits vier — hätte die Doku
  nur drei genannt, wären Code-Kommentar und Doku auseinandergelaufen,
  und ein Validator hätte für v12-Snapshots `Overall` fälschlich
  revalidiert (stiller Falsch-Positiv-Generator).
- **Fix:** Alle Aufzählungen der betroffenen `value_type`s nennen vier —
  F08 §7.4 (v13-Zeile und Bump-Absatz), F08 §7.5 (Validator-Anweisung),
  F14 §7 und §9.4, jeweils in beiden Sprachen. Die `Overall`-Herleitung
  ist an drei Stellen mitgegeben, damit sie nicht wie ein Tippfehler
  aussieht.
- **Dateien:** `docs/features/F08-billing-period{,_de}.md`,
  `docs/features/F14-rebooking{,_de}.md`
- **Commits:** `74949ea2`, `43dced82`

**2. [Rule 2 — Doku-Korrektheit] Neun weitere Fundstellen derselben überholten Aussage**

- **Gefunden bei:** dem im Plan geforderten Gegenlesen am Ende von Task 1
  und Task 3
- **Problem:** Der Plan listet die zu ändernden Sektionen namentlich auf.
  Dieselbe überholte Aussage („Filter nach `source == Manual`",
  „balance-neutral", „bleibt bei 12") stand aber an neun weiteren, im
  Plan nicht genannten Stellen. Sie stehen zu lassen wäre exakt der
  Mechanismus, gegen den dieser Plan geschrieben ist.
- **Fix:**

  | Stelle | Alt | Neu |
  |---|---|---|
  | F14 Kopf, „Zuständige Crates" | „der `source == Manual`-Neutralitäts-Filter" | „die kategoriebasierte Aggregation mit ihrer einen Ausnahme" |
  | F14 §2, F4-Slice-Zeile | „kein Snapshot-Schema-Bump — bleibt bei 12" | „auf 12 → 13 gebumpt (D-57-10)" + Deckel-Hinweis |
  | F14 §5, VoluntaryStats-Aggregationsmodell | „der Filter greift ab Phase 55 zentral im ReportingService" | „die Rebooking-Wirkung fließt automatisch mit ein, dieser Service hält keinen eigenen Filter" |
  | F14 §6, `ist_total` | „absolute Manual-Volunteer-Summe" | „absolute Ehrenamt-Summe" |
  | F14 §7, F04/F07/F08-Bullets | Marker-Filter + „bleibt bei 12" | Provenienz + Kategorie-Semantik + Bump |
  | F14 §9-Intro | „dieselben balance-neutralen Rebooking-Paare" | „dieselben Rebooking-Paare", plus Deckel-Hinweis |
  | F14 Fazit | „ohne Snapshot-Schema-Bump — bleibt durchgehend 12" | Phase-57-Absatz: Übertrag wirksam, Deckel, Schema 13 |
  | F07 §3, Reader-Impact der `source`-Spalte | „die vier Read-Aggregate filtern `source == Manual`" | Provenienz + die eine `assemble_weeks`-Auswertung |
  | F08 Statuszeile, Konstanten-Block, §7.7, Fußnote | `12` | `13` (plus Hinweis auf alle fünf gebrochenen Locking-Gates) |

- **Dateien:** alle sechs Feature-Doku-Dateien
- **Commits:** `74949ea2`, `2af49b2e`, `43dced82`

**3. [Rule 2 — Doku-Korrektheit] F08 §7.5 v12↔v13-Randfall ergänzt**

- **Gefunden bei:** Task 3
- **Problem:** Der Plan ließ ausdrücklich offen, ob ein analoger
  Randfall-Absatz sinnvoll ist („Wenn nicht, unverändert lassen und im
  SUMMARY festhalten"). Ohne ihn stünde die Validator-Anweisung
  ausschließlich im Code-Doc-Kommentar von `billing_period_report.rs` —
  §7.5 ist aber der Ort, an dem ein Validator-Autor sie sucht, und die
  v11↔v12-Vorlage steht direkt darüber.
- **Fix:** Absatz im Format des bestehenden Beispiels ergänzt, mit der
  Ausnahme-Liste (vier `value_type`s) und der Begründung, warum die
  übrigen re-validiert werden dürfen.
- **Dateien:** `docs/features/F08-billing-period{,_de}.md`
- **Commit:** `43dced82`

Keine weiteren Abweichungen. Die drei Tasks folgen ansonsten exakt dem
Plan-Text.

## Auth-Gates

Keine.

## Known Stubs

Keine. Alle im Plan geforderten Sektionen sind inhaltlich gefüllt; es
bleibt kein „[Zu prüfen]"-Marker offen, der durch diesen Plan entstanden
wäre. Die zwei vorbestehenden `[Zu prüfen]`-Marker in F08 (§7.5
Validator-Fundort, §7.6 DTO-Feld `is_snapshot`) wurden bewusst nicht
angefasst — sie sind älter als Phase 57 und nicht Scope.

## Requirements-Status

`requirements mark-complete VOL-ACCT-03 VOL-ACCT-04 REB-AUTO-06`: alle
drei stehen seit Plan 01 (Requirements-Neufassung) auf `[x]`; inhaltlich
gemessen wurden sie in den Plänen 02 (VOL-ACCT-03) und 04 (VOL-ACCT-04,
REB-AUTO-06). Dieser Plan schließt kein neues Requirement, sondern das
Docs-Freshness-Gate für die Phase.

## Docs-Freshness-Gate

**Dieser Plan IST das Gate.** Er schließt es für Phase 57 vollständig:

| Trigger-Datei (Phase 57 geändert) | Geforderte Doku | Status |
|---|---|---|
| `service_impl/src/reporting.rs` (Plan 02) | F07 + time-accounting, beide Sprachen | ✅ Task 2 |
| `service_impl/src/billing_period_report.rs`, insb. `CURRENT_SNAPSHOT_SCHEMA_VERSION` (Plan 05) | F08 + billing-period, beide Sprachen | ✅ Task 3 |
| F14-Service-Fläche / `service/**`-Semantik | F14, beide Sprachen | ✅ Task 1 |

Nicht getriggert und daher unverändert: `docs/architecture/02-service-tiers.md`
und der Runtime-Service-Graph (`shifty_bin/src/main.rs` ist in der ganzen
Phase unberührt — belegt in 57-03-SUMMARY),
`docs/architecture/03-data-model.md` und das ER-Diagramm (`migrations/`
leer, D-57-12), `docs/architecture/04-auth.md` und F12
(`service_impl/src/permission.rs` unberührt).

## Threat Flags

Keine neue Angriffsfläche — reine Markdown-Änderung ohne Laufzeit-Pfad.
Register-Status:

- **T-57-18** (stehen gelassene Non-Bump-Bestätigung) — mitigiert: der
  Absatz ist nicht gelöscht, sondern trägt einen vorangestellten
  Blockquote-Widerruf mit Umschlagpunkt Phase 57 und je Phasen-Bullet
  eine Gültigkeitsbewertung. Ein Leser findet beides: die alte Begründung
  und ihren Widerruf.
- **T-57-19** (einseitige Sprachpflege) — mitigiert: jeder der drei
  Commits enthält beide Sprachvarianten jeder berührten Einheit; die
  Sektionsstruktur ist zwischen den Varianten identisch geblieben.
- **T-57-20** (Information Disclosure) — akzeptiert wie geplant: keine
  Secrets, keine Personendaten.
- **T-57-21** (HR-Formel fälschlich als gedeckelt beschrieben) —
  mitigiert: beide Untergrenzen sind in F14 §9.1, F07 §2.6 und
  time-accounting getrennt benannt; `min(|balance|, ist)` bleibt
  unverändert zitiert.
- **T-57-SC** — keine Paket-Installation, `Cargo.toml` unverändert.

## Wave-Disziplin

Innerhalb der deklarierten `files_modified` geblieben: exakt die zehn
Doku-Dateien, kein Rust-Code, keine Planning-Artefakte anderer Pläne.
`git status --porcelain` war nach jedem der drei Commits leer.

## Commits

| Commit | Inhalt |
|---|---|
| `74949ea2` | `docs(57-06)`: F14 auf Stand nach Phase 57 — Marker-Regel, Balance-Deckel, Snapshot-Bump |
| `2af49b2e` | `docs(57-06)`: F07 und time-accounting auf Kategorie-Semantik umgestellt |
| `43dced82` | `docs(57-06)`: F08 v13-Zeile und Widerruf der Non-Bump-Bestätigung |

## Self-Check: PASSED

- `docs/features/F14-rebooking.md` — FOUND
- `docs/features/F14-rebooking_de.md` — FOUND
- `docs/features/F07-reporting-balance.md` — FOUND
- `docs/features/F07-reporting-balance_de.md` — FOUND
- `docs/features/F08-billing-period.md` — FOUND
- `docs/features/F08-billing-period_de.md` — FOUND
- `docs/domain/time-accounting.md` — FOUND
- `docs/domain/time-accounting_de.md` — FOUND
- `docs/domain/billing-period.md` — FOUND
- `docs/domain/billing-period_de.md` — FOUND
- Commit `74949ea2` — FOUND
- Commit `2af49b2e` — FOUND
- Commit `43dced82` — FOUND
- `cargo test --workspace` / `cargo clippy --workspace -- -D warnings` — grün
