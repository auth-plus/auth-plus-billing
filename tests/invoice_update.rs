mod invoice_update_tests {
    use actix_web::{http::StatusCode, test, web, App};
    use auth_plus_billing::{
        config::database::get_connection,
        core::{
            dto::invoice::InvoiceStatus,
            repository::orm::{create_invoice, create_user, delete_invoice, delete_user},
        },
        presentation::http::routes::invoice::{self, UpdateInvoiceInputSchema},
    };
    use fake::{uuid::UUIDv4, Fake};
    use uuid::Uuid;

    #[actix_web::test]
    async fn should_update_an_invoice() {
        let conn = get_connection().await;
        let user_id: Uuid = UUIDv4.fake();
        let invoice_id: Uuid = UUIDv4.fake();
        let external_id: Uuid = UUIDv4.fake();
        create_user(&conn, user_id, external_id)
            .await
            .expect("should_update_an_invoice: user setup went wrong");
        create_invoice(&conn, invoice_id, user_id, InvoiceStatus::Paid)
            .await
            .expect("should_update_an_invoice: invoice setup went wrong");

        let payload = UpdateInvoiceInputSchema {
            invoice_id: invoice_id.to_string(),
            status: InvoiceStatus::Refunded.to_string(),
        };
        let app = test::init_service(App::new().service(invoice::update_invoice)).await;
        let req = test::TestRequest::patch()
            .uri("/invoice")
            .set_json(web::Json(payload))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        delete_invoice(&conn, invoice_id)
            .await
            .expect("should_update_an_invoice: invoice remove went wrong");
        delete_user(&conn, user_id)
            .await
            .expect("should_update_an_invoice: user remove went wrong");
    }

    #[actix_web::test]
    async fn should_fail_when_user_does_not_exist() {
        let invoice_id: Uuid = UUIDv4.fake();
        let payload = UpdateInvoiceInputSchema {
            invoice_id: invoice_id.to_string(),
            status: InvoiceStatus::Refunded.to_string(),
        };
        let app = test::init_service(App::new().service(invoice::update_invoice)).await;
        let req = test::TestRequest::patch()
            .uri("/invoice")
            .set_json(web::Json(payload))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
