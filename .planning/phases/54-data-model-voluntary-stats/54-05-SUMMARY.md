---
phase: 54-data-model-voluntary-stats
plan: 05
subsystem: [frontend, dioxus, wasm, i18n, report]
tags: [frontend, dioxus, voluntary-stats, hr-gate, i18n, report, VOL-STAT-01, VOL-STAT-02, VOL-ACCT-01, VOL-ACCT-02, D-F1-01]
status: complete
requirements:
  - VOL-STAT-01
  - VOL-STAT-02
  - VOL-ACCT-01
  - VOL-ACCT-02
dependency_graph:
  requires:
    - 54-04 (REST-Endpoint GET /report/{id}/voluntary-stats + VoluntaryStatsTO DTO)
  provides:
    - FE-Sichtbarkeit "Freiwillig Ø/Woche / Soll / Delta" im EmployeeView-Report (HR-only per Backend-Nullable)
    - state::employee::VoluntaryStats + loader::load_voluntary_stats + Component VoluntaryStatsRow
    - i18n-Keys VoluntaryHoursIstPerWeek / VoluntaryHoursSoll / VoluntaryHoursDelta (de/en/cs)
  affects:
    - Plan 54-06 (Docs — F14 Update Reporting-Kette)
tech-stack:
  added: []
  patterns:
    - rest-types Cargo-Dep direkt konsumiert (kein FE-Copy-Struct — Plan-Zweig "wenn Cargo-Dep vorhanden")
    - Nullable-Guard in Component (Non-HR = alle Felder None -> empty rsx) statt FE-Rollencheck
      (Fat Backend, Thin Client — Memory feedback_fat_backend_thin_client)
    - Store-Slot mit VoluntaryStats::default() Fallback bei HTTP-Fehler (Report bleibt nutzbar)
    - EmployeeView-Wrapper reicht Store-Feld an EmployeeViewPlain durch (analog weekly_statistics/attendance_statistics)
    - Prefix-Match-Proxy /report deckt /voluntary-stats mit ab (kein neuer Dioxus.toml-Eintrag)
    - Triple-Locale i18n (de/en/cs) im gleichen Commit (Konvention aus Memory)
key-files:
  created:
    - shifty-dioxus/src/component/voluntary_stats_row.rs
  modified:
    - shifty-dioxus/src/state/employee.rs (VoluntaryStats + From<&VoluntaryStatsTO>)
    - shifty-dioxus/src/api.rs (get_voluntary_stats)
    - shifty-dioxus/src/loader.rs (load_voluntary_stats)
    - shifty-dioxus/src/service/employee.rs (Store-Slot + Loader-Call in load_employee_data)
    - shifty-dioxus/src/component/employee_view.rs (Prop + Row-Einbindung + Wrapper-Wiring)
    - shifty-dioxus/src/component/mod.rs (Modul-Registrierung)
    - shifty-dioxus/src/i18n/mod.rs (3 neue Key-Enum-Varianten)
    - shifty-dioxus/src/i18n/de.rs, en.rs, cs.rs (3x3 Uebersetzungen)
    - shifty-dioxus/src/state/weekly_overview.rs (pre-existing clippy::unnecessary_sort_by fix)
    - .planning/phases/54-data-model-voluntary-stats/54-VALIDATION.md (54-05-01/02/03 -> green)
