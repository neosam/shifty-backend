# Erste Woche als Dev auf Shifty

Ein pragmatischer Leitfaden vom leeren Laptop zum ersten Merge.

## Tag 1 — Setup

### Toolchain

- **Rust:** Stable, via `rustup`. Der Backend-Workspace pinnt Version
  in der `rust-toolchain.toml` (falls vorhanden).
- **Nix:** Für reproduzierbare Builds. `nix develop` gibt dir eine
  Shell mit allen Tools (`sqlx`, `cargo-watch`, etc.).
- **`sqlx-cli`:** In der `nix develop`-Shell verfügbar. Wenn du außerhalb
  arbeitest: `cargo install sqlx-cli`.
- **`dx` (Dioxus CLI):** Version **0.6.x** — im `flake.nix` gepinnt.
  Nicht auf 0.7.x updaten, sonst startet das Frontend nicht.

### Repo klonen

```bash
git clone <repo> && cd shifty-backend
```

Das ist der aktive Monorepo. Nicht das externe `shifty-dioxus/` klonen —
das ist Legacy.

### Backend starten

```bash
cd shifty-backend
cp env.example .env         # anpassen wenn nötig
nix develop                 # Shell mit Tools
sqlx database reset --source migrations/sqlite  # ⚠ destruktiv, nur first-run
cargo run                   # Backend auf Port 3000
```

**Vorsicht:** `sqlx database reset` löscht die DB. Für inkrementelle
Migrations später: `sqlx migrate run --source migrations/sqlite`.

### Frontend starten

In einem zweiten Terminal:

```bash
cd shifty-backend/shifty-dioxus
npx tailwindcss -i ./input.css -o ./assets/tailwind.css --watch  # Terminal A
# In Terminal B:
dx serve --hot-reload       # Frontend auf Port 8080
```

Öffne `http://localhost:8080`. Im Dev-Modus (`mock_auth`) bist du
automatisch als Admin eingeloggt.

### Sanity-Check

- Öffne den Shiftplan.
- Öffne Employees.
- Wenn beide Seiten was zeigen, läuft der Stack.

## Tag 2 — Architektur verstehen

Lies in dieser Reihenfolge:

1. [Layered Architecture](../architecture/01-layered.md) — Warum REST →
   Service → DAO → SQLite.
2. [Service Tiers](../architecture/02-service-tiers.md) — Basic vs
   Business-Logic. Sehr wichtig, damit du keine zyklischen Deps baust.
3. [Transactions](../architecture/05-transactions.md) — Das
   `Option<Transaction>`-Pattern.
4. [Auth](../architecture/04-auth.md) — v.a. der `Full`-Bypass.
5. [Testing](../architecture/07-testing.md) — Mockall, In-Mem-SQLite,
   Clippy-Gate.

## Tag 3 — Fach verstehen

Lies:

1. [Glossary](../domain/glossary.md) — die Begriffe.
2. [Time Accounting](../domain/time-accounting.md) — wie die Balance
   rechnet. Kernwissen.
3. [Billing Period](../domain/billing-period.md) — der Snapshot-Vertrag.
4. [Absence System](../domain/absence-system.md) — v1.0+ Range-basiert.

Dann klick die Frontend-Pages durch und verstehe, welches Fach hinter
welchem UI steht.

## Tag 4 — Erste kleine Änderung

Wähl dir eine kleine Aufgabe — z.B. ein Text-Label ändern oder ein
kleines Feld hinzufügen. Nutz den Flow:

1. `cargo build --workspace` — sicherstellen, dass alles kompiliert.
2. Änderung machen.
3. `cargo build && cargo test` — grün?
4. `cargo clippy --workspace -- -D warnings` — Clippy grün? (**Pflicht**,
   sonst failt `nix build`.)
5. Wenn du SQL geändert hast: `cargo sqlx prepare --workspace` und
   `.sqlx/` mitcommitten.
6. `jj commit -m "..."` — dieses Repo läuft auf jj (siehe
   `CLAUDE.local.md`).
7. Push.

## Tag 5 — Konventionen und Randfälle

- [Edge Cases](../domain/edge-cases.md) — **einmal komplett durchlesen**.
  Nicht auswendig lernen, aber wissen, wo du sie findest.
- Root-`CLAUDE.md` — die Kurzform der Konventionen.
- `shifty-backend/CLAUDE.md` — Backend-Spezifika (Service-Tier-Regeln,
  Snapshot-Versioning-Vertrag, Clippy-Gate).

## Häufige Fehler in Woche 1

1. **`cargo test` läuft, `nix build` failt.** Fast immer Clippy-Warnings.
   Lauf `cargo clippy --workspace -- -D warnings` vor dem Push.
2. **Neuer Endpoint gibt 404 im Dev.** Du hast den Backend-Endpoint
   angelegt, aber `shifty-dioxus/Dioxus.toml` hat keinen
   `[[web.proxy]]`-Eintrag dafür.
3. **Neuer Query, Build failt in CI.** Du hast `query!`/`query_as!`
   verwendet, aber vergessen `cargo sqlx prepare --workspace` zu laufen.
4. **UI-Änderung sieht gut aus, aber Backend hat nichts.** Der
   Roundtrip wurde nicht getestet — Edit-Pfad ist nicht Create-Pfad.
   Testen.
5. **Auth-Fehler nach mock_auth-Wechsel.** OIDC-Modus hat andere
   Rollen. Wenn du Deny-Pfade testen willst: expliziter Unit-Test mit
   Auth-Context ohne Admin.
6. **`Authentication::Full` in einem REST-Handler.** Katastrophaler
   Auth-Bypass. Nie machen.

## Wenn was hängt

- **Backend startet nicht:** `.env` fehlt? DB-URL falsch? Migrations
  nicht gelaufen?
- **Frontend zeigt weiße Seite:** Tailwind-Watcher läuft nicht? dx-CLI
  auf 0.7.x? Style-Pfad in `Dioxus.toml` falsch?
- **Test hängt:** In-Memory-DB nicht geleaked; wahrscheinlich eine TX,
  die nicht committet.

## Slash-Commands

Das Repo nutzt GSD (`.planning/`). Wenn du in Claude Code arbeitest,
sind Skills wie `/gsd-progress`, `/gsd-plan-phase`, `/release-version`
verfügbar. `/gsd-progress` gibt dir einen Situationsbericht.
