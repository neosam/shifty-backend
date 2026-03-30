-- Add shiftplan table for multi-plan support

CREATE TABLE shiftplan (
    id blob(16) NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    is_planning INTEGER NOT NULL DEFAULT 0,
    deleted TEXT,
    update_timestamp TEXT,
    update_process TEXT NOT NULL,
    update_version blob(16) NOT NULL
);

-- Insert default "main" plan
INSERT INTO shiftplan VALUES(
    X'00000000000040008000000000000001',
    'main',
    0,
    NULL,
    NULL,
    'migration',
    X'00000000000040008000000000000002'
);

-- Add shiftplan_id foreign key to slot table (nullable for migration)
ALTER TABLE slot ADD COLUMN shiftplan_id blob(16) REFERENCES shiftplan(id);

-- Backfill all existing slots with the default plan
UPDATE slot SET shiftplan_id = X'00000000000040008000000000000001';

-- Update bookings_view to include shiftplan name
DROP VIEW IF EXISTS bookings_view;

CREATE VIEW bookings_view AS
SELECT
    lower(substr(hex(booking.id), 1, 8)
	|| '-' || substr(hex(booking.id), 9, 4)
	|| '-' || substr(hex(booking.id), 13, 4)
	|| '-' || substr(hex(booking.id), 17, 4)
	|| '-' || substr(hex(booking.id), 21, 12)) booking_hex,
	lower(substr(hex(sales_person.id), 1, 8)
	|| '-' || substr(hex(sales_person.id), 9, 4)
	|| '-' || substr(hex(sales_person.id), 13, 4)
	|| '-' || substr(hex(sales_person.id), 17, 4)
	|| '-' || substr(hex(sales_person.id), 21, 12)) sales_person_hex,
	lower(substr(hex(slot.id), 1, 8)
	|| '-' || substr(hex(slot.id), 9, 4)
	|| '-' || substr(hex(slot.id), 13, 4)
	|| '-' || substr(hex(slot.id), 17, 4)
	|| '-' || substr(hex(slot.id), 21, 12)) slot_hex,
	sales_person.name,
	booking.year,
	booking.calendar_week,
	slot.day_of_week,
	slot.time_from,
	slot.time_to,
	booking.created,
	booking.deleted,
	booking.created_by,
	booking.deleted_by,
	shiftplan.name as shiftplan_name
FROM booking
INNER JOIN sales_person ON sales_person.id = booking.sales_person_id
INNER JOIN slot ON slot.id = booking.slot_id
LEFT JOIN shiftplan ON slot.shiftplan_id = shiftplan.id;
