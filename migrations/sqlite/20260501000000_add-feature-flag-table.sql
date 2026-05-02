-- Phase 2 (Reporting Integration & Snapshot Versioning):
-- Eigene generische feature_flag-Tabelle (D-Phase2-06).
-- Bewusst KEIN Reuse von toggle/ToggleService -- semantische Trennung:
-- Feature-Flags sind Architektur/Migrations-Schalter, Toggles sind User-Toggles.

CREATE TABLE feature_flag (
    key TEXT NOT NULL PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 0,  -- 0 = disabled, 1 = enabled
    description TEXT,
    update_timestamp TEXT,
    update_process TEXT NOT NULL
);

INSERT INTO feature_flag (key, enabled, description, update_process)
VALUES (
    'absence_range_source_active',
    0,
    'When ON, range-based AbsencePeriods are the source of truth for Vacation/Sick/UnpaidLeave hours. Flip atomically with Phase-4 migration; do NOT flip manually.',
    'phase-2-migration'
);

INSERT INTO privilege (name, update_process)
VALUES ('feature_flag_admin', 'initial');
