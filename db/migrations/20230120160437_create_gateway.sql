-- migrate:up
ALTER TABLE gateway ADD COLUMN "deleted_at" timestamp DEFAULT NULL;

INSERT INTO gateway ("name") VALUES ('iugu_gateway');


-- migrate:down

UPDATE gateway SET deleted_at = current_timestamp WHERE "name"="IUGU";