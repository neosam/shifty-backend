-- Phase 8.5 (D-04): Entferne cutover_run_id aus absence_period_migration_source.
-- Der Backlink ueberlebt 8.6 (CutoverDao wird subtraktiv geloescht), aber loesgeloest
-- von Cutover-Run-Semantik. Prod/INT haben 0 Rows (CONTEXT.md Fakt 2026-06-09) -> DROP+RECREATE safe.
DROP TABLE IF EXISTS absence_period_migration_source;
CREATE TABLE absence_period_migration_source (
    extra_hours_id    BLOB(16) NOT NULL PRIMARY KEY,
    absence_period_id BLOB(16) NOT NULL,
    migrated_at       TEXT NOT NULL
);
