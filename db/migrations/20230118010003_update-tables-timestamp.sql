-- migrate:up
ALTER TABLE "user"
ADD COLUMN "created_at" TIMESTAMPTZ NOT NULL DEFAULT timezone('utc', NOW());
ALTER TABLE invoice
ADD COLUMN "created_at" TIMESTAMPTZ NOT NULL DEFAULT timezone('utc', NOW());
ALTER TABLE invoice_item
ADD COLUMN "created_at" TIMESTAMPTZ NOT NULL DEFAULT timezone('utc', NOW());
ALTER TABLE payment_method
ADD COLUMN "created_at" TIMESTAMPTZ NOT NULL DEFAULT timezone('utc', NOW());
ALTER TABLE charge
ADD COLUMN "created_at" TIMESTAMPTZ NOT NULL DEFAULT timezone('utc', NOW());
ALTER TABLE charge
ADD COLUMN "external_id" TEXT NOT NULL;
ALTER TABLE gateway
ADD COLUMN "created_at" TIMESTAMPTZ NOT NULL DEFAULT timezone('utc', NOW());
ALTER TABLE gateway_integration
ADD COLUMN "created_at" TIMESTAMPTZ NOT NULL DEFAULT timezone('utc', NOW());
-- migrate:down
ALTER TABLE "user" DROP COLUMN "created_at";
ALTER TABLE invoice DROP COLUMN "created_at";
ALTER TABLE invoice_item DROP COLUMN "created_at";
ALTER TABLE payment_method DROP COLUMN "created_at";
ALTER TABLE charge DROP COLUMN "created_at";
ALTER TABLE gateway DROP COLUMN "created_at";
ALTER TABLE gateway_integration DROP COLUMN "created_at";