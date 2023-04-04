-- migrate:up
CREATE TABLE IF NOT EXISTS gateway (
    "id" UUID NOT NULL DEFAULT uuid_generate_v4(),
    "name" TEXT UNIQUE NOT NULL,
    CONSTRAINT pk_g_id PRIMARY KEY (id)
);
CREATE TABLE IF NOT EXISTS gateway_integration (
    "id" UUID NOT NULL DEFAULT uuid_generate_v4(),
    "gateway_id" UUID NOT NULL,
    "payment_method_id" UUID NOT NULL,
    "gateway_external_id" TEXT UNIQUE DEFAULT NULL,
    CONSTRAINT pk_gi_id PRIMARY KEY (id),
    CONSTRAINT fk_gi_gateway_id FOREIGN KEY("gateway_id") REFERENCES "gateway"("id"),
    CONSTRAINT fk_gi_payment_method_id FOREIGN KEY("payment_method_id") REFERENCES "payment_method"("id")
);
-- migrate:down
DROP TABLE gateway_integration;
DROP TABLE gateway;