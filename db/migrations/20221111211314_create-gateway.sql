-- migrate:up

CREATE TABLE IF NOT EXISTS gateway (
    "id" UUID NOT NULL DEFAULT uuid_generate_v4(),
    "name" TEXT UNIQUE NOT NULL,
    CONSTRAINT pk_g_id PRIMARY KEY ( id )
);

ALTER TABLE charge
    ADD "gateway_id" UUID NOT NULL,
    ADD CONSTRAINT fk_c_gateway_id FOREIGN KEY ("gateway_id") REFERENCES gateway("id");

-- migrate:down

ALTER TABLE charge
    DROP CONSTRAINT fk_student_cifk_c_gateway_idty_id;
ALTER TABLE charge
    DROP COLUMN  "gateway_id";

DROP TABLE gateway;