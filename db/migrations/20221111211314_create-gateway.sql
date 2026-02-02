-- migrate:up
CREATE TABLE IF NOT EXISTS gateway (
    "id" UUID NOT NULL DEFAULT uuid_generate_v4(),
    "name" TEXT UNIQUE NOT NULL,
    CONSTRAINT pk_g_id PRIMARY KEY (id)
);
CREATE TABLE IF NOT EXISTS gateway_integration (
    "id" UUID NOT NULL DEFAULT uuid_generate_v4(),
    "gateway_id" UUID NOT NULL,
    "user_id" UUID NOT NULL,
    "payment_method_id" UUID DEFAULT NULL,
    "gateway_external_user_id" TEXT NOT NULL,
    "gateway_external_payment_method_id" TEXT UNIQUE DEFAULT NULL,
    CONSTRAINT pk_gi_id PRIMARY KEY (id),
    CONSTRAINT fk_gi_gateway_id FOREIGN KEY("gateway_id") REFERENCES "gateway"("id") ON DELETE CASCADE,
    CONSTRAINT fk_gi_payment_method_id FOREIGN KEY("payment_method_id") REFERENCES "payment_method"("id") ON DELETE CASCADE,
    CONSTRAINT fk_gi_user_id FOREIGN KEY("user_id") REFERENCES "user"("id") ON DELETE CASCADE
);
-- migrate:down
DROP TABLE gateway_integration;
DROP TABLE gateway;