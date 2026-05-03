-- Phase 4 — Pre-cutover snapshot of employee_yearly_carryover for safe rollback.
-- Schema-isomorph to employee_yearly_carryover (sales_person_id, year, carryover_hours, vacation,
-- created, deleted, update_process, update_version) plus cutover_run_id + backed_up_at.
-- PK is composite (cutover_run_id, sales_person_id, year) so multiple cutover runs can coexist.

CREATE TABLE employee_yearly_carryover_pre_cutover_backup (
    cutover_run_id  BLOB(16) NOT NULL,
    sales_person_id BLOB(16) NOT NULL,
    year            INTEGER NOT NULL,
    carryover_hours REAL NOT NULL,
    vacation        INTEGER NOT NULL,
    created         TEXT NOT NULL,
    deleted         TEXT,
    update_process  TEXT NOT NULL,
    update_version  BLOB(16) NOT NULL,
    backed_up_at    TEXT NOT NULL,

    PRIMARY KEY (cutover_run_id, sales_person_id, year),
    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id)
);

CREATE INDEX idx_employee_yearly_carryover_pre_cutover_backup_sp_year
    ON employee_yearly_carryover_pre_cutover_backup(sales_person_id, year);
