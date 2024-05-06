-- Add migration script here

CREATE TABLE sales_person (
    id blob(16) NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    inactive BOOLEAN NOT NULL,
    deleted TEXT,

    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    update_version blob(16) NOT NULL
);