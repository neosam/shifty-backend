-- Phase 54 (VOL-STAT / VOL-ACCT — Data-Model-Basis).
-- Rebooking-Batch Parent + Entry Child; Konsumenten (F3 manual / F4 cron /
-- F5 alert) folgen ab Phase 55/56. Der UNIQUE-Partial-Index enforced
-- (sales_person_id, iso_year, iso_week) global ueber alle kinds
-- (D-54-DM-01: Claim-on-Suggest — hr_suggestion(state=pending) beansprucht
-- die Wochen-Slot direkt via UNIQUE, keine eigene State-Machine).
-- Soft-Delete-Konvention (WHERE deleted IS NULL, kein FK ON DELETE CASCADE).

CREATE TABLE IF NOT EXISTS rebooking_batch (
    id BLOB NOT NULL PRIMARY KEY,
    sales_person_id BLOB NOT NULL,
    iso_year INTEGER NOT NULL,
    iso_week INTEGER NOT NULL,
    kind TEXT NOT NULL,           -- 'manual' | 'hr_suggestion' | 'auto_cron' | 'auto_cron_backfill'
    state TEXT NOT NULL,          -- 'pending' | 'approved' | 'rejected' | 'skipped_locked'
    created TEXT NOT NULL,
    approved TEXT,
    approved_by TEXT,
    deleted TEXT,
    update_process TEXT NOT NULL,
    update_version BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS rebooking_batch_entry (
    id BLOB NOT NULL PRIMARY KEY,
    batch_id BLOB NOT NULL REFERENCES rebooking_batch(id),
    sales_person_id BLOB NOT NULL,
    hours REAL NOT NULL,                 -- positiv, Betrag der Umbuchung
    balance_before REAL NOT NULL,        -- Snapshot Stundenkonto vor Rebooking
    voluntary_actual REAL NOT NULL,      -- F1-Ist zum Zeitpunkt (Audit)
    voluntary_committed REAL NOT NULL,   -- F2-Soll zum Zeitpunkt (Audit)
    extra_hours_out_id BLOB,             -- FK auf extra_hours (-N), NULL bis approved
    extra_hours_in_id BLOB,              -- FK auf extra_hours (+N), NULL bis approved
    created TEXT NOT NULL,
    deleted TEXT,
    update_process TEXT NOT NULL,
    update_version BLOB NOT NULL
);

-- D-54-DM-01: Genau ein aktiver Batch pro (sales_person_id, iso_year, iso_week),
-- global ueber alle kinds. Soft-deleted history ist erlaubt (WHERE deleted IS NULL).
CREATE UNIQUE INDEX IF NOT EXISTS rebooking_batch_week_unique_idx
    ON rebooking_batch (sales_person_id, iso_year, iso_week)
    WHERE deleted IS NULL;

CREATE INDEX IF NOT EXISTS rebooking_batch_state_idx
    ON rebooking_batch (state)
    WHERE deleted IS NULL;

CREATE INDEX IF NOT EXISTS rebooking_batch_entry_sp_idx
    ON rebooking_batch_entry (sales_person_id)
    WHERE deleted IS NULL;
