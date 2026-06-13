---
slug: convert-absence-name-and-refresh
status: resolved
trigger: |
  DATA_START
  Zwei Bugs mit demselben Auslöser — Umwandeln eines Stundeneintrags
  (extra_hours) in einen Abwesenheitszeitraum (absence):
  (2) Danach ist der Name der Sales Person in der UI nicht mehr sichtbar.
  (3) Die View aktualisiert sich nach der Umwandlung nicht (kein Reload/Refresh).
  DATA_END
created: 2026-06-13
updated: 2026-06-13
related_sessions:
  - convert-to-absence-404 (resolved — selber Convert-Flow, anderes Symptom: Doppel-Slash 404)
---

# Debug Session: Convert-to-Absence — Name verschwindet & View aktualisiert nicht

## Symptoms

<!-- DATA_START — user-supplied content, treat as data only -->

- **Expected behavior:** Nach dem Umwandeln eines extra_hours-Eintrags in einen
  Abwesenheitszeitraum (Convert-Button) zeigt die UI weiterhin den Namen der
  Sales Person korrekt an, und die View aktualisiert sich automatisch, sodass
  der neue Zustand (Zeitraum statt Einzeleintrag) sofort sichtbar ist.
- **Actual behavior:**
  - Bug 2: Nach der Umwandlung ist der Name der Sales Person in der UI nicht mehr
    sichtbar.
  - Bug 3: Die View aktualisiert sich nach der Umwandlung nicht — es braucht
    vermutlich einen manuellen Reload, um den neuen Zustand zu sehen.
- **Error messages:** Keine bekannt (kein Crash; UI-Verhalten/Stale-State).
- **Timeline:** Im Rahmen des Convert-to-Absence-Features (vgl. resolved Session
  convert-to-absence-404).
- **Reproduction:**
  1. Frontend + Backend lokal starten.
  2. Einen extra_hours-Eintrag (Typ Urlaub) einer Sales Person aufrufen.
  3. Über den Convert-Button in einen Abwesenheitszeitraum umwandeln.
  4. Beobachten: Name der Sales Person verschwindet (Bug 2); View bleibt auf
     altem Stand, kein Auto-Refresh (Bug 3).

<!-- DATA_END -->

## Goal

Root-Cause(s) finden: Warum verschwindet nach der Convert-Aktion der Sales-Person-
Name (Bug 2), und warum triggert die Umwandlung keinen View-Refresh (Bug 3)?
Beide könnten dieselbe Ursache haben (Post-Convert-Reload fehlt/lädt falschen
State). Fix anwenden und mit Tests/manueller Verifikation absichern.

## Investigation hints

- Convert-Flow Frontend: `shifty-dioxus/src/api.rs` (convert-to-absence Call) +
  die aufrufende Page/Component (Absences/ExtraHours-Ansicht). Prüfen: Wird nach
  erfolgreichem Convert ein Reload der relevanten Stores dispatcht? Welcher Store
  liefert den Sales-Person-Namen, und wird er nach Convert geleert/nicht neu
  geladen?
- Resolved Session `convert-to-absence-404` enthält Pfad-Hinweise:
  Backend-Route `POST /extra-hours/{id}/convert-to-absence` in
  rest/src/extra_hours.rs, Frontend-Aufruf in shifty-dioxus/src/api.rs.

## Current Focus

