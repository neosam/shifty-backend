---
phase: 39-kw-status-grundlage
reviewed: 2026-07-02T12:00:00Z
depth: deep
files_reviewed: 31
files_reviewed_list:
  - migrations/sqlite/20260702000000_create-week-status.sql
  - dao/src/lib.rs
  - dao/src/week_status.rs
  - dao_impl_sqlite/src/lib.rs
  - dao_impl_sqlite/src/week_status.rs
  - service/src/lib.rs
  - service/src/week_status.rs
  - service_impl/src/lib.rs
  - service_impl/src/week_status.rs
  - service_impl/src/test/mod.rs
  - service_impl/src/test/week_status.rs
  - rest-types/src/lib.rs
  - rest/src/lib.rs
  - rest/src/week_status.rs
  - shifty_bin/src/main.rs
  - shifty-dioxus/Dioxus.toml
  - shifty-dioxus/src/api.rs
  - shifty-dioxus/src/app.rs
  - shifty-dioxus/src/component/atoms/mod.rs
  - shifty-dioxus/src/component/atoms/week_status_badge.rs
  - shifty-dioxus/src/component/mod.rs
  - shifty-dioxus/src/component/week_status_dropdown.rs
  - shifty-dioxus/src/i18n/cs.rs
  - shifty-dioxus/src/i18n/de.rs
  - shifty-dioxus/src/i18n/en.rs
  - shifty-dioxus/src/i18n/mod.rs
  - shifty-dioxus/src/page/shiftplan.rs
  - shifty-dioxus/src/service/mod.rs
  - shifty-dioxus/src/service/week_status.rs
  - shifty-dioxus/src/state/mod.rs
  - shifty-dioxus/src/state/week_status.rs
findings:
  critical: 0
  high: 0
  medium: 3
  low: 4
  total: 7
status: issues_found
---

# Phase 39: Code Review — KW-Status Grundlage

**Reviewed:** 2026-07-02T12:00:00Z
**Depth:** deep
**Files Reviewed:** 31
**Status:** issues_found

---

## Zusammenfassung

Phase 39 implementiert den `week_status`-Vertikal-Slice vollständig: Migration, DAO-Trait/Impl (TEXT-Diskriminante, TryFrom-Closed-Match), Service-Trait/-Impl (Basic-Tier, Permission-Gate, Soft-Delete), REST-Handlers, REST-Types, DI-Verdrahtung und Frontend (WeekStatus-Enum, i18n, API-Client, Store/Coroutine, WeekStatusBadge, WeekStatusDropdown, Shiftplan-Integration).

**Sicherheit / Autorisierung:** Der `SHIFTPLANNER_PRIVILEGE`-Gate sitzt korrekt als erste Anweisung in `set_week_status`, vor jedem DAO-Zugriff (D-39-01 eingehalten). Lesepfad ist ungegated (T-39-03). Kein HTTP-423/Locked-Enforcement aus Phase 40 eingeschleppt.

**Korrektheit:** ISO-Jahr korrekt via `to_iso_week_date().0` (in `js.rs`) bzw. via Thursday-Trick (Gregorian-Jahr = ISO-Wochenjahr für Donnerstage, korrekte Navigation). TryFrom-Closed-Match schlägt unbekannte Diskriminanten mit `EnumValueNotFound` zurück. Soft-Delete⇔Unset-Round-Trip korrekt (row absence = Unset). Transaktion umfasst find+write (kein TOCTOU). Fresh-Fetch-Flow korrekt (Coroutine macht GET nach PUT, kein optimistisches Update).

**Qualität:** Drei mittelgradige und vier geringfügige Befunde dokumentiert; kein Blocker.

---

## Narrative Findings

### Befunde nach Schwere

---

### WR-01: `WeekStatusBadge` panikt bei `Unset` — Vertrag nur per Konvention, nicht per Typ (MEDIUM)

**Datei:** `shifty-dioxus/src/component/atoms/week_status_badge.rs:44`

