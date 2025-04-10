-- Add migration script here

CREATE TABLE booking (
    id blob(16) NOT NULL PRIMARY KEY,
    sales_person_id blob(16) NOT NULL,
    slot_id blob(16) NOT NULL,
    calendar_week INTEGER NOT NULL,
    year INTEGER NOT NULL,
    created TEXT NOT NULL,
    deleted TEXT,

    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    update_version blob(16) NOT NULL
);