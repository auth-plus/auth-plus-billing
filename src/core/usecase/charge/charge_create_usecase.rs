use uuid::Uuid;

use crate::core::{
    dto::{charge::Charge, invoice::InvoiceStatus},
    usecase::driven::{
        creating_charge::{CreatingCharge, CreatingChargeError},
        reading_gateway::ReadingGateway,
        reading_invoice::{ReadingInvoice, ReadingInvoiceError},
        reading_invoice_item::ReadingInvoiceItem,
        reading_payment_method::{ReadingPaymentMethod, ReadingPaymentMethodError},
        updating_invoice::{UpdatingInvoice, UpdatingInvoiceError},
    },
};

pub struct ChargeCreateUsecase {
    pub reading_invoice: Box<dyn ReadingInvoice>,
    pub reading_payment_method: Box<dyn ReadingPaymentMethod>,
    pub creating_charge: Box<dyn CreatingCharge>,
    pub updating_invoice: Box<dyn UpdatingInvoice>,
    pub reading_gateway: Box<dyn ReadingGateway>,
    pub reading_invoice_item: Box<dyn ReadingInvoiceItem>,
}

impl ChargeCreateUsecase {
    pub async fn create_charge(&self, invoice_id_str: &str) -> Result<Charge, String> {
        let invoice_id = match Uuid::parse_str(invoice_id_str) {
            Ok(id) => id,
            Err(_) => return Err(String::from("external id provided isn't uuid")),
        };
        let result_invoice = self.reading_invoice.get_by_id(invoice_id).await;
        let invoice = match result_invoice {
            Ok(invoice) => invoice,
            Err(error) => match error {
                ReadingInvoiceError::InvoiceNotFoundError => {
                    return Err(String::from("invoice not found"));
                }
                ReadingInvoiceError::UnmappedError => {
                    return Err(String::from(
                        "ReadingInvoiceError::UnmappedError Something wrong happen",
                    ));
                }
            },
        };
        if invoice.status != InvoiceStatus::Draft
            && invoice.status != InvoiceStatus::ChargedWithError
            && invoice.status != InvoiceStatus::Pending
        {
            return Err(String::from(
                "Invoice can could not be charged because of current status",
            ));
        }
        let result_payment_method = self
            .reading_payment_method
            .get_default_by_user_id(&invoice.user_id)
            .await;
        let payment_method = match result_payment_method {
            Ok(payment_method) => payment_method,
            Err(error) => match error {
                ReadingPaymentMethodError::UnmappedError => {
                    return Err(String::from(
                        "ReadingPaymentMethodError::UnmappedError Something wrong happen",
                    ));
                }
            },
        };
        let amount = self
            .reading_invoice_item
            .get_sum_by_invoice_id(invoice_id)
            .await
            .expect("Not able to sum items from invoice");
        let gateway = self
            .reading_gateway
            .get_priority_list()
            .await
            .expect("CreatingUserError::UnmappedError Something wrong happen");
        let external_charge = gateway
            .charge(amount, "auth-plus-billing")
            .await
            .expect("Problem when creating user on gateway");
        let result_charge = self
            .creating_charge
            .create_charge(invoice.id, payment_method.id, &external_charge.id)
            .await;
        let charge = match result_charge {
            Ok(charge) => charge,
            Err(error) => match error {
                CreatingChargeError::KafkaProducerError => {
                    return Err(String::from("Error on producing on kafka"));
                }
                CreatingChargeError::UnmappedError => {
                    return Err(String::from(
                        "CreatingChargeError::UnmappedError Something wrong happen",
                    ));
                }
            },
        };
        match self
            .updating_invoice
            .update(invoice_id, InvoiceStatus::Pending)
            .await
        {
            Ok(_) => Ok(charge),
            Err(error) => match error {
                UpdatingInvoiceError::UnmappedError => Err(String::from(
                    "UpdatingInvoiceError::UnmappedError Something wrong happen",
                )),
            },
        }
    }
}

#[cfg(test)]
mod test {

