INSERT OR IGNORE INTO toggle (name, enabled, description, update_process)
VALUES (
    'shortday_slot_clipping_active_from',
    0,
    'When a cutoff date is set in `value` (ISO YYYY-MM-DD), slots at short-day dates >= that date are clipped at the ShortDay cutoff time in rendering and hours calculation. Leave value NULL to disable (legacy behavior).',
    'phase-51-migration'
);
