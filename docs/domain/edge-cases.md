# Randfall-Referenz

Diese Datei ist die **zentrale Sammlung der scharfen Kanten** im
Shifty-System. Sie ist Pflichtlektüre, bevor man am Stundenkonto, an der
Balance-Rechnung, am Absence-System, an einer Snapshot-erzeugenden Operation
oder an Cross-Cutting-Themen (Auth, Transaktionen, Zeit) arbeitet.

Jeder Randfall ist kategorisiert:

- **[Verifiziert]** — Verhalten steht so im Code, mit Datei-Verweis.
- **[Konvention]** — Nicht im Code erzwungen, aber vom Team beschlossen.
- **[Zu prüfen]** — Vermutung; muss vor Änderung im Code geprüft werden.
- **[Bekannte Lücke]** — Wird aktuell **nicht** korrekt behandelt.

Für jede feature-spezifische Kante (z.B. "PDF-Export beim leeren Slot")
siehe die jeweilige [Feature-Doku](../features/README.md), Sektion "Randfälle".

---

## Inhaltsverzeichnis

1. [Stundenkonto (Balance-Rechnung)](#1-stundenkonto)
2. [Absence & Extra Hours](#2-absence--extra-hours)
3. [Billing Period & Snapshots](#3-billing-period--snapshots)
4. [Zeit & Zeitzone](#4-zeit--zeitzone)
5. [Rundung & Genauigkeit](#5-rundung--genauigkeit)
6. [Authentifizierung & Autorisierung](#6-authentifizierung--autorisierung)
7. [Transaktionen & Atomarität](#7-transaktionen--atomarität)
8. [Soft-Delete-Konsistenz](#8-soft-delete-konsistenz)
9. [Feature-Toggles & Stichtag-Rollouts](#9-feature-toggles--stichtag-rollouts)
10. [Migrations & sqlx-Offline-Cache](#10-migrations--sqlx-offline-cache)
11. [Frontend-Backend-Kopplung](#11-frontend-backend-kopplung)
12. [Clippy & Toolchain-Split](#12-clippy--toolchain-split)
13. [i18n & Textänderungen](#13-i18n--textänderungen)
14. [Export & Externe Integrationen](#14-export--externe-integrationen)

---

## 1. Stundenkonto

Das Stundenkonto (auch "Balance") berechnet die Differenz zwischen
tatsächlich gearbeiteten und vertraglich erwarteten Stunden, angereichert
um Extras (Urlaub, Krankheit, Feiertag, …).

**Kern-Formel (vereinfacht):**

```
balance = worked_hours − expected_hours + extra_hours_zählen_positiv
```

Der Devil steckt in `worked_hours`, `expected_hours` und der Frage, was
"positiv zählt".

### 1.1 Carryover-Grenze & Jahresrollover

**[Verifiziert]** Carryover-Hours persistieren Jahresend-Balance, damit
historische Zeiträume nicht rekalkuliert werden.
Ausgelöst durch `scheduler.rs:60,68` — `update_carryover_all_employees(year-1, Full)`
und `update_carryover_all_employees(year, Full)`.

- **Randfall — rückwirkende Änderung in einem abgeschlossenen Jahr:**
  Wenn eine Buchung oder Extra-Hours-Zeile in einem Jahr geändert wird, für
  das bereits ein Carryover-Wert steht, wird der Carryover **nicht automatisch
  invalidiert**. Das Live-Reporting zeigt die neue Wahrheit; der persistierte
  Carryover-Wert driftet. [Zu prüfen] Ob `carryover.rs` einen expliziten
  Re-Compute-Pfad hat.
  *Konsequenz:* Balance im Folgejahr kann inkonsistent wirken (Carryover-Wert
  passt nicht zur neu gerechneten Vorjahres-Balance).

- **Randfall — Sales Person startet mitten im Jahr:**
  Es gibt keinen synthetischen Carryover=0-Eintrag für Neueinstellungen — die
  Balance-Rechnung sollte den Startzeitpunkt (`sales_person.from` bzw.
  Vertragsbeginn aus `employee_work_details`) respektieren. [Zu prüfen] Ob
  der Reader tatsächlich vor Vertragsbeginn keine Erwartung anrechnet.

- **Randfall — Neuer Feiertag im abgeschlossenen Jahr:**
  Wird ein `special_day` in ein bereits abgeschlossenes Jahr eingetragen,
  ändert das die "gerechnete Expected" für dieses Jahr. Der Carryover-Wert
  bleibt statisch. → **Konvention:** Nie rückwirkend Special Days in
  abgeschlossene Jahre eintragen, es sei denn, Carryover wird manuell neu
  gerechnet.

### 1.2 Contract-Wechsel mitten im Zeitraum

- **Randfall — Wochenstunden ändern sich innerhalb einer Woche:**
  Sales Person hatte bis Mittwoch 20 h/Woche, ab Donnerstag 30 h/Woche.
  Wie werden die Expected-Hours für DIESE Woche gerechnet? Anteilig
  (Mo–Mi mit 20-h-Verteilung + Do–So mit 30-h-Verteilung) oder pauschal
  (welcher Vertrag am Wochenanfang galt)?
  [Zu prüfen] Konvention in `employee_work_details.rs` — Feld `from`/`to`
  auf Contract-Zeilen.

- **Randfall — Rückwirkende Vertragsänderung:**
  Contract-Zeile mit `from: 2024-01-01` wird in 2026 eingetragen. Live-View
  im Jahr 2024 driftet weg von persistierten Snapshots (Billing-Period und
  Carryover). Ohne Version-Bump ist der Diff unerklärlich für Validatoren.

- **Randfall — Contract-Lücke:**
  Keine `employee_work_details`-Zeile deckt einen Tag ab. [Zu prüfen] Was
  ist die Expected? 0? Fehler? Fallback auf letzte gültige Zeile?

### 1.3 Sales-Person-Zeitgrenzen

**[Zu prüfen]** `sales_person.from` und `sales_person.to` grenzen Aktivität
ein. Der genaue Filterpunkt (Reader, Writer oder beides) muss im Code
geprüft werden.

- **Randfall — Booking vor `sales_person.from`:**
  Wird bei Anlegen abgelehnt? Sichtbar im Read? Silent-Filter?
- **Randfall — Booking nach `sales_person.to`:**
  Ausgetretener Mitarbeiter, historisches Booking eingetragen. Erscheint es
  in Reports?

### 1.4 Special Days & Feiertage

- **[Verifiziert]** Special Days beeinflussen die `expected_hours`
  (Feiertag = 0 Erwartung; halber Tag = anteilig).
- **Randfall — Feiertag am Wochenende:**
  Wenn ohnehin keine Erwartung existiert (Kein-Arbeit-Wochenende), reduziert
  ein Feiertag nichts. Wird aber möglicherweise als "Feiertag angerechnet"
  fehlangezeigt? [Zu prüfen] `special_days.rs`-Reporting-Pfad.
- **Randfall — bewegliche Feiertage:**
  Ostern & Co. werden über `special_days`-Tabellen-Einträge abgebildet,
  nicht algorithmisch. → **Konvention:** Am Jahresbeginn müssen bewegliche
  Feiertage manuell eingetragen werden (oder ein Script tut das).
- **Randfall — Special Day nach Billing-Period-Snapshot:**
  Snapshot bleibt fix (siehe [§3](#3-billing-period--snapshots)), Live-View
  zeigt den neuen Wert. Diff-View im UI muss diesen Fall verständlich machen.

### 1.5 Balance-Perimeter — Was zählt zur Balance?

Nicht alle Extra-Hours-Kategorien zählen gleich in die Balance. Aus
`service/src/extra_hours.rs` gibt es folgende Kategorien:

**[Verifiziert]** in `ExtraHoursCategory`:
`ExtraWork`, `Vacation`, `SickLeave`, `Holiday`, `Unavailable`,
`UnpaidLeave`, `VolunteerWork`, `CustomExtraHours(id)`.

Und in `ExtraHoursReportCategory` (Reporting-Layer) zusätzlich:
`Shiftplan` (aus Bookings abgeleitet).

- **Randfall — `UnpaidLeave` zählt anders:**
  Unbezahlter Urlaub reduziert die *Erwartung*, aber addiert nichts auf die
  *Ist-Seite*. Andere Kategorien (Urlaub, Krankheit) tun beides. Jede neue
  Kategorie muss diese Semantik explizit erklären.
- **Randfall — Custom-Kategorie ohne Semantik-Definition:**
  Wenn eine Custom-Kategorie angelegt wird, ohne die Reporting-Behandlung
  festzulegen, ist das Resultat implementations-abhängig. [Zu prüfen]
  Defaults in `custom_extra_hours.rs`.

### 1.6 Runde Zahlen, unrunde Realität

Siehe [§5 Rundung & Genauigkeit](#5-rundung--genauigkeit).

---

## 2. Absence & Extra Hours

### 2.1 Cutover-Historie

**[Verifiziert per CLAUDE.md]** Das Absence-System (v1.0+) ist range-basiert
und ersetzt Single-Day-Extra-Hours **nach Cutover**. Der Cutover-Zeitpunkt
ist ein Datum, ab dem neue Urlaubs-/Krank-/Unbezahlte-Zeilen im
`absence`-Aggregat landen, während vorherige noch als `extra_hours`
existieren.

- **Randfall — Periode überspannt den Cutover:**
  Alte Zeilen liegen in `extra_hours`, neue in `absence`. Der Report muss
  aus **beiden** Tabellen aggregieren. Wer einen der beiden Pfade vergisst,
  zeigt zu wenig / doppelt.
- **[Verifiziert per Memory]** Bei Toggle-Stichtag-Rollout (D-51-07, HCFG-02)
  muss pro Konsumkette die alte Semantik vor Feature-Einführung im
  Gate-aus-Zweig rekonstruiert werden — nicht blind "None → raw" annehmen.

### 2.2 Range-Randfälle im Absence-System

- **Randfall — Absence spannt zwei Billing-Perioden:**
  Wie wird gesplittet? Anteilig auf beide oder komplett in die Start-Periode?
  [Zu prüfen] `absence_conversion.rs` und Reporting-Aufruf.
- **Randfall — Absence spannt Jahreswechsel:**
  Der Anteil vor dem 31.12. muss in den Carryover einfließen. Wenn Carryover
  vor dem Absence-Insert berechnet wurde, fehlt der Anteil.
- **Randfall — Zwei Absences überlappen:**
  Urlaub 01.-15.06., Krank 10.-12.06. Was zählt in den überlappenden Tagen?
  Krank hat üblicherweise Vorrang. [Zu prüfen] ob der Konfliktlogic
  automatisch splittet oder eine Fehlermeldung wirft.
- **Randfall — Absence auf Nicht-Arbeitstag:**
  Urlaub Sonntag beantragt. Zählt zu 0h? Zu erwarteten Stunden (wenn Kontrakt
  Sonntagsarbeit vorsieht)?
- **Randfall — Absence gegen Booking-Konflikt:**
  Existierende Buchung an einem Tag, dann Absence für denselben Tag.
  Was passiert? Booking bleibt und wird ignoriert? Fehler bei Absence?
  Beide bleiben (Doppelzählung)? [Zu prüfen] `absence.rs`-Service-Logik.

### 2.3 Legacy Extra Hours — Delete-Semantik

- **Randfall — `extra_hours`-Zeile löschen, die schon in Snapshot ist:**
  Snapshot bleibt fix (er persistiert das Aggregat, nicht die Einzelzeilen).
  Live-View driftet. Ohne Version-Bump ist der Diff nicht als "Delete"
  identifizierbar.

---

## 3. Billing Period & Snapshots

### 3.1 Der Snapshot-Vertrag

**[Verifiziert]** `service_impl::billing_period_report::CURRENT_SNAPSHOT_SCHEMA_VERSION`
ist eine `pub const u32` (aktuell **12**, siehe `billing_period_report.rs:117`).

Jede geschriebene `billing_period`-Zeile trägt diesen Wert.
`build_and_persist_billing_period_report()` schreibt ihn (siehe
`billing_period_report.rs:390`).

### 3.2 Bump-Regel

**Bumpe die Version um 1 wenn du:**

- einen neuen persistierten `value_type` zu `billing_period_sales_person`
  hinzufügst (Enum-Erweiterung → Row-Write),
- einen bestehenden `value_type` entfernst / umbenennst,
- die Berechnung eines existierenden `value_type` änderst (andere Formel,
  andere Inputs, anderes Filtering),
- den Input-Set änderst, den die Berechnung liest (z.B. neue
  `extra_hours`-Kategorie mit-aggregierst).

**Bumpe NICHT wenn du:**

- rein additive Änderungen machst, die keinen `value_type` berühren
  (neue REST-Endpoints, Frontend-Änderungen, neue Felder auf unrelated
  Tabellen).

### 3.3 Randfälle im Umgang mit Snapshots

- **Randfall — Alter Snapshot (v11) wird auf Code v12 gelesen:**
  Der Validator (`billing_period_report.rs`) muss diese Version-Diskrepanz
  erkennen und entweder re-compute skippen oder die alte Formel emulieren.
  [Verifiziert per CLAUDE.md] Das ist Sinn des Version-Felds.
- **Randfall — Developer vergisst Bump nach Formeländerung:**
  Validator interpretiert Alt-Snapshot als "same rules" und findet einen
  Diff. Der wird fälschlich als Datenbug gemeldet. → Pattern: PR-Review
  auf Änderungen in `billing_period_report.rs`-Berechnung achten und
  Version-Bump aktiv suchen.
- **Randfall — Snapshot wird während laufender Perioden-Änderung erzeugt:**
  Race zwischen "Booking wird angelegt" und "Snapshot wird gefahren".
  → Snapshot-Erzeugung MUSS unter einer TX laufen, die den Read-Set
  konsistent hält. [Zu prüfen] TX-Verhalten in
  `build_and_persist_billing_period_report`.
- **Randfall — Kein Snapshot vorhanden, aber Report wird angefragt:**
  Live-Rechnung greift. Zeigt das UI kenntlich, dass das kein
  eingefrorener Wert ist? [Zu prüfen] Rest-DTO-Feld für "is_snapshot".
- **Randfall — Snapshot mit falscher Version fürs Feature-Flag-Regime:**
  Wenn ein Toggle die Semantik ändert, MUSS der Snapshot-Version bumpen —
  sonst ist er semantisch mehrdeutig (siehe [§9](#9-feature-toggles--stichtag-rollouts)).

---

## 4. Zeit & Zeitzone

- **Randfall — Sommerzeit-Umschaltung:**
  Mär/Okt-DST-Wechsel. Booking von 02:00 bis 04:00 in der Frühlings-Nacht:
  entweder 1h effektiv (weil 02:00 → 03:00 gesprungen) oder 2h (naive
  Rechnung). [Zu prüfen] Ob Zeitrechnung UTC-basiert oder naive-local.
- **Randfall — SQLite speichert:**
  SQLite ist zeitzonenlos. Die Konvention ist [Zu prüfen] — vermutlich UTC
  oder Berlin-Local. Wichtig zu wissen, wenn ein Client die Werte anzeigt.
- **Randfall — Woche über Jahreswechsel:**
  KW 1 des neuen Jahres beginnt oft schon im Dezember (ISO 8601). Wenn ein
  Report auf "KW 1/2026" verweist, müssen die Boundaries eindeutig sein.
  [Zu prüfen] Nutzung von `iso_week_number` in `datetime_utils.rs`.
- **Randfall — Schaltjahr 29. Februar:**
  Jahresrechnungen (Erwartung "pro Tag" × 365) müssen 366 abfangen.
- **Randfall — Ein-Tag-Absence per DateRange:**
  Halboffener vs geschlossener Range. `[from, to)` oder `[from, to]`?
  Off-by-one bei "3 Tage Urlaub" trivial verursacht.

---

## 5. Rundung & Genauigkeit

- **Randfall — Float-Präzision:**
  `f32` (wie in `ExtraHours::amount` und `WorkingHoursDay::hours`) verliert
  Bits bei Summen über viele Zeilen. 100 × 0.1 = 9.9999… bei f32.
  Für Präzisionsanzeige besser vor Summation multiplizieren oder
  Rational-Arithmetik nutzen.
- **Randfall — Rundung ≠ Assoziativ:**
  `round(a + b + c) ≠ round(a) + round(b) + round(c)`. Wenn UI die
  gerundete Einzelanzeige summiert, weicht sie von der Backend-Summe ab.
  → **Konvention:** Immer die Backend-Gesamtsumme anzeigen, nie im Client
  aus gerundeten Einzelwerten neu addieren.
- **Randfall — Anzeige-Rundung vs Persistenz-Rundung:**
  Anzeige mit einer Nachkommastelle, Persistenz mit f32/vier Stellen. Der
  User sieht "1.2h", Snapshot speichert "1.234h". Beim Vergleich zwischen
  Perioden wirken kleine Differenzen groß.

---

## 6. Authentifizierung & Autorisierung

### 6.1 `Authentication::Full`-Bypass

**[Verifiziert]** In `service_impl/src/permission.rs` gibt `Authentication::Full`
für alle Permission-Prüfungen früh `Ok(())` zurück (`permission.rs:28,41,63,80,90`).

Das ist der Weg, wie **interne Aggregate** (Business-Logic-Services) Basic-Services
konsumieren, ohne dass jeder einzelne Read-Call einen User-Context durch die
Kette schiebt.

- **[Verifiziert per Memory]** Phase 51 (Toggle Full-Context-Bypass):
  Der ToggleService (`service_impl/src/toggle.rs`) hatte einen Guard, der
  Full-Reads verhinderte; das brach interne Aggregat-Aufrufer (Reporting,
  Booking-Information rufen mit Full). Die Reads sind auf Full ausgenommen.
- **Randfall — Ein neuer Service kopiert das Read-Handling und vergisst den
  Full-Bypass:**
  Interne Aggregate scheitern still (oder mit Permission-Fehler). Reporting
  driftet.
- **Randfall — Ein REST-Endpoint bekommt Full statt User-Context:**
  Katastrophaler Bug. `Full` ist **ausschließlich für interne Calls**.
  Alle REST-Handler MÜSSEN den vom Session-Layer bereitgestellten User-Context
  weiterreichen.

### 6.2 OIDC / Mock-Split

- **Randfall — Mock in Dev, OIDC in Prod:**
  Test-Coverage-Gap: RBAC-Deny-Pfade werden in Dev nie durchlaufen, weil
  Mock immer Admin. → Explizite Unit-Tests mit "kein Admin, nur Rolle X"
  sind Pflicht.
- **Randfall — OIDC-Token-Expiry mitten im Request:**
  [Zu prüfen] `session.rs`-Reaktion. Wird refreshed? 401 zurückgegeben?
- **Randfall — Rollenänderung während laufender Session:**
  User verliert eine Rolle, das Frontend zeigt aber noch entsprechende
  Buttons. Klick → 403. [Zu prüfen] Ob Frontend ein Refresh-on-403-Muster
  hat.

### 6.3 User Invitation

- **Randfall — Invitation-Link mehrfach eingelöst:**
  [Zu prüfen] `user_invitation.rs` — Session-Revoke-Semantik
  (`20251020000000_add-session-revoked-at-to-user-invitation.sql` existiert).

---

## 7. Transaktionen & Atomarität

### 7.1 Das `Option<Transaction>`-Pattern

**[Verifiziert per CLAUDE.md]** Jede Service-Methode akzeptiert
`Option<Self::Transaction>`. Wenn `None`, öffnet der Service selbst eine
TX und committet am Ende.

```rust
async fn do_something(&self, tx: Option<Self::Transaction>) -> Result<T, ServiceError> {
    let tx = self.transaction_dao.use_transaction(tx).await?;
    // ... business logic ...
    self.transaction_dao.commit(tx).await?;
    Ok(result)
}
```

- **Randfall — Nested `commit`:**
  Wenn ein äußerer Aufrufer die TX besitzt und ein innerer Service naiv
  committet, ist die TX halbfertig committet. `use_transaction` muss diesen
  Fall abfangen (Muster: intern kein commit, wenn tx vorher `Some` war).
  [Zu prüfen] Implementierung in `transaction_dao`.
- **Randfall — Rollback bei Panik:**
  Wenn zwischen `use_transaction` und `commit` ein Panik-Pfad läuft (nicht
  ein `Result::Err`), gibt es dann sauberes Rollback? [Zu prüfen] Drop-Impl.

### 7.2 Re-Point-Atomarität

**[Verifiziert per Memory]** Bei Daten-Umzügen/Re-Points (Slot-Split,
Booking-Migration): alles in EINER Transaktion (Rollback) + harte Tests
gegen Doppelzählung in Reports/Balance. Phase 23 hat das schmerzlich
gelernt.

- **Randfall — Slot-Split ohne Booking-Migration in derselben TX:**
  Zwischenzustand: Slot A hat "gesplittet"-Marker, aber Bookings hängen noch
  am alten Slot. Report zeigt Bookings 2× oder gar nicht.
- **Randfall — Booking-Umzug scheitert mid-way:**
  Bookings 1-5 umgezogen, Booking 6 failt. Ohne Rollback: inkonsistenter
  Zustand. Mit Rollback: alle 6 bleiben am Ursprungsslot — konsistent.

### 7.3 SQLite als Single-Writer

- **Randfall — Zwei parallele Writes:**
  SQLite serialisiert. Der zweite Write kann `BUSY`/`LOCKED` bekommen.
  [Zu prüfen] Retry-Verhalten und Timeout.
- **Randfall — Long-running TX blockiert alle anderen:**
  Ein Report, der in einer TX läuft und lange dauert, hält Writer auf.
  → **Konvention:** Read-Ops nur dann in einer TX-übergreifenden
  Transaktion, wenn's der Konsistenz dient.

---

## 8. Soft-Delete-Konsistenz

**[Verifiziert per CLAUDE.md]** Alle Reader-Queries filtern
`WHERE deleted IS NULL`.

- **Randfall — Neue Query ohne Filter:**
  Ghost-Rows im Report. Der Filter ist eine Konvention, keine
  strukturelle Sperre. → **Review-Regel:** Neue `query!/query_as!` auf
  `deleted IS NULL` prüfen.
- **Randfall — Aggregat-Parent gelöscht, Children bleiben:**
  Sales Person soft-deletet — was passiert mit seinen Bookings, Absences,
  Extra-Hours? [Zu prüfen] Ob Cascade-Soft-Delete existiert. Ohne
  Cascade: Bookings hängen orphan, Reports können sie noch sehen, wenn
  der Reader auf `sales_person.deleted IS NULL` verzichtet.
- **Randfall — Foreign-Key auf gelöschten Row:**
  SQLite prüft FKs nur, wenn aktiviert. Auf soft-deleted Rows kommt
  keine FK-Verletzung. Silent bad data möglich.

---

## 9. Feature-Toggles & Stichtag-Rollouts

**[Verifiziert per Memory]** Toggle-Stichtag-Features (z.B. D-51-07,
HCFG-02) haben pro Konsumkette einen Gate-aus-Zweig, der die alte
Semantik vor Feature-Einführung rekonstruiert. Nicht blind "None → raw"
annehmen.

### 9.1 Rollout-Randfälle

- **Randfall — Alte Daten unter neuem Toggle:**
  Toggle "an" — Daten vor dem Stichtag müssen unter der alten Semantik
  gerechnet werden, sonst verfälscht die Vergangenheit. Der Gate-aus-Zweig
  in jeder Konsumkette ist Pflicht.
- **Randfall — Toggle zur Laufzeit umgeschaltet:**
  Snapshot war "aus", wird jetzt "an" gelesen. Ohne Snapshot-Version-Bump
  ist das Verhalten mehrdeutig.
- **Randfall — Toggle-Read unter zwei verschiedenen Auth-Kontexten:**
  Reporting ruft mit Full, REST-Handler mit User. Wenn der Toggle-Read
  unterschiedlich antwortet, driftet die Rechnung von der Ansicht.
  → Phase 51 hat sichergestellt, dass Full den Read passieren lässt.

### 9.2 Feature-Flags vs Toggles

Es gibt zwei Mechanismen:

- **`feature_flag`** (siehe `service/src/feature_flag.rs`) —
  vermutlich Compile-Time-orientiert oder Boolean-Store.
- **`toggle`** (siehe `service/src/toggle.rs`) — mit User- und
  Datums-Kontext.

[Zu prüfen] Genaue Semantik-Trennung — siehe
[`../features/F13-system-infrastructure.md`](../features/F13-system-infrastructure.md).

---

## 10. Migrations & sqlx-Offline-Cache

**[Verifiziert per Memory]** CI nutzt `SQLX_OFFLINE=true` + `cargo test`.
Nach jeder neuen `query!/query_as!`-Verwendung MUSS
`cargo sqlx prepare --workspace` laufen und der `.sqlx/`-Cache mitcommittet
werden. Inkrementeller Build kann grün sein, `--doc`-Target/Clean-Build/CI
failt. Phase 33 hat das gefunden.

- **Randfall — Query mit dynamischem SQL:**
  `sqlx::query_with(&format!(…))` wird nicht vom Offline-Check erfasst.
  Silent-Kompilations-OK, Runtime-Fehler.
- **Randfall — Migration entfernt Spalte, Query greift noch drauf zu:**
  Kompiliert, wenn `.sqlx/`-Cache nicht neu erzeugt wurde. → Nach
  Migration IMMER `sqlx prepare` neu.
- **Randfall — Zwei Devs, verschiedene DB-States:**
  `.sqlx/`-Cache im Commit ist verbindlich. Lokale DB muss auf State passen.
  → `sqlx migrate run`, nicht mehrere Migrations mischen.

---

## 11. Frontend-Backend-Kopplung

**[Verifiziert per Memory]** Neue Backend-Routes brauchen `[[web.proxy]]`
in `shifty-dioxus/Dioxus.toml` — sonst 404 im `dx serve`-Dev-Modus.
Phase 28 + 49 haben's beide vergessen.

- **Randfall — Neuer Endpoint ohne Proxy-Eintrag:**
  Prod funktioniert (statisches Bundle proxied vom Reverse-Proxy),
  Dev nicht. Reproduktion in Dev unmöglich, bis Proxy nachgezogen wird.
- **Randfall — DTO-Feld-Änderung ohne WASM-Rebuild:**
  Frontend hält Cache von altem DTO. Deserialisierung failt still oder
  wirft im Browser-Console-Log.
- **Randfall — dx-CLI-Version-Drift:**
  [Verifiziert per Memory] shifty-dioxus braucht dx 0.6.x (Crate dioxus
  0.6.3). nixpkgs rollte auf 0.7.x → App startet nicht, Design gestrippt.
  Im `flake.nix` gepinnt. Style-Pfad `Dioxus.toml`: `/assets/tailwind.css`.
- **Randfall — Backend-Roundtrip in Frontend-Phasen nicht getestet:**
  [Verifiziert per Memory] Frontend-Phasen mit "Backend existiert bereits"-
  Annahme MÜSSEN im Browser e2e verifiziert werden. Create-Pfad ≠ Edit-Pfad
  (Phase 23: `modify_slot` ließ `max_paid_employees` fallen).

---

## 12. Clippy & Toolchain-Split

**[Verifiziert per Memory]** `nix build` erzwingt
`cargo clippy -- --deny warnings`; `cargo test`/`build` und lokale CI tun das
NICHT. Jedes Phase-Gate MUSS `cargo clippy --workspace -- -D warnings`
zusätzlich fahren.

- **Randfall — Test lokal grün, `nix build` failt:**
  Häufigster Fall: Clippy findet Warnings, die `cargo build` ignoriert.
- **Randfall — Frontend-Workspace-Clippy:**
  [Verifiziert per Memory] shifty-dioxus ist eigener Workspace, aus
  CI-Clippy ausgeschlossen (~198 pre-existing Lints). Clippy im
  dioxus-Shell kaputt (E0514), muss aus Backend-Shell laufen.
  → Neue Lints im dioxus-Bereich driften unbemerkt.
- **Randfall — `#[allow(…)]`-Wildwuchs:**
  Wenn Clippy-Findings unterdrückt statt behoben werden, akkumulieren
  sie sich. Kein Gate fängt das.

---

## 13. i18n & Textänderungen

**[Verifiziert per CLAUDE.md]** Neuer Text braucht Übersetzung in alle drei
Locales: **En, De, Cs**.

- **Randfall — Neuer Text nur in einer Sprache:**
  Fallback-Anzeige zeigt Key-Namen oder Leerstring. [Zu prüfen] genaues
  Fallback-Verhalten.
- **Randfall — Plural-Formen:**
  Deutsch hat andere Plural-Regeln als Englisch; Tschechisch hat mehrere
  Plural-Formen (1, 2-4, 5+). [Zu prüfen] Ob i18n-Framework Plural-Rules
  unterstützt.
- **Randfall — Textvariable in falscher Reihenfolge:**
  Deutsch stellt Subjekt/Objekt anders. Wenn Frontend Segmente konkateniert,
  bricht Deutsch/Tschechisch. → Immer Full-Sentence-Templates, nie
  Fragment-Konkatenation.

---

## 14. Export & Externe Integrationen

### 14.1 PDF-Export

- **Randfall — Sales Person mit sehr langem Namen:**
  Layout-Overflow? [Zu prüfen] PDF-Renderer-Verhalten in `pdf_render.rs`.
- **Randfall — Zeitraum mit 0 Bookings:**
  Leere Seite? Gar keine Seite? → Kunde erwartet visuelles Feedback, nicht
  leere Datei.
- **Randfall — Special-Days-Overlay:**
  Feiertag in einem Slot; wird der als "leer" gezeichnet oder mit
  Feiertag-Markierung?
- **Randfall — Scheduler-getriebener PDF-Export:**
  `pdf_export_scheduler.rs` fährt zeitgesteuert. Was passiert bei
  Ausfall des Scheduler-Ticks? [Zu prüfen] Recovery-Verhalten.

### 14.2 iCal

- **Randfall — Zeitzone im iCal:**
  iCal ist streng über TZ-Definitionen. Wenn Backend UTC oder Local
  serialisiert, muss der TZID-Block passen. Sonst zeigt der Kalender das
  Event zur falschen Uhrzeit.
- **Randfall — Wiederkehrende Events:**
  Recurrence-Rules (RRULE) korrekt? [Zu prüfen] `ical.rs`.

### 14.3 WebDAV

- **Randfall — Auth-Fehler beim WebDAV-Upload:**
  `webdav_client.rs` überträgt PDF-Exporte an einen Cloud-Speicher.
  Netzwerk-Fehler, 401, 507 (Insufficient Storage). Retry? Log?
  User-facing Fehler?

---

## Meta-Randfall: Neue Randfälle finden

Wenn du auf einen weiteren Randfall stößt, der hier nicht dokumentiert
ist:

1. Trage ihn in dieselbe Sektion ein.
2. Markiere ihn mit **[Zu prüfen]** bis du im Code verifiziert hast.
3. Verlinke bei Bedarf auf die Feature-Doku, in der die Behandlung
   konkret sitzt.

Die Randfall-Referenz altert. Halte sie mit dem Code lebendig — sonst
wird sie zur Falle.