    use super::ChargeCreateUsecase;
    use crate::core::{
        dto::{
            charge::{Charge, ChargeStatus},
            invoice::{Invoice, InvoiceStatus},
            payment_method::{Method, PaymentMethod, PaymentMethodInfo, PixInfo},
        },
        usecase::driven::{
            creating_charge::{CreatingChargeError, MockCreatingCharge},
            gateway::{GatewayAPI, GatewayCharge, MockGatewayAPI},
            reading_gateway::MockReadingGateway,
            reading_invoice::{MockReadingInvoice, ReadingInvoiceError},
            reading_invoice_item::MockReadingInvoiceItem,
            reading_payment_method::{MockReadingPaymentMethod, ReadingPaymentMethodError},
            updating_invoice::{MockUpdatingInvoice, UpdatingInvoiceError},
        },
    };
    use fake::{Fake, uuid::UUIDv4};
    use mockall::predicate;
    use rust_decimal::prelude::ToPrimitive;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_succeed_creating_charge() {
        let now = time::OffsetDateTime::now_utc().to_string();
        let user_id: Uuid = UUIDv4.fake();
        let invoice_id: Uuid = UUIDv4.fake();
        let payment_method_id: Uuid = UUIDv4.fake();
        let charge_id: Uuid = UUIDv4.fake();
        let sum = dec!(100.0);
        let pix_info = PixInfo {
            key: String::from("any@email.com"),
            external_id: String::from("ABCDEFG"),
        };
        let info = PaymentMethodInfo::PixInfo(pix_info);
        let invoice = Invoice {
            id: invoice_id,
            user_id,
            status: InvoiceStatus::Draft,
            created_at: now,
        };
        let payment_method = PaymentMethod {
            id: payment_method_id,
            user_id,
            is_default: true,
            method: Method::Pix,
            info,
            created_at: 123_456_789_012,
        };
        let charge = Charge {
            id: charge_id,
            invoice_id,
            payment_method_id: payment_method.id,
            status: ChargeStatus::Progress,
        };
        let gateway_charge = GatewayCharge {
            id: UUIDv4.fake(),
            amount: sum.to_f32().unwrap(),
            currency: String::from("USD"),
        };
        let mut mock_ri = MockReadingInvoice::new();
        mock_ri
            .expect_get_by_id()
            .with(predicate::eq(invoice_id))
            .times(1)
            .return_const(Ok(invoice.clone()));
        let mut mock_rpm = MockReadingPaymentMethod::new();
        mock_rpm
            .expect_get_default_by_user_id()
            .with(predicate::eq(user_id))
            .times(1)
            .return_const(Ok(payment_method.clone()));
        let mut mock_ui = MockUpdatingInvoice::new();
        mock_ui.expect_update().return_const(Ok(invoice.clone()));
        let mut mock_rii = MockReadingInvoiceItem::new();
        mock_rii
            .expect_get_sum_by_invoice_id()
            .return_const(Ok(sum));
        let mut mock_g = MockGatewayAPI::new();
        mock_g
            .expect_charge()
            .with(predicate::eq(sum), predicate::eq("auth-plus-billing"))
            .times(1)
            .return_const(Ok(gateway_charge.clone()));
        let mut mock_rg: MockReadingGateway = MockReadingGateway::new();
        mock_rg
            .expect_get_priority_list()
            .with()
            .times(1)
            .return_once(move || {
                let boxed_gateway: Box<dyn GatewayAPI + Send> = Box::new(mock_g);
                Ok(boxed_gateway)
            });
        let mut mock_cc = MockCreatingCharge::new();
        mock_cc
            .expect_create_charge()
            .with(
                predicate::eq(invoice_id),
                predicate::eq(payment_method.id),
                predicate::eq(gateway_charge.id),
            )
            .times(1)
            .return_const(Ok(charge.clone()));
        let mut mock_ui = MockUpdatingInvoice::new();
        mock_ui
            .expect_update()
            .with(
                predicate::eq(invoice_id),
                predicate::eq(InvoiceStatus::Pending),
            )
            .times(1)
            .return_const(Ok(invoice.clone()));
        let charge_usecase = ChargeCreateUsecase {
            reading_invoice: Box::new(mock_ri),
            reading_payment_method: Box::new(mock_rpm),
            creating_charge: Box::new(mock_cc),
            updating_invoice: Box::new(mock_ui),
            reading_gateway: Box::new(mock_rg),
            reading_invoice_item: Box::new(mock_rii),
        };
        let result = charge_usecase.create_charge(&invoice_id.to_string()).await;

        match result {
            Ok(resp) => {
                assert_eq!(charge.id, resp.id);
                assert_eq!(invoice_id, resp.invoice_id);
                assert_eq!(payment_method.id, resp.payment_method_id);
                assert_eq!(ChargeStatus::Progress.to_string(), resp.status.to_string());
            }
            Err(error) => panic!("should_succeed_creating_charge test went wrong: {}", error),
        }
    }