decisions:
  - "Plan-Skizze `shifty-dioxus/src/rest_types.rs` und `shifty-dioxus/src/state/employee_report.rs`
    existieren im Repo nicht — Struktur ist anders: `rest-types` ist als Cargo-Dep verlinkt
    (kein FE-Copy-Struct noetig), und `state::employee::Employee` haelt den Employee-Report.
    Realer Layout-Plan: VoluntaryStatsTO direkt aus `rest_types` importieren, VoluntaryStats
    als Struct neben ExtraHours in `state/employee.rs`. Der Plan-Text nennt genau diesen
    Zweig als Alternative unter Task 1(a) — kein Design-Bruch."
  - "Store-Slot statt Signal in employee_details.rs — der Report nutzt bereits einen
    GlobalSignal-Store (`EMPLOYEE_STORE`) und den Loader-Coroutine-Pfad (`EmployeeAction::LoadEmployeeDataUntilNow`).
    voluntary_stats wird deshalb sequentiell nach dem Main-Report-Load in
    `service::employee::load_employee_data` gefetcht und in denselben Store geschrieben.
    Das erspart einen separaten use_effect + Signal im Page-Component und haelt den
    Refresh-Path konsistent (Refresh -> Store-Update -> automatisches Re-Render)."
  - "VoluntaryStatsRow rendert 3 TupleRows statt 1 kombinierter <tr>-Zeile. Der existierende
    Report nutzt `TupleRow` (label-value-Div), keine <table>. AVG-01-Layout-Praezedenz aus
    Plan-Text ist rein layoutmaessig -- 3 einzelne Rows tragen die gleiche Semantik + sind
    konsistent mit dem umliegenden OVERALL-Block. Non-HR-Guard sitzt an einer Stelle
    (Component-let-else) und entfernt alle 3 Rows atomisch."
  - "Fallback auf `VoluntaryStats::default()` bei HTTP-Fehler statt Report-Abbruch.
    `EmployeeAction::LoadEmployeeDataUntilNow` konsumiert eine `Result`-Chain via `?`;
    ein 500er beim voluntary-stats-Endpoint darf den ganzen Report nicht ausschalten.
    Non-HR-Fall (200 mit lauter None) und Fehler-Fall (Default = alle None) sind aus
    UI-Sicht identisch -> beide unterdrucken die Row."
  - "Clippy-Regression-Fix `state/weekly_overview.rs`: FE-Clippy-Gate `-D warnings`
    lief in eine pre-existing `clippy::unnecessary_sort_by` (Phase 53). Gemaess Plan
    Task 5 `keine allow ohne Reason` -> Cleanup vor Commit (`sort_by` -> `sort_by_key`;
    Sortier-Semantik identisch)."
metrics:
  duration: ~10 min
  completed: 2026-07-07
  tasks: 5
  files_touched: 10
  tests_added: 4
  commits: 4
---

# Phase 54 Plan 05: Frontend HR-Only Freiwillig-Stunden-Row Summary

**One-liner:** HR-only Employee-Report-Row "Freiwillig Ø/Woche / Soll / Delta" mit Nullable-Backend-Guard, DTO-Direktkonsum via rest-types-Cargo-Dep, 3 neue i18n-Keys x 3 Sprachen, 4 SSR-Component-Tests grün, kein FE-Rollencheck (Fat Backend, Thin Client).

## Component-Signatur + Guard-Logik

```rust
// shifty-dioxus/src/component/voluntary_stats_row.rs
#[derive(Props, Clone, PartialEq)]
pub struct VoluntaryStatsRowProps {
    pub stats: VoluntaryStats,
}

#[component]
pub fn VoluntaryStatsRow(props: VoluntaryStatsRowProps) -> Element {
    let stats = props.stats;

    // HR-Only-Guard via Nullable-DTO — kein FE-Rollen-Check.
    // Non-HR: Backend liefert alle Felder None -> Row rendert leer.
    let (Some(ist_per_week), Some(soll), Some(delta)) =
        (stats.ist_per_contract_week, stats.soll_total, stats.delta)
    else {
        return rsx! {};
    };

    let i18n = I18N.read().clone();
    let hours_str: ImStr = ImStr::from(i18n.t(Key::Hours).as_ref());

    let delta_class = if delta < 0.0 {
        "font-mono tabular-nums text-warn"
    } else {
        "font-mono tabular-nums"
    };

    rsx! {
        TupleRow { label: ..., value: rsx! { ... ist_per_week ... } }
        TupleRow { label: ..., value: rsx! { ... soll ... } }
        TupleRow { label: ..., value: rsx! { class: "{delta_class}", ... delta ... } }
    }
}
```

