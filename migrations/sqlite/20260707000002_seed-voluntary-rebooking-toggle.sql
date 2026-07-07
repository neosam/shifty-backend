-- Phase 54 (VOL-ACCT / REB-AUTO-Preparation): idempotenter Toggle-Seed fuer
-- den voluntary-rebooking Auto-Cron-Stichtag. Der Toggle liegt bereits mit
-- default enabled=0 und value=NULL — die Wirkung greift erst in Phase 56
-- (F4-Cron), wo `value` (ISO YYYY-MM-DD) als Cutoff-Datum interpretiert
-- wird. `INSERT OR IGNORE` sorgt fuer Idempotenz (Praezedenz v2.4 SHC-04).
INSERT OR IGNORE INTO toggle (name, enabled, description, update_process)
VALUES (
    'voluntary_rebooking_auto_active_from',
    0,
    'When a cutoff date is set in `value` (ISO YYYY-MM-DD), the voluntary rebooking auto-cron (Phase 56 F4) runs only for ISO weeks >= that date. Leave value NULL to disable (default, no rebooking).',
    'phase-54-migration'
);