    #[actix_rt::test]
    async fn should_fail_when_uuid_is_wrong() {
        let mut mock_ri = MockReadingInvoice::new();
        mock_ri.expect_get_by_id().times(0);
        let mut mock_rpm = MockReadingPaymentMethod::new();
        mock_rpm.expect_get_default_by_user_id().times(0);
        let mut mock_cc = MockCreatingCharge::new();
        mock_cc.expect_create_charge().times(0);
        let mut mock_ui = MockUpdatingInvoice::new();
        mock_ui.expect_update().times(0);
        let mut mock_rg: MockReadingGateway = MockReadingGateway::new();
        mock_rg.expect_get_priority_list().times(0);
        let mut mock_rii = MockReadingInvoiceItem::new();
        mock_rii.expect_get_sum_by_invoice_id().times(0);
        let charge_usecase = ChargeCreateUsecase {
            reading_invoice: Box::new(mock_ri),
            reading_payment_method: Box::new(mock_rpm),
            creating_charge: Box::new(mock_cc),
            updating_invoice: Box::new(mock_ui),
            reading_gateway: Box::new(mock_rg),
            reading_invoice_item: Box::new(mock_rii),
        };
        let result = charge_usecase.create_charge("any-hash-not-uuid").await;

        match result {
            Ok(_) => panic!("should_fail_when_uuid_is_wrong test went wrong"),
            Err(error) => {
                assert_eq!(error, String::from("external id provided isn't uuid"));
            }
        }
    }

    #[actix_rt::test]
    async fn should_fail_when_invoice_provider_went_wrong() {
        let invoice_id: Uuid = UUIDv4.fake();

        let mut mock_ri = MockReadingInvoice::new();
        mock_ri
            .expect_get_by_id()
            .with(predicate::eq(invoice_id))
            .times(1)
            .return_const(Err(ReadingInvoiceError::UnmappedError));
        let mut mock_rpm = MockReadingPaymentMethod::new();
        mock_rpm.expect_get_default_by_user_id().times(0);
        let mut mock_rg: MockReadingGateway = MockReadingGateway::new();
        mock_rg.expect_get_priority_list().times(0);
        let mut mock_rii = MockReadingInvoiceItem::new();
        mock_rii.expect_get_sum_by_invoice_id().times(0);
        let mut mock_cc = MockCreatingCharge::new();
        mock_cc.expect_create_charge().times(0);
        let mut mock_ui = MockUpdatingInvoice::new();
        mock_ui.expect_update().times(0);
        let charge_usecase = ChargeCreateUsecase {
            reading_invoice: Box::new(mock_ri),
            reading_payment_method: Box::new(mock_rpm),
            creating_charge: Box::new(mock_cc),
            updating_invoice: Box::new(mock_ui),
            reading_gateway: Box::new(mock_rg),
            reading_invoice_item: Box::new(mock_rii),
        };
        let result = charge_usecase.create_charge(&invoice_id.to_string()).await;

        match result {
            Ok(_) => panic!("should_fail_when_invoice_provider_went_wrong test went wrong"),
            Err(error) => {
                assert_eq!(
                    error,
                    String::from("ReadingInvoiceError::UnmappedError Something wrong happen")
                );
            }
        }
    }