## Loader-Signatur

```rust
// shifty-dioxus/src/loader.rs
pub async fn load_voluntary_stats(
    config: Config,
    sales_person_id: Uuid,
    year: u32,
) -> Result<VoluntaryStats, ShiftyError> {
    let to = api::get_voluntary_stats(config, sales_person_id, year).await?;
    Ok(VoluntaryStats::from(&to))
}

// shifty-dioxus/src/api.rs
pub async fn get_voluntary_stats(
    config: Config,
    sales_person_id: Uuid,
    year: u32,
) -> Result<VoluntaryStatsTO, reqwest::Error> {
    let url = format!(
        "{}/report/{}/voluntary-stats?year={}",
        config.backend, sales_person_id, year
    );
    let response = reqwest::get(url).await?;
    response.error_for_status_ref()?;
    let res = response.json().await?;
    Ok(res)
}
```

## Store-Wiring

```rust
// shifty-dioxus/src/service/employee.rs — load_employee_data
let voluntary_stats =
    loader::load_voluntary_stats(CONFIG.read().clone(), sales_person_id, year)
        .await
        .unwrap_or_default();   // HTTP-Fehler -> Default (alle None -> Row rendert leer)
*EMPLOYEE_STORE.write() = EmployeeStore {
    employee,
    extra_hours,
    custom_extra_hours_definitions,
    year,
    until_week,
    weekly_statistics,
    attendance_statistics,
    voluntary_stats,
};
```

## i18n-Diff (9 Zeilen)

**shifty-dioxus/src/i18n/mod.rs — Key-Enum:**

```rust
// Phase 54: HR-only Freiwillig-Stunden-Konto (VOL-STAT-01/02)
VoluntaryHoursIstPerWeek,
VoluntaryHoursSoll,
VoluntaryHoursDelta,
```

**de.rs:**

```rust
i18n.add_text(Locale::De, Key::VoluntaryHoursIstPerWeek, "Freiwillig \u{00D8} / Woche");
i18n.add_text(Locale::De, Key::VoluntaryHoursSoll,       "Freiwillig Soll");
i18n.add_text(Locale::De, Key::VoluntaryHoursDelta,      "Freiwillig Delta");
```

**en.rs:**

```rust
i18n.add_text(Locale::En, Key::VoluntaryHoursIstPerWeek, "Voluntary avg / week");
i18n.add_text(Locale::En, Key::VoluntaryHoursSoll,       "Voluntary target");
i18n.add_text(Locale::En, Key::VoluntaryHoursDelta,      "Voluntary delta");
```

**cs.rs (ASSUMED — native-check als Manual-Verify):**

```rust
i18n.add_text(Locale::Cs, Key::VoluntaryHoursIstPerWeek, "Dobrovolné prům. / týden");
i18n.add_text(Locale::Cs, Key::VoluntaryHoursSoll,       "Dobrovolné plán");
i18n.add_text(Locale::Cs, Key::VoluntaryHoursDelta,      "Dobrovolné rozdíl");
```

## Row-Einbindung in EmployeeView

```rust
// shifty-dioxus/src/component/employee_view.rs — OVERALL-Box, nach der volunteer_hours-Zeile
if show_volunteer_work(employee.volunteer_hours) {
    TupleRow {
        label: ImStr::from(volunteer_work_str.as_ref()),
        value: rsx! { span { class: "font-mono tabular-nums",
            {format!("{} {}", format_hours(employee.volunteer_hours, 2), hours_str)}
        } },
    }
}
// Phase 54: HR-only Freiwillig-Stunden-Konto (Nullable-Guard im Component).
crate::component::voluntary_stats_row::VoluntaryStatsRow {
    stats: props.voluntary_stats.clone(),
}
div { class: "border-t border-border my-2" }
```

