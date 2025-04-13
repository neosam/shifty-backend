-- Special type of extra hours which can be defined additional to the default ones
CREATE TABLE IF NOT EXISTS custom_extra_hours (
    id BLOB NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    modifies_balance INTEGER NOT NULL,

    created TEXT NOT NULL,
    deleted TEXT,
    update_version TEXT NOT NULL,
    update_process TEXT NOT NULL,
    PRIMARY KEY (id)
);

-- A table which conntects the custom extra hours to the sales person
CREATE TABLE IF NOT EXISTS custom_extra_hours_sales_person (
    sales_person_id BLOB NOT NULL,
    custom_extra_hours_id BLOB NOT NULL,
    created TEXT NOT NULL,
    deleted TEXT,
    update_process TEXT NOT NULL,
    PRIMARY KEY (sales_person_id, custom_extra_hours_id),
    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id),
    FOREIGN KEY (custom_extra_hours_id) REFERENCES custom_extra_hours(id)
);