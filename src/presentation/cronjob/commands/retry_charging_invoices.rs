use crate::core::get_core;

pub async fn retry_charging_invoices() -> Result<(), String> {
    let core_x = get_core().await;
    let invoices = core_x
        .invoice
        .list
        .get_all_invoices_to_charge()
        .await
        .expect("Selecting invoice with error went wrong");
    let mut error_list = Vec::new();
    for inv in invoices {
        let result = core_x
            .charge
            .create
            .create_charge(&inv.id.to_string())
            .await;
        match result {
            Ok(_) => (),
            Err(error) => error_list.push(error),
        }
    }
    if !error_list.is_empty() {
        let all_errors = error_list.join(";");
        return Err(all_errors);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use fake::{Fake, faker::lorem::en::Word, uuid::UUIDv4};
    use uuid::Uuid;

    use crate::{
        config::database::get_connection,
        core::{
            dto::{
                charge::ChargeStatus,
                invoice::InvoiceStatus,
                payment_method::{CreditCardInfo, Method, PaymentMethodInfo},
            },
            repository::{
                charge_repository::ChargeDAO,
                invoice_repository::InvoiceDAO,
                orm::{
                    create_gateway, create_invoice, create_payment_method, create_user,
                    delete_charge, delete_gateway, delete_invoice, delete_payment_method,
                    delete_user,
                },
            },
        },
    };

    use super::retry_charging_invoices;

    #[actix_rt::test]
    async fn should_succeed_creating_charge() {
        let conn = get_connection().await;
        let user_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        let invoice_id: Uuid = UUIDv4.fake();
        let gateway_id: Uuid = UUIDv4.fake();
        let gateway_name: String = Word().fake();
        let payment_method_id: Uuid = UUIDv4.fake();
        let method = Method::CreditCard;
        let cc_info = CreditCardInfo {
            last4digit: String::from("1234"),
            flag: String::from("visa"),
            expire_data: String::from("01/01/1970"),
            external_id: String::from("ABCDEFG"),
        };
        let charge_id: Uuid;
        let info = PaymentMethodInfo::CreditCardInfo(cc_info);
        create_user(&conn, user_id, external_id)
            .await
            .expect("should_succeed_creating_charge: user setup went wrong");
        create_invoice(&conn, invoice_id, user_id, InvoiceStatus::Pending)
            .await
            .expect("should_succeed_creating_charge: invoice setup went wrong");
        create_gateway(&conn, gateway_id, &gateway_name, 1)
            .await
            .expect("should_succeed_creating_charge: gateway setup went wrong");
        create_payment_method(&conn, payment_method_id, user_id, true, method, info)
            .await
            .expect("should_succeed_creating_charge: payment_method setup went wrong");
        let result = retry_charging_invoices().await;
        match result {
            Ok(_) => {
                let q_invoice =
                    sqlx::query_as::<_, InvoiceDAO>("SELECT * FROM invoice WHERE  id :: text = $1")
                        .bind(invoice_id.to_string())
                        .fetch_one(&conn)
                        .await;
                match q_invoice {
                    Ok(inv) => {
                        assert_eq!(inv.user_id.to_string(), user_id.to_string());
                        assert_eq!(inv.status.to_string(), InvoiceStatus::Pending.to_string());
                    }
                    Err(error) => {
                        panic!("should_succeed_creating_charge test went wrong {:?}", error)
                    }
                };
                let q_charge = sqlx::query_as::<_, ChargeDAO>(
                    "SELECT * FROM charge WHERE invoice_id :: text = $1",
                )
                .bind(invoice_id.to_string())
                .fetch_all(&conn)
                .await;
                match q_charge {
                    Ok(list) => {
                        assert_eq!(list.len(), 1);
                        assert_eq!(
                            list[0].status.to_string(),
                            ChargeStatus::Progress.to_string()
                        );
                        charge_id = list[0].id.unwrap();
                    }
                    Err(error) => {
                        panic!("should_succeed_creating_charge test went wrong {:?}", error)
                    }
                };
            }
            Err(error) => panic!("should_succeed_creating_charge test went wrong {:?}", error),
        }
        delete_charge(&conn, charge_id)
            .await
            .expect("should_succeed_creating_charge: gateway remove went wrong");
        delete_payment_method(&conn, payment_method_id)
            .await
            .expect("should_succeed_creating_charge: gateway remove went wrong");
        delete_gateway(&conn, gateway_id)
            .await
            .expect("should_succeed_creating_charge: gateway remove went wrong");
        delete_invoice(&conn, invoice_id)
            .await
            .expect("should_succeed_creating_charge: invoice remove went wrong");
        delete_user(&conn, user_id)
            .await
            .expect("should_succeed_creating_charge: user remove went wrong");
    }
}
