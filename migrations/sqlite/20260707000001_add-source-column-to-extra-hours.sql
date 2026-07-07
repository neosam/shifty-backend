-- Phase 54 (D-54-DM-02): Rebooking-Marker auf extra_hours.
-- Bestehende Rows bekommen per DEFAULT 'manual' (kein Backfill-Sweep noetig).
-- Werte-Konvention: 'manual' (Standard) | 'rebooking' (Marker fuer F1/F2-Filter).
-- Konsumenten (F1-Ist / F2-Soll Aggregatoren) filtern ab Phase 54 auf
-- source='manual'; Rebooking-Schreiber folgen ab Phase 55.
ALTER TABLE extra_hours
    ADD COLUMN source TEXT NOT NULL DEFAULT 'manual';
