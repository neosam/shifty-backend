-- Phase 8 Plan 08-07 Gap-Closure (Task 1):
-- Stellt sicher, dass die `admin`-Rolle dauerhaft jedes Privileg im System hält.
-- Pattern: AFTER-INSERT-Trigger auf `privilege` + initialer idempotenter Backfill
-- für aktuell verwaiste Privilegien (z.B. `cutover_admin`, `feature_flag_admin`,
-- die in vorherigen Migrationen angelegt, der `admin`-Rolle aber nie zugeordnet
-- wurden).
--
-- Damit ist der DEVUSER und jeder Admin-User in Production nach der nächsten
-- Migration mit Vollzugriff ausgestattet, ohne dass jede neue Privilege-Migration
-- daran denken muss, parallel das Mapping nach `admin` zu pflegen.
--
-- Idempotenz:
-- - Backfill nutzt `INSERT OR IGNORE` (zusammen mit dem UNIQUE-Constraint auf
--   `(role_name, privilege_name)` aus 20240426150045_user-roles.sql).
-- - Trigger-Body nutzt ebenfalls `INSERT OR IGNORE` für den Fall, dass eine
--   spätere Migration ein Privileg + admin-Mapping in derselben Transaktion
--   anlegt; der Trigger feuert dann ohne den vorhandenen Eintrag zu duplizieren.

-- Backfill: Alle bereits existierenden Privilegien an die admin-Rolle binden.
INSERT OR IGNORE INTO role_privilege (role_name, privilege_name, update_process)
SELECT 'admin', name, 'admin-auto-grant-backfill' FROM privilege;

-- Forward: Jedes künftig per INSERT angelegte Privileg wird automatisch an
-- die admin-Rolle gebunden. SQLite-Trigger feuern AFTER INSERT ON <table>
-- standardmäßig FOR EACH ROW (FOR EACH STATEMENT existiert in SQLite nicht).
CREATE TRIGGER privilege_auto_grant_admin
  AFTER INSERT ON privilege
  BEGIN
    INSERT OR IGNORE INTO role_privilege (role_name, privilege_name, update_process)
    VALUES ('admin', NEW.name, 'admin-auto-grant-trigger');
  END;
