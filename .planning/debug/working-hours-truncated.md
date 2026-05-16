---
slug: working-hours-truncated
status: resolved
trigger: |
  DATA_START
  Beim letzten Fix mit den ganzen Zahlen bei den Arbeitsstunden haben wir noch ein Problem.
  Und zwar wird 40.25 immer wieder auf 40 zurückgesetzt. An das Backend wird 40.25 geschickt
  aber vom Backend kommt dann 40.0.
  DATA_END
created: 2026-05-16
updated: 2026-05-16
related_commits:
  - 72e045f feat(hours-input): step=0.01 für alle Hour-Eingaben im Frontend
---

# Debug Session: Working Hours fraction stripped on backend roundtrip

## Symptoms

- **Expected behavior**: User trägt `40.25` Wochenstunden im Arbeitsvertrag ein → Backend speichert `40.25` → Backend liefert auf Reload `40.25` zurück.
- **Actual behavior**: User trägt `40.25` ein → Frontend sendet `40.25` an Backend (im Network-Tab verifiziert) → Backend antwortet mit `40.0`. Wert wird also irgendwo zwischen Deserialize und Response auf Integer-Wert getrunkated.
- **Affected field**: Wochenstunden im Arbeitsvertrag — `FloatInput` in `shifty-dioxus/src/component/employee_work_details_form.rs`. Vermutlich `working_hours` / `expected_hours` o.ä. in `employee_work_details` DTO/Entity.
- **Timeline**: Frontend-Side wurde gerade durch Commit `72e045f` (step=0.01 in 4 Hour-Inputs) gefixt. Das Frontend kann jetzt Dezimal-Eingaben tatsächlich annehmen — dadurch wird ein bisher maskiertes Backend-Problem sichtbar. Vor diesem Fix hat der Browser durch `step=1` (default für number-input) jeden Dezimalwert schon clientseitig auf Integer normalisiert, sodass die Truncation im Backend nie auffiel.
- **Reproduction**:
  1. Backend lokal starten, Frontend mit dx serve starten.
  2. Als Admin Employee aufrufen, Arbeitsvertrag-Sektion editieren.
  3. Wochenstunden auf `40.25` ändern, speichern.
  4. Network-Tab: PUT/POST-Request enthält `40.25`, Response enthält `40.0`.
  5. UI zeigt `40.0` nach Reload.
- **Error messages**: Keine — silent value truncation, kein Validation-Error, kein Log-Eintrag bekannt.

## Hypothesis Space (initial)

Mögliche Stellen, an denen die Dezimalstellen verloren gehen — vom Frontend nach Backend:

1. **rest-types DTO** (Frontend → Backend Request): Feld als `i32`/`u32` typisiert statt `f32`.
2. **REST-Handler in `rest/`**: Konvertiert DTO → Service-Entity über `as i32` oder `.round() as i32`.
3. **Service Layer** (`service_impl/`): Berechnet etwas mit dem Wert und castet zu Integer.
4. **DAO Layer** (`dao_impl_sqlite/`): Bindet Wert als `INTEGER`-Spalte statt `REAL`.
5. **DB-Schema**: Spalte in der Migration als `INTEGER` statt `REAL` deklariert.
6. **Response-Pfad**: Symmetrisches Problem auf Lese-Seite.

## Current Focus

- **hypothesis**: Bestätigt — DAO `update()` castet `expected_hours: f32` zu `i64`, bevor an SQLite gebunden wird. → Truncation.
- **test**: Roundtrip-Test im `shifty_bin`-Integration-Test-Modul (in-memory SQLite, echter Service + echter DAO).
- **expecting**: 3 neue Tests, vorher rot ohne Fix, grün mit Fix.
- **next_action**: (erledigt)

## Evidence

- timestamp: 2026-05-16 — User-Beobachtung: Network-Tab zeigt Request-Payload `40.25`, Response-Payload `40.0`. Roundtrip-Problem reproduzierbar.
- timestamp: 2026-05-16 — Frontend `FloatInput` in `employee_work_details_form.rs` hat seit Commit `72e045f` `step=0.01` — sendet also korrekt 2 Dezimalstellen.
- timestamp: 2026-05-16 — `rest-types/src/lib.rs:601`: `EmployeeWorkDetailsTO.expected_hours: f32`. ✓ DTO-Typ ist korrekt.
- timestamp: 2026-05-16 — `service/src/employee_work_details.rs:17`: `EmployeeWorkDetails.expected_hours: f32`. ✓ Service-Entity-Typ ist korrekt.
- timestamp: 2026-05-16 — `service_impl/src/employee_work_details.rs:244`: `entity.expected_hours = employee_work_details.expected_hours;` — kein Cast. ✓
- timestamp: 2026-05-16 — `migrations/sqlite/20240618125847_paid-sales-persons.sql`: `expected_hours FLOAT NOT NULL`. ✓ DB-Spalte ist `FLOAT`.
- timestamp: 2026-05-16 — `dao_impl_sqlite/src/employee_work_details.rs:17`: `EmployeeWorkDetailsDb.expected_hours: f64`. ✓ Read-Mapping korrekt.
- timestamp: 2026-05-16 — `dao_impl_sqlite/src/employee_work_details.rs:50`: `expected_hours: working_hours.expected_hours as f32`. ✓ TryFrom korrekt.
- timestamp: 2026-05-16 — `dao_impl_sqlite/src/employee_work_details.rs:319` (DAO `create`): `let expected_hours = entity.expected_hours as f64;` ✓ Insert-Pfad korrekt.
- timestamp: 2026-05-16 — `dao_impl_sqlite/src/employee_work_details.rs:416` (DAO `update`): **`let expected_hours = entity.expected_hours as i64;` ✗ ROOT CAUSE** — Update-Pfad castet `f32` zu `i64` und truncated die Nachkommastellen.
- timestamp: 2026-05-16 — Verified mit revert-experiment: ohne Fix sind alle 3 neuen Tests rot (`got 40` / `got 38`); mit Fix grün. Workspace-weite Tests (527 total) bleiben grün.

