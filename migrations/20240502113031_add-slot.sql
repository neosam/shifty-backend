-- Add migration script here

CREATE TABLE slot (
    id blob(16) NOT NULL PRIMARY KEY,
    day_of_week INTEGER NOT NULL,
    time_from TEXT NOT NULL,
    time_to TEXT NOT NULL,
    valid_from TEXT NOT NULL,
    valid_to TEXT,
    deleted TEXT,

    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    update_version blob(16) NOT NULL
);