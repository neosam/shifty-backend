-- Phase 39 (WST-01, D-39-10): per-ISO-(year, calendar_week) KW-Status.
-- Soft-delete row with a TEXT discriminant (InPlanning/Planned/Locked; the
-- empty Unset variant is NEVER persisted — row absence means Unset, D-39-04).
-- Deliberately uses a PARTIAL unique index (WHERE deleted IS NULL) so that
-- soft-deleted history is allowed — unlike week_message's plain UNIQUE
-- (see RESEARCH Pitfall P-6). No FOREIGN KEY, no sales_person_id.
CREATE TABLE IF NOT EXISTS week_status (
    id BLOB NOT NULL PRIMARY KEY,
    year INTEGER NOT NULL,
    calendar_week INTEGER NOT NULL,
    status TEXT NOT NULL,
    created TEXT NOT NULL,
    deleted TEXT,
    update_process TEXT NOT NULL,
    update_version BLOB NOT NULL
);

-- Exactly one active status per (year, calendar_week); soft-deleted history allowed.
CREATE UNIQUE INDEX IF NOT EXISTS idx_week_status_active
    ON week_status (year, calendar_week)
    WHERE deleted IS NULL;
