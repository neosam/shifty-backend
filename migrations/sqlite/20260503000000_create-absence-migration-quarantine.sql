-- Phase 4 (Migration & Cutover) — Quarantine table for ambiguous extra_hours rows.
-- Write-once audit table; NO soft-delete (HR resolves manually).

CREATE TABLE absence_migration_quarantine (
    extra_hours_id  BLOB(16) NOT NULL PRIMARY KEY,
    reason          TEXT NOT NULL,
    sales_person_id BLOB(16) NOT NULL,
    category        TEXT NOT NULL,
    date_time       TEXT NOT NULL,
    amount          REAL NOT NULL,
    cutover_run_id  BLOB(16) NOT NULL,
    migrated_at     TEXT NOT NULL,

    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id),
    FOREIGN KEY (extra_hours_id)  REFERENCES extra_hours(id)
);

CREATE INDEX idx_absence_migration_quarantine_run
    ON absence_migration_quarantine(cutover_run_id);

CREATE INDEX idx_absence_migration_quarantine_sp_cat
    ON absence_migration_quarantine(sales_person_id, category);
