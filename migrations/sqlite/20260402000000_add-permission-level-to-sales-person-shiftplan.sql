ALTER TABLE sales_person_shiftplan ADD COLUMN permission_level TEXT NOT NULL DEFAULT 'available' CHECK(permission_level IN ('available', 'planner_only'));
