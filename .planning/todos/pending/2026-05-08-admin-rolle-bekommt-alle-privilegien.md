---
created: 2026-05-08T10:23:57.816Z
title: admin-Rolle automatisch alle Privilegien (Wildcard oder Trigger)
area: auth
files:
  - migrations/sqlite/20240426150045_user-roles.sql
  - dao_impl_sqlite/src/lib.rs:78
  - service/src/permission.rs
---

## Problem

Default-`admin`-Rolle bekommt in Migration `20240426150045_user-roles.sql` nur einen
Snapshot der damals existierenden Privilegien (`admin`, `sales`, `hr`). Nachfolgende
Migrationen, die neue Privilegien anlegen (`shiftplan.edit`, `cutover_admin`, etc.),
müssen zwingend daran denken, der `admin`-Rolle das neue Privileg in derselben
Migration zuzuordnen — sonst hat der Default-Admin Lücken.

Gefundene Lücke 2026-05-08: `cutover_admin` (Migration `20260503000003_add-cutover-
admin-privilege.sql`) wurde der `admin`-Rolle nie zugeordnet. DEVUSER konnte
trotz `admin`-Rolle keine Cutover-Endpoints aufrufen, bis manuell ein
`INSERT INTO role_privilege ('admin', 'cutover_admin', ...)` ausgeführt wurde.

Generell: Der User-Wunsch ist "der admin sollte alle Rollen haben". Aktuelles Pattern
(snapshot-basiert) ist fragil.

## Solution

Drei Optionen, eine wählen + umsetzen:

**A — Code-side Wildcard:** Im `has_privilege()`-DAO/Service einen Shortcut: wenn
User die Rolle `admin` hat → `true` für jedes Privileg. Vorteil: keine Migrationen
mehr nötig, immer aktuell. Nachteil: Audit-Log zeigt nicht, welche Privilegien
explizit gewährt waren; Production-Sicherheit muss mit `admin = god mode` leben.

**B — DB-Trigger:** SQL-Trigger auf `INSERT INTO privilege` der automatisch ein
`role_privilege ('admin', NEW.name, 'admin-trigger')` einfügt. Vorteil: explicit data,
Audit-Log korrekt. Nachteil: SQLite-Trigger auf seed-Migrationen kann Race-Conditions
mit Carryover-Backfill triggern; muss getestet werden.

**C — Idempotente Sync-Migration + Convention:** Eine permanente Migration die
`INSERT OR IGNORE INTO role_privilege SELECT 'admin', name, 'admin-sync' FROM
privilege` ausführt. Konvention: bei jedem privilege-Insert in einer neuen Migration
eine matching role-grant-Zeile hinzufügen. Vorteil: explicit, keine Magie. Nachteil:
Konvention nicht enforceable, kann wieder vergessen werden.

**Empfohlen: A.** Einfachster, robust, deckt User-Wunsch ("admin hat alle Rollen")
exakt ab. Wenn Production-Audit später feiner gewünscht: B als Phase-2-Verfeinerung.

## Acceptance Criteria

- [ ] Implementierung einer der drei Optionen (vermutlich A)
- [ ] Test: neuer User mit Rolle `admin` hat alle Privilegien (current + future)
- [ ] Test: User mit anderer Rolle ist NICHT betroffen (kein "wildcard escape")
- [ ] Migration die alle aktuell verwaisten Privilegien-Mappings für `admin` nachzieht
- [ ] OpenAPI-surface-Test bleibt grün
