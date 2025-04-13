-- Add migration script here
ALTER TABLE extra_hours
  ADD COLUMN custom_extra_hours_id BLOB
         NOT NULL
         DEFAULT X'00000000000000000000000000000000';