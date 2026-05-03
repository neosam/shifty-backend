-- Phase 4 — New privilege for the destructive cutover commit endpoint.
-- Pattern verbatim from 20260501000000_add-feature-flag-table.sql:22-23.

INSERT INTO privilege (name, update_process)
VALUES ('cutover_admin', 'phase-4-migration');
