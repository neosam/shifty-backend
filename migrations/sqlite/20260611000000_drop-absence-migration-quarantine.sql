-- Phase 8.6 (D-04): Drop absence_migration_quarantine.
-- Tabelle wurde ausschliesslich von dao_impl_sqlite/src/cutover.rs beschrieben (in 8.6 geloescht).
-- In Prod nie befuellt (CONTEXT.md Modell-Fakt 2026-06-11). INT wipeable. Forward-only (D-05).
DROP TABLE IF EXISTS absence_migration_quarantine;
