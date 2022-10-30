-- migrate:up

BEGIN TRANSACTION;

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS user(
    "id" UUID NOT NULL DEFAULT uuid_generate_v4(),
    "external_id" UUID UNIQUE NOT NULL,
    "wallet_id" UUID NOT NULL,
    CONSTRAINT pk_u_id PRIMARY KEY ( id ),
    CONSTRAINT fk_u_wallet_id FOREIGN KEY("user_id") REFERENCES user("id")
)

CREATE TABLE IF NOT EXISTS invoice(
  "id" UUID NOT NULL DEFAULT uuid_generate_v4(),
  "user_id" UUID NOT NULL,
  "status" TEXT NOT NULL DEFAULT 'not_charged', -- could be draft/pending/charged_with_error/paid/cancelled/uncollectible/refunded/in_protest/chargeback
  CONSTRAINT pk_i_id PRIMARY KEY ( id ),
  CONSTRAINT fk_i_user_id FOREIGN KEY("user_id") REFERENCES user("id")
)

CREATE TABLE IF NOT EXISTS invoice_item(
  "id" UUID NOT NULL DEFAULT uuid_generate_v4(), 
  "invoice_id" UUID NOT NULL,
  "description" TEXT NOT NULL,
  "amount" REAL NOT NULL,
  "currency" TEXT NOT NULL,
  CONSTRAINT pk_ii_id PRIMARY KEY ( id ),
  CONSTRAINT fk_ii_invoice_id FOREIGN KEY("invoice_id") REFERENCES invoice("id")
)

CREATE TABLE IF NOT EXISTS payment_method(
    "id" UUID NOT NULL DEFAULT uuid_generate_v4(),
    "user_id" UUID NOT NULL,
    "is_default" BOOLEAN DEFAULT TRUE,
    "type" TEXT NOT NULL, -- could be credit_card, pix, ledger
    CONSTRAINT pk_pm_id PRIMARY KEY ( id ),
    CONSTRAINT fk_pm_user_id FOREIGN KEY("user_id") REFERENCES user("id")
)

CREATE TABLE IF NOT EXISTS charge(
    "id" UUID NOT NULL DEFAULT uuid_generate_v4(), 
    "invoice_id" UUID NOT NULL,
    "status" TEXT NOT NULL DEFAULT 'failed', -- could be failed/progress/succeed
    "payment_method_id" UUID NOT NULL, 
    CONSTRAINT pk_c_id PRIMARY KEY (id), 
    CONSTRAINT fk_c_invoice_id FOREIGN KEY("invoice_id") REFERENCES invoice("id"), 
    CONSTRAINT fk_c_payment_method_id FOREIGN KEY("payment_method_id") REFERENCES payment_method("id")
)

CREATE TABLE IF NOT EXISTS ledger(
    "id" UUID NOT NULL DEFAULT uuid_generate_v4(), 
    "invoice_id" UUID NOT NULL, 
    "charge_id" UUID NOT NULL, 
    CONSTRAINT pk_c_id PRIMARY KEY (id), 
    CONSTRAINT fk_c_invoice_id FOREIGN KEY("invoice_id") REFERENCES invoice("id"), 
    CONSTRAINT fk_c_payment_method_id FOREIGN KEY("payment_method_id") REFERENCES payment_method("id")
)

CREATE TABLE IF NOT EXISTS wallet(
    "id" UUID NOT NULL DEFAULT uuid_generate_v4(),
    "amount" REAL NOT NULL, 
    CONSTRAINT pk_w_id PRIMARY KEY (id), 
    CONSTRAINT fk_w_user_id FOREIGN KEY("user_id") REFERENCES user("id")
)

COMMIT;
-- migrate:down

