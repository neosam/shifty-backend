CREATE TABLE sales_person_unavailable (
    id blob(16) NOT NULL PRIMARY KEY,
    sales_person_id blob(16) NOT NULL,
    year INTEGER NOT NULL,
    calendar_week INTEGER NOT NULL,
    day_of_week INTEGER NOT NULL,
    created TEXT NOT NULL,
    deleted TEXT,

    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    update_version blob(16) NOT NULL,

    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id)
);