## Test-Ergebnis (SSR-Component-Tests)

```
running 4 tests
test component::voluntary_stats_row::tests::renders_empty_when_soll_is_none_even_if_ist_is_some ... ok
test component::voluntary_stats_row::tests::renders_empty_when_ist_per_week_is_none ... ok
test component::voluntary_stats_row::tests::renders_three_rows_when_all_fields_are_some ... ok
test component::voluntary_stats_row::tests::negative_delta_gets_warn_class ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 802 filtered out; finished in 0.00s
```

Alle Component-Guards verifiziert:

| Test | Input | Erwartung |
|------|-------|-----------|
| `renders_empty_when_ist_per_week_is_none` | `VoluntaryStats::default()` (Non-HR) | Kein "Freiwillig"/"Voluntary"/"Dobrovoln" im Output |
| `renders_empty_when_soll_is_none_even_if_ist_is_some` | Nur `ist_per_contract_week: Some(2.0)` | Defence-in-depth: immer noch leer |
| `renders_three_rows_when_all_fields_are_some` | HR-Fall: alle Some | 3 `border-b` Rows + `2.00` + `+0.00` |
| `negative_delta_gets_warn_class` | delta = -4.0 | `text-warn` class + `-4.00` sichtbar |

**Full FE-Suite:** `806 passed; 0 failed; 0 ignored` (`cargo test -p shifty-dioxus`).

## Manual-Verify-Anweisung (Roundtrip)

**Voraussetzungen:**
1. Backend gestartet: `cd shifty-backend && cargo run` (Port 3000)
2. Frontend gestartet: `cd shifty-backend/shifty-dioxus && dx serve --hot-reload` (Port 8080)
3. Datenbank hat mindestens einen Sales-Person mit `committed_voluntary > 0` + Manual-Extra-Hours der Kategorie `VolunteerWork`

**HR-Rolle (positiv):**

1. Login als DEVUSER (default Mock-Auth admin) -> hat `hr`-Privileg.
2. Navigiere zu `Employees -> [Sales-Person mit VoluntaryWork]`.
3. Im OVERALL-Block muss unter der bestehenden `Volunteer Work` / `Freiwilligenarbeit` Zeile eine neue 3-teilige Sektion erscheinen:
   - `Freiwillig Ø / Woche` (DE) / `Voluntary avg / week` (EN): Zahl mit 2 Nachkommastellen + `Stunden`
   - `Freiwillig Soll`: Zahl mit 2 Nachkommastellen + `Stunden`
   - `Freiwillig Delta`: `+X.XX Stunden` (grün/neutral) oder `-X.XX Stunden` (rot / `text-warn`)
4. Verifikation (Memory `reference_dioxus_browser_verify_reports` — Screenshots timeouten, `get_page_text + find` ist stabiler):

```javascript
// In Playwright/CDP-Session:
const text = await page.content();
console.assert(text.includes('Freiwillig'), 'HR row not rendered');
console.assert(/Freiwillig Delta/.test(text), 'Delta label missing');
```

**Non-HR-Rolle (negativ):**

1. Login als Non-HR-User (z.B. `some-non-hr-user`, s. Phase 54-04 Integration-Test-Fixture).
2. Navigiere zu `Employees -> [Sales-Person mit VoluntaryWork]` (falls User Zugriff hat) oder `My Employee Details`.
3. Im OVERALL-Block darf KEINE `Freiwillig`/`Voluntary` Zeile erscheinen (Backend redigiert alle Felder auf `null`, Component rendert `rsx! {}`).
4. Verifikation:

```javascript
const text = await page.content();
console.assert(!/Freiwillig Ø|Freiwillig Soll|Freiwillig Delta/.test(text), 'Non-HR row leaked');
```