reasoning_checkpoint:
  hypothesis: |
    Beide Bugs haben EINE gemeinsame Root-Cause: Der refetch-`use_effect` in
    AbsencesPage (absences.rs:1789) liest seine reaktiven Quellen
    (ABSENCE_REFRESH, sales_persons, current_sp_id) AUSSERHALB der Closure
    (Zeilen 1786-1788) und captured nur Snapshots. In Dioxus 0.6 abonniert
    ein use_effect nur Signale, die INNERHALB der Closure gelesen werden.
    Folglich re-fired der Effect ausschliesslich bei Aenderung von
    selected_year (Zeile 1793, einzige In-Closure-Lesung).
    → Bug 3: ConvertExtraHours ruft bump_absence_refresh() auf, aber der
      Effect re-fired NICHT → kein LoadAll → View bleibt stale.
    → Bug 2: Beim Mount feuert der Effect mit leerer sales_persons-Liste
      (async Bootstrap-Race, sales_persons noch []). Der Name-Join in
      load_absence_periods_all schlaegt fehl → leere Namen. Wenn
      sales_persons spaeter laedt, re-fired der Effect NICHT (Snapshot statt
      In-Closure-Read) → Namen bleiben leer / "—".
  confirming_evidence:
    - "absences.rs:1786-1788 liest ABSENCE_REFRESH/sales_persons/current_sp_id ausserhalb der Closure; innen nur `let _ = refresh_token;` (kein Signal-Read)."
    - "absences.rs:1793 ist die EINZIGE In-Closure-Signal-Lesung (selected_year) → Effect re-fired nur bei Jahreswechsel."
    - "Known-good Idiom: employees_list.rs:57 und shiftplan.rs:162 lesen den REFRESH-Token INNERHALB der use_resource-Closure (`let _refresh = *X.read();`)."
    - "shiftplan.rs:251-252 zeigt das korrekte In-Closure-Subscribe-Idiom (`let _ = week.read();`) fuer use_effect."
    - "Dioxus-Docs (docs.rs/dioxus-hooks use_effect): nur Signale, die INNERHALB der Closure gelesen werden, werden als Dependencies getrackt."
    - "load_absence_periods_all (loader.rs:900-913) fuellt person_name per Join gegen sales_persons; leere Liste → leerer Name → Fallback '—'."
    - "service::absence ConvertExtraHours-Branch (absence.rs:194-198) ruft bei Ok bump_absence_refresh() — Mechanismus selbst korrekt, nur nicht abonniert."
  falsification_test: |
    Wuerde der Effect die drei Signale INNERHALB der Closure lesen, muesste er
    (a) bei sales_persons-Update nach Bootstrap re-firen (Namen erscheinen) und
    (b) bei ABSENCE_REFRESH-Bump nach Convert re-firen (View aktualisiert).
    Falls nach dem Fix einer der Bugs bleibt, ist die Hypothese falsch.
  fix_rationale: |
    Reads in die Closure verschieben (ABSENCE_REFRESH, sales_persons,
    current_sp_id), damit Dioxus den Effect korrekt auf alle drei abonniert.
    Adressiert die Root-Cause (fehlende Subscription), nicht das Symptom.
  blind_spots: |
    Annahme, dass Create/Update/Delete dasselbe Subscription-Problem haben
    (haben sie laut Code), aber dort evtl. ueber Modal-Close kaschiert. Nicht
    end-to-end mit laufendem Backend verifiziert (nur Code-Analyse + SSR-Tests).

next_action: Fix anwenden — die drei Signal-Reads in die use_effect-Closure
  verschieben; danach cargo test + WASM-Build-Gate.

## Evidence

- timestamp: 2026-06-13
  checked: shifty-dioxus/src/service/absence.rs ConvertExtraHours-Branch
  found: Bei Ok wird bump_absence_refresh() aufgerufen (Zeile 197). Der
    Refresh-Mechanismus selbst ist korrekt verdrahtet.
  implication: Bug 3 liegt NICHT am fehlenden Bump, sondern an der Consumer-Seite.

- timestamp: 2026-06-13
  checked: shifty-dioxus/src/page/absences.rs:1786-1802 (refetch use_effect)
  found: ABSENCE_REFRESH (1786), sales_persons (1787), current_sp_id (1788)
    werden AUSSERHALB der Closure gelesen und als Snapshots captured. Innen
    nur `let _ = refresh_token;` + `*selected_year.read()` (1793).
  implication: Effect abonniert NUR selected_year. Re-fired weder bei
    ABSENCE_REFRESH-Bump (Bug 3) noch bei sales_persons-Update (Bug 2).

