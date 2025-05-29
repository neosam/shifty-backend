-- Table for storing custom text messages for calendar weeks
CREATE TABLE IF NOT EXISTS week_message (
    id BLOB(16) NOT NULL,
    year INTEGER NOT NULL,
    calendar_week INTEGER NOT NULL,
    message TEXT NOT NULL,
    created TEXT NOT NULL,
    deleted TEXT,
    update_version BLOB(16) NOT NULL,
    update_process TEXT NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (year, calendar_week)
); 