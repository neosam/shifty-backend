# Phase 29: Urlaubs-Balken-Konsistenz (FE) - Context

**Gathered:** 2026-06-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Der Pro-Person-Urlaubsbalken auf der Abwesenheiten-Seite (`PersonVacationCard` in
`shifty-dioxus/src/page/absences.rs`) stimmt mit der Resturlaub-Zahl daneben überein.
Reiner Frontend-Compute-Fix in der Render-Funktion — **kein** Backend, **kein** neues
DTO-Feld, **kein** neuer Endpoint. Alle benötigten Felder (`used_days`, `planned_days`,
`remaining_days`, `entitled_days`, `carryover_days`) liegen bereits im Frontend-State
`VacationBalance` (`shifty-dioxus/src/state/vacation_balance.rs`).

**Out of scope (bewusst):** Zwei-Segment-Balken (genommen vs. geplant als getrennte
Abschnitte) — in REQUIREMENTS.md als Future deferred markiert. Backend-Formeln bleiben
unangetastet.

</domain>

<decisions>
## Implementation Decisions

### Balken-Formel (VAC-01, Kernbug)
- **D-29-01:** Balken-Füllstand wird von `used_days / total` auf
  **`(used_days + planned_days) / total`** umgestellt (`total = entitled_days +
  carryover_days`). Damit misst der Balken exakt dieselbe Größe wie die Resturlaub-Zahl
  daneben (`remaining = entitled + carryover − used − planned`) → beide stimmen überein.
  Im Kern ein Einzeiler (Zähler `used_days` → `used_days + planned_days` in
  `absences.rs:866-867`).

### Überzug-Visualisierung (`used + planned > total`)
- **D-29-02:** **Volle Breite + Warnfarbe** (User-Entscheidung im Discuss). Der Balken
  wird bei Überzug visuell auf **100% gekappt** (`clamp(0.0, 100.0)` bleibt) und in
  `warn`-Farbe (amber) dargestellt; zusammen mit der **negativen Resturlaub-Zahl**
  daneben signalisiert das den Überzug eindeutig. **Kein** physischer Überlauf über den
  Track hinaus (kein Entfernen von `overflow-hidden`) — vermeidet Layout-Risiko
  (Balken über Kartenkante) und das Design-System hat ohnehin nur die zwei Status-Tokens
  `good`/`warn` (kein drittes „danger/rot").
  - **SC2-Reconciliation:** Die ROADMAP-SC2-Formulierung „verlängert sich der Balken
    über 100% hinaus (kein Clamp)" wird per D-29-02 als **Farb-Signal-Interpretation**
    umgesetzt: Überzug = voller amber Balken + negative Zahl, nicht physisch >100%. Die
    ROADMAP-SC2 wird entsprechend nachgezogen (siehe ROADMAP-Update unten).

### Warnfarben-Schwelle
- **D-29-03:** Die bestehende **eine** Farb-Logik `low = remaining_days <= 3.0` bleibt
  und treibt weiterhin **sowohl** Balken- (`bg-warn`/`bg-good`) **als auch** Zahl-Farbe
  (`text-warn`/`text-good`). Da `remaining < 0 ⊂ remaining <= 3.0`, ist SC3 (Warnfarbe
  bei `remaining_days < 0`) durch die bestehende Logik bereits erfüllt — keine separate
  Schwelle, keine optische Unterscheidung zwischen „fast aufgebraucht" (≤3) und
  „überzogen" (<0), weil dafür ein drittes Farb-Token nötig wäre (Design-Change,
  out of scope). Static-class-Pattern (Pitfall 5, `absences.rs:857-858`) bleibt: Farbe
  per Match auf kleinen Bucket, nicht interpoliert.

### Claude's Discretion
- Genaue Code-Form des Einzeilers (z.B. lokale `let used_planned = ...`-Bindung vs.
  Inline-Summe) — solange das Static-class-Pattern erhalten bleibt und der Wert exakt
  `(used + planned) / total` ist.
- Test-Form (Unit-Test der reinen Prozent-/Farb-Berechnung — bevorzugt als pure Helfer-
  Funktion extrahieren, damit testbar ohne Dioxus-Render; siehe code_context).