**Issue:** `week_status_badge_class` enthält `WeekStatus::Unset => unreachable!("Badge wird nie fuer Unset gerendert")`. Wird die Komponente mit `status = WeekStatus::Unset` aufgerufen, führt das in WASM zu einem Trap (Tab-Absturz oder weißes Bild). Der Schutz liegt ausschließlich in der Konvention, dass der Aufrufer vorab `should_show_badge` prüfen muss. Der Props-Typ `WeekStatusBadgeProps.status: WeekStatus` akzeptiert alle vier Varianten, inkl. `Unset`. Kein zukünftiger Aufrufer erhält einen Compilefehler wenn er `WeekStatusBadge { status: WeekStatus::Unset }` schreibt.

Zusätzlich ist die Inkonsistenz bemerkenswert: `week_status_label_key` behandelt `Unset` normal (liefert `Key::WeekStatusUnset`), während die unmittelbar danach aufgerufene `week_status_badge_class` bei `Unset` panikt — die Initialisierungs-Sequenz in der Komponente macht also einen halben Schritt, bevor der Panic zuschlägt.

**Fix:**
```rust
// Option A — den unreachable!() beibehalten, aber die Props auf nicht-Unset einschränken
// (neuer Typ oder eigenes Enum SetWeekStatus { InPlanning, Planned, Locked }):
#[derive(Props, Clone, PartialEq)]
pub struct WeekStatusBadgeProps {
    pub status: SetWeekStatus, // kein Unset möglich
}

// Option B — graceful fallback: Unset rendert Element::None
WeekStatus::Unset => return None,  // statt unreachable!()
```

---

### WR-02: `WeekStatusStore.loaded_year` / `loaded_week` werden geschrieben, aber nie gelesen — toter Zustand (MEDIUM)

**Datei:** `shifty-dioxus/src/service/week_status.rs:30-31,55-56`

**Issue:** `WeekStatusStore` enthält `loaded_year: Option<u32>` und `loaded_week: Option<u8>`. Sie werden in `load_week_status()` nach jedem erfolgreichen GET-Fetch gesetzt. Kein einziger Aufrufer liest diese Felder aus `WEEK_STATUS_STORE`. Analogerweise implementiert `WeeklySummaryStore.loaded_week` einen echten Render-Guard ("überspringe Reload, falls bereits für diese Woche geladen"), der auch tatsächlich ausgelesen wird. Für `WeekStatusStore` fehlt dieser Konsument vollständig — die Felder sind toter Zustand, der Vewirrung stiftet: sie suggerieren einen Caching-Guard, der nicht existiert.

**Fix:**
```rust
// Felder entfernen, solange kein Render-Guard implementiert ist:
#[derive(Clone, Default)]
pub struct WeekStatusStore {
    pub status: WeekStatus,
    // loaded_year / loaded_week erst wieder ergänzen,
    // wenn ein Render-Guard tatsächlich diese Werte prüft.
}
```

---

### WR-03: Soft-Delete in `WeekStatusDaoImpl::delete` bumpt `update_version` nicht (MEDIUM)

**Datei:** `dao_impl_sqlite/src/week_status.rs:163-175`

**Issue:** Der DELETE-Pfad (Soft-Delete) setzt `deleted` und `update_process`, aber nicht `update_version`. Andere Entitäten (`absence_period`, `booking`, `billing_period`) aktualisieren `update_version` auch beim Soft-Delete:

```sql
-- absence_period delete (Referenz-Pattern):
UPDATE absence_period SET deleted = ?, update_version = ?, update_process = ? WHERE id = ?

-- week_status delete (diese Implementierung — Version fehlt):
UPDATE week_status SET deleted = ?, update_process = ? WHERE id = ?
```

Unmittelbare Auswirkung: Wenn ein zukünftiger Consumer die letzte `update_version` eines Eintrags als "was zuletzt berührt hat" interpretiert, erkennt er den Soft-Delete nicht als Mutation. `week_message` nutzt dasselbe Pattern (kein Version-Bump beim Delete), aber das ist der schwächere Präzedenzfall.

