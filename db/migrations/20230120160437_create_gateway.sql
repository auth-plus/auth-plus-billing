-- migrate:up
ALTER TABLE gateway
ADD COLUMN "deleted_at" timestamptz DEFAULT NULL;
ALTER TABLE gateway
ADD COLUMN "priority" integer NOT NULL;
INSERT INTO gateway ("name", "priority")
VALUES ('iugu_gateway', 0);
-- migrate:down
UPDATE gateway
SET deleted_at = current_timestamp
WHERE "name" = "iugu_gateway";