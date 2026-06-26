INSERT OR IGNORE INTO toggle (name, enabled, description, update_process)
VALUES (
    'paid_limit_hard_enforcement',
    0,  -- 0 = soft (warnings only), default; 1 = hard (block non-shiftplanners)
    'When ON, booking over a slot/week paid-employee limit is blocked for non-shiftplanners. Default OFF (soft, warning-only).',
    'phase-24-migration'
);
