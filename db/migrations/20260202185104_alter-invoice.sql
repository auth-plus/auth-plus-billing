-- migrate:up
ALTER TABLE invoice
ADD COLUMN idempotency_key TEXT UNIQUE NOT NULL;

-- migrate:down
ALTER TABLE invoice
DROP COLUMN IF EXISTS idempotency_key CASCADE;
