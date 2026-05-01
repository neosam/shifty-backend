-- 1. Add nullable logical_id column.
ALTER TABLE extra_hours ADD COLUMN logical_id BLOB(16);

-- 2. Backfill existing rows: logical_id = id.
UPDATE extra_hours SET logical_id = id;

-- 3. Rebuild table to enforce NOT NULL on logical_id.
CREATE TABLE extra_hours_new (
    id BLOB(16) NOT NULL PRIMARY KEY,
    sales_person_id BLOB(16) NOT NULL,
    amount FLOAT NOT NULL,
    category TEXT NOT NULL,
    description TEXT,
    date_time TEXT NOT NULL,
    created TEXT NOT NULL,
    deleted TEXT,

    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    update_version BLOB(16) NOT NULL,

    custom_extra_hours_id BLOB NOT NULL DEFAULT X'00000000000000000000000000000000',

    logical_id BLOB(16) NOT NULL,

    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id)
);

INSERT INTO extra_hours_new (
    id, sales_person_id, amount, category, description, date_time, created, deleted,
    update_timestamp, update_process, update_version, custom_extra_hours_id, logical_id
)
SELECT
    id, sales_person_id, amount, category, description, date_time, created, deleted,
    update_timestamp, update_process, update_version, custom_extra_hours_id, logical_id
FROM extra_hours;

DROP TABLE extra_hours;
ALTER TABLE extra_hours_new RENAME TO extra_hours;

-- 4. Partial unique index: at most one active row per logical_id.
CREATE UNIQUE INDEX idx_extra_hours_logical_id_active
    ON extra_hours(logical_id)
    WHERE deleted IS NULL;