## Eliminated

- **Frontend**: schickt 40.25 nachweislich an Backend (User hat Network-Tab geprüft).
- **rest-types DTO**: `expected_hours: f32` korrekt typisiert.
- **REST-Handler**: kein Custom-Cast — `From<&EmployeeWorkDetailsTO>` overload forwarded `f32`.
- **Service Layer**: kein Cast, kein Rounding.
- **DB-Schema**: Spalte ist `FLOAT NOT NULL` seit erstem Migration (20240618125847).
- **DAO read-Pfad**: korrekt `f64` → `f32`, kein Integer-Roundtrip.
- **DAO create-Pfad**: korrekt `as f64`.
- **Side-bug entdeckt aber NICHT als Root-Cause: `dao_impl_sqlite/src/employee_work_details.rs:338`** in `create()`: `let version = entity.id.as_bytes().to_vec();` — verwendet `id` statt `entity.version` für die `update_version`-Spalte. Folge: nach `create()` ist die in der DB persistierte `update_version` ≠ der vom Service erzeugten Version. Service `update()` re-fetcht und vergleicht — Frontend muss daher nach POST/Reload arbeiten, nicht direkt mit dem create-Response-Body. Das ist ein separater, vorher existierender Bug; NICHT die Ursache der `40.25`-Truncation, aber sollte in einem follow-up Fix aufgeräumt werden. Tests umgehen ihn aktuell mit `reload_active`-Helper.

## Resolution

**Root cause**: `EmployeeWorkDetailsDaoImpl::update` in `dao_impl_sqlite/src/employee_work_details.rs:416` castet `entity.expected_hours` (Typ `f32`) zu `i64`, bevor sqlx den Wert bindet. Damit wird `40.25` zu `40` truncated und beim nächsten `SELECT` als `40.0` zurückgeliefert. Der `create()`-Pfad nutzt korrekt `as f64`, deshalb fiel das Problem nur auf, sobald ein Vertrag editiert (statt neu angelegt) wurde — und auch das erst, nachdem Commit `72e045f` dem Frontend `step=0.01` gegeben hat. Vorher hat das `<input type=number step=1>` die Eingabe schon clientseitig auf Integer normalisiert.

**Fix**: einzeilige Änderung in `dao_impl_sqlite/src/employee_work_details.rs:416`:

```diff
-        let expected_hours = entity.expected_hours as i64;
+        let expected_hours = entity.expected_hours as f64;
```

**Regression test**: neues Modul `shifty_bin/src/integration_test/employee_work_details_update.rs` mit 3 Roundtrip-Tests (Service → DAO → in-memory SQLite → DAO → Service):

- `test_create_preserves_fractional_expected_hours` — pins existing correct behaviour of `create()` (40.25).
- `test_update_preserves_fractional_expected_hours` — locks in the actual fix (40.0 → 40.25).
- `test_update_preserves_two_decimal_expected_hours` — additionally covers 2-decimal precision (38.0 → 38.75) to cover the `step=0.01`-Frontend-Inputs.

Modul-Anmeldung in `shifty_bin/src/integration_test.rs:1445` als `#[cfg(test)] mod employee_work_details_update;`.

**Verifikation**:
- `cargo check --workspace` — clean (33s).
- `cargo test --workspace` — 527 Tests grün (alle 3 neuen plus alle bestehenden).
- Revert-experiment (Fix temporär entfernt): die 3 neuen Tests werden rot mit `got 40` / `got 38`, bestätigt dass die Tests das Bug-Verhalten tatsächlich fangen.
- `cargo run` (kurz angetriggert) — Backend startet sauber, Datenbank-Migrations laufen, REST-Server bindet (lokal Port-Conflict mit laufendem Dev-Server, ist normal).

**Snapshot-Schema-Versioning**: KEIN Bump nötig. Der Fix verändert weder die `value_type`-Menge in `billing_period_sales_person`, noch die Berechnungsformel, noch die Quellspalten — er stellt nur sicher, dass die schon immer als `f32`/`FLOAT` deklarierte Spalte ihre Dezimalstellen behält. Snapshots, die unter dem Bug geschrieben wurden, waren bereits relativ zum Vertrag falsch und der Fix lässt sie unangetastet; eine Re-Berechnung würde dieselbe Drift produzieren wie heute. (Falls der User später entscheidet, dass alte Snapshots invalidiert werden sollen, ist das eine separate, manuelle Migration — nicht teil dieses Fixes.)

**Manual verification recommended** (User-Action): Nach jj-Commit + Restart die Reproduktion durchspielen (40.25 setzen, speichern, Reload, Wert muss erhalten bleiben).

**Commits**: NICHT auto-committed. Geänderte Dateien für jj-Commit:
- `dao_impl_sqlite/src/employee_work_details.rs` (1 Zeile, Bugfix)
- `shifty_bin/src/integration_test/employee_work_details_update.rs` (neue Datei, 3 Tests)
- `shifty_bin/src/integration_test.rs` (1 Zeile, Modul-Anmeldung)

**Follow-up empfohlen (kein Blocker)**: `dao_impl_sqlite/src/employee_work_details.rs:338` (`let version = entity.id.as_bytes().to_vec();` → `entity.version.as_bytes().to_vec()`) in `create()`. Eigenes Debug-Ticket sinnvoll, da das frontend-seitige Optimistic-Locking nach `create()` aktuell nur funktioniert, wenn Clients re-laden.
