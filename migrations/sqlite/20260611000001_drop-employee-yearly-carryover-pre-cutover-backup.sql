-- Phase 8.6 (D-04): Drop employee_yearly_carryover_pre_cutover_backup.
-- Reines Cutover-Backup-Artefakt; nur von dao_impl_sqlite/src/cutover.rs:327 (Cutover-Commit)
-- je beschrieben (in 8.6 geloescht). In Prod nie befuellt. INT wipeable. Forward-only (D-05).
DROP TABLE IF EXISTS employee_yearly_carryover_pre_cutover_backup;
