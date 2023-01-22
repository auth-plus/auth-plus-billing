use uuid::Uuid;

use crate::core::{
    dto::charge::Charge,
    usecase::driven::{
        creating_charge::{CreatingCharge, CreatingChargeError},
        reading_invoice::{ReadingInvoice, ReadingInvoiceError},
        reading_payment_method::{ReadingPaymentMethod, ReadingPaymentMethodError},
    },
};

pub struct ChargeCreateUsecase {
    pub reading_invoice: Box<dyn ReadingInvoice>,
    pub reading_payment_method: Box<dyn ReadingPaymentMethod>,
    pub creating_charge: Box<dyn CreatingCharge>,
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
                    return Err(String::from("invoice not found"))
                }
                ReadingInvoiceError::UnmappedError => {
                    return Err(String::from(
                        "ReadingInvoiceError::UnmappedError Something wrong happen",
                    ))
                }
            },
        };
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
                    ))
                }
            },
        };
        let result_charge = self
            .creating_charge
            .create_charge(invoice.id, payment_method.id)
            .await;
        match result_charge {
            Ok(charge) => Ok(charge),
            Err(error) => match error {
                CreatingChargeError::KafkaProducerError => {
                    Err(String::from("Error on producing on kafka"))
                }
                CreatingChargeError::UnmappedError => Err(String::from(
                    "CreatingChargeError::UnmappedError Something wrong happen",
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
            reading_invoice::{MockReadingInvoice, ReadingInvoiceError},
            reading_payment_method::{MockReadingPaymentMethod, ReadingPaymentMethodError},
        },
    };
    use fake::{uuid::UUIDv4, Fake};
    use mockall::predicate;
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_succeed_creating_charge() {
        let now = chrono::offset::Utc::now().to_string();
        let user_id: Uuid = UUIDv4.fake();
        let invoice_id: Uuid = UUIDv4.fake();
        let payment_method_id: Uuid = UUIDv4.fake();
        let charge_id: Uuid = UUIDv4.fake();
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
        let mut mock_cc = MockCreatingCharge::new();
        mock_cc
            .expect_create_charge()
            .with(predicate::eq(invoice_id), predicate::eq(payment_method.id))
            .times(1)
            .return_const(Ok(charge.clone()));
        let charge_usecase = ChargeCreateUsecase {
            reading_invoice: Box::new(mock_ri),
            reading_payment_method: Box::new(mock_rpm),
            creating_charge: Box::new(mock_cc),
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
        let charge_usecase = ChargeCreateUsecase {
            reading_invoice: Box::new(mock_ri),
            reading_payment_method: Box::new(mock_rpm),
            creating_charge: Box::new(mock_cc),
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
        let mut mock_cc = MockCreatingCharge::new();
        mock_cc.expect_create_charge().times(0);
        let charge_usecase = ChargeCreateUsecase {
            reading_invoice: Box::new(mock_ri),
            reading_payment_method: Box::new(mock_rpm),
            creating_charge: Box::new(mock_cc),
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
        let now = chrono::offset::Utc::now().to_string();
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
        let charge_usecase = ChargeCreateUsecase {
            reading_invoice: Box::new(mock_ri),
            reading_payment_method: Box::new(mock_rpm),
            creating_charge: Box::new(mock_cc),
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
        let now = chrono::offset::Utc::now().to_string();
        let user_id: Uuid = UUIDv4.fake();
        let invoice_id: Uuid = UUIDv4.fake();
        let payment_method_id: Uuid = UUIDv4.fake();
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
        let mut mock_cc = MockCreatingCharge::new();
        mock_cc
            .expect_create_charge()
            .with(predicate::eq(invoice_id), predicate::eq(payment_method.id))
            .times(1)
            .return_const(Err(CreatingChargeError::UnmappedError));
        let charge_usecase = ChargeCreateUsecase {
            reading_invoice: Box::new(mock_ri),
            reading_payment_method: Box::new(mock_rpm),
            creating_charge: Box::new(mock_cc),
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
}
