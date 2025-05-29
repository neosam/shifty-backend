-- Update bookings_view to include user tracking columns

-- Drop the existing view
DROP VIEW IF EXISTS bookings_view;

-- Recreate the view with the new columns
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
	booking.deleted_by
FROM booking
INNER JOIN sales_person ON sales_person.id = booking.sales_person_id
INNER JOIN slot ON slot.id = booking.slot_id; 