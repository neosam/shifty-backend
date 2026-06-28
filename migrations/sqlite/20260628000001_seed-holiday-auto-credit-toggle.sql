INSERT OR IGNORE INTO toggle (name, enabled, description, update_process)
VALUES (
    'holiday_auto_credit',
    0,
    'When a cutoff date is set in `value` (ISO YYYY-MM-DD), holidays on or after that date are auto-credited in reports. Leave value NULL to disable.',
    'phase-25-migration'
);