**cs-Locale:** wechsle auf `Locale::Cs` und prüfe, dass die 3 Labels als `Dobrovolné prům. / týden`, `Dobrovolné plán`, `Dobrovolné rozdíl` erscheinen. `[ASSUMED]` — native-Check bleibt Manual-Verify (RESEARCH §D.4).

## Deviations from Plan

**1. [Rule 3 - Blocker-Fix] Plan-Dateipfade `state/employee_report.rs` und `rest_types.rs` existieren im Repo nicht**

- **Gefunden bei:** Task 1 (Struktur-Verify).
- **Issue:** Plan `files_modified:` listet `shifty-dioxus/src/rest_types.rs` und `shifty-dioxus/src/state/employee_report.rs` — beides existiert nicht. Die reale Struktur: `rest-types` ist als Cargo-Dep verlinkt (kein FE-Copy-Struct noetig), und Employee-State liegt in `state/employee.rs::Employee`.
- **Fix:** Plan-Zweig "wenn Cargo-Dep vorhanden → direkter Import" angewandt (Plan-Text Task 1(a)). `VoluntaryStats` als Struct in `state/employee.rs` neben `ExtraHours`, `VoluntaryStatsTO` direkt aus `rest_types` importiert.
- **Files:** `shifty-dioxus/src/state/employee.rs`, `shifty-dioxus/src/api.rs`, `shifty-dioxus/src/loader.rs`.
- **Commit:** `a1412a7`.

**2. [Rule 3 - Blocker-Fix] Kein use_effect in employee_details.rs — Store-Slot statt Signal**

- **Gefunden bei:** Task 3 (Einbindung).
- **Issue:** Plan-Skizze verwendet `use_signal<Option<VoluntaryStats>>` + `use_effect + spawn` im Page-Component. Der Report nutzt aber bereits einen GlobalSignal-Store (`EMPLOYEE_STORE`) und einen zentralen Coroutine-Loader (`EmployeeAction::LoadEmployeeDataUntilNow`), der `load_employee_data` sequenziell aufruft (`weekly_statistics`, `attendance_statistics` folgen dem gleichen Muster).
- **Fix:** VoluntaryStats-Load in `service::employee::load_employee_data` eingehaengt, in `EMPLOYEE_STORE.voluntary_stats` geschrieben, ueber Store-Wrapper `EmployeeView` an `EmployeeViewPlain` durchgereicht — analog `weekly_statistics/attendance_statistics`. Ergebnis: konsistenter Refresh-Path + kein Signal-Doppel + funktioniert transparent mit `Refresh/NextYear/PrevYear`.
- **Files:** `shifty-dioxus/src/service/employee.rs`, `shifty-dioxus/src/component/employee_view.rs`.
- **Commit:** `50be1b6`.

**3. [Rule 3 - Blocker-Fix] Pre-existing clippy::unnecessary_sort_by in state/weekly_overview.rs**

- **Gefunden bei:** Task 5 (FE-Clippy-Gate).
- **Issue:** `cargo clippy -p shifty-dioxus -- -D warnings` schlug fehl an Zeile 66 in `state/weekly_overview.rs`. Warning ist pre-existing (Phase 53, commit `4da9a6b`) — nicht durch meinen Diff verursacht. Plan Task 5 fordert aber "keine `#[allow(...)]` ohne Reason".
- **Fix:** `v.sort_by(|x, y| x.name.to_lowercase().cmp(&y.name.to_lowercase()))` → `v.sort_by_key(|x| x.name.to_lowercase())`. Semantisch identisch (beides stabil, Namens-Duplikate behalten Insertion-Order).
- **Files:** `shifty-dioxus/src/state/weekly_overview.rs`.
- **Commit:** `ff0a899`.
- **Praezedenz:** Phase 45 D-45-01 (FE-Clippy scharf).

Keine weiteren Abweichungen. Task 4 (Dioxus.toml Proxy) war eine reine Verifikation — der Prefix-Proxy `localhost:3000/report` existiert seit Phase 33 und deckt `/voluntary-stats` per Prefix-Match ab. Kein neuer Eintrag noetig.

