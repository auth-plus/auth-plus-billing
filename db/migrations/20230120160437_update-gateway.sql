-- migrate:up
ALTER TABLE gateway
ADD COLUMN "deleted_at" timestamptz DEFAULT NULL;
ALTER TABLE gateway
ADD COLUMN "priority" integer NOT NULL;
INSERT INTO gateway ("name", "priority")
VALUES ('stripe', 0);
-- migrate:down
UPDATE gateway
SET deleted_at = current_timestamp
WHERE "name" = "stripe";