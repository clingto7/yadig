ALTER TABLE operation_plan_items
ADD COLUMN metadata_json TEXT NOT NULL DEFAULT '{}';
