CREATE TABLE special_day (
    id blob(16) NOT NULL PRIMARY KEY,
    year INTEGER NOT NULL,
    calendar_week INTEGER NOT NULL,
    day_of_week INTEGER NOT NULL,
    day_type TEXT NOT NULL,
    time_of_day TEXT,
    created TEXT NOT NULL,
    deleted TEXT,

    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    update_version blob(16) NOT NULL
);