### Folded Todos
- **„Urlaubs-Balken pro Person konsistent zu Resturlaub-Zahl machen"**
  (`.planning/todos/pending/2026-06-29-urlaubs-balken-pro-person-konsistent-zu-resturlaub-zahl.md`,
  `resolves_phase: 29`, score 0.9). Original-Problem: Balken zählt nur `used`, ignoriert
  `planned` → „−1 Resturlaub bei ⅓-Balken". Folded → vollständig durch D-29-01..03
  abgedeckt. Die im Todo genannte „Variante 2" (Zwei-Segment) bleibt deferred.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirement & Roadmap
- `.planning/REQUIREMENTS.md` §VAC-01 — Balken `(used+planned)/total`, kein 100%-Clamp,
  Warnfarbe bei `remaining<0`; Zwei-Segment als Future deferred.
- `.planning/ROADMAP.md` §"Phase 29" — Goal + Success Criteria 1–3.

### Code (Single Source of Truth)
- `shifty-dioxus/src/page/absences.rs:843-898` — `PersonVacationCard` (die zu ändernde
  Render-Funktion); aktuelle Formel Z.865-871, Farb-Flag Z.857-864.
- `shifty-dioxus/src/state/vacation_balance.rs:10-27` — `VacationBalance`-Struct mit allen
  benötigten Feldern (FE-State, kein Backend-Touch nötig).
- `service/src/vacation_balance.rs:43` — Backend-Referenzformel für `remaining_days`
  (nur lesen, um Konsistenz zu prüfen; NICHT ändern).
- `shifty-dioxus/tailwind.config.js` — Status-Tokens `good`/`warn` (+ `-soft`); Safelist
  für dynamisch gesetzte Klassen (Static-class-Pattern beachten).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `format_decimal(...)`: wird bereits für die Resturlaub-Zahl genutzt — unverändert.
- Static-class-Match (`bg-warn`/`bg-good`, `text-warn`/`text-good`): vorhandenes Muster
  in `PersonVacationCard`, wird beibehalten.

### Established Patterns
- **Pitfall 5 (Static Tailwind classes):** Farbklassen werden per Match auf einen kleinen
  Bucket gewählt, NIE interpoliert/zusammengebaut — sonst purged Tailwind sie weg. Der Fix
  muss dieses Muster bewahren.
- Prozentwert ist `u32` via `... as u32` mit `clamp(0.0, 100.0)` — bleibt geklammert
  (D-29-02).

### Integration Points
- Einzige Änderungsstelle: `PersonVacationCard` (`absences.rs:843-898`). Empfehlung für
  Testbarkeit: die reine Berechnung (Prozent + `low`-Flag aus den fünf Feldern) in eine
  **pure, testbare Helfer-Funktion** extrahieren (kein Dioxus-Render nötig), analog zur
  Testbarkeits-Konvention der bestehenden FE-Tests. Damit ist VAC-01 per `cargo test`
  abdeckbar (Browser-Datepicker-/Render-Tests sind laut Projekt-Memory unzuverlässig).

</code_context>

<specifics>
## Specific Ideas

Beispiel aus dem Todo, das als Test-Fixture dienen kann:
`entitled+carryover = 18, used = 6, planned = 13` → `remaining = −1`, Balken = `19/18`
→ gekappt auf 100% + amber. Erwartung nach Fix: Balken voll + warn, Zahl `−1` in warn.
Gegenbeispiel: `used = 6, planned = 0` → 33% grün (passt zu remaining 12).

</specifics>

<deferred>
## Deferred Ideas

- **Zwei-Segment-Urlaubsbalken (genommen vs. geplant)** — eigenes, rein additives
  Feature; in REQUIREMENTS.md §"Future Requirements" deferred. Nicht in Phase 29.

### Reviewed Todos (not folded)
- Off-theme Treffer aus `todo.match-phase 29` (gehören zu anderen Phasen/Backlog):
  Booking-Log-500er, Cutover-UI, Urlaub-für-Freiwillige-eintragen, Stale-Daten (P30),
  Nicht-Verfügbar-Markierung (P31), Impersonate (P32), Dependency-Update (999.1) u.a. —
  bewusst nicht in P29 gefolded.

</deferred>

---

*Phase: 29-Urlaubs-Balken-Konsistenz (FE)*
*Context gathered: 2026-06-29*
