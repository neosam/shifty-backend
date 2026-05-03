-- Phase 4 — Mapping from legacy extra_hours.id to migrated absence_period.id.
-- Idempotency key: extra_hours_id PK. Re-run skips already-mapped rows.
-- Write-once audit table; NO soft-delete.

CREATE TABLE absence_period_migration_source (
    extra_hours_id    BLOB(16) NOT NULL PRIMARY KEY,
    absence_period_id BLOB(16) NOT NULL,
    cutover_run_id    BLOB(16) NOT NULL,
    migrated_at       TEXT NOT NULL,

    FOREIGN KEY (absence_period_id) REFERENCES absence_period(id),
    FOREIGN KEY (extra_hours_id)    REFERENCES extra_hours(id)
);

CREATE INDEX idx_absence_period_migration_source_period
    ON absence_period_migration_source(absence_period_id);

CREATE INDEX idx_absence_period_migration_source_run
    ON absence_period_migration_source(cutover_run_id);
