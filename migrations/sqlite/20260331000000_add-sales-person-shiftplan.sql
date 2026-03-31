CREATE TABLE sales_person_shiftplan (
    sales_person_id blob(16) NOT NULL,
    shiftplan_id blob(16) NOT NULL,
    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    PRIMARY KEY (sales_person_id, shiftplan_id),
    FOREIGN KEY (sales_person_id) REFERENCES sales_person(id),
    FOREIGN KEY (shiftplan_id) REFERENCES shiftplan(id)
);