    #[actix_rt::test]
    async fn should_fail_when_payment_method_provider_went_wrong() {
        let now = time::OffsetDateTime::now_utc().to_string();
        let user_id: Uuid = UUIDv4.fake();
        let invoice_id: Uuid = UUIDv4.fake();
        let invoice = Invoice {
            id: invoice_id,
            user_id,
            status: InvoiceStatus::Draft,
            created_at: now,
        };
        let mut mock_ri = MockReadingInvoice::new();
        mock_ri
            .expect_get_by_id()
            .with(predicate::eq(invoice_id))
            .times(1)
            .return_const(Ok(invoice.clone()));
        let mut mock_rpm = MockReadingPaymentMethod::new();
        mock_rpm
            .expect_get_default_by_user_id()
            .with(predicate::eq(user_id))
            .times(1)
            .return_const(Err(ReadingPaymentMethodError::UnmappedError));
        let mut mock_cc = MockCreatingCharge::new();
        mock_cc.expect_create_charge().times(0);
        let mut mock_ui = MockUpdatingInvoice::new();
        mock_ui.expect_update().times(0);
        let mut mock_rg: MockReadingGateway = MockReadingGateway::new();
        mock_rg.expect_get_priority_list().times(0);
        let mut mock_rii = MockReadingInvoiceItem::new();
        mock_rii.expect_get_sum_by_invoice_id().times(0);
        let charge_usecase = ChargeCreateUsecase {
            reading_invoice: Box::new(mock_ri),
            reading_payment_method: Box::new(mock_rpm),
            creating_charge: Box::new(mock_cc),
            updating_invoice: Box::new(mock_ui),
            reading_gateway: Box::new(mock_rg),
            reading_invoice_item: Box::new(mock_rii),
        };
        let result = charge_usecase.create_charge(&invoice_id.to_string()).await;

        match result {
            Ok(_) => panic!("should_fail_when_payment_method_provider_went_wrong test went wrong"),
            Err(error) => {
                assert_eq!(
                    error,
                    String::from("ReadingPaymentMethodError::UnmappedError Something wrong happen")
                );
            }
        }
    }

    #[actix_rt::test]
    async fn should_fail_when_charge_provider_went_wrong() {
        let now = time::OffsetDateTime::now_utc().to_string();
        let user_id: Uuid = UUIDv4.fake();
        let invoice_id: Uuid = UUIDv4.fake();
        let payment_method_id: Uuid = UUIDv4.fake();
        let sum = dec!(100.0);
        let gateway_charge = GatewayCharge {
            id: UUIDv4.fake(),
            amount: sum.to_f32().unwrap(),
            currency: String::from("USD"),
        };
        let pix_info = PixInfo {
            key: String::from("any@email.com"),
            external_id: String::from("ABCDEFG"),
        };
        let info = PaymentMethodInfo::PixInfo(pix_info);
        let invoice = Invoice {
            id: invoice_id,
            user_id,
            status: InvoiceStatus::Draft,
            created_at: now,
        };
        let payment_method = PaymentMethod {
            id: payment_method_id,
            user_id,
            is_default: true,
            method: Method::Pix,
            info,
            created_at: 123_456_789_012,
        };
        let mut mock_ri = MockReadingInvoice::new();
        mock_ri
            .expect_get_by_id()
            .with(predicate::eq(invoice_id))
            .times(1)
            .return_const(Ok(invoice.clone()));
        let mut mock_rpm = MockReadingPaymentMethod::new();
        mock_rpm
            .expect_get_default_by_user_id()
            .with(predicate::eq(user_id))
            .times(1)
            .return_const(Ok(payment_method.clone()));
        let mut mock_g = MockGatewayAPI::new();
        mock_g
            .expect_charge()
            .with(predicate::eq(sum), predicate::eq("auth-plus-billing"))
            .times(1)
            .return_const(Ok(gateway_charge.clone()));
        let mut mock_rii = MockReadingInvoiceItem::new();
        mock_rii
            .expect_get_sum_by_invoice_id()
            .return_const(Ok(sum));
        let mut mock_cc = MockCreatingCharge::new();
        mock_cc
            .expect_create_charge()
            .with(
                predicate::eq(invoice_id),
                predicate::eq(payment_method.id),
                predicate::eq(gateway_charge.id),
            )
            .times(1)
            .return_const(Err(CreatingChargeError::UnmappedError));

        let mut mock_rg: MockReadingGateway = MockReadingGateway::new();
        mock_rg
            .expect_get_priority_list()
            .with()
            .times(1)
            .return_once(move || {
                let boxed_gateway: Box<dyn GatewayAPI + Send> = Box::new(mock_g);
                Ok(boxed_gateway)
            });
        let mut mock_ui = MockUpdatingInvoice::new();
        mock_ui.expect_update().times(0);
        let charge_usecase = ChargeCreateUsecase {
            reading_invoice: Box::new(mock_ri),
            reading_payment_method: Box::new(mock_rpm),
            creating_charge: Box::new(mock_cc),
            updating_invoice: Box::new(mock_ui),
            reading_gateway: Box::new(mock_rg),
            reading_invoice_item: Box::new(mock_rii),
        };
        let result = charge_usecase.create_charge(&invoice_id.to_string()).await;

        match result {
            Ok(_) => panic!("should_fail_when_charge_provider_went_wrong test went wrong"),
            Err(error) => {
                assert_eq!(
                    error,
                    String::from("CreatingChargeError::UnmappedError Something wrong happen")
                );
            }
        }
    }

