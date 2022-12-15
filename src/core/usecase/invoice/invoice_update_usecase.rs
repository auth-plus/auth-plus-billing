use uuid::Uuid;

use crate::core::{
    dto::invoice::{Invoice, InvoiceStatus},
    usecase::driven::{reading_invoice::ReadingInvoice, updating_invoice::UpdatingInvoiceError},
    usecase::driven::{reading_invoice::ReadingInvoiceError, updating_invoice::UpdatingInvoice},
};

pub struct InvoiceUpdateUsecase {
    pub reading_invoice: Box<dyn ReadingInvoice>,
    pub updating_invoice: Box<dyn UpdatingInvoice>,
}

impl InvoiceUpdateUsecase {
    pub async fn update(
        &self,
        invoice_id_str: &str,
        new_status_str: &str,
    ) -> Result<Invoice, String> {
        let invoice_id = match Uuid::parse_str(invoice_id_str) {
            Ok(id) => id,
            Err(_error) => return Err(String::from("external id provided isn't uuid")),
        };
        let invoice_result = self.reading_invoice.get_by_id(invoice_id).await;
        let invoice = match invoice_result {
            Ok(invoice) => invoice,
            Err(error) => match error {
                ReadingInvoiceError::InvoiceNotFoundError => {
                    return Err(String::from("Invoice not found"))
                }
                ReadingInvoiceError::UnmappedError => {
                    return Err(String::from(
                        "ReadingInvoiceError::UnmappedError went wrong",
                    ))
                }
            },
        };
        let new_status = InvoiceStatus::from(new_status_str);
        if !Self::validate_update_status_change(invoice.status, new_status) {
            return Err(String::from("Updating to invalid status"));
        }
        let result_invoice_update = self.updating_invoice.update(invoice.id, new_status).await;
        match result_invoice_update {
            Ok(invoice) => Ok(invoice),
            Err(error) => match error {
                UpdatingInvoiceError::UnmappedError => Err(String::from(
                    "UpdatingInvoiceError::UnmappedError went wrong",
                )),
            },
        }
    }

