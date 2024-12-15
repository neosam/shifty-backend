-- Add migration script here
CREATE TABLE IF NOT EXISTS employee_yearly_carryover (
    sales_person_id BLOB NOT NULL,
    year INTEGER NOT NULL,
    carryover_hours REAL NOT NULL,
    created TEXT NOT NULL,
    deleted TEXT,
    update_process TEXT NOT NULL,
    update_version BLOB NOT NULL,
    PRIMARY KEY (sales_person_id, year),
    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id)
);
