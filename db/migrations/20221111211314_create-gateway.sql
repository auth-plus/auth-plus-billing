-- migrate:up

CREATE TABLE IF NOT EXISTS gateway (
    "id" UUID NOT NULL DEFAULT uuid_generate_v4(),
    "name" TEXT UNIQUE NOT NULL,
    CONSTRAINT pk_g_id PRIMARY KEY ( id )
);

ALTER TABLE payment_method
    ADD "gateway_id" UUID NOT NULL,
    ADD CONSTRAINT fk_pm_gateway_id FOREIGN KEY ("gateway_id") REFERENCES gateway("id");

-- migrate:down

ALTER TABLE payment_method
    DROP CONSTRAINT fk_pm_gateway_id;
ALTER TABLE payment_method
    DROP COLUMN  "gateway_id";

DROP TABLE gateway;