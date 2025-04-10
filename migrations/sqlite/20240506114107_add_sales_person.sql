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

CREATE TABLE sales_person_user (
    sales_person_id blob(16) NOT NULL,
    user_id TEXT NOT NULL,
    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    PRIMARY KEY (sales_person_id, user_id),
    UNIQUE(sales_person_id),
    UNIQUE(user_id),
    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id)
    FOREIGN KEY (user_id) REFERENCES user(name)
)