## Auth-Gates

Keine — DEVUSER-Setup ist im Mock-Auth-Feature-Flag.

## Commits

- `a1412a7` **feat(54-05): add VoluntaryStats state mirror + loader** — `state::employee::VoluntaryStats` mit 5 Option-Feldern, `From<&VoluntaryStatsTO>`, `api::get_voluntary_stats` + `loader::load_voluntary_stats`.
- `546c137` **feat(54-05): add VoluntaryStatsRow component + i18n de/en/cs** — Component + 3 neue Key-Enum-Varianten + 9 Uebersetzungen + 4 SSR-Tests.
- `50be1b6` **feat(54-05): wire voluntary-stats into EmployeeView / EmployeeStore** — Store-Slot, Loader-Call in `load_employee_data`, Prop + Row-Einbindung in `EmployeeView`.
- `ff0a899` **chore(54-05): fix pre-existing clippy::unnecessary_sort_by warning** — `Vec::sort_by` → `Vec::sort_by_key` in `state/weekly_overview.rs`.

## Verification-Log

| Gate | Command | Result |
|------|---------|--------|
| WASM-Build (Task 1) | `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` | Finished (34.68s, 3 warnings — dead-code, verschwinden mit Task 3) |
| WASM-Build (Task 2) | `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` | Finished (26.25s) |
| WASM-Build (Task 3 Final) | `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` | Finished (9.93s, keine Warnings) |
| Component-Tests | `cd shifty-dioxus && cargo test -p shifty-dioxus voluntary_stats` | 4 passed; 0 failed |
| Proxy-Verify | `grep -q 'localhost:3000/report' shifty-dioxus/Dioxus.toml` | OK: /report proxy exists |
| Full FE-Suite | `cd shifty-dioxus && cargo test -p shifty-dioxus` | 806 passed; 0 failed; 0 ignored |
| Backend-Clippy Gate | `SQLX_OFFLINE=true cargo clippy --workspace -- -D warnings` | Finished (0.25s), keine Warnings |
| FE-Clippy Gate | `cd shifty-dioxus && cargo clippy -p shifty-dioxus -- -D warnings` | Finished (22.84s, nach sort_by_key-Fix keine Warnings) |

## Self-Check: PASSED

- `shifty-dioxus/src/state/employee.rs` enthaelt `pub struct VoluntaryStats` + `impl From<&VoluntaryStatsTO>` ✓
- `shifty-dioxus/src/api.rs` enthaelt `pub async fn get_voluntary_stats` ✓
- `shifty-dioxus/src/loader.rs` enthaelt `pub async fn load_voluntary_stats` ✓
- `shifty-dioxus/src/service/employee.rs` `EmployeeStore.voluntary_stats` Feld existiert + wird in `load_employee_data` gesetzt ✓
- `shifty-dioxus/src/component/voluntary_stats_row.rs` existiert mit Nullable-Guard + 4 Tests ✓
- `shifty-dioxus/src/component/employee_view.rs` reicht `voluntary_stats` an `EmployeeViewPlain` durch und rendert `VoluntaryStatsRow` unter `volunteer_hours`-Zeile ✓
- i18n `mod.rs` enthaelt `VoluntaryHoursIstPerWeek/Soll/Delta` Keys ✓
- i18n `de.rs`, `en.rs`, `cs.rs` haben je 3 `add_text`-Zeilen fuer die neuen Keys ✓
- `shifty-dioxus/Dioxus.toml` `localhost:3000/report` Proxy existiert (Prefix-Match) ✓
- Commits `a1412a7`, `546c137`, `50be1b6`, `ff0a899` in git log ✓
- `cargo build --target wasm32-unknown-unknown` grün, `cargo test -p shifty-dioxus` 806 passed, `cargo clippy -p shifty-dioxus -- -D warnings` grün ✓
