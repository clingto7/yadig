ALTER TABLE operation_plan_items
ADD COLUMN source_collection_external_id TEXT;

ALTER TABLE operation_plan_items
ADD COLUMN source_collection_title TEXT;

ALTER TABLE operation_plan_items
ADD COLUMN target_collection_external_id TEXT;

ALTER TABLE operation_plan_items
ADD COLUMN target_collection_title TEXT;

ALTER TABLE operation_plan_items
ADD COLUMN resource_id TEXT;

ALTER TABLE operation_plan_items
ADD COLUMN resource_type TEXT;
