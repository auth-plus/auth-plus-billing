-- migrate:up
CREATE INDEX idx_invoice_user_id ON invoice(user_id);
CREATE INDEX idx_invoice_item_invoice_id ON invoice_item(invoice_id);
CREATE INDEX idx_payment_method_user_id ON payment_method(user_id);
CREATE INDEX idx_charge_invoice_id ON charge(invoice_id);
CREATE INDEX idx_charge_payment_method_id ON charge(payment_method_id);
CREATE INDEX idx_gateway_integration_gateway_id ON gateway_integration(gateway_id);
CREATE INDEX idx_gateway_integration_payment_method_id ON gateway_integration(payment_method_id);

CREATE INDEX idx_invoice_status ON invoice("status");
CREATE INDEX idx_charge_status ON charge("status");

CREATE INDEX idx_invoice_created_at ON invoice(created_at);
CREATE INDEX idx_charge_created_at ON charge(created_at);

CREATE INDEX idx_user_external_id ON "user"(external_id);

-- migrate:down
DROP INDEX IF EXISTS idx_user_external_id;

DROP INDEX IF EXISTS idx_charge_created_at;
DROP INDEX IF EXISTS idx_invoice_created_at;

DROP INDEX IF EXISTS idx_charge_status;
DROP INDEX IF EXISTS idx_invoice_status;

DROP INDEX IF EXISTS idx_gateway_integration_payment_method_id;
DROP INDEX IF EXISTS idx_gateway_integration_gateway_id;
DROP INDEX IF EXISTS idx_charge_payment_method_id;
DROP INDEX IF EXISTS idx_charge_invoice_id;
DROP INDEX IF EXISTS idx_payment_method_user_id;
DROP INDEX IF EXISTS idx_invoice_item_invoice_id;
DROP INDEX IF EXISTS idx_invoice_user_id;
