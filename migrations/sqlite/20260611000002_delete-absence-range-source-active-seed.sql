-- Phase 8.6 (D-01): Entferne absence_range_source_active Seed-Row.
-- Alle Reader (top_bar.rs) und Writer (service_impl/cutover.rs:1319) sind in 8.6 entfernt.
-- Tabelle feature_flag BLEIBT als generischer Mechanismus erhalten. Forward-only.
DELETE FROM feature_flag WHERE key = 'absence_range_source_active';