**Fix:**
```rust
// In delete(): einen new_version-Parameter (oder UuidService) übergeben und setzen:
query!(
    r#"UPDATE week_status
       SET deleted = ?, update_process = ?, update_version = ?
       WHERE id = ?"#,
    now_str,
    process,
    new_version_vec, // neu: generierter UUID
    id_vec,
)
```

Da der DAO-Trait keinen `UuidService` kennt, kann der neue Versions-UUID auch direkt im Service generiert und an `delete()` übergeben werden (Trait-Erweiterung um `new_version: Uuid`), oder die Version bleibt bewusst unverändert — aber das sollte dann dokumentiert werden.

---

### LO-01: Veraltetes `#[allow(dead_code)]` auf `pub enum WeekStatusAction` (LOW)

**Datei:** `shifty-dioxus/src/service/week_status.rs:39`

**Issue:** Der Kommentar erklärt: "Consumers (send-sites) arrive in Plan 39-05". Dieser Commit ist Phase 39-05, d.h. beide Varianten (`Load`, `Set`) sind jetzt aktiv in `page/shiftplan.rs` verwendet. In einem Binary-Crate (Dioxus WASM) können `pub`-Elemente mit ungenutzten Varianten tatsächlich `dead_code`-Warnungen auslösen — aber da beide Varianten nun konsumiert werden, ist die Suppression überflüssig und irreführend.

**Fix:**
```rust
// #[allow(dead_code)] entfernen:
pub enum WeekStatusAction {
    Load { year: u32, week: u8 },
    Set { year: u32, week: u8, status: WeekStatus },
}
```

---

### LO-02: Toter 404-Pfad in `api::get_week_status` (LOW)

**Datei:** `shifty-dioxus/src/api.rs:1215-1217`

**Issue:**
```rust
if response.status() == 404 {
    return Ok(None);
}
```

Der Backend-Handler `get_week_status_by_year_and_week` gibt immer HTTP 200 zurück — auch wenn kein Eintrag vorhanden ist, wird `WeekStatusTO { status: "unset" }` geliefert, niemals 404. Die `None`-Rückgabe triggert also nie; der Code in der Coroutine (`.unwrap_or(WeekStatus::Unset)`) hat daher ebenfalls keinen Pfad zum Ausführen. Die Ablesung `Some(to).map(|to| WeekStatus::from(&to))` liefert immer `Some`.

Kein Laufzeit-Bug, aber dead code, der zukünftige Leser über das Protokoll verwirrt.

**Fix:** Entweder den 404-Branch entfernen (und API-Kontrakt im Kommentar dokumentieren) oder `get_week_status` auf `Result<WeekStatusTO, reqwest::Error>` umstellen (kein `Option` mehr):
```rust
pub async fn get_week_status(
    config: Config,
    year: u32,
    week: u8,
) -> Result<WeekStatusTO, reqwest::Error> {
    // ...
    response.error_for_status_ref()?;
    Ok(response.json().await?)
}
```

---

### LO-03: PUT-Handler ignoriert `body.year` / `body.calendar_week` stillschweigend (LOW)

**Datei:** `rest/src/week_status.rs:84-111`

**Issue:** `upsert_week_status` akzeptiert `Json(body): Json<WeekStatusTO>`, das `year`, `calendar_week` und `status` enthält. Nur `body.status` wird verwendet; `body.year` und `body.calendar_week` werden stillschweigend verworfen. Der Pfad (`year`, `week`) ist allein autoritativ. Sendet ein Client `PUT /by-year-and-week/2026/27` mit Body `{ "year": 2025, "calendar_week": 1, "status": "planned" }`, wird Woche 2026/27 gesetzt — ohne Fehlermeldung über den Mismatch.

