-- Phase 8.3: Halbtag-Support für Absences (additiv).
-- Strikt additiv: keine Änderungen an anderen Tabellen, kein Backfill nötig
-- (Default 'full' garantiert no-drift für bestehende Rows — siehe
-- .planning/phases/08.3-halbtag-support-f-r-absences/08.3-CONTEXT.md
-- "No-Drift-Garantie für Bestandsdaten").

ALTER TABLE absence_period
ADD COLUMN day_fraction TEXT NOT NULL DEFAULT 'full'
    CHECK (day_fraction IN ('full', 'half'));