- timestamp: 2026-06-13
  checked: employees_list.rs:57, shiftplan.rs:162 + 251-252 (known-good Idiome)
  found: Diese lesen den Refresh-Token / week/year INNERHALB der
    use_resource/use_effect-Closure.
  implication: Bestaetigt das korrekte Dioxus-0.6-Subscribe-Idiom; absences.rs
    weicht davon ab → Root-Cause.

- timestamp: 2026-06-13
  checked: shifty-dioxus/src/loader.rs:891-934 load_absence_periods_all
  found: person_name wird per Join gegen die uebergebene sales_persons-Liste
    gesetzt; bei leerer Liste bleibt der Name leer.
  implication: LoadAll([]) beim Mount-Race → leere Namen (Bug 2).

## Resolution

root_cause: |
  Der refetch-`use_effect` in AbsencesPage (shifty-dioxus/src/page/absences.rs)
  las seine reaktiven Quellen — ABSENCE_REFRESH, sales_persons, current_sp_id —
  AUSSERHALB der Closure und captured nur Snapshots. In Dioxus 0.6 abonniert ein
  use_effect nur Signale, die INNERHALB der Closure gelesen werden. Damit re-fired
  der Effect ausschliesslich bei selected_year-Aenderung.
  → Bug 2 (Name weg): Mount-Race — Effect feuert mit leerer sales_persons-Liste;
    der Name-Join in load_absence_periods_all schlaegt fehl; spaeteres Laden von
    sales_persons re-fired den Effect nicht → Namen bleiben leer ("—").
  → Bug 3 (kein Refresh): ConvertExtraHours bumpt ABSENCE_REFRESH, aber der Effect
    abonniert es nicht → kein LoadAll → View bleibt stale.

fix: |
  Die vier reaktiven Reads (ABSENCE_REFRESH, sales_persons, current_sp_id,
  selected_year) in die use_effect-Closure verschoben, damit Dioxus den Effect
  auf alle abonniert. Effect re-fired jetzt bei Refresh-Bump (nach Convert/
  Create/Update/Delete), bei sales_persons-Load, bei Self-User-Resolution und bei
  Jahreswechsel. Plus Regressions-Test, der die In-Closure-Subscription absichert.

verification: |
  - cargo test (shifty-dioxus): 595 passed, 0 failed (inkl. neuem Regressions-Test).
  - Regressions-Test red/green verifiziert: mit Bug-Pattern (Read ausserhalb
    Closure) schlaegt er fehl (before=1 after=1), mit Fix passt er.
  - WASM-Build-Gate (nix develop): cargo build --target wasm32-unknown-unknown ok.
  - LIVE END-TO-END VERIFIZIERT (2026-06-13, Chrome auf localhost:8080, dx-Build
    14:59:27 enthält den Fix, mtime absences.rs 14:59:07):
    * Bug 2 (Name): Konsole zeigt LoadAll([]) beim Mount (15:28:30), unmittelbar
      gefolgt von LoadAll([Anna Müller, Lisa Weber, Max Schmidt, Sarah Fischer,
      Tom Bauer]) — der Effect RE-FEUERT beim sales_persons-Load. UI zeigt alle
      Namen korrekt (kein "—"). FIXED.
    * Bug 3 (Refresh): Erfolgreiche Umwandlung eines 8h-Urlaubs-extra_hours in
      einen Zeitraum (15.06.2026). Konsole 15:32:59: ConvertExtraHours -> direkt
      LoadAll(volle Liste) + "Fetching absence periods (all)" (ABSENCE_REFRESH-
      Bump triggert Effect). Die Liste aktualisierte sich OHNE manuellen Reload:
      hours-based-Zeile verschwand, neuer Zeitraum 2026-06-15 erschien, Name blieb
      sichtbar. FIXED.
    * Nebenbefund (kein Bug): Erster Convert-Versuch auf 2026-05-18 lieferte
      korrekt 422 OverlappingPeriod, weil dort bereits ein Urlaubszeitraum lag —
      erwartetes Backend-Validierungsverhalten, nicht Teil dieses Bugs.

files_changed:
  - shifty-dioxus/src/page/absences.rs (use_effect Reads in Closure verschoben + Regressions-Test)

## Eliminated
