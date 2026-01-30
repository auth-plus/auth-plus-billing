-- migrate:up
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TABLE IF NOT EXISTS "user" (
    "id" UUID NOT NULL DEFAULT uuid_generate_v4(),
    "external_id" UUID UNIQUE NOT NULL,
    CONSTRAINT pk_u_id PRIMARY KEY (id)
);
CREATE TABLE IF NOT EXISTS invoice (
    "id" UUID NOT NULL DEFAULT uuid_generate_v4(),
    "user_id" UUID NOT NULL,
    "status" TEXT NOT NULL DEFAULT 'not_charged',
    -- could be draft/pending/charged_with_error/paid/cancelled/uncollectible/refunded/in_protest/chargeback
    CONSTRAINT pk_i_id PRIMARY KEY (id),
    CONSTRAINT fk_i_user_id FOREIGN KEY("user_id") REFERENCES "user"("id") ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS invoice_item (
    "id" UUID NOT NULL DEFAULT uuid_generate_v4(),
    "invoice_id" UUID NOT NULL,
    "description" TEXT NOT NULL,
    "quantity" INT NOT NULL,
    "amount" REAL NOT NULL,
    "currency" TEXT NOT NULL,
    CONSTRAINT pk_ii_id PRIMARY KEY (id),
    CONSTRAINT fk_ii_invoice_id FOREIGN KEY("invoice_id") REFERENCES invoice("id") ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS payment_method (
    "id" UUID NOT NULL DEFAULT uuid_generate_v4(),
    "user_id" UUID NOT NULL,
    "is_default" BOOLEAN DEFAULT TRUE,
    "method" TEXT NOT NULL,
    -- could be credit_card, pix, ledger
    "info" JSONB NOT NULL,
    -- should contain credit_card info, or pix key
    CONSTRAINT pk_pm_id PRIMARY KEY (id),
    CONSTRAINT fk_pm_user_id FOREIGN KEY("user_id") REFERENCES "user"("id") ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS charge (
    "id" UUID NOT NULL DEFAULT uuid_generate_v4(),
    "invoice_id" UUID NOT NULL,
    "status" TEXT NOT NULL DEFAULT 'progress',
    -- could be failed/progress/succeed
    "payment_method_id" UUID NOT NULL,
    CONSTRAINT pk_c_id PRIMARY KEY (id),
    CONSTRAINT fk_c_invoice_id FOREIGN KEY("invoice_id") REFERENCES invoice("id") ON DELETE CASCADE,
    CONSTRAINT fk_c_payment_method_id FOREIGN KEY("payment_method_id") REFERENCES payment_method("id") ON DELETE CASCADE
);
-- migrate:down
DROP TABLE IF EXISTS charge;
DROP TABLE IF EXISTS payment_method;
DROP TABLE IF EXISTS invoice_item;
DROP TABLE IF EXISTS invoice;
DROP TABLE IF EXISTS "user";