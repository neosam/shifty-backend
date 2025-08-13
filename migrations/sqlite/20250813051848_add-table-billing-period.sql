CREATE TABLE IF NOT EXISTS billing_period (
    id BLOB(16) NOT NULL,

    from_date_time TEXT NOT NULL,
    to_date_time TEXT NOT NULL,

    created TEXT NOT NULL,
    created_by TEXT NOT NULL,
    deleted TEXT,
    deleted_by TEXT,
    update_version BLOB(16) NOT NULL,
    update_process TEXT NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS billing_period_sales_person (
    id BLOB(16) NOT NULL,
    billing_period_id BLOB(16) NOT NULL,
    sales_person_id BLOB(16) NOT NULL,

    value_type TEXT NOT NULL,
    value_delta FLOAT NOT NULL,
    value_ytd_from FLOAT NOT NULL,
    value_ytd_to FLOAT NOT NULL,
    value_full_year FLOAT NOT NULL,

    created_at TEXT NOT NULL,
    created_by TEXT NOT NULL,
    deleted_at TEXT,
    deleted_by TEXT,
    update_version BLOB(16) NOT NULL,
    update_process TEXT NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (billing_period_id) REFERENCES billing_period(id),
    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id),
    UNIQUE (billing_period_id, sales_person_id, value_type)
);