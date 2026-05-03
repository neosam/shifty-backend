# Phase 04 — Deferred Items

Außerhalb des Scopes der jeweiligen Plans entdeckt, aber pre-existing
und nicht durch die Plan-Aktionen verursacht (per `<deviation_rules>`
Scope-Boundary).

## Pre-existing: Lokale `localdb.sqlite3`-Provisionierung (D-Phase4-15)

- **Lokale `localdb.sqlite3`-Provisionierung** (D-Phase4-15): Beim Updaten auf Phase 4 müssen lokale Dev-Datenbanken neu provisioniert werden (`rm localdb.sqlite3 && nix-shell --run 'sqlx setup --source migrations/sqlite'`). Die alte localdb fehlt noch alle Phase-1..4-Migrations und alle Phase-2-Seeds. Kein Code-Fix nötig — lokal pro Dev.
