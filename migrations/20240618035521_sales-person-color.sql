-- Add migration script here
ALTER TABLE sales_person
ADD COLUMN background_color TEXT DEFAULT "#FFF" NOT NULL;