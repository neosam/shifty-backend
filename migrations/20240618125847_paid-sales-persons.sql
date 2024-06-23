-- Add migration script here

ALTER TABLE sales_person
ADD COLUMN is_paid BOOLEAN DEFAULT 0 NOT NULL;

CREATE TABLE working_hours (
    id blob(16) NOT NULL PRIMARY KEY,
    sales_person_id blob(16) NOT NULL,
    expected_hours FLOAT NOT NULL,
    from_calendar_week INTEGER NOT NULL,
    from_year INTEGER NOT NULL,
    to_calendar_week INTEGER NOT NULL,
    to_year INTEGER NOT NULL,
    created TEXT NOT NULL,
    deleted TEXT,

    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    update_version blob(16) NOT NULL,
 
    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id)
);

CREATE TABLE extra_hours (
    id blob(16) NOT NULL PRIMARY KEY,
    sales_person_id blob(16) NOT NULL,
    amount FLOAT NOT NULL,
    category TEXT NOT NULL,
    description TEXT,
    date_time TEXT NOT NULL,
    created TEXT NOT NULL,
    deleted TEXT,

    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    update_version blob(16) NOT NULL,

    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id)
);