    #[actix_rt::test]
    async fn should_fail_when_invoice_updating_provider_went_wrong() {
        let now = time::OffsetDateTime::now_utc().to_string();
        let user_id: Uuid = UUIDv4.fake();
        let invoice_id: Uuid = UUIDv4.fake();
        let payment_method_id: Uuid = UUIDv4.fake();
        let charge_id: Uuid = UUIDv4.fake();
        let sum = dec!(100.0);
        let gateway_charge = GatewayCharge {
            id: UUIDv4.fake(),
            amount: sum.to_f32().unwrap(),
            currency: String::from("USD"),
        };
        let pix_info = PixInfo {
            key: String::from("any@email.com"),
            external_id: String::from("ABCDEFG"),
        };
        let info = PaymentMethodInfo::PixInfo(pix_info);
        let invoice = Invoice {
            id: invoice_id,
            user_id,
            status: InvoiceStatus::Draft,
            created_at: now,
        };
        let payment_method = PaymentMethod {
            id: payment_method_id,
            user_id,
            is_default: true,
            method: Method::Pix,
            info,
            created_at: 123_456_789_012,
        };
        let charge = Charge {
            id: charge_id,
            invoice_id,
            payment_method_id: payment_method.id,
            status: ChargeStatus::Progress,
        };
        let mut mock_ri = MockReadingInvoice::new();
        mock_ri
            .expect_get_by_id()
            .with(predicate::eq(invoice_id))
            .times(1)
            .return_const(Ok(invoice.clone()));
        let mut mock_rpm = MockReadingPaymentMethod::new();
        mock_rpm
            .expect_get_default_by_user_id()
            .with(predicate::eq(user_id))
            .times(1)
            .return_const(Ok(payment_method.clone()));
        let mut mock_g = MockGatewayAPI::new();
        mock_g
            .expect_charge()
            .with(predicate::eq(sum), predicate::eq("auth-plus-billing"))
            .times(1)
            .return_const(Ok(gateway_charge.clone()));
        let mut mock_rii = MockReadingInvoiceItem::new();
        mock_rii
            .expect_get_sum_by_invoice_id()
            .times(1)
            .return_const(Ok(sum));
        let mut mock_cc = MockCreatingCharge::new();
        mock_cc
            .expect_create_charge()
            .with(
                predicate::eq(invoice_id),
                predicate::eq(payment_method.id),
                predicate::eq(gateway_charge.id),
            )
            .times(1)
            .return_const(Ok(charge.clone()));
        let mut mock_ui = MockUpdatingInvoice::new();
        mock_ui
            .expect_update()
            .with(
                predicate::eq(invoice_id),
                predicate::eq(InvoiceStatus::Pending),
            )
            .return_const(Err(UpdatingInvoiceError::UnmappedError))
            .times(1);
        let mut mock_rg: MockReadingGateway = MockReadingGateway::new();
        mock_rg
            .expect_get_priority_list()
            .with()
            .times(1)
            .return_once(move || {
                let boxed_gateway: Box<dyn GatewayAPI + Send> = Box::new(mock_g);
                Ok(boxed_gateway)
            });
        let charge_usecase = ChargeCreateUsecase {
            reading_invoice: Box::new(mock_ri),
            reading_payment_method: Box::new(mock_rpm),
            creating_charge: Box::new(mock_cc),
            updating_invoice: Box::new(mock_ui),
            reading_gateway: Box::new(mock_rg),
            reading_invoice_item: Box::new(mock_rii),
        };
        let result = charge_usecase.create_charge(&invoice_id.to_string()).await;

        match result {
            Ok(_) => panic!("should_fail_when_charge_provider_went_wrong test went wrong"),
            Err(error) => {
                assert_eq!(
                    error,
                    String::from("UpdatingInvoiceError::UnmappedError Something wrong happen")
                );
            }
        }
    }
}