    fn validate_update_status_change(old: InvoiceStatus, new: InvoiceStatus) -> bool {
        match old {
            InvoiceStatus::Draft => new == InvoiceStatus::Pending,
            InvoiceStatus::Pending => {
                new == InvoiceStatus::Paid || new == InvoiceStatus::ChargedWithError
            }
            InvoiceStatus::ChargedWithError => {
                new == InvoiceStatus::Paid || new == InvoiceStatus::Uncollectible
            }
            InvoiceStatus::Paid => {
                new == InvoiceStatus::Cancelled
                    || new == InvoiceStatus::Refunded
                    || new == InvoiceStatus::InProtest
            }
            InvoiceStatus::InProtest => new == InvoiceStatus::Chargeback,
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {

    use super::InvoiceUpdateUsecase;
    use crate::core::{
        dto::invoice::{Invoice, InvoiceStatus},
        usecase::driven::{
            reading_invoice::{MockReadingInvoice, ReadingInvoiceError},
            updating_invoice::{MockUpdatingInvoice, UpdatingInvoiceError},
        },
    };
    use fake::{uuid::UUIDv4, Fake};
    use mockall::predicate;
    use uuid::Uuid;

    #[actix_rt::test]
    async fn should_succeed_updating_invoice() {
        let user_id: Uuid = UUIDv4.fake();
        let invoice_id: Uuid = UUIDv4.fake();
        let invoice = Invoice {
            id: invoice_id,
            user_id,
            status: InvoiceStatus::Draft,
        };
        let updated_invoice = Invoice {
            id: invoice_id,
            user_id,
            status: InvoiceStatus::Pending,
        };
        let mut mock_ri = MockReadingInvoice::new();
        mock_ri
            .expect_get_by_id()
            .with(predicate::eq(invoice_id))
            .times(1)
            .return_const(Ok(invoice.clone()));
        let mut mock_ui = MockUpdatingInvoice::new();
        mock_ui
            .expect_update()
            .with(
                predicate::eq(invoice.id),
                predicate::eq(InvoiceStatus::Pending),
            )
            .times(1)
            .return_const(Ok(updated_invoice.clone()));
        let invoice_usecase = InvoiceUpdateUsecase {
            reading_invoice: Box::new(mock_ri),
            updating_invoice: Box::new(mock_ui),
        };
        let result = invoice_usecase
            .update(invoice_id.to_string().as_str(), "pending")
            .await;

        match result {
            Ok(resp) => {
                assert_eq!(resp.id, invoice_id);
                assert_eq!(resp.user_id, user_id);
                assert_eq!(resp.status, InvoiceStatus::Pending);
            }
            Err(error) => panic!(
                "should_succeed_updating_invoice test went wrong: {:?}",
                error
            ),
        }
    }

    #[actix_rt::test]
    async fn should_fail_when_uuid_is_wrong() {
        let mut mock_ri = MockReadingInvoice::new();
        mock_ri.expect_get_by_id().times(0);
        let mut mock_ui = MockUpdatingInvoice::new();
        mock_ui.expect_update().times(0);
        let invoice_usecase = InvoiceUpdateUsecase {
            reading_invoice: Box::new(mock_ri),
            updating_invoice: Box::new(mock_ui),
        };

        let result = invoice_usecase.update("any-hash-not-uuid", "pending").await;

        match result {
            Ok(_) => panic!("should_fail_when_uuid_is_wrong test went wrong"),
            Err(error) => assert_eq!(error, String::from("external id provided isn't uuid")),
        }
    }

    #[actix_rt::test]
    async fn should_fail_when_invoice_reading_provider_went_wrong() {
        let invoice_id: Uuid = UUIDv4.fake();
        let mut mock_ri = MockReadingInvoice::new();
        mock_ri
            .expect_get_by_id()
            .with(predicate::eq(invoice_id))
            .times(1)
            .return_const(Err(ReadingInvoiceError::InvoiceNotFoundError));
        let mut mock_ui = MockUpdatingInvoice::new();
        mock_ui.expect_update().times(0);
        let invoice_usecase = InvoiceUpdateUsecase {
            reading_invoice: Box::new(mock_ri),
            updating_invoice: Box::new(mock_ui),
        };
        let result = invoice_usecase
            .update(invoice_id.to_string().as_str(), "pending")
            .await;

        match result {
            Ok(_) => panic!("should_fail_when_invoice_reading_provider_went_wrong test went wrong"),
            Err(error) => assert_eq!(error, String::from("Invoice not found")),
        }
    }

    #[actix_rt::test]
    async fn should_fail_when_invoice_updating_provider_went_wrong() {
        let user_id: Uuid = UUIDv4.fake();
        let invoice_id: Uuid = UUIDv4.fake();
        let invoice = Invoice {
            id: invoice_id,
            user_id,
            status: InvoiceStatus::Draft,
        };
        let mut mock_ri = MockReadingInvoice::new();
        mock_ri
            .expect_get_by_id()
            .with(predicate::eq(invoice_id))
            .times(1)
            .return_const(Ok(invoice.clone()));
        let mut mock_ui = MockUpdatingInvoice::new();
        mock_ui
            .expect_update()
            .with(
                predicate::eq(invoice.id),
                predicate::eq(InvoiceStatus::Pending),
            )
            .times(1)
            .return_const(Err(UpdatingInvoiceError::UnmappedError));
        let invoice_usecase = InvoiceUpdateUsecase {
            reading_invoice: Box::new(mock_ri),
            updating_invoice: Box::new(mock_ui),
        };
        let result = invoice_usecase
            .update(invoice_id.to_string().as_str(), "pending")
            .await;

        match result {
            Ok(_) => {
                panic!("should_fail_when_invoice_updating_provider_went_wrong test went wrong")
            }
            Err(error) => assert_eq!(
                error,
                String::from("UpdatingInvoiceError::UnmappedError went wrong")
            ),
        }
    }

    #[actix_rt::test]
    async fn should_succeed_when_validating_status_update() {
        let old_list = Vec::from([
            InvoiceStatus::Draft,
            InvoiceStatus::Pending,
            InvoiceStatus::ChargedWithError,
            InvoiceStatus::Paid,
            InvoiceStatus::Cancelled,
            InvoiceStatus::Uncollectible,
            InvoiceStatus::Refunded,
            InvoiceStatus::InProtest,
            InvoiceStatus::Chargeback,
            InvoiceStatus::UnmappedStatus,
        ]);
        let new_list = old_list.clone();
        let result_expect = Vec::from([
            // Draft
            [
                false, true, false, false, false, false, false, false, false, false,
            ],
            // Pending
            [
                false, false, true, true, false, false, false, false, false, false,
            ],
            // ChargedWithError
            [
                false, false, false, true, false, true, false, false, false, false,
            ],
            // Paid
            [
                false, false, false, false, true, false, true, true, false, false,
            ],
            // Cancelled
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
            // Uncollectible
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
            // Refunded
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
            // InProtest
            [
                false, false, false, false, false, false, false, false, true, false,
            ],
            // Chargeback
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
            // UnmappedStatus
            [
                false, false, false, false, false, false, false, false, false, false,
            ],
        ]);

        for idx in 0..old_list.len() {
            // let mut list: Vec<bool> = Vec::new();
            // for new in new_list {
            //     let r = InvoiceUpdateUsecase::validate_update_status_change(old, new);
            //     list.push(r)
            // }
            let old = old_list[idx];
            let list: Vec<bool> = new_list
                .clone()
                .into_iter()
                .map(|new| InvoiceUpdateUsecase::validate_update_status_change(old, new))
                .collect();

            assert_eq!(list, result_expect[idx]);
        }
    }
}
