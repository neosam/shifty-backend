-- Phase 28 (VAC-OFFSET-01, D-28-01): per-(sales_person, year) signed
-- vacation-entitlement offset. Soft-delete row, structurally mirrors
-- employee_yearly_carryover but with an explicit `id` PK and a partial
-- unique index enforcing one ACTIVE offset per person+year.
CREATE TABLE IF NOT EXISTS vacation_entitlement_offset (
    id BLOB NOT NULL PRIMARY KEY,
    sales_person_id BLOB NOT NULL,
    year INTEGER NOT NULL,
    offset_days INTEGER NOT NULL,
    created TEXT NOT NULL,
    deleted TEXT,
    update_process TEXT NOT NULL,
    update_version BLOB NOT NULL,
    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id)
);

-- One active offset per (sales_person, year); soft-deleted history allowed.
CREATE UNIQUE INDEX IF NOT EXISTS idx_vacation_entitlement_offset_active
    ON vacation_entitlement_offset (sales_person_id, year)
    WHERE deleted IS NULL;