Kein Sicherheitsproblem (Pfad gewinnt), aber schlechte API-Ergonomie: der Request-Body-Typ ist für diesen Endpunkt zu breit und erzeugt stille Inkonsistenz.

**Fix (zwei Optionen):**
```rust
// Option A: Body-Typ auf ein schlankes Struct reduzieren
#[derive(Deserialize)]
struct WeekStatusSetBody { pub status: WeekStatusKindTO }
// Dann: Json(body): Json<WeekStatusSetBody>

// Option B: Pfad-Params gegen Body validieren und bei Mismatch 400 zurückgeben
if body.year != year || body.calendar_week != week {
    return error_handler(Err(ServiceError::BadRequest("year/week mismatch between path and body".into())));
}
```

---

### LO-04: `WeekStatusDaoImpl::update` prüft `rows_affected` nicht (LOW)

**Datei:** `dao_impl_sqlite/src/week_status.rs:131-154`

**Issue:** Die UPDATE-Query gibt kein Ergebnis (keine `rows_affected`-Prüfung) zurück. Wenn die Zeile zwischen `find_by_year_and_week` und `update` anderweitig gelöscht wurde (Concurrent-Write-Bug oder Fehlgebrauch des DAO), schlägt die UPDATE-Query lautlos durch (0 Zeilen betroffen, kein Fehler). Für die aktuell einzige Nutzung im `service_impl` liegt alles in einer Transaktion, womit dieses Szenario praktisch unmöglich ist (SQLite ist Single-Writer). Langfristig bei komplexerer Nutzung ist ein stilles No-Op ein schwer zu diagnostizierendes Symptom.

Das gleiche Pattern gilt für `week_message` und andere einfache DAOs im Projekt — es ist also keine Regression, sondern ein etabliertes (aber fragiles) Muster.

**Fix:**
```rust
let result = query!(...)
    .execute(tx.tx.lock().await.as_mut())
    .await
    .map_db_error()?;

if result.rows_affected() == 0 {
    return Err(DaoError::NotFound);
}
```

---

## Überprüfte Spezifika laut Aufgabenstellung

| Kriterium | Ergebnis |
|---|---|
| SHIFTPLANNER_PRIVILEGE vor jedem DAO-Zugriff in `set_week_status` | Korrekt (Zeile 73-75 in service_impl) |
| Lesepfad ungegated | Korrekt (kein `check_permission` in `get_week_status`) |
| ISO-Jahr via `to_iso_week_date()` (nie `.year()` auf Date) | Korrekt in `js.rs`; Thursday-Trick in Shiftplan-Navigation korrekt |
| TryFrom Closed-Match (unbekannte Diskriminante rejected) | Korrekt (`EnumValueNotFound` auf unknown value) |
| Soft-Delete⇔Unset Round-Trip | Korrekt (kein Eintrag = Unset; Unset soft-deletes vorhandene Zeile) |
| Fresh-Fetch nach Mutation (kein optimistisches Drift) | Korrekt (Coroutine: GET nach erfolgreichem PUT) |
| find+write in einer Transaktion | Korrekt (`tx.clone()` teilt dieselbe Connection) |
| `#[allow(dead_code)]` Unset (nicht None) | Korrekt (Enum: `Unset` nicht `None`) |
| OpenAPI: `utoipa::path` + ApiDoc registriert | Korrekt (beide Handler annotiert, ApiDoc in REST lib eingetragen) |
| Keine rohen Tailwind-Farbklassen | Korrekt (Badge + Dropdown verwenden nur Design-Token-Klassen) |
| i18n vollständig (De/En/Cs, alle 6 Keys) | Korrekt (alle Keys in allen drei Locales vorhanden) |
| Kein Phase-40-Scope-Creep (assert_week_not_locked / HTTP 423) | Korrekt — kein Hinweis auf Lock-Enforcement gefunden |

---

_Reviewed: 2026-07-02T12:00:00Z_
_Reviewer: Claude Sonnet 4.6 (gsd-code-reviewer, adversarial mode)_
_Depth: deep_
